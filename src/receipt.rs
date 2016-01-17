use util::*;
use basic_types::LogBloom;
use log_entry::LogEntry;

/// Information describing execution of a transaction.
#[derive(Debug)]
pub struct Receipt {
	pub state_root: H256,
	pub gas_used: U256,
	pub log_bloom: LogBloom,
	pub logs: Vec<LogEntry>,
}

impl Receipt {
	pub fn new(state_root: H256, gas_used: U256, logs: Vec<LogEntry>) -> Receipt {
		Receipt {
			state_root: state_root,
			gas_used: gas_used,
			log_bloom: logs.iter().fold(LogBloom::new(), |mut b, l| { b |= &l.bloom(); b }),
			logs: logs,
		}
	}
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
		for l in &self.logs {
			l.rlp_append(s);
		}
	}
}
