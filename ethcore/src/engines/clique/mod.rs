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

//! Implementation of Clique (POA) Engine.
//!
//! mod.rs -> CliqueEngine, the engine api implementation, with additional block state tracking.
//! block_state.rs -> CliqueBlockState , record clique state for given block.
//! param.rs -> Clique Params.
//! step_service.rs -> an event loop to trigger sealing.
//! util.rs -> various standalone util functions
//!

/// How syncing code path works:
/// 1. Client calls `engine.verify_block_basic()` then `engine.verify_block_unordered()`.
/// 2. Client calls `engine.verify_block_family(header, parent)`.
/// 3. Engine first needs to find parent state: `state = self.state(parent.hash())`
///    if not found, trigger a back-fill from last checkpoint.
/// 4. Engine calls `state.apply(header)` and record the new state.

/// About executive_author()
/// Clique use author field for voting, the real author is hidden in the extra_data field. So
/// When executing transactions (in `enact()`, it will calls engine.executive_author() and use that.

/// How sealing works:
/// 1. implement `engine.set_signer()`. on startup, if miner account was setup on config/cli,
///    `miner.set_author()` which will eventually be pass to here.
/// 2. make `engine.seals_internally()` return Some(true).
/// 3. on Clique::new setup IOService that implement an timer that just calls `engine.step()`,
///    which just calls `engine.client.update_sealing()` which triggers generating an new block.
/// 4. `engine.generate_seal()` will be called by miner, which should return either Seal::None or Seal:Regular.
///   a. return `Seal::None` if no signer is available or no signer is not authorized.
///   b. if period == 0 and block has transactions -> Seal::Regular, else Seal::None
///   c. if we INTURN, wait for at least `period` since last block, otherwise wait for an random using algorithm as
///      specified in the EIP.
/// 5. Miner will create new block, in process it will call several engine method, which they need to do following:
///   a. `engine_open_header_timestamp()` must set timestamp correctly.
///   b. `engine.populate_from_parent()` must set difficulty to correct value. NOTE: this is used both in SYNCing and
///       SEALing code path, for now we use an ugly hack to differentiate.
/// 6. Implement `engine.on_seal_block()`, which is the new hook that allow modifying header after block is locked.
/// 7. `engine.verify_local_seal()` will later be called, then normal syncing code path will also be called to import
///    the new block.

use std::cmp;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Weak};
use std::thread;
use std::time;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use block::{ExecutedBlock, IsBlock};
use client::{BlockId, EngineClient};
use engines::clique::util::{extract_signers, recover_creator};
use engines::{Engine, Seal};
use error::Error;
use ethereum_types::{Address, H160, H256, U256};
use ethkey::Signature;
use hash::KECCAK_EMPTY_LIST_RLP;
use itertools::Itertools;
use lru_cache::LruCache;
use machine::{Call, EthereumMachine};
use parking_lot::RwLock;
use rand::Rng;
use rlp::encode;
use super::signer::EngineSigner;
use types::BlockNumber;
use types::header::{ExtendedHeader, Header};

use self::block_state::CliqueBlockState;
use self::params::CliqueParams;
use self::step_service::StepService;

mod params;
mod block_state;
mod step_service;
mod util;

#[cfg(test)]
mod tests;

// protocol constants
/// Fixed number of extra-data prefix bytes reserved for signer vanity
pub const VANITY_LENGTH: usize = 32;
/// Fixed number of extra-data suffix bytes reserved for signer signature
pub const SIGNATURE_LENGTH: usize = 65;
/// Address length of signer
pub const ADDRESS_LENGTH: usize = 20;
/// Nonce value for DROP vote
pub const NONCE_DROP_VOTE: &[u8] = &[0x00; 8];
/// Nonce value for AUTH vote
pub const NONCE_AUTH_VOTE: &[u8] = &[0xff; 8];
/// Difficulty for INTURN block
pub const DIFF_INTURN: U256 = U256([2, 0, 0, 0]);
/// Difficulty for NOTURN block
pub const DIFF_NOTURN: U256 = U256([1, 0, 0, 0]);
/// Default empty author field value
pub const NULL_AUTHOR: Address = H160([0x00; 20]);
/// Default empty nonce value
pub const NULL_NONCE: &[u8] = NONCE_DROP_VOTE;
/// Default value for mixhash
pub const NULL_MIXHASH: &[u8] = &[0x00; 32];
/// Default value for uncles hash
pub const NULL_UNCLES_HASH: H256 = KECCAK_EMPTY_LIST_RLP;
/// Default noturn block wiggle factor defined in spec.
pub const SIGNING_DELAY_NOTURN_MS: u64 = 500;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum VoteType {
	Add,
	Remove,
}

