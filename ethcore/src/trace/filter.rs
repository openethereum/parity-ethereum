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
use super::trace::{Trace, Action};

/// Traces filter.
pub struct Filter {
	/// Block range.
	pub range: Range<usize>,

	/// From address. If empty, match all, if not, match one of the values.
	pub from_address: Vec<Address>,

	/// To address. If empty, match all, if not, match one of the values.
	pub to_address: Vec<Address>,
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
		let blooms = match self.from_address.is_empty() {
			true => vec![LogBloom::new()],
			false => self.from_address
				.iter()
				.map(|address| LogBloom::from_bloomed(&address.sha3()))
				.collect()
		};

		match self.to_address.is_empty() {
			true => blooms,
			false => blooms
				.into_iter()
				.flat_map(|bloom| self.to_address
					.iter()
					.map(| address | bloom.with_bloomed(&address.sha3()))
					.collect::<Vec<_>>())
				.collect()
		}
	}

	/// Returns true if given trace matches the filter.
	pub fn matches(&self, trace: &Trace) -> bool {
		let matches = match trace.action {
			Action::Call(ref call) => {
				let from_matches = self.from_address.is_empty() || self.from_address.contains(&call.from);
				let to_matches = self.to_address.is_empty() || self.to_address.contains(&call.to);
				from_matches && to_matches
			},
			Action::Create(ref create) => {
				let from_matches = self.from_address.is_empty() || self.from_address.contains(&create.from);
				let to_matches = self.to_address.is_empty();
				from_matches && to_matches
			}
		};

		matches || trace.subs.iter().any(|subtrace| self.matches(subtrace))
	}
}

#[cfg(test)]
mod tests {
	use util::{FixedHash, Address};
	use util::sha3::Hashable;
	use trace::Filter;
	use client::BlockId;
	use basic_types::LogBloom;

	#[test]
	fn empty_trace_filter_bloom_possibilies() {
		let filter = Filter {
			range: (0..0),
			from_address: vec![],
			to_address: vec![],
		};

		let blooms = filter.bloom_possibilities();
		assert_eq!(blooms, vec![LogBloom::new()]);
	}

	#[test]
	fn single_trace_filter_bloom_possibility() {
		let filter = Filter {
			range: (0..0),
			from_address: vec![Address::from(1)],
			to_address: vec![Address::from(2)],
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
			from_address: vec![Address::from(1)],
			to_address: vec![],
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
			from_address: vec![],
			to_address: vec![Address::from(1)],
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
			from_address: vec![Address::from(1), Address::from(3)],
			to_address: vec![Address::from(2), Address::from(4)],
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
}
