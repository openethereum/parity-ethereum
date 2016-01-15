use common::*;

/// State changes which should be applied in finalize,
/// after transaction is fully executed.
#[derive(Debug)]
pub struct Substate {
	/// Any accounts that have suicided.
	pub suicides: HashSet<Address>,
	/// Any logs.
	pub logs: Vec<LogEntry>,
	/// Refund counter of SSTORE nonzero->zero.
	pub refunds_count: U256,
	/// Created contracts.
	pub contracts_created: Vec<Address>
}

impl Substate {
	/// Creates new substate.
	pub fn new() -> Self {
		Substate {
			suicides: HashSet::new(),
			logs: vec![],
			refunds_count: U256::zero(),
			contracts_created: vec![]
		}
	}

	pub fn accrue(&mut self, s: Substate) {
		self.suicides.extend(s.suicides.into_iter());
		self.logs.extend(s.logs.into_iter());
		self.refunds_count = self.refunds_count + s.refunds_count;
		self.contracts_created.extend(s.contracts_created.into_iter());
	}
}
