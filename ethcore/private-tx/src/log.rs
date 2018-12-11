// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Private transactions logs.

use ethereum_types::{H256, Address};
use std::collections::{HashMap};
use std::fs::{File};
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::{RwLock};

#[derive(Clone, Serialize, Deserialize)]
enum Status {
	Created,
	Validating,
	Deployed,
}

#[derive(Clone, Serialize, Deserialize)]
struct ValidatorLog {
	account: Address,
	validated: bool,
	validation_timestamp: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
struct TransactionLog {
	tx_hash: H256,
	status: Status,
	creation_timestamp: u64,
	validators: Vec<ValidatorLog>,
	deployment_timestamp: Option<u64>,
	public_tx_hash: Option<H256>,
}

/// Private transactions logging
pub struct Logging {
	logs: RwLock<HashMap<H256, TransactionLog>>,
	logs_dir: Option<String>,
}

impl Logging {
	pub fn new(logs_dir: Option<String>) -> Self {
		let logging = Logging {
			logs: RwLock::new(HashMap::new()),
			logs_dir,
		};
		logging.read_logs();
		logging
	}

	pub fn private_tx_created(&self, tx_hash: H256, validators: &Vec<Address>) {
		let mut validator_logs = Vec::new();
		for account in validators {
			validator_logs.push(ValidatorLog {
				account: *account,
				validated: false,
				validation_timestamp: None,
			});
		}
		let mut logs = self.logs.write();
		logs.insert(tx_hash, TransactionLog {
			tx_hash,
			status: Status::Created,
			creation_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
			validators: validator_logs,
			deployment_timestamp: None,
			public_tx_hash: None,
		});
	}

	pub fn signature_added(&self, tx_hash: H256, validator: Address) {
		let mut logs = self.logs.write();
		if let Some(transaction_log) = logs.get_mut(&tx_hash) {
			transaction_log.status = Status::Validating;
			if let Some(ref mut validator_log) = transaction_log.validators.iter_mut().find(|log| log.account == validator) {
				validator_log.validated = true;
				validator_log.validation_timestamp = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
			}
		}
	}

	pub fn tx_deployed(&self, tx_hash: H256, public_tx_hash: H256) {
		let mut logs = self.logs.write();
		if let Some(log) = logs.get_mut(&tx_hash) {
			log.status = Status::Deployed;
			log.deployment_timestamp = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
			log.public_tx_hash = Some(public_tx_hash);
		}
	}

	fn read_logs(&self) {
		let log_file = match self.logs_dir {
			Some(ref path) => {
				let mut file_path = path.clone();
				file_path.push_str("private_tx.log");
				match File::open(&file_path) {
					Ok(file) => file,
					Err(err) => {
						trace!(target: "privatetx", "Cannot open logs file: {}", err);
						return;
					}
				}
			}
			None => {
				warn!(target: "privatetx", "Logs path is not defined");
				return;
			}
		};
		let transaction_logs: Vec<TransactionLog> = match serde_json::from_reader(log_file) {
			Ok(logs) => logs,
			Err(err) => {
				error!(target: "privatetx", "Cannot deserialize logs from file: {}", err);
				return;
			}
		};
		let mut logs = self.logs.write();
		for log in transaction_logs {
			logs.insert(log.tx_hash, log);
		}
	}

	fn flush_logs(&self) {
		if self.logs.read().is_empty() {
			// Do not create empty file
			return;
		}
		let log_file = match self.logs_dir {
			Some(ref path) => {
				let mut file_path = path.clone();
				file_path.push_str("private_tx.log");
				match File::open(&file_path) {
					Ok(file) => Some(file),
					Err(_) => File::create(&file_path).ok()
				}
			}
			None => None,
		};
		let log_file = match log_file {
			Some(file) => file,
			None => {
				error!(target: "privatetx", "Cannot open logs file");
				return;
			}
		};
		if let Err(err) = serde_json::to_writer(log_file, &self.logs.read().values().cloned().collect::<Vec<TransactionLog>>()) {
			error!(target: "privatetx", "Error during logs serialisation: {}", err);
		}
	}
}

// Flush all logs on drop
impl Drop for Logging {
	fn drop(&mut self) {
		self.flush_logs();
	}
}
