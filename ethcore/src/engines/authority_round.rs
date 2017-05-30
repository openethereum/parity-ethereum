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
use std::sync::Weak;
use std::time::{UNIX_EPOCH, Duration};
use util::*;
use ethkey::{verify_address, Signature};
use rlp::{UntrustedRlp, encode};
use account_provider::AccountProvider;
use block::*;
use spec::CommonParams;
use engines::{Call, Engine, Seal, EngineError};
use header::{Header, BlockNumber};
use error::{Error, TransactionError, BlockError};
use evm::Schedule;
use ethjson;
use io::{IoContext, IoHandler, TimerToken, IoService};
use builtin::Builtin;
use transaction::UnverifiedTransaction;
use client::{Client, EngineClient};
use state::CleanupMode;
use super::signer::EngineSigner;
use super::validator_set::{ValidatorSet, SimpleList, new_validator_set};

/// `AuthorityRound` params.
pub struct AuthorityRoundParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// Time to wait before next block or authority switching.
	pub step_duration: Duration,
	/// Block reward.
	pub block_reward: U256,
	/// Namereg contract address.
	pub registrar: Address,
	/// Starting step,
	pub start_step: Option<u64>,
	/// Valid validators.
	pub validators: Box<ValidatorSet>,
	/// Chain score validation transition block.
	pub validate_score_transition: u64,
	/// Number of first block where EIP-155 rules are validated.
	pub eip155_transition: u64,
	/// Monotonic step validation transition block.
	pub validate_step_transition: u64,
}

impl From<ethjson::spec::AuthorityRoundParams> for AuthorityRoundParams {
	fn from(p: ethjson::spec::AuthorityRoundParams) -> Self {
		AuthorityRoundParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			step_duration: Duration::from_secs(p.step_duration.into()),
			validators: new_validator_set(p.validators),
			block_reward: p.block_reward.map_or_else(U256::zero, Into::into),
			registrar: p.registrar.map_or_else(Address::new, Into::into),
			start_step: p.start_step.map(Into::into),
			validate_score_transition: p.validate_score_transition.map_or(0, Into::into),
			eip155_transition: p.eip155_transition.map_or(0, Into::into),
			validate_step_transition: p.validate_step_transition.map_or(0, Into::into),
		}
	}
}

// Helper for managing the step.
#[derive(Debug)]
struct Step {
	calibrate: bool, // whether calibration is enabled.
	inner: AtomicUsize,
	duration: Duration,
}

impl Step {
	fn load(&self) -> usize { self.inner.load(AtomicOrdering::SeqCst) }
	fn duration_remaining(&self) -> Duration {
		let now = unix_now();
		let step_end = self.duration * (self.load() as u32 + 1);
		if step_end > now {
			step_end - now
		} else {
			Duration::from_secs(0)
		}
	}
	fn increment(&self) {
		self.inner.fetch_add(1, AtomicOrdering::SeqCst);
	}
	fn calibrate(&self) {
		if self.calibrate {
			let new_step = unix_now().as_secs() / self.duration.as_secs();
			self.inner.store(new_step as usize, AtomicOrdering::SeqCst);
		}
	}
	fn is_future(&self, given: usize) -> bool {
		if given > self.load() + 1 {
			// Make absolutely sure that the given step is correct.
			self.calibrate();
			given > self.load() + 1
		} else {
			false
		}
	}
}

/// Engine using `AuthorityRound` proof-of-authority BFT consensus.
pub struct AuthorityRound {
	params: CommonParams,
	gas_limit_bound_divisor: U256,
	block_reward: U256,
	registrar: Address,
	builtins: BTreeMap<Address, Builtin>,
	transition_service: IoService<()>,
	step: Arc<Step>,
	proposed: AtomicBool,
	client: RwLock<Option<Weak<EngineClient>>>,
	signer: EngineSigner,
	validators: Box<ValidatorSet>,
	validate_score_transition: u64,
	eip155_transition: u64,
	validate_step_transition: u64,
}

// header-chain validator.
struct EpochVerifier {
	epoch_number: u64,
	step: Arc<Step>,
	subchain_validators: SimpleList,
}

impl super::EpochVerifier for EpochVerifier {
	fn epoch_number(&self) -> u64 { self.epoch_number.clone() }
	fn verify_light(&self, header: &Header) -> Result<(), Error> {
		// always check the seal since it's fast.
		// nothing heavier to do.
		verify_external(header, &self.subchain_validators, &*self.step)
	}
}

fn header_step(header: &Header) -> Result<usize, ::rlp::DecoderError> {
	UntrustedRlp::new(&header.seal().get(0).expect("was either checked with verify_block_basic or is genesis; has 2 fields; qed (Make sure the spec file has a correct genesis seal)")).as_val()
}

