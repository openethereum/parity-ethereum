// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Tendermint BFT consensus engine with round robin proof-of-authority.

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Weak;
use common::*;
use rlp::{UntrustedRlp, View, encode};
use account_provider::AccountProvider;
use block::*;
use spec::CommonParams;
use engines::{Engine, EngineError, ProposeCollect};
use evm::Schedule;
use ethjson;
use io::{IoContext, IoHandler, TimerToken, IoService};
use time::get_time;

/// `Tendermint` params.
#[derive(Debug)]
pub struct TendermintParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// List of validators.
	pub validators: Vec<Address>,
	/// Number of validators.
	pub validator_n: usize,
	/// Timeout durations for different steps.
	timeouts: DefaultTimeouts,
}

impl Default for TendermintParams {
	fn default() -> Self {
		let validators = vec!["0x7d577a597b2742b498cb5cf0c26cdcd726d39e6e".into(), "0x82a978b3f5962a5b0957d9ee9eef472ee55b42f1".into()];
		let val_n = validators.len();
		let propose_timeout = 3000;
		TendermintParams {
			gas_limit_bound_divisor: 0x0400.into(),
			validators: validators,
			validator_n: val_n,
			timeouts: DefaultTimeouts {
				propose: propose_timeout,
				prevote: 3000,
				precommit: 3000,
				commit: 3000
			},
		}
	}
}

#[derive(Debug)]
enum Step {
	Propose,
	Prevote(ProposeCollect),
	/// Precommit step storing the precommit vote and accumulating seal.
	Precommit(ProposeCollect, Seal),
	/// Commit step storing a complete valid seal.
	Commit(Seal)
}

impl From<ethjson::spec::TendermintParams> for TendermintParams {
	fn from(p: ethjson::spec::TendermintParams) -> Self {
		let val: Vec<_> = p.validators.into_iter().map(Into::into).collect();
		let val_n = val.len();
		let propose_timeout = 3000;
		TendermintParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			validators: val,
			validator_n: val_n,
			timeouts: DefaultTimeouts {
				propose: propose_timeout,
				prevote: 3000,
				precommit: 3000,
				commit: 3000
			},
		}
	}
}

#[derive(Clone)]
struct StepMessage;

/// Engine using `Tendermint` consensus algorithm, suitable for EVM chain.
pub struct Tendermint {
	params: CommonParams,
	our_params: TendermintParams,
	builtins: BTreeMap<Address, Builtin>,
	timeout_service: IoService<StepMessage>,
	/// Consensus round.
	r: u64,
	/// Consensus step.
	s: RwLock<Step>,
	/// Current step timeout in ms.
	timeout: AtomicMs,
	/// Used to swith proposer.
	proposer_nonce: AtomicUsize,
}

struct TimerHandler {
	engine: Weak<Tendermint>,
}

impl Tendermint {
	/// Create a new instance of Tendermint engine
	pub fn new(params: CommonParams, our_params: TendermintParams, builtins: BTreeMap<Address, Builtin>) -> Arc<Self> {
		let engine = Arc::new(
			Tendermint {
				params: params,
				timeout: AtomicUsize::new(our_params.timeouts.propose),
				our_params: our_params,
				builtins: builtins,
				timeout_service: IoService::<StepMessage>::start().expect("Error creating engine timeout service"),
				r: 0,
				s: RwLock::new(Step::Propose),
				proposer_nonce: AtomicUsize::new(0)
			});
		let handler = TimerHandler { engine: Arc::downgrade(&engine) };
		engine.timeout_service.register_handler(Arc::new(handler)).expect("Error creating engine timeout service");
		engine
	}

	fn proposer(&self) -> Address {
		let ref p = self.our_params;
		p.validators.get(self.proposer_nonce.load(AtomicOrdering::Relaxed)%p.validator_n).unwrap().clone()
	}

	fn is_proposer(&self, address: &Address) -> bool {
		self.proposer() == *address
	}

	fn is_validator(&self, address: &Address) -> bool {
		self.our_params.validators.contains(address)
	}

	fn new_vote(&self, proposal: H256) -> ProposeCollect {
		ProposeCollect::new(proposal,
							self.our_params.validators.iter().cloned().collect(),
							self.threshold())
	}

