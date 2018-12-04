// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Storage writing utils

extern crate csv;
extern crate dir;
extern crate ethereum_types;
extern crate tempdir;
extern crate vm;

use dir::default_data_path;
use dir::helpers::replace_home;
use ethereum_types::{H256, Address};
use std::collections::HashMap;
use std::io;
use std::{fmt, str};
use std::path::PathBuf;


mod csv_storage_writer;
mod noop;


/// Something that can write storage values to disk.
pub trait StorageWriter: Send + Sync {
    /// Returns a copy of ourself, in a box.
    fn boxed_clone(&self) -> Box<StorageWriter>;

    /// Whether storage writing is enabled.
    fn enabled(&self) -> bool;

    /// Write storage diffs for modified accounts to disk
    fn write_storage_diffs(&mut self, header_hash: H256, header_number: u64, accounts_storage_changes: HashMap<Address, HashMap<H256, H256>>) -> io::Result<()>;
}

impl Clone for Box<StorageWriter> {
    fn clone(&self) -> Box<StorageWriter> {
        self.boxed_clone()
    }
}

/// Create a new `StorageWriter` trait object.
pub fn new(config: StorageWriterConfig) -> Box<StorageWriter> {
    match config.database {
        Database::Csv => Box::new(csv_storage_writer::CsvStorageWriter::new(config)),
        Database::None => Box::new(noop::NoopStorageWriter::new()),
    }
}


/// Storage writer database.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Database {
    /// CSV file
    Csv,
    /// No Storage Writer
    None,

}

impl str::FromStr for Database {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "csv" => Ok(Database::Csv),
            "none" => Ok(Database::None),
            e => Err(format!("Invalid storage writing database: {}", e)),
        }
    }
}

impl Database {
    /// Returns static str describing database.
    pub fn as_str(&self) -> &'static str {
        match *self {
            Database::Csv => "csv",
            Database::None => "none",
        }
    }

    /// Returns all algorithm types.
    pub fn all_types() -> Vec<Database> {
        vec![Database::Csv, Database::None]
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


/// Configuration for writing storage diffs from watched contracts
#[derive(PartialEq, Debug, Clone)]
pub struct StorageWriterConfig {
    /// Database used for persisting contract storage diffs.
    pub database: Database,
    /// Whether storage diff writing is enabled.
    pub enabled: bool,
    /// Where to locate database for storage diffs.
    pub path: PathBuf,
    /// Contracts to be watched.
    pub watched_contracts: Vec<Address>,
}

impl Default for StorageWriterConfig {
    fn default() -> Self {
        let data_dir = default_data_path();
        StorageWriterConfig {
            database: Database::None,
            enabled: false,
            path: replace_home(&data_dir, "$BASE/storage_diffs").into(),
            watched_contracts: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Database;

    #[test]
    fn test_storage_writer_database_parsing() {
        assert_eq!(Database::Csv, "csv".parse().unwrap());
        assert_eq!(Database::None, "none".parse().unwrap());
    }

    #[test]
    fn test_storage_writer_database_printing() {
        assert_eq!(Database::Csv.to_string(), "csv".to_owned());
        assert_eq!(Database::None.to_string(), "none".to_owned());
    }

    #[test]
    fn test_storage_writer_database_all_types() {
        // compiling should fail if some cases are not covered
        let mut csv = 0;
        let mut none = 0;

        for db in &Database::all_types() {
            match *db {
                Database::Csv => csv += 1,
                Database::None => none += 1,
            }
        }

        assert_eq!(csv, 1);
        assert_eq!(none, 1);
    }
}