// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Trace filters type definitions

use std::ops::Range;
use ethereum_types::{Address, Bloom, BloomInput};
use trace::flat::FlatTrace;
use super::trace::{Action, Res};

/// Addresses filter.
///
/// Used to create bloom possibilities and match filters.
#[derive(Debug)]
pub struct AddressesFilter {
	list: Vec<Address>
}

impl From<Vec<Address>> for AddressesFilter {
	fn from(addresses: Vec<Address>) -> Self {
		AddressesFilter { list: addresses }
	}
}

impl AddressesFilter {
	/// Returns true if address matches one of the searched addresses.
	pub fn matches(&self, address: &Address) -> bool {
		self.matches_all() || self.list.contains(address)
	}

	/// Returns true if this address filter matches everything.
	pub fn matches_all(&self) -> bool {
		self.list.is_empty()
	}

	/// Returns blooms of this addresses filter.
	pub fn blooms(&self) -> Vec<Bloom> {
		match self.list.is_empty() {
			true => vec![Bloom::default()],
			false => self.list.iter()
				.map(|address| Bloom::from(BloomInput::Raw(address.as_bytes())))
				.collect(),
		}
	}

	/// Returns vector of blooms zipped with blooms of this addresses filter.
	pub fn with_blooms(&self, blooms: Vec<Bloom>) -> Vec<Bloom> {
		match self.list.is_empty() {
			true => blooms,
			false => blooms
				.into_iter()
				.flat_map(|bloom| self.list.iter()
					.map(|address| {
						let mut bloom = bloom.clone();
						bloom.accrue(BloomInput::Raw(address.as_bytes()));
						bloom
					})
					.collect::<Vec<_>>())
				.collect(),
		}
	}
}

#[derive(Debug)]
/// Traces filter.
pub struct Filter {
	/// Block range.
	pub range: Range<usize>,

	/// From address filter.
	pub from_address: AddressesFilter,

	/// To address filter.
	pub to_address: AddressesFilter,
}

impl Filter {
	/// Returns combinations of each address.
	pub fn bloom_possibilities(&self) -> Vec<Bloom> {
		self.to_address.with_blooms(self.from_address.blooms())
	}