	fn to_step(&self, step: Step) {
		let mut guard = self.s.try_write().unwrap();
		*guard = step;
	}

	fn to_propose(&self) {
		trace!(target: "tendermint", "step: entering propose");
		println!("step: entering propose");
		self.proposer_nonce.fetch_add(1, AtomicOrdering::Relaxed);
		self.to_step(Step::Propose);
	}

	fn propose_message(&self, message: UntrustedRlp) -> Result<Bytes, Error> {
		// Check if message is for correct step.
		match *self.s.try_read().unwrap() {
			Step::Propose => (),
			_ => try!(Err(EngineError::WrongStep)),
		}
		let proposal = try!(message.as_val());
		self.to_prevote(proposal);
		Ok(message.as_raw().to_vec())
	}

	fn to_prevote(&self, proposal: H256) {
		trace!(target: "tendermint", "step: entering prevote");
		println!("step: entering prevote");
		// Proceed to the prevote step.
		self.to_step(Step::Prevote(self.new_vote(proposal)));
	}

	fn prevote_message(&self, sender: Address, message: UntrustedRlp) -> Result<Bytes, Error> {
		// Check if message is for correct step.
		match *self.s.try_write().unwrap() {
			Step::Prevote(ref mut vote) => {
				// Vote if message is about the right block.
				if vote.hash == try!(message.as_val()) {
					vote.vote(sender);
					// Move to next step is prevote is won.
					if vote.is_won() {
						//self.our_params.timeouts.precommit
						self.to_precommit(vote.hash);
					}
					Ok(message.as_raw().to_vec())
				} else {
					try!(Err(EngineError::WrongVote))
				}
			},
			_ => try!(Err(EngineError::WrongStep)),
		}
	}

	fn to_precommit(&self, proposal: H256) {
		trace!(target: "tendermint", "step: entering precommit");
		println!("step: entering precommit");
		self.to_step(Step::Precommit(self.new_vote(proposal), Vec::new()));
	}

	fn precommit_message(&self, sender: Address, signature: H520, message: UntrustedRlp) -> Result<Bytes, Error> {
		// Check if message is for correct step.
		match *self.s.try_write().unwrap() {
			Step::Precommit(ref mut vote, ref mut seal) => {
				// Vote and accumulate seal if message is about the right block.
				if vote.hash == try!(message.as_val()) {
					if vote.vote(sender) { seal.push(encode(&signature).to_vec()); }
					// Commit if precommit is won.
					if vote.is_won() { self.to_commit(seal.clone()); }
					Ok(message.as_raw().to_vec())
				} else {
					try!(Err(EngineError::WrongVote))
				}
			},
			_ => try!(Err(EngineError::WrongStep)),
		}
	}

	fn to_commit(&self, seal: Seal) {
		trace!(target: "tendermint", "step: entering commit");
		println!("step: entering commit");
		self.to_step(Step::Commit(seal));
	}

	fn threshold(&self) -> usize {
		self.our_params.validator_n*2/3
	}

	fn next_timeout(&self) -> u64 {
		self.timeout.load(AtomicOrdering::Relaxed) as u64
	}
}

