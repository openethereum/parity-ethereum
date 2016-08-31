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
use common::*;
use account_provider::AccountProvider;
use block::*;
use spec::CommonParams;
use engines::{Engine, EngineError, ProposeCollect};
use evm::Schedule;
use ethjson;
use io::{IoContext, IoHandler, TimerToken};
use service::{ClientIoMessage, ENGINE_TIMEOUT_TOKEN};
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
	/// Consensus round.
	r: u64,
	/// Consensus step.
	s: RwLock<Step>,
	/// Current step timeout in ms.
	timeout: AtomicTimerToken,
	/// Used to swith proposer.
	proposer_nonce: AtomicUsize,
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
			r: 0,
			s: RwLock::new(Step::Propose),
			timeout: AtomicUsize::new(propose_timeout),
			proposer_nonce: AtomicUsize::new(0)
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

#[derive(Debug)]
struct DefaultTimeouts {
	propose: TimerToken,
	prevote: TimerToken,
	precommit: TimerToken,
	commit: TimerToken
}

type Seal = Vec<Bytes>;
type AtomicTimerToken = AtomicUsize;

impl IoHandler<ClientIoMessage> for Tendermint {
	fn initialize(&self, io: &IoContext<ClientIoMessage>) {
		io.register_timer(ENGINE_TIMEOUT_TOKEN, self.our_params.timeout.load(AtomicOrdering::Relaxed) as u64).expect("Error registering engine timeout");
	}

	fn timeout(&self, io: &IoContext<ClientIoMessage>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			println!("Timeout: {:?}", get_time().sec);
			io.clear_timer(ENGINE_TIMEOUT_TOKEN).expect("Failed to cancel consensus timer.");
			match *self.our_params.s.try_read().unwrap() {
				Step::Propose => self.to_propose(),
				Step::Prevote(ref proposal) => self.to_precommit(proposal.hash.clone()),
				Step::Precommit(_, _) => self.to_propose(),
				Step::Commit(_) => self.to_propose(),
			};
			io.register_timer(ENGINE_TIMEOUT_TOKEN, 3000).expect("Failed to start new consensus timer.")
		}
	}

	fn message(&self, io: &IoContext<ClientIoMessage>, net_message: &ClientIoMessage) {
		if let &ClientIoMessage::ConsensusStep(next_timeout) = net_message {
			println!("Message: {:?}", get_time().sec);
			io.clear_timer(ENGINE_TIMEOUT_TOKEN).expect("Failed to cancel consensus timer.");
			io.register_timer(ENGINE_TIMEOUT_TOKEN, next_timeout).expect("Failed to start new consensus timer.")
		}
	}
}

impl From<ethjson::spec::TendermintParams> for TendermintParams {
	fn from(p: ethjson::spec::TendermintParams) -> Self {
		let val: Vec<_> = p.validators.into_iter().map(Into::into).collect();
		let val_n = val.len();
		let propose_timeout = 3;
		TendermintParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			validators: val,
			validator_n: val_n,
			timeouts: DefaultTimeouts {
				propose: propose_timeout,
				prevote: 3,
				precommit: 3,
				commit: 3
			},
			r: 0,
			s: RwLock::new(Step::Propose),
			timeout: AtomicUsize::new(propose_timeout),
			proposer_nonce: AtomicUsize::new(0)
		}
	}
}

/// Engine using `Tendermint` consensus algorithm, suitable for EVM chain.
pub struct Tendermint {
	params: CommonParams,
	our_params: TendermintParams,
	builtins: BTreeMap<Address, Builtin>,
}

impl Tendermint {
	/// Create a new instance of Tendermint engine
	pub fn new(params: CommonParams, our_params: TendermintParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		Tendermint {
			params: params,
			our_params: our_params,
			builtins: builtins,
		}
	}

