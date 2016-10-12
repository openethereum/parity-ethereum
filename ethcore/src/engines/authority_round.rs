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

//! A blockchain engine that supports a basic, non-BFT proof-of-authority.

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Weak;
use common::*;
use ethkey::verify_address;
use rlp::{UntrustedRlp, View, encode, decode};
use account_provider::AccountProvider;
use block::*;
use spec::CommonParams;
use engines::Engine;
use evm::Schedule;
use ethjson;
use io::{IoContext, IoHandler, TimerToken, IoService, IoChannel};
use service::ClientIoMessage;

/// `AuthorityRound` params.
#[derive(Debug, PartialEq)]
pub struct AuthorityRoundParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// Time to wait before next block or authority switching.
	pub step_duration: u64,
	/// Valid authorities.
	pub authorities: Vec<Address>,
	/// Number of authorities.
	pub authority_n: usize,
}

impl From<ethjson::spec::AuthorityRoundParams> for AuthorityRoundParams {
	fn from(p: ethjson::spec::AuthorityRoundParams) -> Self {
		AuthorityRoundParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			step_duration: p.step_duration.into(),
			authority_n: p.authorities.len(),
			authorities: p.authorities.into_iter().map(Into::into).collect::<Vec<_>>(),
		}
	}
}

/// Engine using `AuthorityRound` proof-of-work consensus algorithm, suitable for Ethereum
/// mainnet chains in the Olympic, Frontier and Homestead eras.
pub struct AuthorityRound {
	params: CommonParams,
	our_params: AuthorityRoundParams,
	builtins: BTreeMap<Address, Builtin>,
	transistion_service: IoService<BlockArrived>,
	message_channel: Mutex<Option<IoChannel<ClientIoMessage>>>,
	step: AtomicUsize,
}

impl AuthorityRound {
	/// Create a new instance of AuthorityRound engine
	pub fn new(params: CommonParams, our_params: AuthorityRoundParams, builtins: BTreeMap<Address, Builtin>) -> Arc<Self> {
		let engine = Arc::new(
			AuthorityRound {
				params: params,
				our_params: our_params,
				builtins: builtins,
				transistion_service: IoService::<BlockArrived>::start().expect("Error creating engine timeout service"),
				message_channel: Mutex::new(None),
				step: AtomicUsize::new(0),
			});
		let handler = TransitionHandler { engine: Arc::downgrade(&engine) };
		engine.transistion_service.register_handler(Arc::new(handler)).expect("Error creating engine timeout service");
		engine
	}

	fn step(&self) -> usize {
		self.step.load(AtomicOrdering::SeqCst)
	}

	fn step_proposer(&self, step: usize) -> &Address {
		let ref p = self.our_params;
		p.authorities.get(step%p.authority_n).unwrap()
	}

	fn is_step_proposer(&self, step: usize, address: &Address) -> bool {
		self.step_proposer(step) == address
	}
}

struct TransitionHandler {
	engine: Weak<AuthorityRound>,
}

#[derive(Clone)]
struct BlockArrived;

const ENGINE_TIMEOUT_TOKEN: TimerToken = 0;

impl IoHandler<BlockArrived> for TransitionHandler {
	fn initialize(&self, io: &IoContext<BlockArrived>) {
		if let Some(engine) = self.engine.upgrade() {
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.our_params.step_duration).expect("Error registering engine timeout");
		}
	}

	fn timeout(&self, io: &IoContext<BlockArrived>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			if let Some(engine) = self.engine.upgrade() {
				debug!(target: "authorityround", "Timeout step: {}", engine.step.load(AtomicOrdering::Relaxed));
				engine.step.fetch_add(1, AtomicOrdering::SeqCst);
				if let Some(ref channel) = *engine.message_channel.try_lock().unwrap() {
					match channel.send(ClientIoMessage::UpdateSealing) {
						Ok(_) => trace!(target: "authorityround", "timeout: UpdateSealing message sent."),
						Err(_) => trace!(target: "authorityround", "timeout: Could not send a sealing message."),
					}
				}
				io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.our_params.step_duration).expect("Failed to restart consensus step timer.")
			}
		}
	}

//	fn message(&self, io: &IoContext<BlockArrived>, _net_message: &BlockArrived) {
//		if let Some(engine) = self.engine.upgrade() {
//			trace!(target: "authorityround", "Message: {:?}", get_time().sec);
//			engine.step.fetch_add(1, AtomicOrdering::SeqCst);
//			io.clear_timer(ENGINE_TIMEOUT_TOKEN).expect("Failed to restart consensus step timer.");
//			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.our_params.step_duration).expect("Failed to restart consensus step timer.")
//		}
//	}
}

