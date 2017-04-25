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

//! Consensus engine specification and basic implementations.

mod authority_round;
mod basic_authority;
mod epoch_verifier;
mod instant_seal;
mod null_engine;
mod signer;
mod tendermint;
mod transition;
mod validator_set;
mod vote_collector;

pub use self::authority_round::AuthorityRound;
pub use self::basic_authority::BasicAuthority;
pub use self::epoch_verifier::EpochVerifier;
pub use self::instant_seal::InstantSeal;
pub use self::null_engine::NullEngine;
pub use self::tendermint::Tendermint;

use std::sync::Weak;

use account_provider::AccountProvider;
use block::ExecutedBlock;
use builtin::Builtin;
use client::Client;
use env_info::{EnvInfo, LastHashes};
use error::Error;
use evm::Schedule;
use header::{Header, BlockNumber};
use receipt::Receipt;
use snapshot::SnapshotComponents;
use spec::CommonParams;
use transaction::{UnverifiedTransaction, SignedTransaction};
use evm::CreateContractAddress;

use ethkey::Signature;
use util::*;

/// Default EIP-210 contrat code
pub const DEFAULT_BLOCKHASH_CONTRACT: &'static str = "73fffffffffffffffffffffffffffffffffffffffe33141561007a57600143036020526000356101006020510755600061010060205107141561005057600035610100610100602051050761010001555b6000620100006020510714156100755760003561010062010000602051050761020001555b610161565b436000351215801561008c5780610095565b623567e0600035125b9050156100a757600060605260206060f35b610100600035430312156100ca57610100600035075460805260206080f3610160565b62010000600035430312156100e857600061010060003507146100eb565b60005b1561010d576101006101006000350507610100015460a052602060a0f361015f565b63010000006000354303121561012d576000620100006000350714610130565b60005b1561015357610100620100006000350507610200015460c052602060c0f361015e565b600060e052602060e0f35b5b5b5b5b";

/// Voting errors.
#[derive(Debug)]
pub enum EngineError {
	/// Signature does not belong to an authority.
	NotAuthorized(Address),
	/// The same author issued different votes at the same step.
	DoubleVote(Address),
	/// The received block is from an incorrect proposer.
	NotProposer(Mismatch<Address>),
	/// Message was not expected.
	UnexpectedMessage,
	/// Seal field has an unexpected size.
	BadSealFieldSize(OutOfBounds<usize>),
	/// Validation proof insufficient.
	InsufficientProof(String),
}

impl fmt::Display for EngineError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::EngineError::*;
		let msg = match *self {
			DoubleVote(ref address) => format!("Author {} issued too many blocks.", address),
			NotProposer(ref mis) => format!("Author is not a current proposer: {}", mis),
			NotAuthorized(ref address) => format!("Signer {} is not authorized.", address),
			UnexpectedMessage => "This Engine should not be fed messages.".into(),
			BadSealFieldSize(ref oob) => format!("Seal field has an unexpected length: {}", oob),
			InsufficientProof(ref msg) => format!("Insufficient validation proof: {}", msg),
		};

		f.write_fmt(format_args!("Engine error ({})", msg))
	}
}

/// Seal type.
#[derive(Debug, PartialEq, Eq)]
pub enum Seal {
	/// Proposal seal; should be broadcasted, but not inserted into blockchain.
	Proposal(Vec<Bytes>),
	/// Regular block seal; should be part of the blockchain.
	Regular(Vec<Bytes>),
	/// Engine does generate seal for this block right now.
	None,
}

/// Type alias for a function we can make calls through synchronously.
pub type Call<'a> = Fn(Address, Bytes) -> Result<Bytes, String> + 'a;

/// Results of a query of whether an epoch change occurred at the given block.
#[derive(Debug, Clone, PartialEq)]
pub enum EpochChange {
	/// Cannot determine until more data is passed.
	Unsure(Unsure),
	/// No epoch change.
	No,
	/// Validation proof required, and the new epoch number.
	Yes(u64),
}

/// More data required to determine if an epoch change occurred at a given block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unsure {
	/// Needs the body.
	NeedsBody,
	/// Needs the receipts.
	NeedsReceipts,
	/// Needs both body and receipts.
	NeedsBoth,
}

/// A consensus mechanism for the chain. Generally either proof-of-work or proof-of-stake-based.
/// Provides hooks into each of the major parts of block import.
pub trait Engine : Sync + Send {
	/// The name of this engine.
	fn name(&self) -> &str;
	/// The version of this engine. Should be of the form
	fn version(&self) -> SemanticVersion { SemanticVersion::new(0, 0, 0) }

	/// The number of additional header fields required for this engine.
	fn seal_fields(&self) -> usize { 0 }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> BTreeMap<String, String> { BTreeMap::new() }

	/// Additional information.
	fn additional_params(&self) -> HashMap<String, String> { HashMap::new() }

	/// Get the general parameters of the chain.
	fn params(&self) -> &CommonParams;

	/// Get the EVM schedule for the given `block_number`.
	fn schedule(&self, block_number: BlockNumber) -> Schedule;

