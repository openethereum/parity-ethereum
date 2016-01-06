use util::hash::*;
use util::bytes::*;
use util::sha3::*;

/// Data sturcture used to represent Evm log entry.
pub struct LogEntry {
	address: Address,
	topics: Vec<H256>,
	data: Bytes
}

impl LogEntry {
	/// This function should be called to create new log entry.
	pub fn new(address: Address, topics: Vec<H256>, data: Bytes) -> LogEntry {
		LogEntry {
			address: address,
			topics: topics,
			data: data
		}
	}

	/// Returns reference to address.
	pub fn address(&self) -> &Address {
		&self.address
	}

	/// Returns reference to topics.
	pub fn topics(&self) -> &[H256] {
		&self.topics
	}

	/// Returns reference to data.
	pub fn data(&self) -> &Bytes {
		&self.data
	}

	/// Returns log bloom of given log entry.
	pub fn bloom(&self) -> H2048 {
		let mut bloom = H2048::new();
		bloom.shift_bloom(&self.address.sha3());
		for topic in self.topics.iter() {
			bloom.shift_bloom(&topic.sha3());
		}
		bloom
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::*;
	use util::bytes::*;
	use evm::LogEntry;

	#[test]
	fn test_empty_log_bloom() {
		let bloom = H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let log = LogEntry::new(address, vec![], vec![]);
		assert_eq!(log.bloom(), bloom);
	}
}
