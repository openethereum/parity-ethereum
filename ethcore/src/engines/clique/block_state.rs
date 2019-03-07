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
use std::time::{Duration, SystemTime};
use std::time::UNIX_EPOCH;

use ethereum_types::Address;
use rand::Rng;

use engines::clique::{VoteType, DIFF_INTURN, DIFF_NOTURN, NULL_AUTHOR, SIGNING_DELAY_NOTURN_MS};
use engines::clique::util::{extract_signers, recover_creator};
use error::Error;
use types::header::Header;

/// Clique state for each block.
#[cfg(not(test))]
#[derive(Clone, Debug, Default)]
pub struct CliqueBlockState {
	/// all recorded votes before this blocks, k: (Voter, beneficiary), v: VoteType
	votes: HashMap<(Address, Address), VoteType>,
	/// a list of all vote happened before this block, item is an 4 item tuple: blockNumber, Voter, VoteType, beneficiary
	votes_history: Vec<(u64, Address, VoteType, Address)>,
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
	/// all recorded votes before this blocks, k: (Voter, beneficiary), v: VoteType
	pub votes: HashMap<(Address, Address), VoteType>,
	/// a list of all vote happened before this block, item is an 4 item tuple: blockNumber, Voter, VoteType, beneficiary
	pub votes_history: Vec<(u64, Address, VoteType, Address)>,
	/// a list of all valid signer, sorted by ascending order.
	pub signers: Vec<Address>,
	/// a deque of recent signer, new entry should be pushed front, apply() modifies this.
	pub recent_signers: VecDeque<Address>,
	/// inturn signing should wait until this time
	pub next_timestamp_inturn: Option<SystemTime>,
	/// noturn signing should wait until this time
	pub next_timestamp_noturn: Option<SystemTime>,
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
	fn verify(&self, header: &Header) -> Result<(Address), Error>{
		let creator = recover_creator(header)?.clone();

		// Check signer list
		if !self.signers.contains(&creator) {
			trace!(target: "engine", "current state: {:?}", self);
			return Err(From::from(format!("Error applying #{}({}): {} is not in the signer list!",
											header.number(),
											header.hash(),
											creator)));
		}

		// Check recent signer.
		if self.recent_signers.contains(&creator) {
			trace!(target: "engine", "current state: {:?}", self);
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

		// rotate recent signers.
		self.recent_signers.push_front(creator);
		if self.recent_signers.len() >= ( self.signers.len() / 2 ) + 1 {
			self.recent_signers.pop_back();
		}

		if is_checkpoint {
			// checkpoint block should not affect previous tallying, so we check that.
			let signers = extract_signers(header)?;
			if self.signers != signers {
				return Err(From::from("checkpoint block signers is different than expected"));
			};

			self.votes.clear();
			self.votes_history.clear();
		}

		// Contains vote
		if *header.author() != NULL_AUTHOR {
			let nonce = *header.decode_seal::<Vec<_>>()?.get(1).ok_or("Error decoding seal")?;
			self.update_signers_on_vote(VoteType::from_nonce(nonce)?, creator, *header.author(), header.number())?;
		}

		Ok(creator)
	}

	fn update_signers_on_vote(
		&mut self,
		vote_type: VoteType,
		creator: Address,
		beneficiary: Address,
		block_number: u64
	) -> Result<(), Error> {

		// Record this vote, also since we are using an hashmap, it will override previous vote.
		self.votes.insert((creator, beneficiary), vote_type);
		self.votes_history.push((block_number, creator, vote_type, beneficiary));

		// Tally up current target votes.
		let threshold = self.signers.len() / 2;
		let vote = self.votes.iter()
			.filter(|(key, value)| (**key).1 == beneficiary && **value == vote_type)
			.count();

		if vote > threshold {
			match vote_type {
				VoteType::Add => {
					self.signers.push(beneficiary);
				},
				VoteType::Remove => {
					let pos = self.signers.binary_search(&beneficiary)
						.map_err(|_| "Unable to find beneficiary in signer list when removing".to_string())?;
					self.signers.remove(pos);
				}
			}

			// signers are highly likely to be < 10.
			self.signers.sort();

			// make sure `recent_signers` is updated after add/remove
			if self.recent_signers.len() >= ( self.signers.len() / 2 ) + 1 {
				self.recent_signers.pop_back();
			}

			// Remove all votes about or made by this beneficiary
			{
				let items: Vec<_> = self.votes.keys()
					.filter(|key| (**key).0 == beneficiary || (**key).1 == beneficiary)
					.cloned()
					.collect();

				for key in items {
					self.votes.remove(&key);
				}
			}
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
		let res = self.signers.binary_search(address);
		match vote_type {
			VoteType::Add => res.is_ok(),
			VoteType::Remove => res.is_err(),
		}
	}

	pub fn signers(&self) -> &Vec<Address> {
		return &self.signers;
	}
}
