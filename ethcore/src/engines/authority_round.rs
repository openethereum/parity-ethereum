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

//! A blockchain engine that supports a non-instant BFT proof-of-authority.

use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering as AtomicOrdering};
use std::sync::Weak;
use std::time::{UNIX_EPOCH, Duration};
use util::*;
use ethkey::{verify_address, Signature};
use rlp::{UntrustedRlp, View, encode};
use account_provider::AccountProvider;
use block::*;
use spec::CommonParams;
use engines::Engine;
use header::Header;
use error::{Error, BlockError};
use evm::Schedule;
use ethjson;
use io::{IoContext, IoHandler, TimerToken, IoService, IoChannel};
use service::ClientIoMessage;
use transaction::SignedTransaction;
use env_info::EnvInfo;
use builtin::Builtin;

/// `AuthorityRound` params.
#[derive(Debug, PartialEq)]
pub struct AuthorityRoundParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// Time to wait before next block or authority switching.
	pub step_duration: Duration,
	/// Valid authorities.
	pub authorities: Vec<Address>,
	/// Number of authorities.
	pub authority_n: usize,
}

impl From<ethjson::spec::AuthorityRoundParams> for AuthorityRoundParams {
	fn from(p: ethjson::spec::AuthorityRoundParams) -> Self {
		AuthorityRoundParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			step_duration: Duration::from_secs(p.step_duration.into()),
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
	transition_service: IoService<BlockArrived>,
	message_channel: Mutex<Option<IoChannel<ClientIoMessage>>>,
	step: AtomicUsize,
	proposed: AtomicBool,
}

fn header_step(header: &Header) -> Result<usize, ::rlp::DecoderError> {
	UntrustedRlp::new(&header.seal()[0]).as_val()
}

fn header_signature(header: &Header) -> Result<Signature, ::rlp::DecoderError> {
	UntrustedRlp::new(&header.seal()[1]).as_val::<H520>().map(Into::into)
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
		let initial_step = (unix_now().as_secs() / our_params.step_duration.as_secs()) as usize;
		let engine = Arc::new(
			AuthorityRound {
				params: params,
				our_params: our_params,
				builtins: builtins,
				transition_service: try!(IoService::<BlockArrived>::start()),
				message_channel: Mutex::new(None),
				step: AtomicUsize::new(initial_step),
				proposed: AtomicBool::new(false)
			});
		let handler = TransitionHandler { engine: Arc::downgrade(&engine) };
		try!(engine.transition_service.register_handler(Arc::new(handler)));
		Ok(engine)
	}

	fn step(&self) -> usize {
		self.step.load(AtomicOrdering::SeqCst)
	}

	fn remaining_step_duration(&self) -> Duration {
		let now = unix_now();
		let step_end = self.our_params.step_duration * (self.step() as u32 + 1);
		if step_end > now {
			step_end - now
		} else {
			Duration::from_secs(0)
		}
	}

	fn step_proposer(&self, step: usize) -> &Address {
		let ref p = self.our_params;
		p.authorities.get(step % p.authority_n).expect("There are authority_n authorities; taking number modulo authority_n gives number in authority_n range; qed")
	}

	fn is_step_proposer(&self, step: usize, address: &Address) -> bool {
		self.step_proposer(step) == address
	}
}

fn unix_now() -> Duration {
	UNIX_EPOCH.elapsed().expect("Valid time has to be set in your system.")
}

struct TransitionHandler {
	engine: Weak<AuthorityRound>,
}

#[derive(Clone)]
struct BlockArrived;

const ENGINE_TIMEOUT_TOKEN: TimerToken = 23;

