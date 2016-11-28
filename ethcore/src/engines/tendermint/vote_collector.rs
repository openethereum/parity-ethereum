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

//! Collects votes on hashes at each height and round.

use util::*;
use super::message::ConsensusMessage;
use super::{Height, Round, Step};

#[derive(Debug)]
pub struct VoteCollector {
	/// Storing all Proposals, Prevotes and Precommits.
	votes: RwLock<BTreeMap<ConsensusMessage, Address>>
}

#[derive(Debug, PartialEq, Eq)]
pub struct SealSignatures {
	pub proposal: H520,
	pub votes: Vec<H520>
}

impl VoteCollector {
	pub fn new() -> VoteCollector {
		VoteCollector { votes: RwLock::new(BTreeMap::new()) }
	}

	pub fn vote(&self, message: ConsensusMessage, voter: Address) -> Option<Address> {
		self.votes.write().insert(message, voter)
	}

	pub fn seal_signatures(&self, height: Height, round: Round, block_hash: Option<H256>) -> Option<SealSignatures> {
		let guard = self.votes.read();
		let mut current_signatures = guard.keys()
			.skip_while(|m| !m.is_block_hash(height, round, Step::Propose, block_hash));
		current_signatures.next().map(|proposal| SealSignatures {
			proposal: proposal.signature,
			votes: current_signatures
				.skip_while(|m| !m.is_block_hash(height, round, Step::Precommit, block_hash))
				.filter(|m| m.is_block_hash(height, round, Step::Precommit, block_hash))
				.map(|m| m.signature.clone())
				.collect()
		})
	}

	pub fn count_aligned_votes(&self, message: &ConsensusMessage) -> usize {
		let guard = self.votes.read();
		guard.keys()
			.skip_while(|m| !m.is_aligned(message))
			// sorted by signature so might not be continuous
			.filter(|m| m.is_aligned(message))
			.count()
	}

	pub fn count_step_votes(&self, height: Height, round: Round, step: Step) -> usize {
		self.votes
			.read()
			.keys()
			.skip_while(|m| !m.is_step(height, round, step))
			.take_while(|m| m.is_step(height, round, step))
			.count()	
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use super::*;
	use super::super::{Height, Round, BlockHash, Step};
	use super::super::message::ConsensusMessage;

	fn simple_vote(collector: &VoteCollector, signature: H520, h: Height, r: Round, step: Step, block_hash: Option<BlockHash>) -> Option<H160> {
		collector.vote(ConsensusMessage { signature: signature, height: h, round: r, step: step, block_hash: block_hash }, H160::default())
	}

	#[test]
	fn seal_retrieval() {
		let collector = VoteCollector::new();	
		let bh = Some("1".sha3());
		let h = 1;
		let r = 2;
		let mut signatures = Vec::new();
		for _ in 0..5 {
			signatures.push(H520::random());
		}
		// Wrong height proposal.
		simple_vote(&collector, signatures[4].clone(), h - 1, r, Step::Propose, bh.clone());
		// Good proposal.
		simple_vote(&collector, signatures[0].clone(), h, r, Step::Propose, bh.clone());
		// Wrong block proposal.
		simple_vote(&collector, signatures[0].clone(), h, r, Step::Propose, Some("0".sha3()));
		// Wrong block precommit.
		simple_vote(&collector, signatures[3].clone(), h, r, Step::Precommit, Some("0".sha3()));
		// Wrong round proposal.
		simple_vote(&collector, signatures[0].clone(), h, r - 1, Step::Propose, bh.clone());
		// Prevote.
		simple_vote(&collector, signatures[0].clone(), h, r, Step::Prevote, bh.clone());
		// Relevant precommit.
		simple_vote(&collector, signatures[2].clone(), h, r, Step::Precommit, bh.clone());
		// Replcated vote.
		simple_vote(&collector, signatures[2].clone(), h, r, Step::Precommit, bh.clone());
		// Wrong round precommit.
		simple_vote(&collector, signatures[4].clone(), h, r + 1, Step::Precommit, bh.clone());
		// Wrong height precommit.
		simple_vote(&collector, signatures[3].clone(), h + 1, r, Step::Precommit, bh.clone());
		// Relevant precommit.
		simple_vote(&collector, signatures[1].clone(), h, r, Step::Precommit, bh.clone());
		// Wrong round precommit, same signature.
		simple_vote(&collector, signatures[1].clone(), h, r + 1, Step::Precommit, bh.clone());
		// Wrong round precommit.
		simple_vote(&collector, signatures[4].clone(), h, r - 1, Step::Precommit, bh.clone());
		let seal = SealSignatures {
			proposal: signatures[0],
			votes: signatures[1..3].to_vec()
		};
		assert_eq!(seal, collector.seal_signatures(h, r, bh).unwrap());
	}

	#[test]
	fn count_votes() {
		let collector = VoteCollector::new();	
		// good prevote
		simple_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("0".sha3()));
		simple_vote(&collector, H520::random(), 3, 1, Step::Prevote, Some("0".sha3()));
		// good precommit
		simple_vote(&collector, H520::random(), 3, 2, Step::Precommit, Some("0".sha3()));
		simple_vote(&collector, H520::random(), 3, 3, Step::Precommit, Some("0".sha3()));
		// good prevote
		simple_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("1".sha3()));
		// good prevote
		let same_sig = H520::random();
		simple_vote(&collector, same_sig.clone(), 3, 2, Step::Prevote, Some("1".sha3()));
		simple_vote(&collector, same_sig, 3, 2, Step::Prevote, Some("1".sha3()));
		// good precommit
		simple_vote(&collector, H520::random(), 3, 2, Step::Precommit, Some("1".sha3()));
		// good prevote
		simple_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("0".sha3()));
		simple_vote(&collector, H520::random(), 2, 2, Step::Precommit, Some("2".sha3()));

		assert_eq!(collector.count_step_votes(3, 2, Step::Prevote), 4);
		assert_eq!(collector.count_step_votes(3, 2, Step::Precommit), 2);

		let message = ConsensusMessage {
			signature: H520::default(),
			height: 3,
			round: 2,
			step: Step::Prevote,
			block_hash: Some("1".sha3())
		};
		assert_eq!(collector.count_aligned_votes(&message), 2);
	}
}
