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

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{Duration, SystemTime};
use std::time::UNIX_EPOCH;

use ethereum_types::Address;
use rand::Rng;

use engines::clique::{VoteType, DIFF_INTURN, DIFF_NOTURN, NULL_AUTHOR, SIGNING_DELAY_NOTURN_MS};
use engines::clique::util::{extract_signers, recover_creator};
use error::Error;
use types::header::Header;
use types::BlockNumber;

/// Type that represent a pending vote
/// Votes that go against the proposal aren't counted since it's equivalent to not voting
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct PendingVote {
	kind: VoteType,
	beneficiary: Address,
}

/// Type that represent a vote
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Vote {
	block_number: BlockNumber,
	beneficiary: Address,
	kind: VoteType,
	signer: Address,
	reverted: bool,
}

/// Clique state for each block.
#[cfg(not(test))]
#[derive(Clone, Debug, Default)]
pub struct CliqueBlockState {
	/// All recorded votes for a given signer, `Vec<PendingVote>` is a stack of votes
	votes: HashMap<Address, Vec<PendingVote>>,
	/// A list of all votes for the given epoch
	votes_history: Vec<Vote>,
	/// a list of all valid signer, sorted by ascending order.
	signers: Vec<Address>,
	/// a deque of recent signer, new entry should be pushed front, apply() modifies this.
	recent_signers: VecDeque<Address>,
	/// inturn signing should wait until this time
	pub next_timestamp_inturn: Option<SystemTime>,
	/// noturn signing should wait until this time
	pub next_timestamp_noturn: Option<SystemTime>,
}

#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub struct CliqueBlockState {
	/// All recorded votes for a given signer, `Vec<PendingVote>` is a stack of votes
	pub votes: HashMap<Address, Vec<PendingVote>>,
	/// A list of all votes for the given epoch
	pub votes_history: Vec<Vote>,
	/// a list of all valid signer, sorted by ascending order.
	pub signers: Vec<Address>,
	/// a deque of recent signer, new entry should be pushed front, apply() modifies this.
	pub recent_signers: VecDeque<Address>,
	/// inturn signing should wait until this time
	pub next_timestamp_inturn: Option<SystemTime>,
	/// noturn signing should wait until this time
	pub next_timestamp_noturn: Option<SystemTime>,
}

impl fmt::Display for CliqueBlockState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let signers: Vec<String> = self.signers.iter()
			.map(|s| format!("{} vote_stack len: {}", s, self.votes.get(s).map_or(0, |v| v.len())))
			.collect();

		let recent_signers: Vec<String> = self.recent_signers.iter().map(|s| format!("{}", s)).collect();
		let num_votes = self.votes_history.len();
		let add_votes = self.votes_history.iter().filter(|v| v.kind == VoteType::Add).count();
		let rm_votes = self.votes_history.iter().filter(|v| v.kind == VoteType::Remove).count();
		let reverted_votes = self.votes_history.iter().filter(|v| v.reverted).count();

        write!(f,
		"Votes {{ \n signers: {:?} \n recent_signers: {:?} \n number of votes: {} \n number of add votes {}
		\r number of remove votes {} \n number of reverted votes: {}}}",
		signers, recent_signers, num_votes, add_votes, rm_votes, reverted_votes)
    }
}

impl CliqueBlockState {
	/// Create new state with given information, this is used creating new state from Checkpoint block.
	pub fn new(signers_sorted: Vec<Address>) -> Self {
		CliqueBlockState {
			signers: signers_sorted,
			..Default::default()
		}
	}

	// see https://github.com/ethereum/go-ethereum/blob/master/consensus/clique/clique.go#L474
	fn verify(&self, header: &Header) -> Result<Address, Error> {
		let creator = recover_creator(header)?.clone();

		// Check signer list
		if !self.signers.contains(&creator) {
			trace!(target: "engine", "current state: {}", self);
			return Err(From::from(format!("Error applying #{}({}): {} is not in the signer list!",
											header.number(),
											header.hash(),
											creator)));
		}

		// Check recent signer.
		if self.recent_signers.contains(&creator) {
			trace!(target: "engine", "current state: {}", self);
			return Err(From::from(format!("Error applying #{}({}): {} is in the recent_signer list!",
											header.number(),
											header.hash(),
											creator)));
		}

		// Ensure that the difficulty corresponds to the turn-ness of the signer
		let inturn = self.is_inturn(header.number(), &creator);

		if (inturn && *header.difficulty() != DIFF_INTURN) ||
			(!inturn && *header.difficulty() != DIFF_NOTURN) {
			return Err(From::from(format!("Error applying #{}({}): wrong difficulty!",
											header.number(),
											header.hash())));
		}

		Ok(creator)
	}

	/// Verify and apply an new header to current state, might fail with error.
	pub fn apply(&mut self, header: &Header, is_checkpoint: bool) -> Result<Address, Error> {
		let creator = self.verify(header)?;
		self.recent_signers.push_front(creator);
		self.rotate_recent_signers();

		if is_checkpoint {
			// checkpoint block should not affect previous tallying, so we check that.
			let signers = extract_signers(header)?;
			if self.signers != signers {
				return Err(From::from("checkpoint block signers is different than expected"));
			};

			// TODO(niklasad1): I'm not sure if we should shrink here because it is likely that next epoch
			// will need some memory and might be better for allocation algorithm to decide whether to shrink or
			// (typically double or halves the allocted memory when necessary)
			self.votes.clear();
			self.votes_history.clear();
			self.votes.shrink_to_fit();
			self.votes_history.shrink_to_fit();
		}

		// Contains vote
		if *header.author() != NULL_AUTHOR {
			let nonce = *header.decode_seal::<Vec<_>>()?.get(1).ok_or("Error decoding seal")?;
			self.update_signers_on_vote(VoteType::from_nonce(nonce)?, creator, *header.author(), header.number())?;
		}

		Ok(creator)
	}

