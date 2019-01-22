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

use engines::clique::{recover, SIGNER_SIG_LENGTH, SIGNER_VANITY_LENGTH};
use error::Error;
use ethkey::public_to_address;
use types::header::Header;

pub const NONCE_DROP_VOTE: &[u8; 8] = &[0x00; 8];
pub const NONCE_AUTH_VOTE: &[u8; 8] = &[0xff; 8];
pub const NULL_AUTHOR: [u8; 20] = [0; 20];
pub const DIFF_INTURN: u8 = 2;
pub const DIFF_NOT_INTURN: u8 = 1;
pub const STATE_CACHE_NUM: usize = 4096;

pub enum SignerAuthorization {
	InTurn,
	OutOfTurn,
	Unauthorized,
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum VoteType {
	Add,
	Remove
}

#[derive(Debug)]
pub struct CliqueBlock {
	is_checkpoint_block: bool,
	creator: Address,
	header: Header,
}

pub struct CliqueState {
	epoch_length: u64,
	states_by_hash: LruCache<H256, SnapshotState>,
	signer: RwLock<Option<Address>>,
	active_prop_delay: Option<(H256, SystemTime, Duration)>
}

impl fmt::Debug for CliqueState {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "CliqueState {{ epoch_length: {}, states_by_hash: {:?}, signer: {} }}", self.epoch_length, &self.states_by_hash, self.signer.read().unwrap_or(Address::new()))
	}
}

#[derive(Clone, Debug)]
pub struct SnapshotState {
	pub votes: HashMap<(Address, Address), VoteType>, // k: (Voter, beneficiary), VoteType)
	pub votes_history: Vec<(u64, Address, VoteType, Address)>, // blockNumber, Voter, VoteType, beneficiary
	pub signers: Vec<Address>,
	pub recent_signers: VecDeque<Address>,
}

impl CliqueState {
	pub fn new(epoch_length: u64) -> Self {
		CliqueState {
			epoch_length: epoch_length,
			states_by_hash: LruCache::new(STATE_CACHE_NUM),
				signer: RwLock::new(None),
				active_prop_delay: None,
		}
	}

	/// Get an valid state
	pub fn state(&mut self, hash: &H256) -> Option<SnapshotState> {
		let db = self.states_by_hash.borrow_mut();
		return db.get_mut(hash).cloned();
	}

	pub fn turn_delay(&mut self, header: &Header) -> bool {
		match self.active_prop_delay {
			Some((parent_hash, start, duration)) => {
				if *header.parent_hash() != parent_hash {
					// reorg.  make sure the timer is reset
					self.active_prop_delay = Some((header.parent_hash().clone(),
																					SystemTime::now(),
																					Duration::from_millis(thread_rng().gen_range::<u64>(0, self.state(header.parent_hash()).unwrap().signers.len() as u64 * 500))));
					return false;
				}

				if start.elapsed().expect("start delay was after current time") >= duration {
					return true
				} else {
					return false
				}
			},
			None => {
					self.active_prop_delay = Some((header.parent_hash().clone(),
																					SystemTime::now(),
																					Duration::from_millis(thread_rng().gen_range::<u64>(0, self.state(header.parent_hash()).unwrap().signers.len() as u64 * 500))));
					return false;
			}
		}
	}

	/// Apply an new header
	pub fn apply(&mut self, header: &Header) -> Result<(), Error> {
		let db = self.states_by_hash.borrow_mut();

		// make sure current hash is not in the db
		match db.get_mut(header.parent_hash()).cloned() {
			Some(ref mut new_state) => {
				let creator = match process_header(&header, new_state, self.epoch_length) {
					Err(e) => {
						return Err(From::from(
							format!("Error applying header: {}, current state: {:?}", e, new_state)
						));
					},
					Ok(creator) => {creator} ,
				};

				new_state.recent_signers.push_front(creator);

				if new_state.recent_signers.len() >= ( new_state.signers.len() / 2 ) + 1 {
					new_state.recent_signers.pop_back();
				}

				db.insert(header.hash(), new_state.clone());
				Ok(())
			}
			None => {
				Err(From::from(
					format!("Parent block (hash: {}) for Block {}, hash {} is not found!",
									header.parent_hash(),
									header.number(), header.hash() )))
			}
		}
	}

	pub fn apply_checkpoint(&mut self, header: &Header) -> Result<(), Error> {
		let db = self.states_by_hash.borrow_mut();
		let state = &mut SnapshotState {
			votes: HashMap::new(),
			votes_history: Vec::new(),
			signers: Vec::new(),
			recent_signers: VecDeque::new(),
		};
		process_genesis_header(header, state)?;

		trace!("inserting {} {:?}", header.hash(), &state);
		db.insert(header.hash(), state.clone());

		Ok(())
	}

	pub fn set_signer_address(&self, signer_address: Address) {
		trace!(target: "engine", "setting signer {}", signer_address);
		*self.signer.write() = Some(signer_address.clone());
	}

	pub fn proposer_authorization(&mut self, header: &Header) -> SignerAuthorization {
		let mut db = self.states_by_hash.borrow_mut();

		let proposer_address = match *self.signer.read() {
			Some(address) => address.clone(),
			None => { return SignerAuthorization::Unauthorized }
		};

		match db.get_mut(header.parent_hash()).cloned() {
			Some(ref state) => {
				return state.get_signer_authorization(header.number(), &proposer_address);
			},
			None => {
				panic!("Parent block (hash: {}) for Block {}, hash {} is not found!",
								header.parent_hash(),
								header.number(), header.hash())
			}
		}
	}
}

