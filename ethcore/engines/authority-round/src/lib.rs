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

//! A blockchain engine that supports a non-instant BFT proof-of-authority.
//!
//! It is recommended to use the `two_thirds_majority_transition` option, to defend against the
//! ["Attack of the Clones"](https://arxiv.org/pdf/1902.10244.pdf). Newly started networks can
//! set this option to `0`, to use a 2/3 quorum from the beginning.
//!
//! To support on-chain governance, the [ValidatorSet] is pluggable: Aura supports simple
//! constant lists of validators as well as smart contract-based dynamic validator sets.
//! Misbehavior is reported to the [ValidatorSet] as well, so that e.g. governance contracts
//! can penalize or ban attacker's nodes.
//!
//! * "Benign" misbehavior are faults that can happen in normal operation, like failing
//!   to propose a block in your slot, which could be due to a temporary network outage, or
//!   wrong timestamps (due to out-of-sync clocks).
//! * "Malicious" reports are made only if the sender misbehaved deliberately (or due to a
//!   software bug), e.g. if they proposed multiple blocks with the same step number.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::{cmp, fmt};
use std::iter::{self, FromIterator};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Weak, Arc};
use std::time::{UNIX_EPOCH, Duration};
use std::u64;

use client_traits::{EngineClient, ForceUpdateSealing, TransactionRequest};
use engine::{Engine, ConstructedVerifier};
use block_gas_limit::block_gas_limit;
use block_reward::{self, BlockRewardContract, RewardKind};
use ethjson;
use machine::{
	ExecutedBlock,
	Machine,
};
use macros::map;
use keccak_hash::keccak;
use log::{info, debug, error, trace, warn};
use lru_cache::LruCache;
use engine::signer::EngineSigner;
use parity_crypto::publickey::Signature;
use io::{IoContext, IoHandler, TimerToken, IoService};
use itertools::{self, Itertools};
use rand::rngs::OsRng;
use rlp::{encode, Decodable, DecoderError, Encodable, RlpStream, Rlp};
use ethereum_types::{H256, H520, Address, U128, U256};
use parking_lot::{Mutex, RwLock};
use time_utils::CheckedSystemTime;
use common_types::{
	ancestry_action::AncestryAction,
	BlockNumber,
	header::{Header, ExtendedHeader},
	engines::{
		Headers,
		params::CommonParams,
		PendingTransitionStore,
		Seal,
		SealingState,
		machine::{Call, AuxiliaryData},
	},
	errors::{BlockError, EthcoreError as Error, EngineError},
	ids::BlockId,
	snapshot::Snapshotting,
	transaction::SignedTransaction,
};
use unexpected::{Mismatch, OutOfBounds};
use validator_set::{ValidatorSet, SimpleList, new_validator_set};

mod finality;
mod randomness;
pub(crate) mod util;

use self::finality::RollingFinality;

/// `AuthorityRound` params.
pub struct AuthorityRoundParams {
	/// A map defining intervals of blocks with the given times (in seconds) to wait before next
	/// block or authority switching. The keys in the map are steps of starting blocks of those
	/// periods. The entry at `0` should be defined.
	///
	/// Wait times (durations) are additionally required to be less than 65535 since larger values
	/// lead to slow block issuance.
	pub step_durations: BTreeMap<u64, u64>,
	/// Starting step,
	pub start_step: Option<u64>,
	/// Valid validators.
	pub validators: Box<dyn ValidatorSet>,
	/// Chain score validation transition block.
	pub validate_score_transition: u64,
	/// Monotonic step validation transition block.
	pub validate_step_transition: u64,
	/// Immediate transitions.
	pub immediate_transitions: bool,
	/// Block reward in base units.
	pub block_reward: U256,
	/// Block reward contract addresses with their associated starting block numbers.
	pub block_reward_contract_transitions: BTreeMap<u64, BlockRewardContract>,
	/// Number of accepted uncles transition block.
	pub maximum_uncle_count_transition: u64,
	/// Number of accepted uncles.
	pub maximum_uncle_count: usize,
	/// Empty step messages transition block.
	pub empty_steps_transition: u64,
	/// First block for which a 2/3 quorum (instead of 1/2) is required.
	pub two_thirds_majority_transition: BlockNumber,
	/// Number of accepted empty steps.
	pub maximum_empty_steps: usize,
	/// Transition block to strict empty steps validation.
	pub strict_empty_steps_transition: u64,
	/// If set, enables random number contract integration. It maps the transition block to the contract address.
	pub randomness_contract_address: BTreeMap<u64, Address>,
	/// The addresses of contracts that determine the block gas limit with their associated block
	/// numbers.
	pub block_gas_limit_contract_transitions: BTreeMap<u64, Address>,
}

const U16_MAX: usize = ::std::u16::MAX as usize;

/// The number of recent block hashes for which the gas limit override is memoized.
const GAS_LIMIT_OVERRIDE_CACHE_CAPACITY: usize = 10;

impl From<ethjson::spec::AuthorityRoundParams> for AuthorityRoundParams {
	fn from(p: ethjson::spec::AuthorityRoundParams) -> Self {
		let map_step_duration = |u: ethjson::uint::Uint| {
			let mut step_duration_usize: usize = u.into();
			if step_duration_usize == 0 {
				panic!("AuthorityRoundParams: step duration cannot be 0");
			}
			if step_duration_usize > U16_MAX {
				warn!(target: "engine", "step duration is too high ({}), setting it to {}", step_duration_usize, U16_MAX);
				step_duration_usize = U16_MAX;
			}
			step_duration_usize as u64
		};
		let step_durations: BTreeMap<_, _> = match p.step_duration {
			ethjson::spec::StepDuration::Single(u) =>
				iter::once((0,  map_step_duration(u))).collect(),
			ethjson::spec::StepDuration::Transitions(tr) => {
				if tr.is_empty() {
					panic!("AuthorityRoundParams: step duration transitions cannot be empty");
				}
				tr.into_iter().map(|(timestamp, u)| (timestamp.into(), map_step_duration(u))).collect()
			}
		};
		let transition_block_num = p.block_reward_contract_transition.map_or(0, Into::into);
		let mut br_transitions: BTreeMap<_, _> = p.block_reward_contract_transitions
			.unwrap_or_default()
			.into_iter()
			.map(|(block_num, address)|
				 (block_num.into(), BlockRewardContract::new_from_address(address.into())))
			.collect();
		if (p.block_reward_contract_code.is_some() || p.block_reward_contract_address.is_some()) &&
			br_transitions.keys().next().map_or(false, |&block_num| block_num <= transition_block_num)
		{
			let s = "blockRewardContractTransition";
			panic!("{} should be less than any of the keys in {}s", s, s);
		}
		if let Some(code) = p.block_reward_contract_code {
			br_transitions.insert(
				transition_block_num,
				BlockRewardContract::new_from_code(Arc::new(code.into()))
			);
		} else if let Some(address) = p.block_reward_contract_address {
			br_transitions.insert(
				transition_block_num,
				BlockRewardContract::new_from_address(address.into())
			);
		}
		let randomness_contract_address = p.randomness_contract_address.map_or_else(BTreeMap::new, |transitions| {
			transitions.into_iter().map(|(ethjson::uint::Uint(block), addr)| {
				(block.as_u64(), addr.into())
			}).collect()
		});
		let block_gas_limit_contract_transitions: BTreeMap<_, _> =
			p.block_gas_limit_contract_transitions
			.unwrap_or_default()
			.into_iter()
			.map(|(block_num, address)| (block_num.into(), address.into()))
			.collect();
		AuthorityRoundParams {
			step_durations,
			validators: new_validator_set(p.validators),
			start_step: p.start_step.map(Into::into),
			validate_score_transition: p.validate_score_transition.map_or(0, Into::into),
			validate_step_transition: p.validate_step_transition.map_or(0, Into::into),
			immediate_transitions: p.immediate_transitions.unwrap_or(false),
			block_reward: p.block_reward.map_or_else(Default::default, Into::into),
			block_reward_contract_transitions: br_transitions,
			maximum_uncle_count_transition: p.maximum_uncle_count_transition.map_or(0, Into::into),
			maximum_uncle_count: p.maximum_uncle_count.map_or(0, Into::into),
			empty_steps_transition: p.empty_steps_transition.map_or(u64::max_value(), |n| ::std::cmp::max(n.into(), 1)),
			maximum_empty_steps: p.maximum_empty_steps.map_or(0, Into::into),
			two_thirds_majority_transition: p.two_thirds_majority_transition.map_or_else(BlockNumber::max_value, Into::into),
			strict_empty_steps_transition: p.strict_empty_steps_transition.map_or(0, Into::into),
			randomness_contract_address,
			block_gas_limit_contract_transitions,
		}
	}
}

/// A triple containing the first step number and the starting timestamp of the given step duration.
#[derive(Clone, Copy, Debug)]
struct StepDurationInfo {
	transition_step: u64,
	transition_timestamp: u64,
	step_duration: u64,
}

/// Helper for managing the step.
#[derive(Debug)]
struct Step {
	calibrate: bool, // whether calibration is enabled.
	inner: AtomicU64,
	/// Planned durations of steps.
	durations: Vec<StepDurationInfo>,
}

impl Step {
	fn load(&self) -> u64 { self.inner.load(AtomicOrdering::SeqCst) }

	/// Finds the remaining duration of the current step. Panics if there was a counter under- or
	/// overflow.
	fn duration_remaining(&self) -> Duration {
		self.opt_duration_remaining().unwrap_or_else(|| {
			let ctr = self.load();
			error!(target: "engine", "Step counter under- or overflow: {}, aborting", ctr);
			panic!("step counter under- or overflow: {}", ctr)
		})
	}

	/// Finds the remaining duration of the current step. Returns `None` if there was a counter
	/// under- or overflow.
	fn opt_duration_remaining(&self) -> Option<Duration> {
		let next_step = self.load().checked_add(1)?;
		let StepDurationInfo { transition_step, transition_timestamp, step_duration } =
			self.durations.iter()
			.take_while(|info| info.transition_step < next_step)
			.last()
			.expect("durations cannot be empty")
			.clone();
		let next_time = transition_timestamp
			.checked_add(next_step.checked_sub(transition_step)?.checked_mul(step_duration)?)?;
		Some(Duration::from_secs(next_time.saturating_sub(unix_now().as_secs())))
	}

	/// Increments the step number.
	///
	/// Panics if the new step number is `u64::MAX`.
	fn increment(&self) {
		// fetch_add won't panic on overflow but will rather wrap
		// around, leading to zero as the step counter, which might
		// lead to unexpected situations, so it's better to shut down.
		if self.inner.fetch_add(1, AtomicOrdering::SeqCst) == u64::MAX {
			error!(target: "engine", "Step counter is too high: {}, aborting", u64::MAX);
			panic!("step counter is too high: {}", u64::MAX);
		}
	}

	fn calibrate(&self) {
		if self.calibrate {
			if self.opt_calibrate().is_none() {
				let ctr = self.load();
				error!(target: "engine", "Step counter under- or overflow: {}, aborting", ctr);
				panic!("step counter under- or overflow: {}", ctr)
			}
		}
	}

	/// Calibrates the AuRa step number according to the current time.
	fn opt_calibrate(&self) -> Option<()> {
		let now = unix_now().as_secs();
		let StepDurationInfo { transition_step, transition_timestamp, step_duration } =
			self.durations.iter()
			.take_while(|info| info.transition_timestamp < now)
			.last()
			.expect("durations cannot be empty")
			.clone();
		let new_step = (now.checked_sub(transition_timestamp)? / step_duration)
			.checked_add(transition_step)?;
		self.inner.store(new_step, AtomicOrdering::SeqCst);
		Some(())
	}

	fn check_future(&self, given: u64) -> Result<(), Option<OutOfBounds<u64>>> {
		const REJECTED_STEP_DRIFT: u64 = 4;

		// Verify if the step is correct.
		if given <= self.load() {
			return Ok(());
		}

		// Make absolutely sure that the given step is incorrect.
		self.calibrate();
		let current = self.load();

		// reject blocks too far in the future
		if given > current + REJECTED_STEP_DRIFT {
			Err(None)
		// wait a bit for blocks in near future
		} else if given > current {
			let d = self.durations.iter().take_while(|info| info.transition_step <= current).last()
				.expect("Duration map has at least a 0 entry.")
				.step_duration;
			Err(Some(OutOfBounds {
				min: None,
				max: Some(d * current),
				found: d * given,
			}))
		} else {
			Ok(())
		}
	}
}

