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

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::{cmp, fmt};
use std::iter::FromIterator;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Weak, Arc};
use std::time::{UNIX_EPOCH, SystemTime, Duration};

use block::*;
use client::EngineClient;
use engines::{Engine, Seal, EngineError, ConstructedVerifier};
use engines::block_reward;
use engines::block_reward::{BlockRewardContract, RewardKind};
use error::{Error, ErrorKind, BlockError};
use ethjson;
use machine::{AuxiliaryData, Call, EthereumMachine};
use hash::keccak;
use super::signer::EngineSigner;
use super::validator_set::{ValidatorSet, SimpleList, new_validator_set};
use self::finality::RollingFinality;
use ethkey::{self, Signature};
use io::{IoContext, IoHandler, TimerToken, IoService};
use itertools::{self, Itertools};
use rlp::{encode, Decodable, DecoderError, Encodable, RlpStream, Rlp};
use ethereum_types::{H256, H520, Address, U128, U256};
use parking_lot::{Mutex, RwLock};
use types::BlockNumber;
use types::header::{Header, ExtendedHeader};
use types::ancestry_action::AncestryAction;
use unexpected::{Mismatch, OutOfBounds};

mod finality;

/// `AuthorityRound` params.
pub struct AuthorityRoundParams {
	/// Time to wait before next block or authority switching,
	/// in seconds.
	///
	/// Deliberately typed as u16 as too high of a value leads
	/// to slow block issuance.
	pub step_duration: u16,
	/// Starting step,
	pub start_step: Option<u64>,
	/// Valid validators.
	pub validators: Box<ValidatorSet>,
	/// Chain score validation transition block.
	pub validate_score_transition: u64,
	/// Monotonic step validation transition block.
	pub validate_step_transition: u64,
	/// Immediate transitions.
	pub immediate_transitions: bool,
	/// Block reward in base units.
	pub block_reward: U256,
	/// Block reward contract transition block.
	pub block_reward_contract_transition: u64,
	/// Block reward contract.
	pub block_reward_contract: Option<BlockRewardContract>,
	/// Number of accepted uncles transition block.
	pub maximum_uncle_count_transition: u64,
	/// Number of accepted uncles.
	pub maximum_uncle_count: usize,
	/// Empty step messages transition block.
	pub empty_steps_transition: u64,
	/// Number of accepted empty steps.
	pub maximum_empty_steps: usize,
	/// Transition block to strict empty steps validation.
	pub strict_empty_steps_transition: u64,
}

const U16_MAX: usize = ::std::u16::MAX as usize;

impl From<ethjson::spec::AuthorityRoundParams> for AuthorityRoundParams {
	fn from(p: ethjson::spec::AuthorityRoundParams) -> Self {
		let mut step_duration_usize: usize = p.step_duration.into();
		if step_duration_usize > U16_MAX {
			step_duration_usize = U16_MAX;
			warn!(target: "engine", "step_duration is too high ({}), setting it to {}", step_duration_usize, U16_MAX);
		}
		AuthorityRoundParams {
			step_duration: step_duration_usize as u16,
			validators: new_validator_set(p.validators),
			start_step: p.start_step.map(Into::into),
			validate_score_transition: p.validate_score_transition.map_or(0, Into::into),
			validate_step_transition: p.validate_step_transition.map_or(0, Into::into),
			immediate_transitions: p.immediate_transitions.unwrap_or(false),
			block_reward: p.block_reward.map_or_else(Default::default, Into::into),
			block_reward_contract_transition: p.block_reward_contract_transition.map_or(0, Into::into),
			block_reward_contract: match (p.block_reward_contract_code, p.block_reward_contract_address) {
				(Some(code), _) => Some(BlockRewardContract::new_from_code(Arc::new(code.into()))),
				(_, Some(address)) => Some(BlockRewardContract::new_from_address(address.into())),
				(None, None) => None,
			},
			maximum_uncle_count_transition: p.maximum_uncle_count_transition.map_or(0, Into::into),
			maximum_uncle_count: p.maximum_uncle_count.map_or(0, Into::into),
			empty_steps_transition: p.empty_steps_transition.map_or(u64::max_value(), |n| ::std::cmp::max(n.into(), 1)),
			maximum_empty_steps: p.maximum_empty_steps.map_or(0, Into::into),
			strict_empty_steps_transition: p.strict_empty_steps_transition.map_or(0, Into::into),
		}
	}
}

// Helper for managing the step.
#[derive(Debug)]
struct Step {
	calibrate: bool, // whether calibration is enabled.
	inner: AtomicUsize,
	duration: u16,
}

