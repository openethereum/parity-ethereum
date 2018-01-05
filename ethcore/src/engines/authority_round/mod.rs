// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! A blockchain engine that supports a non-instant BFT proof-of-authority.

use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Weak, Arc};
use std::time::{UNIX_EPOCH, Duration};
use std::collections::{BTreeMap, HashSet};

use account_provider::AccountProvider;
use block::*;
use client::EngineClient;
use engines::{Engine, Seal, EngineError, ConstructedVerifier};
use error::{Error, BlockError};
use ethjson;
use machine::{AuxiliaryData, Call, EthereumMachine};
use header::{Header, BlockNumber};

use super::signer::EngineSigner;
use super::validator_set::{ValidatorSet, SimpleList, new_validator_set};

use self::finality::RollingFinality;

use ethkey::{verify_address, Signature};
use io::{IoContext, IoHandler, TimerToken, IoService};
use itertools::{self, Itertools};
use rlp::{UntrustedRlp, encode};
use bigint::prelude::{U256, U128};
use bigint::hash::{H256, H520};
use semantic_version::SemanticVersion;
use parking_lot::{Mutex, RwLock};
use unexpected::{Mismatch, OutOfBounds};
use util::*;
use bytes::Bytes;

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
	/// Number of accepted uncles transition block.
	pub maximum_uncle_count_transition: u64,
	/// Number of accepted uncles.
	pub maximum_uncle_count: usize,
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
			maximum_uncle_count_transition: p.maximum_uncle_count_transition.map_or(0, Into::into),
			maximum_uncle_count: p.maximum_uncle_count.map_or(0, Into::into),
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
	fn load(&self) -> usize { self.inner.load(AtomicOrdering::SeqCst) }
	fn duration_remaining(&self) -> Duration {
		let now = unix_now();
		let expected_seconds = (self.load() as u64)
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

	fn check_future(&self, given: usize) -> Result<(), Option<OutOfBounds<u64>>> {
		const VALID_STEP_DRIFT: usize = 1;
		const REJECTED_STEP_DRIFT: usize = 4;

		// Give one step slack if step is lagging, double vote is still not possible.
		if given <= self.load() + VALID_STEP_DRIFT {
			return Ok(());
		}

		// Make absolutely sure that the given step is incorrect.
		self.calibrate();
		let current = self.load();

		// reject blocks too far in the future
		if given > current + REJECTED_STEP_DRIFT {
			Err(None)
		// wait a bit for blocks in near future
		} else if given > current + VALID_STEP_DRIFT {
			let d = self.duration as u64;
			Err(Some(OutOfBounds {
				min: None,
				max: Some(d * (current + VALID_STEP_DRIFT) as u64),
				found: d * given as u64,
			}))
		} else {
			Ok(())
		}
	}
}

