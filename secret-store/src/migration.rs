// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Secret Store DB migration module.


use std::fmt::{Display, Error as FmtError, Formatter};
use std::fs;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read as _, Write as _};
use std::path::PathBuf;

use kvdb::DBTransaction;
use kvdb_rocksdb::{Database, DatabaseConfig};

/// We used to store the version in the database (until version 4).
const LEGACY_DB_META_KEY_VERSION: &[u8; 7] = b"version";
/// Current db version.
const CURRENT_VERSION: u8 = 4;
/// Database is assumed to be at the default version, when no version file is found.
const DEFAULT_VERSION: u8 = 3;
/// Version file name.
const VERSION_FILE_NAME: &str = "db_version";

/// Migration related erorrs.
#[derive(Debug)]
pub enum Error {
	/// Returned when current version cannot be read or guessed.
	UnknownDatabaseVersion,
	/// Existing DB is newer than the known one.
	FutureDBVersion,
	/// Migration was completed succesfully,
	/// but there was a problem with io.
	Io(IoError),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		let out = match *self {
			Error::UnknownDatabaseVersion =>
				"Current Secret Store database version cannot be read".into(),
			Error::FutureDBVersion =>
				"Secret Store database was created with newer client version.\
				Upgrade your client or delete DB and resync.".into(),
			Error::Io(ref err) =>
				format!("Unexpected io error on Secret Store database migration: {}.", err),
		};
		write!(f, "{}", out)
	}
}

impl From<IoError> for Error {
	fn from(err: IoError) -> Self {
		Error::Io(err)
	}
}

// Moves "default" column to column 0 in preparation for a kvdb-rocksdb 0.3 migration.
fn migrate_to_v4(parent_dir: &str) -> Result<(), Error> {
	// Naïve implementation until
	// https://github.com/facebook/rocksdb/issues/6130 is resolved
	let old_db_config = DatabaseConfig::with_columns(Some(1));
	let new_db_config = DatabaseConfig::with_columns(Some(1));
	const BATCH_SIZE: usize = 1024;

	let old_dir = db_dir(parent_dir);
	let new_dir = migration_dir(parent_dir);
	let old_db = Database::open(&old_db_config, &old_dir)?;
	let new_db = Database::open(&new_db_config, &new_dir)?;

	const OLD_COLUMN: Option<u32> = None;
	const NEW_COLUMN: Option<u32> = Some(0);

	// remove legacy version key
	{
		let mut batch = DBTransaction::with_capacity(1);
		batch.delete(OLD_COLUMN, LEGACY_DB_META_KEY_VERSION);
		if let Err(err) = old_db.write(batch) {
			error!(target: "migration", "Failed to delete db version {}", &err);
			return Err(err.into());
		}
	}

	let mut batch = DBTransaction::with_capacity(BATCH_SIZE);
	for (i, (key, value)) in old_db.iter(OLD_COLUMN).enumerate() {
		batch.put(NEW_COLUMN, &key, &value);
		if i % BATCH_SIZE == 0 {
			new_db.write(batch)?;
			batch = DBTransaction::with_capacity(BATCH_SIZE);
			info!(target: "migration", "Migrating Secret Store DB: {} keys written", i);
		}
	}
	new_db.write(batch)?;
	drop(new_db);
	old_db.restore(&new_dir)?;

	info!(target: "migration", "Secret Store migration finished");

	Ok(())
}

/// Apply all migrations if possible.
pub fn upgrade_db(db_path: &str) -> Result<(), Error> {
	match current_version(db_path)? {
		old_version if old_version < CURRENT_VERSION => {
			migrate_to_v4(db_path)?;
			update_version(db_path)?;
			Ok(())
		},
		CURRENT_VERSION => Ok(()),
		_ => Err(Error::FutureDBVersion),
	}
}

fn db_dir(path: &str) -> String {
	let mut dir = PathBuf::from(path);
	dir.push("db");
	dir.to_string_lossy().to_string()
}

fn migration_dir(path: &str) -> String {
	let mut dir = PathBuf::from(path);
	dir.push("migration");
	dir.to_string_lossy().to_string()
}

/// Returns the version file path.
fn version_file_path(path: &str) -> PathBuf {
	let mut file_path = PathBuf::from(path);
	file_path.push(VERSION_FILE_NAME);
	file_path
}

/// Reads current database version from the file at given path.
/// If the file does not exist returns `DEFAULT_VERSION`.
fn current_version(path: &str) -> Result<u8, Error> {
	match fs::File::open(version_file_path(path)) {
		Err(ref err) if err.kind() == IoErrorKind::NotFound => Ok(DEFAULT_VERSION),
		Err(err) => Err(err.into()),
		Ok(mut file) => {
			let mut s = String::new();
			file.read_to_string(&mut s)?;
			u8::from_str_radix(&s, 10).map_err(|_| Error::UnknownDatabaseVersion)
		},
	}
}

/// Writes current database version to the file.
/// Creates a new file if the version file does not exist yet.
fn update_version(path: &str) -> Result<(), Error> {
	let mut file = fs::File::create(version_file_path(path))?;
	file.write_all(format!("{}", CURRENT_VERSION).as_bytes())?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempdir::TempDir;

	#[test]
	fn migration_works() -> Result<(), Error> {
		let parent = TempDir::new("secret_store_migration")?.into_path();

		let mut db_path = parent.clone();
		db_path.push("db");
		let db_path = db_path.to_str().unwrap();
		let parent_path = parent.to_str().unwrap();

		let old_db = Database::open(&DatabaseConfig::with_columns(None), db_path)?;

		let mut batch = old_db.transaction();
		batch.put(None, b"key1", b"value1");
		batch.put(None, b"key2", b"value2");
		old_db.write(batch)?;
		drop(old_db);

		upgrade_db(parent_path)?;
		let migrated = Database::open(&DatabaseConfig::with_columns(Some(1)), db_path)?;

		assert_eq!(migrated.get(Some(0), b"key1")?.expect("key1"), b"value1".to_vec());
		assert_eq!(migrated.get(Some(0), b"key2")?.expect("key2"), b"value2".to_vec());

		Ok(())
	}
}
