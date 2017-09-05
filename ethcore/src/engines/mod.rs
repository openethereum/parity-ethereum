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
mod instant_seal;
mod null_engine;
mod signer;
mod tendermint;
mod transition;
mod validator_set;
mod vote_collector;

pub mod epoch;

pub use self::authority_round::AuthorityRound;
pub use self::basic_authority::BasicAuthority;
pub use self::epoch::{EpochVerifier, Transition as EpochTransition};
pub use self::instant_seal::InstantSeal;
pub use self::null_engine::NullEngine;
pub use self::tendermint::Tendermint;

use std::sync::{Weak, Arc};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

use self::epoch::PendingTransition;

use account_provider::AccountProvider;
use block::ExecutedBlock;
use builtin::Builtin;
use client::Client;
use vm::{EnvInfo, LastHashes, Schedule, CreateContractAddress};
use error::Error;
use header::{Header, BlockNumber};
use receipt::Receipt;
use snapshot::SnapshotComponents;
use spec::CommonParams;
use transaction::{UnverifiedTransaction, SignedTransaction};

use ethkey::Signature;
use bigint::prelude::U256;
use bigint::hash::H256;
use semantic_version::SemanticVersion;
use util::*;
use unexpected::{Mismatch, OutOfBounds};

/// Default EIP-210 contrat code.
/// As defined in https://github.com/ethereum/EIPs/pull/210/commits/9df24a3714af42e3bf350265bdc75b486c909d7f#diff-e02a92c2fb96c1a1bfb05e4c6e2ef5daR49
pub const DEFAULT_BLOCKHASH_CONTRACT: &'static str = "73fffffffffffffffffffffffffffffffffffffffe33141561007a57600143036020526000356101006020510755600061010060205107141561005057600035610100610100602051050761010001555b6000620100006020510714156100755760003561010062010000602051050761020001555b610161565b436000351215801561008c5780610095565b623567e0600035125b9050156100a757600060605260206060f35b610100600035430312156100ca57610100600035075460805260206080f3610160565b62010000600035430312156100e857600061010060003507146100eb565b60005b1561010d576101006101006000350507610100015460a052602060a0f361015f565b63010000006000354303121561012d576000620100006000350714610130565b60005b1561015357610100620100006000350507610200015460c052602060c0f361015e565b600060e052602060e0f35b5b5b5b5b";

/// Voting errors.
#[derive(Debug)]
pub enum EngineError {
	/// Signature or author field does not belong to an authority.
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
	/// Failed system call.
	FailedSystemCall(String),
	/// Requires client ref, but none registered.
	RequiresClient,
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
			FailedSystemCall(ref msg) => format!("Failed to make system call: {}", msg),
			RequiresClient => format!("Call requires client but none registered"),
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
/// Returns the call result and state proof for each call.
pub type Call<'a> = Fn(Address, Bytes) -> Result<(Bytes, Vec<Vec<u8>>), String> + 'a;

/// Type alias for a function we can get headers by hash through.
pub type Headers<'a> = Fn(H256) -> Option<Header> + 'a;

/// Type alias for a function we can query pending transitions by block hash through.
pub type PendingTransitionStore<'a> = Fn(H256) -> Option<PendingTransition> + 'a;

/// Proof generated on epoch change.
pub enum Proof {
	/// Known proof (exctracted from signal)
	Known(Vec<u8>),
	/// Extract proof from caller.
	WithState(Box<Fn(&Call) -> Result<Vec<u8>, String>>),
}

/// Generated epoch verifier.
pub enum ConstructedVerifier<'a> {
	/// Fully trusted verifier.
	Trusted(Box<EpochVerifier>),
	/// Verifier unconfirmed. Check whether given finality proof finalizes given hash
	/// under previous epoch.
	Unconfirmed(Box<EpochVerifier>, &'a [u8], H256),
	/// Error constructing verifier.
	Err(Error),
}

impl<'a> ConstructedVerifier<'a> {
	/// Convert to a result, indicating that any necessary confirmation has been done
	/// already.
	pub fn known_confirmed(self) -> Result<Box<EpochVerifier>, Error> {
		match self {
			ConstructedVerifier::Trusted(v) | ConstructedVerifier::Unconfirmed(v, _, _) => Ok(v),
			ConstructedVerifier::Err(e) => Err(e),
		}
	}
}

/// Results of a query of whether an epoch change occurred at the given block.
pub enum EpochChange {
	/// Cannot determine until more data is passed.
	Unsure(Unsure),
	/// No epoch change.
	No,
	/// The epoch will change, with proof.
	Yes(Proof),
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
	fn schedule(&self, block_number: BlockNumber) -> Schedule {
		self.params().schedule(block_number)
	}