	fn proposer(&self) -> Address {
		let ref p = self.our_params;
		p.validators.get(p.proposer_nonce.load(AtomicOrdering::Relaxed)%p.validator_n).unwrap().clone()
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

	fn to_propose(&self) {
		self.our_params.proposer_nonce.fetch_add(1, AtomicOrdering::Relaxed);
		let mut guard = self.our_params.s.write();
		*guard = Step::Propose;
	}

	fn propose_message(&self, message: UntrustedRlp) -> Result<Bytes, Error> {
		// Check if message is for correct step.
		match *self.our_params.s.try_read().unwrap() {
			Step::Propose => (),
			_ => try!(Err(EngineError::WrongStep)),
		}
		let proposal = try!(message.as_val());
		self.to_prevote(proposal);
		Ok(message.as_raw().to_vec())
	}

	fn to_prevote(&self, proposal: H256) {
		let mut guard = self.our_params.s.write();
		// Proceed to the prevote step.
		*guard = Step::Prevote(self.new_vote(proposal));
	}

	fn prevote_message(&self, sender: Address, message: UntrustedRlp) -> Result<Bytes, Error> {
		// Check if message is for correct step.
		match *self.our_params.s.try_write().unwrap() {
			Step::Prevote(ref mut vote) => {
				// Vote if message is about the right block.
				if vote.hash == try!(message.as_val()) {
					vote.vote(sender);
					// Move to next step is prevote is won.
					if vote.is_won() { self.to_precommit(vote.hash); }
					Ok(message.as_raw().to_vec())
				} else {
					try!(Err(EngineError::WrongVote))
				}
			},
			_ => try!(Err(EngineError::WrongStep)),
		}
	}

	fn to_precommit(&self, proposal: H256) {
		let mut guard = self.our_params.s.write();
		*guard = Step::Precommit(self.new_vote(proposal), Vec::new());
	}

	fn precommit_message(&self, sender: Address, signature: H520, message: UntrustedRlp) -> Result<Bytes, Error> {
		// Check if message is for correct step.
		match *self.our_params.s.try_write().unwrap() {
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
		let mut guard = self.our_params.s.write();
		*guard = Step::Commit(seal);
	}

	fn threshold(&self) -> usize {
		self.our_params.validator_n*2/3
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
		header.difficulty = parent.difficulty;
		header.gas_limit = {
			let gas_limit = parent.gas_limit;
			let bound_divisor = self.our_params.gas_limit_bound_divisor;
			if gas_limit < gas_floor_target {
				min(gas_floor_target, gas_limit + gas_limit / bound_divisor - 1.into())
			} else {
				max(gas_floor_target, gas_limit - gas_limit / bound_divisor + 1.into())
			}
		};
		header.note_dirty();
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
		if self.our_params.r != try!(message.val_at(0)) { try!(Err(EngineError::WrongRound)) }
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
		if header.seal.len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal.len() }
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
		let min_gas = parent.gas_limit - parent.gas_limit / gas_limit_divisor;
		let max_gas = parent.gas_limit + parent.gas_limit / gas_limit_divisor;
		if header.gas_limit <= min_gas || header.gas_limit >= max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit })));
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

#[cfg(test)]
mod tests {
	use common::*;
	use std::thread::sleep;
	use std::time::{Duration, Instant};
	use block::*;
	use tests::helpers::*;
	use service::{ClientService, ClientIoMessage};
	use devtools::RandomTempPath;
	use client::ClientConfig;
	use miner::Miner;
	use account_provider::AccountProvider;
	use spec::Spec;
	use engines::Engine;
	use super::{Tendermint, TendermintParams};

	/// Create a new test chain spec with `Tendermint` consensus engine.
	/// Account "0".sha3() and "1".sha3() are a validators.
	fn new_test_tendermint() -> Spec { Spec::load(include_bytes!("../../res/tendermint.json")) }

	fn new_test_client_service() -> ClientService {
		let temp_path = RandomTempPath::new();
		let mut path = temp_path.as_path().to_owned();
		path.push("pruning");
		path.push("db");

		let spec = get_test_spec();
		let service = ClientService::start(
			ClientConfig::default(),
			&spec,
			&path,
			&path,
			Arc::new(Miner::with_spec(&spec)),
		);
		service.unwrap()
	}

	fn propose_default(engine: &Arc<Engine>, proposer: Address) -> Result<Bytes, Error> {
		let mut s = RlpStream::new_list(3);
		let header = Header::default();
		s.append(&0u8).append(&0u8).append(&header.bare_hash());
		let drain = s.out();
		let propose_rlp = UntrustedRlp::new(&drain);

		engine.handle_message(proposer, H520::default(), propose_rlp)
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
		assert_eq!(vec![160, 39, 191, 179, 126, 80, 124, 233, 13, 161, 65, 48, 114, 4, 177, 198, 186, 36, 25, 67, 128, 97, 53, 144, 172, 80, 202, 75, 29, 113, 152, 255, 101],
				   propose_default(&engine, proposer_addr).unwrap());

		assert!(propose_default(&engine, proposer_addr).is_err());
		assert!(propose_default(&engine, not_proposer_addr).is_err());
	}

	#[test]
	fn prevote_step() {
		let engine = new_test_tendermint().engine;
		propose_default(&engine, Address::default());
	}

	#[test]
	fn handle_message() {
		false;
	}

	#[test]
	fn timeout_switching() {
		let service = new_test_client_service();
		let engine = new_test_tendermint().engine;
		let tender = Tendermint::new(engine.params().clone(), TendermintParams::default(), BTreeMap::new());
		service.register_io_handler(Arc::new(tender));
	
		println!("Waiting for timeout");
		sleep(Duration::from_secs(10));

		let message_channel = service.io().channel();
		message_channel.send(ClientIoMessage::ConsensusStep(1000));
		sleep(Duration::from_secs(5));
	}
}