	// TODO(niklasad1): this could be more efficient (very naive)
	fn update_signers_on_vote(
		&mut self,
		kind: VoteType,
		signer: Address,
		beneficiary: Address,
		block_number: u64
	) -> Result<(), Error> {

		trace!(target: "engine", "Attempt vote {:?} {:?}", kind, beneficiary);

		// Vote is valid either build a new `stack` or push to the existing stack
		let reverted = if self.is_valid_vote(&beneficiary, kind) {
			self.add_vote(signer, beneficiary, kind)
		} else {
			// This case only happens if a `signer` wants to revert their previous vote
			// (does nothing if no previous vote was found)
			self.revert_vote(signer, kind)
		};

		// Add all votes to the history
		self.votes_history.push(
			Vote {
			block_number,
			beneficiary,
			kind,
			signer,
			reverted,
		});

		let threshold = self.signers.len() / 2;
		// Make it explicit that that we ignore the `vote_kind` if votes == 0
		let (votes, vote_kind) = match self.get_current_votes_and_kind(beneficiary) {
			Some((v, k)) => (v, k),
			None => (0, VoteType::Add),
		};
		debug!(target: "engine", "{}/{} votes to have consensus", votes, threshold + 1);
		trace!(target: "engine", "votes: {:?}", votes);

		if votes > threshold {
			match vote_kind {
				VoteType::Add => {
					debug!(target: "engine", "added new signer: {}", beneficiary);
					self.signers.push(beneficiary);
				}
				VoteType::Remove => {
					let pos = self.signers.binary_search(&beneficiary)
						.map_err(|_| "Unable to find beneficiary in signer list when removing".to_string())?;
					debug!(target: "engine", "removed signer: {}", beneficiary);
					self.signers.remove(pos);
				}
			}

			// signers are highly likely to be < 10.
			// TODO(niklasad1): only sort when pushing
			self.signers.sort();
			self.rotate_recent_signers();
			self.remove_all_votes_from(beneficiary);
		}

		Ok(())
	}

	pub fn calc_next_timestamp(&mut self, header: &Header, period: u64) {
		let base_time = UNIX_EPOCH + Duration::from_secs(header.timestamp());

		self.next_timestamp_inturn = Some(base_time + Duration::from_secs(period));

		let delay = Duration::from_millis(
			rand::thread_rng().gen_range(0u64, (self.signers.len() as u64 / 2 + 1) * SIGNING_DELAY_NOTURN_MS));
		self.next_timestamp_noturn = Some(base_time + Duration::from_secs(period) + delay);
	}

	pub fn is_inturn(&self, current_block_number: u64, author: &Address) -> bool {
		if let Some(pos) = self.signers.iter().position(|x| *author == *x) {
			return current_block_number % self.signers.len() as u64 == pos as u64;
		}
		false
	}

	pub fn is_authorized(&self, author: &Address) -> bool {
		self.signers.contains(author) && !self.recent_signers.contains(author)
	}

	// returns whether it makes sense to cast the specified vote in the
	// current state (e.g. don't try to add an already authorized signer).
	pub fn is_valid_vote(&self, address: &Address, vote_type: VoteType) -> bool {
		let in_signer = self.signers.contains(address);
		match vote_type {
			VoteType::Add => !in_signer,
			VoteType::Remove => in_signer,
		}
	}

	pub fn signers(&self) -> &Vec<Address> {
		return &self.signers;
	}

	// Note this method will always return `true` but it is intendend for a unifrom `API`
	fn add_vote(&mut self, signer: Address, beneficiary: Address, kind: VoteType) -> bool {
		let new_vote = PendingVote {
			kind,
			beneficiary
		};

		self.votes.entry(signer).and_modify(|vote_stack| {
			vote_stack.push(new_vote);
		})
		.or_insert_with(|| vec![new_vote]);
		true
	}

	fn revert_vote(&mut self, signer: Address, kind: VoteType) -> bool {
		let mut revert = false;

		self.votes.entry(signer).and_modify(|votes| {
			trace!(target: "engine", "signer {} attempted to revert its vote with stack_len: {}", signer, votes.len());
			if let Some(v) = votes.pop() {
				revert = true;
				assert!(v.kind != kind);
			}
		});
		revert
	}

	fn get_current_votes_and_kind(&self, beneficiary: Address) -> Option<(usize, VoteType)> {
		// Tally up current target votes.
		let kind = self.votes.values()
			.filter_map(|votes| votes.last())
			.filter(|vote| vote.beneficiary == beneficiary)
			.nth(0)?
			.kind;

		let votes = self.votes.values()
			.filter_map(|votes| votes.last())
			.filter(|vote| vote.beneficiary == beneficiary)
			.count();

		Some((votes, kind))
	}

	fn rotate_recent_signers(&mut self) {
		if self.recent_signers.len() >= ( self.signers.len() / 2 ) + 1 {
			self.recent_signers.pop_back();
		}
	}

	fn remove_all_votes_from(&mut self, beneficiary: Address) {
		self.votes.remove(&beneficiary);

		for votes in self.votes.values_mut() {
			votes.retain(|v| v.beneficiary != beneficiary);
		}
	}
}
