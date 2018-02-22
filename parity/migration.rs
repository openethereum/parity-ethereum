// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::fs;
use std::io::{Read, Write, Error as IoError, ErrorKind};
use std::path::{Path, PathBuf};
use std::fmt::{Display, Formatter, Error as FmtError};
use migr::{self, Manager as MigrationManager, Config as MigrationConfig};
use kvdb_rocksdb::CompactionProfile;
use migrations;

/// Database is assumed to be at default version, when no version file is found.
const DEFAULT_VERSION: u32 = 5;
/// Current version of database models.
const CURRENT_VERSION: u32 = 13;
/// First version of the consolidated database.
const CONSOLIDATION_VERSION: u32 = 9;
/// Defines how many items are migrated to the new version of database at once.
const BATCH_SIZE: usize = 1024;
/// Version file name.
const VERSION_FILE_NAME: &'static str = "db_version";

/// Migration related erorrs.
#[derive(Debug)]
pub enum Error {
	/// Returned when current version cannot be read or guessed.
	UnknownDatabaseVersion,
	/// Existing DB is newer than the known one.
	FutureDBVersion,
	/// Migration is not possible.
	MigrationImpossible,
	/// Internal migration error.
	Internal(migr::Error),
	/// Migration was completed succesfully,
	/// but there was a problem with io.
	Io(IoError),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		let out = match *self {
			Error::UnknownDatabaseVersion => "Current database version cannot be read".into(),
			Error::FutureDBVersion => "Database was created with newer client version. Upgrade your client or delete DB and resync.".into(),
			Error::MigrationImpossible => format!("Database migration to version {} is not possible.", CURRENT_VERSION),
			Error::Internal(ref err) => format!("{}", err),
			Error::Io(ref err) => format!("Unexpected io error on DB migration: {}.", err),
		};

		write!(f, "{}", out)
	}
}

impl From<IoError> for Error {
	fn from(err: IoError) -> Self {
		Error::Io(err)
	}
}

impl From<migr::Error> for Error {
	fn from(err: migr::Error) -> Self {
		match err.into() {
			migr::ErrorKind::Io(e) => Error::Io(e),
			err => Error::Internal(err.into()),
		}
	}
}

/// Returns the version file path.
fn version_file_path(path: &Path) -> PathBuf {
	let mut file_path = path.to_owned();
	file_path.push(VERSION_FILE_NAME);
	file_path
}

/// Reads current database version from the file at given path.
/// If the file does not exist returns `DEFAULT_VERSION`.
fn current_version(path: &Path) -> Result<u32, Error> {
	match fs::File::open(version_file_path(path)) {
		Err(ref err) if err.kind() == ErrorKind::NotFound => Ok(DEFAULT_VERSION),
		Err(_) => Err(Error::UnknownDatabaseVersion),
		Ok(mut file) => {
			let mut s = String::new();
			file.read_to_string(&mut s).map_err(|_| Error::UnknownDatabaseVersion)?;
			u32::from_str_radix(&s, 10).map_err(|_| Error::UnknownDatabaseVersion)
		},
	}
}

/// Writes current database version to the file.
/// Creates a new file if the version file does not exist yet.
fn update_version(path: &Path) -> Result<(), Error> {
	fs::create_dir_all(path)?;
	let mut file = fs::File::create(version_file_path(path))?;
	file.write_all(format!("{}", CURRENT_VERSION).as_bytes())?;
	Ok(())
}

/// Consolidated database path
fn consolidated_database_path(path: &Path) -> PathBuf {
	let mut state_path = path.to_owned();
	state_path.push("db");
	state_path
}

/// Database backup
fn backup_database_path(path: &Path) -> PathBuf {
	let mut backup_path = path.to_owned();
	backup_path.pop();
	backup_path.push("temp_backup");
	backup_path
}

/// Default migration settings.
pub fn default_migration_settings(compaction_profile: &CompactionProfile) -> MigrationConfig {
	MigrationConfig {
		batch_size: BATCH_SIZE,
		compaction_profile: *compaction_profile,
	}
}

/// Migrations on the consolidated database.
fn consolidated_database_migrations(compaction_profile: &CompactionProfile) -> Result<MigrationManager, Error> {
	let mut manager = MigrationManager::new(default_migration_settings(compaction_profile));
	manager.add_migration(migrations::TO_V11).map_err(|_| Error::MigrationImpossible)?;
	manager.add_migration(migrations::TO_V12).map_err(|_| Error::MigrationImpossible)?;
	manager.add_migration(migrations::ToV13::default()).map_err(|_| Error::MigrationImpossible)?;
	Ok(manager)
}

/// Migrates database at given position with given migration rules.
fn migrate_database(version: u32, db_path: PathBuf, mut migrations: MigrationManager) -> Result<(), Error> {
	// check if migration is needed
	if !migrations.is_needed(version) {
		return Ok(())
	}

	let backup_path = backup_database_path(&db_path);
	// remove the backup dir if it exists
	let _ = fs::remove_dir_all(&backup_path);

	// migrate old database to the new one
	let temp_path = migrations.execute(&db_path, version)?;

	// completely in-place migration leads to the paths being equal.
	// in that case, no need to shuffle directories.
	if temp_path == db_path { return Ok(()) }

	// create backup
	fs::rename(&db_path, &backup_path)?;

	// replace the old database with the new one
	if let Err(err) = fs::rename(&temp_path, &db_path) {
		// if something went wrong, bring back backup
		fs::rename(&backup_path, &db_path)?;
		return Err(err.into());
	}

	// remove backup
	fs::remove_dir_all(&backup_path).map_err(Into::into)
}

fn exists(path: &Path) -> bool {
	fs::metadata(path).is_ok()
}

/// Migrates the database.
pub fn migrate(path: &Path, compaction_profile: CompactionProfile) -> Result<(), Error> {
	// read version file.
	let version = current_version(path)?;

	// migrate the databases.
	// main db directory may already exists, so let's check if we have blocks dir
	if version > CURRENT_VERSION {
		return Err(Error::FutureDBVersion);
	}

	// We are in the latest version, yay!
	if version == CURRENT_VERSION {
		return Ok(())
	}

	// Further migrations
	if version >= CONSOLIDATION_VERSION && version < CURRENT_VERSION && exists(&consolidated_database_path(path)) {
		println!("Migrating database from version {} to {}", ::std::cmp::max(CONSOLIDATION_VERSION, version), CURRENT_VERSION);
		migrate_database(version, consolidated_database_path(path), consolidated_database_migrations(&compaction_profile)?)?;
		println!("Migration finished");
	}

	// update version file.
	update_version(path)
}