impl Step {
	fn load(&self) -> u64 { self.inner.load(AtomicOrdering::SeqCst) as u64 }
	fn duration_remaining(&self) -> Duration {
		let now = unix_now();
		let expected_seconds = self.load()
			.checked_add(1)
			.and_then(|ctr| ctr.checked_mul(self.duration as u64))
			.map(Duration::from_secs);

		match expected_seconds {
			Some(step_end) if step_end > now => step_end - now,
			Some(_) => Duration::from_secs(0),
			None => {
				let ctr = self.load();
				error!(target: "engine", "Step counter is too high: {}, aborting", ctr);
				panic!("step counter is too high: {}", ctr)
			},
		}

	}

	fn increment(&self) {
		use std::usize;
		// fetch_add won't panic on overflow but will rather wrap
		// around, leading to zero as the step counter, which might
		// lead to unexpected situations, so it's better to shut down.
		if self.inner.fetch_add(1, AtomicOrdering::SeqCst) == usize::MAX {
			error!(target: "engine", "Step counter is too high: {}, aborting", usize::MAX);
			panic!("step counter is too high: {}", usize::MAX);
		}

	}

	fn calibrate(&self) {
		if self.calibrate {
			let new_step = unix_now().as_secs() / (self.duration as u64);
			self.inner.store(new_step as usize, AtomicOrdering::SeqCst);
		}
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
			let d = self.duration as u64;
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
	fn blank() -> Self {
		EpochManager {
			epoch_transition_hash: H256::default(),
			epoch_transition_number: 0,
			finality_checker: RollingFinality::blank(Vec::new()),
			force: true,
		}
	}

	// zoom to epoch for given header. returns true if succeeded, false otherwise.
	fn zoom_to(&mut self, client: &EngineClient, machine: &EthereumMachine, validators: &ValidatorSet, header: &Header) -> bool {
		let last_was_parent = self.finality_checker.subchain_head() == Some(*header.parent_hash());

		// early exit for current target == chain head, but only if the epochs are
		// the same.
		if last_was_parent && !self.force {
			return true;
		}

		self.force = false;
		debug!(target: "engine", "Zooming to epoch for block {}", header.hash());

		// epoch_transition_for can be an expensive call, but in the absence of
		// forks it will only need to be called for the block directly after
		// epoch transition, in which case it will be O(1) and require a single
		// DB lookup.
		let last_transition = match client.epoch_transition_for(*header.parent_hash()) {
			Some(t) => t,
			None => {
				// this really should never happen unless the block passed
				// hasn't got a parent in the database.
				debug!(target: "engine", "No genesis transition found.");
				return false;
			}
		};

		// extract other epoch set if it's not the same as the last.
		if last_transition.block_hash != self.epoch_transition_hash {
			let (signal_number, set_proof, _) = destructure_proofs(&last_transition.proof)
				.expect("proof produced by this engine; therefore it is valid; qed");

			trace!(target: "engine", "extracting epoch set for epoch ({}, {}) signalled at #{}",
				last_transition.block_number, last_transition.block_hash, signal_number);

			let first = signal_number == 0;
			let epoch_set = validators.epoch_set(
				first,
				machine,
				signal_number, // use signal number so multi-set first calculation is correct.
				set_proof,
			)
				.ok()
				.map(|(list, _)| list.into_inner())
				.expect("proof produced by this engine; therefore it is valid; qed");

			self.finality_checker = RollingFinality::blank(epoch_set);
		}

		self.epoch_transition_hash = last_transition.block_hash;
		self.epoch_transition_number = last_transition.block_number;

		true
	}

	// note new epoch hash. this will force the next block to re-load
	// the epoch set
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

	fn verify(&self, validators: &ValidatorSet) -> Result<bool, Error> {
		let message = keccak(empty_step_rlp(self.step, &self.parent_hash));
		let correct_proposer = step_proposer(validators, &self.parent_hash, self.step);

		ethkey::verify_address(&correct_proposer, &self.signature.into(), &message)
			.map_err(|e| e.into())
	}

	fn author(&self) -> Result<Address, Error> {
		let message = keccak(empty_step_rlp(self.step, &self.parent_hash));
		let public = ethkey::recover(&self.signature.into(), &message)?;
		Ok(ethkey::public_to_address(&public))
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
	client: Arc<RwLock<Option<Weak<EngineClient>>>>,
	signer: RwLock<Option<Box<EngineSigner>>>,
	validators: Box<ValidatorSet>,
	validate_score_transition: u64,
	validate_step_transition: u64,
	empty_steps: Mutex<BTreeSet<EmptyStep>>,
	epoch_manager: Mutex<EpochManager>,
	immediate_transitions: bool,
	block_reward: U256,
	block_reward_contract_transition: u64,
	block_reward_contract: Option<BlockRewardContract>,
	maximum_uncle_count_transition: u64,
	maximum_uncle_count: usize,
	empty_steps_transition: u64,
	strict_empty_steps_transition: u64,
	maximum_empty_steps: usize,
	machine: EthereumMachine,
}

// header-chain validator.
struct EpochVerifier {
	step: Arc<PermissionedStep>,
	subchain_validators: SimpleList,
	empty_steps_transition: u64,
}

impl super::EpochVerifier<EthereumMachine> for EpochVerifier {
	fn verify_light(&self, header: &Header) -> Result<(), Error> {
		// Validate the timestamp
		verify_timestamp(&self.step.inner, header_step(header, self.empty_steps_transition)?)?;
		// always check the seal since it's fast.
		// nothing heavier to do.
		verify_external(header, &self.subchain_validators, self.empty_steps_transition)
	}

	fn check_finality_proof(&self, proof: &[u8]) -> Option<Vec<H256>> {
		let mut finality_checker = RollingFinality::blank(self.subchain_validators.clone().into_inner());
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

				let newly_finalized = finality_checker.push_hash(parent_header.hash(), signers).ok()?;
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
			let mut message = header.bare_hash().to_vec();
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
		panic!("was either checked with verify_block_basic or is genesis; has {} fields; qed (Make sure the spec
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

fn step_proposer(validators: &ValidatorSet, bh: &H256, step: u64) -> Address {
	let proposer = validators.get(bh, step as usize);
	trace!(target: "engine", "Fetched proposer for step {}: {}", step, proposer);
	proposer
}

fn is_step_proposer(validators: &ValidatorSet, bh: &H256, step: u64, address: &Address) -> bool {
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
			let oob = oob.map(|n| SystemTime::now() + Duration::from_secs(n));
			Err(BlockError::TemporarilyInvalid(oob).into())
		},
		Ok(_) => Ok(()),
	}
}

fn verify_external(header: &Header, validators: &ValidatorSet, empty_steps_transition: u64) -> Result<(), Error> {
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
		!ethkey::verify_address(&correct_proposer, &proposer_signature, &header_seal_hash)?
	};

	if is_invalid_proposer {
		trace!(target: "engine", "verify_block_external: bad proposer for step: {}", header_step);
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
		self.as_secs()*1_000 + (self.subsec_nanos()/1_000_000) as u64
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
	pub fn new(our_params: AuthorityRoundParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
		if our_params.step_duration == 0 {
			error!(target: "engine", "Authority Round step duration can't be zero, aborting");
			panic!("authority_round: step duration can't be zero")
		}
		let should_timeout = our_params.start_step.is_none();
		let initial_step = our_params.start_step.unwrap_or_else(|| (unix_now().as_secs() / (our_params.step_duration as u64)));
		let engine = Arc::new(
			AuthorityRound {
				transition_service: IoService::<()>::start()?,
				step: Arc::new(PermissionedStep {
					inner: Step {
						inner: AtomicUsize::new(initial_step as usize),
						calibrate: our_params.start_step.is_none(),
						duration: our_params.step_duration,
					},
					can_propose: AtomicBool::new(true),
				}),
				client: Arc::new(RwLock::new(None)),
				signer: RwLock::new(None),
				validators: our_params.validators,
				validate_score_transition: our_params.validate_score_transition,
				validate_step_transition: our_params.validate_step_transition,
				empty_steps: Default::default(),
				epoch_manager: Mutex::new(EpochManager::blank()),
				immediate_transitions: our_params.immediate_transitions,
				block_reward: our_params.block_reward,
				block_reward_contract_transition: our_params.block_reward_contract_transition,
				block_reward_contract: our_params.block_reward_contract,
				maximum_uncle_count_transition: our_params.maximum_uncle_count_transition,
				maximum_uncle_count: our_params.maximum_uncle_count,
				empty_steps_transition: our_params.empty_steps_transition,
				maximum_empty_steps: our_params.maximum_empty_steps,
				strict_empty_steps_transition: our_params.strict_empty_steps_transition,
				machine: machine,
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
	fn epoch_set<'a>(&'a self, header: &Header) -> Result<(CowLike<ValidatorSet, SimpleList>, BlockNumber), Error> {
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

			if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, header) {
				debug!(target: "engine", "Unable to zoom to epoch.");
				return Err(EngineError::RequiresClient.into())
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

	fn report_skipped(&self, header: &Header, current_step: u64, parent_step: u64, validators: &ValidatorSet, set_number: u64) {
		// we're building on top of the genesis block so don't report any skipped steps
		if header.number() == 1 {
			return;
		}

		if let (true, Some(me)) = (current_step > parent_step + 1, self.signer.read().as_ref().map(|s| s.address())) {
			debug!(target: "engine", "Author {} built block with step gap. current step: {}, parent step: {}",
				   header.author(), current_step, parent_step);
			let mut reported = HashSet::new();
			for step in parent_step + 1..current_step {
				let skipped_primary = step_proposer(validators, header.parent_hash(), step);
				// Do not report this signer.
				if skipped_primary != me {
					// Stop reporting once validators start repeating.
					if !reported.insert(skipped_primary) { break; }
					self.validators.report_benign(&skipped_primary, set_number, header.number());
 				}
 			}
		}
	}

	// Returns the hashes of all ancestor blocks that are finalized by the given `chain_head`.
	fn build_finality(&self, chain_head: &Header, ancestry: &mut Iterator<Item=Header>) -> Vec<H256> {
		if self.immediate_transitions { return Vec::new() }

		let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
			Some(client) => client,
			None => {
				warn!(target: "engine", "Unable to apply ancestry actions: missing client ref.");
				return Vec::new();
			}
		};

		let mut epoch_manager = self.epoch_manager.lock();
		if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, chain_head) {
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
					let res = (header.hash(), signers);
					trace!(target: "finality", "Ancestry iteration: yielding {:?}", res);

					parent_empty_steps_signers = empty_step_signers;

					Some(res)

				} else {
					warn!(target: "finality", "Failed to get empty step signatures from block {}", header.hash());
					None
				}
			})
				.while_some()
				.take_while(|&(h, _)| h != epoch_transition_hash);

			if let Err(e) = epoch_manager.finality_checker.build_ancestry_subchain(ancestry_iter) {
				debug!(target: "engine", "inconsistent validator set within epoch: {:?}", e);
				return Vec::new();
			}
		}

		let finalized = epoch_manager.finality_checker.push_hash(chain_head.hash(), vec![*chain_head.author()]);
		finalized.unwrap_or_default()
	}
}

fn unix_now() -> Duration {
	UNIX_EPOCH.elapsed().expect("Valid time has to be set in your system.")
}

struct TransitionHandler {
	step: Arc<PermissionedStep>,
	client: Arc<RwLock<Option<Weak<EngineClient>>>>,
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
						c.update_sealing();
					}
				}
			}

			let next_run_at = AsMillis::as_millis(&self.step.inner.duration_remaining()) >> 2;
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, Duration::from_millis(next_run_at))
				.unwrap_or_else(|e| warn!(target: "engine", "Failed to restart consensus step timer: {}.", e))
		}
	}
}

