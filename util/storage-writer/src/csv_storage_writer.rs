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

use std::collections::HashMap;
use std::io;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::{Database,StorageWriter,StorageWriterConfig};
use csv::Writer;
use ethereum_types::{Address, H256};

/// Implementation of `StorageWriter` trait for writing to CSV file.
#[derive(Clone)]
pub struct CsvStorageWriter {
    /// Path to CSV
    path: PathBuf,
    /// Contracts for which to write storage diffs
    watched_contracts: Vec<Address>,
    /// File writing connection to CSV
    writer: Arc<Mutex<Writer<File>>>
}

impl CsvStorageWriter {
    pub fn new(config: StorageWriterConfig) -> CsvStorageWriter {
        let path = config.path;
        let file = path.join("storage_diffs.csv");

        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(file)
            .expect("Error creating csv file.");

        let wtr = csv::Writer::from_writer(file);

        CsvStorageWriter {
            path: path,
            watched_contracts: config.watched_contracts,
            writer: Arc::new(Mutex::new(wtr))
        }
    }

    fn write_storage_node(&mut self, contract: Address, block_hash: H256, block_number: u64, key: H256, value: H256) -> io::Result<()> {
        let mut wtr = self.writer.lock().unwrap();
        wtr.write_record(&[format!("{:x}", contract), format!("{:x}", block_hash), format!("{}", block_number), format!("{:x}", key), format!("{:x}", value)])?;
        wtr.flush()?;
        Ok(())
    }

    fn watching_all_diffs(&self) -> bool {
        self.watched_contracts.len() == 0
    }
}

impl StorageWriter for CsvStorageWriter {
    fn boxed_clone(&self) -> Box<StorageWriter> {
        let config = StorageWriterConfig {
            database: Database::Csv,
            enabled: true,
            path: self.path.clone(),
            watched_contracts: self.watched_contracts.to_vec(),
        };
        Box::new(CsvStorageWriter::new(config))
    }

    fn enabled(&self) -> bool {
        true
    }