fn extract_signers(header: &Header) -> Result<Vec<Address>, Error> {
	let min_extra_data_size = (SIGNER_VANITY_LENGTH as usize) + (SIGNER_SIG_LENGTH as usize);

	assert!(header.extra_data().len() >= min_extra_data_size, "need minimum genesis extra data size {}.  found {}.", min_extra_data_size, header.extra_data().len());

	// extract only the portion of extra_data which includes the signer list
	let signers_raw = &header.extra_data()[(SIGNER_VANITY_LENGTH as usize)..header.extra_data().len() - (SIGNER_SIG_LENGTH as usize)];

	assert_eq!(signers_raw.len() % 20, 0, "bad signer list length {}", signers_raw.len());

	let num_signers = signers_raw.len() / 20;
	let mut signers_list: Vec<Address> = vec![];

	for i in 0..num_signers {
		let mut signer = Address::default();
		signer.copy_from_slice(&signers_raw[i * 20..(i + 1) * 20]);
		signers_list.push(signer);
	}
	// NOTE: base on geth implmentation , signers list area always sorted to ascending order.
	signers_list.sort();

	Ok(signers_list)
}

impl SnapshotState {
	pub fn get_signer_authorization(&self, current_block_number: u64, author: &Address) -> SignerAuthorization {
		// TODO: Implement recent signer check list.
		if let Some(pos) = self.signers.iter().position(|x| *author == *x) {
			if current_block_number % self.signers.len() as u64 == pos as u64 {
				return SignerAuthorization::InTurn;
			} else {
				if self.recent_signers.contains(&self.signers[pos]) && pos != self.signers.len()-1 {
					return SignerAuthorization::Unauthorized;
				} else {
					// author didn't sign recently, or will be shifted out of the recent
					// signer list this block
					return SignerAuthorization::OutOfTurn;
				}
			}
		}

		trace!(target: "engine", "get_signer_authorization, didn't find {} in signers list {:?}", author, &self.signers);
		return SignerAuthorization::Unauthorized;
	}
}

fn process_genesis_header(header: &Header, state: &mut SnapshotState) -> Result<(), Error> {
	state.signers = extract_signers(header)?;
	state.votes.clear();
	state.votes_history.clear();
	state.recent_signers.clear();
	trace!(target: "engine", "genesis signers are {:?}", &state.signers);
	Ok(())
}

// get the hash of a block ommitting the signature bytes.  Assumes that the block header contains
// the signature bytes
/*
fn clique_hash(h: &Header) -> U256 {
	let mut header = header.clone();
	let new_extra_data_len = h.header.extra_data.len()-SIGNER_SIG_LENGTH
	let old_extra_data = &h.header.extra_data()[0..new_extra_data_len];
	let mut extra_data = Vec<u8>::new(new_extra_data_len);

	extra_data.copy_from_slice(old_extra_data);
	header.set_extra_data(extra_data);

	return header.hash();
}
*/

/// Apply header to the state, used in block sealing and external block import
fn process_header(header: &Header, state: &mut SnapshotState, epoch_length: u64) -> Result<Address, Error> {

	if header.extra_data().len() < SIGNER_VANITY_LENGTH as usize + SIGNER_SIG_LENGTH as usize {
		return Err(From::from(format!("header extra data was too small: {}", header.extra_data().len())));
	}

	let creator = public_to_address(&recover(header).unwrap()).clone();

	match state.get_signer_authorization(header.number(), &creator) {
		SignerAuthorization::InTurn => {
			if *header.difficulty() != U256::from(DIFF_INTURN) {
				return Err(From::from("difficulty must be set to DIFF_INTURN"));
			}
		}
		SignerAuthorization::OutOfTurn => {
			if *header.difficulty() != U256::from(DIFF_NOT_INTURN) {
				return Err(From::from("difficulty must be set to DIFF_NOT_INTURN"));
			}
		}
		SignerAuthorization::Unauthorized => {
			return Err(From::from(
				format!("unauthorized to sign at this time: creator: {}", creator)
			));
		}
	}

	// header Authorized

	// If this is checkpoint blocks
	if header.number() % epoch_length == 0 {
		let signers = extract_signers(header)?;

			assert!(signers.iter().zip(state.signers.iter()).filter(|(a, b)| { a != b }).count() == 0, "received signer list did not match computed");

			state.signers = signers;
		state.votes.clear();
		state.votes_history.clear();
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
		return Err(From::from("beneficiary specificed but nonce was not AUTH or DROP"));
	};

	state.votes_history.push((header.number(), creator, vote_type, beneficiary));

	// Discard any of previous votes
	state.votes.remove(&(creator, beneficiary));

	state.votes.insert((creator, beneficiary), vote_type);

	// Tally up current target votes.
	let threshold = state.signers.len() / 2;
	let vote = state.votes.iter().filter(|(key, value)| {
		(**key).1 == beneficiary && **value == vote_type
	}).count();

	if vote > threshold {
		match vote_type {
			VoteType::Add => {
				state.signers.push(beneficiary);
			},
			VoteType::Remove => {
				let pos = state.signers.binary_search(&beneficiary);
				if pos.is_ok() {
					state.signers.remove(pos.unwrap());
				}
			}
		}

		state.signers.sort();

		// Remove all votes about or made by this beneficiary
		{
			let votes_copy = state.votes.clone();
			let items: Vec<_> = votes_copy.iter().filter(|(key, value)| {
				(**key).0 == beneficiary || (**key).1 == beneficiary
			}).collect();

			for (key, _) in items {
				state.votes.remove(&key);
			}
		}
	}
	// No cascading votes.

	Ok(creator)
}