impl Engine for AuthorityRound {
	fn name(&self) -> &str { "AuthorityRound" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	// One field - the proposer signature.
	fn seal_fields(&self) -> usize { 2 }

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

	fn is_sealer(&self, author: &Address) -> Option<bool> {
		let ref p = self.our_params;
		Some(p.authorities.contains(author))
	}

	/// Attempt to seal the block internally.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which `false` will
	/// be returned.
	fn generate_seal(&self, block: &ExecutedBlock, accounts: Option<&AccountProvider>) -> Option<Vec<Bytes>> {
		let header = block.header();
		let step = self.step();
		if self.is_step_proposer(step, header.author()) {
			if let Some(ap) = accounts {
				// Account should be permanently unlocked, otherwise sealing will fail.
				if let Ok(signature) = ap.sign(*header.author(), header.bare_hash()) {
					return Some(vec![encode(&step).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
				} else {
					trace!(target: "authorityround", "generate_seal: FAIL: accounts secret key unavailable");
				}
			} else {
				trace!(target: "authorityround", "generate_seal: FAIL: accounts not provided");
			}
		}
		None
	}

	/// Check the number of seal fields.
	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		if header.seal().len() != self.seal_fields() {
			trace!(target: "authorityround", "verify_block_basic: wrong number of seal fields");
			Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)))
		} else {
			Ok(())
		}
	}

	/// Check if the signature belongs to the correct proposer.
	/// TODO: Keep track of BlockNumber to step relationship
	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		let step = try!(UntrustedRlp::new(&header.seal()[0]).as_val::<usize>());
		if step <= self.step() {
			let sig = try!(UntrustedRlp::new(&header.seal()[1]).as_val::<H520>());
			let ok_sig = try!(verify_address(self.step_proposer(step), &sig.into(), &header.bare_hash()));
			if ok_sig {
				Ok(())
			} else {
				trace!(target: "authorityround", "verify_block_unordered: invalid seal signature");
				try!(Err(BlockError::InvalidSeal))
			}
		} else {
			trace!(target: "authorityround", "verify_block_unordered: block from the future");
			try!(Err(BlockError::InvalidSeal))
		}
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		// Don't calculate difficulty for genesis blocks.
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

	fn verify_transaction_basic(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		try!(t.check_low_s());
		Ok(())
	}

	fn verify_transaction(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		t.sender().map(|_|()) // Perform EC recovery and cache sender
	}

	fn register_message_channel(&self, message_channel: IoChannel<ClientIoMessage>) {
		let mut guard = self.message_channel.try_lock().unwrap();
		*guard = Some(message_channel);
	}
}

impl Header {
	/// Get the none field of the header.
	pub fn signature(&self) -> H520 {
		decode(&self.seal()[0])
	}
}

#[cfg(test)]
mod tests {
	use common::*;
	use rlp::encode;
	use block::*;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use spec::Spec;
	use std::thread::sleep;
	use std::time::Duration;

	#[test]
	fn has_valid_metadata() {
		let engine = Spec::new_test_round().engine;
		assert!(!engine.name().is_empty());
		assert!(engine.version().major >= 1);
	}

	#[test]
	fn can_return_schedule() {
		let engine = Spec::new_test_round().engine;
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

		let verify_result = engine.verify_block_unordered(&header, None);
		assert!(verify_result.is_err());
	}

	#[test]
	fn can_generate_seal() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("1".sha3(), "1").unwrap();
		tap.unlock_account_permanently(addr, "1".into()).unwrap();

		let spec = Spec::new_test_round();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_state_db();
		let mut db = db_result.take();
		spec.ensure_db_good(&mut db).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, addr, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b = b.close_and_lock();
		let seal = engine.generate_seal(b.block(), Some(&tap)).unwrap();
		assert!(b.try_seal(engine, seal).is_ok());
	}

	#[test]
	fn proposer_switching() {
		let mut header: Header = Header::default();
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("0".sha3(), "0").unwrap();

		header.set_author(addr);

		let signature = tap.sign_with_password(addr, "0".into(), header.bare_hash()).unwrap();
		header.set_seal(vec![vec![1u8], encode(&(&*signature as &[u8])).to_vec()]);

		let engine = Spec::new_test_round().engine;

		// Too early.
		assert!(engine.verify_block_seal(&header).is_err());

		sleep(Duration::from_millis(2000));

		// Right step.
		assert!(engine.verify_block_seal(&header).is_ok());
	}
}