// Chain scoring: total weight is sqrt(U256::max_value())*height - step
fn calculate_score(parent_step: u64, current_step: u64, current_empty_steps: usize) -> U256 {
	U256::from(U128::max_value()) + U256::from(parent_step) - U256::from(current_step) + U256::from(current_empty_steps)
}

struct EpochManager {
	epoch_transition_hash: H256,
	epoch_transition_number: BlockNumber,
	finality_checker: RollingFinality,
	force: bool,
}

impl EpochManager {
	fn blank(two_thirds_majority_transition: BlockNumber) -> Self {
		EpochManager {
			epoch_transition_hash: H256::zero(),
			epoch_transition_number: 0,
			finality_checker: RollingFinality::blank(Vec::new(), two_thirds_majority_transition),
			force: true,
		}
	}

	// Zooms to the epoch after the header with the given hash. Returns true if succeeded, false otherwise.
	fn zoom_to_after(
		&mut self,
		client: &dyn EngineClient,
		machine: &Machine,
		validators: &dyn ValidatorSet,
		hash: H256
	) -> bool {
		let last_was_parent = self.finality_checker.subchain_head() == Some(hash);

		// early exit for current target == chain head, but only if the epochs are
		// the same.
		if last_was_parent && !self.force {
			return true;
		}

		self.force = false;
		debug!(target: "engine", "Zooming to epoch after block {}", hash);
		trace!(target: "engine", "Current validator set: {:?}", self.validators());


		// epoch_transition_for can be an expensive call, but in the absence of
		// forks it will only need to be called for the block directly after
		// epoch transition, in which case it will be O(1) and require a single
		// DB lookup.
		let last_transition = match client.epoch_transition_for(hash) {
			Some(t) => t,
			None => {
				// this really should never happen unless the block passed
				// hasn't got a parent in the database.
				warn!(target: "engine", "No genesis transition found. Block hash {} does not have a parent in the DB", hash);
				return false;
			}
		};

		// extract other epoch set if it's not the same as the last.
		if last_transition.block_hash != self.epoch_transition_hash {
			let (signal_number, set_proof, _) = destructure_proofs(&last_transition.proof)
				.expect("proof produced by this engine; therefore it is valid; qed");

			trace!(target: "engine", "extracting epoch validator set for epoch ({}, {}) signalled at #{}",
				last_transition.block_number, last_transition.block_hash, signal_number);

			let first = signal_number == 0;
			let epoch_set = validators.epoch_set(
				first,
				machine,
				signal_number, // use signal number so multi-set first calculation is correct.
				set_proof,
			)
				.ok()
				.map(|(list, _)| {
					trace!(target: "engine", "Updating finality checker with new validator set extracted from epoch ({}, {}): {:?}",
						last_transition.block_number, last_transition.block_hash, &list);

					list.into_inner()
				})
				.expect("proof produced by this engine; therefore it is valid; qed");

			let two_thirds_majority_transition = self.finality_checker.two_thirds_majority_transition();
			self.finality_checker = RollingFinality::blank(epoch_set, two_thirds_majority_transition);
		}

		self.epoch_transition_hash = last_transition.block_hash;
		self.epoch_transition_number = last_transition.block_number;

		true
	}

	// Note new epoch hash. This will force the next block to re-load
	// the epoch set.
	// TODO: optimize and don't require re-loading after epoch change.
	fn note_new_epoch(&mut self) {
		self.force = true;
	}

	/// Get validator set. Zoom to the correct epoch first.
	fn validators(&self) -> &SimpleList {
		self.finality_checker.validators()
	}
}

/// A message broadcast by authorities when it's their turn to seal a block but there are no
/// transactions. Other authorities accumulate these messages and later include them in the seal as
/// proof.
#[derive(Clone, Debug, PartialEq, Eq)]
struct EmptyStep {
	signature: H520,
	step: u64,
	parent_hash: H256,
}

impl PartialOrd for EmptyStep {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for EmptyStep {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		self.step.cmp(&other.step)
			.then_with(|| self.parent_hash.cmp(&other.parent_hash))
			.then_with(|| self.signature.cmp(&other.signature))
	}
}

impl EmptyStep {
	fn from_sealed(sealed_empty_step: SealedEmptyStep, parent_hash: &H256) -> EmptyStep {
		let signature = sealed_empty_step.signature;
		let step = sealed_empty_step.step;
		let parent_hash = parent_hash.clone();
		EmptyStep { signature, step, parent_hash }
	}

	fn verify(&self, validators: &dyn ValidatorSet) -> Result<bool, Error> {
		let message = keccak(empty_step_rlp(self.step, &self.parent_hash));
		let correct_proposer = step_proposer(validators, &self.parent_hash, self.step);

		parity_crypto::publickey::verify_address(&correct_proposer, &self.signature.into(), &message)
			.map_err(|e| e.into())
	}

	fn author(&self) -> Result<Address, Error> {
		let message = keccak(empty_step_rlp(self.step, &self.parent_hash));
		let public = parity_crypto::publickey::recover(&self.signature.into(), &message)?;
		Ok(parity_crypto::publickey::public_to_address(&public))
	}

	fn sealed(&self) -> SealedEmptyStep {
		let signature = self.signature;
		let step = self.step;
		SealedEmptyStep { signature, step }
	}
}

impl fmt::Display for EmptyStep {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({:x}, {}, {:x})", self.signature, self.step, self.parent_hash)
	}
}

impl Encodable for EmptyStep {
	fn rlp_append(&self, s: &mut RlpStream) {
		let empty_step_rlp = empty_step_rlp(self.step, &self.parent_hash);
		s.begin_list(2)
			.append(&self.signature)
			.append_raw(&empty_step_rlp, 1);
	}
}

impl Decodable for EmptyStep {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		let signature = rlp.val_at(0)?;
		let empty_step_rlp = rlp.at(1)?;

		let step = empty_step_rlp.val_at(0)?;
		let parent_hash = empty_step_rlp.val_at(1)?;

		Ok(EmptyStep { signature, step, parent_hash })
	}
}

pub fn empty_step_full_rlp(signature: &H520, empty_step_rlp: &[u8]) -> Vec<u8> {
	let mut s = RlpStream::new_list(2);
	s.append(signature).append_raw(empty_step_rlp, 1);
	s.out()
}

pub fn empty_step_rlp(step: u64, parent_hash: &H256) -> Vec<u8> {
	let mut s = RlpStream::new_list(2);
	s.append(&step).append(parent_hash);
	s.out()
}

/// An empty step message that is included in a seal, the only difference is that it doesn't include
/// the `parent_hash` in order to save space. The included signature is of the original empty step
/// message, which can be reconstructed by using the parent hash of the block in which this sealed
/// empty message is included.
struct SealedEmptyStep {
	signature: H520,
	step: u64,
}

impl Encodable for SealedEmptyStep {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2)
			.append(&self.signature)
			.append(&self.step);
	}
}

impl Decodable for SealedEmptyStep {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		let signature = rlp.val_at(0)?;
		let step = rlp.val_at(1)?;

		Ok(SealedEmptyStep { signature, step })
	}
}

struct PermissionedStep {
	inner: Step,
	can_propose: AtomicBool,
}

/// Engine using `AuthorityRound` proof-of-authority BFT consensus.
pub struct AuthorityRound {
	transition_service: IoService<()>,
	step: Arc<PermissionedStep>,
	client: Arc<RwLock<Option<Weak<dyn EngineClient>>>>,
	signer: RwLock<Option<Box<dyn EngineSigner>>>,
	validators: Box<dyn ValidatorSet>,
	validate_score_transition: u64,
	validate_step_transition: u64,
	empty_steps: Mutex<BTreeSet<EmptyStep>>,
	epoch_manager: Mutex<EpochManager>,
	immediate_transitions: bool,
	block_reward: U256,
	block_reward_contract_transitions: BTreeMap<u64, BlockRewardContract>,
	maximum_uncle_count_transition: u64,
	maximum_uncle_count: usize,
	empty_steps_transition: u64,
	strict_empty_steps_transition: u64,
	two_thirds_majority_transition: BlockNumber,
	maximum_empty_steps: usize,
	machine: Machine,
	/// History of step hashes recently received from peers.
	received_step_hashes: RwLock<BTreeMap<(u64, Address), H256>>,
	/// If set, enables random number contract integration. It maps the transition block to the contract address.
	randomness_contract_address: BTreeMap<u64, Address>,
	/// The addresses of contracts that determine the block gas limit.
	block_gas_limit_contract_transitions: BTreeMap<u64, Address>,
	/// Memoized gas limit overrides, by block hash.
	gas_limit_override_cache: Mutex<LruCache<H256, Option<U256>>>,
}

// header-chain validator.
struct EpochVerifier {
	step: Arc<PermissionedStep>,
	subchain_validators: SimpleList,
	empty_steps_transition: u64,
	/// First block for which a 2/3 quorum (instead of 1/2) is required.
	two_thirds_majority_transition: BlockNumber,
}

impl engine::EpochVerifier for EpochVerifier {
	fn verify_light(&self, header: &Header) -> Result<(), Error> {
		// Validate the timestamp
		verify_timestamp(&self.step.inner, header_step(header, self.empty_steps_transition)?)?;
		// always check the seal since it's fast.
		// nothing heavier to do.
		verify_external(header, &self.subchain_validators, self.empty_steps_transition)
	}

	fn check_finality_proof(&self, proof: &[u8]) -> Option<Vec<H256>> {
		let signers = self.subchain_validators.clone().into_inner();
		let mut finality_checker = RollingFinality::blank(signers, self.two_thirds_majority_transition);
		let mut finalized = Vec::new();

		let headers: Vec<Header> = Rlp::new(proof).as_list().ok()?;

		{
			let mut push_header = |parent_header: &Header, header: Option<&Header>| {
				// ensure all headers have correct number of seal fields so we can `verify_external`
				// and get `empty_steps` without panic.
				if parent_header.seal().len() != header_expected_seal_fields(parent_header, self.empty_steps_transition) {
					return None
				}
				if header.iter().any(|h| h.seal().len() != header_expected_seal_fields(h, self.empty_steps_transition)) {
					return None
				}

				// `verify_external` checks that signature is correct and author == signer.
				verify_external(parent_header, &self.subchain_validators, self.empty_steps_transition).ok()?;

				let mut signers = match header {
					Some(header) => header_empty_steps_signers(header, self.empty_steps_transition).ok()?,
					_ => Vec::new(),
				};
				signers.push(*parent_header.author());

				let newly_finalized =
					finality_checker.push_hash(parent_header.hash(), parent_header.number(), signers).ok()?;
				finalized.extend(newly_finalized);

				Some(())
			};

			for window in headers.windows(2) {
				push_header(&window[0], Some(&window[1]))?;
			}

			if let Some(last) = headers.last() {
				push_header(last, None)?;
			}
		}

		if finalized.is_empty() { None } else { Some(finalized) }
	}
}

fn header_seal_hash(header: &Header, empty_steps_rlp: Option<&[u8]>) -> H256 {
	match empty_steps_rlp {
		Some(empty_steps_rlp) => {
			let mut message = header.bare_hash().as_bytes().to_vec();
			message.extend_from_slice(empty_steps_rlp);
			keccak(message)
		},
		None => {
			header.bare_hash()
		},
	}
}

fn header_expected_seal_fields(header: &Header, empty_steps_transition: u64) -> usize {
	if header.number() >= empty_steps_transition {
		3
	} else {
		2
	}
}

fn header_step(header: &Header, empty_steps_transition: u64) -> Result<u64, ::rlp::DecoderError> {
	Rlp::new(&header.seal().get(0).unwrap_or_else(||
		panic!("was either checked with verify_block_basic or is genesis; has {} fields; qed (Make sure the spec \
				file has a correct genesis seal)", header_expected_seal_fields(header, empty_steps_transition))
	))
	.as_val()
}

fn header_signature(header: &Header, empty_steps_transition: u64) -> Result<Signature, ::rlp::DecoderError> {
	Rlp::new(&header.seal().get(1).unwrap_or_else(||
		panic!("was checked with verify_block_basic; has {} fields; qed",
			header_expected_seal_fields(header, empty_steps_transition))
	))
	.as_val::<H520>().map(Into::into)
}

// extracts the raw empty steps vec from the header seal. should only be called when there are 3 fields in the seal
// (i.e. header.number() >= self.empty_steps_transition)
fn header_empty_steps_raw(header: &Header) -> &[u8] {
	header.seal().get(2).expect("was checked with verify_block_basic; has 3 fields; qed")
}

