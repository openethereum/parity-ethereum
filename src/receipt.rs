use util::*;
use basic_types::LogBloom;

/// A single log's entry.
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
}

/// Information describing execution of a transaction.
pub struct Receipt {
	pub state_root: H256,
	pub gas_used: U256,
	pub log_bloom: LogBloom,
	pub logs: Vec<LogEntry>,
}

impl RlpStandard for Receipt {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append_list(4);
		s.append(&self.state_root);
		s.append(&self.gas_used);
		s.append(&self.log_bloom);
		// TODO: make work:
		//s.append(&self.logs);
		s.append_list(self.logs.len());
		for l in self.logs.iter() {
			l.rlp_append(s);
		}
	}
}