    fn write_storage_diffs(&mut self, header_hash: H256, header_number: u64, accounts_storage_diffs: HashMap<Address, HashMap<H256, H256>>) -> io::Result<()> {
        for (addr, diffs) in accounts_storage_diffs {
            if self.watching_all_diffs() || self.watched_contracts.contains(&addr) {
                for (k, v) in diffs {
                    self.write_storage_node(addr, header_hash, header_number, k, v)?;
                }
            }
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use ethereum_types::{clean_0x, Address, H256};
    use tempdir::TempDir;
    use super::{CsvStorageWriter,Database,StorageWriter,StorageWriterConfig};

    #[test]
    fn test_enabled() {
        let tempdir = TempDir::new("temp_storage_csv").unwrap();
        let config = StorageWriterConfig {
            database: Database::Csv,
            enabled: true,
            path: tempdir.path().into(),
            watched_contracts: vec![],
        };
        let storage_writer = CsvStorageWriter::new(config);

        assert!(storage_writer.enabled())
    }

    #[test]
    fn test_writes_watched_diff() {
        // setup storage writer with watched contract specified
        let tempdir = TempDir::new("temp_storage_csv").unwrap();
        let file_path = tempdir.path().join("storage_diffs.csv");
        let watched_contract : Address = clean_0x("0xdeadbeefcafe0000000000000000000000000000").parse().unwrap();
        let watched_contracts = vec![watched_contract];
        let config = StorageWriterConfig {
            database: Database::Csv,
            enabled: true,
            path: tempdir.path().into(),
            watched_contracts: watched_contracts,
        };
        let mut storage_writer = CsvStorageWriter::new(config);

        // setup args for writing storage diffs
        let watched_contract_storage_key = H256::from("0000000000000000000000000000000000000000000000000000000000000001");
        let watched_contract_storage_value = H256::from("0000000000000000000000000000000000000000000000000000000000000002");
        let watched_contract_storage_diff : HashMap<H256, H256> = [(watched_contract_storage_key, watched_contract_storage_value)].iter().cloned().collect();
        let unwatched_contract : Address = clean_0x("0x123456789abc0000000000000000000000000000").parse().unwrap();
        let unwatched_contract_storage_key = H256::from("0000000000000000000000000000000000000000000000000000000000000003");
        let unwatched_contract_storage_value = H256::from("0000000000000000000000000000000000000000000000000000000000000004");
        let unwatched_contract_storage_diff : HashMap<H256, H256> = [(unwatched_contract_storage_key, unwatched_contract_storage_value)].iter().cloned().collect();
        let accounts_storage_diffs : HashMap<Address, HashMap<H256, H256>> =
            [(unwatched_contract, unwatched_contract_storage_diff),
                (watched_contract, watched_contract_storage_diff)]
                .iter().cloned().collect();
        let header_hash = H256::from("0xa3c565fc15c7478862d50ccd6561e3c06b24cc509bf388941c25ea985ce32cb9");

        // execute storage writer
        let _ = storage_writer.write_storage_diffs(header_hash, 0, accounts_storage_diffs);

        // verify only watched contract storage diffs written
        let file = fs::OpenOptions::new()
            .read(true)
            .open(file_path.clone())
            .expect("Error opening temp csv file.");
        let mut rdr = csv::Reader::from_reader(file);
        let expected_record = csv::StringRecord::from(vec![format!("{:x}", watched_contract), format!("{:x}", header_hash), format!("{}", 0), format!("{:x}", watched_contract_storage_key), format!("{:x}", watched_contract_storage_value)]);
        for result in rdr.records() {
            match result {
                Ok(record) => assert_eq!(record, expected_record),
                Err(_err) => panic!("Unexpected record in storage diffs"),
            }
        }
    }

    #[test]
    fn test_writes_all_diffs_if_watched_contracts_not_specified() {
        // setup storage writer with no watched contracts specified
        let tempdir = TempDir::new("temp_storage_csv").unwrap();
        let file_path = tempdir.path().join("storage_diffs.csv");
        let config = StorageWriterConfig {
            database: Database::Csv,
            enabled: true,
            path: tempdir.path().into(),
            watched_contracts: vec![],
        };
        let mut storage_writer = CsvStorageWriter::new(config);

        // setup args for writing storage diffs
        let contract : Address = clean_0x("0xdeadbeefcafe0000000000000000000000000000").parse().unwrap();
        let contract_storage_key = H256::from("0000000000000000000000000000000000000000000000000000000000000003");
        let contract_storage_value = H256::from("0000000000000000000000000000000000000000000000000000000000000004");
        let contract_storage_diff : HashMap<H256, H256> = [(contract_storage_key, contract_storage_value)].iter().cloned().collect();
        let accounts_storage_diffs : HashMap<Address, HashMap<H256, H256>> = [(contract, contract_storage_diff)].iter().cloned().collect();
        let header_hash = H256::from("0xa3c565fc15c7478862d50ccd6561e3c06b24cc509bf388941c25ea985ce32cb9");

        // execute storage writer
        let _ = storage_writer.write_storage_diffs(header_hash, 0, accounts_storage_diffs);

        // verify storage diffs written
        let file = fs::OpenOptions::new()
            .read(true)
            .open(file_path.clone())
            .expect("Error opening temp csv file.");
        let mut rdr = csv::Reader::from_reader(file);
        let expected_record = csv::StringRecord::from(vec![format!("{:x}", contract), format!("{:x}", header_hash), format!("{}", 0), format!("{:x}", contract_storage_key), format!("{:x}", contract_storage_value)]);
        for result in rdr.records() {
            match result {
                Ok(record) => assert_eq!(record, expected_record),
                Err(_err) => panic!("Unexpected record in storage diffs"),
            }
        }
    }
}