impl Engine<EthereumMachine> for AuthorityRound {
	fn name(&self) -> &str { "AuthorityRound" }

	fn machine(&self) -> &EthereumMachine { &self.machine }

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
				c.update_sealing();
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
	}

	fn seals_internally(&self) -> Option<bool> {
		// TODO: accept a `&Call` here so we can query the validator set.
		Some(self.signer.read().is_some())
	}

	fn handle_message(&self, rlp: &[u8]) -> Result<(), EngineError> {
		fn fmt_err<T: ::std::fmt::Debug>(x: T) -> EngineError {
			EngineError::MalformedMessage(format!("{:?}", x))
		}

		let rlp = Rlp::new(rlp);
		let empty_step: EmptyStep = rlp.as_val().map_err(fmt_err)?;;

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

		let header = block.header();
		let parent_step = header_step(parent, self.empty_steps_transition)
			.expect("Header has been verified; qed");

		let step = self.step.inner.load();

		// filter messages from old and future steps and different parents
		let empty_steps = if header.number() >= self.empty_steps_transition {
			self.empty_steps(parent_step.into(), step.into(), *header.parent_hash())
		} else {
			Vec::new()
		};

		let expected_diff = calculate_score(parent_step, step.into(), empty_steps.len().into());

		if header.difficulty() != &expected_diff {
			debug!(target: "engine", "Aborting seal generation. The step or empty_steps have changed in the meantime. {:?} != {:?}",
				   header.difficulty(), expected_diff);
			return Seal::None;
		}

		if parent_step > step.into() {
			warn!(target: "engine", "Aborting seal generation for invalid step: {} > {}", parent_step, step);
			return Seal::None;
		}

		let (validators, set_number) = match self.epoch_set(header) {
			Err(err) => {
				warn!(target: "engine", "Unable to generate seal: {}", err);
				return Seal::None;
			},
			Ok(ok) => ok,
		};

		if is_step_proposer(&*validators, header.parent_hash(), step, header.author()) {
			// this is guarded against by `can_propose` unless the block was signed
			// on the same step (implies same key) and on a different node.
			if parent_step == step {
				warn!("Attempted to seal block on the same step as parent. Is this authority sealing with more than one node?");
				return Seal::None;
			}

			// if there are no transactions to include in the block, we don't seal and instead broadcast a signed
			// `EmptyStep(step, parent_hash)` message. If we exceed the maximum amount of `empty_step` rounds we proceed
			// with the seal.
			if header.number() >= self.empty_steps_transition &&
				block.transactions().is_empty() &&
				empty_steps.len() < self.maximum_empty_steps {

				if self.step.can_propose.compare_and_swap(true, false, AtomicOrdering::SeqCst) {
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
						self.report_skipped(header, step, parent_step, &*validators, set_number);
					}

					let mut fields = vec![
						encode(&step),
						encode(&(&H520::from(signature) as &[u8])),
					];

					if let Some(empty_steps_rlp) = empty_steps_rlp {
						fields.push(empty_steps_rlp);
					}

					return Seal::Regular(fields);
				}
			} else {
				warn!(target: "engine", "generate_seal: FAIL: Accounts secret key unavailable.");
			}
		} else {
			trace!(target: "engine", "generate_seal: {} not a proposer for step {}.",
				header.author(), step);
		}

		Seal::None
	}

	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
		Ok(())
	}

	fn on_new_block(
		&self,
		block: &mut ExecutedBlock,
		epoch_begin: bool,
		_ancestry: &mut Iterator<Item=ExtendedHeader>,
	) -> Result<(), Error> {
		// with immediate transitions, we don't use the epoch mechanism anyway.
		// the genesis is always considered an epoch, but we ignore it intentionally.
		if self.immediate_transitions || !epoch_begin { return Ok(()) }

		// genesis is never a new block, but might as well check.
		let header = block.header().clone();
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
	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		let mut beneficiaries = Vec::new();
		if block.header().number() >= self.empty_steps_transition {
			let empty_steps = if block.header().seal().is_empty() {
				// this is a new block, calculate rewards based on the empty steps messages we have accumulated
				let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
					Some(client) => client,
					None => {
						debug!(target: "engine", "Unable to close block: missing client ref.");
						return Err(EngineError::RequiresClient.into())
					},
				};

				let parent = client.block_header(::client::BlockId::Hash(*block.header().parent_hash()))
					.expect("hash is from parent; parent header must exist; qed")
					.decode()?;

				let parent_step = header_step(&parent, self.empty_steps_transition)?;
				let current_step = self.step.inner.load();
				self.empty_steps(parent_step.into(), current_step.into(), parent.hash())
			} else {
				// we're verifying a block, extract empty steps from the seal
				header_empty_steps(block.header())?
			};

			for empty_step in empty_steps {
				let author = empty_step.author()?;
				beneficiaries.push((author, RewardKind::EmptyStep));
			}
		}

		let author = *block.header().author();
		beneficiaries.push((author, RewardKind::Author));

		let rewards: Vec<_> = match self.block_reward_contract {
			Some(ref c) if block.header().number() >= self.block_reward_contract_transition => {
				let mut call = super::default_system_or_code_call(&self.machine, block);

				let rewards = c.reward(&beneficiaries, &mut call)?;
				rewards.into_iter().map(|(author, amount)| (author, RewardKind::External, amount)).collect()
			},
			_ => {
				beneficiaries.into_iter().map(|(author, reward_kind)| (author, reward_kind, self.block_reward)).collect()
			},
		};

		block_reward::apply_block_rewards(&rewards, block, &self.machine)
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
				// - Only authorities can report and it's expected that they'll be up-to-date and
				//   importing, therefore the parent header will most likely be available
				// - Even if you are an authority that is syncing the chain, the contract will most
				//   likely ignore old reports
				// - This specific check is only relevant if you're importing (since it checks
				//   against wall clock)
				if let Ok((_, set_number)) = self.epoch_set(header) {
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
			trace!(target: "engine", "Multiple blocks proposed for step {}.", parent_step);

			self.validators.report_malicious(header.author(), set_number, header.number(), Default::default());
			Err(EngineError::DoubleVote(*header.author()))?;
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
			Err(Error(ErrorKind::Engine(EngineError::NotProposer(_)), _)) => {
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

	fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData)
		-> super::EpochChange<EthereumMachine>
	{
		if self.immediate_transitions { return super::EpochChange::No }

		let first = header.number() == 0;
		self.validators.signals_epoch_end(first, header, aux)
	}

	fn is_epoch_end_light(
		&self,
		chain_head: &Header,
		chain: &super::Headers<Header>,
		transition_store: &super::PendingTransitionStore,
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
			if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, chain_head) {
				return None;
			}

			epoch_manager.epoch_transition_hash
		};

		let mut hash = *chain_head.parent_hash();

		let mut ancestry = itertools::repeat_call(move || {
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
		chain: &super::Headers<Header>,
		transition_store: &super::PendingTransitionStore,
	) -> Option<Vec<u8>> {
		// epochs only matter if we want to support light clients.
		if self.immediate_transitions { return None }

		let first = chain_head.number() == 0;

		// apply immediate transitions.
		if let Some(change) = self.validators.is_epoch_end(first, chain_head) {
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
				let mut finality_proof: Vec<_> = itertools::repeat_call(move || {
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

	fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {
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
				});

				match finalize {
					Some(finalize) => ConstructedVerifier::Unconfirmed(verifier, finality_proof, finalize),
					None => ConstructedVerifier::Trusted(verifier),
				}
			}
			Err(e) => ConstructedVerifier::Err(e),
		}
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		*self.client.write() = Some(client.clone());
		self.validators.register_client(client);
	}

	fn set_signer(&self, signer: Box<EngineSigner>) {
		*self.signer.write() = Some(signer);
	}

	fn sign(&self, hash: H256) -> Result<Signature, Error> {
		Ok(self.signer.read()
			.as_ref()
			.ok_or(ethkey::Error::InvalidAddress)?
			.sign(hash)?
		)
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		if self.immediate_transitions {
			None
		} else {
			Some(Box::new(::snapshot::PoaSnapshot))
		}
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
		super::total_difficulty_fork_choice(new, current)
	}

	fn ancestry_actions(&self, header: &Header, ancestry: &mut Iterator<Item=ExtendedHeader>) -> Vec<AncestryAction> {
		let finalized = self.build_finality(
			header,
			&mut ancestry.take_while(|e| !e.is_finalized).map(|e| e.header),
		);

		if !finalized.is_empty() {
			debug!(target: "finality", "Finalizing blocks: {:?}", finalized);
		}

		finalized.into_iter().map(AncestryAction::MarkFinalized).collect()
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
	use hash::keccak;
	use accounts::AccountProvider;
	use ethereum_types::{Address, H520, H256, U256};
	use ethkey::Signature;
	use types::header::Header;
	use rlp::encode;
	use block::*;
	use test_helpers::{
		generate_dummy_client_with_spec, get_temp_state_db,
		TestNotify
	};
	use spec::Spec;
	use types::transaction::{Action, Transaction};
	use engines::{Seal, Engine, EngineError, EthEngine};
	use engines::validator_set::{TestSet, SimpleList};
	use error::{Error, ErrorKind};
	use super::{AuthorityRoundParams, AuthorityRound, EmptyStep, SealedEmptyStep, calculate_score};

	fn aura<F>(f: F) -> Arc<AuthorityRound> where
		F: FnOnce(&mut AuthorityRoundParams),
	{
		let mut params = AuthorityRoundParams {
			step_duration: 1,
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
			block_reward_contract_transition: 0,
			block_reward_contract: Default::default(),
			strict_empty_steps_transition: 0,
		};

		// mutate aura params
		f(&mut params);

		// create engine
		let mut c_params = ::spec::CommonParams::default();
		c_params.gas_limit_bound_divisor = 5.into();
		let machine = ::machine::EthereumMachine::regular(c_params, Default::default());
		AuthorityRound::new(params, machine).unwrap()
	}

	#[test]
	fn has_valid_metadata() {
		let engine = Spec::new_test_round().engine;
		assert!(!engine.name().is_empty());
	}

	#[test]
	fn can_return_schedule() {
		let engine = Spec::new_test_round().engine;
		let schedule = engine.schedule(10000000);

		assert!(schedule.stack_limit > 0);
	}

	#[test]
	fn can_do_signature_verification_fail() {
		let engine = Spec::new_test_round().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![encode(&H520::default())]);

		let verify_result = engine.verify_block_external(&header);
		assert!(verify_result.is_err());
	}

	#[test]
	fn generates_seal_and_does_not_double_propose() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
		let addr2 = tap.insert_account(keccak("2").into(), &"2".into()).unwrap();

		let spec = Spec::new_test_round();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b1 = b1.close_and_lock().unwrap();
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes, addr2, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b2 = b2.close_and_lock().unwrap();

		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		if let Seal::Regular(seal) = engine.generate_seal(b1.block(), &genesis_header) {
			assert!(b1.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(b1.block(), &genesis_header) == Seal::None);
		}

		engine.set_signer(Box::new((tap, addr2, "2".into())));
		if let Seal::Regular(seal) = engine.generate_seal(b2.block(), &genesis_header) {
			assert!(b2.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(b2.block(), &genesis_header) == Seal::None);
		}
	}

	#[test]
	fn checks_difficulty_in_generate_seal() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
		let addr2 = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

		let spec = Spec::new_test_round();
		let engine = &*spec.engine;

		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b1 = b1.close_and_lock().unwrap();
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes, addr2, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b2 = b2.close_and_lock().unwrap();

		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		match engine.generate_seal(b1.block(), &genesis_header) {
			Seal::None | Seal::Proposal(_) => panic!("wrong seal"),
			Seal::Regular(_) => {
				engine.step();

				engine.set_signer(Box::new((tap.clone(), addr2, "0".into())));
				match engine.generate_seal(b2.block(), &genesis_header) {
					Seal::Regular(_) | Seal::Proposal(_) => panic!("sealed despite wrong difficulty"),
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

		let engine = Spec::new_test_round().engine;

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

		let engine = Spec::new_test_round().engine;

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

		let engine = Spec::new_test_round().engine;

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
		let last_benign = Arc::new(AtomicUsize::new(0));
		let aura = aura(|p| {
			p.validators = Box::new(TestSet::new(Default::default(), last_benign.clone()));
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

		aura.set_signer(Box::new((Arc::new(AccountProvider::transient_provider()), Default::default(), "".into())));

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
	fn test_uncles_transition() {
		let aura = aura(|params| {
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
            inner: AtomicUsize::new(::std::usize::MAX),
            duration: 1,
        };
        step.increment();
	}

	#[test]
	#[should_panic(expected="counter is too high")]
	fn test_counter_duration_remaining_too_high() {
		use super::Step;
		let step = Step {
			calibrate: false,
			inner: AtomicUsize::new(::std::usize::MAX),
			duration: 1,
		};
		step.duration_remaining();
	}

	#[test]
	#[should_panic(expected="authority_round: step duration can't be zero")]
	fn test_step_duration_zero() {
		aura(|params| {
			params.step_duration = 0;
		});
	}

	fn setup_empty_steps() -> (Spec, Arc<AccountProvider>, Vec<Address>) {
		let spec = Spec::new_test_round_empty_steps();
		let tap = Arc::new(AccountProvider::transient_provider());

		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
		let addr2 = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

		let accounts = vec![addr1, addr2];

		(spec, tap, accounts)
	}

	fn empty_step(engine: &EthEngine, step: u64, parent_hash: &H256) -> EmptyStep {
		let empty_step_rlp = super::empty_step_rlp(step, parent_hash);
		let signature = engine.sign(keccak(&empty_step_rlp)).unwrap().into();
		let parent_hash = parent_hash.clone();
		EmptyStep { step, signature, parent_hash }
	}

	fn sealed_empty_step(engine: &EthEngine, step: u64, parent_hash: &H256) -> SealedEmptyStep {
		let empty_step_rlp = super::empty_step_rlp(step, parent_hash);
		let signature = engine.sign(keccak(&empty_step_rlp)).unwrap().into();
		SealedEmptyStep { signature, step }
	}

	fn set_empty_steps_seal(header: &mut Header, step: u64, block_signature: &ethkey::Signature, empty_steps: &[SealedEmptyStep]) {
		header.set_seal(vec![
			encode(&(step as usize)),
			encode(&(&**block_signature as &[u8])),
			::rlp::encode_list(&empty_steps),
		]);
	}

	fn assert_insufficient_proof<T: ::std::fmt::Debug>(result: Result<T, Error>, contains: &str) {
		match result {
			Err(Error(ErrorKind::Engine(EngineError::InsufficientProof(ref s)), _)) =>{
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

		let client = generate_dummy_client_with_spec(Spec::new_test_round_empty_steps);
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client) as _);

		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));

		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// the block is empty so we don't seal and instead broadcast an empty step message
		assert_eq!(engine.generate_seal(b1.block(), &genesis_header), Seal::None);

		// spec starts with step 2
		let empty_step_rlp = encode(&empty_step(engine, 2, &genesis_header.hash()));

		// we've received the message
		assert!(notify.messages.read().contains(&empty_step_rlp));
		let len = notify.messages.read().len();

		// make sure that we don't generate empty step for the second time
		assert_eq!(engine.generate_seal(b1.block(), &genesis_header), Seal::None);
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

		let client = generate_dummy_client_with_spec(Spec::new_test_round_empty_steps);
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client) as _);

		// step 2
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// since the block is empty it isn't sealed and we generate empty steps
		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		assert_eq!(engine.generate_seal(b1.block(), &genesis_header), Seal::None);
		engine.step();

		// step 3
		let mut b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes.clone(), addr2, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
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
		engine.set_signer(Box::new((tap.clone(), addr2, "0".into())));
		if let Seal::Regular(seal) = engine.generate_seal(b2.block(), &genesis_header) {
			engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
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

		let client = generate_dummy_client_with_spec(Spec::new_test_round_empty_steps);
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client) as _);

		// step 2
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// since the block is empty it isn't sealed and we generate empty steps
		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		assert_eq!(engine.generate_seal(b1.block(), &genesis_header), Seal::None);
		engine.step();

		// step 3
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes.clone(), addr2, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b2 = b2.close_and_lock().unwrap();
		engine.set_signer(Box::new((tap.clone(), addr2, "0".into())));
		assert_eq!(engine.generate_seal(b2.block(), &genesis_header), Seal::None);
		engine.step();

		// step 4
		// the spec sets the maximum_empty_steps to 2 so we will now seal an empty block and include the empty step messages
		let b3 = OpenBlock::new(engine, Default::default(), false, db3, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b3 = b3.close_and_lock().unwrap();

		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		if let Seal::Regular(seal) = engine.generate_seal(b3.block(), &genesis_header) {
			let empty_step2 = sealed_empty_step(engine, 2, &genesis_header.hash());
			engine.set_signer(Box::new((tap.clone(), addr2, "0".into())));
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

		let client = generate_dummy_client_with_spec(Spec::new_test_round_empty_steps);
		engine.register_client(Arc::downgrade(&client) as _);

		// step 2
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// since the block is empty it isn't sealed and we generate empty steps
		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		assert_eq!(engine.generate_seal(b1.block(), &genesis_header), Seal::None);
		engine.step();

		// step 3
		// the signer of the accumulated empty step message should be rewarded
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let addr1_balance = b2.block().state().balance(&addr1).unwrap();

		// after closing the block `addr1` should be reward twice, one for the included empty step message and another for block creation
		let b2 = b2.close_and_lock().unwrap();

		// the spec sets the block reward to 10
		assert_eq!(b2.block().state().balance(&addr1).unwrap(), addr1_balance + (10 * 2))
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
		let empty_steps = vec![SealedEmptyStep { signature: 0.into(), step: 2 }];
		set_empty_steps_seal(&mut header, 2, &signature, &empty_steps);

		assert_insufficient_proof(
			engine.verify_block_family(&header, &parent_header),
			"invalid step"
		);

		// empty step with invalid signature
		let empty_steps = vec![SealedEmptyStep { signature: 0.into(), step: 1 }];
		set_empty_steps_seal(&mut header, 2, &signature, &empty_steps);

		assert_insufficient_proof(
			engine.verify_block_family(&header, &parent_header),
			"invalid empty step proof"
		);

		// empty step with valid signature from incorrect proposer for step
		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		let empty_steps = vec![sealed_empty_step(engine, 1, &parent_header.hash())];
		set_empty_steps_seal(&mut header, 2, &signature, &empty_steps);

		assert_insufficient_proof(
			engine.verify_block_family(&header, &parent_header),
			"invalid empty step proof"
		);

		// valid empty steps
		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		let empty_step2 = sealed_empty_step(engine, 2, &parent_header.hash());
		engine.set_signer(Box::new((tap.clone(), addr2, "0".into())));
		let empty_step3 = sealed_empty_step(engine, 3, &parent_header.hash());

		let empty_steps = vec![empty_step2, empty_step3];
		header.set_difficulty(calculate_score(0, 4, 2));
		let signature = tap.sign(addr1, Some("1".into()), header.bare_hash()).unwrap();
		set_empty_steps_seal(&mut header, 4, &signature, &empty_steps);

		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
	}

	#[test]
	fn block_reward_contract() {
		let spec = Spec::new_test_round_block_reward_contract();
		let tap = Arc::new(AccountProvider::transient_provider());

		let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();

		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();

		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let client = generate_dummy_client_with_spec(Spec::new_test_round_block_reward_contract);
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
			&mut Vec::new().into_iter(),
		).unwrap();
		let b1 = b1.close_and_lock().unwrap();

		// since the block is empty it isn't sealed and we generate empty steps
		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
		assert_eq!(engine.generate_seal(b1.block(), &genesis_header), Seal::None);
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
			&mut Vec::new().into_iter(),
		).unwrap();
		let addr1_balance = b2.block().state().balance(&addr1).unwrap();

		// after closing the block `addr1` should be reward twice, one for the included empty step
		// message and another for block creation
		let b2 = b2.close_and_lock().unwrap();

		// the contract rewards (1000 + kind) for each benefactor/reward kind
		assert_eq!(
			b2.block().state().balance(&addr1).unwrap(),
			addr1_balance + (1000 + 0) + (1000 + 2),
		)
	}

	#[test]
	fn extra_info_from_seal() {
		let (spec, tap, accounts) = setup_empty_steps();
		let engine = &*spec.engine;

		let addr1 = accounts[0];
		engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));

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
		let engine = aura(|p| {
			p.step_duration = 4;
			p.empty_steps_transition = 0;
			p.maximum_empty_steps = 0;
		});

		let parent_hash: H256 = 1.into();
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
		let engine = aura(|p| {
			p.validators = Box::new(SimpleList::new(accounts.clone()));
			p.step_duration = 4;
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
		engine.set_signer(Box::new((tap.clone(), accounts[1], "0".into())));
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
		let engine = aura(|p| {
			p.validators = Box::new(SimpleList::new(accounts.clone()));
			p.step_duration = 4;
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
		engine.set_signer(Box::new((tap.clone(), accounts[1], "0".into())));
		let es1 = sealed_empty_step(&*engine, 1, &parent.hash());
		engine.set_signer(Box::new((tap.clone(), accounts[0], "1".into())));
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
}