fn header_signature(header: &Header) -> Result<Signature, ::rlp::DecoderError> {
	UntrustedRlp::new(&header.seal().get(1).expect("was checked with verify_block_basic; has 2 fields; qed")).as_val::<H520>().map(Into::into)
}

fn verify_external(header: &Header, validators: &ValidatorSet, step: &Step) -> Result<(), Error> {
	let header_step = header_step(header)?;

	// Give one step slack if step is lagging, double vote is still not possible.
	if step.is_future(header_step) {
		trace!(target: "engine", "verify_block_unordered: block from the future");
		validators.report_benign(header.author(), header.number());
		Err(BlockError::InvalidSeal)?
	} else {
		let proposer_signature = header_signature(header)?;
		let correct_proposer = validators.get(header.parent_hash(), header_step);
		if !verify_address(&correct_proposer, &proposer_signature, &header.bare_hash())? {
			trace!(target: "engine", "verify_block_unordered: bad proposer for step: {}", header_step);
			Err(EngineError::NotProposer(Mismatch { expected: correct_proposer, found: header.author().clone() }))?
		} else {
			Ok(())
		}
	}
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
	pub fn new(params: CommonParams, our_params: AuthorityRoundParams, builtins: BTreeMap<Address, Builtin>) -> Result<Arc<Self>, Error> {
		let should_timeout = our_params.start_step.is_none();
		let initial_step = our_params.start_step.unwrap_or_else(|| (unix_now().as_secs() / our_params.step_duration.as_secs())) as usize;
		let engine = Arc::new(
			AuthorityRound {
				params: params,
				gas_limit_bound_divisor: our_params.gas_limit_bound_divisor,
				block_reward: our_params.block_reward,
				registrar: our_params.registrar,
				builtins: builtins,
				transition_service: IoService::<()>::start()?,
				step: Arc::new(Step {
					inner: AtomicUsize::new(initial_step),
					calibrate: our_params.start_step.is_none(),
					duration: our_params.step_duration,
				}),
				proposed: AtomicBool::new(false),
				client: RwLock::new(None),
				signer: Default::default(),
				validators: our_params.validators,
				validate_score_transition: our_params.validate_score_transition,
				eip155_transition: our_params.eip155_transition,
				validate_step_transition: our_params.validate_step_transition,
			});
		// Do not initialize timeouts for tests.
		if should_timeout {
			let handler = TransitionHandler { engine: Arc::downgrade(&engine) };
			engine.transition_service.register_handler(Arc::new(handler))?;
		}
		Ok(engine)
	}

	fn step_proposer(&self, bh: &H256, step: usize) -> Address {
		self.validators.get(bh, step)
	}

	fn is_step_proposer(&self, bh: &H256, step: usize, address: &Address) -> bool {
		self.step_proposer(bh, step) == *address
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

impl Engine for AuthorityRound {
	fn name(&self) -> &str { "AuthorityRound" }

	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }

	/// Two fields - consensus step and the corresponding proposer signature.
	fn seal_fields(&self) -> usize { 2 }

	fn params(&self) -> &CommonParams { &self.params }

	fn additional_params(&self) -> HashMap<String, String> { hash_map!["registrar".to_owned() => self.registrar.hex()] }

	fn builtins(&self) -> &BTreeMap<Address, Builtin> { &self.builtins }

	fn step(&self) {
		self.step.increment();
		self.proposed.store(false, AtomicOrdering::SeqCst);
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

	fn schedule(&self, block_number: BlockNumber) -> Schedule {
		let eip86 = block_number >= self.params.eip86_transition;
		Schedule::new_post_eip150(usize::max_value(), true, true, true, eip86)
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256, _gas_ceil_target: U256) {
		// Chain scoring: total weight is sqrt(U256::max_value())*height - step
		let new_difficulty = U256::from(U128::max_value()) + header_step(parent).expect("Header has been verified; qed").into() - self.step.load().into();
		header.set_difficulty(new_difficulty);
		header.set_gas_limit({
			let gas_limit = parent.gas_limit().clone();
			let bound_divisor = self.gas_limit_bound_divisor;
			if gas_limit < gas_floor_target {
				min(gas_floor_target, gas_limit + gas_limit / bound_divisor - 1.into())
			} else {
				max(gas_floor_target, gas_limit - gas_limit / bound_divisor + 1.into())
			}
		});
	}

	fn seals_internally(&self) -> Option<bool> {
		Some(self.signer.address() != Address::default())
	}

	/// Attempt to seal the block internally.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which `false` will
	/// be returned.
	fn generate_seal(&self, block: &ExecutedBlock) -> Seal {
		if self.proposed.load(AtomicOrdering::SeqCst) { return Seal::None; }
		let header = block.header();
		let step = self.step.load();
		if self.is_step_proposer(header.parent_hash(), step, header.author()) {
			if let Ok(signature) = self.signer.sign(header.bare_hash()) {
				trace!(target: "engine", "generate_seal: Issuing a block for step {}.", step);
				self.proposed.store(true, AtomicOrdering::SeqCst);
				return Seal::Regular(vec![encode(&step).to_vec(), encode(&(&H520::from(signature) as &[u8])).to_vec()]);
			} else {
				warn!(target: "engine", "generate_seal: FAIL: Accounts secret key unavailable.");
			}
		} else {
			trace!(target: "engine", "generate_seal: Not a proposer for step {}.", step);
		}
		Seal::None
	}

	/// Apply the block reward on finalisation of the block.
	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		let fields = block.fields_mut();
		// Bestow block reward
		let res = fields.state.add_balance(fields.header.author(), &self.block_reward, CleanupMode::NoEmpty)
			.map_err(::error::Error::from)
			.and_then(|_| fields.state.commit());
		// Commit state so that we can actually figure out the state root.
		if let Err(ref e) = res {
			warn!("Encountered error on closing block: {}", e);
		}
		res
	}

	/// Check the number of seal fields.
	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		if header.seal().len() != self.seal_fields() {
			trace!(target: "engine", "verify_block_basic: wrong number of seal fields");
			Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)))
		} else if header.number() >= self.validate_score_transition && *header.difficulty() >= U256::from(U128::max_value()) {
			Err(From::from(BlockError::DifficultyOutOfBounds(
				OutOfBounds { min: None, max: Some(U256::from(U128::max_value())), found: *header.difficulty() }
			)))
		} else {
			Ok(())
		}
	}

	fn verify_block_unordered(&self, _header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		Ok(())
	}

	/// Do the step and gas limit validation.
	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		let step = header_step(header)?;

		// Do not calculate difficulty for genesis blocks.
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		let parent_step = header_step(parent)?;
		// Ensure header is from the step after parent.
		if step == parent_step
			|| (header.number() >= self.validate_step_transition && step <= parent_step) {
			trace!(target: "engine", "Multiple blocks proposed for step {}.", parent_step);
			self.validators.report_malicious(header.author(), header.number(), Default::default());
			Err(EngineError::DoubleVote(header.author().clone()))?;
		}
		// Report skipped primaries.
		if step > parent_step + 1 {
			for s in parent_step + 1..step {
				let skipped_primary = self.step_proposer(&parent.hash(), s);
				trace!(target: "engine", "Author {} did not build his block on top of the intermediate designated primary {}.", header.author(), skipped_primary);
				self.validators.report_benign(&skipped_primary, header.number());
			}
		}

		let gas_limit_divisor = self.gas_limit_bound_divisor;
		let min_gas = parent.gas_limit().clone() - parent.gas_limit().clone() / gas_limit_divisor;
		let max_gas = parent.gas_limit().clone() + parent.gas_limit().clone() / gas_limit_divisor;
		if header.gas_limit() <= &min_gas || header.gas_limit() >= &max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit().clone() })));
		}
		Ok(())
	}

	// Check the validators.
	fn verify_block_external(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		verify_external(header, &*self.validators, &*self.step)
	}

	// the proofs we need just allow us to get the full validator set.
	fn epoch_proof(&self, header: &Header, caller: &Call) -> Result<Bytes, Error> {
		self.validators.epoch_proof(header, caller)
			.map_err(|e| EngineError::InsufficientProof(e).into())
	}

	fn is_epoch_end(&self, header: &Header, block: Option<&[u8]>, receipts: Option<&[::receipt::Receipt]>)
		-> super::EpochChange
	{
		self.validators.is_epoch_end(header, block, receipts)
	}

	fn epoch_verifier(&self, header: &Header, proof: &[u8]) -> Result<Box<super::EpochVerifier>, Error> {
		// extract a simple list from the proof.
		let (num, simple_list) = self.validators.epoch_set(header, proof)?;

		Ok(Box::new(EpochVerifier {
			epoch_number: num,
			step: self.step.clone(),
			subchain_validators: simple_list,
		}))
	}

	fn verify_transaction_basic(&self, t: &UnverifiedTransaction, header: &Header) -> result::Result<(), Error> {
		t.check_low_s()?;

		if let Some(n) = t.network_id() {
			if header.number() >= self.eip155_transition && n != self.params().chain_id {
				return Err(TransactionError::InvalidNetworkId.into());
			}
		}

		Ok(())
	}

	fn register_client(&self, client: Weak<Client>) {
		*self.client.write() = Some(client.clone());
		self.validators.register_contract(client);
	}

	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: String) {
		self.signer.set(ap, address, password);
	}

	fn sign(&self, hash: H256) -> Result<Signature, Error> {
		self.signer.sign(hash).map_err(Into::into)
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		Some(Box::new(::snapshot::PoaSnapshot))
	}
}

