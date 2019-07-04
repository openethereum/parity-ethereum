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

//! Implementation of the Clique PoA Engine.
//!
//! File structure:
//! - mod.rs -> Provides the engine API implementation, with additional block state tracking
//! - block_state.rs -> Records the Clique state for given block.
//! - params.rs -> Contains the parameters for the Clique engine.
//! - step_service.rs -> An event loop to trigger sealing.
//! - util.rs -> Various standalone utility functions.
//! - tests.rs -> Consensus tests as defined in EIP-225.

/// How syncing works:
///
/// 1. Client will call:
///    - `Clique::verify_block_basic()`
///    - `Clique::verify_block_unordered()`
///    - `Clique::verify_block_family()`
/// 2. Using `Clique::state()` we try and retrieve the parent state. If this isn't found
///    we need to back-fill it from the last known checkpoint.
/// 3. Once we have a good state, we can record it using `CliqueBlockState::apply()`.

/// How sealing works:
///
/// 1. Set a signer using `Engine::set_signer()`. If a miner account was set up through
///    a config file or CLI flag `MinerService::set_author()` will eventually set the signer
/// 2. We check that the engine is ready for sealing through `Clique::sealing_state()`
///    Note: This is always `SealingState::Ready` for Clique
/// 3. Calling `Clique::new()` will spawn a `StepService` thread. This thread will call `Engine::step()`
///    periodically. Internally, the Clique `step()` function calls `Client::update_sealing()`, which is
///    what makes and seals a block.
/// 4. `Clique::generate_seal()` will then be called by `miner`. This will return a `Seal` which
///     is either a `Seal::None` or `Seal:Regular`. The following shows how a `Seal` variant is chosen:
///       a. We return `Seal::None` if no signer is available or the signer is not authorized.
///       b. If period == 0 and block has transactions, we return `Seal::Regular`, otherwise return `Seal::None`.
///       c. If we're `INTURN`, wait for at least `period` since last block before trying to seal.
///       d. If we're not `INTURN`, we wait for a random amount of time using the algorithm specified
///          in EIP-225 before trying to seal again.
/// 5. Miner will create new block, in process it will call several engine methods to do following:
///   a. `Clique::open_block_header_timestamp()` must set timestamp correctly.
///   b. `Clique::populate_from_parent()` must set difficulty to correct value.
///       Note: `Clique::populate_from_parent()` is used in both the syncing and sealing code paths.
/// 6. We call `Clique::on_seal_block()` which will allow us to modify the block header during seal generation.
/// 7. Finally, `Clique::verify_local_seal()` is called. After this, the syncing code path will be followed
///    in order to import the new block.

use std::cmp;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Weak};
use std::thread;
use std::time;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

use block::ExecutedBlock;
use client::{BlockId, EngineClient};
use engines::clique::util::{extract_signers, recover_creator};
use engines::{Engine, EngineError, Seal, SealingState};
use error::{BlockError, Error};
use ethereum_types::{Address, H64, H160, H256, U256};
use ethkey::Signature;
use hash::KECCAK_EMPTY_LIST_RLP;
use itertools::Itertools;
use lru_cache::LruCache;
use machine::{Call, Machine};
use parking_lot::RwLock;
use rand::Rng;
use super::signer::EngineSigner;
use unexpected::{Mismatch, OutOfBounds};
use time_utils::CheckedSystemTime;
use types::BlockNumber;
use types::header::{ExtendedHeader, Header};

use self::block_state::CliqueBlockState;
use self::params::CliqueParams;

mod params;
mod block_state;
mod util;

// TODO(niklasad1): extract tester types into a separate mod to be shared in the code base
#[cfg(test)]
mod tests;

