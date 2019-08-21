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

//! Finality proof generation and checking.

use std::collections::{VecDeque};
use std::collections::hash_map::{HashMap, Entry};

use ethereum_types::{H256, Address};
use types::BlockNumber;

use engines::validator_set::SimpleList;

/// Error indicating unknown validator.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UnknownValidator;

/// Rolling finality checker for authority round consensus.
/// Stores a chain of unfinalized hashes that can be pushed onto.
pub struct RollingFinality {
	headers: VecDeque<(H256, BlockNumber, Vec<Address>)>,
	signers: SimpleList,
	sign_count: HashMap<Address, usize>,
	last_pushed: Option<H256>,
	/// First block for which a 2/3 quorum (instead of 1/2) is required.
	two_thirds_majority_transition: BlockNumber,
}

impl RollingFinality {
	/// Create a blank finality checker under the given validator set.
	pub fn blank(signers: Vec<Address>, two_thirds_majority_transition: BlockNumber) -> Self {
		trace!(target: "finality", "Instantiating blank RollingFinality with {} signers: {:?}", signers.len(), signers);
		RollingFinality {
			headers: VecDeque::new(),
			signers: SimpleList::new(signers),
			sign_count: HashMap::new(),
			last_pushed: None,
			two_thirds_majority_transition,
		}
	}

	/// Extract unfinalized subchain from ancestry iterator.
	/// Clears the current subchain.
	///
	/// Fails if any provided signature isn't part of the signers set.
	pub fn build_ancestry_subchain<I>(&mut self, iterable: I) -> Result<(), UnknownValidator>
		where I: IntoIterator<Item=(H256, BlockNumber, Vec<Address>)>,
	{
		self.clear();
		for (hash, number, signers) in iterable {
			if signers.iter().any(|s| !self.signers.contains(s)) { return Err(UnknownValidator) }
			if self.last_pushed.is_none() { self.last_pushed = Some(hash) }
			self.add_signers(&signers);
			self.headers.push_front((hash, number, signers));
			// break when we've got our first finalized block.
			if self.is_finalized() {
				let (hash, _, signers) = self.headers.pop_front().expect("we just pushed a block; qed");
				self.remove_signers(&signers);
				trace!(target: "finality", "Encountered already finalized block {}", hash);
				break
			}
		}

		trace!(target: "finality", "Rolling finality state: {:?}", self.headers);
		Ok(())
	}

	/// Clears the finality status, but keeps the validator set.
	pub fn clear(&mut self) {
		self.headers.clear();
		self.sign_count.clear();
		self.last_pushed = None;
	}

	/// Returns the last pushed hash.
	pub fn subchain_head(&self) -> Option<H256> {
		self.last_pushed
	}

	/// Get an iterator over stored hashes in order.
	#[cfg(test)]
	pub fn unfinalized_hashes(&self) -> impl Iterator<Item=&H256> {
		self.headers.iter().map(|(h, _, _)| h)
	}

	/// Get the validator set.
	pub fn validators(&self) -> &SimpleList { &self.signers }

	/// Push a hash onto the rolling finality checker (implying `subchain_head` == head.parent)
	///
	/// Fails if `signer` isn't a member of the active validator set.
	/// Returns a list of all newly finalized headers.
	// TODO: optimize with smallvec.
	pub fn push_hash(&mut self, head: H256, number: BlockNumber, signers: Vec<Address>)
		-> Result<Vec<H256>, UnknownValidator>
	{
		for their_signer in signers.iter() {
			if !self.signers.contains(their_signer) {
				warn!(target: "finality",  "Unknown validator: {}", their_signer);
				return Err(UnknownValidator)
			}
		}

		self.add_signers(&signers);
		self.headers.push_back((head, number, signers));

		let mut newly_finalized = Vec::new();

		while self.is_finalized() {
			let (hash, _, signers) = self.headers.pop_front()
				.expect("headers length always greater than sign count length; qed");
			self.remove_signers(&signers);
			newly_finalized.push(hash);
		}

		trace!(target: "finality", "{} Blocks finalized by {:?}: {:?}", newly_finalized.len(), head, newly_finalized);

		self.last_pushed = Some(head);
		Ok(newly_finalized)
	}