	/// Builtin-contracts we would like to see in the chain.
	/// (In principle these are just hints for the engine since that has the last word on them.)
	fn builtins(&self) -> &BTreeMap<Address, Builtin>;

	/// Some intrinsic operation parameters; by default they take their value from the `spec()`'s `engine_params`.
	fn maximum_extra_data_size(&self) -> usize { self.params().maximum_extra_data_size }
	/// Maximum number of uncles a block is allowed to declare.
	fn maximum_uncle_count(&self) -> usize { 2 }
	/// The number of generations back that uncles can be.
	fn maximum_uncle_age(&self) -> usize { 6 }
	/// The nonce with which accounts begin.
	fn account_start_nonce(&self) -> U256 { self.params().account_start_nonce }

	/// Block transformation functions, before the transactions.
	fn on_new_block(&self, block: &mut ExecutedBlock, last_hashes: Arc<LastHashes>) -> Result<(), Error> {
		let parent_hash = block.fields().header.parent_hash().clone();
		common::push_last_hash(block, last_hashes, self, &parent_hash)
	}

	/// Block transformation functions, after the transactions.
	fn on_close_block(&self, _block: &mut ExecutedBlock) -> Result<(), Error> {
		Ok(())
	}

	/// None means that it requires external input (e.g. PoW) to seal a block.
	/// Some(true) means the engine is currently prime for seal generation (i.e. node is the current validator).
	/// Some(false) means that the node might seal internally but is not qualified now.
	fn seals_internally(&self) -> Option<bool> { None }
	/// Attempt to seal the block internally.
	///
	/// If `Some` is returned, then you get a valid seal.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which None will
	/// be returned.
	fn generate_seal(&self, _block: &ExecutedBlock) -> Seal { Seal::None }

	/// Phase 1 quick block verification. Only does checks that are cheap. `block` (the header's full block)
	/// may be provided for additional checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_basic(&self, _header: &Header,  _block: Option<&[u8]>) -> Result<(), Error> { Ok(()) }

	/// Phase 2 verification. Perform costly checks such as transaction signatures. `block` (the header's full block)
	/// may be provided for additional checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_unordered(&self, _header: &Header, _block: Option<&[u8]>) -> Result<(), Error> { Ok(()) }

	/// Phase 3 verification. Check block information against parent and uncles. `block` (the header's full block)
	/// may be provided for additional checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_family(&self, _header: &Header, _parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> { Ok(()) }

	/// Phase 4 verification. Verify block header against potentially external data.
	fn verify_block_external(&self, _header: &Header, _block: Option<&[u8]>) -> Result<(), Error> { Ok(()) }

	/// Additional verification for transactions in blocks.
	// TODO: Add flags for which bits of the transaction to check.
	// TODO: consider including State in the params.
	fn verify_transaction_basic(&self, t: &UnverifiedTransaction, _header: &Header) -> Result<(), Error> {
		t.verify_basic(true, Some(self.params().network_id), true)?;
		Ok(())
	}

	/// Verify a particular transaction is valid.
	fn verify_transaction(&self, t: UnverifiedTransaction, _header: &Header) -> Result<SignedTransaction, Error> {
		SignedTransaction::new(t)
	}

	/// The network ID that transactions should be signed with.
	fn signing_network_id(&self, _env_info: &EnvInfo) -> Option<u64> {
		Some(self.params().chain_id)
	}

	/// Verify the seal of a block. This is an auxilliary method that actually just calls other `verify_` methods
	/// to get the job done. By default it must pass `verify_basic` and `verify_block_unordered`. If more or fewer
	/// methods are needed for an Engine, this may be overridden.
	fn verify_block_seal(&self, header: &Header) -> Result<(), Error> {
		self.verify_block_basic(header, None).and_then(|_| self.verify_block_unordered(header, None))
	}

	/// Generate epoch change proof.
	///
	/// This will be used to generate proofs of epoch change as well as verify them.
	/// Must be called on blocks that have already passed basic verification.
	///
	/// Return the "epoch proof" generated.
	/// This must be usable to generate a `EpochVerifier` for verifying all blocks
	/// from the supplied header up to the next one where proof is required.
	///
	/// For example, for PoA chains the proof will be a validator set,
	/// and the corresponding `EpochVerifier` can be used to correctly validate
	/// all blocks produced under that `ValidatorSet`
	///
	/// It must be possible to generate an epoch proof for any block in an epoch,
	/// and it should always be equivalent to the proof of the transition block.
	fn epoch_proof(&self, _header: &Header, _caller: &Call)
		-> Result<Vec<u8>, Error>
	{
		Ok(Vec::new())
	}

	/// Whether an epoch change occurred at the given header.
	///
	/// If the block or receipts are required, return `Unsure` and the function will be
	/// called again with them.
	/// Return `Yes` or `No` when the answer is definitively known.
	///
	/// Should not interact with state.
	fn is_epoch_end(&self, _header: &Header, _block: Option<&[u8]>, _receipts: Option<&[Receipt]>)
		-> EpochChange
	{
		EpochChange::No
	}