// Protocol constants
/// Fixed number of extra-data prefix bytes reserved for signer vanity
pub const VANITY_LENGTH: usize = 32;
/// Fixed number of extra-data suffix bytes reserved for signer signature
pub const SIGNATURE_LENGTH: usize = 65;
/// Address length of signer
pub const ADDRESS_LENGTH: usize = 20;
/// Nonce value for DROP vote
pub const NONCE_DROP_VOTE: H64 = H64([0; 8]);
/// Nonce value for AUTH vote
pub const NONCE_AUTH_VOTE: H64 = H64([0xff; 8]);
/// Difficulty for INTURN block
pub const DIFF_INTURN: U256 = U256([2, 0, 0, 0]);
/// Difficulty for NOTURN block
pub const DIFF_NOTURN: U256 = U256([1, 0, 0, 0]);
/// Default empty author field value
pub const NULL_AUTHOR: Address = H160([0x00; 20]);
/// Default empty nonce value
pub const NULL_NONCE: H64 = NONCE_DROP_VOTE;
/// Default value for mixhash
pub const NULL_MIXHASH: H256 = H256([0; 32]);
/// Default value for uncles hash
pub const NULL_UNCLES_HASH: H256 = KECCAK_EMPTY_LIST_RLP;
/// Default noturn block wiggle factor defined in spec.
pub const SIGNING_DELAY_NOTURN_MS: u64 = 500;

/// How many CliqueBlockState to cache in the memory.
pub const STATE_CACHE_NUM: usize = 128;

/// Vote to add or remove the beneficiary
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum VoteType {
	Add,
	Remove,
}

impl VoteType {
	/// Try to construct a `Vote` from a nonce
	pub fn from_nonce(nonce: H64) -> Result<Self, Error> {
		if nonce == NONCE_AUTH_VOTE {
			Ok(VoteType::Add)
		} else if nonce == NONCE_DROP_VOTE {
			Ok(VoteType::Remove)
		} else {
			Err(EngineError::CliqueInvalidNonce(nonce))?
		}
	}

	/// Get the rlp encoding of the vote
	pub fn as_rlp(&self) -> Vec<Vec<u8>> {
		match self {
			VoteType::Add => vec![rlp::encode(&NULL_MIXHASH), rlp::encode(&NONCE_AUTH_VOTE)],
			VoteType::Remove => vec![rlp::encode(&NULL_MIXHASH), rlp::encode(&NONCE_DROP_VOTE)],
		}
	}
}

/// Clique Engine implementation
// block_state_by_hash -> block state indexed by header hash.
#[cfg(not(test))]
pub struct Clique {
	epoch_length: u64,
	period: u64,
	machine: Machine,
	client: RwLock<Option<Weak<dyn EngineClient>>>,
	block_state_by_hash: RwLock<LruCache<H256, CliqueBlockState>>,
	proposals: RwLock<HashMap<Address, VoteType>>,
	signer: RwLock<Option<Box<dyn EngineSigner>>>,
}

#[cfg(test)]
/// Test version of `CliqueEngine` to make all fields public
pub struct Clique {
	pub epoch_length: u64,
	pub period: u64,
	pub machine: Machine,
	pub client: RwLock<Option<Weak<dyn EngineClient>>>,
	pub block_state_by_hash: RwLock<LruCache<H256, CliqueBlockState>>,
	pub proposals: RwLock<HashMap<Address, VoteType>>,
	pub signer: RwLock<Option<Box<dyn EngineSigner>>>,
}

impl Clique {
	/// Initialize Clique engine from empty state.
	pub fn new(params: CliqueParams, machine: Machine) -> Result<Arc<Self>, Error> {
		/// Step Clique at most every 2 seconds
		const SEALING_FREQ: Duration = Duration::from_secs(2);

		let engine = Clique {
			epoch_length: params.epoch,
			period: params.period,
			client: Default::default(),
			block_state_by_hash: RwLock::new(LruCache::new(STATE_CACHE_NUM)),
			proposals: Default::default(),
			signer: Default::default(),
			machine,
		};
		let engine = Arc::new(engine);
		let weak_eng = Arc::downgrade(&engine);

		thread::Builder::new().name("StepService".into())
			.spawn(move || {
				loop {
 					let next_step_at = Instant::now() + SEALING_FREQ;
					trace!(target: "miner", "StepService: triggering sealing");
					if let Some(eng) = weak_eng.upgrade() {
						eng.step()
					} else {
						warn!(target: "shutdown", "StepService: engine is dropped; exiting.");
						break;
					}

					let now = Instant::now();
					if now < next_step_at {
						thread::sleep(next_step_at - now);
					}
				}
			})?;
		Ok(engine)
	}