impl VoteType {
	pub fn from_nonce(nonce: &[u8]) -> Result<Self, Error> {
		match nonce {
			NONCE_AUTH_VOTE => Ok(VoteType::Add),
			NONCE_DROP_VOTE => Ok(VoteType::Remove),
			_ => Err(From::from("nonce was not AUTH or DROP"))
		}
	}

	pub fn as_rlp(&self) -> Vec<Vec<u8>> {
		match self {
			VoteType::Add => vec![encode(&NULL_MIXHASH.to_vec()), encode(&NONCE_AUTH_VOTE.to_vec())],
			VoteType::Remove => vec![encode(&NULL_MIXHASH.to_vec()), encode(&NONCE_DROP_VOTE.to_vec())],
		}
	}
}

// Caches
/// How many CliqueBlockState to cache in the memory.
pub const STATE_CACHE_NUM: usize = 128;

/// Clique Engine implementation
/// block_state_by_hash -> block state indexed by header hash.
#[cfg(not(test))]
pub struct Clique {
	epoch_length: u64,
	period: u64,
	machine: EthereumMachine,
	client: RwLock<Option<Weak<EngineClient>>>,
	block_state_by_hash: RwLock<LruCache<H256, CliqueBlockState>>,
	proposals: RwLock<HashMap<Address, VoteType>>,
	signer: RwLock<Option<Box<EngineSigner>>>,
	step_service: Option<Arc<StepService>>,
}

#[cfg(test)]
/// Test version of `CliqueEngine` to make all fields public
pub struct Clique {
	pub epoch_length: u64,
	pub period: u64,
	pub machine: EthereumMachine,
	pub client: RwLock<Option<Weak<EngineClient>>>,
	pub block_state_by_hash: RwLock<LruCache<H256, CliqueBlockState>>,
	pub proposals: RwLock<HashMap<Address, VoteType>>,
	pub signer: RwLock<Option<Box<EngineSigner>>>,
	pub step_service: Option<Arc<StepService>>,
}

impl Clique {
	/// Initialize Clique engine from empty state.
	pub fn new(our_params: CliqueParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
		let mut engine = Clique {
			epoch_length: our_params.epoch,
			period: our_params.period,
			client: Default::default(),
			block_state_by_hash: RwLock::new(LruCache::new(STATE_CACHE_NUM)),
			proposals: Default::default(),
			signer: Default::default(),
			machine,
			step_service: None,
		};

		let res = Arc::new(engine);

		if our_params.period > 0 {
			engine.step_service = Some(StepService::start(Arc::downgrade(&res) as Weak<Engine<_>>));
		}

		Ok(res)
	}

	#[cfg(test)]
	/// Initialize test variant of `CliqueEngine`,
	/// Note we need to `mock` miner and is introduced to test block verification to trigger new blocks
	/// to test consensus edge cases
	pub fn with_test(epoch_length: u64, period: u64) -> Self {
		use spec::Spec;

		Self {
			epoch_length,
			period,
			client: Default::default(),
			block_state_by_hash: RwLock::new(LruCache::new(STATE_CACHE_NUM)),
			proposals: Default::default(),
			signer: Default::default(),
			machine: Spec::new_test_machine(),
			step_service: None,
		}
	}

