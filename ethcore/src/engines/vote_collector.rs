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

//! Collects votes on hashes at each Message::Round.

use std::fmt::Debug;
use util::*;
use rlp::Encodable;

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
	voted: HashSet<Address>,
	pub block_votes: HashMap<Option<H256>, HashMap<H520, Address>>,
	messages: HashSet<M>,
}

impl <M: Message> StepCollector<M> {
	/// Returns Some(&Address) when validator is double voting.
	fn insert<'a>(&mut self, message: M, address: &'a Address) -> Option<&'a Address> {
		// Do nothing when message was seen.
		if self.messages.insert(message.clone()) {
			if self.voted.insert(address.clone()) {
				self
					.block_votes
					.entry(message.block_hash())
					.or_insert_with(HashMap::new)
					.insert(message.signature(), address.clone());
			} else {
				// Bad validator sent a different message.
				return Some(address);
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
	pub fn vote<'a>(&self, message: M, voter: &'a Address) -> Option<&'a Address> {
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
				if is_known { trace!(target: "poa", "Known message: {:?}.", message); }
				is_known
			})
		|| {
			let guard = self.votes.read();
			let is_old = guard.keys().next().map_or(true, |oldest| message.round() <= oldest);
			if is_old { trace!(target: "poa", "Old message {:?}.", message); }
			is_old
		}
	}

	/// Throws out messages older than message, leaves message as marker for the oldest.
	pub fn throw_out_old(&self, vote_round: &M::Round) {
		let mut guard = self.votes.write();
		let new_collector = guard.split_off(vote_round);
		*guard = new_collector;
	}

	/// Collects the signatures used to seal a block.
	pub fn seal_signatures(&self, proposal_round: M::Round, commit_round: M::Round, block_hash: &H256) -> Option<SealSignatures> {
		let ref bh = Some(*block_hash);
		let maybe_seal = {
			let guard = self.votes.read();
			guard
				.get(&proposal_round)
				.and_then(|c| c.block_votes.get(bh))
				.and_then(|proposals| proposals.keys().next())
				.map(|proposal| SealSignatures {
					proposal: proposal.clone(),
					votes: guard
						.get(&commit_round)
						.and_then(|c| c.block_votes.get(bh))
						.map(|precommits| precommits.keys().cloned().collect())
						.unwrap_or_else(Vec::new),
				})
				.and_then(|seal| if seal.votes.is_empty() { None } else { Some(seal) })
		};
		if maybe_seal.is_some() {
				// Remove messages that are no longer relevant.
				self.throw_out_old(&commit_round);
		}
		maybe_seal
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
	pub fn len(&self) -> usize {
		self.votes.read().len()
	}
}