// extracts the empty steps from the header seal. should only be called when there are 3 fields in the seal
// (i.e. header.number() >= self.empty_steps_transition).
fn header_empty_steps(header: &Header) -> Result<Vec<EmptyStep>, ::rlp::DecoderError> {
	let empty_steps = Rlp::new(header_empty_steps_raw(header)).as_list::<SealedEmptyStep>()?;
	Ok(empty_steps.into_iter().map(|s| EmptyStep::from_sealed(s, header.parent_hash())).collect())
}

// gets the signers of empty step messages for the given header, does not include repeated signers
fn header_empty_steps_signers(header: &Header, empty_steps_transition: u64) -> Result<Vec<Address>, Error> {
	if header.number() >= empty_steps_transition {
		let mut signers = HashSet::new();
		for empty_step in header_empty_steps(header)? {
			signers.insert(empty_step.author()?);
		}

		Ok(Vec::from_iter(signers.into_iter()))
	} else {
		Ok(Vec::new())
	}
}

fn step_proposer(validators: &dyn ValidatorSet, bh: &H256, step: u64) -> Address {
	let proposer = validators.get(bh, step as usize);
	trace!(target: "engine", "step_proposer: Fetched proposer for step {}: {}", step, proposer);
	proposer
}

fn is_step_proposer(validators: &dyn ValidatorSet, bh: &H256, step: u64, address: &Address) -> bool {
	step_proposer(validators, bh, step) == *address
}

fn verify_timestamp(step: &Step, header_step: u64) -> Result<(), BlockError> {
	match step.check_future(header_step) {
		Err(None) => {
			trace!(target: "engine", "verify_timestamp: block from the future");
			Err(BlockError::InvalidSeal.into())
		},
		Err(Some(oob)) => {
			// NOTE This error might be returned only in early stage of verification (Stage 1).
			// Returning it further won't recover the sync process.
			trace!(target: "engine", "verify_timestamp: block too early");

			let found = CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(oob.found))
				.ok_or(BlockError::TimestampOverflow)?;
			let max = oob.max.and_then(|m| CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(m)));
			let min = oob.min.and_then(|m| CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(m)));

			let new_oob = OutOfBounds { min, max, found };

			Err(BlockError::TemporarilyInvalid(new_oob.into()))
		},
		Ok(_) => Ok(()),
	}
}

fn verify_external(header: &Header, validators: &dyn ValidatorSet, empty_steps_transition: u64) -> Result<(), Error> {
	let header_step = header_step(header, empty_steps_transition)?;

	let proposer_signature = header_signature(header, empty_steps_transition)?;
	let correct_proposer = validators.get(header.parent_hash(), header_step as usize);
	let is_invalid_proposer = *header.author() != correct_proposer || {
		let empty_steps_rlp = if header.number() >= empty_steps_transition {
			Some(header_empty_steps_raw(header))
		} else {
			None
		};

		let header_seal_hash = header_seal_hash(header, empty_steps_rlp);
		!parity_crypto::publickey::verify_address(&correct_proposer, &proposer_signature, &header_seal_hash)?
	};

	if is_invalid_proposer {
		warn!(target: "engine", "verify_block_external: bad proposer for step: {}", header_step);
		Err(EngineError::NotProposer(Mismatch { expected: correct_proposer, found: *header.author() }))?
	} else {
		Ok(())
	}
}

fn combine_proofs(signal_number: BlockNumber, set_proof: &[u8], finality_proof: &[u8]) -> Vec<u8> {
	let mut stream = ::rlp::RlpStream::new_list(3);
	stream.append(&signal_number).append(&set_proof).append(&finality_proof);
	stream.out()
}


fn destructure_proofs(combined: &[u8]) -> Result<(BlockNumber, &[u8], &[u8]), Error> {
	let rlp = Rlp::new(combined);
	Ok((
		rlp.at(0)?.as_val()?,
		rlp.at(1)?.data()?,
		rlp.at(2)?.data()?,
	))
}

trait AsMillis {
	fn as_millis(&self) -> u64;
}

impl AsMillis for Duration {
	fn as_millis(&self) -> u64 {
		self.as_secs() * 1_000 + (self.subsec_nanos() / 1_000_000) as u64
	}
}

// A type for storing owned or borrowed data that has a common type.
// Useful for returning either a borrow or owned data from a function.
enum CowLike<'a, A: 'a + ?Sized, B> {
	Borrowed(&'a A),
	Owned(B),
}

impl<'a, A: ?Sized, B> Deref for CowLike<'a, A, B> where B: AsRef<A> {
	type Target = A;
	fn deref(&self) -> &A {
		match self {
			CowLike::Borrowed(b) => b,
			CowLike::Owned(o) => o.as_ref(),
		}
	}
}

impl AuthorityRound {
	/// Create a new instance of AuthorityRound engine.
	pub fn new(our_params: AuthorityRoundParams, machine: Machine) -> Result<Arc<Self>, Error> {
		if !our_params.step_durations.contains_key(&0) {
			error!(target: "engine", "Authority Round step 0 duration is undefined, aborting");
			return Err(Error::Engine(EngineError::Custom(String::from("step 0 duration is undefined"))));
		}
		if our_params.step_durations.values().any(|v| *v == 0) {
			error!(target: "engine", "Authority Round step duration cannot be 0");
			return Err(Error::Engine(EngineError::Custom(String::from("step duration cannot be 0"))));
		}
		let should_timeout = our_params.start_step.is_none();
		let initial_step = our_params.start_step.unwrap_or(0);

		let mut durations = Vec::new();
		let mut prev_step = 0u64;
		let mut prev_time = 0u64;
		let mut prev_dur = our_params.step_durations[&0];
		durations.push(StepDurationInfo {
			transition_step: prev_step,
			transition_timestamp: prev_time,
			step_duration: prev_dur
		});
		for (time, dur) in our_params.step_durations.iter().skip(1) {
			let (step, time) = next_step_time_duration(
				StepDurationInfo{
					transition_step: prev_step,
					transition_timestamp: prev_time,
					step_duration: prev_dur,
				}, *time)
				.ok_or(BlockError::TimestampOverflow)?;
			durations.push(StepDurationInfo {
				transition_step: step,
				transition_timestamp: time,
				step_duration: *dur
			});
			prev_step = step;
			prev_time = time;
			prev_dur = *dur;
		}

		let step = Step {
			inner: AtomicU64::new(initial_step),
			calibrate: our_params.start_step.is_none(),
			durations,
		};
		step.calibrate();
		let engine = Arc::new(
			AuthorityRound {
				transition_service: IoService::<()>::start()?,
				step: Arc::new(PermissionedStep { inner: step, can_propose: AtomicBool::new(true) }),
				client: Arc::new(RwLock::new(None)),
				signer: RwLock::new(None),
				validators: our_params.validators,
				validate_score_transition: our_params.validate_score_transition,
				validate_step_transition: our_params.validate_step_transition,
				empty_steps: Default::default(),
				epoch_manager: Mutex::new(EpochManager::blank(our_params.two_thirds_majority_transition)),
				immediate_transitions: our_params.immediate_transitions,
				block_reward: our_params.block_reward,
				block_reward_contract_transitions: our_params.block_reward_contract_transitions,
				maximum_uncle_count_transition: our_params.maximum_uncle_count_transition,
				maximum_uncle_count: our_params.maximum_uncle_count,
				empty_steps_transition: our_params.empty_steps_transition,
				maximum_empty_steps: our_params.maximum_empty_steps,
				two_thirds_majority_transition: our_params.two_thirds_majority_transition,
				strict_empty_steps_transition: our_params.strict_empty_steps_transition,
				machine,
				received_step_hashes: RwLock::new(Default::default()),
				randomness_contract_address: our_params.randomness_contract_address,
				block_gas_limit_contract_transitions: our_params.block_gas_limit_contract_transitions,
				gas_limit_override_cache: Mutex::new(LruCache::new(GAS_LIMIT_OVERRIDE_CACHE_CAPACITY)),
			});

		// Do not initialize timeouts for tests.
		if should_timeout {
			let handler = TransitionHandler {
				step: engine.step.clone(),
				client: engine.client.clone(),
			};
			engine.transition_service.register_handler(Arc::new(handler))?;
		}
		Ok(engine)
	}

	// fetch correct validator set for epoch at header, taking into account
	// finality of previous transitions.
	fn epoch_set<'a>(&'a self, header: &Header) -> Result<(CowLike<dyn ValidatorSet, SimpleList>, BlockNumber), Error> {
		Ok(if self.immediate_transitions {
			(CowLike::Borrowed(&*self.validators), header.number())
		} else {
			let mut epoch_manager = self.epoch_manager.lock();
			let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
				Some(client) => client,
				None => {
					debug!(target: "engine", "Unable to verify sig: missing client ref.");
					return Err(EngineError::RequiresClient.into())
				}
			};

			if !epoch_manager.zoom_to_after(&*client, &self.machine, &*self.validators, *header.parent_hash()) {
				debug!(target: "engine", "Unable to zoom to epoch.");
				return Err(EngineError::MissingParent(*header.parent_hash()).into())
			}

			(CowLike::Owned(epoch_manager.validators().clone()), epoch_manager.epoch_transition_number)
		})
	}

	fn empty_steps(&self, from_step: u64, to_step: u64, parent_hash: H256) -> Vec<EmptyStep> {
		let from = EmptyStep {
			step: from_step + 1,
			parent_hash,
			signature: Default::default(),
		};
		let to = EmptyStep {
			step: to_step,
			parent_hash: Default::default(),
			signature: Default::default(),
		};

		if from >= to {
			return vec![];
		}

		self.empty_steps.lock()
			.range(from..to)
			.filter(|e| e.parent_hash == parent_hash)
			.cloned()
			.collect()
	}

	fn clear_empty_steps(&self, step: u64) {
		// clear old `empty_steps` messages
		let mut empty_steps = self.empty_steps.lock();
		*empty_steps = empty_steps.split_off(&EmptyStep {
			step: step + 1,
			parent_hash: Default::default(),
			signature: Default::default(),
		});
	}

	fn handle_empty_step_message(&self, empty_step: EmptyStep) {
		self.empty_steps.lock().insert(empty_step);
	}

	fn generate_empty_step(&self, parent_hash: &H256) {
		let step = self.step.inner.load();
		let empty_step_rlp = empty_step_rlp(step, parent_hash);

		if let Ok(signature) = self.sign(keccak(&empty_step_rlp)).map(Into::into) {
			let message_rlp = empty_step_full_rlp(&signature, &empty_step_rlp);

			let parent_hash = *parent_hash;
			let empty_step = EmptyStep { signature, step, parent_hash };

			trace!(target: "engine", "broadcasting empty step message: {:?}", empty_step);
			self.broadcast_message(message_rlp);
			self.handle_empty_step_message(empty_step);

		} else {
			warn!(target: "engine", "generate_empty_step: FAIL: accounts secret key unavailable");
		}
	}

	fn broadcast_message(&self, message: Vec<u8>) {
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				c.broadcast_consensus_message(message);
			}
		}
	}

	fn report_skipped(&self, header: &Header, current_step: u64, parent_step: u64, validators: &dyn ValidatorSet, set_number: u64) {
		// we're building on top of the genesis block so don't report any skipped steps
		if header.number() == 1 {
			return;
		}

		if let (true, Some(me)) = (current_step > parent_step + 1, self.address()) {
			debug!(target: "engine", "Author {} built block with step gap. current step: {}, parent step: {}",
				header.author(), current_step, parent_step);
			let mut reported = HashSet::new();
			for step in parent_step + 1..current_step {
				let skipped_primary = step_proposer(validators, header.parent_hash(), step);
				// Do not report this signer.
				if skipped_primary != me {
					// Stop reporting once validators start repeating.
					if !reported.insert(skipped_primary) { break; }
					trace!(target: "engine", "Reporting benign misbehaviour (cause: skipped step) at block #{}, epoch set number {}, step proposer={:#x}. Own address: {}",
						header.number(), set_number, skipped_primary, me);
					self.validators.report_benign(&skipped_primary, set_number, header.number());
				} else {
					trace!(target: "engine", "Primary that skipped is self, not self-reporting. Own address: {}", me);
				}
			}
		}
	}

	// Returns the hashes of all ancestor blocks that are finalized by the given `chain_head`.
	fn build_finality(&self, chain_head: &Header, ancestry: &mut dyn Iterator<Item=Header>) -> Vec<H256> {
		if self.immediate_transitions { return Vec::new() }

		let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
			Some(client) => client,
			None => {
				warn!(target: "engine", "Unable to apply ancestry actions: missing client ref.");
				return Vec::new();
			}
		};

		let mut epoch_manager = self.epoch_manager.lock();
		if !epoch_manager.zoom_to_after(&*client, &self.machine, &*self.validators, *chain_head.parent_hash()) {
			return Vec::new();
		}

		if epoch_manager.finality_checker.subchain_head() != Some(*chain_head.parent_hash()) {
			// build new finality checker from unfinalized ancestry of chain head, not including chain head itself yet.
			trace!(target: "finality", "Building finality up to parent of {} ({})",
				chain_head.hash(), chain_head.parent_hash());

			// the empty steps messages in a header signal approval of the
			// parent header.
			let mut parent_empty_steps_signers = match header_empty_steps_signers(&chain_head, self.empty_steps_transition) {
				Ok(empty_step_signers) => empty_step_signers,
				Err(_) => {
					warn!(target: "finality", "Failed to get empty step signatures from block {}", chain_head.hash());
					return Vec::new();
				}
			};

			let epoch_transition_hash = epoch_manager.epoch_transition_hash;
			let ancestry_iter = ancestry.map(|header| {
				let mut signers = vec![*header.author()];
				signers.extend(parent_empty_steps_signers.drain(..));

				if let Ok(empty_step_signers) = header_empty_steps_signers(&header, self.empty_steps_transition) {
					let res = (header.hash(), header.number(), signers);
					trace!(target: "finality", "Ancestry iteration: yielding {:?}", res);

					parent_empty_steps_signers = empty_step_signers;

					Some(res)

				} else {
					warn!(target: "finality", "Failed to get empty step signatures from block {}", header.hash());
					None
				}
			})
				.while_some()
				.take_while(|&(h, _, _)| h != epoch_transition_hash);

			if let Err(e) = epoch_manager.finality_checker.build_ancestry_subchain(ancestry_iter) {
				debug!(target: "engine", "inconsistent validator set within epoch: {:?}", e);
				return Vec::new();
			}
		}

		let finalized = epoch_manager.finality_checker.push_hash(
			chain_head.hash(), chain_head.number(), vec![*chain_head.author()]);
		finalized.unwrap_or_default()
	}

	fn address(&self) -> Option<Address> {
		self.signer.read().as_ref().map(|s| s.address() )
	}

	/// Make calls to the randomness contract.
	fn run_randomness_phase(&self, block: &ExecutedBlock) -> Result<Vec<SignedTransaction>, Error> {
		let contract_addr = match self.randomness_contract_address.range(..=block.header.number()).last() {
			Some((_, &contract_addr)) => contract_addr,
			None => return Ok(Vec::new()), // No randomness contract.
		};

		let opt_signer = self.signer.read();
		let signer = match opt_signer.as_ref() {
			Some(signer) => signer,
			None => return Ok(Vec::new()), // We are not a validator, so we shouldn't call the contracts.
		};
		let our_addr = signer.address();
		let client = self.client.read().as_ref().and_then(|weak| weak.upgrade()).ok_or_else(|| {
			debug!(target: "engine", "Unable to prepare block: missing client ref.");
			EngineError::RequiresClient
		})?;
		let full_client = client.as_full_client()
			.ok_or_else(|| EngineError::FailedSystemCall("Failed to upgrade to BlockchainClient.".to_string()))?;

		// Random number generation
		let contract = util::BoundContract::new(&*client, BlockId::Latest, contract_addr);
		let phase = randomness::RandomnessPhase::load(&contract, our_addr)
			.map_err(|err| EngineError::Custom(format!("Randomness error in load(): {:?}", err)))?;
		let data = match phase.advance(&contract, &mut OsRng, signer.as_ref())
				.map_err(|err| EngineError::Custom(format!("Randomness error in advance(): {:?}", err)))? {
			Some(data) => data,
			None => return Ok(Vec::new()), // Nothing to commit or reveal at the moment.
		};

		let nonce = block.state.nonce(&our_addr)?;
		let tx_request = TransactionRequest::call(contract_addr, data).gas_price(U256::zero()).nonce(nonce);
		Ok(vec![full_client.create_transaction(tx_request)?])
	}
}

