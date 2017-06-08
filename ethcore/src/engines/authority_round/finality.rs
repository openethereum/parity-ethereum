// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Finality proof generation and checking.

use std::collections::{HashMap, HashSet, VecDeque};
use util::{Address, H256};
use header::Header;

/// Error indicating unknown validator.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UnknownValidator;

/// Rolling finality checker for authority round consensus.
pub struct RollingFinality {
	headers: VecDeque<(H256, Address)>,
	signers: HashSet<Address>,
	sign_count: HashMap<Address, usize>,
}

impl RollingFinality {
	/// Create a blank finality checker under the given validator set.
	pub fn blank(signers: HashSet<Address>) -> Self {
		RollingFinality {
			headers: VecDeque::new(),
			signers: signers,
			sign_count: HashMap::new(),
		}
	}

	/// Create a rolling finality checker from an iterator of hash, signer pairs in reverse.
	/// This will take as many items from the iterator until a finalized block is found.
	///
	/// Fails if any provided signature isn't part of the set.
	pub fn from_ancestry<I>(signers: HashSet<Address>, iterable: I) -> Result<Self, UnknownValidator>
		where I: IntoIterator<Item=(H256, Address)>
	{
		let mut checker = RollingFinality::blank(signers);

		for (hash, signer) in iterable {
			if !checker.signers.contains(&signer) { return Err(UnknownValidator) }

			checker.headers.push_front((hash, signer));
			*checker.sign_count.entry(signer).or_insert(0) += 1;

			// break when we've got our first finalized block.
			if checker.sign_count.len() * 2 > checker.signers.len() { break }
		}

		Ok(checker)
	}

	/// Returns the last pushed hash.
	pub fn subchain_head(&self) -> Option<H256> {
		self.headers.back().map(|&(h, _)| h)
	}

	/// Get an iterator over stored hashes in order.
	pub fn iter(&self) -> Iter { Iter(self.headers.iter()) }

	/// Push a hash onto the rolling finality checker (implying `subchain_head` == head.parent)
	///
	/// Fails if `signer` isn't a member of the active validator set.
	/// Returns a list of all newly finalized headers.
	// TODO: optimize with smallvec.
	pub fn push_hash(&mut self, head: H256, signer: Address) -> Result<Vec<H256>, UnknownValidator> {
		use std::collections::hash_map::Entry;

		if !self.signers.contains(&signer) { return Err(UnknownValidator) }

		self.headers.push_back((head, signer));
		*self.sign_count.entry(signer).or_insert(0) += 1;

		let mut newly_finalized = Vec::new();

		while self.sign_count.len() * 2 > self.signers.len() {
			let (hash, signer) = self.headers.pop_front()
				.expect("headers length always greater than sign count length; qed");

			newly_finalized.push(hash);

			match self.sign_count.entry(signer) {
				Entry::Occupied(mut entry) => {
					// decrement count for this signer and purge on zero.
					*entry.get_mut() -= 1;

					if *entry.get() == 0 {
						entry.remove();
					}
				}
				Entry::Vacant(_) => panic!("all hashes in `header` should have an entry in `sign_count` for their signer; qed"),
			}
		}

		Ok(newly_finalized)
	}
}

pub struct Iter<'a>(::std::collections::vec_deque::Iter<'a, (H256, Address)>);

impl<'a> Iterator for Iter<'a> {
	type Item = H256;

	fn next(&mut self) -> Option<H256> {
		self.0.next().map(|&(h, _)| h)
	}
}

#[cfg(test)]
mod tests {
	use util::Address;
	use super::RollingFinality;

	#[test]
	fn rejects_unknown_signer() {

	}
}