	#[cfg(test)]
	/// Initialize test variant of `CliqueEngine`,
	/// Note we need to `mock` the miner and it is introduced to test block verification to trigger new blocks
	/// to mainly test consensus edge cases
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
		}
	}

	fn sign_header(&self, header: &Header) -> Result<(Signature, H256), Error> {

		match self.signer.read().as_ref() {
			None => {
				Err(EngineError::RequiresSigner)?
			}
			Some(signer) => {
				let digest = header.hash();
				match signer.sign(digest) {
					Ok(sig) => Ok((sig, digest)),
					Err(e) => Err(EngineError::Custom(e.into()))?,
				}
			}
		}
	}

	/// Construct an new state from given checkpoint header.
	fn new_checkpoint_state(&self, header: &Header) -> Result<CliqueBlockState, Error> {
		debug_assert_eq!(header.number() % self.epoch_length, 0);

		let mut state = CliqueBlockState::new(
			extract_signers(header)?);

		// TODO(niklasad1): refactor to perform this check in the `CliqueBlockState` constructor instead
		state.calc_next_timestamp(header.timestamp(), self.period)?;

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
				return Err(EngineError::RequiresClient)?;
			}
			Some(c) => {
				let last_checkpoint_number = header.number() - header.number() % self.epoch_length as u64;
				debug_assert_ne!(last_checkpoint_number, header.number());

				// Catching up state, note that we don't really store block state for intermediary blocks,
				// for speed.
				let backfill_start = time::Instant::now();
				trace!(target: "engine",
						"Back-filling block state. last_checkpoint_number: {}, target: {}({}).",
						last_checkpoint_number, header.number(), header.hash());

				let mut chain = VecDeque::with_capacity((header.number() - last_checkpoint_number + 1) as usize);

				// Put ourselves in.
				chain.push_front(header.clone());

				// populate chain to last checkpoint
				loop {
					let (last_parent_hash, last_num) = {
						let l = chain.front().expect("chain has at least one element; qed");
						(*l.parent_hash(), l.number())
					};

					if last_num == last_checkpoint_number + 1 {
						break;
					}
					match c.block_header(BlockId::Hash(last_parent_hash)) {
						None => {
							return Err(BlockError::UnknownParent(last_parent_hash))?;
						}
						Some(next) => {
							chain.push_front(next.decode()?);
						}
					}
				}

				// Get the state for last checkpoint.
				let last_checkpoint_hash = *chain.front()
					.expect("chain has at least one element; qed")
					.parent_hash();

				let last_checkpoint_header = match c.block_header(BlockId::Hash(last_checkpoint_hash)) {
					None => return Err(EngineError::CliqueMissingCheckpoint(last_checkpoint_hash))?,
					Some(header) => header.decode()?,
				};

				let last_checkpoint_state = match block_state_by_hash.get_mut(&last_checkpoint_hash) {
					Some(state) => state.clone(),
					None => self.new_checkpoint_state(&last_checkpoint_header)?,
				};

				block_state_by_hash.insert(last_checkpoint_header.hash(), last_checkpoint_state.clone());

				// Backfill!
				let mut new_state = last_checkpoint_state.clone();
				for item in &chain {
					new_state.apply(item, false)?;
				}
				new_state.calc_next_timestamp(header.timestamp(), self.period)?;
				block_state_by_hash.insert(header.hash(), new_state.clone());

				let elapsed = backfill_start.elapsed();
				trace!(target: "engine", "Back-filling succeed, took {} ms.", elapsed.as_millis());
				Ok(new_state)
			}
		}
	}
}

impl Engine for Clique {
	fn name(&self) -> &str { "Clique" }

	fn machine(&self) -> &Machine { &self.machine }