fn unix_now() -> Duration {
	UNIX_EPOCH.elapsed().expect("Valid time has to be set in your system.")
}

struct TransitionHandler {
	step: Arc<PermissionedStep>,
	client: Arc<RwLock<Option<Weak<dyn EngineClient>>>>,
}

const ENGINE_TIMEOUT_TOKEN: TimerToken = 23;

impl IoHandler<()> for TransitionHandler {
	fn initialize(&self, io: &IoContext<()>) {
		let remaining = AsMillis::as_millis(&self.step.inner.duration_remaining());
		io.register_timer_once(ENGINE_TIMEOUT_TOKEN, Duration::from_millis(remaining))
			.unwrap_or_else(|e| warn!(target: "engine", "Failed to start consensus step timer: {}.", e))
	}

	fn timeout(&self, io: &IoContext<()>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			// NOTE we might be lagging by couple of steps in case the timeout
			// has not been called fast enough.
			// Make sure to advance up to the actual step.
			while AsMillis::as_millis(&self.step.inner.duration_remaining()) == 0 {
				self.step.inner.increment();
				self.step.can_propose.store(true, AtomicOrdering::SeqCst);
				if let Some(ref weak) = *self.client.read() {
					if let Some(c) = weak.upgrade() {
						c.update_sealing(ForceUpdateSealing::No);
					}
				}
			}

			let next_run_at = Duration::from_millis(
				AsMillis::as_millis(&self.step.inner.duration_remaining()) >> 2
			);
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, next_run_at)
				.unwrap_or_else(|e| warn!(target: "engine", "Failed to restart consensus step timer: {}.", e))
		}
	}
}

impl Engine for AuthorityRound {
	fn name(&self) -> &str { "AuthorityRound" }

	fn machine(&self) -> &Machine { &self.machine }

	/// Three fields - consensus step and the corresponding proposer signature, and a list of empty
	/// step messages (which should be empty if no steps are skipped)
	fn seal_fields(&self, header: &Header) -> usize {
		header_expected_seal_fields(header, self.empty_steps_transition)
	}

