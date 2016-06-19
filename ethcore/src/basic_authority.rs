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

use common::*;
use account_provider::AccountProvider;
use block::*;
use spec::{CommonParams, Spec};
use engine::*;
use evm::Schedule;
use ethjson;

/// `BasicAuthority` params.
#[derive(Debug, PartialEq)]
pub struct BasicAuthorityParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// Block duration.
	pub duration_limit: u64,
	/// Valid signatories.
	pub authorities: HashSet<Address>,
}

impl From<ethjson::spec::BasicAuthorityParams> for BasicAuthorityParams {
	fn from(p: ethjson::spec::BasicAuthorityParams) -> Self {
		BasicAuthorityParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			duration_limit: p.duration_limit.into(),
			authorities: p.authorities.into_iter().map(Into::into).collect::<HashSet<_>>(),
		}
	}
}

/// Engine using `BasicAuthority` proof-of-work consensus algorithm, suitable for Ethereum
/// mainnet chains in the Olympic, Frontier and Homestead eras.
pub struct BasicAuthority {
	params: CommonParams,
	our_params: BasicAuthorityParams,
	builtins: BTreeMap<Address, Builtin>,
}

impl BasicAuthority {
	/// Create a new instance of BasicAuthority engine
	pub fn new(params: CommonParams, our_params: BasicAuthorityParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		BasicAuthority {
			params: params,
			our_params: our_params,
			builtins: builtins,
		}
	}
}

impl Engine for BasicAuthority {
	fn name(&self) -> &str { "BasicAuthority" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	// One field - the signature
	fn seal_fields(&self) -> usize { 1 }

	fn params(&self) -> &CommonParams { &self.params }
	fn builtins(&self) -> &BTreeMap<Address, Builtin> { &self.builtins }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> HashMap<String, String> { hash_map!["signature".to_owned() => "TODO".to_owned()] }

	fn schedule(&self, _env_info: &EnvInfo) -> Schedule {
		Schedule::new_homestead()
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256) {
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
//		info!("ethash: populate_from_parent #{}: difficulty={} and gas_limit={}", header.number, header.difficulty, header.gas_limit);
	}

	/// Apply the block reward on finalisation of the block.
	/// This assumes that all uncles are valid uncles (i.e. of at least one generation before the current).
	fn on_close_block(&self, _block: &mut ExecutedBlock) {}

	/// Attempt to seal the block internally.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which `false` will
	/// be returned.
	fn generate_seal(&self, block: &ExecutedBlock, accounts: Option<&AccountProvider>) -> Option<Vec<Bytes>> {
		if let Some(ap) = accounts {
			let header = block.header();
			let message = header.bare_hash();
			// account should be pernamently unlocked, otherwise sealing will fail
			if let Ok(signature) = ap.sign(*block.header().author(), message) {
				return Some(vec![encode(&signature).to_vec()]);
			} else {
				trace!(target: "basicauthority", "generate_seal: FAIL: accounts secret key unavailable");
			}
		} else {
			trace!(target: "basicauthority", "generate_seal: FAIL: accounts not provided");
		}
		None
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

	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// check the signature is legit.
		let sig = try!(UntrustedRlp::new(&header.seal[0]).as_val::<H520>());
		let signer = Address::from(try!(ec::recover(&sig, &header.bare_hash())).sha3());
		if !self.our_params.authorities.contains(&signer) {
			return try!(Err(BlockError::InvalidSeal));
		}
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

impl Header {
	/// Get the none field of the header.
	pub fn signature(&self) -> H520 {
		decode(&self.seal()[0])
	}
}

/// Create a new test chain spec with `BasicAuthority` consensus engine.
pub fn new_test_authority() -> Spec { Spec::load(include_bytes!("../res/test_authority.json")) }

#[cfg(test)]
mod tests {
	use super::*;
	use common::*;
	use block::*;
	use tests::helpers::*;
	use account_provider::AccountProvider;

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
			last_hashes: vec![],
			dao_rescue_block_gas_limit: None,
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
		header.set_seal(vec![rlp::encode(&Signature::zero()).to_vec()]);

		let verify_result = engine.verify_block_unordered(&header, None);

		match verify_result {
			Err(Error::Util(UtilError::Crypto(CryptoError::InvalidSignature))) => {},
			Err(_) => { panic!("should be block difficulty error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_generate_seal() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("".sha3(), "").unwrap();
		tap.unlock_account_permanently(addr, "".into()).unwrap();

		let spec = new_test_authority();
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let last_hashes = vec![genesis_header.hash()];
		let vm_factory = Default::default();
		let b = OpenBlock::new(engine.deref(), &vm_factory, false, db, &genesis_header, last_hashes, None, addr, 3141562.into(), vec![]).unwrap();
		let b = b.close_and_lock();
		let seal = engine.generate_seal(b.block(), Some(&tap)).unwrap();
		assert!(b.try_seal(engine.deref(), seal).is_ok());
	}
}
