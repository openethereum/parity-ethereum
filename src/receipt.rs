use util::*;
use basic_types::LogBloom;
use log_entry::LogEntry;

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
