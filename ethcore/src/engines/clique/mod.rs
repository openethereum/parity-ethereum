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

//! Implemenation of Clique (POA) Engine.
//!
//! mod.rs -> CliqueEngine, the engine api implementation, with additional block state tracking.
//!
//! CliqueEngine -> {
//!   block_state_by_hash  -> block state indexed by header hash.
//!   ...rest
//! }
//!
//! snapshot.rs -> CliqueBlockState , record clique state for given block.
//! param.rs -> Clique Params.
//!

/// How syncing code path works:
/// 1. Client calls engine.verify_block_basic() and engine.verify_block_unordered().
/// 2. Client calls engine.verify_block_family(header, parent).
/// 3. Engine first find parent state: last_state = self.state(parent.hash())
///    if not found, trigger an back-fill from last checkpoint.
/// 4. Engine calls last_state.apply(header)
///
/// executive_author()
/// Clique use author field for voting, the real author is hidden in the extra_data field. So
/// When executing transactions, Client will calls engine.executive_author().

/// How sealing works:
/// 1. client call engine.set_signer() to activate sealing
///

use core::borrow::BorrowMut;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::mem;
use std::sync::{Arc, Weak};
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use ethereum_types::{Address, H160, H256, Public, U256};
use hash::KECCAK_EMPTY_LIST_RLP;
use lru_cache::LruCache;
use parking_lot::RwLock;
use parking_lot::RwLockUpgradableReadGuard;
use rand::thread_rng;
use rlp::encode;

use account_provider::AccountProvider;
use block::*;
use client::{BlockId, EngineClient};
use engines::{ConstructedVerifier, Engine, Headers, PendingTransitionStore, Seal};
use error::Error;
use ethkey::{Password, public_to_address, recover as ec_recover, Signature};
use io::IoService;
use machine::{AuxiliaryData, Call, EthereumMachine};
use types::BlockNumber;
use types::header::{ExtendedHeader, Header};

use super::signer::EngineSigner;

use self::params::CliqueParams;
use self::snapshot::CliqueBlockState;
use self::step_service::StepService;

mod params;
mod snapshot;
mod step_service;

// protocol constants
/// Fixed number of extra-data prefix bytes reserved for signer vanity
pub const SIGNER_VANITY_LENGTH: usize = 32;
/// Fixed number of extra-data suffix bytes reserved for signer signature
pub const SIGNER_SIG_LENGTH: usize = 65;
pub const NONCE_DROP_VOTE: [u8; 8] = [0x00; 8];
pub const NONCE_AUTH_VOTE: [u8; 8] = [0xff; 8];
pub const DIFF_INTURN: U256 = U256([2, 0, 0, 0]);
pub const DIFF_NOT_INTURN: U256 = U256([1, 0, 0, 0]);
pub const NULL_AUTHOR: Address = H160([0x00; 20]);
pub const NULL_NONCE: [u8; 8] = NONCE_DROP_VOTE;
pub const NULL_MIXHASH: [u8; 32] = [0x00; 32];
pub const NULL_UNCLES_HASH: H256 = KECCAK_EMPTY_LIST_RLP;

// Caches
/// How many CliqueBlockState to cache in the memory.
pub const STATE_CACHE_NUM: usize = 128;
/// How many recovered signature to cache in the memory.
pub const CREATOR_CACHE_NUM: usize = 4096;
lazy_static! {
	/// key: header hash
	/// value: creator address
	static ref CREATOR_BY_HASH: RwLock<LruCache<H256, Address>> = RwLock::new(LruCache::new(CREATOR_CACHE_NUM));
}

/// Clique Engine implementation
pub struct Clique {
	epoch_length: u64,
	period: u64,
	machine: EthereumMachine,
	client: RwLock<Option<Weak<EngineClient>>>,
	block_state_by_hash: RwLock<LruCache<H256, CliqueBlockState>>,
	active_prop_delay: Option<(H256, SystemTime, Duration)>,
	signer: RwLock<EngineSigner>,
	step_service: IoService<Duration>,
}

/*
 * only sign over non-signature bytes (vanity data).  There shouldn't be a signature here to sign
 * yet.
 */
pub fn sig_hash(header: &Header) -> Result<H256, Error> {
	if header.extra_data().len() >= SIGNER_VANITY_LENGTH {
		let extra_data = header.extra_data().clone();
		let mut reduced_header = header.clone();
		reduced_header.set_extra_data(
			extra_data[..extra_data.len() - SIGNER_SIG_LENGTH].to_vec());

		Ok(reduced_header.hash())
	} else {
		Ok(header.hash())
	}
}

