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

//! Voting on a hash, where each vote has to come from a set of addresses.

use std::sync::atomic::{AtomicBool, Ordering};
use util::{HashSet, RwLock, H256, Address};

/// Collect votes on a hash.
#[derive(Debug)]
pub struct ProposeCollect {
	/// Proposed hash.
	pub hash: H256,
	/// Allowed voter addresses.
	pub voters: HashSet<Address>,
	/// Threshold vote number for success.
	pub threshold: usize,
	/// Votes.
	votes: RwLock<HashSet<Address>>,
	/// Was enough votes reached.
	is_won: AtomicBool
}

impl ProposeCollect {
	/// Create a new instance of BFT engine
	pub fn new(hash: H256, voters: HashSet<Address>, threshold: usize) -> Self {
		assert!(voters.len() > threshold);
		ProposeCollect {
			hash: hash,
			voters: voters,
			threshold: threshold,
			votes: RwLock::new(HashSet::new()),
			is_won: AtomicBool::new(false)
		}
	}

	/// Vote on hash using the signed hash, true if vote counted.
	pub fn vote(&self, voter: Address) -> bool {
		let is_known = self.votes.try_read().unwrap().contains(&voter);
		if !is_known && self.voters.contains(&voter) {
		 	self.votes.try_write().unwrap().insert(voter);
		 	true
		} else {
			false
		}
	}

	/// Some winner if voting threshold was reached.
	pub fn is_won(&self) -> bool {
		let threshold_checker = || match self.votes.try_read().unwrap().len() >= self.threshold {
			true => { self.is_won.store(true, Ordering::Relaxed); true },
			false => false,
		};
		self.is_won.load(Ordering::Relaxed) || threshold_checker()
	}
}

#[cfg(test)]	
mod tests {
	use engines::propose_collect::ProposeCollect;
	use account_provider::AccountProvider;
	use util::*;
	use header::Header;

	#[test]
	fn simple_propose_collect() {
		let tap = AccountProvider::transient_provider();
		let addr1 = tap.insert_account("1".sha3(), "1").unwrap();
		tap.unlock_account_permanently(addr1, "1".into()).unwrap();

		let addr2 = tap.insert_account("2".sha3(), "2").unwrap();
		tap.unlock_account_permanently(addr2, "2".into()).unwrap();

		let addr3 = tap.insert_account("3".sha3(), "3").unwrap();
		tap.unlock_account_permanently(addr3, "3".into()).unwrap();

		let header = Header::default();
		let bare_hash = header.bare_hash();
		let voters: HashSet<_> = vec![addr1.clone(), addr2.clone(), Address::default()].into_iter().map(Into::into).collect();
		let vote = ProposeCollect::new(bare_hash, voters.into(), 2);
		assert!(!vote.is_won());

		// Unapproved voter.
		assert!(!vote.vote(addr3));
		assert!(!vote.is_won());
		// First good vote.
		assert!(vote.vote(addr1.clone()));
		assert!(!vote.is_won());
		// Voting again is ineffective.
		assert!(!vote.vote(addr1));
		assert!(!vote.is_won());
		// Second valid vote thus win.
		assert!(vote.vote(addr2));
		assert!(vote.is_won());
	}
}
