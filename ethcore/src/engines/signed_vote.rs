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

//! Voting on hashes, where each vote has to come from a set of public keys.

use super::EngineError;
use util::*;
use error::Error;
use ethkey::{Signature, recover};

/// Signed voting on hashes.
#[derive(Debug)]
pub struct SignedVote {
	/// Voter public keys.
	pub voters: HashSet<Address>,
	/// Number of voters.
	pub voter_n: usize,
	/// Threshold vote number for success.
	pub threshold: usize,
	/// Votes.
	votes: RwLock<HashMap<H256, HashSet<Signature>>>,
	/// Winner hash, set after enough votes are reached.
	winner: RwLock<Option<H256>>
}

impl SignedVote {
	/// Create a new instance of BFT engine
	pub fn new(voters: HashSet<Address>, threshold: usize) -> Self {
		let voters_n = voters.len();
		assert!(voters_n > threshold);
		SignedVote {
			voter_n: voters_n,
			voters: voters,
			threshold: threshold,
			votes: RwLock::new(HashMap::new()),
			winner: RwLock::new(None)
		}
	}

	/// Vote on hash using the signed hash, true if vote counted.
	pub fn vote(&self, bare_hash: H256, signature: Signature) -> bool {
		if !self.can_vote(&bare_hash, &signature).is_ok() { return false; }
		let mut guard = self.votes.try_write().unwrap();
		let set = guard.entry(bare_hash.clone()).or_insert_with(|| HashSet::new());
		if !set.insert(signature) { return false; }
		// Set the winner if threshold is reached.
		if set.len() >= self.threshold {
			let mut guard = self.winner.try_write().unwrap();
			*guard = Some(bare_hash);
		}
		true
	}

	fn can_vote(&self, bare_hash: &H256, signature: &Signature) -> Result<(), Error> {
		let signer = Address::from(try!(recover(&signature, bare_hash)).sha3());
		match self.voters.contains(&signer) {
			false => try!(Err(EngineError::UnauthorisedVoter)),
			true => Ok(()),
		}
	}

	/// Some winner if voting threshold was reached.
	pub fn winner(&self) -> Option<H256> { self.winner.try_read().unwrap().clone() }

	/// Get signatures backing given hash.
	pub fn votes(&self, bare_hash: &H256) -> Option<HashSet<Signature>> {
		self.votes.try_read().unwrap().get(bare_hash).cloned()
	}
}

#[cfg(test)]	
mod tests {
	use util::*;
	use header::Header;
	use engines::signed_vote::SignedVote;
	use account_provider::AccountProvider;

	#[test]
	fn simple_vote() {
		let tap = AccountProvider::transient_provider();
		let addr1 = tap.insert_account("1".sha3(), "1").unwrap();
		tap.unlock_account_permanently(addr1, "1".into()).unwrap();

		let addr2 = tap.insert_account("2".sha3(), "2").unwrap();
		tap.unlock_account_permanently(addr2, "2".into()).unwrap();

		let addr3 = tap.insert_account("3".sha3(), "3").unwrap();
		tap.unlock_account_permanently(addr3, "3".into()).unwrap();

		let voters: HashSet<_> = vec![addr1, addr2].into_iter().map(Into::into).collect();
		let vote = SignedVote::new(voters.into(), 1);
		assert!(vote.winner().is_none());
		let header = Header::default();
		let bare_hash = header.bare_hash();

		// Unapproved voter.
		let signature = tap.sign(addr3, None, bare_hash).unwrap();
		assert!(!vote.vote(bare_hash, signature));
		assert!(vote.winner().is_none());
		// First good vote.
		let signature = tap.sign(addr1, None, bare_hash).unwrap();
		assert!(vote.vote(bare_hash, signature));
		assert_eq!(vote.winner().unwrap(), bare_hash);
		// Voting again is ineffective.
		let signature = tap.sign(addr1, None, bare_hash).unwrap();
		assert!(!vote.vote(bare_hash, signature));
		// Second valid vote.
		let signature = tap.sign(addr2, None, bare_hash).unwrap();
		assert!(vote.vote(bare_hash, signature));
		assert_eq!(vote.winner().unwrap(), bare_hash);
	}
}