impl IoHandler<BlockArrived> for TransitionHandler {
	fn initialize(&self, io: &IoContext<BlockArrived>) {
		if let Some(engine) = self.engine.upgrade() {
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.remaining_step_duration().as_millis())
				.unwrap_or_else(|e| warn!(target: "poa", "Failed to start consensus step timer: {}.", e))
		}
	}

	fn timeout(&self, io: &IoContext<BlockArrived>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			if let Some(engine) = self.engine.upgrade() {
				engine.step.fetch_add(1, AtomicOrdering::SeqCst);
				engine.proposed.store(false, AtomicOrdering::SeqCst);
				if let Some(ref channel) = *engine.message_channel.lock() {
					match channel.send(ClientIoMessage::UpdateSealing) {
						Ok(_) => trace!(target: "poa", "timeout: UpdateSealing message sent for step {}.", engine.step.load(AtomicOrdering::Relaxed)),
						Err(err) => trace!(target: "poa", "timeout: Could not send a sealing message {} for step {}.", err, engine.step.load(AtomicOrdering::Relaxed)),
					}
				}
				io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.remaining_step_duration().as_millis())
					.unwrap_or_else(|e| warn!(target: "poa", "Failed to restart consensus step timer: {}.", e))
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
	fn builtins(&self) -> &BTreeMap<Address, Builtin> { &self.builtins }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, header: &Header) -> BTreeMap<String, String> {
		map![
			"step".into() => header_step(header).as_ref().map(ToString::to_string).unwrap_or("".into()),
			"signature".into() => header_signature(header).as_ref().map(ToString::to_string).unwrap_or("".into())
		]
	}

	fn schedule(&self, _env_info: &EnvInfo) -> Schedule {
		Schedule::new_post_eip150(true, true, true)
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
		if self.proposed.load(AtomicOrdering::SeqCst) { return None; }
		let header = block.header();
		let step = self.step();
		if self.is_step_proposer(step, header.author()) {
			if let Some(ap) = accounts {
				// Account should be permanently unlocked, otherwise sealing will fail.
				if let Ok(signature) = ap.sign(*header.author(), None, header.bare_hash()) {
					trace!(target: "poa", "generate_seal: Issuing a block for step {}.", step);
					self.proposed.store(true, AtomicOrdering::SeqCst);
					return Some(vec![encode(&step).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
				} else {
					warn!(target: "poa", "generate_seal: FAIL: Accounts secret key unavailable.");
				}
			} else {
				warn!(target: "poa", "generate_seal: FAIL: Accounts not provided.");
			}
		}
		None
	}

	/// Check the number of seal fields.
	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		if header.seal().len() != self.seal_fields() {
			trace!(target: "poa", "verify_block_basic: wrong number of seal fields");
			Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)))
		} else {
			Ok(())
		}
	}

	/// Check if the signature belongs to the correct proposer.
	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
        let header_step = try!(header_step(header));
        // Give one step slack if step is lagging, double vote is still not possible.
		if header_step <= self.step() + 1 {
			let proposer_signature = try!(header_signature(header));
			let ok_sig = try!(verify_address(self.step_proposer(header_step), &proposer_signature, &header.bare_hash()));
			if ok_sig {
				Ok(())
			} else {
				trace!(target: "poa", "verify_block_unordered: invalid seal signature");
				try!(Err(BlockError::InvalidSeal))
			}
		} else {
			trace!(target: "poa", "verify_block_unordered: block from the future");
			try!(Err(BlockError::InvalidSeal))
		}
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		// Don't calculate difficulty for genesis blocks.
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		let step = try!(header_step(header));
		// Check if parent is from a previous step.
		if step == try!(header_step(parent)) { 
			trace!(target: "poa", "Multiple blocks proposed for step {}.", step);
			try!(Err(BlockError::DoubleVote(header.author().clone())));
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
		let mut guard = self.message_channel.lock();
		*guard = Some(message_channel);
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use env_info::EnvInfo;
	use header::Header;
	use error::{Error, BlockError};
	use rlp::encode;
	use block::*;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use spec::Spec;
	use std::time::UNIX_EPOCH;

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
	fn generates_seal_and_does_not_double_propose() {
		let tap = AccountProvider::transient_provider();
		let addr1 = tap.insert_account("1".sha3(), "1").unwrap();
		tap.unlock_account_permanently(addr1, "1".into()).unwrap();
		let addr2 = tap.insert_account("2".sha3(), "2").unwrap();
		tap.unlock_account_permanently(addr2, "2".into()).unwrap();

		let spec = Spec::new_test_round();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db1 = get_temp_state_db().take();
		spec.ensure_db_good(&mut db1).unwrap();
		let mut db2 = get_temp_state_db().take();
		spec.ensure_db_good(&mut db2).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b1 = OpenBlock::new(engine, Default::default(), false, db1, &genesis_header, last_hashes.clone(), addr1, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b1 = b1.close_and_lock();
		let b2 = OpenBlock::new(engine, Default::default(), false, db2, &genesis_header, last_hashes, addr2, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b2 = b2.close_and_lock();

		if let Some(seal) = engine.generate_seal(b1.block(), Some(&tap)) {
			assert!(b1.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(b1.block(), Some(&tap)).is_none());
		}

		if let Some(seal) = engine.generate_seal(b2.block(), Some(&tap)) {
			assert!(b2.clone().try_seal(engine, seal).is_ok());
			// Second proposal is forbidden.
			assert!(engine.generate_seal(b2.block(), Some(&tap)).is_none());
		}
	}

	#[test]
	fn proposer_switching() {
		let mut header: Header = Header::default();
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("0".sha3(), "0").unwrap();

		header.set_author(addr);

		let engine = Spec::new_test_round().engine;

		let signature = tap.sign(addr, Some("0".into()), header.bare_hash()).unwrap();
		let mut step = UNIX_EPOCH.elapsed().unwrap().as_secs();
		header.set_seal(vec![encode(&step).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
		let first_ok = engine.verify_block_seal(&header).is_ok();
		step = step + 1;
		header.set_seal(vec![encode(&step).to_vec(), encode(&(&*signature as &[u8])).to_vec()]);
		let second_ok = engine.verify_block_seal(&header).is_ok();

		assert!(first_ok ^ second_ok);
	}
}
