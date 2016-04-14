// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Execution environment substate.
use common::*;

/// State changes which should be applied in finalize,
/// after transaction is fully executed.
#[derive(Debug, Default)]
pub struct Substate {
	/// Any accounts that have suicided.
	pub suicides: HashSet<Address>,

	/// Any logs.
	pub logs: Vec<LogEntry>,

	/// Refund counter of SSTORE nonzero -> zero.
	pub sstore_clears_count: U256,

	/// Created contracts.
	pub contracts_created: Vec<Address>,
}

impl Substate {
	/// Creates new substate.
	pub fn new() -> Self {
		Substate {
			suicides: Default::default(),
			logs: Default::default(),
			sstore_clears_count: Default::default(),
			contracts_created: Default::default(),
		}
	}

	/// Merge secondary substate `s` into self, accruing each element correspondingly.
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
	fn created() {
		let sub_state = Substate::new();
		assert_eq!(sub_state.suicides.len(), 0);
	}

	#[test]
	fn accrue() {
		let mut sub_state = Substate::new();
		sub_state.contracts_created.push(address_from_u64(1u64));
		sub_state.logs.push(LogEntry {
			address: address_from_u64(1u64),
			topics: vec![],
			data: vec![]
		});
		sub_state.sstore_clears_count = x!(5);
		sub_state.suicides.insert(address_from_u64(10u64));

		let mut sub_state_2 = Substate::new();
		sub_state_2.contracts_created.push(address_from_u64(2u64));
		sub_state_2.logs.push(LogEntry {
			address: address_from_u64(1u64),
			topics: vec![],
			data: vec![]
		});
		sub_state_2.sstore_clears_count = x!(7);

		sub_state.accrue(sub_state_2);
		assert_eq!(sub_state.contracts_created.len(), 2);
		assert_eq!(sub_state.sstore_clears_count, x!(12));
		assert_eq!(sub_state.suicides.len(), 1);
	}
}
