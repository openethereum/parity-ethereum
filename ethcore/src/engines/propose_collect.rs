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
use common::{HashSet, RwLock, H256, Signature, Address, Error, ec, Hashable};
use engines::VoteError;

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
	votes: RwLock<HashSet<Signature>>,
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
	pub fn vote(&self, signature: &Signature) -> bool {
		if self.votes.try_read().unwrap().contains(signature) { return false; }
		if !self.can_vote(signature).is_ok() { return false; }
		self.votes.try_write().unwrap().insert(signature.clone());
		true
	}

	fn can_vote(&self, signature: &Signature) -> Result<(), Error> {
		let signer = Address::from(try!(ec::recover(&signature, &self.hash)).sha3());
		match self.voters.contains(&signer) {
			false => try!(Err(VoteError::UnauthorisedVoter)),
			true => Ok(()),
		}
	}

	/// Some winner if voting threshold was reached.
	pub fn winner(&self) -> Option<H256> {
		let threshold_checker = || match self.votes.try_read().unwrap().len() >= self.threshold {
			true => { self.is_won.store(true, Ordering::Relaxed); true },
			false => false,
		};
		match self.is_won.load(Ordering::Relaxed) || threshold_checker() {
			true => Some(self.hash),
			false => None,
		}
	}

	/// Get signatures backing given hash.
	pub fn votes(&self) -> HashSet<Signature> {
		self.votes.try_read().unwrap().clone()
	}
}

#[cfg(test)]	
mod tests {
	use common::*;
	use engines::propose_collect::ProposeCollect;
	use account_provider::AccountProvider;

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
		let voters: HashSet<_> = vec![addr1, addr2].into_iter().map(Into::into).collect();
		let vote = ProposeCollect::new(bare_hash, voters.into(), 1);
		assert!(vote.winner().is_none());

		// Unapproved voter.
		let signature = tap.sign(addr3, bare_hash).unwrap();
		assert!(!vote.vote(&signature.into()));
		assert!(vote.winner().is_none());
		// First good vote.
		let signature = tap.sign(addr1, bare_hash).unwrap();
		assert!(vote.vote(&signature.into()));
		assert_eq!(vote.winner().unwrap(), bare_hash);
		// Voting again is ineffective.
		let signature = tap.sign(addr1, bare_hash).unwrap();
		assert!(!vote.vote(&signature.into()));
		// Second valid vote.
		let signature = tap.sign(addr2, bare_hash).unwrap();
		assert!(vote.vote(&signature.into()));
		assert_eq!(vote.winner().unwrap(), bare_hash);
	}
}