/// Recover block creator from signature
pub fn recover_creator(header: &Header) -> Result<Address, Error> {
	// Initialization
	let mut cache = CREATOR_BY_HASH.write();

	if let Some(creator) = cache.get_mut(&header.hash()) {
		return Ok(*creator);
	}

	let data = header.extra_data();
	let mut sig = [0; SIGNER_SIG_LENGTH];
	sig.copy_from_slice(&data[(data.len() - SIGNER_SIG_LENGTH)..]);

	let msg = sig_hash(header)?;
	let pubkey = ec_recover(&Signature::from(sig), &msg)?;
	let creator = public_to_address(&pubkey);

	cache.insert(header.hash(), creator.clone());
	Ok(creator)
}

/// Extract signer list from extra_data.
///
/// Layout of extra_data:
/// ----
/// VANITY: 32 bytes
/// Signers: N * 32 bytes as hex encoded (20 characters)
/// Signature: 65 bytes
/// --
pub fn extract_signers(header: &Header) -> Result<Vec<Address>, Error> {
	let data = header.extra_data();

	if data.len() <= SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH {
		return Err(Box::new("Invalid extra_data size.").into());
	}

	// extract only the portion of extra_data which includes the signer list
	let signers_raw = &data[(SIGNER_VANITY_LENGTH)..data.len() - (SIGNER_SIG_LENGTH)];

	if signers_raw.len() % 20 != 0 {
		return Err(Box::new("bad signer list.").into());
	}

	let num_signers = signers_raw.len() / 20;
	let mut signers_list: Vec<Address> = Vec::with_capacity(num_signers);

	for i in 0..num_signers {
		let mut signer = Address::default();
		signer.copy_from_slice(&signers_raw[i * 20..(i + 1) * 20]);
		signers_list.push(signer);
	}

	// NOTE: signers list must be sorted by ascending order.
	signers_list.sort();

	Ok(signers_list)
}


const step_time: Duration = Duration::from_millis(100);

impl Clique {
	/// initialize Clique engine from empty state.
	pub fn new(our_params: CliqueParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
		let engine = Arc::new(
			Clique {
				epoch_length: our_params.epoch,
				period: our_params.period,
				client: RwLock::new(Option::default()),
				block_state_by_hash: RwLock::new(LruCache::new(STATE_CACHE_NUM)),
				signer: RwLock::new(Default::default()),
				active_prop_delay: None,
				machine,
				step_service: IoService::<Duration>::start()?,
			});

		let handler = StepService::new(Arc::downgrade(&engine) as Weak<Engine<_>>, step_time);
		engine.step_service.register_handler(Arc::new(handler))?;

		return Ok(engine);
	}

	fn sign_header(&self, header: &Header) -> Result<(Signature, H256), Error> {
		let digest = sig_hash(header)?;

		match (*self.signer.read()).sign(digest) {
			Ok(sig) => { Ok((sig, digest)) }
			Err(e) => { Err(From::from("failed to sign header")) }
		}
	}

	/// Construct an new state from given checkpoint header.
	#[inline]
	fn new_checkpoint_state(&self, header: &Header) -> Result<CliqueBlockState, Error> {
		assert_eq!(header.number() % self.epoch_length, 0);

		let state = CliqueBlockState::new(
			match header.number() {
				0 => NULL_AUTHOR,
				_ => recover_creator(header)?,
			},
			extract_signers(header)?);

		Ok(state)
	}

