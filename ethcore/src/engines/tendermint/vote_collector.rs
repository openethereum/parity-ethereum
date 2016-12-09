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
	votes: RwLock<BTreeMap<ConsensusMessage, Address>>,
}

#[derive(Debug)]
pub struct SealSignatures {
	pub proposal: H520,
	pub votes: Vec<H520>,
}

impl PartialEq for SealSignatures {
	fn eq(&self, other: &SealSignatures) -> bool {
		self.proposal == other.proposal
			&& self.votes.iter().collect::<HashSet<_>>() == other.votes.iter().collect::<HashSet<_>>()
	}
}

impl Eq for SealSignatures {}

impl VoteCollector {
	pub fn new() -> VoteCollector {
		let mut collector = BTreeMap::new();
		// Insert dummy message to fulfill invariant: "only messages newer than the oldest are inserted".
		collector.insert(ConsensusMessage {
			signature: H520::default(),
			height: 0,
			round: 0,
			step: Step::Propose,
			block_hash: None
		},
		Address::default());
		VoteCollector { votes: RwLock::new(collector) }
	}

	/// Insert vote if it is newer than the oldest one.
	pub fn vote(&self, message: ConsensusMessage, voter: Address) -> Option<Address> {
		self.votes.write().insert(message, voter)
	}

	pub fn is_old_or_known(&self, message: &ConsensusMessage) -> bool {
		self.votes.read().contains_key(message)
			|| {
				let guard = self.votes.read();
				guard.keys().next().map_or(true, |oldest| message <= oldest)
			}
	}

	/// Throws out messages older than message, leaves message as marker for the oldest.
	pub fn throw_out_old(&self, message: &ConsensusMessage) {
		let mut guard = self.votes.write();
		let new_collector = guard.split_off(message);
		*guard = new_collector;
	}

