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

use std::collections::{HashMap, BTreeSet, VecDeque};
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use engines::clique::util::{extract_signers, recover_creator};
use engines::clique::{VoteType, DIFF_INTURN, DIFF_NOTURN, NULL_AUTHOR, SIGNING_DELAY_NOTURN_MS};
use ethereum_types::{Address, H64};
use rand::Rng;
use time_utils::CheckedSystemTime;
use types::{
	BlockNumber,
	header::Header,
	errors::{BlockError, EthcoreError as Error, EngineError},
};
use unexpected::Mismatch;

/// Type that keeps track of the state for a given vote
// Votes that go against the proposal aren't counted since it's equivalent to not voting
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct VoteState {
	kind: VoteType,
	votes: u64,
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

/// Type that represent a pending vote
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct PendingVote {
	signer: Address,
	beneficiary: Address,
}

/// Clique state for each block.
#[cfg(not(test))]
#[derive(Clone, Debug, Default)]
pub struct CliqueBlockState {
	/// Current votes for a beneficiary
	votes: HashMap<PendingVote, VoteState>,
	/// A list of all votes for the given epoch
	votes_history: Vec<Vote>,
	/// a list of all valid signer, sorted by ascending order.
	signers: BTreeSet<Address>,
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
	pub votes: HashMap<PendingVote, VoteState>,
	/// A list of all votes for the given epoch
	pub votes_history: Vec<Vote>,
	/// a list of all valid signer, sorted by ascending order.
	pub signers: BTreeSet<Address>,
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
			.map(|s|
				 format!("{} {:?}",
					s,
					self.votes.iter().map(|(v, s)| format!("[beneficiary {}, votes: {}]", v.beneficiary, s.votes))
					.collect::<Vec<_>>()
				)
			)
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
	pub fn new(signers: BTreeSet<Address>) -> Self {
		CliqueBlockState {
			signers,
			..Default::default()
		}
	}

	// see https://github.com/ethereum/go-ethereum/blob/master/consensus/clique/clique.go#L474
	fn verify(&self, header: &Header) -> Result<Address, Error> {
		let creator = recover_creator(header)?.clone();

		// The signer is not authorized
		if !self.signers.contains(&creator) {
			trace!(target: "engine", "current state: {}", self);
			Err(EngineError::NotAuthorized(creator))?
		}

		// The signer has signed a block too recently
		if self.recent_signers.contains(&creator) {
			trace!(target: "engine", "current state: {}", self);
			Err(EngineError::CliqueTooRecentlySigned(creator))?
		}

		// Wrong difficulty
		let inturn = self.is_inturn(header.number(), &creator);

		if inturn && *header.difficulty() != DIFF_INTURN {
			Err(BlockError::InvalidDifficulty(Mismatch {
				expected: DIFF_INTURN,
				found: *header.difficulty(),
			}))?
		}

		if !inturn && *header.difficulty() != DIFF_NOTURN {
			Err(BlockError::InvalidDifficulty(Mismatch {
				expected: DIFF_NOTURN,
				found: *header.difficulty(),
			}))?
		}

		Ok(creator)
	}

	/// Verify and apply a new header to current state
	pub fn apply(&mut self, header: &Header, is_checkpoint: bool) -> Result<Address, Error> {
		let creator = self.verify(header)?;
		self.recent_signers.push_front(creator);
		self.rotate_recent_signers();

		if is_checkpoint {
			// checkpoint block should not affect previous tallying, so we check that.
			let signers = extract_signers(header)?;
			if self.signers != signers {
				let invalid_signers: Vec<String> = signers.into_iter()
					.filter(|s| !self.signers.contains(s))
					.map(|s| format!("{}", s))
					.collect();
				Err(EngineError::CliqueFaultyRecoveredSigners(invalid_signers))?
			};

			// TODO(niklasad1): I'm not sure if we should shrink here because it is likely that next epoch
			// will need some memory and might be better for allocation algorithm to decide whether to shrink or not
			// (typically doubles or halves the allocted memory when necessary)
			self.votes.clear();
			self.votes_history.clear();
			self.votes.shrink_to_fit();
			self.votes_history.shrink_to_fit();
		}

		// Contains vote
		if *header.author() != NULL_AUTHOR {
			let decoded_seal = header.decode_seal::<Vec<_>>()?;
			if decoded_seal.len() != 2 {
				Err(BlockError::InvalidSealArity(Mismatch { expected: 2, found: decoded_seal.len() }))?
			}

			let nonce = H64::from_slice(decoded_seal[1]);
			self.update_signers_on_vote(VoteType::from_nonce(nonce)?, creator, *header.author(), header.number())?;
		}

		Ok(creator)
	}

	fn update_signers_on_vote(
		&mut self,
		kind: VoteType,
		signer: Address,
		beneficiary: Address,
		block_number: u64
	) -> Result<(), Error> {

		trace!(target: "engine", "Attempt vote {:?} {:?}", kind, beneficiary);

		let pending_vote = PendingVote { signer, beneficiary };

		let reverted = if self.is_valid_vote(&beneficiary, kind) {
			self.add_vote(pending_vote, kind)
		} else {
			// This case only happens if a `signer` wants to revert their previous vote
			// (does nothing if no previous vote was found)
			self.revert_vote(pending_vote)
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

		// If no vote was found for the beneficiary return `early` but don't propogate an error
		let (votes, vote_kind) = match self.get_current_votes_and_kind(beneficiary) {
			Some((v, k)) => (v, k),
			None => return Ok(()),
		};
		let threshold = self.signers.len() / 2;

		debug!(target: "engine", "{}/{} votes to have consensus", votes, threshold + 1);
		trace!(target: "engine", "votes: {:?}", votes);

		if votes > threshold {
			match vote_kind {
				VoteType::Add => {
					if self.signers.insert(beneficiary) {
						debug!(target: "engine", "added new signer: {}", beneficiary);
					}
				}
				VoteType::Remove => {
					if self.signers.remove(&beneficiary) {
						debug!(target: "engine", "removed signer: {}", beneficiary);
					}
				}
			}

			self.rotate_recent_signers();
			self.remove_all_votes_from(beneficiary);
		}

		Ok(())
	}

	/// Calculate the next timestamp for `inturn` and `noturn` fails if any of them can't be represented as
	/// `SystemTime`
	// TODO(niklasad1): refactor this method to be in constructor of `CliqueBlockState` instead.
	// This is a quite bad API because we must mutate both variables even when already `inturn` fails
	// That's why we can't return early and must have the `if-else` in the end
	pub fn calc_next_timestamp(&mut self, timestamp: u64, period: u64) -> Result<(), Error> {
		let inturn = CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(timestamp.saturating_add(period)));

		self.next_timestamp_inturn = inturn;

		let delay = Duration::from_millis(
			rand::thread_rng().gen_range(0u64, (self.signers.len() as u64 / 2 + 1) * SIGNING_DELAY_NOTURN_MS));
		self.next_timestamp_noturn = inturn.map(|inturn|  {
			inturn + delay
		});

		if self.next_timestamp_inturn.is_some() && self.next_timestamp_noturn.is_some() {
			Ok(())
		} else {
			Err(BlockError::TimestampOverflow)?
		}
	}

	/// Returns true if the block difficulty should be `inturn`
	pub fn is_inturn(&self, current_block_number: u64, author: &Address) -> bool {
		if let Some(pos) = self.signers.iter().position(|x| *author == *x) {
			return current_block_number % self.signers.len() as u64 == pos as u64;
		}
		false
	}

	/// Returns whether the signer is authorized to sign a block
	pub fn is_authorized(&self, author: &Address) -> bool {
		self.signers.contains(author) && !self.recent_signers.contains(author)
	}

	/// Returns whether it makes sense to cast the specified vote in the
	/// current state (e.g. don't try to add an already authorized signer).
	pub fn is_valid_vote(&self, address: &Address, vote_type: VoteType) -> bool {
		let in_signer = self.signers.contains(address);
		match vote_type {
			VoteType::Add => !in_signer,
			VoteType::Remove => in_signer,
		}
	}

	/// Returns the list of current signers
	pub fn signers(&self) -> &BTreeSet<Address> {
		&self.signers
	}

	// Note this method will always return `true` but it is intended for a uniform `API`
	fn add_vote(&mut self, pending_vote: PendingVote, kind: VoteType) -> bool {

		self.votes.entry(pending_vote)
			.and_modify(|state| {
				state.votes = state.votes.saturating_add(1);
			})
			.or_insert_with(|| VoteState { kind, votes: 1 });
		true
	}

	fn revert_vote(&mut self, pending_vote: PendingVote) -> bool {
		let mut revert = false;
		let mut remove = false;

		self.votes.entry(pending_vote).and_modify(|state| {
			if state.votes.saturating_sub(1) == 0 {
				remove = true;
			}
			revert = true;
		});

		if remove {
			self.votes.remove(&pending_vote);
		}

		revert
	}

	fn get_current_votes_and_kind(&self, beneficiary: Address) -> Option<(usize, VoteType)> {
		let kind = self.votes.iter()
			.find(|(v, _t)| v.beneficiary == beneficiary)
			.map(|(_v, t)| t.kind)?;

		let votes = self.votes.keys()
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
		self.votes = std::mem::replace(&mut self.votes, HashMap::new())
			.into_iter()
			.filter(|(v, _t)| v.signer != beneficiary && v.beneficiary != beneficiary)
			.collect();
	}
}
