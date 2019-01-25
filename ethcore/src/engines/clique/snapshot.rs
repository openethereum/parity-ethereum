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
use lru_cache::LruCache;
use parking_lot::RwLock;
use rand::{Rng, thread_rng};

use engines::clique::{recover_creator, SIGNER_SIG_LENGTH, SIGNER_VANITY_LENGTH};
use error::Error;
use ethkey::public_to_address;
use types::header::Header;
use engines::clique::extract_signers;

pub const NONCE_DROP_VOTE: &[u8; 8] = &[0x00; 8];
pub const NONCE_AUTH_VOTE: &[u8; 8] = &[0xff; 8];
pub const NULL_AUTHOR: [u8; 20] = [0; 20];
pub const DIFF_INTURN: u8 = 2;
pub const DIFF_NOT_INTURN: u8 = 1;

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
	// Verify given header, this is an internal check for data integrity and consensus rule complaint, should
	// not be used externally.
	fn verify(&self, header: &Header) -> Result<(Address), Error>{
		if header.extra_data().len() < SIGNER_VANITY_LENGTH as usize + SIGNER_SIG_LENGTH as usize {
			return Err(From::from(format!("header extra data was too small: {}", header.extra_data().len())));
		}
		let creator = recover_creator(header)?.clone();

		// Check signer list
		if !self.signers.contains(&creator) {
			return Err(From::from(format!("{} is not in the signer list!", creator)));
		}

		// Check recent signer.
		if self.recent_signers.contains(&creator) {
			return Err(From::from(format!("{} is in the recent_signer list!", creator)));
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

			// release some memory.
			self.votes.shrink_to_fit();
			self.votes_history.shrink_to_fit();

			return Ok(creator);
		}

		let beneficiary = header.author().clone();

		// No vote, ignore.
		if beneficiary[0..20] == NULL_AUTHOR {
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