	/// Builtin-contracts we would like to see in the chain.
	/// (In principle these are just hints for the engine since that has the last word on them.)
	fn builtins(&self) -> &BTreeMap<Address, Builtin>;

	/// Some intrinsic operation parameters; by default they take their value from the `spec()`'s `engine_params`.
	fn maximum_extra_data_size(&self) -> usize { self.params().maximum_extra_data_size }
	/// Maximum number of uncles a block is allowed to declare.
	fn maximum_uncle_count(&self) -> usize { 2 }
	/// The number of generations back that uncles can be.
	fn maximum_uncle_age(&self) -> usize { 6 }
	/// The nonce with which accounts begin at given block.
	fn account_start_nonce(&self, block: u64) -> U256 {
		if block >= self.params().dust_protection_transition {
			U256::from(self.params().nonce_cap_increment) * U256::from(block)
		} else {
			self.params().account_start_nonce
		}
	}

	/// Block transformation functions, before the transactions.
	/// `epoch_begin` set to true if this block kicks off an epoch.
	fn on_new_block(
		&self,
		block: &mut ExecutedBlock,
		last_hashes: Arc<LastHashes>,
		_epoch_begin: bool,
	) -> Result<(), Error> {
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
		t.verify_basic(true, Some(self.params().chain_id), true)?;
		Ok(())
	}

	/// Verify a particular transaction is valid.
	fn verify_transaction(&self, t: UnverifiedTransaction, _header: &Header) -> Result<SignedTransaction, Error> {
		SignedTransaction::new(t)
	}

	/// The network ID that transactions should be signed with.
	fn signing_chain_id(&self, _env_info: &EnvInfo) -> Option<u64> {
		Some(self.params().chain_id)
	}

	/// Verify the seal of a block. This is an auxilliary method that actually just calls other `verify_` methods
	/// to get the job done. By default it must pass `verify_basic` and `verify_block_unordered`. If more or fewer
	/// methods are needed for an Engine, this may be overridden.
	fn verify_block_seal(&self, header: &Header) -> Result<(), Error> {
		self.verify_block_basic(header, None).and_then(|_| self.verify_block_unordered(header, None))
	}

	/// Genesis epoch data.
	fn genesis_epoch_data(&self, _header: &Header, _call: &Call) -> Result<Vec<u8>, String> { Ok(Vec::new()) }

	/// Whether an epoch change is signalled at the given header but will require finality.
	/// If a change can be enacted immediately then return `No` from this function but
	/// `Yes` from `is_epoch_end`.
	///
	/// If the block or receipts are required, return `Unsure` and the function will be
	/// called again with them.
	/// Return `Yes` or `No` when the answer is definitively known.
	///
	/// Should not interact with state.
	fn signals_epoch_end(&self, _header: &Header, _block: Option<&[u8]>, _receipts: Option<&[Receipt]>)
		-> EpochChange
	{
		EpochChange::No
	}

	/// Whether a block is the end of an epoch.
	///
	/// This either means that an immediate transition occurs or a block signalling transition
	/// has reached finality. The `Headers` given are not guaranteed to return any blocks
	/// from any epoch other than the current.
	///
	/// Return optional transition proof.
	fn is_epoch_end(
		&self,
		_chain_head: &Header,
		_chain: &Headers,
		_transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		None
	}

	/// Create an epoch verifier from validation proof and a flag indicating
	/// whether finality is required.
	fn epoch_verifier<'a>(&self, _header: &Header, _proof: &'a [u8]) -> ConstructedVerifier<'a> {
		ConstructedVerifier::Trusted(Box::new(self::epoch::NoOp))
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

