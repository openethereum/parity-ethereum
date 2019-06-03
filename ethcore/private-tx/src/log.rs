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
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, Duration, Instant};
use parking_lot::RwLock;
use serde::ser::{Serializer, SerializeSeq};
use error::Error;

#[cfg(not(time_checked_add))]
use time_utils::CheckedSystemTime;

/// Maximum amount of stored private transaction logs.
const MAX_JOURNAL_LEN: usize = 1000;

/// Maximum period for storing private transaction logs.
/// Logs older than 20 days will not be processed
const MAX_STORING_TIME: Duration = Duration::from_secs(60 * 60 * 24 * 20);

/// Source of monotonic time for log timestamps
struct MonoTime {
	start_time: SystemTime,
	start_inst: Instant
}

impl MonoTime {
	fn new(start: SystemTime) -> Self {
		Self {
			start_time: start,
			start_inst: Instant::now()
		}
	}

	fn elapsed(&self) -> Duration {
		self.start_inst.elapsed()
	}

	fn to_system_time(&self) -> SystemTime {
		self.start_time + self.elapsed()
	}
}

impl Default for MonoTime {
	fn default() -> Self {
		MonoTime::new(SystemTime::now())
	}
}

/// Current status of the private transaction
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum PrivateTxStatus {
	/// Private tx was created but no validation received yet
	Created,
	/// Several validators (but not all) validated the transaction
	Validating,
	/// All validators has validated the private tx
	/// Corresponding public tx was created and added into the pool
	Deployed,
}

/// Information about private tx validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorLog {
	/// Account of the validator
	pub account: Address,
	/// Validation timestamp, None if the transaction is not validated
	pub validation_timestamp: Option<SystemTime>,
}

#[cfg(test)]
impl PartialEq for ValidatorLog {
	fn eq(&self, other: &Self) -> bool {
		self.account == other.account
	}
}

/// Information about the private transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLog {
	/// Original signed transaction hash (used as a source for private tx)
	pub tx_hash: H256,
	/// Current status of the private transaction
	pub status: PrivateTxStatus,
	/// Creation timestamp
	pub creation_timestamp: SystemTime,
	/// List of validations
	pub validators: Vec<ValidatorLog>,
	/// Timestamp of the resulting public tx deployment
	pub deployment_timestamp: Option<SystemTime>,
	/// Hash of the resulting public tx
	pub public_tx_hash: Option<H256>,
}

#[cfg(test)]
impl PartialEq for TransactionLog {
	fn eq(&self, other: &Self) -> bool {
		self.tx_hash == other.tx_hash &&
		self.status == other.status &&
		self.validators == other.validators &&
		self.public_tx_hash == other.public_tx_hash
	}
}

/// Wrapper other JSON serializer
pub trait LogsSerializer: Send + Sync + 'static {
	/// Read logs from the source
	fn read_logs(&self) -> Result<Vec<TransactionLog>, Error>;

	/// Write all logs to the source
	fn flush_logs(&self, logs: &HashMap<H256, TransactionLog>) -> Result<(), Error>;
}

/// Logs serializer to the json file
pub struct FileLogsSerializer {
	logs_dir: PathBuf,
}

impl FileLogsSerializer {
	pub fn with_path<P: Into<PathBuf>>(logs_dir: P) -> Self {
		FileLogsSerializer {
			logs_dir: logs_dir.into(),
		}
	}

	fn open_file(&self, to_create: bool) -> Result<File, Error> {
		let file_path = self.logs_dir.with_file_name("private_tx.log");
		if to_create {
			File::create(&file_path).map_err(From::from)
		} else {
			File::open(&file_path).map_err(From::from)
		}
	}
}

impl LogsSerializer for FileLogsSerializer {
	fn read_logs(&self) -> Result<Vec<TransactionLog>, Error> {
		let log_file = self.open_file(false)?;
		match serde_json::from_reader(log_file) {
			Ok(logs) => Ok(logs),
			Err(err) => {
				error!(target: "privatetx", "Cannot deserialize logs from file: {}", err);
				return Err(format!("Cannot deserialize logs from file: {:?}", err).into());
			}
		}
	}

	fn flush_logs(&self, logs: &HashMap<H256, TransactionLog>) -> Result<(), Error> {
		if logs.is_empty() {
			// Do not create empty file
			return Ok(());
		}
		let log_file = self.open_file(true)?;
		let mut json = serde_json::Serializer::new(log_file);
		let mut json_array = json.serialize_seq(Some(logs.len()))?;
		for v in logs.values() {
			json_array.serialize_element(v)?;
		}
		json_array.end()?;
		Ok(())
	}
}

/// Private transactions logging
pub struct Logging {
	logs: RwLock<HashMap<H256, TransactionLog>>,
	logs_serializer: Arc<LogsSerializer>,
	mono_time: MonoTime,
}