	// Clique use same fields, nonce + mixHash
	fn seal_fields(&self, _header: &Header) -> usize { 2 }

	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 0 }

	fn on_new_block(
		&self,
		_block: &mut ExecutedBlock,
		_epoch_begin: bool,
	) -> Result<(), Error> {
		Ok(())
	}

	// Clique has no block reward.
	fn on_close_block(
		&self,
		_block: &mut ExecutedBlock,
		_parent_header: &Header
	) -> Result<(), Error> {
		Ok(())
	}

	fn on_seal_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		trace!(target: "engine", "on_seal_block");

		let header = &mut block.header;

		let state = self.state_no_backfill(header.parent_hash())
			.ok_or_else(|| BlockError::UnknownParent(*header.parent_hash()))?;

		let is_checkpoint = header.number() % self.epoch_length == 0;

		header.set_author(NULL_AUTHOR);

		// Cast a random Vote if not checkpoint
		if !is_checkpoint {
			// TODO(niklasad1): this will always be false because `proposals` is never written to
			let votes = self.proposals.read().iter()
				.filter(|(address, vote_type)| state.is_valid_vote(*address, **vote_type))
				.map(|(address, vote_type)| (*address, *vote_type))
				.collect_vec();

			if !votes.is_empty() {
				// Pick a random vote.
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
		if header.extra_data().len() != VANITY_LENGTH {
			Err(BlockError::ExtraDataOutOfBounds(OutOfBounds {
				min: Some(VANITY_LENGTH),
				max: Some(VANITY_LENGTH),
				found: header.extra_data().len()
			}))?;
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
		new_state.calc_next_timestamp(header.timestamp(), self.period)?;
		self.block_state_by_hash.write().insert(header.hash(), new_state);

		trace!(target: "engine", "on_seal_block: finished, final header: {:?}", header);

		Ok(())
	}

	/// Clique doesn't require external work to seal, so we always return true here.
	fn sealing_state(&self) -> SealingState {
		SealingState::Ready
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
			let limit = CheckedSystemTime::checked_add(SystemTime::now(), Duration::from_secs(self.period))
				.ok_or(BlockError::TimestampOverflow)?;

			// This should succeed under the contraints that the system clock works
			let limit_as_dur = limit.duration_since(UNIX_EPOCH).map_err(|e| {
				Box::new(format!("Converting SystemTime to Duration failed: {}", e))
			})?;

			let hdr = Duration::from_secs(header.timestamp());
			if hdr > limit_as_dur {
				let found = CheckedSystemTime::checked_add(UNIX_EPOCH, hdr).ok_or(BlockError::TimestampOverflow)?;

				Err(BlockError::TemporarilyInvalid(OutOfBounds {
					min: None,
					max: Some(limit),
					found,
				}.into()))?
			}
		}

		let is_checkpoint = header.number() % self.epoch_length == 0;

		if is_checkpoint && *header.author() != NULL_AUTHOR {
			return Err(EngineError::CliqueWrongAuthorCheckpoint(Mismatch {
				expected: H160::zero(),
				found: *header.author(),
			}))?;
		}

		let seal_fields = header.decode_seal::<Vec<_>>()?;
		if seal_fields.len() != 2 {
			Err(BlockError::InvalidSealArity(Mismatch {
				expected: 2,
				found: seal_fields.len(),
			}))?
		}

		let mixhash = H256::from_slice(seal_fields[0]);
		let nonce = H64::from_slice(seal_fields[1]);

		// Nonce must be 0x00..0 or 0xff..f
		if nonce != NONCE_DROP_VOTE && nonce != NONCE_AUTH_VOTE {
			Err(EngineError::CliqueInvalidNonce(nonce))?;
		}

		if is_checkpoint && nonce != NULL_NONCE {
			Err(EngineError::CliqueInvalidNonce(nonce))?;
		}

		// Ensure that the mix digest is zero as Clique don't have fork protection currently
		if mixhash != NULL_MIXHASH {
			Err(BlockError::MismatchedH256SealElement(Mismatch {
				expected: NULL_MIXHASH,
				found: mixhash,
			}))?
		}

		let extra_data_len = header.extra_data().len();

		if extra_data_len < VANITY_LENGTH {
			Err(EngineError::CliqueMissingVanity)?
		}

		if extra_data_len < VANITY_LENGTH + SIGNATURE_LENGTH {
			Err(EngineError::CliqueMissingSignature)?
		}

		let signers = extra_data_len - (VANITY_LENGTH + SIGNATURE_LENGTH);

		// Checkpoint blocks must at least contain one signer
		if is_checkpoint && signers == 0 {
			Err(EngineError::CliqueCheckpointNoSigner)?
		}

		// Addresses must be be divisable by 20
		if is_checkpoint && signers % ADDRESS_LENGTH != 0 {
			Err(EngineError::CliqueCheckpointInvalidSigners(signers))?
		}

		// Ensure that the block doesn't contain any uncles which are meaningless in PoA
		if *header.uncles_hash() != NULL_UNCLES_HASH {
			Err(BlockError::InvalidUnclesHash(Mismatch {
				expected: NULL_UNCLES_HASH,
				found: *header.uncles_hash(),
			}))?
		}

		// Ensure that the block's difficulty is meaningful (may not be correct at this point)
		if *header.difficulty() != DIFF_INTURN && *header.difficulty() != DIFF_NOTURN {
			Err(BlockError::DifficultyOutOfBounds(OutOfBounds {
				min: Some(DIFF_NOTURN),
				max: Some(DIFF_INTURN),
				found: *header.difficulty(),
			}))?
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
			Err(BlockError::UnknownParent(parent.hash()))?
		}

		// Ensure that the block's timestamp isn't too close to it's parent
		let limit = parent.timestamp().saturating_add(self.period);
		if limit > header.timestamp() {
			let max = CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(header.timestamp()));
			let found = CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(limit))
				.ok_or(BlockError::TimestampOverflow)?;

			Err(BlockError::InvalidTimestamp(OutOfBounds {
				min: None,
				max,
				found,
			}.into()))?
		}

		// Retrieve the parent state
		let parent_state = self.state(&parent)?;
		// Try to apply current state, apply() will further check signer and recent signer.
		let mut new_state = parent_state.clone();
		new_state.apply(header, header.number() % self.epoch_length == 0)?;
		new_state.calc_next_timestamp(header.timestamp(), self.period)?;
		self.block_state_by_hash.write().insert(header.hash(), new_state);

		Ok(())
	}

	fn genesis_epoch_data(&self, header: &Header, _call: &Call) -> Result<Vec<u8>, String> {
		let mut state = self.new_checkpoint_state(header).expect("Unable to parse genesis data.");
		state.calc_next_timestamp(header.timestamp(), self.period).map_err(|e| format!("{}", e))?;
		self.block_state_by_hash.write().insert(header.hash(), state);

		// no proof.
		Ok(Vec::new())
	}

	// Our task here is to set difficulty
	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
		// TODO(https://github.com/paritytech/parity-ethereum/issues/10410): this is a horrible hack,
		// it is due to the fact that enact and miner both use OpenBlock::new() which will both call
		// this function. more refactoring is definitely needed.
		if header.extra_data().len() < VANITY_LENGTH + SIGNATURE_LENGTH {
			trace!(target: "engine", "populate_from_parent in sealing");

			// It's unclear how to prevent creating new blocks unless we are authorized, the best way (and geth does this too)
			// it's just to ignore setting a correct difficulty here, we will check authorization in next step in generate_seal anyway.
			if let Some(signer) = self.signer.read().as_ref() {
				let state = match self.state(&parent) {
					Err(e) =>  {
						trace!(target: "engine", "populate_from_parent: Unable to find parent state: {}, ignored.", e);
						return;
					}
					Ok(state) => state,
				};

				if state.is_authorized(&signer.address()) {
					if state.is_inturn(header.number(), &signer.address()) {
						header.set_difficulty(DIFF_INTURN);
					} else {
						header.set_difficulty(DIFF_NOTURN);
					}
				}

				let zero_padding_len = VANITY_LENGTH.saturating_sub(header.extra_data().len());
				if zero_padding_len > 0 {
					let mut resized_extra_data = header.extra_data().clone();
					resized_extra_data.resize(VANITY_LENGTH, 0);
					header.set_extra_data(resized_extra_data);
				}
			} else {
				trace!(target: "engine", "populate_from_parent: no signer registered");
			}
		}
	}

	fn set_signer(&self, signer: Box<dyn EngineSigner>) {
		trace!(target: "engine", "set_signer: {}", signer.address());
		*self.signer.write() = Some(signer);
	}

	fn register_client(&self, client: Weak<dyn EngineClient>) {
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

	// Clique uses the author field for voting, the real author is hidden in the `extra_data` field.
	// So when executing tx's (like in `enact()`) we want to use the executive author
	fn executive_author(&self, header: &Header) -> Result<Address, Error> {
		recover_creator(header)
	}
}
