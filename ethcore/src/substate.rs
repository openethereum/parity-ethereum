//! Execution environment substate.
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

#[cfg(test)]
mod tests {
	use super::*;
	use common::*;

	#[test]
	fn accrue() {
		let mut sub_state = Substate::new();
		sub_state.contracts_created.push(address_from_u64(1u64));
		sub_state.logs.push(LogEntry::new(address_from_u64(1u64), vec![], vec![]));
		sub_state.sstore_clears_count = x!(5);
		sub_state.suicides.insert(address_from_u64(10u64));

		let mut sub_state_2 = Substate::new();
		sub_state_2.contracts_created.push(address_from_u64(2u64));
		sub_state_2.logs.push(LogEntry::new(address_from_u64(1u64), vec![], vec![]));
		sub_state_2.sstore_clears_count = x!(7);

		sub_state.accrue(sub_state_2);
		assert_eq!(sub_state.contracts_created.len(), 2);
		assert_eq!(sub_state.sstore_clears_count, x!(12));
		assert_eq!(sub_state.suicides.len(), 1);
	}
}