	fn sign_header(&self, header: &Header) -> Result<(Signature, H256), Error> {
		match self.signer.read().as_ref() {
			None => {
				return Err(Box::new("sign_header: No signer available.").into());
			}
			Some(signer) => {
				let digest = header.hash();
				match signer.sign(digest) {
					Ok(sig) => {
						return Ok((sig, digest));
					}
					Err(e) => {
						return Err(Box::new(format!("sign_header: failed to sign header, error: {}", e)).into());
					}
				}
			}
		}
	}

	/// Construct an new state from given checkpoint header.
	fn new_checkpoint_state(&self, header: &Header) -> Result<CliqueBlockState, Error> {
		debug_assert_eq!(header.number() % self.epoch_length, 0);

		let mut state = CliqueBlockState::new(
			extract_signers(header)?);

		state.calc_next_timestamp(header, self.period);

		Ok(state)
	}

	fn state_no_backfill(&self, hash: &H256) -> Option<CliqueBlockState> {
		self.block_state_by_hash.write().get_mut(hash).cloned()
	}

	/// Get `CliqueBlockState` for given header, backfill from last checkpoint if needed.
	fn state(&self, header: &Header) -> Result<CliqueBlockState, Error> {
		let mut block_state_by_hash = self.block_state_by_hash.write();
		if let Some(state) = block_state_by_hash.get_mut(&header.hash()) {
			return Ok(state.clone());
		}
		// If we are looking for an checkpoint block state, we can directly reconstruct it.
		if header.number() % self.epoch_length == 0 {
			let state = self.new_checkpoint_state(header)?;
			block_state_by_hash.insert(header.hash(), state.clone());
			return Ok(state);
		}
		// BlockState is not found in memory, which means we need to reconstruct state from last checkpoint.
		match self.client.read().as_ref().and_then(|w| w.upgrade()) {
			None => {
				return Err(From::from("failed to upgrade client reference"));
			}
			Some(c) => {
				let last_checkpoint_number = header.number() - header.number() % self.epoch_length as u64;
				debug_assert_ne!(last_checkpoint_number, header.number());

				let mut chain: &mut VecDeque<Header> = &mut VecDeque::with_capacity(
					(header.number() - last_checkpoint_number + 1) as usize);

				// Put ourselves in.
				chain.push_front(header.clone());

				// populate chain to last checkpoint
				loop {
					let (last_parent_hash, last_hash, last_num) = {
						let l = chain.front().expect("chain has at least one element; qed");
						(*l.parent_hash(), l.hash(), l.number())
					};

					if last_num == last_checkpoint_number + 1 {
						break;
					}
					match c.block_header(BlockId::Hash(last_parent_hash)) {
						None => {
							return Err(Box::new(format!("parent block {} of {} could not be recovered.", last_parent_hash, last_hash)).into());
						}
						Some(next) => {
							chain.push_front(next.decode()?);
						}
					}
				}

				// Catching up state, note that we don't really store block state for intermediary blocks,
				// for speed.
				let backfill_start = time::Instant::now();
				info!(target: "engine",
						"Back-filling block state. last_checkpoint_number: {}, target: {}({}).",
						last_checkpoint_number, header.number(), header.hash());

				// Get the state for last checkpoint.
				let last_checkpoint_hash = *(chain.front().ok_or(
					"just pushed to front, reference must exist; qed"
				)?.parent_hash());
				let last_checkpoint_header = match c.block_header(BlockId::Hash(last_checkpoint_hash)) {
					None => return Err(From::from("Unable to find last checkpoint block")),
					Some(header) => header.decode()?,
				};

				let last_checkpoint_state = match block_state_by_hash.get_mut(&last_checkpoint_hash) {
					Some(state) => state.clone(),
					None => self.new_checkpoint_state(&last_checkpoint_header)?,
				};

				block_state_by_hash.insert(last_checkpoint_header.hash(), last_checkpoint_state.clone());

				// Backfill!
				let mut new_state = last_checkpoint_state.clone();
				for item in chain {
					new_state.apply(item, false)?;
				}
				new_state.calc_next_timestamp(header, self.period);
				block_state_by_hash.insert(header.hash(), new_state.clone());

				let elapsed = backfill_start.elapsed();
				info!(target: "engine",
						"Back-filling succeed, took {} ms.",
						// replace with Duration::as_millis after rust 1.33
						elapsed.as_secs() as u128 * 1000 + elapsed.subsec_millis() as u128,
				);

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

	// No Uncle in Clique
	fn maximum_uncle_age(&self) -> usize { 0 }

	fn on_new_block(
		&self,
		_block: &mut ExecutedBlock,
		_epoch_begin: bool,
		_ancestry: &mut Iterator<Item=ExtendedHeader>,
	) -> Result<(), Error> {
		Ok(())
	}

	fn on_close_block(&self, _block: &mut ExecutedBlock) -> Result<(), Error> {
		// Clique has no block reward.
		Ok(())
	}

	fn on_seal_block(&self, block: &ExecutedBlock) -> Result<Option<Header>, Error> {
		trace!(target: "engine", "on_seal_block");

		let mut header = block.header().clone();

		let state = self.state_no_backfill(header.parent_hash()).ok_or_else(
			|| format!("on_seal_block: Unable to get parent state: {}", header.parent_hash())
		)?;

		let is_checkpoint = header.number() % self.epoch_length == 0;

		header.set_author(NULL_AUTHOR);

		// cast an random Vote if not checkpoint
		if !is_checkpoint {
			// TODO(niklasad1): this will always be false because `proposals` is never written to
			let votes = self.proposals.read().iter()
				.filter(|(address, vote_type)| state.is_valid_vote(*address, **vote_type))
				.map(|(address, vote_type)| (*address, *vote_type))
				.collect_vec();

			if !votes.is_empty() {
				// Pick an random vote.
				let random_vote = rand::thread_rng().gen_range(0 as usize, votes.len());
				let (beneficiary, vote_type) = votes[random_vote];

				trace!(target: "engine", "Casting vote: beneficiary {}, type {:?} ", beneficiary, vote_type);

				header.set_author(beneficiary);
				header.set_seal(vote_type.as_rlp());
			}
		}

		// Work on clique seal.

		let mut seal: Vec<u8> = Vec::with_capacity(VANITY_LENGTH + SIGNATURE_LENGTH);

		// At this point, extra_data should only contain miner vanity.
		if header.extra_data().len() > VANITY_LENGTH {
			panic!("on_seal_block: unexpected extra_data: {:?}", header);
		}
		// vanity
		{
			seal.extend_from_slice(&header.extra_data()[0..VANITY_LENGTH]);
		}

		// If we are building an checkpoint block, add all signers now.
		if is_checkpoint {
			seal.reserve(state.signers().len() * 20);
			state.signers().iter().foreach(|addr| {
				seal.extend_from_slice(&addr[..]);
			});
		}

		header.set_extra_data(seal.clone());

		// append signature onto extra_data
		let (sig, _msg) = self.sign_header(&header)?;
		seal.extend_from_slice(&sig[..]);
		header.set_extra_data(seal.clone());

		header.compute_hash();

		// locally sealed block don't go through valid_block_family(), so we have to record state here.
		let mut new_state = state.clone();
		new_state.apply(&header, is_checkpoint)?;
		new_state.calc_next_timestamp(&header, self.period);
		self.block_state_by_hash.write().insert(header.hash(), new_state);

		trace!(target: "engine", "on_seal_block: finished, final header: {:?}", header);

		Ok(Some(header))
	}

	/// Clique doesn't require external work to seal, so we always return true here.
	fn seals_internally(&self) -> Option<bool> {
		Some(true)
	}

	/// Returns if we are ready to seal, the real sealing (signing extra_data) is actually done in `on_seal_block()`.
	fn generate_seal(&self, block: &ExecutedBlock, parent: &Header) -> Seal {
		trace!(target: "engine", "tried to generate_seal");
		let null_seal = util::null_seal();

		if block.header.number() == 0 {
			trace!(target: "engine", "attempted to seal genesis block");
			return Seal::None;
		}

		// if sealing period is 0, and not an checkpoint block, refuse to seal
		if self.period == 0 {
			if block.transactions.is_empty() && block.header.number() % self.epoch_length != 0 {
				return Seal::None;
			}
			return Seal::Regular(null_seal);
		}

		// Check we actually have authority to seal.
		if let Some(author) = self.signer.read().as_ref().map(|x| x.address()) {

			// ensure the voting state exists
			match self.state(&parent) {
				Err(e) => {
					warn!(target: "engine", "generate_seal: can't get parent state(number: {}, hash: {}): {} ",
							parent.number(), parent.hash(), e);
					return Seal::None;
				}
				Ok(state) => {
					// Are we authorized to seal?
					if !state.is_authorized(&author) {
						trace!(target: "engine", "generate_seal: Not authorized to sign right now.");
						// wait for one third of period to try again.
						thread::sleep(Duration::from_secs(self.period / 3 + 1));
						return Seal::None;
					}

					let inturn = state.is_inturn(block.header.number(), &author);

					let now = SystemTime::now();

					let limit = match inturn {
						true => state.next_timestamp_inturn.unwrap_or(now),
						false => state.next_timestamp_noturn.unwrap_or(now),
					};

					// Wait for the right moment.
					if now < limit {
						trace!(target: "engine",
								"generate_seal: sleeping to sign: inturn: {}, now: {:?}, to: {:?}.",
								inturn, now, limit);
						match limit.duration_since(SystemTime::now()) {
							Ok(duration) => {
								thread::sleep(duration);
							},
							Err(e) => {
								warn!(target:"engine", "generate_seal: unable to sleep, err: {}", e);
								return Seal::None;
							}
						}
					}

					trace!(target: "engine", "generate_seal: seal ready for block {}, txs: {}.",
							block.header.number(), block.transactions.len());
					return Seal::Regular(null_seal);
				}
			}
		}
		Seal::None
	}

	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> { Ok(()) }

	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
		// Largely same as https://github.com/ethereum/go-ethereum/blob/master/consensus/clique/clique.go#L275

		// Ignore genesis block.
		if header.number() == 0 {
			return Ok(());
		}

		// Don't waste time checking blocks from the future
		{
			let limit = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() + self.period;
			if header.timestamp() > limit {
				return Err(Box::new(
					format!("Block is too far in the future, timestamp: {}, limit: {}", header.timestamp(), limit)
				).into());
			}
		}

		let is_checkpoint = header.number() % self.epoch_length == 0;

		if is_checkpoint && *header.author() != NULL_AUTHOR {
			return Err(Box::new("Checkpoint block must enforce zero beneficiary").into());
		}

		// Nonce must be 0x00..0 or 0xff..f
		let seal_fields = header.decode_seal::<Vec<_>>()?;
		let mixhash = seal_fields.get(0).ok_or("No mixhash field.")?;
		let nonce = seal_fields.get(1).ok_or("No nonce field.")?;
		if *nonce != NONCE_DROP_VOTE && *nonce != NONCE_AUTH_VOTE {
			return Err(Box::new("nonce must be 0x00..0 or 0xff..f").into());
		}
		if is_checkpoint && *nonce != NULL_NONCE {
			return Err(Box::new("Checkpoint block must have zero nonce").into());
		}

		// Ensure that the extra-data contains a signer list on checkpoint, but none otherwise.
		if (!is_checkpoint && header.extra_data().len() != (VANITY_LENGTH + SIGNATURE_LENGTH))
			|| (is_checkpoint && header.extra_data().len() <= (VANITY_LENGTH + SIGNATURE_LENGTH))
			|| (is_checkpoint && (header.extra_data().len() - (VANITY_LENGTH + SIGNATURE_LENGTH)) % ADDRESS_LENGTH != 0) {
			return Err(Box::new(format!("Invalid extra_data length, got {}", header.extra_data().len())).into());
		}

		// Ensure that the mix digest is zero as Clique don't have fork protection currently
		if *mixhash != NULL_MIXHASH {
			return Err(Box::new("mixhash must be 0x00..0 or 0xff..f.").into());
		}

		// Ensure that the block doesn't contain any uncles which are meaningless in PoA
		if *header.uncles_hash() != NULL_UNCLES_HASH {
			return Err(Box::new(format!(
				"Invalid uncle hash, got: {}, expected: {}.",
				header.uncles_hash(),
				NULL_UNCLES_HASH,
			)).into());
		}

		// Ensure that the block's difficulty is meaningful (may not be correct at this point)
		if *header.difficulty() != DIFF_INTURN && *header.difficulty() != DIFF_NOTURN {
			return Err(Box::new(format!(
				"invalid difficulty: expected {} or {}, got: {}.",
				DIFF_INTURN,
				DIFF_NOTURN,
				header.difficulty(),
			)).into());
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
		if parent.timestamp().saturating_add(self.period) > header.timestamp() {
			return Err(Box::new("invalid timestamp").into());
		}

		// Retrieve the parent state
		let parent_state = self.state(&parent)?;
		// Try to apply current state, apply() will further check signer and recent signer.
		let mut new_state = parent_state.clone();
		new_state.apply(header, header.number() % self.epoch_length == 0)?;
		new_state.calc_next_timestamp(header, self.period);
		self.block_state_by_hash.write().insert(header.hash(), new_state);

		Ok(())
	}

	fn genesis_epoch_data(&self, header: &Header, _call: &Call) -> Result<Vec<u8>, String> {
		let mut state = self.new_checkpoint_state(header).expect("Unable to parse genesis data.");
		state.calc_next_timestamp(header, self.period);
		self.block_state_by_hash.write().insert(header.hash(), state);

		// no proof.
		Ok(Vec::new())
	}

	// Our task here is to set difficulty
	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
		// TODO(https://github.com/paritytech/parity-ethereum/issues/10410): this is a horrible hack,
		// it is due to the fact that enact and miner both use OpenBlock::new() which will both call
		// this function. more refactoring is definitely needed.
		match header.extra_data().len() >= VANITY_LENGTH + SIGNATURE_LENGTH {
			true => {
				// we are importing blocks, do nothing.
			}
			false => {
				trace!(target: "engine", "populate_from_parent in sealing");

				// It's unclear how to prevent creating new blocks unless we are authorized, the best way (and geth does this too)
				// it's just to ignore setting an correct difficulty here, we will check authorization in next step in generate_seal anyway.
				if let Some(signer) = self.signer.read().as_ref() {
					match self.state(&parent) {
						Err(e) => {
							trace!(target: "engine", "populate_from_parent: Unable to find parent state: {}, ignored.", e);
						}
						Ok(state) => {
							if state.is_authorized(&signer.address()) {
								if state.is_inturn(header.number(), &signer.address()) {
									header.set_difficulty(DIFF_INTURN);
								} else {
									header.set_difficulty(DIFF_NOTURN);
								}
							}
						}
					}
				}
			}
		}
	}

	fn set_signer(&self, signer: Box<EngineSigner>) {
		trace!(target: "engine", "set_signer: {}", signer.address());

		*self.signer.write() = Some(signer);
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		*self.client.write() = Some(client.clone());
	}

	fn step(&self) {
		if self.signer.read().is_some() {
			if let Some(ref weak) = *self.client.read() {
				if let Some(c) = weak.upgrade() {
					c.update_sealing();
				}
			}
		}
	}

	fn stop(&mut self) {
		if let Some(mut s) = self.step_service.as_mut() {
			Arc::get_mut(&mut s).map(|x| x.stop());
		}
	}

	/// Clique timestamp is set to parent + period , or current time which ever is higher.
	fn open_block_header_timestamp(&self, parent_timestamp: u64) -> u64 {
		let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap_or_default();
		cmp::max(now.as_secs() as u64, parent_timestamp.saturating_add(self.period))
	}

	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp >= parent_timestamp.saturating_add(self.period)
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
		super::total_difficulty_fork_choice(new, current)
	}

	fn executive_author(&self, header: &Header) -> Address {
		// Should have been verified now.
		recover_creator(header).expect("Unable to extract creator.")
	}
}
