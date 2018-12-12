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

/// Maximum amount of stored private transaction logs.
const MAX_JOURNAL_LEN: usize = 1000;

/// Maximum period for storing private transaction logs.
/// Older logs will not be processed, 20 days
const MAX_STORING_TIME: u64 = 60 * 60 * 24 * 20;

/// Current status of the private transaction
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum PrivateTxStatus {
	/// Private tx was created but no validation received yet
	Created,
	/// Several validators (but not all) validated the transaction
	Validating,
	/// All validators validated the private tx
	/// Corresponding public tx was created and added into the pool
	Deployed,
}

/// Information about private tx validation
#[derive(Clone, Serialize, Deserialize)]
pub struct ValidatorLog {
	/// Account of the validator
	pub account: Address,
	/// Validation flag
	pub validated: bool,
	/// Validation timestamp
	pub validation_timestamp: Option<u64>,
}

/// Information about the private transaction
#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionLog {
	/// Original signed transaction hash (used as a source for private tx)
	pub tx_hash: H256,
	/// Current status of the private transaction
	pub status: PrivateTxStatus,
	/// Creation timestamp
	pub creation_timestamp: u64,
	/// List of validations
	pub validators: Vec<ValidatorLog>,
	/// Timestamp of the resulting public tx deployment
	pub deployment_timestamp: Option<u64>,
	/// Hash of the resulting public tx
	pub public_tx_hash: Option<H256>,
}

/// Private transactions logging
pub struct Logging {
	logs: RwLock<HashMap<H256, TransactionLog>>,
	logs_dir: Option<String>,
}

impl Logging {
	/// Creates the logging object
	pub fn new(logs_dir: Option<String>) -> Self {
		let logging = Logging {
			logs: RwLock::new(HashMap::new()),
			logs_dir,
		};
		logging.read_logs();
		logging
	}

	/// Retrieves log for the corresponding tx hash
	pub fn tx_log(&self, tx_hash: H256) -> Option<TransactionLog> {
		self.logs.read().get(&tx_hash).cloned()
	}

	/// Logs the creation of private transaction
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
		if logs.len() > MAX_JOURNAL_LEN {
			// Remove the oldest log
			let mut sorted_logs = logs.values().cloned().collect::<Vec<TransactionLog>>();
			sorted_logs.sort_by(|a, b| a.creation_timestamp.cmp(&b.creation_timestamp));
			logs.remove(&sorted_logs[0].tx_hash);
		}
		logs.insert(tx_hash, TransactionLog {
			tx_hash,
			status: PrivateTxStatus::Created,
			creation_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
			validators: validator_logs,
			deployment_timestamp: None,
			public_tx_hash: None,
		});
	}

	/// Logs the obtaining of the signature for the private transaction
	pub fn signature_added(&self, tx_hash: H256, validator: Address) {
		let mut logs = self.logs.write();
		if let Some(transaction_log) = logs.get_mut(&tx_hash) {
			transaction_log.status = PrivateTxStatus::Validating;
			if let Some(ref mut validator_log) = transaction_log.validators.iter_mut().find(|log| log.account == validator) {
				validator_log.validated = true;
				validator_log.validation_timestamp = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
			}
		}
	}

	/// Logs the final deployment of the resulting public transaction
	pub fn tx_deployed(&self, tx_hash: H256, public_tx_hash: H256) {
		let mut logs = self.logs.write();
		if let Some(log) = logs.get_mut(&tx_hash) {
			log.status = PrivateTxStatus::Deployed;
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
		let mut transaction_logs: Vec<TransactionLog> = match serde_json::from_reader(log_file) {
			Ok(logs) => logs,
			Err(err) => {
				error!(target: "privatetx", "Cannot deserialize logs from file: {}", err);
				return;
			}
		};
		// Drop old logs
		let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
		transaction_logs.retain(|tx_log| tx_log.creation_timestamp - current_timestamp < MAX_STORING_TIME);
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

#[cfg(test)]
mod tests {
	use serde_json;
	use transaction::{Transaction};
	use super::{TransactionLog, Logging, PrivateTxStatus};

	#[test]
	fn private_log_format() {
		let s = r#"{
			"tx_hash":"0x64f648ca7ae7f4138014f860ae56164d8d5732969b1cea54d8be9d144d8aa6f6",
			"status":"Deployed",
			"creation_timestamp":1544528180,
			"validators":[{
				"account":"0x82a978b3f5962a5b0957d9ee9eef472ee55b42f1",
				"validated":true,
				"validation_timestamp":1544528181
			}],
			"deployment_timestamp":1544528181,
			"public_tx_hash":"0x69b9c691ede7993effbcc88911c309af1c82be67b04b3882dd446b808ae146da"
		}"#;

		let _deserialized: TransactionLog = serde_json::from_str(s).unwrap();
	}

	#[test]
	fn private_log_status() {
		let logger = Logging::new(None);
		let private_tx = Transaction::default();
		let hash = private_tx.hash(None);
		logger.private_tx_created(hash, &vec!["0x82a978b3f5962a5b0957d9ee9eef472ee55b42f1".into()]);
		logger.signature_added(hash, "0x82a978b3f5962a5b0957d9ee9eef472ee55b42f1".into());
		logger.tx_deployed(hash, hash);
		let tx_log = logger.tx_log(hash).unwrap();
		assert_eq!(tx_log.status, PrivateTxStatus::Deployed);
	}
}