	/// get CliqueBlockState for given header, backfill from last checkpoint if needed.
	fn state(&self, header: &Header) -> Result<CliqueBlockState, Error> {
		let mut block_state_by_hash = self.block_state_by_hash.write();
		if let Some(state) = block_state_by_hash.get_mut(&header.hash()) {
			return Ok(state.clone());
		}
		if header.number() % self.epoch_length == 0 {
			let state = self.new_checkpoint_state(header)?;
			block_state_by_hash.insert(header.hash().clone(), state.clone());
			return Ok(state);
		}
		// parent BlockState is not found in memory, which means we need to reconstruct state from last
		// checkpoint.
		match self.client.read().as_ref().and_then(|w| { w.upgrade() }) {
			None => {
				return Err(From::from("failed to upgrade client reference"));
			}
			Some(c) => {
				let last_checkpoint_number = (header.number() / self.epoch_length as u64) * self.epoch_length;
				assert_ne!(last_checkpoint_number, header.number());

				let mut chain: &mut VecDeque<Header> = &mut VecDeque::with_capacity((header.number() - last_checkpoint_number + 1) as usize);

				// Put ourselves in.
				chain.push_front(header.clone());

				// populate chain to last checkpoint
				let mut last = chain.front().unwrap().clone();

				while last.number() != last_checkpoint_number + 1 {
					match c.block_header(BlockId::Hash(*last.parent_hash())) {
						None => {
							return Err(From::from(format!("parent block {} of {} could not be recovered.",
							                              &last.parent_hash(),
							                              &last.hash())));
						}
						Some(next) => {
							chain.push_front(next.decode().unwrap().clone());
							last = chain.front().unwrap().clone();
						}
					}
				}

				// Catching up state, note that we don't really store block state for intermediary blocks,
				// for speed.
				trace!(target: "engine",
				       "Back-filling block state. last_checkpoint_number: {}, current_number: {}.",
				       last_checkpoint_number, header.number());

				// Get the state for last checkpoint.
				let last_checkpoint_hash = *(chain.front().unwrap().parent_hash());
				let last_checkpoint_header = match c.block_header(BlockId::Hash(last_checkpoint_hash)) {
					None => return Err(From::from("Unable to find last checkpoint block")),
					Some(header) => header.decode().unwrap(),
				};

				let last_checkpoint_state: CliqueBlockState;

				// We probably don't have it cached, but try anyway.
				if let Some(st) = block_state_by_hash.get_mut(&last_checkpoint_hash) {
					last_checkpoint_state = (*st).clone();
				} else {
					last_checkpoint_state = self.new_checkpoint_state(&last_checkpoint_header)?;
				}
				block_state_by_hash.insert(last_checkpoint_header.hash().clone(),last_checkpoint_state.clone());

				// Backfill!
				let mut new_state = last_checkpoint_state.clone();
				for item in chain {
					new_state.apply(item, false)?;
				}
				block_state_by_hash.insert(header.hash(), new_state.clone());

				Ok(new_state)
			}
		}
	}
}

impl Engine<EthereumMachine> for Clique {
	fn name(&self) -> &str { "Clique" }

	fn machine(&self) -> &EthereumMachine { &self.machine }
	/// Clique use same fields, nonce + mixHash
	fn seal_fields(&self, _header: &Header) -> usize { 2 }
	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 0 }

//	fn seal_header(&self, header: &mut Header) {
//		trace!(target: "seal", "sealed header");
//
//		let mut state = self.state.write();
//		match state.proposer_authorization(header) {
//			SignerAuthorization::InTurn => {
//				header.set_difficulty(U256::from(DIFF_INTURN));
//			}
//			SignerAuthorization::OutOfTurn => {
//				header.set_difficulty(U256::from(DIFF_NOT_INTURN));
//			}
//			SignerAuthorization::Unauthorized => {
//				panic!("sealed header should be authorized to sign");
//			}
//		}
//
//		let signers = state.state(&header.parent_hash()).unwrap().signers;
//		let mut seal: Vec<u8> = vec![0; SIGNER_VANITY_LENGTH as usize + SIGNER_SIG_LENGTH as usize];
//
//		let mut sig_offset = SIGNER_VANITY_LENGTH as usize;
//
//		if header.number() % self.epoch_length == 0 {
//			sig_offset += 20 * signers.len();
//			for i in 0..signers.len() {
//				seal[SIGNER_VANITY_LENGTH as usize + i * 20..SIGNER_VANITY_LENGTH as usize + (i + 1) * 20].clone_from_slice(&signers[i]);
//			}
//		}
//
//		header.set_extra_data(seal.clone());
//
//		let (sig, msg) = self.sign_header(&header).expect("should be able to sign header");
//		seal[sig_offset..].copy_from_slice(&sig[..]);
//		header.set_extra_data(seal.clone());
//
//		state.apply(&header).unwrap();
//	}

