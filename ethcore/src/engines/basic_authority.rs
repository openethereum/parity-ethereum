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

//! A blockchain engine that supports a basic, non-BFT proof-of-authority.

use std::sync::Weak;
use ethereum_types::{H256, H520};
use parking_lot::RwLock;
use ethkey::{self, Signature};
use block::*;
use engines::{Engine, Seal, ConstructedVerifier, EngineError};
use engines::signer::EngineSigner;
use error::{BlockError, Error};
use ethjson;
use client::EngineClient;
use machine::{AuxiliaryData, Call, EthereumMachine};
use types::header::{Header, ExtendedHeader};
use super::validator_set::{ValidatorSet, SimpleList, new_validator_set};

/// `BasicAuthority` params.
#[derive(Debug, PartialEq)]
pub struct BasicAuthorityParams {
	/// Valid signatories.
	pub validators: ethjson::spec::ValidatorSet,
}

impl From<ethjson::spec::BasicAuthorityParams> for BasicAuthorityParams {
	fn from(p: ethjson::spec::BasicAuthorityParams) -> Self {
		BasicAuthorityParams {
			validators: p.validators,
		}
	}
}

struct EpochVerifier {
	list: SimpleList,
}

impl super::EpochVerifier<EthereumMachine> for EpochVerifier {
	fn verify_light(&self, header: &Header) -> Result<(), Error> {
		verify_external(header, &self.list)
	}
}

fn verify_external(header: &Header, validators: &ValidatorSet) -> Result<(), Error> {
	use rlp::Rlp;

	// Check if the signature belongs to a validator, can depend on parent state.
	let sig = Rlp::new(&header.seal()[0]).as_val::<H520>()?;
	let signer = ethkey::public_to_address(&ethkey::recover(&sig.into(), &header.bare_hash())?);

	if *header.author() != signer {
		return Err(EngineError::NotAuthorized(*header.author()).into())
	}

	match validators.contains(header.parent_hash(), &signer) {
		false => Err(BlockError::InvalidSeal.into()),
		true => Ok(())
	}
}

/// Engine using `BasicAuthority`, trivial proof-of-authority consensus.
pub struct BasicAuthority {
	machine: EthereumMachine,
	signer: RwLock<Option<Box<EngineSigner>>>,
	validators: Box<ValidatorSet>,
}

impl BasicAuthority {
	/// Create a new instance of BasicAuthority engine
	pub fn new(our_params: BasicAuthorityParams, machine: EthereumMachine) -> Self {
		BasicAuthority {
			machine: machine,
			signer: RwLock::new(None),
			validators: new_validator_set(our_params.validators),
		}
	}
}

impl Engine<EthereumMachine> for BasicAuthority {
	fn name(&self) -> &str { "BasicAuthority" }

	fn machine(&self) -> &EthereumMachine { &self.machine }

	// One field - the signature
	fn seal_fields(&self, _header: &Header) -> usize { 1 }

	fn seals_internally(&self) -> Option<bool> {
		Some(self.signer.read().is_some())
	}

