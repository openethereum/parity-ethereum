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

use std::hash::BuildHasher;
use std::collections::{HashSet, HashMap};

use crate::api::TransactionStats;

use ethereum_types::{H256, H512};
use fastmap::H256FastMap;
use common_types::BlockNumber;

type NodeId = H512;

#[derive(Debug, PartialEq, Clone, MallocSizeOf)]
pub struct Stats {
	first_seen: BlockNumber,
	propagated_to: HashMap<NodeId, usize>,
}

impl Stats {
	pub fn new(number: BlockNumber) -> Self {
		Stats {
			first_seen: number,
			propagated_to: Default::default(),
		}
	}
}

impl<'a> From<&'a Stats> for TransactionStats {
	fn from(other: &'a Stats) -> Self {
		TransactionStats {
			first_seen: other.first_seen,
			propagated_to: other.propagated_to
				.iter()
				.map(|(hash, size)| (*hash, *size))
				.collect(),
		}
	}
}

#[derive(Debug, Default, MallocSizeOf)]
pub struct TransactionsStats {
	pending_transactions: H256FastMap<Stats>,
}

impl TransactionsStats {
	/// Increases number of propagations to given `enodeid`.
	pub fn propagated(&mut self, hash: &H256, enode_id: Option<NodeId>, current_block_num: BlockNumber) {
		let enode_id = enode_id.unwrap_or_default();
		let stats = self.pending_transactions.entry(*hash).or_insert_with(|| Stats::new(current_block_num));
		let count = stats.propagated_to.entry(enode_id).or_insert(0);
		*count = count.saturating_add(1);
	}

	/// Returns propagation stats for given hash or `None` if hash is not known.
	#[cfg(test)]
	pub fn get(&self, hash: &H256) -> Option<&Stats> {
		self.pending_transactions.get(hash)
	}

	pub fn stats(&self) -> &H256FastMap<Stats> {
		&self.pending_transactions
	}

	/// Retains only transactions present in given `HashSet`.
	pub fn retain<S: BuildHasher>(&mut self, hashes: &HashSet<H256, S>) {
		let to_remove = self.pending_transactions.keys()
			.filter(|hash| !hashes.contains(hash))
			.cloned()
			.collect::<Vec<_>>();

		for hash in to_remove {
			self.pending_transactions.remove(&hash);
		}
	}
}

#[cfg(test)]
mod tests {
	use std::collections::{HashMap, HashSet};
	use super::{Stats, TransactionsStats, NodeId, H256};
	use macros::hash_map;

	#[test]
	fn should_keep_track_of_propagations() {
		// given
		let mut stats = TransactionsStats::default();
		let hash = H256::from_low_u64_be(5);
		let enodeid1 = NodeId::from_low_u64_be(2);
		let enodeid2 = NodeId::from_low_u64_be(5);

		// when
		stats.propagated(&hash, Some(enodeid1), 5);
		stats.propagated(&hash, Some(enodeid1), 10);
		stats.propagated(&hash, Some(enodeid2), 15);

		// then
		let stats = stats.get(&hash);
		assert_eq!(stats, Some(&Stats {
			first_seen: 5,
			propagated_to: hash_map![
				enodeid1 => 2,
				enodeid2 => 1
			],
		}));
	}

	#[test]
	fn should_remove_hash_from_tracking() {
		// given
		let mut stats = TransactionsStats::default();
		let hash = H256::from_low_u64_be(5);
		let enodeid1 = NodeId::from_low_u64_be(5);
		stats.propagated(&hash, Some(enodeid1), 10);

		// when
		stats.retain(&HashSet::new());

		// then
		let stats = stats.get(&hash);
		assert_eq!(stats, None);
	}
}
