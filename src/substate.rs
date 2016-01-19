use common::*;

/// State changes which should be applied in finalize,
/// after transaction is fully executed.
#[derive(Debug)]
pub struct Substate {
	/// Any accounts that have suicided.
	pub suicides: HashSet<Address>,
	/// Any logs.
	pub logs: Vec<LogEntry>,
	/// Refund counter of SSTORE nonzero -> zero.
	pub sstore_clears_count: U256,
	/// Created contracts.
	pub contracts_created: Vec<Address>
}

impl Substate {
	/// Creates new substate.
	pub fn new() -> Self {
		Substate {
			suicides: HashSet::new(),
			logs: vec![],
			sstore_clears_count: U256::zero(),
			contracts_created: vec![]
		}
	}

	/// TODO [Gav Wood] Please document me
	pub fn accrue(&mut self, s: Substate) {
		self.suicides.extend(s.suicides.into_iter());
		self.logs.extend(s.logs.into_iter());
		self.sstore_clears_count = self.sstore_clears_count + s.sstore_clears_count;
		self.contracts_created.extend(s.contracts_created.into_iter());
	}
}