#[cfg(test)]
mod tests {
	use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
	use util::*;
	use header::Header;
	use error::{Error, BlockError};
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
	fn verification_fails_on_short_seal() {
		let engine = Spec::new_test_round().engine;
		let header: Header = Header::default();

		let verify_result = engine.verify_block_basic(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidSealArity(_))) => {},
			Err(_) => { panic!("should be block seal-arity mismatch error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_do_signature_verification_fail() {
		let engine = Spec::new_test_round().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![encode(&H520::default()).to_vec()]);

		let verify_result = engine.verify_block_external(&header, None);
		assert!(verify_result.is_err());
	}

	#[test]
	fn generates_seal_and_does_not_double_propose() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr1 = tap.insert_account("1".sha3().into(), "1").unwrap();
		let addr2 = tap.insert_account("2".sha3().into(), "2").unwrap();

		let spec = Spec::new_test_round();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db1 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let db2 = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b1 = b1.close_and_lock();
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes, addr2, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b2 = b2.close_and_lock();

		engine.set_signer(tap.clone(), addr1, "1".into());
		if let Seal::Regular(seal) = engine.generate_seal(b1.block()) {
			assert!(b1.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(b1.block()) == Seal::None);
		}

		engine.set_signer(tap, addr2, "2".into());
		if let Seal::Regular(seal) = engine.generate_seal(b2.block()) {
			assert!(b2.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(b2.block()) == Seal::None);
		}
	}

	#[test]
	fn proposer_switching() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("0".sha3().into(), "0").unwrap();
		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&0usize).to_vec()]);
		parent_header.set_gas_limit(U256::from_str("222222").unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit(U256::from_str("222222").unwrap());
		header.set_author(addr);

		let engine = Spec::new_test_round().engine;

		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		// Two validators.
		// Spec starts with step 2.
		header.set_seal(vec![encode(&2usize).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header, None).is_ok());
		assert!(engine.verify_block_external(&header, None).is_err());
		header.set_seal(vec![encode(&1usize).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header, None).is_ok());
		assert!(engine.verify_block_external(&header, None).is_ok());
	}

	#[test]
	fn rejects_future_block() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("0".sha3().into(), "0").unwrap();

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&0usize).to_vec()]);
		parent_header.set_gas_limit(U256::from_str("222222").unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit(U256::from_str("222222").unwrap());
		header.set_author(addr);

		let engine = Spec::new_test_round().engine;

		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		// Two validators.
		// Spec starts with step 2.
		header.set_seal(vec![encode(&1usize).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header, None).is_ok());
		assert!(engine.verify_block_external(&header, None).is_ok());
		header.set_seal(vec![encode(&5usize).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header, None).is_ok());
		assert!(engine.verify_block_external(&header, None).is_err());
	}

	#[test]
	fn rejects_step_backwards() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("0".sha3().into(), "0").unwrap();

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&4usize).to_vec()]);
		parent_header.set_gas_limit(U256::from_str("222222").unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit(U256::from_str("222222").unwrap());
		header.set_author(addr);

		let engine = Spec::new_test_round().engine;

		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		// Two validators.
		// Spec starts with step 2.
		header.set_seal(vec![encode(&5usize).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header, None).is_ok());
		header.set_seal(vec![encode(&3usize).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
		assert!(engine.verify_block_family(&header, &parent_header, None).is_err());
	}

	#[test]
	fn reports_skipped() {
		let last_benign = Arc::new(AtomicUsize::new(0));
		let params = AuthorityRoundParams {
			gas_limit_bound_divisor: U256::from_str("400").unwrap(),
			step_duration: Default::default(),
			block_reward: Default::default(),
			registrar: Default::default(),
			start_step: Some(1),
			validators: Box::new(TestSet::new(Default::default(), last_benign.clone())),
			validate_score_transition: 0,
			validate_step_transition: 0,
			eip155_transition: 0,
		};
		let aura = AuthorityRound::new(Default::default(), params, Default::default()).unwrap();

		let mut parent_header: Header = Header::default();
		parent_header.set_seal(vec![encode(&1usize).to_vec()]);
		parent_header.set_gas_limit(U256::from_str("222222").unwrap());
		let mut header: Header = Header::default();
		header.set_number(1);
		header.set_gas_limit(U256::from_str("222222").unwrap());
		header.set_seal(vec![encode(&3usize).to_vec()]);

		assert!(aura.verify_block_family(&header, &parent_header, None).is_ok());
		assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 1);
	}
}
