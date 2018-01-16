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

//! Collects votes on hashes at each Message::Round.

use std::fmt::Debug;
use std::collections::{BTreeMap, HashSet, HashMap};
use std::hash::Hash;
use ethereum_types::{H256, H520, Address};
use parking_lot:: RwLock;
use bytes::Bytes;
use rlp::{Encodable, RlpStream};

pub trait Message: Clone + PartialEq + Eq + Hash + Encodable + Debug {
	type Round: Clone + PartialEq + Eq + Hash + Default + Debug + Ord;

	fn signature(&self) -> H520;

	fn block_hash(&self) -> Option<H256>;

	fn round(&self) -> &Self::Round;

	fn is_broadcastable(&self) -> bool;
}

/// Storing all Proposals, Prevotes and Precommits.
#[derive(Debug)]
pub struct VoteCollector<M: Message> {
	votes: RwLock<BTreeMap<M::Round, StepCollector<M>>>,
}

#[derive(Debug, Default)]
struct StepCollector<M: Message> {
	voted: HashMap<Address, M>,
	pub block_votes: HashMap<Option<H256>, HashMap<H520, Address>>,
	messages: HashSet<M>,
}

#[derive(Debug)]
pub struct DoubleVote<'a, M: Message> {
	pub author: &'a Address,
	vote_one: M,
	vote_two: M,
}

impl<'a, M: Message> Encodable for DoubleVote<'a, M> {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2)
			.append(&self.vote_one)
			.append(&self.vote_two);
	}
}

impl <M: Message> StepCollector<M> {
	/// Returns Some(&Address) when validator is double voting.
	fn insert<'a>(&mut self, message: M, address: &'a Address) -> Option<DoubleVote<'a, M>> {
		// Do nothing when message was seen.
		if self.messages.insert(message.clone()) {
			if let Some(previous) = self.voted.insert(address.clone(), message.clone()) {
				// Bad validator sent a different message.
				return Some(DoubleVote {
					author: address,
					vote_one: previous,
					vote_two: message
				});
			} else {
				self
					.block_votes
					.entry(message.block_hash())
					.or_insert_with(HashMap::new)
					.insert(message.signature(), address.clone());
			}
		}
		None
	}

	/// Count all votes for the given block hash at this round.
	fn count_block(&self, block_hash: &Option<H256>) -> usize {
		self.block_votes.get(block_hash).map_or(0, HashMap::len)
	}

	/// Count all votes collected for the given round.
	fn count(&self) -> usize {
		self.block_votes.values().map(HashMap::len).sum()
	}
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

impl <M: Message + Default> Default for VoteCollector<M> {
	fn default() -> Self {
		let mut collector = BTreeMap::new();
		// Insert dummy entry to fulfill invariant: "only messages newer than the oldest are inserted".
		collector.insert(Default::default(), Default::default());
		VoteCollector { votes: RwLock::new(collector) }
	}
}

impl <M: Message + Default + Encodable + Debug> VoteCollector<M> {
	/// Insert vote if it is newer than the oldest one.
	pub fn vote<'a>(&self, message: M, voter: &'a Address) -> Option<DoubleVote<'a, M>> {
		self
			.votes
			.write()
			.entry(message.round().clone())
			.or_insert_with(Default::default)
			.insert(message, voter)
	}

	/// Checks if the message should be ignored.
	pub fn is_old_or_known(&self, message: &M) -> bool {
		self
			.votes
			.read()
			.get(&message.round())
			.map_or(false, |c| {
				let is_known = c.messages.contains(message);
				if is_known { trace!(target: "engine", "Known message: {:?}.", message); }
				is_known
			})
		|| {
			let guard = self.votes.read();
			let is_old = guard.keys().next().map_or(true, |oldest| message.round() <= oldest);
			if is_old { trace!(target: "engine", "Old message {:?}.", message); }
			is_old
		}
	}

	/// Throws out messages older than message, leaves message as marker for the oldest.
	pub fn throw_out_old(&self, vote_round: &M::Round) {
		let mut guard = self.votes.write();
		let new_collector = guard.split_off(vote_round);
		*guard = new_collector;
	}

	/// Collects the signatures for a given round and hash.
	pub fn round_signatures(&self, round: &M::Round, block_hash: &H256) -> Vec<H520> {
		let guard = self.votes.read();
		guard
			.get(round)
			.and_then(|c| c.block_votes.get(&Some(*block_hash)))
			.map(|votes| votes.keys().cloned().collect())
			.unwrap_or_else(Vec::new)
	}

	/// Count votes which agree with the given message.
	pub fn count_aligned_votes(&self, message: &M) -> usize {
		self
			.votes
			.read()
			.get(&message.round())
			.map_or(0, |m| m.count_block(&message.block_hash()))
	}

	/// Count all votes collected for a given round.
	pub fn count_round_votes(&self, vote_round: &M::Round) -> usize {
		self.votes.read().get(vote_round).map_or(0, StepCollector::count)
	}

	/// Get all messages older than the round.
	pub fn get_up_to(&self, round: &M::Round) -> Vec<Bytes> {
		let guard = self.votes.read();
		guard
			.iter()
			.take_while(|&(r, _)| r <= round)
			.map(|(_, c)| c.messages.iter().filter(|m| m.is_broadcastable()).map(|m| ::rlp::encode(m).to_vec()).collect::<Vec<_>>())
			.fold(Vec::new(), |mut acc, mut messages| { acc.append(&mut messages); acc })
	}

	/// Retrieve address from which the message was sent from cache.
	pub fn get(&self, message: &M) -> Option<Address> {
		let guard = self.votes.read();
		guard.get(&message.round()).and_then(|c| c.block_votes.get(&message.block_hash())).and_then(|origins| origins.get(&message.signature()).cloned())
	}

	/// Count the number of total rounds kept track of.
	#[cfg(test)]
	pub fn len(&self) -> usize {
		self.votes.read().len()
	}
}