//
//
//		{
//			let signers = self.state.get_signers();
//			trace!(target: "engine", "applied.  found {} signers", signers.len());
//
//			//let mut v: Vec<u8> = vec![0; SIGNER_VANITY_LENGTH as usize+SIGNER_SIG_LENGTH as usize];
//			let mut sig_offset = SIGNER_VANITY_LENGTH as usize;
//
//			if _header.number() % self.epoch_length == 0 {
//				sig_offset += 20 * signers.len();
//
//				for i in 0..signers.len() {
//					v[SIGNER_VANITY_LENGTH as usize + i * 20..SIGNER_VANITY_LENGTH as usize + (i + 1) * 20].clone_from_slice(&signers[i]);
//				}
//			}
//
//			h.set_extra_data(v.clone());
//
//			let (sig, msg) = self.sign_header(&h).expect("should be able to sign header");
//			v[sig_offset..].copy_from_slice(&sig[..]);
//
//			trace!(target: "engine", "header hash: {}", h.hash());
//			trace!(target: "engine", "Sig: {}", sig);
//			trace!(target: "engine", "Message: {:02x}", msg.iter().format(""));
//
//			//trace!(target: "engine", "we are {}", self.signer.read().address().unwrap());
//		}
//
//		return Some(v);
//	}

	fn on_new_block(
		&self,
		_block: &mut ExecutedBlock,
		_epoch_begin: bool,
		_ancestry: &mut Iterator<Item=ExtendedHeader>,
	) -> Result<(), Error> {
//trace!(target: "engine", "new block {}", _block.header().number());

		/*
		if let Some(ref mut snapshot) = *self.snapshot.write() {
			snapshot.rollback();
		} else {
			panic!("could not get write access to snapshot");
		}
		*/

		/*
		if let Some(ref mut snapshot) = *self.snapshot.write() {
			snapshot.apply(_block.header());
		}
		*/

		Ok(())
	}

	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		/*
		 * TODO:
		if not checkpoint block:
		if the block was successfully sealed, then grab the signature from the seal data and
		append it to the block extraData
		*/