	/// Returns the first block for which a 2/3 quorum (instead of 1/2) is required.
	pub fn two_thirds_majority_transition(&self) -> BlockNumber {
		self.two_thirds_majority_transition
	}

	/// Returns whether the first entry in `self.headers` is finalized.
	fn is_finalized(&self) -> bool {
		match self.headers.front() {
			None => false,
			Some((_, number, _)) if *number < self.two_thirds_majority_transition => {
				self.sign_count.len() * 2 > self.signers.len()
			}
			Some((_, _, _)) => {
				self.sign_count.len() * 3 > self.signers.len() * 2
			}
		}
	}

	/// Adds the signers to the sign count.
	fn add_signers(&mut self, signers: &[Address]) {
		for signer in signers {
			*self.sign_count.entry(*signer).or_insert(0) += 1;
		}
	}

	/// Removes the signers from the sign count.
	fn remove_signers(&mut self, signers: &[Address]) {
		for signer in signers {
			match self.sign_count.entry(*signer) {
				Entry::Occupied(mut entry) => {
					// decrement count for this signer and purge on zero.
					if *entry.get() <= 1 {
						entry.remove();
					} else {
						*entry.get_mut() -= 1;
					}
				}
				Entry::Vacant(_) => {
					panic!("all hashes in `header` should have entries in `sign_count` for their signers; qed");
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use ethereum_types::{H256, Address};
	use types::BlockNumber;
	use super::RollingFinality;

	#[test]
	fn rejects_unknown_signers() {
		let signers = (0..3).map(|_| Address::random()).collect::<Vec<_>>();
		let mut finality = RollingFinality::blank(signers.clone(), BlockNumber::max_value());
		assert!(finality.push_hash(H256::random(), 0, vec![signers[0], Address::random()]).is_err());
	}

	#[test]
	fn finalize_multiple() {
		let signers: Vec<_> = (0..6).map(|_| Address::random()).collect();

		let mut finality = RollingFinality::blank(signers.clone(), BlockNumber::max_value());
		let hashes: Vec<_> = (0..7).map(|_| H256::random()).collect();

		// 3 / 6 signers is < 51% so no finality.
		for (i, hash) in hashes.iter().take(6).cloned().enumerate() {
			let i = i % 3;
			assert!(finality.push_hash(hash, i as u64, vec![signers[i]]).unwrap().len() == 0);
		}

		// after pushing a block signed by a fourth validator, the first four
		// blocks of the unverified chain become verified.
		assert_eq!(finality.push_hash(hashes[6], 6, vec![signers[4]]).unwrap(),
			vec![hashes[0], hashes[1], hashes[2], hashes[3]]);
	}

	#[test]
	fn finalize_multiple_signers() {
		let signers: Vec<_> = (0..6).map(|_| Address::random()).collect();
		let mut finality = RollingFinality::blank(signers.clone(), BlockNumber::max_value());
		let hash = H256::random();

		// after pushing a block signed by four validators, it becomes verified right away.
		assert_eq!(finality.push_hash(hash, 0, signers[0..4].to_vec()).unwrap(), vec![hash]);
	}

	#[test]
	fn from_ancestry() {
		let signers: Vec<_> = (0..6).map(|_| Address::random()).collect();
		let hashes: Vec<_> = (0..12).map(|i| (H256::random(), i as u64, vec![signers[i % 6]])).collect();

		let mut finality = RollingFinality::blank(signers.clone(), BlockNumber::max_value());
		finality.build_ancestry_subchain(hashes.iter().rev().cloned()).unwrap();

		assert_eq!(finality.unfinalized_hashes().count(), 3);
		assert_eq!(finality.subchain_head(), Some(hashes[11].0));
	}

	#[test]
	fn from_ancestry_multiple_signers() {
		let signers: Vec<_> = (0..6).map(|_| Address::random()).collect();
		let hashes: Vec<_> = (0..12).map(|i| {
			(H256::random(), i as u64, vec![signers[i % 6], signers[(i + 1) % 6], signers[(i + 2) % 6]])
		}).collect();

		let mut finality = RollingFinality::blank(signers.clone(), BlockNumber::max_value());
		finality.build_ancestry_subchain(hashes.iter().rev().cloned()).unwrap();

		// only the last hash has < 51% of authorities' signatures
		assert_eq!(finality.unfinalized_hashes().count(), 1);
		assert_eq!(finality.unfinalized_hashes().next(), Some(&hashes[11].0));
		assert_eq!(finality.subchain_head(), Some(hashes[11].0));
	}

	#[test]
	fn rejects_unknown_signers_2_3() {
		let signers = (0..3).map(|_| Address::random()).collect::<Vec<_>>();
		let mut finality = RollingFinality::blank(signers.clone(), 0);
		assert!(finality.push_hash(H256::random(), 0, vec![signers[0], Address::random()]).is_err());
	}

	#[test]
	fn finalize_multiple_2_3() {
		let signers: Vec<_> = (0..7).map(|_| Address::random()).collect();

		let mut finality = RollingFinality::blank(signers.clone(), 0);
		let hashes: Vec<_> = (0..9).map(|_| H256::random()).collect();

		// 4 / 7 signers is < 67% so no finality.
		for (i, hash) in hashes.iter().take(8).cloned().enumerate() {
			let i = i % 4;
			assert!(finality.push_hash(hash, i as u64, vec![signers[i]]).unwrap().len() == 0);
		}

		// after pushing a block signed by a fifth validator, the first five
		// blocks of the unverified chain become verified.
		assert_eq!(finality.push_hash(hashes[8], 8, vec![signers[4]]).unwrap(),
			vec![hashes[0], hashes[1], hashes[2], hashes[3], hashes[4]]);
	}

	#[test]
	fn finalize_multiple_signers_2_3() {
		let signers: Vec<_> = (0..5).map(|_| Address::random()).collect();
		let mut finality = RollingFinality::blank(signers.clone(), 0);
		let hash = H256::random();

		// after pushing a block signed by four validators, it becomes verified right away.
		assert_eq!(finality.push_hash(hash, 0, signers[0..4].to_vec()).unwrap(), vec![hash]);
	}

	#[test]
	fn from_ancestry_2_3() {
		let signers: Vec<_> = (0..6).map(|_| Address::random()).collect();
		let hashes: Vec<_> = (0..12).map(|i| (H256::random(), i as u64, vec![signers[i % 6]])).collect();

		let mut finality = RollingFinality::blank(signers, 0);
		finality.build_ancestry_subchain(hashes.iter().rev().cloned()).unwrap();

		// The last four hashes, with index 11, 10, 9, and 8, have been pushed. 7 would have finalized a block.
		assert_eq!(finality.unfinalized_hashes().count(), 4);
		assert_eq!(finality.subchain_head(), Some(hashes[11].0));
	}

	#[test]
	fn from_ancestry_multiple_signers_2_3() {
		let signers: Vec<_> = (0..6).map(|_| Address::random()).collect();
		let hashes: Vec<_> = (0..12).map(|i| {
			let hash_signers = signers.iter().cycle().skip(i).take(4).cloned().collect();
			(H256::random(), i as u64, hash_signers)
		}).collect();

		let mut finality = RollingFinality::blank(signers.clone(), 0);
		finality.build_ancestry_subchain(hashes.iter().rev().cloned()).unwrap();

		// only the last hash has < 67% of authorities' signatures
		assert_eq!(finality.unfinalized_hashes().count(), 1);
		assert_eq!(finality.unfinalized_hashes().next(), Some(&hashes[11].0));
		assert_eq!(finality.subchain_head(), Some(hashes[11].0));
	}
}