	pub fn seal_signatures(&self, height: Height, round: Round, block_hash: H256) -> Option<SealSignatures> {
		let bh = Some(block_hash);
		let (proposal, votes) = {
			let guard = self.votes.read();
			let mut current_signatures = guard.keys().skip_while(|m| !m.is_block_hash(height, round, Step::Propose, bh));
			let proposal = current_signatures.next().cloned();
			let votes = current_signatures
					.skip_while(|m| !m.is_block_hash(height, round, Step::Precommit, bh))
					.filter(|m| m.is_block_hash(height, round, Step::Precommit, bh))
					.cloned()
					.collect::<Vec<_>>();
			(proposal, votes)
		};
		if votes.is_empty() {
			return None;
		}
		// Remove messages that are no longer relevant.
		votes.last().map(|m| self.throw_out_old(m));
		let mut votes_vec: Vec<_> = votes.into_iter().map(|m| m.signature).collect();
		votes_vec.sort();
		proposal.map(|p| SealSignatures {
			proposal: p.signature,
			votes: votes_vec,
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
		let guard = self.votes.read();
		let current = guard.iter().skip_while(|&(m, _)| !m.is_step(height, round, step));
		let mut origins = HashSet::new();
		let mut n = 0;
		for (message, origin) in current {
			if message.is_step(height, round, step) {
				if origins.insert(origin) {
					n += 1;
				} else {
					warn!("count_step_votes: Authority {} has cast multiple step votes, this indicates malicious behaviour.", origin)
				}
			}
		}
		n
	}

	pub fn get_up_to(&self, height: Height) -> Vec<Bytes> {
		let guard = self.votes.read();
		guard
			.keys()
			.filter(|m| m.step.is_pre())
			.take_while(|m| m.height <= height)
			.map(|m| ::rlp::encode(m).to_vec())
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use super::*;
	use super::super::{Height, Round, BlockHash, Step};
	use super::super::message::ConsensusMessage;

	fn random_vote(collector: &VoteCollector, signature: H520, h: Height, r: Round, step: Step, block_hash: Option<BlockHash>) -> Option<H160> {
		full_vote(collector, signature, h, r, step, block_hash, H160::random())
	}

	fn full_vote(collector: &VoteCollector, signature: H520, h: Height, r: Round, step: Step, block_hash: Option<BlockHash>, address: Address) -> Option<H160> {
		collector.vote(ConsensusMessage { signature: signature, height: h, round: r, step: step, block_hash: block_hash }, address)
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
		random_vote(&collector, signatures[4].clone(), h - 1, r, Step::Propose, bh.clone());
		// Good proposal.
		random_vote(&collector, signatures[0].clone(), h, r, Step::Propose, bh.clone());
		// Wrong block proposal.
		random_vote(&collector, signatures[0].clone(), h, r, Step::Propose, Some("0".sha3()));
		// Wrong block precommit.
		random_vote(&collector, signatures[3].clone(), h, r, Step::Precommit, Some("0".sha3()));
		// Wrong round proposal.
		random_vote(&collector, signatures[0].clone(), h, r - 1, Step::Propose, bh.clone());
		// Prevote.
		random_vote(&collector, signatures[0].clone(), h, r, Step::Prevote, bh.clone());
		// Relevant precommit.
		random_vote(&collector, signatures[2].clone(), h, r, Step::Precommit, bh.clone());
		// Replcated vote.
		random_vote(&collector, signatures[2].clone(), h, r, Step::Precommit, bh.clone());
		// Wrong round precommit.
		random_vote(&collector, signatures[4].clone(), h, r + 1, Step::Precommit, bh.clone());
		// Wrong height precommit.
		random_vote(&collector, signatures[3].clone(), h + 1, r, Step::Precommit, bh.clone());
		// Relevant precommit.
		random_vote(&collector, signatures[1].clone(), h, r, Step::Precommit, bh.clone());
		// Wrong round precommit, same signature.
		random_vote(&collector, signatures[1].clone(), h, r + 1, Step::Precommit, bh.clone());
		// Wrong round precommit.
		random_vote(&collector, signatures[4].clone(), h, r - 1, Step::Precommit, bh.clone());
		let seal = SealSignatures {
			proposal: signatures[0],
			votes: signatures[1..3].to_vec()
		};
		assert_eq!(seal, collector.seal_signatures(h, r, bh.unwrap()).unwrap());
	}

	#[test]
	fn count_votes() {
		let collector = VoteCollector::new();	
		// good prevote
		random_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("0".sha3()));
		random_vote(&collector, H520::random(), 3, 1, Step::Prevote, Some("0".sha3()));
		// good precommit
		random_vote(&collector, H520::random(), 3, 2, Step::Precommit, Some("0".sha3()));
		random_vote(&collector, H520::random(), 3, 3, Step::Precommit, Some("0".sha3()));
		// good prevote
		random_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("1".sha3()));
		// good prevote
		let same_sig = H520::random();
		random_vote(&collector, same_sig.clone(), 3, 2, Step::Prevote, Some("1".sha3()));
		random_vote(&collector, same_sig, 3, 2, Step::Prevote, Some("1".sha3()));
		// good precommit
		random_vote(&collector, H520::random(), 3, 2, Step::Precommit, Some("1".sha3()));
		// good prevote
		random_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("0".sha3()));
		random_vote(&collector, H520::random(), 2, 2, Step::Precommit, Some("2".sha3()));

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

	#[test]
	fn remove_old() {
		let collector = VoteCollector::new();	
		random_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("0".sha3()));
		random_vote(&collector, H520::random(), 3, 1, Step::Prevote, Some("0".sha3()));
		random_vote(&collector, H520::random(), 3, 3, Step::Precommit, Some("0".sha3()));
		random_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("1".sha3()));
		random_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("1".sha3()));
		random_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("0".sha3()));
		random_vote(&collector, H520::random(), 2, 2, Step::Precommit, Some("2".sha3()));

		let message = ConsensusMessage {
			signature: H520::default(),
			height: 3,
			round: 2,
			step: Step::Precommit,
			block_hash: Some("1".sha3())
		};
		collector.throw_out_old(&message);
		assert_eq!(collector.votes.read().len(), 1);
	}

	#[test]
	fn malicious_authority() {
		let collector = VoteCollector::new();	
		full_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("0".sha3()), Address::default());
		full_vote(&collector, H520::random(), 3, 2, Step::Prevote, Some("1".sha3()), Address::default());
		assert_eq!(collector.count_step_votes(3, 2, Step::Prevote), 1);
	}
}
