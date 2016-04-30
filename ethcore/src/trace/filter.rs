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

use std::ops::Range;
use bloomchain::{Filter as BloomFilter, Bloom, Number};
use util::{Address, FixedHash};
use util::sha3::Hashable;
use basic_types::LogBloom;
use super::flat::FlatTrace;
use super::trace::Action;

/// Addresses filter.
///
/// Used to create bloom possibilities and match filters.
pub struct AddressesFilter(Vec<Address>);

impl From<Vec<Address>> for AddressesFilter {
	fn from(addresses: Vec<Address>) -> Self {
		AddressesFilter(addresses)
	}
}

impl AddressesFilter {
	/// Returns true if address matches one of the searched addresses.
	pub fn matches(&self, address: &Address) -> bool {
		self.matches_all() || self.0.contains(address)
	}

	/// Returns true if this address filter matches everything.
	pub fn matches_all(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns blooms of this addresses filter.
	pub fn blooms(&self) -> Vec<LogBloom> {
		match self.0.is_empty() {
			true => vec![LogBloom::new()],
			false => self.0.iter()
				.map(|address| LogBloom::from_bloomed(&address.sha3()))
				.collect()
		}
	}

	/// Returns vector of blooms zipped with blooms of this addresses filter.
	pub fn with_blooms(&self, blooms: Vec<LogBloom>) -> Vec<LogBloom> {
		match self.0.is_empty() {
			true => blooms,
			false => blooms
				.into_iter()
				.flat_map(|bloom| self.0.iter()
					.map(|address| bloom.with_bloomed(&address.sha3()))
					.collect::<Vec<_>>())
				.collect()
		}
	}
}

/// Traces filter.
pub struct Filter {
	/// Block range.
	pub range: Range<usize>,

	/// From address filter.
	pub from_address: AddressesFilter,

	/// To address filter.
	pub to_address: AddressesFilter,
}

impl BloomFilter for Filter {
	fn bloom_possibilities(&self) -> Vec<Bloom> {
		self.bloom_possibilities()
			.into_iter()
			.map(|b| Bloom::from(b.0))
			.collect()
	}

	fn range(&self) -> Range<Number> {
		self.range.clone()
	}
}

impl Filter {
	/// Returns combinations of each address.
	fn bloom_possibilities(&self) -> Vec<LogBloom> {
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
				let to_matches = self.to_address.matches_all();
				from_matches && to_matches
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use util::{FixedHash, Address, U256};
	use util::sha3::Hashable;
	use trace::trace::{Action, Call, Res};
	use trace::flat::FlatTrace;
	use trace::{Filter, AddressesFilter};
	use basic_types::LogBloom;

	#[test]
	fn empty_trace_filter_bloom_possibilities() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![]),
			to_address: AddressesFilter::from(vec![]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms, vec![LogBloom::new()]);
	}

	#[test]
	fn single_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from(1)]),
			to_address: AddressesFilter::from(vec![Address::from(2)]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms.len(), 1);

		assert!(blooms[0].contains_bloomed(&Address::from(1).sha3()));
		assert!(blooms[0].contains_bloomed(&Address::from(2).sha3()));
		assert!(!blooms[0].contains_bloomed(&Address::from(3).sha3()));
	}

	#[test]
	fn only_from_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms.len(), 1);

		assert!(blooms[0].contains_bloomed(&Address::from(1).sha3()));
		assert!(!blooms[0].contains_bloomed(&Address::from(2).sha3()));
	}

	#[test]
	fn only_to_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![]),
			to_address: AddressesFilter::from(vec![Address::from(1)]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms.len(), 1);

		assert!(blooms[0].contains_bloomed(&Address::from(1).sha3()));
		assert!(!blooms[0].contains_bloomed(&Address::from(2).sha3()));
	}

	#[test]
	fn multiple_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from(1), Address::from(3)]),
			to_address: AddressesFilter::from(vec![Address::from(2), Address::from(4)]),
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms.len(), 4);

		assert!(blooms[0].contains_bloomed(&Address::from(1).sha3()));
		assert!(blooms[0].contains_bloomed(&Address::from(2).sha3()));
		assert!(!blooms[0].contains_bloomed(&Address::from(3).sha3()));
		assert!(!blooms[0].contains_bloomed(&Address::from(4).sha3()));

		assert!(blooms[1].contains_bloomed(&Address::from(1).sha3()));
		assert!(blooms[1].contains_bloomed(&Address::from(4).sha3()));
		assert!(!blooms[1].contains_bloomed(&Address::from(2).sha3()));
		assert!(!blooms[1].contains_bloomed(&Address::from(3).sha3()));

		assert!(blooms[2].contains_bloomed(&Address::from(2).sha3()));
		assert!(blooms[2].contains_bloomed(&Address::from(3).sha3()));
		assert!(!blooms[2].contains_bloomed(&Address::from(1).sha3()));
		assert!(!blooms[2].contains_bloomed(&Address::from(4).sha3()));

		assert!(blooms[3].contains_bloomed(&Address::from(3).sha3()));
		assert!(blooms[3].contains_bloomed(&Address::from(4).sha3()));
		assert!(!blooms[3].contains_bloomed(&Address::from(1).sha3()));
		assert!(!blooms[3].contains_bloomed(&Address::from(2).sha3()));
	}

	#[test]
	fn filter_matches() {
		let f0 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let f1 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from(3), Address::from(1)]),
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
			to_address: AddressesFilter::from(vec![Address::from(2)]),
		};

		let f4 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![]),
			to_address: AddressesFilter::from(vec![Address::from(2), Address::from(3)]),
		};

		let f5 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from(1)]),
			to_address: AddressesFilter::from(vec![Address::from(2), Address::from(3)]),
		};

		let f6 = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from(1)]),
			to_address: AddressesFilter::from(vec![Address::from(4)]),
		};

		let trace = FlatTrace {
			action: Action::Call(Call {
				from: Address::from(1),
				to: Address::from(2),
				value: U256::from(3),
				gas: U256::from(4),
				input: vec![0x5],
			}),
			result: Res::FailedCall,
			trace_address: vec![0],
			subtraces: 0,
		};

		assert!(f0.matches(&trace));
		assert!(f1.matches(&trace));
		assert!(f2.matches(&trace));
		assert!(f3.matches(&trace));
		assert!(f4.matches(&trace));
		assert!(f5.matches(&trace));
		assert!(!f6.matches(&trace));
	}
}