impl Engine for Tendermint {
	fn name(&self) -> &str { "Tendermint" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	/// Possibly signatures of all validators.
	fn seal_fields(&self) -> usize { self.our_params.validator_n }

	fn params(&self) -> &CommonParams { &self.params }
	fn builtins(&self) -> &BTreeMap<Address, Builtin> { &self.builtins }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> HashMap<String, String> { hash_map!["signature".to_owned() => "TODO".to_owned()] }

	fn schedule(&self, _env_info: &EnvInfo) -> Schedule {
		Schedule::new_homestead()
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256, _gas_ceil_target: U256) {
		header.set_difficulty(parent.difficulty().clone());
		header.set_gas_limit({
			let gas_limit = parent.gas_limit().clone();
			let bound_divisor = self.our_params.gas_limit_bound_divisor;
			if gas_limit < gas_floor_target {
				min(gas_floor_target, gas_limit + gas_limit / bound_divisor - 1.into())
			} else {
				max(gas_floor_target, gas_limit - gas_limit / bound_divisor + 1.into())
			}
		});
	}

	/// Apply the block reward on finalisation of the block.
	/// This assumes that all uncles are valid uncles (i.e. of at least one generation before the current).
	fn on_close_block(&self, _block: &mut ExecutedBlock) {}

	/// Attempt to seal the block internally using all available signatures.
	///
	/// None is returned if not enough signatures can be collected.
	fn generate_seal(&self, block: &ExecutedBlock, accounts: Option<&AccountProvider>) -> Option<Vec<Bytes>> {
		accounts.and_then(|ap| {
			let header = block.header();
			if header.author() == &self.proposer() {
				ap.sign(*header.author(), header.bare_hash())
					.ok()
					.and_then(|signature| Some(vec![encode(&(&*signature as &[u8])).to_vec()]))
			} else {
				None
			}
		})
	}

	fn handle_message(&self, sender: Address, signature: H520, message: UntrustedRlp) -> Result<Bytes, Error> {
		// Check if correct round.
		if self.r != try!(message.val_at(0)) { try!(Err(EngineError::WrongRound)) }
		// Handle according to step.
		match try!(message.val_at(1)) {
			0u8 if self.is_proposer(&sender) => self.propose_message(try!(message.at(2))),
			1 if self.is_validator(&sender) => self.prevote_message(sender, try!(message.at(2))),
			2 if self.is_validator(&sender) => self.precommit_message(sender, signature, try!(message.at(2))),
			_ => try!(Err(EngineError::UnknownStep)),
		}
	}

	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// check the seal fields.
		// TODO: pull this out into common code.
		if header.seal().len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)));
		}
		Ok(())
	}

	fn verify_block_unordered(&self, _header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// we should not calculate difficulty for genesis blocks
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		// Check difficulty is correct given the two timestamps.
		if header.difficulty() != parent.difficulty() {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: *parent.difficulty(), found: *header.difficulty() })))
		}
		let gas_limit_divisor = self.our_params.gas_limit_bound_divisor;
		let min_gas = parent.gas_limit().clone() - parent.gas_limit().clone() / gas_limit_divisor;
		let max_gas = parent.gas_limit().clone() + parent.gas_limit().clone() / gas_limit_divisor;
		if header.gas_limit() <= &min_gas || header.gas_limit() >= &max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit().clone() })));
		}
		Ok(())
	}

	fn verify_transaction_basic(&self, t: &SignedTransaction, _header: &Header) -> result::Result<(), Error> {
		try!(t.check_low_s());
		Ok(())
	}

	fn verify_transaction(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		t.sender().map(|_|()) // Perform EC recovery and cache sender
	}
}

/// Base timeout of each step in ms.
#[derive(Debug)]
struct DefaultTimeouts {
	propose: Ms,
	prevote: Ms,
	precommit: Ms,
	commit: Ms
}

type Ms = usize;
type Seal = Vec<Bytes>;
type AtomicMs = AtomicUsize;

/// Timer token representing the consensus step timeouts.
pub const ENGINE_TIMEOUT_TOKEN: TimerToken = 0;

impl IoHandler<StepMessage> for TimerHandler {
	fn initialize(&self, io: &IoContext<StepMessage>) {
		if let Some(engine) = self.engine.upgrade() {
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.next_timeout()).expect("Error registering engine timeout");
		}
	}

	fn timeout(&self, io: &IoContext<StepMessage>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			if let Some(engine) = self.engine.upgrade() {
				println!("Timeout: {:?}", get_time());
				engine.to_propose();
				io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.next_timeout()).expect("Failed to restart consensus step timer.")
			}
		}
	}

	fn message(&self, io: &IoContext<StepMessage>, _net_message: &StepMessage) {
		if let Some(engine) = self.engine.upgrade() {
			println!("Message: {:?}", get_time().sec);
			io.clear_timer(ENGINE_TIMEOUT_TOKEN).expect("Failed to restart consensus step timer.");
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.next_timeout()).expect("Failed to restart consensus step timer.")
		}
	}
}

