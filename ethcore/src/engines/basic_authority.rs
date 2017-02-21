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

//! A blockchain engine that supports a basic, non-BFT proof-of-authority.

use std::sync::Weak;
use util::*;
use ethkey::{recover, public_to_address, Signature};
use account_provider::AccountProvider;
use block::*;
use builtin::Builtin;
use spec::CommonParams;
use engines::{Engine, Seal};
use env_info::EnvInfo;
use error::{BlockError, Error};
use evm::Schedule;
use ethjson;
use header::Header;
use client::Client;
use super::signer::EngineSigner;
use super::validator_set::{ValidatorSet, new_validator_set};

/// `BasicAuthority` params.
#[derive(Debug, PartialEq)]
pub struct BasicAuthorityParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// Valid signatories.
	pub validators: ethjson::spec::ValidatorSet,
}

impl From<ethjson::spec::BasicAuthorityParams> for BasicAuthorityParams {
	fn from(p: ethjson::spec::BasicAuthorityParams) -> Self {
		BasicAuthorityParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			validators: p.validators,
		}
	}
}

/// Engine using `BasicAuthority` proof-of-work consensus algorithm, suitable for Ethereum
/// mainnet chains in the Olympic, Frontier and Homestead eras.
pub struct BasicAuthority {
	params: CommonParams,
	gas_limit_bound_divisor: U256,
	builtins: BTreeMap<Address, Builtin>,
	signer: EngineSigner,
	validators: Box<ValidatorSet + Send + Sync>,
}

impl BasicAuthority {
	/// Create a new instance of BasicAuthority engine
	pub fn new(params: CommonParams, our_params: BasicAuthorityParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		BasicAuthority {
			params: params,
			gas_limit_bound_divisor: our_params.gas_limit_bound_divisor,
			builtins: builtins,
			validators: new_validator_set(our_params.validators),
			signer: Default::default(),
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
	fn extra_info(&self, _header: &Header) -> BTreeMap<String, String> { map!["signature".to_owned() => "TODO".to_owned()] }

	fn schedule(&self, _env_info: &EnvInfo) -> Schedule {
		Schedule::new_homestead()
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256, _gas_ceil_target: U256) {
		header.set_difficulty(parent.difficulty().clone());
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
		Some(self.validators.contains(&self.signer.address()))
	}

	/// Attempt to seal the block internally.
	fn generate_seal(&self, block: &ExecutedBlock) -> Seal {
		let header = block.header();
		let author = header.author();
		if self.validators.contains(author) {
			// account should be pernamently unlocked, otherwise sealing will fail
			if let Ok(signature) = self.signer.sign(header.bare_hash()) {
				return Seal::Regular(vec![::rlp::encode(&(&H520::from(signature) as &[u8])).to_vec()]);
			} else {
				trace!(target: "basicauthority", "generate_seal: FAIL: accounts secret key unavailable");
			}
		}
		Seal::None
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

	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		use rlp::{UntrustedRlp, View};

		// check the signature is legit.
		let sig = UntrustedRlp::new(&header.seal()[0]).as_val::<H520>()?;
		let signer = public_to_address(&recover(&sig.into(), &header.bare_hash())?);
		if !self.validators.contains(&signer) {
			return Err(BlockError::InvalidSeal)?;
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
		let gas_limit_divisor = self.gas_limit_bound_divisor;
		let min_gas = parent.gas_limit().clone() - parent.gas_limit().clone() / gas_limit_divisor;
		let max_gas = parent.gas_limit().clone() + parent.gas_limit().clone() / gas_limit_divisor;
		if header.gas_limit() <= &min_gas || header.gas_limit() >= &max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit().clone() })));
		}
		Ok(())
	}

	fn register_client(&self, client: Weak<Client>) {
		self.validators.register_contract(client);
	}

	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: String) {
		self.signer.set(ap, address, password);
	}

	fn sign(&self, hash: H256) -> Result<Signature, Error> {
		self.signer.sign(hash).map_err(Into::into)
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use block::*;
	use env_info::EnvInfo;
	use error::{BlockError, Error};
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use ethkey::Secret;
	use header::Header;
	use spec::Spec;
	use engines::Seal;

	/// Create a new test chain spec with `BasicAuthority` consensus engine.
	fn new_test_authority() -> Spec {
		let bytes: &[u8] = include_bytes!("../../res/basic_authority.json");
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
		header.set_seal(vec![::rlp::encode(&H520::default()).to_vec()]);

		let verify_result = engine.verify_block_unordered(&header, None);
		assert!(verify_result.is_err());
	}

	#[test]
	fn can_generate_seal() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account(Secret::from_slice(&"".sha3()).unwrap(), "").unwrap();

		let spec = new_test_authority();
		let engine = &*spec.engine;
		engine.set_signer(Arc::new(tap), addr, "".into());
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_state_db();
		let db = spec.ensure_db_good(db_result.take(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, addr, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b = b.close_and_lock();
		if let Seal::Regular(seal) = engine.generate_seal(b.block()) {
			assert!(b.try_seal(engine, seal).is_ok());
		}
	}

	#[test]
	fn seals_internally() {
		let tap = AccountProvider::transient_provider();
		let authority = tap.insert_account(Secret::from_slice(&"".sha3()).unwrap(), "").unwrap();

		let engine = new_test_authority().engine;
		assert!(!engine.seals_internally().unwrap());
		engine.set_signer(Arc::new(tap), authority, "".into());
		assert!(engine.seals_internally().unwrap());
	}
}
