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

use std::borrow::BorrowMut;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{Duration, SystemTime};

use ethereum_types::{Address, H256, U256};
use rand::{Rng, thread_rng};

use engines::clique::{DIFF_INTURN, DIFF_NOT_INTURN, NONCE_AUTH_VOTE, NONCE_DROP_VOTE, NULL_AUTHOR};
use engines::clique::util::{extract_signers, recover_creator};
use error::Error;
use types::header::Header;

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum SignerAuthorization {
	InTurn,
	OutOfTurn,
	TooRecently,
	Unauthorized,
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum VoteType {
	Add,
	Remove
}

#[derive(Clone, Debug)]
pub struct CliqueBlockState {
	pub votes: HashMap<(Address, Address), VoteType>, // k: (Voter, beneficiary), VoteType)
	pub votes_history: Vec<(u64, Address, VoteType, Address)>, // blockNumber, Voter, VoteType, beneficiary
	pub signers: Vec<Address>,
	pub recent_signers: VecDeque<Address>,
}

impl CliqueBlockState {
	/// Create new state with given information, this is used creating new state from Checkpoint block.
	pub fn new(author: Address, signers_sorted: Vec<Address>) -> Self {
		return CliqueBlockState {
			votes: Default::default(),
			votes_history: Default::default(),
			signers: signers_sorted,
			recent_signers: VecDeque::from(vec![author]),
		};
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
		let inturn = self.inturn(header.number(), &creator);

		if (inturn && *header.difficulty() != DIFF_INTURN) ||
			(!inturn && *header.difficulty() != DIFF_NOT_INTURN) {
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
		self.recent_signers.push_front(creator.clone());
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

			// maybe release some memory.
			self.votes.shrink_to_fit();
			self.votes_history.shrink_to_fit();

			return Ok(creator);
		}

		let beneficiary = *header.author();

		// No vote, ignore.
		if beneficiary == NULL_AUTHOR {
			return Ok(creator);
		}

		let nonce = header.decode_seal::<Vec<&[u8]>>().unwrap()[1];

		let mut vote_type = VoteType::Add;
		if NONCE_AUTH_VOTE == nonce {
			vote_type = VoteType::Add;
		} else if NONCE_DROP_VOTE == nonce {
			vote_type = VoteType::Remove;
		} else {
			return Err(From::from("beneficiary specified but nonce was not AUTH or DROP"));
		};

		// Record this vote, also since we are using an hashmap, it will override previous vote.
		self.votes.insert((creator.clone(), beneficiary), vote_type);
		self.votes_history.push((header.number(), creator.clone(), vote_type, beneficiary));

		// Tally up current target votes.
		let threshold = self.signers.len() / 2;
		let vote = self.votes.iter().filter(|(key, value)| {
			(**key).1 == beneficiary && **value == vote_type
		}).count();

		if vote > threshold {
			match vote_type {
				VoteType::Add => {
					self.signers.push(beneficiary);
				},
				VoteType::Remove => {
					let pos = self.signers.binary_search(&beneficiary);
					if pos.is_ok() {
						self.signers.remove(pos.unwrap());
					}
				}
			}

			self.signers.sort();

			// Remove all votes about or made by this beneficiary
			{
				let votes_copy = self.votes.clone();
				let items: Vec<_> = votes_copy.iter().filter(|(key, value)| {
					(**key).0 == beneficiary || (**key).1 == beneficiary
				}).collect();

				for (key, _) in items {
					self.votes.remove(&key);
				}
			}
		}

		// No cascading votes.
		Ok(creator)
	}

	fn inturn(&self, current_block_number: u64, author: &Address) -> bool {
		if let Some(pos) = self.signers.iter().position(|x| *author == *x) {
			return current_block_number % self.signers.len() as u64 == pos as u64;
		}
		return false;
	}
}