// trace!(target: "engine", "closing block {}...", block.header().number());

		Ok(())
	}

	/// None means that it requires external input (e.g. PoW) to seal a block.
	/// /// Some(true) means the engine is currently prime for seal generation (i.e. node
	///     is the current validator).
	/// /// Some(false) means that the node might seal internally but is not qualified
	///     now.
	///
	fn seals_internally(&self) -> Option<bool> {
		Some(true)
	}

	/// Attempt to seal generate a proposal seal.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which case
	/// `Seal::None` will be returned.
	fn generate_seal(&self, block: &ExecutedBlock, _parent: &Header) -> Seal {
		trace!(target: "engine", "tried to generate seal");
//
//		let mut header = block.header.clone();
//
//		trace!(target: "engine", "attempting to seal...");
//
//		// don't seal the genesis block
		if block.header.number() == 0 {
			trace!(target: "engine", "attempted to seal genesis block");
			return Seal::None;
		}

//		// if sealing period is 0, refuse to seal
		if self.period == 0 {
			return Seal::None;
		}
//
// let vote_snapshot = self.snapshot.get(bh);
//
//		// if we are not authorized to sign, don't seal
//
//		// if we signed recently, don't seal
//
		if block.header.timestamp() <= _parent.timestamp() + self.period {
			return Seal::None;
		}

//ensure the voting state exists
		self.state(&_parent).unwrap();

//		let mut state = self.state.write();

//		match state.proposer_authorization(&block.header) {
//			SignerAuthorization::Unauthorized => {
//				trace!(target: "engine", "tried to seal: not authorized");
//				return Seal::None;
//			}
//			SignerAuthorization::InTurn => {
//				trace!(target: "engine", "seal generated for {}", block.header.number());
//				return Seal::Regular(vec![encode(&vec![0; 32]), encode(&vec![0; 8])]);
//			}
//			SignerAuthorization::OutOfTurn => {
//				if state.turn_delay(&block.header) {
//					trace!(target: "engine", "seal generated for {}", block.header.number());
//					return Seal::Regular(vec![encode(&vec![0; 32]), encode(&vec![0; 8])]);
//				} else {
//					trace!(target: "engine", "not in turn. seal delayed for {}", block.header.number());
//					return Seal::None;
//				}
//			}
//		}
		Seal::None
	}

	fn verify_local_seal(&self, header: &Header) -> Result<(), Error> { Ok(()) }

	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
		// Largely same as https://github.com/ethereum/go-ethereum/blob/master/consensus/clique/clique.go#L275

		// Ignore genesis block.
		if header.number() == 0 {
			return Ok(());
		}

		// Don't waste time checking blocks from the future
		if header.timestamp() > SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() {
			return Err(Box::new("block in the future").into());
		}

		let is_checkpoint = header.number() % self.epoch_length == 0;

		if is_checkpoint && *header.author() != NULL_AUTHOR {
			return Err(Box::new("Checkpoint block must enforce zero beneficiary").into());
		}

		// Nonce must be 0x00..0 or 0xff..f
		let nonce = header.decode_seal::<Vec<&[u8]>>()?[1];
		if *nonce != NONCE_DROP_VOTE && *nonce != NONCE_AUTH_VOTE {
			return Err(Box::new("nonce must be 0x00..0 or 0xff..f").into());
		}
		if is_checkpoint && *nonce != NULL_NONCE[..] {
			return Err(Box::new("Checkpoint block must have zero nonce").into());
		}

		// Ensure that the extra-data contains a signer list on checkpoint, but none otherwise.
		if (!is_checkpoint && header.extra_data().len() != (SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH))
			|| (is_checkpoint && header.extra_data().len() <= (SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH)) {
			return Err(Box::new("Invalid extra_data length").into());
		}

		// Ensure that the mix digest is zero as we don't have fork protection currently
		let mixhash = header.decode_seal::<Vec<&[u8]>>()?[0];
		if mixhash != NULL_MIXHASH {
			return Err(Box::new("mixhash must be 0x00..0 or 0xff..f.").into())
		}

		// Ensure that the block doesn't contain any uncles which are meaningless in PoA
		if *header.uncles_hash() != NULL_UNCLES_HASH {
			return Err(Box::new(format!(
				"Invalid uncle hash, got: {}, expected: {}.",
				header.uncles_hash(),
				NULL_UNCLES_HASH,
			)).into())
		}

		// Ensure that the block's difficulty is meaningful (may not be correct at this point)
		if *header.difficulty() != DIFF_INTURN && *header.difficulty() != DIFF_NOT_INTURN {
			return Err(Box::new(format!(
				"invalid difficulty: expected {} or {}, got: {}.",
				DIFF_INTURN,
				DIFF_NOT_INTURN,
				header.difficulty(),
			)).into())
		}

		// All basic checks passed, continue to next phase
		Ok(())
	}

	fn verify_block_unordered(&self, _header: &Header) -> Result<(), Error> {
		// Nothing to check here.
		Ok(())
	}

	/// Verify block family by looking up parent state (backfill if needed), then try to apply current header.
	/// see https://github.com/ethereum/go-ethereum/blob/master/consensus/clique/clique.go#L338
	fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
		// Ignore genesis block.
		if header.number() == 0 {
			return Ok(());
		}

		// parent sanity check
		if parent.hash() != *header.parent_hash() || header.number() != parent.number() + 1 {
			return Err(Box::new("invalid parent").into());
		}

		// Ensure that the block's timestamp isn't too close to it's parent
		if parent.timestamp() + self.period > header.timestamp() {
			return Err(Box::new("invalid timestamp").into());
		}

		// Retrieve the parent state
		let parent_state = self.state(&parent)?;

		// Try to apply current state, apply() will further check signer and recent signer.
		let mut new_state = parent_state.clone();
		new_state.apply(header, header.number() % self.epoch_length == 0)?;
		self.block_state_by_hash.write().insert(header.hash(), new_state);

		Ok(())
	}

//	fn on_block_applied(&self, header: &Header) -> Result<(), Error> {
//		self.snapshot.apply(&header);
//		self.snapshot.commit();
//
//		Ok(())
//	}

	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		let state = CliqueBlockState::new(
			NULL_AUTHOR,
			extract_signers(header).expect("Unable to recover signers"),
		);
		self.block_state_by_hash.write().insert(header.hash(), state);
		Ok(Vec::new())
	}

	fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData)
	                     -> super::EpochChange<EthereumMachine>
	{
		super::EpochChange::No
	}

	fn is_epoch_end(
		&self,
		chain_head: &Header,
		_finalized: &[H256],
		_chain: &Headers<Header>,
		_transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		None
	}

	fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {
		ConstructedVerifier::Trusted(Box::new(super::epoch::NoOp))
	}

	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {
		trace!(target: "engine", "called set_signer");
		self.signer.write().set(ap, address, password);
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		*self.client.write() = Some(client.clone());
	}

	fn step(&self) {
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				c.update_sealing();
			}
		}
	}

	fn stop(&self) {}

	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp >= parent_timestamp + self.period
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
		super::total_difficulty_fork_choice(new, current)
	}