	/// Attempt to seal the block internally.
	fn generate_seal(&self, block: &ExecutedBlock, _parent: &Header) -> Seal {
		let header = &block.header;
		let author = header.author();
		if self.validators.contains(header.parent_hash(), author) {
			// account should be pernamently unlocked, otherwise sealing will fail
			if let Ok(signature) = self.sign(header.bare_hash()) {
				return Seal::Regular(vec![::rlp::encode(&(&H520::from(signature) as &[u8]))]);
			} else {
				trace!(target: "basicauthority", "generate_seal: FAIL: accounts secret key unavailable");
			}
		}
		Seal::None
	}

	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
		Ok(())
	}

	fn verify_block_external(&self, header: &Header) -> Result<(), Error> {
		verify_external(header, &*self.validators)
	}

	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		self.validators.genesis_epoch_data(header, call)
	}

	#[cfg(not(test))]
	fn signals_epoch_end(&self, _header: &Header, _auxiliary: AuxiliaryData)
		-> super::EpochChange<EthereumMachine>
	{
		// don't bother signalling even though a contract might try.
		super::EpochChange::No
	}

	#[cfg(test)]
	fn signals_epoch_end(&self, header: &Header, auxiliary: AuxiliaryData)
		-> super::EpochChange<EthereumMachine>
	{
		// in test mode, always signal even though they don't be finalized.
		let first = header.number() == 0;
		self.validators.signals_epoch_end(first, header, auxiliary)
	}

	fn is_epoch_end(
		&self,
		chain_head: &Header,
		_finalized: &[H256],
		_chain: &super::Headers<Header>,
		_transition_store: &super::PendingTransitionStore,
	) -> Option<Vec<u8>> {
		let first = chain_head.number() == 0;

		// finality never occurs so only apply immediate transitions.
		self.validators.is_epoch_end(first, chain_head)
	}

	fn is_epoch_end_light(
		&self,
		chain_head: &Header,
		chain: &super::Headers<Header>,
		transition_store: &super::PendingTransitionStore,
	) -> Option<Vec<u8>> {
		self.is_epoch_end(chain_head, &[], chain, transition_store)
	}

	fn epoch_verifier<'a>(&self, header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {
		let first = header.number() == 0;

		match self.validators.epoch_set(first, &self.machine, header.number(), proof) {
			Ok((list, finalize)) => {
				let verifier = Box::new(EpochVerifier { list: list });

				// our epoch verifier will ensure no unverified verifier is ever verified.
				match finalize {
					Some(finalize) => ConstructedVerifier::Unconfirmed(verifier, proof, finalize),
					None => ConstructedVerifier::Trusted(verifier),
				}
			}
			Err(e) => ConstructedVerifier::Err(e),
		}
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		self.validators.register_client(client);
	}

	fn set_signer(&self, signer: Box<EngineSigner>) {
		*self.signer.write() = Some(signer);
	}

	fn sign(&self, hash: H256) -> Result<Signature, Error> {
		Ok(self.signer.read()
			.as_ref()
			.ok_or_else(|| ethkey::Error::InvalidAddress)?
			.sign(hash)?
		)
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		None
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
		super::total_difficulty_fork_choice(new, current)
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use hash::keccak;
	use ethereum_types::H520;
	use block::*;
	use test_helpers::get_temp_state_db;
	use accounts::AccountProvider;
	use types::header::Header;
	use spec::Spec;
	use engines::Seal;
	use tempdir::TempDir;

	/// Create a new test chain spec with `BasicAuthority` consensus engine.
	fn new_test_authority() -> Spec {
		let bytes: &[u8] = include_bytes!("../../res/basic_authority.json");
		let tempdir = TempDir::new("").unwrap();
		Spec::load(&tempdir.path(), bytes).expect("invalid chain spec")
	}

	#[test]
	fn has_valid_metadata() {
		let engine = new_test_authority().engine;
		assert!(!engine.name().is_empty());
	}

	#[test]
	fn can_return_schedule() {
		let engine = new_test_authority().engine;
		let schedule = engine.schedule(10000000);
		assert!(schedule.stack_limit > 0);
	}

	#[test]
	fn can_do_signature_verification_fail() {
		let engine = new_test_authority().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![::rlp::encode(&H520::default())]);

		let verify_result = engine.verify_block_external(&header);
		assert!(verify_result.is_err());
	}

	#[test]
	fn can_generate_seal() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account(keccak("").into(), &"".into()).unwrap();

		let spec = new_test_authority();
		let engine = &*spec.engine;
		engine.set_signer(Box::new((Arc::new(tap), addr, "".into())));
		let genesis_header = spec.genesis_header();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, addr, (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b = b.close_and_lock().unwrap();
		if let Seal::Regular(seal) = engine.generate_seal(&b, &genesis_header) {
			assert!(b.try_seal(engine, seal).is_ok());
		}
	}

	#[test]
	fn seals_internally() {
		let tap = AccountProvider::transient_provider();
		let authority = tap.insert_account(keccak("").into(), &"".into()).unwrap();

		let engine = new_test_authority().engine;
		assert!(!engine.seals_internally().unwrap());
		engine.set_signer(Box::new((Arc::new(tap), authority, "".into())));
		assert!(engine.seals_internally().unwrap());
	}
}
