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

use common::*;
use account_provider::AccountProvider;
use block::*;
use spec::CommonParams;
use engines::Engine;
use evm::Schedule;
use ethjson;

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

#[derive(Debug)]
pub enum VoteError {
	UnauthorisedVoter
}

impl SignedVote {
	/// Create a new instance of BFT engine
	pub fn new(voters: HashSet<Address>, threshold: usize) -> Self {
		SignedVote {
			voter_n: voters.len(),
			voters: voters,
			threshold: threshold,
			votes: RwLock::new(HashMap::new()),
			winner: RwLock::new(None)
		}
	}

	pub fn vote(&self, bare_hash: H256, signature: &Signature) -> bool {
		if !self.can_vote(&bare_hash, signature).is_ok() { return false; }
		let n = if let Some(mut old) = self.votes.write().get_mut(&bare_hash) {
			old.insert(signature.clone());
			old.len()
		} else {
			let mut new = HashSet::new();
			new.insert(signature.clone());
			assert!(self.votes.write().insert(bare_hash.clone(), new).is_none());
			1
		};
		if self.is_won(n) {
			let mut guard = self.winner.write();
			*guard = Some(bare_hash);
		}
		true
	}

	fn can_vote(&self, bare_hash: &H256, signature: &Signature) -> Result<(), Error> {
		let signer = Address::from(try!(ec::recover(&signature, bare_hash)).sha3());
		match self.voters.contains(&signer) {
			false => try!(Err(VoteError::UnauthorisedVoter)),
			true => Ok(()),
		}
	}

	fn is_won(&self, valid_votes: usize) -> bool {
		valid_votes > self.threshold
	}

	pub fn winner(&self) -> Option<H256> { self.winner.read().clone() }
}

#[cfg(test)]	
mod tests {
	use common::{HashSet, Address};
	use engines::signed_vote::SignedVote;
	#[test]
	fn simple_vote() {
		let voters: HashSet<_> = vec![Address::default()].into_iter().collect();
		let vote = SignedVote::new(voters, 2);
	}
}