//	/// Check if current signer is the current proposer.
//	fn is_signer_proposer(&self, bn: u64) -> bool {
//		let mut authorized = false;
//
//		let address = match self.snapshot.signer_address() {
//			Some(addr) => { addr }
//			None => { return false; }
//		};
//
//		let signers = self.snapshot.get_signers();
//
//		let authorized = if let Some(pos) = signers.iter().position(|x| self.snapshot.signer_address().unwrap() == *x) {
//			bn % signers.len() as u64 == pos as u64
//		} else {
//			false
//		};
//		return authorized;
//	}

	fn executive_author(&self, header: &Header) -> Address {
		// Should have been verified now.
		return recover_creator(header).unwrap();
	}
}

//impl CliqueState {
//	pub fn new(epoch_length: u64) -> Self {
//		CliqueState {
//			epoch_length: epoch_length,
//			states: RwLock::new(Default::default()),
//		}
//	}
//	/// Get an valid block state
//	pub fn state(&mut self, hash: &H256) -> Option<CliqueBlockState> {
//		let db = self.states_by_hash.borrow_mut();
//		return db.get_mut(hash).cloned();
//	}
//
//	pub fn turn_delay(&mut self, header: &Header) -> bool {
//		match self.active_prop_delay {
//			Some((parent_hash, start, duration)) => {
//				if *header.parent_hash() != parent_hash {
//					// reorg.  make sure the timer is reset
//					self.active_prop_delay = Some((header.parent_hash().clone(),
//					                               SystemTime::now(),
//					                               Duration::from_millis(thread_rng().gen_range::<u64>(0, self.state(header.parent_hash()).unwrap().signers.len() as u64 * 500))));
//					return false;
//				}
//
//				if start.elapsed().expect("start delay was after current time") >= duration {
//					return true;
//				} else {
//					return false;
//				}
//			}
//			None => {
//				self.active_prop_delay = Some((header.parent_hash().clone(),
//				                               SystemTime::now(),
//				                               Duration::from_millis(thread_rng().gen_range::<u64>(0, self.state(header.parent_hash()).unwrap().signers.len() as u64 * 500))));
//				return false;
//			}
//		}
//	}
//
//	/// Apply an new header
//	pub fn apply(&mut self, header: &Header) -> Result<(), Error> {
//		let db = self.states_by_hash.borrow_mut();
//
//		// make sure current hash is not in the db
//		match db.get_mut(header.parent_hash()).cloned() {
//			Some(ref mut new_state) => {
//				let creator = match process_header(&header, new_state, self.epoch_length) {
//					Err(e) => {
//						return Err(From::from(
//							format!("Error applying header: {}, current state: {:?}", e, new_state)
//						));
//					}
//					Ok(creator) => { creator }
//				};
//
//				db.insert(header.hash(), new_state.clone());
//				Ok(())
//			}
//			None => {
//				Err(From::from(
//					format!("Parent block (hash: {}) for Block {}, hash {} is not found!",
//					        header.parent_hash(),
//					        header.number(), header.hash())))
//			}
//		}
//	}
//
//	pub fn apply_checkpoint(&mut self, header: &Header) -> Result<(), Error> {
//		let db = self.states.write().borrow_mut();
//		let state = &mut CliqueBlockState {
//			votes: HashMap::new(),
//			votes_history: Vec::new(),
//			signers: Vec::new(),
//			recent_signers: VecDeque::new(),
//		};
//		process_genesis_header(header, state)?;
//
//		trace!("inserting {} {:?}", header.hash(), &state);
//		db.insert(header.hash(), state.clone());
//
//		Ok(())
//	}
//
//	pub fn set_signer_address(&self, signer_address: Address) {
//		trace!(target: "engine", "setting signer {}", signer_address);
//		*self.signer.write() = Some(signer_address.clone());
//	}
//
//	pub fn proposer_authorization(&mut self, header: &Header) -> SignerAuthorization {
//		let mut db = self.states_by_hash.borrow_mut();
//
//		let proposer_address = match *self.signer.read() {
//			Some(address) => address.clone(),
//			None => { return SignerAuthorization::Unauthorized; }
//		};
//
//		match db.get_mut(header.parent_hash()).cloned() {
//			Some(ref state) => {
//				return state.get_signer_authorization(header.number(), &proposer_address);
//			}
//			None => {
//				panic!("Parent block (hash: {}) for Block {}, hash {} is not found!",
//				       header.parent_hash(),
//				       header.number(), header.hash())
//			}
//		}
//	}
//}