	/// Create an epoch verifier from validation proof.
	///
	/// The proof should be one generated by `epoch_proof`.
	/// See docs of `epoch_proof` for description.
	fn epoch_verifier(&self, _header: &Header, _proof: &[u8]) -> Result<Box<EpochVerifier>, Error> {
		Ok(Box::new(self::epoch_verifier::NoOp))
	}

	/// Populate a header's fields based on its parent's header.
	/// Usually implements the chain scoring rule based on weight.
	/// The gas floor target must not be lower than the engine's minimum gas limit.
	fn populate_from_parent(&self, header: &mut Header, parent: &Header, _gas_floor_target: U256, _gas_ceil_target: U256) {
		header.set_difficulty(parent.difficulty().clone());
		header.set_gas_limit(parent.gas_limit().clone());
	}

	/// Handle any potential consensus messages;
	/// updating consensus state and potentially issuing a new one.
	fn handle_message(&self, _message: &[u8]) -> Result<(), Error> { Err(EngineError::UnexpectedMessage.into()) }

	/// Attempt to get a handle to a built-in contract.
	/// Only returns references to activated built-ins.
	// TODO: builtin contract routing - to do this properly, it will require removing the built-in configuration-reading logic
	// from Spec into here and removing the Spec::builtins field.
	fn builtin(&self, a: &Address, block_number: ::header::BlockNumber) -> Option<&Builtin> {
		self.builtins()
			.get(a)
			.and_then(|b| if b.is_active(block_number) { Some(b) } else { None })
	}

	/// Find out if the block is a proposal block and should not be inserted into the DB.
	/// Takes a header of a fully verified block.
	fn is_proposal(&self, _verified_header: &Header) -> bool { false }

	/// Register an account which signs consensus messages.
	fn set_signer(&self, _account_provider: Arc<AccountProvider>, _address: Address, _password: String) {}

	/// Sign using the EngineSigner, to be used for consensus tx signing.
	fn sign(&self, _hash: H256) -> Result<Signature, Error> { unimplemented!() }

	/// Add Client which can be used for sealing, querying the state and sending messages.
	fn register_client(&self, _client: Weak<Client>) {}

	/// Trigger next step of the consensus engine.
	fn step(&self) {}

	/// Stops any services that the may hold the Engine and makes it safe to drop.
	fn stop(&self) {}

	/// Create a factory for building snapshot chunks and restoring from them.
	/// Returning `None` indicates that this engine doesn't support snapshot creation.
	fn snapshot_components(&self) -> Option<Box<SnapshotComponents>> {
		None
	}

	/// Whether this engine supports warp sync.
	fn supports_warp(&self) -> bool {
		self.snapshot_components().is_some()
	}

	/// Returns new contract address generation scheme at given block number.
	fn create_address_scheme(&self, number: BlockNumber) -> CreateContractAddress {
		if number >= self.params().eip86_transition { CreateContractAddress::FromCodeHash } else { CreateContractAddress::FromSenderAndNonce }
	}
}


/// Common engine utilities
pub mod common {
	use block::ExecutedBlock;
	use env_info::{EnvInfo, LastHashes};
	use error::Error;
	use transaction::SYSTEM_ADDRESS;
	use executive::Executive;
	use types::executed::CallType;
	use action_params::{ActionParams, ActionValue};
	use trace::{NoopTracer, NoopVMTracer};
	use state::Substate;

	use util::*;
	use super::Engine;

	/// Push last known block hash to the state.
	pub fn push_last_hash<E: Engine + ?Sized>(block: &mut ExecutedBlock, last_hashes: Arc<LastHashes>, engine: &E, hash: &H256) -> Result<(), Error> {
		if block.fields().header.number() == engine.params().eip210_transition {
			let state = block.fields_mut().state;
			state.init_code(&engine.params().eip210_contract_address, engine.params().eip210_contract_code.clone())?;
		}
		if block.fields().header.number() >= engine.params().eip210_transition {
			let env_info = {
				let header = block.fields().header;
				EnvInfo {
					number: header.number(),
					author: header.author().clone(),
					timestamp: header.timestamp(),
					difficulty: header.difficulty().clone(),
					last_hashes: last_hashes,
					gas_used: U256::zero(),
					gas_limit: engine.params().eip210_contract_gas,
				}
			};
			let mut state = block.fields_mut().state;
			let contract_address = engine.params().eip210_contract_address;
			let params = ActionParams {
				code_address: contract_address.clone(),
				address: contract_address.clone(),
				sender: SYSTEM_ADDRESS.clone(),
				origin: SYSTEM_ADDRESS.clone(),
				gas: engine.params().eip210_contract_gas,
				gas_price: 0.into(),
				value: ActionValue::Transfer(0.into()),
				code: state.code(&contract_address)?,
				code_hash: state.code_hash(&contract_address)?,
				data: Some(hash.to_vec()),
				call_type: CallType::Call,
			};
			let mut ex = Executive::new(&mut state, &env_info, engine);
			let mut substate = Substate::new();
			let mut output = [];
			if let Err(e) = ex.call(params, &mut substate, BytesRef::Fixed(&mut output), &mut NoopTracer, &mut NoopVMTracer) {
				warn!("Encountered error on updating last hashes: {}", e);
			}
		}
		Ok(())
	}
}