#[cfg(test)]
mod tests {
	use common::*;
	use std::thread::sleep;
	use std::time::{Duration};
	use rlp::{UntrustedRlp, RlpStream, Stream, View};
	use block::*;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use spec::Spec;
	use engines::Engine;
	use super::{Tendermint, TendermintParams};

	/// Create a new test chain spec with `Tendermint` consensus engine.
	/// Account "0".sha3() and "1".sha3() are a validators.
	fn new_test_tendermint() -> Spec { Spec::load(include_bytes!("../../res/tendermint.json")) }

	fn propose_default(engine: &Arc<Engine>, proposer: Address) -> Result<Bytes, Error> {
		let mut s = RlpStream::new_list(3);
		let header = Header::default();
		s.append(&0u8).append(&0u8).append(&header.bare_hash());
		let drain = s.out();
		let propose_rlp = UntrustedRlp::new(&drain);

		engine.handle_message(proposer, H520::default(), propose_rlp)
	}

	fn default_block() -> Vec<u8> {
		vec![160, 39, 191, 179, 126, 80, 124, 233, 13, 161, 65, 48, 114, 4, 177, 198, 186, 36, 25, 67, 128, 97, 53, 144, 172, 80, 202, 75, 29, 113, 152, 255, 101]
	}

	#[test]
	fn has_valid_metadata() {
		let engine = new_test_tendermint().engine;
		assert!(!engine.name().is_empty());
		assert!(engine.version().major >= 1);
	}

	#[test]
	fn can_return_schedule() {
		let engine = new_test_tendermint().engine;
		let schedule = engine.schedule(&EnvInfo {
			number: 10000000,
			author: 0.into(),
			timestamp: 0,
			difficulty: 0.into(),
			last_hashes: Arc::new(vec![]),
			gas_used: 0.into(),
			gas_limit: 0.into(),
		});

		assert!(schedule.stack_limit > 0);
	}

	#[test]
	fn can_do_seal_verification_fail() {
		let engine = new_test_tendermint().engine;
		let header: Header = Header::default();

		let verify_result = engine.verify_block_basic(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidSealArity(_))) => {},
			Err(_) => { panic!("should be block seal-arity mismatch error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_generate_seal() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("".sha3(), "").unwrap();
		tap.unlock_account_permanently(addr, "".into()).unwrap();

		let spec = new_test_tendermint();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, addr, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b = b.close_and_lock();
		let seal = engine.generate_seal(b.block(), Some(&tap)).unwrap();
		assert!(b.try_seal(engine, seal).is_ok());
	}

	#[test]
	fn propose_step() {
		let engine = new_test_tendermint().engine;
		let tap = AccountProvider::transient_provider();

		let not_validator_addr = tap.insert_account("101".sha3(), "101").unwrap();
		assert!(propose_default(&engine, not_validator_addr).is_err());

		let not_proposer_addr = tap.insert_account("0".sha3(), "0").unwrap();
		assert!(propose_default(&engine, not_proposer_addr).is_err());

		let proposer_addr = tap.insert_account("1".sha3(), "1").unwrap();
		assert_eq!(default_block(), propose_default(&engine, proposer_addr).unwrap());

		assert!(propose_default(&engine, proposer_addr).is_err());
		assert!(propose_default(&engine, not_proposer_addr).is_err());
	}

	#[test]
	fn proposer_switching() {
		let engine = new_test_tendermint().engine;
		let tap = AccountProvider::transient_provider();

		let not_proposer_addr = tap.insert_account("0".sha3(), "0").unwrap();
		assert!(propose_default(&engine, not_proposer_addr).is_err());

		sleep(Duration::from_secs(3));

		assert_eq!(default_block(), propose_default(&engine, not_proposer_addr).unwrap());
	}

	#[test]
	fn prevote_step() {
		let engine = new_test_tendermint().engine;
		propose_default(&engine, Address::default());
	}

	#[test]
	fn timeout_switching() {
		let engine = new_test_tendermint().engine;
		let tender = Tendermint::new(engine.params().clone(), TendermintParams::default(), BTreeMap::new());
	
		println!("Waiting for timeout");
		sleep(Duration::from_secs(60));

	}
}