// Chain scoring: total weight is sqrt(U256::max_value())*height - step
fn calculate_score(parent_step: U256, current_step: U256) -> U256 {
	U256::from(U128::max_value()) + parent_step - current_step
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
		let last_was_parent = self.finality_checker.subchain_head() == Some(header.parent_hash().clone());

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

/// Engine using `AuthorityRound` proof-of-authority BFT consensus.
pub struct AuthorityRound {
	transition_service: IoService<()>,
	step: Arc<Step>,
	can_propose: AtomicBool,
	client: RwLock<Option<Weak<EngineClient>>>,
	signer: RwLock<EngineSigner>,
	validators: Box<ValidatorSet>,
	validate_score_transition: u64,
	validate_step_transition: u64,
	epoch_manager: Mutex<EpochManager>,
	immediate_transitions: bool,
	block_reward: U256,
	maximum_uncle_count_transition: u64,
	maximum_uncle_count: usize,
	machine: EthereumMachine,
}

// header-chain validator.
struct EpochVerifier {
	step: Arc<Step>,
	subchain_validators: SimpleList,
}

impl super::EpochVerifier<EthereumMachine> for EpochVerifier {
	fn verify_light(&self, header: &Header) -> Result<(), Error> {
		// always check the seal since it's fast.
		// nothing heavier to do.
		verify_external(header, &self.subchain_validators, &*self.step, |_| {})
	}

	fn check_finality_proof(&self, proof: &[u8]) -> Option<Vec<H256>> {
		macro_rules! otry {
			($e: expr) => {
				match $e {
					Some(x) => x,
					None => return None,
				}
			}
		}

		let mut finality_checker = RollingFinality::blank(self.subchain_validators.clone().into_inner());
		let mut finalized = Vec::new();

		let headers: Vec<Header> = otry!(UntrustedRlp::new(proof).as_list().ok());

		for header in &headers {
			// ensure all headers have correct number of seal fields so we can `verify_external`
			// without panic.
			//
			// `verify_external` checks that signature is correct and author == signer.
			if header.seal().len() != 2 { return None }
			otry!(verify_external(header, &self.subchain_validators, &*self.step, |_| {}).ok());

			let newly_finalized = otry!(finality_checker.push_hash(header.hash(), header.author().clone()).ok());
			finalized.extend(newly_finalized);
		}

		if finalized.is_empty() { None } else { Some(finalized) }
	}
}

// Report misbehavior
#[derive(Debug)]
#[allow(dead_code)]
enum Report {
	// Malicious behavior
	Malicious(Address, BlockNumber, Bytes),
	// benign misbehavior
	Benign(Address, BlockNumber),
}

fn header_step(header: &Header) -> Result<usize, ::rlp::DecoderError> {
	UntrustedRlp::new(&header.seal().get(0).expect("was either checked with verify_block_basic or is genesis; has 2 fields; qed (Make sure the spec file has a correct genesis seal)")).as_val()
}

fn header_signature(header: &Header) -> Result<Signature, ::rlp::DecoderError> {
	UntrustedRlp::new(&header.seal().get(1).expect("was checked with verify_block_basic; has 2 fields; qed")).as_val::<H520>().map(Into::into)
}

fn step_proposer(validators: &ValidatorSet, bh: &H256, step: usize) -> Address {
	let proposer = validators.get(bh, step);
	trace!(target: "engine", "Fetched proposer for step {}: {}", step, proposer);
	proposer
}

fn is_step_proposer(validators: &ValidatorSet, bh: &H256, step: usize, address: &Address) -> bool {
	step_proposer(validators, bh, step) == *address
}

fn verify_external<F: Fn(Report)>(header: &Header, validators: &ValidatorSet, step: &Step, report: F)
	-> Result<(), Error>
{
	let header_step = header_step(header)?;

	match step.check_future(header_step) {
		Err(None) => {
			trace!(target: "engine", "verify_block_external: block from the future");
			report(Report::Benign(*header.author(), header.number()));
			return Err(BlockError::InvalidSeal.into())
		},
		Err(Some(oob)) => {
			trace!(target: "engine", "verify_block_external: block too early");
			return Err(BlockError::TemporarilyInvalid(oob).into())
		},
		Ok(_) => {
			let proposer_signature = header_signature(header)?;
			let correct_proposer = validators.get(header.parent_hash(), header_step);
			let is_invalid_proposer = *header.author() != correct_proposer ||
				!verify_address(&correct_proposer, &proposer_signature, &header.bare_hash())?;

			if is_invalid_proposer {
				trace!(target: "engine", "verify_block_external: bad proposer for step: {}", header_step);
				Err(EngineError::NotProposer(Mismatch { expected: correct_proposer, found: header.author().clone() }))?
			} else {
				Ok(())
			}
		}
	}
}

fn combine_proofs(signal_number: BlockNumber, set_proof: &[u8], finality_proof: &[u8]) -> Vec<u8> {
	let mut stream = ::rlp::RlpStream::new_list(3);
	stream.append(&signal_number).append(&set_proof).append(&finality_proof);
	stream.out()
}

fn destructure_proofs(combined: &[u8]) -> Result<(BlockNumber, &[u8], &[u8]), Error> {
	let rlp = UntrustedRlp::new(combined);
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

impl AuthorityRound {
	/// Create a new instance of AuthorityRound engine.
	pub fn new(our_params: AuthorityRoundParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
		if our_params.step_duration == 0 {
			error!(target: "engine", "Authority Round step duration can't be zero, aborting");
			panic!("authority_round: step duration can't be zero")
		}
		let should_timeout = our_params.start_step.is_none();
		let initial_step = our_params.start_step.unwrap_or_else(|| (unix_now().as_secs() / (our_params.step_duration as u64))) as usize;
		let engine = Arc::new(
			AuthorityRound {
				transition_service: IoService::<()>::start()?,
				step: Arc::new(Step {
					inner: AtomicUsize::new(initial_step),
					calibrate: our_params.start_step.is_none(),
					duration: our_params.step_duration,
				}),
				can_propose: AtomicBool::new(true),
				client: RwLock::new(None),
				signer: Default::default(),
				validators: our_params.validators,
				validate_score_transition: our_params.validate_score_transition,
				validate_step_transition: our_params.validate_step_transition,
				epoch_manager: Mutex::new(EpochManager::blank()),
				immediate_transitions: our_params.immediate_transitions,
				block_reward: our_params.block_reward,
				maximum_uncle_count_transition: our_params.maximum_uncle_count_transition,
				maximum_uncle_count: our_params.maximum_uncle_count,
				machine: machine,
			});

		// Do not initialize timeouts for tests.
		if should_timeout {
			let handler = TransitionHandler { engine: Arc::downgrade(&engine) };
			engine.transition_service.register_handler(Arc::new(handler))?;
		}
		Ok(engine)
	}
}

fn unix_now() -> Duration {
	UNIX_EPOCH.elapsed().expect("Valid time has to be set in your system.")
}

struct TransitionHandler {
	engine: Weak<AuthorityRound>,
}

const ENGINE_TIMEOUT_TOKEN: TimerToken = 23;

impl IoHandler<()> for TransitionHandler {
	fn initialize(&self, io: &IoContext<()>) {
		if let Some(engine) = self.engine.upgrade() {
			let remaining = engine.step.duration_remaining();
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, remaining.as_millis())
				.unwrap_or_else(|e| warn!(target: "engine", "Failed to start consensus step timer: {}.", e))
		}
	}

	fn timeout(&self, io: &IoContext<()>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			if let Some(engine) = self.engine.upgrade() {
				engine.step();
				let remaining = engine.step.duration_remaining();
				io.register_timer_once(ENGINE_TIMEOUT_TOKEN, remaining.as_millis())
					.unwrap_or_else(|e| warn!(target: "engine", "Failed to restart consensus step timer: {}.", e))
			}
		}
	}
}