impl Logging {
	/// Creates the logging object
	pub fn new(logs_serializer: Arc<LogsSerializer>) -> Self {
		let mut logging = Logging {
			logs: RwLock::new(HashMap::new()),
			logs_serializer,
			mono_time: MonoTime::default(),
		};
		match logging.read_logs() {
			// Initialize time source by max from current system time and max creation time from already saved logs
			Ok(initial_time) => logging.mono_time = MonoTime::new(initial_time),
			Err(err) => warn!(target: "privatetx", "Cannot read logs: {:?}", err),
		}
		logging
	}

	/// Retrieves log for the corresponding tx hash
	pub fn tx_log(&self, tx_hash: &H256) -> Option<TransactionLog> {
		self.logs.read().get(&tx_hash).cloned()
	}

	/// Logs the creation of the private transaction
	pub fn private_tx_created(&self, tx_hash: &H256, validators: &[Address]) {
		let mut validator_logs = Vec::new();
		for account in validators {
			validator_logs.push(ValidatorLog {
				account: *account,
				validation_timestamp: None,
			});
		}
		let mut logs = self.logs.write();
		if logs.len() > MAX_JOURNAL_LEN {
			// Remove the oldest log
			if let Some(tx_hash) = logs.values()
				.min_by(|x, y| x.creation_timestamp.cmp(&y.creation_timestamp))
				.map(|oldest| oldest.tx_hash)
			{
				logs.remove(&tx_hash);
			}
		}
		logs.insert(*tx_hash, TransactionLog {
			tx_hash: *tx_hash,
			status: PrivateTxStatus::Created,
			creation_timestamp: self.mono_time.to_system_time(),
			validators: validator_logs,
			deployment_timestamp: None,
			public_tx_hash: None,
		});
	}

	/// Logs the validation of the private transaction by one of its validators
	pub fn signature_added(&self, tx_hash: &H256, validator: &Address) {
		let mut logs = self.logs.write();
		if let Some(transaction_log) = logs.get_mut(&tx_hash) {
			if let Some(ref mut validator_log) = transaction_log.validators.iter_mut().find(|log| log.account == *validator) {
				transaction_log.status = PrivateTxStatus::Validating;
				validator_log.validation_timestamp = Some(self.mono_time.to_system_time());
			}
		}
	}

	/// Logs the final deployment of the resulting public transaction
	pub fn tx_deployed(&self, tx_hash: &H256, public_tx_hash: &H256) {
		let mut logs = self.logs.write();
		if let Some(log) = logs.get_mut(&tx_hash) {
			log.status = PrivateTxStatus::Deployed;
			log.deployment_timestamp = Some(self.mono_time.to_system_time());
			log.public_tx_hash = Some(*public_tx_hash);
		}
	}

	fn read_logs(&self) -> Result<SystemTime, Error> {
		let mut transaction_logs = self.logs_serializer.read_logs()?;
		// Drop old logs
		let earliest_possible = SystemTime::now().checked_sub(MAX_STORING_TIME).ok_or(Error::TimestampOverflow)?;
		transaction_logs.retain(|tx_log| tx_log.creation_timestamp > earliest_possible);
		// Sort logs by their creation time in order to find the most recent
		transaction_logs.sort_by(|a, b| b.creation_timestamp.cmp(&a.creation_timestamp));
		let initial_timestamp = transaction_logs.first()
			.map_or(SystemTime::now(), |l| std::cmp::max(SystemTime::now(), l.creation_timestamp));
		let mut logs = self.logs.write();
		for log in transaction_logs {
			logs.insert(log.tx_hash, log);
		}
		Ok(initial_timestamp)
	}

	fn flush_logs(&self) -> Result<(), Error> {
		let logs = self.logs.read();
		self.logs_serializer.flush_logs(&logs)
	}
}

