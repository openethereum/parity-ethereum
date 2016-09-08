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
use ethkey::{recover, public_to_address};
use rlp::{UntrustedRlp, View, encode, decode};
use account_provider::AccountProvider;
use block::*;
use spec::CommonParams;
use engines::Engine;
use evm::Schedule;
use ethjson;
use io::{IoContext, IoHandler, TimerToken, IoService};
use time::get_time;

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
				step: AtomicUsize::new(0),
			});
		let handler = TransitionHandler { engine: Arc::downgrade(&engine) };
		engine.transistion_service.register_handler(Arc::new(handler)).expect("Error creating engine timeout service");
		engine
	}

	fn proposer(&self) -> &Address {
		let ref p = self.our_params;
		p.authorities.get(self.step.load(AtomicOrdering::Relaxed)%p.authority_n).unwrap()
	}

	fn is_proposer(&self, address: &Address) -> bool {
		self.proposer() == address
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
				engine.step.fetch_add(1, AtomicOrdering::Relaxed);
				io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.our_params.step_duration).expect("Failed to restart consensus step timer.")
			}
		}
	}

	fn message(&self, io: &IoContext<BlockArrived>, _net_message: &BlockArrived) {
		if let Some(engine) = self.engine.upgrade() {
			println!("Message: {:?}", get_time().sec);
			engine.step.fetch_add(1, AtomicOrdering::Relaxed);
			io.clear_timer(ENGINE_TIMEOUT_TOKEN).expect("Failed to restart consensus step timer.");
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.our_params.step_duration).expect("Failed to restart consensus step timer.")
		}
	}
}

impl Engine for AuthorityRound {
	fn name(&self) -> &str { "AuthorityRound" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	// One field - the proposer signature.
	fn seal_fields(&self) -> usize { 1 }

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

	/// Attempt to seal the block internally.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which `false` will
	/// be returned.
	fn generate_seal(&self, block: &ExecutedBlock, accounts: Option<&AccountProvider>) -> Option<Vec<Bytes>> {
		if self.is_proposer(block.header().author()) {
			if let Some(ap) = accounts {
				let header = block.header();
				let message = header.bare_hash();
				// Account should be pernamently unlocked, otherwise sealing will fail.
				if let Ok(signature) = ap.sign(*header.author(), message) {
					return Some(vec![encode(&(&*signature as &[u8])).to_vec()]);
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
	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		if header.seal().len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)));
		}
		Ok(())
	}

	/// Check if the signature belongs to the correct proposer.
	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		let sig = try!(UntrustedRlp::new(&header.seal()[0]).as_val::<H520>());
		let signer = public_to_address(&try!(recover(&sig.into(), &header.bare_hash())));
		if self.is_proposer(&signer) {
			Ok(())
		} else {
			try!(Err(BlockError::InvalidSeal))
		}
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
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

	fn verify_transaction_basic(&self, t: &SignedTransaction, _header: &Header) -> result::Result<(), Error> {
		try!(t.check_low_s());
		Ok(())
	}

	fn verify_transaction(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		t.sender().map(|_|()) // Perform EC recovery and cache sender
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

	/// Create a new test chain spec with `AuthorityRound` consensus engine.
	fn new_test_authority() -> Spec {
		let bytes: &[u8] = include_bytes!("../../res/authority_round.json");
		Spec::load(bytes).expect("invalid chain spec")
	}

	#[test]
	fn has_valid_metadata() {
		let engine = new_test_authority().engine;
		assert!(!engine.name().is_empty());
		assert!(engine.version().major >= 1);
	}

	#[test]
	fn can_return_schedule() {
		let engine = new_test_authority().engine;
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
		let engine = new_test_authority().engine;
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
		let engine = new_test_authority().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![encode(&H520::default()).to_vec()]);

		let verify_result = engine.verify_block_unordered(&header, None);
		assert!(verify_result.is_err());
	}

	#[test]
	fn can_generate_seal() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("".sha3(), "").unwrap();
		tap.unlock_account_permanently(addr, "".into()).unwrap();

		let spec = new_test_authority();
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
}
