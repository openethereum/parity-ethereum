// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read as _};
use std::path::PathBuf;

/// Current db version.
const CURRENT_VERSION: u8 = 4;
/// Database is assumed to be at the default version, when no version file is found.
const DEFAULT_VERSION: u8 = 3;
/// Version file name.
const VERSION_FILE_NAME: &str = "db_version";

/// Migration related errors.
#[derive(Debug)]
pub enum Error {
	/// Returned when current version cannot be read or guessed.
	UnknownDatabaseVersion,
	/// Existing DB is newer than the known one.
	FutureDBVersion,
	/// Migration using parity-ethereum 2.6.7 is required.
	MigrationWithLegacyVersionRequired,
	/// Migration was completed successfully,
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
			Error::MigrationWithLegacyVersionRequired =>
				"Secret Store database was created with an older client version.\
				To migrate, use parity-ethereum v2.6.7, then retry using the latest.".into(),
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

/// Apply all migrations if possible.
pub fn upgrade_db(db_path: &str) -> Result<(), Error> {
	match current_version(db_path)? {
		old_version if old_version < CURRENT_VERSION => {
			Err(Error::MigrationWithLegacyVersionRequired)
		},
		CURRENT_VERSION => Ok(()),
		_ => Err(Error::FutureDBVersion),
	}
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

