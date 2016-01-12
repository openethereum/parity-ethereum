use util::*;
use basic_types::LogBloom;

/// A single log's entry.
#[derive(Debug)]
pub struct LogEntry {
	pub address: Address,
	pub topics: Vec<H256>,
	pub data: Bytes,
}

impl RlpStandard for LogEntry {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append_list(3);
		s.append(&self.address);
		s.append(&self.topics);
		s.append(&self.data);
	}
}

impl LogEntry {
	pub fn bloom(&self) -> LogBloom {
		self.topics.iter().fold(LogBloom::from_bloomed(&self.address.sha3()), |b, t| b.with_bloomed(&t.sha3()))
	}

	/// Create a new log entry.
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
	pub fn topics(&self) -> &Vec<H256> {
		&self.topics
	}

	/// Returns reference to data.
	pub fn data(&self) -> &Bytes {
		&self.data
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use super::LogEntry;

	#[test]
	fn test_empty_log_bloom() {
		let bloom = H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let log = LogEntry::new(address, vec![], vec![]);
		assert_eq!(log.bloom(), bloom);
	}
}