impl Engine<EthereumMachine> for AuthorityRound {
	fn name(&self) -> &str { "AuthorityRound" }

	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }

	fn machine(&self) -> &EthereumMachine { &self.machine }

	/// Two fields - consensus step and the corresponding proposer signature.
	fn seal_fields(&self) -> usize { 2 }

	fn step(&self) {
		self.step.increment();
		self.can_propose.store(true, AtomicOrdering::SeqCst);
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				c.update_sealing();
			}
		}
	}

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, header: &Header) -> BTreeMap<String, String> {
		map![
			"step".into() => header_step(header).as_ref().map(ToString::to_string).unwrap_or("".into()),
			"signature".into() => header_signature(header).as_ref().map(ToString::to_string).unwrap_or("".into())
		]
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
		let parent_step = header_step(parent).expect("Header has been verified; qed");
		let score = calculate_score(parent_step.into(), self.step.load().into());
		header.set_difficulty(score);
	}

	fn seals_internally(&self) -> Option<bool> {
		// TODO: accept a `&Call` here so we can query the validator set.
		Some(self.signer.read().is_some())
	}

	/// Attempt to seal the block internally.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which case
	/// `Seal::None` will be returned.
	fn generate_seal(&self, block: &ExecutedBlock, parent: &Header) -> Seal {
		// first check to avoid generating signature most of the time
		// (but there's still a race to the `compare_and_swap`)
		if !self.can_propose.load(AtomicOrdering::SeqCst) { return Seal::None; }

		let header = block.header();
		let parent_step: U256 = header_step(parent)
			.expect("Header has been verified; qed").into();

		let step = self.step.load();

		let expected_diff = calculate_score(parent_step, step.into());

		if header.difficulty() != &expected_diff {
			return Seal::None;
		}

		// fetch correct validator set for current epoch, taking into account
		// finality of previous transitions.
		let active_set;

		let validators = if self.immediate_transitions {
			&*self.validators
		} else {
			let mut epoch_manager = self.epoch_manager.lock();
			let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
				Some(client) => client,
				None => {
					warn!(target: "engine", "Unable to generate seal: missing client ref.");
					return Seal::None;
				}
			};

			if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, header) {
				debug!(target: "engine", "Unable to zoom to epoch.");
				return Seal::None;
			}

			active_set = epoch_manager.validators().clone();
			&active_set as &_
		};

		if is_step_proposer(validators, header.parent_hash(), step, header.author()) {
			// this is guarded against by `can_propose` unless the block was signed
			// on the same step (implies same key) and on a different node.
			if parent_step == step.into() {
				warn!("Attempted to seal block on the same step as parent. Is this authority sealing with more than one node?");
				return Seal::None;
			}

			if let Ok(signature) = self.sign(header.bare_hash()) {
				trace!(target: "engine", "generate_seal: Issuing a block for step {}.", step);

				// only issue the seal if we were the first to reach the compare_and_swap.
				if self.can_propose.compare_and_swap(true, false, AtomicOrdering::SeqCst) {
					return Seal::Regular(vec![encode(&step).into_vec(), encode(&(&H520::from(signature) as &[u8])).into_vec()]);
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
	) -> Result<(), Error> {
		// with immediate transitions, we don't use the epoch mechanism anyway.
		// the genesis is always considered an epoch, but we ignore it intentionally.
		if self.immediate_transitions || !epoch_begin { return Ok(()) }

		// genesis is never a new block, but might as well check.
		let header = block.fields().header.clone();
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
		// TODO: move to "machine::WithBalances" trait.
		::engines::common::bestow_block_reward(block, self.block_reward)
	}

	/// Check the number of seal fields.
	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
		if header.number() >= self.validate_score_transition && *header.difficulty() >= U256::from(U128::max_value()) {
			Err(From::from(BlockError::DifficultyOutOfBounds(
				OutOfBounds { min: None, max: Some(U256::from(U128::max_value())), found: *header.difficulty() }
			)))
		} else {
			Ok(())
		}
	}

	/// Do the step and gas limit validation.
	fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
		let step = header_step(header)?;

		let parent_step = header_step(parent)?;

		// Ensure header is from the step after parent.
		if step == parent_step
			|| (header.number() >= self.validate_step_transition && step <= parent_step) {
			trace!(target: "engine", "Multiple blocks proposed for step {}.", parent_step);

			self.validators.report_malicious(header.author(), header.number(), header.number(), Default::default());
			Err(EngineError::DoubleVote(header.author().clone()))?;
		}

		// Report skipped primaries.
		if let (true, Some(me)) = (step > parent_step + 1, self.signer.read().address()) {
			debug!(target: "engine", "Author {} built block with step gap. current step: {}, parent step: {}",
				header.author(), step, parent_step);
			let mut reported = HashSet::new();
			for s in parent_step + 1..step {
				let skipped_primary = step_proposer(&*self.validators, &parent.hash(), s);
				// Do not report this signer.
				if skipped_primary != me {
					self.validators.report_benign(&skipped_primary, header.number(), header.number());
				}
				// Stop reporting once validators start repeating.
				if !reported.insert(skipped_primary) { break; }
			}
		}

		Ok(())
	}

	// Check the validators.
	fn verify_block_external(&self, header: &Header) -> Result<(), Error> {
		// fetch correct validator set for current epoch, taking into account
		// finality of previous transitions.
		let active_set;

		let (validators, set_number) = if self.immediate_transitions {
			(&*self.validators, header.number())
		} else {
			// get correct validator set for epoch.
			let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
				Some(client) => client,
				None => {
					debug!(target: "engine", "Unable to verify sig: missing client ref.");
					return Err(EngineError::RequiresClient.into())
				}
			};

			let mut epoch_manager = self.epoch_manager.lock();
			if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, header) {
				debug!(target: "engine", "Unable to zoom to epoch.");
				return Err(EngineError::RequiresClient.into())
			}

			active_set = epoch_manager.validators().clone();
			(&active_set as &_, epoch_manager.epoch_transition_number)
		};

		// always report with "self.validators" so that the report actually gets
		// to the contract.
		let report = |report| match report {
			Report::Benign(address, block_number) =>
				self.validators.report_benign(&address, set_number, block_number),
			Report::Malicious(address, block_number, proof) =>
				self.validators.report_malicious(&address, set_number, block_number, proof),
		};

		// verify signature against fixed list, but reports should go to the
		// contract itself.
		verify_external(header, validators, &*self.step, report)
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

	fn is_epoch_end(
		&self,
		chain_head: &Header,
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

		let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
			Some(client) => client,
			None => {
				warn!(target: "engine", "Unable to check for epoch end: missing client ref.");
				return None;
			}
		};

		// find most recently finalized blocks, then check transition store for pending transitions.
		let mut epoch_manager = self.epoch_manager.lock();
		if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, chain_head) {
			return None;
		}

		if epoch_manager.finality_checker.subchain_head() != Some(*chain_head.parent_hash()) {
			// build new finality checker from ancestry of chain head,
			// not including chain head itself yet.
			trace!(target: "finality", "Building finality up to parent of {} ({})",
				chain_head.hash(), chain_head.parent_hash());

			let mut hash = chain_head.parent_hash().clone();
			let epoch_transition_hash = epoch_manager.epoch_transition_hash;

			// walk the chain within current epoch backwards.
			// author == ec_recover(sig) known since
			// the blocks are in the DB.
			let ancestry_iter = itertools::repeat_call(move || {
				chain(hash).and_then(|header| {
					if header.number() == 0 { return None }

					let res = (hash, header.author().clone());
					trace!(target: "finality", "Ancestry iteration: yielding {:?}", res);

					hash = header.parent_hash().clone();
					Some(res)
				})
			})
				.while_some()
				.take_while(|&(h, _)| h != epoch_transition_hash);

			if let Err(_) = epoch_manager.finality_checker.build_ancestry_subchain(ancestry_iter) {
				debug!(target: "engine", "inconsistent validator set within epoch");
				return None;
			}
		}

		{
			if let Ok(finalized) = epoch_manager.finality_checker.push_hash(chain_head.hash(), *chain_head.author()) {
				let mut finalized = finalized.into_iter();
				while let Some(finalized_hash) = finalized.next() {
					if let Some(pending) = transition_store(finalized_hash) {
						let finality_proof = ::std::iter::once(finalized_hash)
							.chain(finalized)
							.chain(epoch_manager.finality_checker.unfinalized_hashes())
							.map(|h| if h == chain_head.hash() {
								// chain closure only stores ancestry, but the chain head is also
								// unfinalized.
								chain_head.clone()
							} else {
								chain(h).expect("these headers fetched before when constructing finality checker; qed")
							})
							.collect::<Vec<Header>>();

						// this gives us the block number for `hash`, assuming it's ancestry.
						let signal_number = chain_head.number()
							- finality_proof.len() as BlockNumber
							+ 1;
						let finality_proof = ::rlp::encode_list(&finality_proof);
						epoch_manager.note_new_epoch();

						info!(target: "engine", "Applying validator set change signalled at block {}", signal_number);

						// We turn off can_propose here because upon validator set change there can
						// be two valid proposers for a single step: one from the old set and
						// one from the new.
						//
						// This way, upon encountering an epoch change, the proposer from the
						// new set will be forced to wait until the next step to avoid sealing a
						// block that breaks the invariant that the parent's step < the block's step.
						self.can_propose.store(false, AtomicOrdering::SeqCst);
						return Some(combine_proofs(signal_number, &pending.proof, &*finality_proof));
					}
				}
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

	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: String) {
		self.signer.write().set(ap, address, password);
	}

	fn sign(&self, hash: H256) -> Result<Signature, Error> {
		self.signer.read().sign(hash).map_err(Into::into)
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		if self.immediate_transitions {
			None
		} else {
			Some(Box::new(::snapshot::PoaSnapshot))
		}
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
	use hash::keccak;
	use bigint::prelude::U256;
	use bigint::hash::H520;
	use header::Header;
	use rlp::encode;
	use block::*;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use spec::Spec;
	use engines::{Seal, Engine};
	use engines::validator_set::TestSet;
	use super::{AuthorityRoundParams, AuthorityRound};

	#[test]
	fn has_valid_metadata() {
		let engine = Spec::new_test_round().engine;
		assert!(!engine.name().is_empty());
		assert!(engine.version().major >= 1);
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
		header.set_seal(vec![encode(&H520::default()).into_vec()]);

		let verify_result = engine.verify_block_external(&header);
		assert!(verify_result.is_err());
	}

	#[test]
	fn generates_seal_and_does_not_double_propose() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr1 = tap.insert_account(keccak("1").into(), "1").unwrap();
		let addr2 = tap.insert_account(keccak("2").into(), "2").unwrap();

		let spec = Spec::new_test_round();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b1 = b1.close_and_lock();
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes, addr2, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b2 = b2.close_and_lock();

		engine.set_signer(tap.clone(), addr1, "1".into());
		if let Seal::Regular(seal) = engine.generate_seal(b1.block(), &genesis_header) {
			assert!(b1.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(b1.block(), &genesis_header) == Seal::None);
		}

		engine.set_signer(tap, addr2, "2".into());
		if let Seal::Regular(seal) = engine.generate_seal(b2.block(), &genesis_header) {
			assert!(b2.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(b2.block(), &genesis_header) == Seal::None);
		}
	}

	#[test]
	fn checks_difficulty_in_generate_seal() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr1 = tap.insert_account(keccak("1").into(), "1").unwrap();
		let addr2 = tap.insert_account(keccak("0").into(), "0").unwrap();

		let spec = Spec::new_test_round();
		let engine = &*spec.engine;

		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);

		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b1 = b1.close_and_lock();
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes, addr2, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b2 = b2.close_and_lock();

		engine.set_signer(tap.clone(), addr1, "1".into());
		match engine.generate_seal(b1.block(), &genesis_header) {
			Seal::None | Seal::Proposal(_) => panic!("wrong seal"),
			Seal::Regular(_) => {
				engine.step();

				engine.set_signer(tap.clone(), addr2, "0".into());
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
		let addr = tap.insert_account(keccak("0").into(), "0").unwrap();
		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&0usize).into_vec()]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_author(addr);

		let engine = Spec::new_test_round().engine;

		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		// Two validators.
		// Spec starts with step 2.
		header.set_seal(vec![encode(&2usize).into_vec(), encode(&(&*signature as &[u8])).into_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		assert!(engine.verify_block_external(&header).is_err());
		header.set_seal(vec![encode(&1usize).into_vec(), encode(&(&*signature as &[u8])).into_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		assert!(engine.verify_block_external(&header).is_ok());
	}

	#[test]
	fn rejects_future_block() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account(keccak("0").into(), "0").unwrap();

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&0usize).into_vec()]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_author(addr);

		let engine = Spec::new_test_round().engine;

		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		// Two validators.
		// Spec starts with step 2.
		header.set_seal(vec![encode(&1usize).into_vec(), encode(&(&*signature as &[u8])).into_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		assert!(engine.verify_block_external(&header).is_ok());
		header.set_seal(vec![encode(&5usize).into_vec(), encode(&(&*signature as &[u8])).into_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		assert!(engine.verify_block_external(&header).is_err());
	}

	#[test]
	fn rejects_step_backwards() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account(keccak("0").into(), "0").unwrap();

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&4usize).into_vec()]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_author(addr);

		let engine = Spec::new_test_round().engine;

		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		// Two validators.
		// Spec starts with step 2.
		header.set_seal(vec![encode(&5usize).into_vec(), encode(&(&*signature as &[u8])).into_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header).is_ok());
		header.set_seal(vec![encode(&3usize).into_vec(), encode(&(&*signature as &[u8])).into_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header).is_err());
	}

	#[test]
	fn reports_skipped() {
		let last_benign = Arc::new(AtomicUsize::new(0));
		let params = AuthorityRoundParams {
			step_duration: 1,
			start_step: Some(1),
			validators: Box::new(TestSet::new(Default::default(), last_benign.clone())),
			validate_score_transition: 0,
			validate_step_transition: 0,
			immediate_transitions: true,
			maximum_uncle_count_transition: 0,
			maximum_uncle_count: 0,
			block_reward: Default::default(),
		};

		let aura = {
			let mut c_params = ::spec::CommonParams::default();
			c_params.gas_limit_bound_divisor = 5.into();
			let machine = ::machine::EthereumMachine::regular(c_params, Default::default());
			AuthorityRound::new(params, machine).unwrap()
		};

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&1usize).into_vec()]);
		parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit("222222".parse::<U256>().unwrap());
		header.set_seal(vec![encode(&3usize).into_vec()]);

		// Do not report when signer not present.
		assert!(aura.verify_block_family(&header, &parent_header).is_ok());
		assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 0);

		aura.set_signer(Arc::new(AccountProvider::transient_provider()), Default::default(), Default::default());

		assert!(aura.verify_block_family(&header, &parent_header).is_ok());
		assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 1);
	}

	#[test]
	fn test_uncles_transition() {
		let last_benign = Arc::new(AtomicUsize::new(0));
		let params = AuthorityRoundParams {
			step_duration: 1,
			start_step: Some(1),
			validators: Box::new(TestSet::new(Default::default(), last_benign.clone())),
			validate_score_transition: 0,
			validate_step_transition: 0,
			immediate_transitions: true,
			maximum_uncle_count_transition: 1,
			maximum_uncle_count: 0,
			block_reward: Default::default(),
		};

		let aura = {
			let mut c_params = ::spec::CommonParams::default();
			c_params.gas_limit_bound_divisor = 5.into();
			let machine = ::machine::EthereumMachine::regular(c_params, Default::default());
			AuthorityRound::new(params, machine).unwrap()
		};

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
		let last_benign = Arc::new(AtomicUsize::new(0));
		let params = AuthorityRoundParams {
			step_duration: 0,
			start_step: Some(1),
			validators: Box::new(TestSet::new(Default::default(), last_benign.clone())),
			validate_score_transition: 0,
			validate_step_transition: 0,
			immediate_transitions: true,
			maximum_uncle_count_transition: 0,
			maximum_uncle_count: 0,
			block_reward: Default::default(),
		};

		let mut c_params = ::spec::CommonParams::default();
		c_params.gas_limit_bound_divisor = 5.into();
		let machine = ::machine::EthereumMachine::regular(c_params, Default::default());
		AuthorityRound::new(params, machine).unwrap();
	}
}