#[cfg(test)]
mod tests {
	use hash::keccak;
	use ethereum_types::{H160, H256};
	use rlp::*;
	use super::*;

	#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
	struct TestMessage {
		step: TestStep,
		block_hash: Option<H256>,
		signature: H520,
	}

	type TestStep = u64;

	impl Message for TestMessage {
		type Round = TestStep;

		fn signature(&self) -> H520 { self.signature }

		fn block_hash(&self) -> Option<H256> { self.block_hash }

		fn round(&self) -> &TestStep { &self.step }

		fn is_broadcastable(&self) -> bool { true }
	}

	impl Encodable for TestMessage {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.begin_list(3)
				.append(&self.signature)
				.append(&self.step)
				.append(&self.block_hash.unwrap_or_else(H256::zero));
		}
	}

	fn random_vote(collector: &VoteCollector<TestMessage>, signature: H520, step: TestStep, block_hash: Option<H256>) -> bool {
		full_vote(collector, signature, step, block_hash, &H160::random())
	}

	fn full_vote<'a>(collector: &VoteCollector<TestMessage>, signature: H520, step: TestStep, block_hash: Option<H256>, address: &'a Address) -> bool {
		collector.vote(TestMessage { signature: signature, step: step, block_hash: block_hash }, address).is_none()
	}

	#[test]
	fn seal_retrieval() {
		let collector = VoteCollector::default();
		let bh = Some(keccak("1"));
		let mut signatures = Vec::new();
		for _ in 0..5 {
			signatures.push(H520::random());
		}
		let propose_round = 3;
		let commit_round = 5;
		// Wrong round.
		random_vote(&collector, signatures[4].clone(), 1, bh.clone());
		// Good proposal
		random_vote(&collector, signatures[0].clone(), propose_round.clone(), bh.clone());
		// Wrong block proposal.
		random_vote(&collector, signatures[0].clone(), propose_round.clone(), Some(keccak("0")));
		// Wrong block commit.
		random_vote(&collector, signatures[3].clone(), commit_round.clone(), Some(keccak("0")));
		// Wrong round.
		random_vote(&collector, signatures[0].clone(), 6, bh.clone());
		// Wrong round.
		random_vote(&collector, signatures[0].clone(), 4, bh.clone());
		// Relevant commit.
		random_vote(&collector, signatures[2].clone(), commit_round.clone(), bh.clone());
		// Replicated vote.
		random_vote(&collector, signatures[2].clone(), commit_round.clone(), bh.clone());
		// Wrong round.
		random_vote(&collector, signatures[4].clone(), 6, bh.clone());
		// Relevant precommit.
		random_vote(&collector, signatures[1].clone(), commit_round.clone(), bh.clone());
		// Wrong round, same signature.
		random_vote(&collector, signatures[1].clone(), 7, bh.clone());

		assert_eq!(signatures[0..1].to_vec(), collector.round_signatures(&propose_round, &bh.unwrap()));
		assert_eq!(signatures[1..3].iter().collect::<HashSet<_>>(), collector.round_signatures(&commit_round, &bh.unwrap()).iter().collect::<HashSet<_>>());
	}

	#[test]
	fn count_votes() {
		let collector = VoteCollector::default();
		let round1 = 1;
		let round3 = 3;
		// good 1
		random_vote(&collector, H520::random(), round1, Some(keccak("0")));
		random_vote(&collector, H520::random(), 0, Some(keccak("0")));
		// good 3
		random_vote(&collector, H520::random(), round3, Some(keccak("0")));
		random_vote(&collector, H520::random(), 2, Some(keccak("0")));
		// good prevote
		random_vote(&collector, H520::random(), round1, Some(keccak("1")));
		// good prevote
		let same_sig = H520::random();
		random_vote(&collector, same_sig.clone(), round1, Some(keccak("1")));
		random_vote(&collector, same_sig, round1, Some(keccak("1")));
		// good precommit
		random_vote(&collector, H520::random(), round3, Some(keccak("1")));
		// good prevote
		random_vote(&collector, H520::random(), round1, Some(keccak("0")));
		random_vote(&collector, H520::random(), 4, Some(keccak("2")));

		assert_eq!(collector.count_round_votes(&round1), 4);
		assert_eq!(collector.count_round_votes(&round3), 2);

		let message = TestMessage {
			signature: H520::default(),
			step: round1,
			block_hash: Some(keccak("1"))
		};
		assert_eq!(collector.count_aligned_votes(&message), 2);
	}

	#[test]
	fn remove_old() {
		let collector = VoteCollector::default();
		let vote = |round, hash| {
			random_vote(&collector, H520::random(), round, hash);
		};
		vote(6, Some(keccak("0")));
		vote(3, Some(keccak("0")));
		vote(7, Some(keccak("0")));
		vote(8, Some(keccak("1")));
		vote(1, Some(keccak("1")));

		collector.throw_out_old(&7);
		assert_eq!(collector.len(), 2);
	}

	#[test]
	fn malicious_authority() {
		let collector = VoteCollector::default();
		let round = 3;
		// Vote is inserted fine.
		assert!(full_vote(&collector, H520::random(), round, Some(keccak("0")), &Address::default()));
		// Returns the double voting address.
		assert!(!full_vote(&collector, H520::random(), round, Some(keccak("1")), &Address::default()));
		assert_eq!(collector.count_round_votes(&round), 1);
	}
}