	fn step(&self) {
		self.step.inner.increment();
		self.step.can_propose.store(true, AtomicOrdering::SeqCst);
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				c.update_sealing(ForceUpdateSealing::No);
			}
		}
	}

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, header: &Header) -> BTreeMap<String, String> {
		if header.seal().len() < header_expected_seal_fields(header, self.empty_steps_transition) {
			return BTreeMap::default();
		}

		let step = header_step(header, self.empty_steps_transition).as_ref()
			.map(ToString::to_string)
			.unwrap_or_default();
		let signature = header_signature(header, self.empty_steps_transition).as_ref()
			.map(ToString::to_string)
			.unwrap_or_default();

		let mut info = map![
			"step".into() => step,
			"signature".into() => signature
		];

		if header.number() >= self.empty_steps_transition {
			let empty_steps =
				if let Ok(empty_steps) = header_empty_steps(header).as_ref() {
					format!("[{}]",
						empty_steps.iter().fold(
							"".to_string(),
							|acc, e| if acc.len() > 0 { acc + ","} else { acc } + &e.to_string()))

				} else {
					"".into()
				};

			info.insert("emptySteps".into(), empty_steps);
		}

		info
	}

	fn maximum_uncle_count(&self, block: BlockNumber) -> usize {
		if block >= self.maximum_uncle_count_transition {
			self.maximum_uncle_count
		} else {
			// fallback to default value
			2
		}
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
		let parent_step = header_step(parent, self.empty_steps_transition).expect("Header has been verified; qed");
		let current_step = self.step.inner.load();

		let current_empty_steps_len = if header.number() >= self.empty_steps_transition {
			self.empty_steps(parent_step, current_step, parent.hash()).len()
		} else {
			0
		};

		let score = calculate_score(parent_step, current_step, current_empty_steps_len);
		header.set_difficulty(score);
		if let Some(gas_limit) = self.gas_limit_override(header) {
			trace!(target: "engine", "Setting gas limit to {} for block {}.", gas_limit, header.number());
			let parent_gas_limit = *parent.gas_limit();
			header.set_gas_limit(gas_limit);
			if parent_gas_limit != gas_limit {
				info!(target: "engine", "Block gas limit was changed from {} to {}.", parent_gas_limit, gas_limit);
			}
		}
	}

	fn sealing_state(&self) -> SealingState {
		let our_addr = match *self.signer.read() {
			Some(ref signer) => signer.address(),
			None => {
				warn!(target: "engine", "Not preparing block; cannot sign.");
				return SealingState::NotReady;
			}
		};

		let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
			Some(client) => client,
			None => {
				warn!(target: "engine", "Not preparing block: missing client ref.");
				return SealingState::NotReady;
			}
		};

		let parent = match client.as_full_client() {
			Some(full_client) => full_client.best_block_header(),
			None => {
				debug!(target: "engine", "Not preparing block: not a full client.");
				return SealingState::NotReady;
			},
		};

		let validators = if self.immediate_transitions {
			CowLike::Borrowed(&*self.validators)
		} else {
			let mut epoch_manager = self.epoch_manager.lock();
			if !epoch_manager.zoom_to_after(&*client, &self.machine, &*self.validators, parent.hash()) {
				debug!(target: "engine", "Not preparing block: Unable to zoom to epoch.");
				return SealingState::NotReady;
			}
			CowLike::Owned(epoch_manager.validators().clone())
		};

		let step = self.step.inner.load();

		if !is_step_proposer(&*validators, &parent.hash(), step, &our_addr) {
			trace!(target: "engine", "Not preparing block: not a proposer for step {}. (Our address: {})",
				step, our_addr);
			return SealingState::NotReady;
		}

		SealingState::Ready
	}

	fn handle_message(&self, rlp: &[u8]) -> Result<(), EngineError> {
		fn fmt_err<T: ::std::fmt::Debug>(x: T) -> EngineError {
			EngineError::MalformedMessage(format!("{:?}", x))
		}

		let rlp = Rlp::new(rlp);
		let empty_step: EmptyStep = rlp.as_val().map_err(fmt_err)?;

		if empty_step.verify(&*self.validators).unwrap_or(false) {
			if self.step.inner.check_future(empty_step.step).is_ok() {
				trace!(target: "engine", "handle_message: received empty step message {:?}", empty_step);
				self.handle_empty_step_message(empty_step);
			} else {
				trace!(target: "engine", "handle_message: empty step message from the future {:?}", empty_step);
			}
		} else {
			trace!(target: "engine", "handle_message: received invalid step message {:?}", empty_step);
		};

		Ok(())
	}

	/// Attempt to seal the block internally.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which case
	/// `Seal::None` will be returned.
	fn generate_seal(&self, block: &ExecutedBlock, parent: &Header) -> Seal {
		// first check to avoid generating signature most of the time
		// (but there's still a race to the `compare_and_swap`)
		if !self.step.can_propose.load(AtomicOrdering::SeqCst) {
			trace!(target: "engine", "Aborting seal generation. Can't propose.");
			return Seal::None;
		}

		let header = &block.header;
		let parent_step = header_step(parent, self.empty_steps_transition)
			.expect("Header has been verified; qed");

		let step = self.step.inner.load();

		// filter messages from old and future steps and different parents
		let empty_steps = if header.number() >= self.empty_steps_transition {
			self.empty_steps(parent_step, step, *header.parent_hash())
		} else {
			Vec::new()
		};

		let expected_diff = calculate_score(parent_step, step, empty_steps.len());

		if header.difficulty() != &expected_diff {
			debug!(target: "engine", "Aborting seal generation. The step or empty_steps have changed in the meantime. {:?} != {:?}",
				header.difficulty(), expected_diff);
			return Seal::None;
		}

		if parent_step > step {
			warn!(target: "engine", "Aborting seal generation for invalid step: {} > {}", parent_step, step);
			return Seal::None;
		} else if parent_step == step {
			// this is guarded against by `can_propose` unless the block was signed
			// on the same step (implies same key) and on a different node.
			warn!("Attempted to seal block on the same step as parent. Is this authority sealing with more than one node?");
			return Seal::None;
		}

		let (validators, epoch_transition_number) = match self.epoch_set(header) {
			Err(err) => {
				warn!(target: "engine", "Unable to generate seal: {}", err);
				return Seal::None;
			},
			Ok(ok) => ok,
		};

		if is_step_proposer(&*validators, header.parent_hash(), step, header.author()) {
			trace!(target: "engine", "generate_seal: we are step proposer for step={}, block=#{}", step, header.number());
			// if there are no transactions to include in the block, we don't seal and instead broadcast a signed
			// `EmptyStep(step, parent_hash)` message. If we exceed the maximum amount of `empty_step` rounds we proceed
			// with the seal.
			if header.number() >= self.empty_steps_transition &&
				block.transactions.is_empty() &&
				empty_steps.len() < self.maximum_empty_steps {

				if self.step.can_propose.compare_and_swap(true, false, AtomicOrdering::SeqCst) {
					trace!(target: "engine", "generate_seal: generating empty step at step={}, block=#{}", step, header.number());
					self.generate_empty_step(header.parent_hash());
				}

				return Seal::None;
			}

			let empty_steps_rlp = if header.number() >= self.empty_steps_transition {
				let empty_steps: Vec<_> = empty_steps.iter().map(|e| e.sealed()).collect();
				Some(::rlp::encode_list(&empty_steps))
			} else {
				None
			};

			if let Ok(signature) = self.sign(header_seal_hash(header, empty_steps_rlp.as_ref().map(|e| &**e))) {
				trace!(target: "engine", "generate_seal: Issuing a block for step {}.", step);

				// only issue the seal if we were the first to reach the compare_and_swap.
				if self.step.can_propose.compare_and_swap(true, false, AtomicOrdering::SeqCst) {
					// we can drop all accumulated empty step messages that are
					// older than the parent step since we're including them in
					// the seal
					self.clear_empty_steps(parent_step);

					// report any skipped primaries between the parent block and
					// the block we're sealing, unless we have empty steps enabled
					if header.number() < self.empty_steps_transition {
						self.report_skipped(header, step, parent_step, &*validators, epoch_transition_number);
					}

					let mut fields = vec![
						encode(&step),
						encode(&(H520::from(signature).as_bytes())),
					];

					if let Some(empty_steps_rlp) = empty_steps_rlp {
						fields.push(empty_steps_rlp);
					}
					trace!(target: "engine", "generate_seal: returning Seal::Regular for step={}, block=#{}", step, header.number());
					return Seal::Regular(fields);
				}
			} else {
				warn!(target: "engine", "generate_seal: FAIL: Accounts secret key unavailable.");
			}
		} else {
			trace!(target: "engine", "generate_seal: {} not a proposer for step {}.",
				header.author(), step);
		}
		trace!(target: "engine", "generate_seal: returning Seal::None for step={}, block=#{}", step, header.number());
		Seal::None
	}

	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
		Ok(())
	}

	fn on_new_block(
		&self,
		block: &mut ExecutedBlock,
		epoch_begin: bool,
	) -> Result<(), Error> {
		// with immediate transitions, we don't use the epoch mechanism anyway.
		// the genesis is always considered an epoch, but we ignore it intentionally.
		if self.immediate_transitions || !epoch_begin { return Ok(()) }

		// genesis is never a new block, but might as well check.
		let header = block.header.clone();
		let first = header.number() == 0;

		let mut call = |to, data| {
			let result = self.machine.execute_as_system(
				block,
				to,
				U256::max_value(), // unbounded gas? maybe make configurable.
				Some(data),
			);

			result.map_err(|e| format!("{}", e))
		};

		self.validators.on_epoch_begin(first, &header, &mut call)
	}

	/// Apply the block reward on finalisation of the block.
	fn on_close_block(
		&self,
		block: &mut ExecutedBlock,
		parent: &Header,
	) -> Result<(), Error> {
		let mut beneficiaries = Vec::new();

		if block.header.number() == self.two_thirds_majority_transition {
			info!(target: "engine", "Block {}: Transitioning to 2/3 quorum.", self.two_thirds_majority_transition);
		}

		if block.header.number() >= self.empty_steps_transition {
			let empty_steps = if block.header.seal().is_empty() {
				// this is a new block, calculate rewards based on the empty steps messages we have accumulated
				let parent_step = header_step(parent, self.empty_steps_transition)?;
				let current_step = self.step.inner.load();
				self.empty_steps(parent_step, current_step, parent.hash())
			} else {
				// we're verifying a block, extract empty steps from the seal
				header_empty_steps(&block.header)?
			};

			for empty_step in empty_steps {
				let author = empty_step.author()?;
				beneficiaries.push((author, RewardKind::EmptyStep));
			}
		}

		let author = *block.header.author();
		beneficiaries.push((author, RewardKind::Author));

		let block_reward_contract_transition = self
			.block_reward_contract_transitions
			.range(..=block.header.number())
			.last();
		let rewards: Vec<_> = if let Some((_, contract)) = block_reward_contract_transition {
			let mut call = engine::default_system_or_code_call(&self.machine, block);
			let rewards = contract.reward(beneficiaries, &mut call)?;
			rewards.into_iter().map(|(author, amount)| (author, RewardKind::External, amount)).collect()
		} else {
			beneficiaries.into_iter().map(|(author, reward_kind)| (author, reward_kind, self.block_reward)).collect()
		};

		block_reward::apply_block_rewards(&rewards, block, &self.machine)
	}

	fn generate_engine_transactions(&self, block: &ExecutedBlock) -> Result<Vec<SignedTransaction>, Error> {
		self.run_randomness_phase(block)
	}

	/// Check the number of seal fields.
	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
		if header.number() >= self.validate_score_transition && *header.difficulty() >= U256::from(U128::max_value()) {
			return Err(From::from(BlockError::DifficultyOutOfBounds(
				OutOfBounds { min: None, max: Some(U256::from(U128::max_value())), found: *header.difficulty() }
			)));
		}

		match verify_timestamp(&self.step.inner, header_step(header, self.empty_steps_transition)?) {
			Err(BlockError::InvalidSeal) => {
				// This check runs in Phase 1 where there is no guarantee that the parent block is
				// already imported, therefore the call to `epoch_set` may fail. In that case we
				// won't report the misbehavior but this is not a concern because:
				// - Authorities will have a signing key available to report and it's expected that
				//   they'll be up-to-date and importing, therefore the parent header will most likely
				//   be available
				// - Even if you are an authority that is syncing the chain, the contract will most
				//   likely ignore old reports
				// - This specific check is only relevant if you're importing (since it checks
				//   against wall clock)
				if let Ok((_, set_number)) = self.epoch_set(header) {
					trace!(target: "engine", "Reporting benign misbehaviour (cause: InvalidSeal) at block #{}, epoch set number {}. Own address: {}",
						header.number(), set_number, self.address().unwrap_or_default());
					self.validators.report_benign(header.author(), set_number, header.number());
				}

				Err(BlockError::InvalidSeal.into())
			}
			Err(e) => Err(e.into()),
			Ok(()) => Ok(()),
		}
	}

	/// Do the step and gas limit validation.
	fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
		let step = header_step(header, self.empty_steps_transition)?;
		let parent_step = header_step(parent, self.empty_steps_transition)?;

		let (validators, set_number) = self.epoch_set(header)?;

		// Ensure header is from the step after parent.
		if step == parent_step
			|| (header.number() >= self.validate_step_transition && step <= parent_step) {
			warn!(target: "engine", "Multiple blocks proposed for step {}.", parent_step);

			self.validators.report_malicious(header.author(), set_number, header.number(), Default::default());
			Err(EngineError::DoubleVote(*header.author()))?;
		}

		// Report malice if the validator produced other sibling blocks in the same step.
		let received_step_key = (step, *header.author());
		let new_hash = header.hash();
		if self.received_step_hashes.read().get(&received_step_key).map_or(false, |h| *h != new_hash) {
			trace!(target: "engine", "Validator {} produced sibling blocks in the same step", header.author());
			self.validators.report_malicious(header.author(), set_number, header.number(), Default::default());
		} else {
			self.received_step_hashes.write().insert(received_step_key, new_hash);
		}

		// Remove hash records older than two full rounds of steps (picked as a reasonable trade-off between
		// memory consumption and fault-tolerance).
		let sibling_malice_detection_period = 2 * validators.count(&parent.hash()) as u64;
		let oldest_step = parent_step.saturating_sub(sibling_malice_detection_period);
		if oldest_step > 0 {
			let mut rsh = self.received_step_hashes.write();
			let new_rsh = rsh.split_off(&(oldest_step, Address::zero()));
			*rsh = new_rsh;
		}

		// If empty step messages are enabled we will validate the messages in the seal, missing messages are not
		// reported as there's no way to tell whether the empty step message was never sent or simply not included.
		let empty_steps_len = if header.number() >= self.empty_steps_transition {
			let validate_empty_steps = || -> Result<usize, Error> {
				let strict_empty_steps = header.number() >= self.strict_empty_steps_transition;
				let empty_steps = header_empty_steps(header)?;
				let empty_steps_len = empty_steps.len();
				let mut prev_empty_step = 0;

				for empty_step in empty_steps {
					if empty_step.step <= parent_step || empty_step.step >= step {
						Err(EngineError::InsufficientProof(
							format!("empty step proof for invalid step: {:?}", empty_step.step)))?;
					}

					if empty_step.parent_hash != *header.parent_hash() {
						Err(EngineError::InsufficientProof(
							format!("empty step proof for invalid parent hash: {:?}", empty_step.parent_hash)))?;
					}

					if !empty_step.verify(&*validators).unwrap_or(false) {
						Err(EngineError::InsufficientProof(
							format!("invalid empty step proof: {:?}", empty_step)))?;
					}

					if strict_empty_steps {
						if empty_step.step <= prev_empty_step {
							Err(EngineError::InsufficientProof(format!(
								"{} empty step: {:?}",
								if empty_step.step == prev_empty_step { "duplicate" } else { "unordered" },
								empty_step
							)))?;
						}

						prev_empty_step = empty_step.step;
					}
				}

				Ok(empty_steps_len)
			};

			match validate_empty_steps() {
				Ok(len) => len,
				Err(err) => {
					trace!(target: "engine", "Reporting benign misbehaviour (cause: invalid empty steps) at block #{}, epoch set number {}. Own address: {}",
						header.number(), set_number, self.address().unwrap_or_default());
					self.validators.report_benign(header.author(), set_number, header.number());
					return Err(err);
				},
			}
		} else {
			self.report_skipped(header, step, parent_step, &*validators, set_number);

			0
		};

		if header.number() >= self.validate_score_transition {
			let expected_difficulty = calculate_score(parent_step.into(), step.into(), empty_steps_len.into());
			if header.difficulty() != &expected_difficulty {
				return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: expected_difficulty, found: header.difficulty().clone() })));
			}
		}

		Ok(())
	}

	// Check the validators.
	fn verify_block_external(&self, header: &Header) -> Result<(), Error> {
		let (validators, set_number) = self.epoch_set(header)?;

		// verify signature against fixed list, but reports should go to the
		// contract itself.
		let res = verify_external(header, &*validators, self.empty_steps_transition);
		match res {
			Err(Error::Engine(EngineError::NotProposer(_))) => {
				trace!(target: "engine", "Reporting benign misbehaviour (cause: block from incorrect proposer) at block #{}, epoch set number {}. Own address: {}",
					header.number(), set_number, self.address().unwrap_or_default());
				self.validators.report_benign(header.author(), set_number, header.number());
			},
			Ok(_) => {
				// we can drop all accumulated empty step messages that are older than this header's step
				let header_step = header_step(header, self.empty_steps_transition)?;
				self.clear_empty_steps(header_step.into());
			},
			_ => {},
		}
		res
	}

	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		self.validators.genesis_epoch_data(header, call)
			.map(|set_proof| combine_proofs(0, &set_proof, &[]))
	}

	fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData) -> engine::EpochChange {
		if self.immediate_transitions { return engine::EpochChange::No }

		let first = header.number() == 0;
		self.validators.signals_epoch_end(first, header, aux)
	}

	fn is_epoch_end_light(
		&self,
		chain_head: &Header,
		chain: &Headers<Header>,
		transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		// epochs only matter if we want to support light clients.
		if self.immediate_transitions { return None }

		let epoch_transition_hash = {
			let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
				Some(client) => client,
				None => {
					warn!(target: "engine", "Unable to check for epoch end: missing client ref.");
					return None;
				}
			};

			let mut epoch_manager = self.epoch_manager.lock();
			if !epoch_manager.zoom_to_after(&*client, &self.machine, &*self.validators, *chain_head.parent_hash()) {
				return None;
			}

			epoch_manager.epoch_transition_hash
		};

		let mut hash = *chain_head.parent_hash();

		let mut ancestry = std::iter::repeat_with(move || {
			chain(hash).and_then(|header| {
				if header.number() == 0 { return None }
				hash = *header.parent_hash();
				Some(header)
			})
		})
			.while_some()
			.take_while(|header| header.hash() != epoch_transition_hash);

		let finalized = self.build_finality(chain_head, &mut ancestry);

		self.is_epoch_end(chain_head, &finalized, chain, transition_store)
	}

	fn is_epoch_end(
		&self,
		chain_head: &Header,
		finalized: &[H256],
		chain: &Headers<Header>,
		transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		// epochs only matter if we want to support light clients.
		if self.immediate_transitions { return None }

		let first = chain_head.number() == 0;

		// Apply transitions that don't require finality and should be enacted immediately (e.g from chain spec)
		if let Some(change) = self.validators.is_epoch_end(first, chain_head) {
			info!(target: "engine", "Immediately applying validator set change signalled at block {}", chain_head.number());
			self.epoch_manager.lock().note_new_epoch();
			let change = combine_proofs(chain_head.number(), &change, &[]);
			return Some(change)
		}

		// check transition store for pending transitions against recently finalized blocks
		for finalized_hash in finalized {
			if let Some(pending) = transition_store(*finalized_hash) {
				// walk the chain backwards from current head until finalized_hash
				// to construct transition proof. author == ec_recover(sig) known
				// since the blocks are in the DB.
				let mut hash = chain_head.hash();
				let mut finality_proof: Vec<_> = std::iter::repeat_with(move || {
					chain(hash).and_then(|header| {
						hash = *header.parent_hash();
						if header.number() == 0 { None }
						else { Some(header) }
					})
				})
					.while_some()
					.take_while(|h| h.hash() != *finalized_hash)
					.collect();

				let finalized_header = if *finalized_hash == chain_head.hash() {
					// chain closure only stores ancestry, but the chain head is also unfinalized.
					chain_head.clone()
				} else {
					chain(*finalized_hash)
						.expect("header is finalized; finalized headers must exist in the chain; qed")
				};

				let signal_number = finalized_header.number();
				info!(target: "engine", "Applying validator set change signalled at block {}", signal_number);

				finality_proof.push(finalized_header);
				finality_proof.reverse();

				let finality_proof = ::rlp::encode_list(&finality_proof);

				self.epoch_manager.lock().note_new_epoch();

				// We turn off can_propose here because upon validator set change there can
				// be two valid proposers for a single step: one from the old set and
				// one from the new.
				//
				// This way, upon encountering an epoch change, the proposer from the
				// new set will be forced to wait until the next step to avoid sealing a
				// block that breaks the invariant that the parent's step < the block's step.
				self.step.can_propose.store(false, AtomicOrdering::SeqCst);
				return Some(combine_proofs(signal_number, &pending.proof, &*finality_proof));
			}
		}

		None
	}

	fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a> {
		let (signal_number, set_proof, finality_proof) = match destructure_proofs(proof) {
			Ok(x) => x,
			Err(e) => return ConstructedVerifier::Err(e),
		};

		let first = signal_number == 0;
		match self.validators.epoch_set(first, &self.machine, signal_number, set_proof) {
			Ok((list, finalize)) => {
				let verifier = Box::new(EpochVerifier {
					step: self.step.clone(),
					subchain_validators: list,
					empty_steps_transition: self.empty_steps_transition,
					two_thirds_majority_transition: self.two_thirds_majority_transition,
				});

				match finalize {
					Some(finalize) => ConstructedVerifier::Unconfirmed(verifier, finality_proof, finalize),
					None => ConstructedVerifier::Trusted(verifier),
				}
			}
			Err(e) => ConstructedVerifier::Err(e),
		}
	}

	fn register_client(&self, client: Weak<dyn EngineClient>) {
		*self.client.write() = Some(client.clone());
		self.validators.register_client(client);
	}

	fn set_signer(&self, signer: Option<Box<dyn EngineSigner>>) {
		*self.signer.write() = signer;
	}

	fn sign(&self, hash: H256) -> Result<Signature, Error> {
		Ok(self.signer.read()
			.as_ref()
			.ok_or(parity_crypto::publickey::Error::InvalidAddress)?
			.sign(hash)?
		)
	}

	fn snapshot_mode(&self) -> Snapshotting {
		if self.immediate_transitions {
			Snapshotting::Unsupported
		} else {
			Snapshotting::PoA
		}
	}

	fn ancestry_actions(&self, header: &Header, ancestry: &mut dyn Iterator<Item=ExtendedHeader>) -> Vec<AncestryAction> {
		let finalized = self.build_finality(
			header,
			&mut ancestry.take_while(|e| !e.is_finalized).map(|e| e.header),
		);

		if !finalized.is_empty() {
			debug!(target: "finality", "Finalizing blocks: {:?}", finalized);
		}

		finalized.into_iter().map(AncestryAction::MarkFinalized).collect()
	}

	fn params(&self) -> &CommonParams {
		self.machine.params()
	}

	fn gas_limit_override(&self, header: &Header) -> Option<U256> {
		let (_, &address) = self.block_gas_limit_contract_transitions.range(..=header.number()).last()?;
		let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
			Some(client) => client,
			None => {
				error!(target: "engine", "Unable to prepare block: missing client ref.");
				return None;
			}
		};
		let full_client = match client.as_full_client() {
			Some(full_client) => full_client,
			None => {
				error!(target: "engine", "Failed to upgrade to BlockchainClient.");
				return None;
			}
		};
		if let Some(limit) = self.gas_limit_override_cache.lock().get_mut(&header.hash()) {
			return *limit;
		}
		let limit = block_gas_limit(full_client, header, address);
		self.gas_limit_override_cache.lock().insert(header.hash(), limit);
		limit
	}
}