	/// If this engine supports wasm contracts.
	fn supports_wasm(&self) -> bool {
		self.params().wasm
	}

	/// Returns new contract address generation scheme at given block number.
	fn create_address_scheme(&self, number: BlockNumber) -> CreateContractAddress {
		if number >= self.params().eip86_transition {
			CreateContractAddress::FromCodeHash
		} else {
			CreateContractAddress::FromSenderAndNonce
		}
	}
}

/// Common engine utilities
pub mod common {
	use std::sync::Arc;
	use block::ExecutedBlock;
	use error::Error;
	use transaction::SYSTEM_ADDRESS;
	use executive::Executive;
	use vm::{CallType, ActionParams, ActionValue, EnvInfo, LastHashes};
	use trace::{NoopTracer, NoopVMTracer, Tracer, ExecutiveTracer, RewardType};
	use state::Substate;
	use state::CleanupMode;

	use bigint::prelude::U256;
	use bigint::hash::H256;
	use util::*;
	use super::Engine;

	/// Execute a call as the system address.
	pub fn execute_as_system<E: Engine + ?Sized>(
		block: &mut ExecutedBlock,
		last_hashes: Arc<LastHashes>,
		engine: &E,
		contract_address: Address,
		gas: U256,
		data: Option<Bytes>,
	) -> Result<Bytes, Error> {
		let env_info = {
			let header = block.fields().header;
			EnvInfo {
				number: header.number(),
				author: header.author().clone(),
				timestamp: header.timestamp(),
				difficulty: header.difficulty().clone(),
				last_hashes: last_hashes,
				gas_used: U256::zero(),
				gas_limit: gas,
			}
		};

		let mut state = block.fields_mut().state;
		let params = ActionParams {
			code_address: contract_address.clone(),
			address: contract_address.clone(),
			sender: SYSTEM_ADDRESS.clone(),
			origin: SYSTEM_ADDRESS.clone(),
			gas: gas,
			gas_price: 0.into(),
			value: ActionValue::Transfer(0.into()),
			code: state.code(&contract_address)?,
			code_hash: Some(state.code_hash(&contract_address)?),
			data: data,
			call_type: CallType::Call,
		};
		let mut ex = Executive::new(&mut state, &env_info, engine);
		let mut substate = Substate::new();
		let mut output = Vec::new();
		if let Err(e) = ex.call(params, &mut substate, BytesRef::Flexible(&mut output), &mut NoopTracer, &mut NoopVMTracer) {
			warn!("Encountered error on making system call: {}", e);
		}

		Ok(output)
	}

	/// Push last known block hash to the state.
	pub fn push_last_hash<E: Engine + ?Sized>(block: &mut ExecutedBlock, last_hashes: Arc<LastHashes>, engine: &E, hash: &H256) -> Result<(), Error> {
		if block.fields().header.number() == engine.params().eip210_transition {
			let state = block.fields_mut().state;
			state.init_code(&engine.params().eip210_contract_address, engine.params().eip210_contract_code.clone())?;
		}
		if block.fields().header.number() >= engine.params().eip210_transition {
			let _ = execute_as_system(
				block,
				last_hashes,
				engine,
				engine.params().eip210_contract_address,
				engine.params().eip210_contract_gas,
				Some(hash.to_vec()),
			)?;
		}
		Ok(())
	}

	/// Trace rewards on closing block
	pub fn bestow_block_reward<E: Engine + ?Sized>(block: &mut ExecutedBlock, engine: &E) -> Result<(), Error> {
		let fields = block.fields_mut();
		// Bestow block reward
		let reward = engine.params().block_reward;
		let res = fields.state.add_balance(fields.header.author(), &reward, CleanupMode::NoEmpty)
			.map_err(::error::Error::from)
			.and_then(|_| fields.state.commit());

		let block_author = fields.header.author().clone();
		fields.traces.as_mut().map(|mut traces| {
  			let mut tracer = ExecutiveTracer::default();
  			tracer.trace_reward(block_author, engine.params().block_reward, RewardType::Block);
  			traces.push(tracer.drain())
		});

		// Commit state so that we can actually figure out the state root.
		if let Err(ref e) = res {
			warn!("Encountered error on bestowing reward: {}", e);
		}
		res
	}
}