// Flush all logs on drop
impl Drop for Logging {
	fn drop(&mut self) {
		if let Err(err) = self.flush_logs() {
			warn!(target: "privatetx", "Cannot write logs: {:?}", err);
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use error::Error;
	use ethereum_types::{H256, Address};
	use std::collections::{HashMap, BTreeMap};
	use std::sync::Arc;
	use std::time::{SystemTime, Duration};
	use std::str::FromStr;
	use types::transaction::Transaction;
	use parking_lot::RwLock;
	use super::{TransactionLog, Logging, PrivateTxStatus, LogsSerializer, ValidatorLog};

	#[cfg(not(time_checked_add))]
	use time_utils::CheckedSystemTime;

	struct StringLogSerializer {
		string_log: RwLock<String>,
	}

	impl StringLogSerializer {
		fn new(source: String) -> Self {
			StringLogSerializer {
				string_log: RwLock::new(source),
			}
		}

		fn log(&self) -> String {
			let log = self.string_log.read();
			log.clone()
		}
	}

	impl LogsSerializer for StringLogSerializer {
		fn read_logs(&self) -> Result<Vec<TransactionLog>, Error> {
			let source = self.string_log.read();
			if source.is_empty() {
				return Ok(Vec::new())
			}
			let logs = serde_json::from_str(&source).unwrap();
			Ok(logs)
		}

		fn flush_logs(&self, logs: &HashMap<H256, TransactionLog>) -> Result<(), Error> {
			// Sort logs in order to have the same order
			let sorted_logs: BTreeMap<&H256, &TransactionLog> = logs.iter().collect();
			*self.string_log.write() = serde_json::to_string(&sorted_logs.values().collect::<Vec<&&TransactionLog>>())?;
			Ok(())
		}
	}

	#[test]
	fn private_log_format() {
		let s = r#"{
			"tx_hash":"0x64f648ca7ae7f4138014f860ae56164d8d5732969b1cea54d8be9d144d8aa6f6",
			"status":"Deployed",
			"creation_timestamp":{"secs_since_epoch":1557220355,"nanos_since_epoch":196382053},
			"validators":[{
				"account":"0x82a978b3f5962a5b0957d9ee9eef472ee55b42f1",
				"validation_timestamp":{"secs_since_epoch":1557220355,"nanos_since_epoch":196382053}
			}],
			"deployment_timestamp":{"secs_since_epoch":1557220355,"nanos_since_epoch":196382053},
			"public_tx_hash":"0x69b9c691ede7993effbcc88911c309af1c82be67b04b3882dd446b808ae146da"
		}"#;

		let _deserialized: TransactionLog = serde_json::from_str(s).unwrap();
	}

	#[test]
	fn private_log_status() {
		let logger = Logging::new(Arc::new(StringLogSerializer::new("".into())));
		let private_tx = Transaction::default();
		let hash = private_tx.hash(None);
		logger.private_tx_created(&hash, &vec![Address::from_str("82a978b3f5962a5b0957d9ee9eef472ee55b42f1").unwrap()]);
		logger.signature_added(&hash, &Address::from_str("82a978b3f5962a5b0957d9ee9eef472ee55b42f1").unwrap());
		logger.tx_deployed(&hash, &hash);
		let tx_log = logger.tx_log(&hash).unwrap();
		assert_eq!(tx_log.status, PrivateTxStatus::Deployed);
	}

	#[test]
	fn serialization() {
		let current_timestamp = SystemTime::now();
		let initial_validator_log = ValidatorLog {
			account: Address::from_str("82a978b3f5962a5b0957d9ee9eef472ee55b42f1").unwrap(),
			validation_timestamp: Some(current_timestamp.checked_add(Duration::from_secs(1)).unwrap()),
		};
		let initial_log = TransactionLog {
			tx_hash: H256::from_str("64f648ca7ae7f4138014f860ae56164d8d5732969b1cea54d8be9d144d8aa6f6").unwrap(),
			status: PrivateTxStatus::Deployed,
			creation_timestamp: current_timestamp,
			validators: vec![initial_validator_log],
			deployment_timestamp: Some(current_timestamp.checked_add(Duration::from_secs(2)).unwrap()),
			public_tx_hash: Some(H256::from_str("69b9c691ede7993effbcc88911c309af1c82be67b04b3882dd446b808ae146da").unwrap()),
		};
		let serializer = Arc::new(StringLogSerializer::new(serde_json::to_string(&vec![initial_log.clone()]).unwrap()));
		let logger = Logging::new(serializer.clone());
		let hash = H256::from_str("63c715e88f7291e66069302f6fcbb4f28a19ef5d7cbd1832d0c01e221c0061c6").unwrap();
		logger.private_tx_created(&hash, &vec![Address::from_str("7ffbe3512782069be388f41be4d8eb350672d3a5").unwrap()]);
		logger.signature_added(&hash, &Address::from_str("7ffbe3512782069be388f41be4d8eb350672d3a5").unwrap());
		logger.tx_deployed(&hash, &H256::from_str("de2209a8635b9cab9eceb67928b217c70ab53f6498e5144492ec01e6f43547d7").unwrap());
		drop(logger);
		let added_validator_log = ValidatorLog {
			account: Address::from_str("7ffbe3512782069be388f41be4d8eb350672d3a5").unwrap(),
			validation_timestamp: Some(current_timestamp.checked_add(Duration::from_secs(7)).unwrap()),
		};
		let added_log = TransactionLog {
			tx_hash: H256::from_str("63c715e88f7291e66069302f6fcbb4f28a19ef5d7cbd1832d0c01e221c0061c6").unwrap(),
			status: PrivateTxStatus::Deployed,
			creation_timestamp: current_timestamp.checked_add(Duration::from_secs(6)).unwrap(),
			validators: vec![added_validator_log],
			deployment_timestamp: Some(current_timestamp.checked_add(Duration::from_secs(8)).unwrap()),
			public_tx_hash: Some(H256::from_str("de2209a8635b9cab9eceb67928b217c70ab53f6498e5144492ec01e6f43547d7").unwrap()),
		};
		let should_be_final = vec![added_log, initial_log];
		let deserialized_logs: Vec<TransactionLog> = serde_json::from_str(&serializer.log()).unwrap();
		assert_eq!(deserialized_logs, should_be_final);
	}
}