/// A helper accumulator function mapping a step duration and a step duration transition timestamp
/// to the corresponding step number and the correct starting second of the step.
fn next_step_time_duration(info: StepDurationInfo, time: u64) -> Option<(u64, u64)>
{
	let step_diff = time.checked_add(info.step_duration)?
		.checked_sub(1)?
		.checked_sub(info.transition_timestamp)?
		.checked_div(info.step_duration)?;
	let time_diff = step_diff.checked_mul(info.step_duration)?;
	Some((
		info.transition_step.checked_add(step_diff)?,
		info.transition_timestamp.checked_add(time_diff)?,
	))
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use std::str::FromStr;
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering as AtomicOrdering};
	use std::time::Duration;
	use keccak_hash::keccak;
	use accounts::AccountProvider;
	use ethabi_contract::use_contract;
	use ethereum_types::{Address, H520, H256, U256};
	use parity_crypto::publickey::Signature;
	use common_types::{
		header::Header,
		engines::{Seal, params::CommonParams},
		ids::BlockId,
		errors::{EthcoreError as Error, EngineError},
		transaction::{Action, Transaction},
	};
	use rlp::encode;
	use ethcore::{
		block::*,
		miner::{Author, MinerService},
		test_helpers::{
			generate_dummy_client_with_spec, generate_dummy_client_with_spec_and_data, get_temp_state_db,
			TestNotify
		},
	};
	use engine::Engine;
	use block_reward::BlockRewardContract;
	use machine::Machine;
	use spec::{self, Spec};
	use validator_set::{TestSet, SimpleList};
	use ethjson;
	use serde_json;

	use super::{
		AuthorityRoundParams, AuthorityRound, EmptyStep, SealedEmptyStep, StepDurationInfo,
		calculate_score, util::BoundContract, next_step_time_duration,
	};

	fn build_aura<F>(f: F) -> Arc<AuthorityRound> where
		F: FnOnce(&mut AuthorityRoundParams),
	{
		let mut params = AuthorityRoundParams {
			step_durations: [(0, 1)].to_vec().into_iter().collect(),
			start_step: Some(1),
			validators: Box::new(TestSet::default()),
			validate_score_transition: 0,
			validate_step_transition: 0,
			immediate_transitions: true,
			maximum_uncle_count_transition: 0,
			maximum_uncle_count: 0,
			empty_steps_transition: u64::max_value(),
			maximum_empty_steps: 0,
			block_reward: Default::default(),
			block_reward_contract_transitions: Default::default(),
			strict_empty_steps_transition: 0,
			two_thirds_majority_transition: 0,
			randomness_contract_address: BTreeMap::new(),
			block_gas_limit_contract_transitions: BTreeMap::new(),
		};

		// mutate aura params
		f(&mut params);
		// create engine
		let mut c_params = CommonParams::default();
		c_params.gas_limit_bound_divisor = 5.into();
		let machine = Machine::regular(c_params, Default::default());
		AuthorityRound::new(params, machine).unwrap()
	}

	#[test]
	fn has_valid_metadata() {
		let engine = spec::new_test_round().engine;
		assert_eq!(engine.name(), "AuthorityRound");
	}

	#[test]
	fn can_return_schedule() {
		let engine = spec::new_test_round().engine;
		let schedule = engine.schedule(10000000);

		assert!(schedule.stack_limit > 0);
	}

	#[test]
	fn can_do_signature_verification_fail() {
		let engine = spec::new_test_round().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![encode(&H520::default())]);

		let verify_result = engine.verify_block_external(&header);
		assert!(verify_result.is_err());
	}

	#[test]
	fn generates_seal_and_does_not_double_propose() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
		let spec = spec::new_test_round();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		if let Seal::Regular(seal) = engine.generate_seal(&b1, &genesis_header) {
			assert!(b1.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(&b1, &genesis_header) == Seal::None);
		} else {
			panic!("block 1 not sealed");
		}
	}

	#[test]
	fn generates_seal_iff_sealer_is_set() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
		let spec = spec::new_test_round();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header,
								last_hashes.clone(), addr1, (3141562.into(), 31415620.into()),
								vec![], false)
			.unwrap().close_and_lock().unwrap();
		// Not a signer. A seal cannot be generated.
		assert!(engine.generate_seal(&b1, &genesis_header) == Seal::None);
		// Become a signer.
		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		if let Seal::Regular(seal) = engine.generate_seal(&b1, &genesis_header) {
			assert!(b1.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(&b1, &genesis_header) == Seal::None);
		} else {
			panic!("block 1 not sealed");
		}
		// Stop being a signer.
		engine.set_signer(None);
		// Make a step first and then create a new block in that new step.
		engine.step();
		let addr2 = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();
		let mut header2 = genesis_header.clone();
		header2.set_number(2);
		header2.set_author(addr2);
		header2.set_parent_hash(header2.hash());
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &header2,
								last_hashes, addr2, (3141562.into(), 31415620.into()),
								vec![], false)
			.unwrap().close_and_lock().unwrap();
		// Not a signer. A seal cannot be generated.
		assert!(engine.generate_seal(&b2, &header2) == Seal::None);
		// Become a signer once more.
		engine.set_signer(Some(Box::new((tap, addr2, "0".into()))));
		if let Seal::Regular(seal) = engine.generate_seal(&b2, &header2) {
			assert!(b2.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(&b2, &header2) == Seal::None);
		} else {
			panic!("block 2 not sealed");
		}
	}

	#[test]
	fn checks_difficulty_in_generate_seal() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
		let addr2 = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

		let spec = spec::new_test_round();
		let engine = &*spec.engine;

		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b1 = b1.close_and_lock().unwrap();
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes, addr2, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b2 = b2.close_and_lock().unwrap();

		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		match engine.generate_seal(&b1, &genesis_header) {
			Seal::None => panic!("wrong seal"),
			Seal::Regular(_) => {
				engine.step();

				engine.set_signer(Some(Box::new((tap.clone(), addr2, "0".into()))));
				match engine.generate_seal(&b2, &genesis_header) {
					Seal::Regular(_) => panic!("sealed despite wrong difficulty"),
					Seal::None => {}
				}
			}
		}
	}

	#[test]
	fn proposer_switching() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();
		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&0usize)]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_author(addr);

		let engine = spec::new_test_round().engine;

		// Two validators.
		// Spec starts with step 2.
		header.set_difficulty(calculate_score(0, 2, 0));
		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		header.set_seal(vec![encode(&2usize), encode(&(&*signature as &[u8]))]);
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		assert!(engine.verify_block_external(&header).is_err());
		header.set_difficulty(calculate_score(0, 1, 0));
		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		header.set_seal(vec![encode(&1usize), encode(&(&*signature as &[u8]))]);
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		assert!(engine.verify_block_external(&header).is_ok());
	}

	#[test]
	fn rejects_future_block() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&0usize)]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_author(addr);

		let engine = spec::new_test_round().engine;

		// Two validators.
		// Spec starts with step 2.
		header.set_difficulty(calculate_score(0, 1, 0));
		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		header.set_seal(vec![encode(&1usize), encode(&(&*signature as &[u8]))]);
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		assert!(engine.verify_block_external(&header).is_ok());
		header.set_seal(vec![encode(&5usize), encode(&(&*signature as &[u8]))]);
		assert!(engine.verify_block_basic(&header).is_err());
	}

	#[test]
	fn rejects_step_backwards() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&4usize)]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_author(addr);

		let engine = spec::new_test_round().engine;

		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		// Two validators.
		// Spec starts with step 2.
		header.set_seal(vec![encode(&5usize), encode(&(&*signature as &[u8]))]);
		header.set_difficulty(calculate_score(4, 5, 0));
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		header.set_seal(vec![encode(&3usize), encode(&(&*signature as &[u8]))]);
		header.set_difficulty(calculate_score(4, 3, 0));
		assert!(engine.verify_block_family(&header, &parent_header).is_err());
	}

	#[test]
	fn reports_skipped() {
		let validator1 = Address::from_low_u64_be(1);
		let validator2 = Address::from_low_u64_be(2);
		let last_benign = Arc::new(AtomicUsize::new(0));

		let aura = build_aura(|p| {
			let validator_set = TestSet::new(
				Default::default(),
				last_benign.clone(),
				vec![validator1, validator2],
			);
			p.validators = Box::new(validator_set);
		});

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&1usize)]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_difficulty(calculate_score(1, 3, 0));
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_seal(vec![encode(&3usize)]);

		// Do not report when signer not present.
		assert!(aura.verify_block_family(&header, &parent_header).is_ok());
		assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 0);

		aura.set_signer(Some(Box::new((
			Arc::new(AccountProvider::transient_provider()),
			validator2,
			"".into(),
		))));

		// Do not report on steps skipped between genesis and first block.
		header.set_number(1);
		assert!(aura.verify_block_family(&header, &parent_header).is_ok());
		assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 0);

		// Report on skipped steps otherwise.
		header.set_number(2);
		assert!(aura.verify_block_family(&header, &parent_header).is_ok());
		assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 2);
	}

	#[test]
	fn reports_multiple_blocks_per_step() {
		let tap = AccountProvider::transient_provider();
		let addr0 = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();
		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();

		let validator_set = TestSet::from_validators(vec![addr0, addr1]);
		let aura = build_aura(|p| p.validators = Box::new(validator_set.clone()));

		aura.set_signer(Some(Box::new((Arc::new(tap), addr0, "0".into()))));

		let mut parent_header: Header = Header::default();
		parent_header.set_number(2);
		parent_header.set_seal(vec![encode(&1usize)]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_number(3);
		header.set_difficulty(calculate_score(1, 2, 0));
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_seal(vec![encode(&2usize)]);
		header.set_author(addr1);

		// First sibling block.
		assert!(aura.verify_block_family(&header, &parent_header).is_ok());
		assert_eq!(validator_set.last_malicious(), 0);

		// Second sibling block: should be reported.
		header.set_gas_limit("222223".parse::<U256>().unwrap());
		assert!(aura.verify_block_family(&header, &parent_header).is_ok());
		assert_eq!(validator_set.last_malicious(), 3);
	}

	#[test]
	fn test_uncles_transition() {
		let aura = build_aura(|params| {
			params.maximum_uncle_count_transition = 1;
		});

		assert_eq!(aura.maximum_uncle_count(0), 2);
		assert_eq!(aura.maximum_uncle_count(1), 0);
		assert_eq!(aura.maximum_uncle_count(100), 0);
	}

	#[test]
	#[should_panic(expected="counter is too high")]
	fn test_counter_increment_too_high() {
		use super::Step;
		let step = Step {
			calibrate: false,
			inner: AtomicU64::new(::std::u64::MAX),
			durations: [StepDurationInfo {
				transition_step: 0,
				transition_timestamp: 0,
				step_duration: 1,
			}].to_vec().into_iter().collect(),
		};
		step.increment();
	}

	#[test]
	#[should_panic(expected="step counter under- or overflow")]
	fn test_counter_duration_remaining_too_high() {
		use super::Step;
		let step = Step {
			calibrate: false,
			inner: AtomicU64::new(::std::u64::MAX),
			durations: [StepDurationInfo {
				transition_step: 0,
				transition_timestamp: 0,
				step_duration: 1,
			}].to_vec().into_iter().collect(),
		};
		step.duration_remaining();
	}

	#[test]
	fn test_next_step_time_duration() {
		// At step 7 (time 1000), we transitioned to step duration 10.
		let info = StepDurationInfo {
			step_duration: 10,
			transition_step: 7,
			transition_timestamp: 1000,
		};
		// So the next transition can happen e.g. at step 12 (time 1050) or 13 (time 1060).
		assert_eq!(Some((12, 1050)), next_step_time_duration(info, 1050));
		assert_eq!(Some((13, 1060)), next_step_time_duration(info, 1051));
		assert_eq!(Some((13, 1060)), next_step_time_duration(info, 1055));
		// The next transition could also happen immediately.
		assert_eq!(Some((7, 1000)), next_step_time_duration(info, 1000));
	}

	#[test]
	fn test_change_step_duration() {
		use super::Step;
		use std::thread;

		let now = super::unix_now().as_secs();
		let step = Step {
			calibrate: true,
			inner: AtomicU64::new(::std::u64::MAX),
			durations: [
				StepDurationInfo { transition_step: 0, transition_timestamp: 0, step_duration: 1 },
				StepDurationInfo { transition_step: now, transition_timestamp: now, step_duration: 2 },
				StepDurationInfo { transition_step: now + 1, transition_timestamp: now + 2, step_duration: 4 },
			].to_vec().into_iter().collect(),
		};
		// calibrated step `now`
		step.calibrate();
		let duration_remaining = step.duration_remaining();
		assert_eq!(step.inner.load(AtomicOrdering::SeqCst), now);
		assert!(duration_remaining <= Duration::from_secs(2));
		thread::sleep(duration_remaining);
		step.increment();
		// calibrated step `now + 1`
		step.calibrate();
		let duration_remaining = step.duration_remaining();
		assert_eq!(step.inner.load(AtomicOrdering::SeqCst), now + 1);
		assert!(duration_remaining > Duration::from_secs(2));
		assert!(duration_remaining <= Duration::from_secs(4));
	}

	#[test]
	#[should_panic(expected="called `Result::unwrap()` on an `Err` value: Engine(Custom(\"step duration cannot be 0\"))")]
	fn test_step_duration_zero() {
		build_aura(|params| {
			params.step_durations = [(0, 0)].to_vec().into_iter().collect();
		});
	}

	fn setup_empty_steps() -> (Spec, Arc<AccountProvider>, Vec<Address>) {
		let spec = spec::new_test_round_empty_steps();
		let tap = Arc::new(AccountProvider::transient_provider());

		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
		let addr2 = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

		let accounts = vec![addr1, addr2];

		(spec, tap, accounts)
	}

	fn empty_step(engine: &dyn Engine, step: u64, parent_hash: &H256) -> EmptyStep {
		let empty_step_rlp = super::empty_step_rlp(step, parent_hash);
		let signature = engine.sign(keccak(&empty_step_rlp)).unwrap().into();
		let parent_hash = parent_hash.clone();
		EmptyStep { step, signature, parent_hash }
	}

	fn sealed_empty_step(engine: &dyn Engine, step: u64, parent_hash: &H256) -> SealedEmptyStep {
		let empty_step_rlp = super::empty_step_rlp(step, parent_hash);
		let signature = engine.sign(keccak(&empty_step_rlp)).unwrap().into();
		SealedEmptyStep { signature, step }
	}

	fn set_empty_steps_seal(header: &mut Header, step: u64, block_signature: &Signature, empty_steps: &[SealedEmptyStep]) {
		header.set_seal(vec![
			encode(&(step as usize)),
			encode(&(&**block_signature as &[u8])),
			::rlp::encode_list(&empty_steps),
		]);
	}

	fn assert_insufficient_proof<T: ::std::fmt::Debug>(result: Result<T, Error>, contains: &str) {
		match result {
			Err(Error::Engine(EngineError::InsufficientProof(ref s))) =>{
				assert!(s.contains(contains), "Expected {:?} to contain {:?}", s, contains);
			},
			e => assert!(false, "Unexpected result: {:?}", e),
		}
	}

	#[test]
	fn broadcast_empty_step_message() {
		let (spec, tap, accounts) = setup_empty_steps();

		let addr1 = accounts[0];

		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();

		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let client = generate_dummy_client_with_spec(spec::new_test_round_empty_steps);
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client) as _);

		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));

		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// the block is empty so we don't seal and instead broadcast an empty step message
		assert_eq!(engine.generate_seal(&b1, &genesis_header), Seal::None);

		// spec starts with step 2
		let empty_step_rlp = encode(&empty_step(engine, 2, &genesis_header.hash()));

		// we've received the message
		assert!(notify.messages.read().contains(&empty_step_rlp));
		let len = notify.messages.read().len();

		// make sure that we don't generate empty step for the second time
		assert_eq!(engine.generate_seal(&b1, &genesis_header), Seal::None);
		assert_eq!(len, notify.messages.read().len());
	}

	#[test]
	fn seal_with_empty_steps() {
		let (spec, tap, accounts) = setup_empty_steps();

		let addr1 = accounts[0];
		let addr2 = accounts[1];

		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();

		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let client = generate_dummy_client_with_spec(spec::new_test_round_empty_steps);
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client) as _);

		// step 2
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// since the block is empty it isn't sealed and we generate empty steps
		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		assert_eq!(engine.generate_seal(&b1, &genesis_header), Seal::None);
		engine.step();

		// step 3
		let mut b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes.clone(), addr2, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		b2.push_transaction(Transaction {
			action: Action::Create,
			nonce: U256::from(0),
			gas_price: U256::from(3000),
			gas: U256::from(53_000),
			value: U256::from(1),
			data: vec![],
		}.fake_sign(addr2), None).unwrap();
		let b2 = b2.close_and_lock().unwrap();

		// we will now seal a block with 1tx and include the accumulated empty step message
		engine.set_signer(Some(Box::new((tap.clone(), addr2, "0".into()))));
		if let Seal::Regular(seal) = engine.generate_seal(&b2, &genesis_header) {
			engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
			let empty_step2 = sealed_empty_step(engine, 2, &genesis_header.hash());
			let empty_steps = ::rlp::encode_list(&vec![empty_step2]);

			assert_eq!(seal[0], encode(&3usize));
			assert_eq!(seal[2], empty_steps);
		}
	}

	#[test]
	fn seal_empty_block_with_empty_steps() {
		let (spec, tap, accounts) = setup_empty_steps();

		let addr1 = accounts[0];
		let addr2 = accounts[1];

		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db3 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();

		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let client = generate_dummy_client_with_spec(spec::new_test_round_empty_steps);
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client) as _);

		// step 2
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// since the block is empty it isn't sealed and we generate empty steps
		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		assert_eq!(engine.generate_seal(&b1, &genesis_header), Seal::None);
		engine.step();

		// step 3
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes.clone(), addr2, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b2 = b2.close_and_lock().unwrap();
		engine.set_signer(Some(Box::new((tap.clone(), addr2, "0".into()))));
		assert_eq!(engine.generate_seal(&b2, &genesis_header), Seal::None);
		engine.step();

		// step 4
		// the spec sets the maximum_empty_steps to 2 so we will now seal an empty block and include the empty step messages
		let b3 = OpenBlock::new(engine, Default::default(), false, db3, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b3 = b3.close_and_lock().unwrap();

		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		if let Seal::Regular(seal) = engine.generate_seal(&b3, &genesis_header) {
			let empty_step2 = sealed_empty_step(engine, 2, &genesis_header.hash());
			engine.set_signer(Some(Box::new((tap.clone(), addr2, "0".into()))));
			let empty_step3 = sealed_empty_step(engine, 3, &genesis_header.hash());

			let empty_steps = ::rlp::encode_list(&vec![empty_step2, empty_step3]);

			assert_eq!(seal[0], encode(&4usize));
			assert_eq!(seal[2], empty_steps);
		}
	}

	#[test]
	fn reward_empty_steps() {
		let (spec, tap, accounts) = setup_empty_steps();

		let addr1 = accounts[0];

		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();

		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let client = generate_dummy_client_with_spec(spec::new_test_round_empty_steps);
		engine.register_client(Arc::downgrade(&client) as _);

		// step 2
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// since the block is empty it isn't sealed and we generate empty steps
		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		assert_eq!(engine.generate_seal(&b1, &genesis_header), Seal::None);
		engine.step();

		// step 3
		// the signer of the accumulated empty step message should be rewarded
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let addr1_balance = b2.state.balance(&addr1).unwrap();

		// after closing the block `addr1` should be reward twice, one for the included empty step message and another for block creation
		let b2 = b2.close_and_lock().unwrap();

		// the spec sets the block reward to 10
		assert_eq!(b2.state.balance(&addr1).unwrap(), addr1_balance + (10 * 2))
	}

	#[test]
	fn verify_seal_empty_steps() {
		let (spec, tap, accounts) = setup_empty_steps();
		let addr1 = accounts[0];
		let addr2 = accounts[1];
		let engine = &*spec.engine;

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&0usize)]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());

		let mut header: Header = Header::default();
		header.set_parent_hash(parent_header.hash());
		header.set_number(1);
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_author(addr1);

		let signature = tap.sign(addr1, Some("1".into()), header.bare_hash()).unwrap();

		// empty step with invalid step
		let empty_steps = vec![SealedEmptyStep { signature: H520::zero(), step: 2 }];
		set_empty_steps_seal(&mut header, 2, &signature, &empty_steps);

		assert_insufficient_proof(
			engine.verify_block_family(&header, &parent_header),
			"invalid step"
		);

		// empty step with invalid signature
		let empty_steps = vec![SealedEmptyStep { signature: H520::zero(), step: 1 }];
		set_empty_steps_seal(&mut header, 2, &signature, &empty_steps);

		assert_insufficient_proof(
			engine.verify_block_family(&header, &parent_header),
			"invalid empty step proof"
		);

		// empty step with valid signature from incorrect proposer for step
		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		let empty_steps = vec![sealed_empty_step(engine, 1, &parent_header.hash())];
		set_empty_steps_seal(&mut header, 2, &signature, &empty_steps);

		assert_insufficient_proof(
			engine.verify_block_family(&header, &parent_header),
			"invalid empty step proof"
		);

		// valid empty steps
		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		let empty_step2 = sealed_empty_step(engine, 2, &parent_header.hash());
		engine.set_signer(Some(Box::new((tap.clone(), addr2, "0".into()))));
		let empty_step3 = sealed_empty_step(engine, 3, &parent_header.hash());

		let empty_steps = vec![empty_step2, empty_step3];
		header.set_difficulty(calculate_score(0, 4, 2));
		let signature = tap.sign(addr1, Some("1".into()), header.bare_hash()).unwrap();
		set_empty_steps_seal(&mut header, 4, &signature, &empty_steps);

		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
	}

	#[test]
	fn block_reward_contract() {
		let spec = spec::new_test_round_block_reward_contract();
		let tap = Arc::new(AccountProvider::transient_provider());

		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();

		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();

		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let client = generate_dummy_client_with_spec(spec::new_test_round_block_reward_contract);
		engine.register_client(Arc::downgrade(&client) as _);

		// step 2
		let b1 = OpenBlock::new(
			engine,
			Default::default(),
			false,
			db1,
			&genesis_header,
			last_hashes.clone(),
			addr1,
			(3141562.into(), 31415620.into()),
			vec![],
			false,
		).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// since the block is empty it isn't sealed and we generate empty steps
		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));
		assert_eq!(engine.generate_seal(&b1, &genesis_header), Seal::None);
		engine.step();

		// step 3
		// the signer of the accumulated empty step message should be rewarded
		let b2 = OpenBlock::new(
			engine,
			Default::default(),
			false,
			db2,
			&genesis_header,
			last_hashes.clone(),
			addr1,
			(3141562.into(), 31415620.into()),
			vec![],
			false,
		).unwrap();
		let addr1_balance = b2.state.balance(&addr1).unwrap();

		// after closing the block `addr1` should be reward twice, one for the included empty step
		// message and another for block creation
		let b2 = b2.close_and_lock().unwrap();

		// the contract rewards (1000 + kind) for each benefactor/reward kind
		assert_eq!(
			b2.state.balance(&addr1).unwrap(),
			addr1_balance + (1000 + 0) + (1000 + 2),
		)
	}

	#[test]
	fn randomness_contract() -> Result<(), super::util::CallError> {
		use_contract!(rand_contract, "../../res/contracts/test_authority_round_random.json");

		env_logger::init();

		let contract_addr = Address::from_str("0000000000000000000000000000000000000042").unwrap();
		let client = generate_dummy_client_with_spec_and_data(
			spec::new_test_round_randomness_contract, 0, 0, &[], true
		);

		let tap = Arc::new(AccountProvider::transient_provider());

		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
		// Unlock account so that the engine can decrypt the secret.
		tap.unlock_account_permanently(addr1, "1".into()).expect("unlock");

		let signer = Box::new((tap.clone(), addr1, "1".into()));
		client.miner().set_author(Author::Sealer(signer.clone()));
		client.miner().set_gas_range_target((U256::from(1000000), U256::from(1000000)));

		let engine = client.engine();
		engine.set_signer(Some(signer));
		engine.register_client(Arc::downgrade(&client) as _);
		let bc = BoundContract::new(&*client, BlockId::Latest, contract_addr);

		// First the contract is in the commit phase, and we haven't committed yet.
		assert!(bc.call_const(rand_contract::functions::is_commit_phase::call())?);
		assert!(!bc.call_const(rand_contract::functions::is_committed::call(0, addr1))?);

		// We produce a block and commit.
		engine.step();
		assert!(bc.call_const(rand_contract::functions::is_committed::call(0, addr1))?);

		// After two more blocks we are in the reveal phase...
		engine.step();
		engine.step();
		assert!(bc.call_const(rand_contract::functions::is_reveal_phase::call())?);
		assert!(!bc.call_const(rand_contract::functions::sent_reveal::call(0, addr1))?);
		assert!(bc.call_const(rand_contract::functions::get_value::call())?.is_zero());

		// ...so in the next step, we reveal our random value, and the contract's random value is not zero anymore.
		engine.step();
		assert!(bc.call_const(rand_contract::functions::sent_reveal::call(0, addr1))?);
		assert!(!bc.call_const(rand_contract::functions::get_value::call())?.is_zero());
		Ok(())
	}

	#[test]
	fn extra_info_from_seal() {
		let (spec, tap, accounts) = setup_empty_steps();
		let engine = &*spec.engine;

		let addr1 = accounts[0];
		engine.set_signer(Some(Box::new((tap.clone(), addr1, "1".into()))));

		let mut header: Header = Header::default();
		let empty_step = empty_step(engine, 1, &header.parent_hash());
		let sealed_empty_step = empty_step.sealed();

		header.set_number(2);
		header.set_seal(vec![
			encode(&2usize),
			encode(&H520::default()),
			::rlp::encode_list(&vec![sealed_empty_step]),
		]);

		let info = engine.extra_info(&header);

		let mut expected = BTreeMap::default();
		expected.insert("step".into(), "2".into());
		expected.insert("signature".into(), Signature::from(H520::default()).to_string());
		expected.insert("emptySteps".into(), format!("[{}]", empty_step));

		assert_eq!(info, expected);

		header.set_seal(vec![]);

		assert_eq!(
			engine.extra_info(&header),
			BTreeMap::default(),
		);
	}

	#[test]
	fn test_empty_steps() {
		let engine = build_aura(|p| {
			p.step_durations = [(0, 4)].to_vec().into_iter().collect();
			p.empty_steps_transition = 0;
			p.maximum_empty_steps = 0;
		});

		let parent_hash = H256::from_low_u64_be(1);
		let signature = H520::default();
		let step = |step: u64| EmptyStep {
			step,
			parent_hash,
			signature,
		};

		engine.handle_empty_step_message(step(1));
		engine.handle_empty_step_message(step(3));
		engine.handle_empty_step_message(step(2));
		engine.handle_empty_step_message(step(1));

		assert_eq!(engine.empty_steps(0, 4, parent_hash), vec![step(1), step(2), step(3)]);
		assert_eq!(engine.empty_steps(2, 3, parent_hash), vec![]);
		assert_eq!(engine.empty_steps(2, 4, parent_hash), vec![step(3)]);

		engine.clear_empty_steps(2);

		assert_eq!(engine.empty_steps(0, 3, parent_hash), vec![]);
		assert_eq!(engine.empty_steps(0, 4, parent_hash), vec![step(3)]);
	}

	#[test]
	fn should_reject_duplicate_empty_steps() {
		// given
		let (_spec, tap, accounts) = setup_empty_steps();
		let engine = build_aura(|p| {
			p.validators = Box::new(SimpleList::new(accounts.clone()));
			p.step_durations = [(0, 4)].to_vec().into_iter().collect();
			p.empty_steps_transition = 0;
			p.maximum_empty_steps = 0;
		});

		let mut parent = Header::default();
		parent.set_seal(vec![encode(&0usize)]);

		let mut header = Header::default();
		header.set_number(parent.number() + 1);
		header.set_parent_hash(parent.hash());
		header.set_author(accounts[0]);

		// when
		engine.set_signer(Some(Box::new((tap.clone(), accounts[1], "0".into()))));
		let empty_steps = vec![
			sealed_empty_step(&*engine, 1, &parent.hash()),
			sealed_empty_step(&*engine, 1, &parent.hash()),
		];
		let step = 2;
		let signature = tap.sign(accounts[0], Some("1".into()), header.bare_hash()).unwrap();
		set_empty_steps_seal(&mut header, step, &signature, &empty_steps);
		header.set_difficulty(calculate_score(0, step, empty_steps.len()));

		// then
		assert_insufficient_proof(
			engine.verify_block_family(&header, &parent),
			"duplicate empty step"
		);
	}

	#[test]
	fn should_reject_empty_steps_out_of_order() {
		// given
		let (_spec, tap, accounts) = setup_empty_steps();
		let engine = build_aura(|p| {
			p.validators = Box::new(SimpleList::new(accounts.clone()));
			p.step_durations = [(0, 4)].to_vec().into_iter().collect();
			p.empty_steps_transition = 0;
			p.maximum_empty_steps = 0;
		});

		let mut parent = Header::default();
		parent.set_seal(vec![encode(&0usize)]);

		let mut header = Header::default();
		header.set_number(parent.number() + 1);
		header.set_parent_hash(parent.hash());
		header.set_author(accounts[0]);

		// when
		engine.set_signer(Some(Box::new((tap.clone(), accounts[1], "0".into()))));
		let es1 = sealed_empty_step(&*engine, 1, &parent.hash());
		engine.set_signer(Some(Box::new((tap.clone(), accounts[0], "1".into()))));
		let es2 = sealed_empty_step(&*engine, 2, &parent.hash());

		let mut empty_steps = vec![es2, es1];

		let step = 3;
		let signature = tap.sign(accounts[1], Some("0".into()), header.bare_hash()).unwrap();
		set_empty_steps_seal(&mut header, step, &signature, &empty_steps);
		header.set_difficulty(calculate_score(0, step, empty_steps.len()));

		// then make sure it's rejected because of the order
		assert_insufficient_proof(
			engine.verify_block_family(&header, &parent),
			"unordered empty step"
		);

		// now try to fix the order
		empty_steps.reverse();
		set_empty_steps_seal(&mut header, step, &signature, &empty_steps);
		assert_eq!(engine.verify_block_family(&header, &parent).unwrap(), ());
	}

	#[test]
	fn should_collect_block_reward_transitions() {
		let config = r#"{
			"params": {
				"stepDuration": "5",
				"validators": {
					"list" : ["0x1000000000000000000000000000000000000001"]
				},
				"blockRewardContractTransition": "0",
				"blockRewardContractAddress": "0x2000000000000000000000000000000000000002",
				"blockRewardContractTransitions": {
					"7": "0x3000000000000000000000000000000000000003",
					"42": "0x4000000000000000000000000000000000000004"
				}
			}
		}"#;
		let deserialized: ethjson::spec::AuthorityRound = serde_json::from_str(config).unwrap();
		let params = AuthorityRoundParams::from(deserialized.params);
		for ((block_num1, address1), (block_num2, address2)) in
			params.block_reward_contract_transitions.iter().zip(
				[(0u64, BlockRewardContract::new_from_address(Address::from_str("2000000000000000000000000000000000000002").unwrap())),
				 (7u64, BlockRewardContract::new_from_address(Address::from_str("3000000000000000000000000000000000000003").unwrap())),
				 (42u64, BlockRewardContract::new_from_address(Address::from_str("4000000000000000000000000000000000000004").unwrap())),
				].iter())
		{
			assert_eq!(block_num1, block_num2);
			assert_eq!(address1, address2);
		}
	}

	#[test]
	#[should_panic(expected="blockRewardContractTransition should be less than any of the keys in blockRewardContractTransitions")]
	fn should_reject_out_of_order_block_reward_transition() {
		let config = r#"{
			"params": {
				"stepDuration": "5",
				"validators": {
					"list" : ["0x1000000000000000000000000000000000000001"]
				},
				"blockRewardContractTransition": "7",
				"blockRewardContractAddress": "0x2000000000000000000000000000000000000002",
				"blockRewardContractTransitions": {
					"0": "0x3000000000000000000000000000000000000003",
					"42": "0x4000000000000000000000000000000000000004"
				}
			}
		}"#;
		let deserialized: ethjson::spec::AuthorityRound = serde_json::from_str(config).unwrap();
		AuthorityRoundParams::from(deserialized.params);
	}
}
