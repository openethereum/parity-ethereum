use util::*;
use basic_types::LogBloom;
use log_entry::LogEntry;

/// Information describing execution of a transaction.
#[derive(Debug)]
pub struct Receipt {
	/// TODO [Gav Wood] Please document me
	pub state_root: H256,
	/// TODO [Gav Wood] Please document me
	pub gas_used: U256,
	/// TODO [Gav Wood] Please document me
	pub log_bloom: LogBloom,
	/// TODO [Gav Wood] Please document me
	pub logs: Vec<LogEntry>,
}

impl Receipt {
	/// TODO [Gav Wood] Please document me
	pub fn new(state_root: H256, gas_used: U256, logs: Vec<LogEntry>) -> Receipt {
		Receipt {
			state_root: state_root,
			gas_used: gas_used,
			log_bloom: logs.iter().fold(LogBloom::new(), |mut b, l| { b |= &l.bloom(); b }),
			logs: logs,
		}
	}
}

impl Encodable for Receipt {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.state_root);
		s.append(&self.gas_used);
		s.append(&self.log_bloom);
		s.append_list(&self.logs);
	}
}