	/// Returns true if given trace matches the filter.
	pub fn matches(&self, trace: &FlatTrace) -> bool {
		match trace.action {
			Action::Call(ref call) => {
				let from_matches = self.from_address.matches(&call.from);
				let to_matches = self.to_address.matches(&call.to);
				from_matches && to_matches
			},
			Action::Create(ref create) => {
				let from_matches = self.from_address.matches(&create.from);

				let to_matches = match trace.result {
					Res::Create(ref create_result) => self.to_address.matches(&create_result.address),
					_ => self.to_address.matches_all(),
				};

				from_matches && to_matches
			},
			Action::Suicide(ref suicide) => {
				let from_matches = self.from_address.matches(&suicide.address);
				let to_matches = self.to_address.matches(&suicide.refund_address);
				from_matches && to_matches
			},
			Action::Reward(ref reward) => {
				self.from_address.matches_all() && self.to_address.matches(&reward.author)
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use ethereum_types::{Address, Bloom, BloomInput};
	use trace::trace::{Action, Call, Res, Create, CreateResult, Suicide, Reward};
	use trace::flat::FlatTrace;
	use trace::{Filter, AddressesFilter, TraceError, RewardType};
	use evm::CallType;

	#[test]
	fn empty_trace_filter_bloom_possibilities() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![]),
			to_address: AddressesFilter::from(vec![]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms, vec![Bloom::default()]);
	}

	#[test]
	fn single_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(1)]),
			to_address: AddressesFilter::from(vec![Address::from_low_u64_be(2)]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms.len(), 1);

		assert!(blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(1).as_bytes())));
		assert!(blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(2).as_bytes())));
		assert!(!blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(3).as_bytes())));
	}

	#[test]
	fn only_from_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms.len(), 1);

		assert!(blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(1).as_bytes())));
		assert!(!blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(2).as_bytes())));
	}

	#[test]
	fn only_to_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![]),
			to_address: AddressesFilter::from(vec![Address::from_low_u64_be(1)]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms.len(), 1);

		assert!(blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(1).as_bytes())));
		assert!(!blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(2).as_bytes())));
	}

	#[test]
	fn multiple_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(1), Address::from_low_u64_be(3)]),
			to_address: AddressesFilter::from(vec![Address::from_low_u64_be(2), Address::from_low_u64_be(4)]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms.len(), 4);

		assert!(blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(1).as_bytes())));
		assert!(blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(2).as_bytes())));
		assert!(!blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(3).as_bytes())));
		assert!(!blooms[0].contains_input(BloomInput::Raw(Address::from_low_u64_be(4).as_bytes())));

		assert!(blooms[1].contains_input(BloomInput::Raw(Address::from_low_u64_be(1).as_bytes())));
		assert!(blooms[1].contains_input(BloomInput::Raw(Address::from_low_u64_be(4).as_bytes())));
		assert!(!blooms[1].contains_input(BloomInput::Raw(Address::from_low_u64_be(2).as_bytes())));
		assert!(!blooms[1].contains_input(BloomInput::Raw(Address::from_low_u64_be(3).as_bytes())));

		assert!(blooms[2].contains_input(BloomInput::Raw(Address::from_low_u64_be(2).as_bytes())));
		assert!(blooms[2].contains_input(BloomInput::Raw(Address::from_low_u64_be(3).as_bytes())));
		assert!(!blooms[2].contains_input(BloomInput::Raw(Address::from_low_u64_be(1).as_bytes())));
		assert!(!blooms[2].contains_input(BloomInput::Raw(Address::from_low_u64_be(4).as_bytes())));

		assert!(blooms[3].contains_input(BloomInput::Raw(Address::from_low_u64_be(3).as_bytes())));
		assert!(blooms[3].contains_input(BloomInput::Raw(Address::from_low_u64_be(4).as_bytes())));
		assert!(!blooms[3].contains_input(BloomInput::Raw(Address::from_low_u64_be(1).as_bytes())));
		assert!(!blooms[3].contains_input(BloomInput::Raw(Address::from_low_u64_be(2).as_bytes())));
	}

	#[test]
	fn filter_matches() {
		let f0 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let f1 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(3), Address::from_low_u64_be(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let f2 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![]),
			to_address: AddressesFilter::from(vec![]),
		};

		let f3 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![]),
			to_address: AddressesFilter::from(vec![Address::from_low_u64_be(2)]),
		};

		let f4 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![]),
			to_address: AddressesFilter::from(vec![Address::from_low_u64_be(2), Address::from_low_u64_be(3)]),
		};

		let f5 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(1)]),
			to_address: AddressesFilter::from(vec![Address::from_low_u64_be(2), Address::from_low_u64_be(3)]),
		};

		let f6 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(1)]),
			to_address: AddressesFilter::from(vec![Address::from_low_u64_be(4)]),
		};

		let trace = FlatTrace {
			action: Action::Call(Call {
				from: Address::from_low_u64_be(1),
				to: Address::from_low_u64_be(2),
				value: 3.into(),
				gas: 4.into(),
				input: vec![0x5],
				call_type: CallType::Call,
			}),
			result: Res::FailedCall(TraceError::OutOfGas),
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
		};

		assert!(f0.matches(&trace));
		assert!(f1.matches(&trace));
		assert!(f2.matches(&trace));
		assert!(f3.matches(&trace));
		assert!(f4.matches(&trace));
		assert!(f5.matches(&trace));
		assert!(!f6.matches(&trace));

		let trace = FlatTrace {
			action: Action::Create(Create {
				from: Address::from_low_u64_be(1),
				value: 3.into(),
				gas: 4.into(),
				init: vec![0x5],
			}),
			result: Res::Create(CreateResult {
				gas_used: 10.into(),
				code: vec![],
				address: Address::from_low_u64_be(2),
			}),
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
		};

		assert!(f0.matches(&trace));
		assert!(f1.matches(&trace));
		assert!(f2.matches(&trace));
		assert!(f3.matches(&trace));
		assert!(f4.matches(&trace));
		assert!(f5.matches(&trace));
		assert!(!f6.matches(&trace));

		let trace = FlatTrace {
			action: Action::Suicide(Suicide {
				address: Address::from_low_u64_be(1),
				refund_address: Address::from_low_u64_be(2),
				balance: 3.into(),
			}),
			result: Res::None,
			trace_address: vec![].into_iter().collect(),
			subtraces: 0
		};

		assert!(f0.matches(&trace));
		assert!(f1.matches(&trace));
		assert!(f2.matches(&trace));
		assert!(f3.matches(&trace));
		assert!(f4.matches(&trace));
		assert!(f5.matches(&trace));
		assert!(!f6.matches(&trace));

		let trace = FlatTrace {
			action: Action::Reward(Reward {
				author: Address::from_low_u64_be(2),
				value: 100.into(),
				reward_type: RewardType::Block,
			}),
			result: Res::None,
			trace_address: vec![].into_iter().collect(),
			subtraces: 0
		};

		assert!(!f0.matches(&trace));
		assert!(!f1.matches(&trace));
		assert!(f2.matches(&trace));
		assert!(f3.matches(&trace));
		assert!(f4.matches(&trace));
		assert!(!f5.matches(&trace));
		assert!(!f6.matches(&trace));
	}

	#[test]
	fn filter_match_block_reward_fix_8070() {
		let f0 = Filter {
			range: (0..0),
			from_address: vec![Address::from_low_u64_be(1)].into(),
			to_address: vec![].into(),
		};

		let f1 = Filter {
			range: (0..0),
			from_address: vec![].into(),
			to_address: vec![].into(),
		};

		let f2 = Filter {
			range: (0..0),
			from_address: vec![].into(),
			to_address: vec![Address::from_low_u64_be(2)].into(),
		};

		let trace = FlatTrace {
			action: Action::Reward(Reward {
				author: Address::from_low_u64_be(2),
				value: 10.into(),
				reward_type: RewardType::Block,
			}),
			result: Res::None,
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
		};

		assert!(!f0.matches(&trace));
		assert!(f1.matches(&trace));
		assert!(f2.matches(&trace));
	}

  #[test]
  fn filter_match_failed_contract_creation_fix_9822() {

      let f0 = Filter {
          range: (0..0),
          from_address: vec![Address::from_low_u64_be(1)].into(),
          to_address: vec![].into(),
      };

      let f1 = Filter {
          range: (0..0),
          from_address: vec![].into(),
          to_address: vec![].into(),
      };

      let f2 = Filter {
          range: (0..0),
          from_address: vec![].into(),
          to_address: vec![Address::from_low_u64_be(2)].into(),
      };

      let trace = FlatTrace {
          action: Action::Create(Create {
              from: Address::from_low_u64_be(1),
              gas: 4.into(),
              init: vec![0x5],
              value: 3.into(),
          }),
          result: Res::FailedCall(TraceError::BadInstruction),
          trace_address: vec![].into_iter().collect(),
          subtraces: 0
      };

      assert!(f0.matches(&trace));
      assert!(f1.matches(&trace));
      assert!(!f2.matches(&trace));
  }

}
