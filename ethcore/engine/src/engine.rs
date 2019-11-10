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

//! Consensus engine specification and basic implementations.

use std::sync::{Weak, Arc};
use std::collections::BTreeMap;

use builtin::Builtin;
use common_types::{
	BlockNumber,
	ancestry_action::AncestryAction,
	header::{Header, ExtendedHeader},
	engines::{
		Seal, SealingState, Headers, PendingTransitionStore,
		params::CommonParams,
		machine as machine_types,
		machine::{AuxiliaryData, AuxiliaryRequest},
	},
	errors::{EthcoreError as Error, EngineError},
	snapshot::Snapshotting,
	transaction::{self, UnverifiedTransaction},
};
use client_traits::EngineClient;

use ethereum_types::{H256, U256, Address};
use parity_crypto::publickey::Signature;
use machine::{
	Machine,
	executed_block::ExecutedBlock,
};
use vm::{EnvInfo, Schedule, CallType, ActionValue};

use crate::signer::EngineSigner;

/// A system-calling closure. Enacts calls on a block's state from the system address.
pub type SystemCall<'a> = dyn FnMut(Address, Vec<u8>) -> Result<Vec<u8>, String> + 'a;

/// A system-calling closure. Enacts calls on a block's state with code either from an on-chain contract, or hard-coded EVM or WASM (if enabled on-chain) codes.
pub type SystemOrCodeCall<'a> = dyn FnMut(SystemOrCodeCallKind, Vec<u8>) -> Result<Vec<u8>, String> + 'a;

/// Kind of SystemOrCodeCall, this is either an on-chain address, or code.
#[derive(PartialEq, Debug, Clone)]
pub enum SystemOrCodeCallKind {
	/// On-chain address.
	Address(Address),
	/// Hard-coded code.
	Code(Arc<Vec<u8>>, H256),
}

/// Default SystemOrCodeCall implementation.
pub fn default_system_or_code_call<'a>(machine: &'a Machine, block: &'a mut ExecutedBlock) -> impl FnMut(SystemOrCodeCallKind, Vec<u8>) -> Result<Vec<u8>, String> + 'a {
	move |to, data| {
		let result = match to {
			SystemOrCodeCallKind::Address(address) => {
				machine.execute_as_system(
					block,
					address,
					U256::max_value(),
					Some(data),
				)
			},
			SystemOrCodeCallKind::Code(code, code_hash) => {
				machine.execute_code_as_system(
					block,
					None,
					Some(code),
					Some(code_hash),
					Some(ActionValue::Apparent(U256::zero())),
					U256::max_value(),
					Some(data),
					Some(CallType::StaticCall),
				)
			},
		};

		result.map_err(|e| format!("{}", e))
	}
}

/// Proof dependent on state.
pub trait StateDependentProof: Send + Sync {
	/// Generate a proof, given the state.
	fn generate_proof<'a>(&self, state: &machine_types::Call) -> Result<Vec<u8>, String>;
	/// Check a proof generated elsewhere (potentially by a peer).
	// `engine` needed to check state proofs, while really this should
	// just be state machine params.
	fn check_proof(&self, machine: &Machine, proof: &[u8]) -> Result<(), String>;
}

/// Proof generated on epoch change.
pub enum Proof {
	/// Known proof (extracted from signal)
	Known(Vec<u8>),
	/// State dependent proof.
	WithState(Arc<dyn StateDependentProof>),
}

/// Generated epoch verifier.
pub enum ConstructedVerifier<'a> {
	/// Fully trusted verifier.
	Trusted(Box<dyn EpochVerifier>),
	/// Verifier unconfirmed. Check whether given finality proof finalizes given hash
	/// under previous epoch.
	Unconfirmed(Box<dyn EpochVerifier>, &'a [u8], H256),
	/// Error constructing verifier.
	Err(Error),
}

impl<'a> ConstructedVerifier<'a> {
	/// Convert to a result, indicating that any necessary confirmation has been done
	/// already.
	pub fn known_confirmed(self) -> Result<Box<dyn EpochVerifier>, Error> {
		match self {
			ConstructedVerifier::Trusted(v) | ConstructedVerifier::Unconfirmed(v, _, _) => Ok(v),
			ConstructedVerifier::Err(e) => Err(e),
		}
	}
}

/// Results of a query of whether an epoch change occurred at the given block.
pub enum EpochChange {
	/// Cannot determine until more data is passed.
	Unsure(AuxiliaryRequest),
	/// No epoch change.
	No,
	/// The epoch will change, with proof.
	Yes(Proof),
}

/// A consensus mechanism for the chain. Generally either proof-of-work or proof-of-stake-based.
/// Provides hooks into each of the major parts of block import.
pub trait Engine: Sync + Send {
	/// The name of this engine.
	fn name(&self) -> &str;

	/// Get access to the underlying state machine.
	// TODO: decouple.
	fn machine(&self) -> &Machine;

	/// The number of additional header fields required for this engine.
	fn seal_fields(&self, _header: &Header) -> usize { 0 }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> BTreeMap<String, String> { BTreeMap::new() }

	/// Maximum number of uncles a block is allowed to declare.
	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 0 }

	/// Optional maximum gas limit.
	fn maximum_gas_limit(&self) -> Option<U256> { None }

	/// Block transformation functions, before the transactions.
	/// `epoch_begin` set to true if this block kicks off an epoch.
	fn on_new_block(
		&self,
		_block: &mut ExecutedBlock,
		_epoch_begin: bool,
	) -> Result<(), Error> {
		Ok(())
	}

	/// Block transformation functions, after the transactions.
	fn on_close_block(
		&self,
		_block: &mut ExecutedBlock,
		_parent_header: &Header,
	) -> Result<(), Error> {
		Ok(())
	}

	/// Allow mutating the header during seal generation. Currently only used by Clique.
	fn on_seal_block(&self, _block: &mut ExecutedBlock) -> Result<(), Error> { Ok(()) }

	/// Returns the engine's current sealing state.
	fn sealing_state(&self) -> SealingState { SealingState::External }

	/// Called in `miner.chain_new_blocks` if the engine wishes to `update_sealing`
	/// after a block was recently sealed.
	///
	/// returns false by default
	fn should_reseal_on_update(&self) -> bool {
		false
	}

	/// Attempt to seal the block internally.
	///
	/// If `Some` is returned, then you get a valid seal.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which None will
	/// be returned.
	///
	/// It is fine to require access to state or a full client for this function, since
	/// light clients do not generate seals.
	fn generate_seal(&self, _block: &ExecutedBlock, _parent: &Header) -> Seal { Seal::None }

	/// Verify a locally-generated seal of a header.
	///
	/// If this engine seals internally,
	/// no checks have to be done here, since all internally generated seals
	/// should be valid.
	///
	/// Externally-generated seals (e.g. PoW) will need to be checked for validity.
	///
	/// It is fine to require access to state or a full client for this function, since
	/// light clients do not generate seals.
	fn verify_local_seal(&self, header: &Header) -> Result<(), Error>;

	/// Phase 1 quick block verification. Only does checks that are cheap. Returns either a null `Ok` or a general error detailing the problem with import.
	/// The verification module can optionally avoid checking the seal (`check_seal`), if seal verification is disabled this method won't be called.
	fn verify_block_basic(&self, _header: &Header) -> Result<(), Error> { Ok(()) }

	/// Phase 2 verification. Perform costly checks such as transaction signatures. Returns either a null `Ok` or a general error detailing the problem with import.
	/// The verification module can optionally avoid checking the seal (`check_seal`), if seal verification is disabled this method won't be called.
	fn verify_block_unordered(&self, _header: &Header) -> Result<(), Error> { Ok(()) }

	/// Phase 3 verification. Check block information against parent. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_family(&self, _header: &Header, _parent: &Header) -> Result<(), Error> { Ok(()) }

	/// Phase 4 verification. Verify block header against potentially external data.
	/// Should only be called when `register_client` has been called previously.
	fn verify_block_external(&self, _header: &Header) -> Result<(), Error> { Ok(()) }

	/// Genesis epoch data.
	fn genesis_epoch_data<'a>(&self, _header: &Header, _state: &machine_types::Call) -> Result<Vec<u8>, String> { Ok(Vec::new()) }

	/// Whether an epoch change is signalled at the given header but will require finality.
	/// If a change can be enacted immediately then return `No` from this function but
	/// `Yes` from `is_epoch_end`.
	///
	/// If auxiliary data of the block is required, return an auxiliary request and the function will be
	/// called again with them.
	/// Return `Yes` or `No` when the answer is definitively known.
	///
	/// Should not interact with state.
	fn signals_epoch_end<'a>(&self, _header: &Header, _aux: AuxiliaryData<'a>) -> EpochChange {
		EpochChange::No
	}

	/// Whether a block is the end of an epoch.
	///
	/// This either means that an immediate transition occurs or a block signalling transition
	/// has reached finality. The `Headers` given are not guaranteed to return any blocks
	/// from any epoch other than the current. The client must keep track of finality and provide
	/// the latest finalized headers to check against the transition store.
	///
	/// Return optional transition proof.
	fn is_epoch_end(
		&self,
		_chain_head: &Header,
		_finalized: &[H256],
		_chain: &Headers<Header>,
		_transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		None
	}

	/// Whether a block is the end of an epoch.
	///
	/// This either means that an immediate transition occurs or a block signalling transition
	/// has reached finality. The `Headers` given are not guaranteed to return any blocks
	/// from any epoch other than the current. This is a specialized method to use for light
	/// clients since the light client doesn't track finality of all blocks, and therefore finality
	/// for blocks in the current epoch is built inside this method by the engine.
	///
	/// Return optional transition proof.
	fn is_epoch_end_light(
		&self,
		_chain_head: &Header,
		_chain: &Headers<Header>,
		_transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		None
	}

	/// Create an epoch verifier from validation proof and a flag indicating
	/// whether finality is required.
	fn epoch_verifier<'a>(&self, _header: &Header, _proof: &'a [u8]) -> ConstructedVerifier<'a> {
		ConstructedVerifier::Trusted(Box::new(NoOp))
	}

	/// Populate a header's fields based on its parent's header.
	/// Usually implements the chain scoring rule based on weight.
	fn populate_from_parent(&self, _header: &mut Header, _parent: &Header) { }

	/// Handle any potential consensus messages;
	/// updating consensus state and potentially issuing a new one.
	fn handle_message(&self, _message: &[u8]) -> Result<(), EngineError> { Err(EngineError::UnexpectedMessage) }

	/// Register a component which signs consensus messages.
	fn set_signer(&self, _signer: Option<Box<dyn EngineSigner>>) {}

	/// Sign using the EngineSigner, to be used for consensus tx signing.
	fn sign(&self, _hash: H256) -> Result<Signature, Error> { unimplemented!() }

	/// Add Client which can be used for sealing, potentially querying the state and sending messages.
	fn register_client(&self, _client: Weak<dyn EngineClient>) {}

	/// Trigger next step of the consensus engine.
	fn step(&self) {}

	/// Snapshot mode for the engine: Unsupported, PoW or PoA
	fn snapshot_mode(&self) -> Snapshotting { Snapshotting::Unsupported }

	/// Return a new open block header timestamp based on the parent timestamp.
	fn open_block_header_timestamp(&self, parent_timestamp: u64) -> u64 {
		use std::{time, cmp};

		let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap_or_default();
		cmp::max(now.as_secs() as u64, parent_timestamp + 1)
	}

	/// Check whether the parent timestamp is valid.
	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp > parent_timestamp
	}

	/// Gather all ancestry actions. Called at the last stage when a block is committed. The Engine must guarantee that
	/// the ancestry exists.
	fn ancestry_actions(&self, _header: &Header, _ancestry: &mut dyn Iterator<Item = ExtendedHeader>) -> Vec<AncestryAction> {
		Vec::new()
	}

	/// Returns author should used when executing tx's for this block.
	fn executive_author(&self, header: &Header) -> Result<Address, Error> {
		Ok(*header.author())
	}

	/// Get the general parameters of the chain.
	fn params(&self) -> &CommonParams;

	/// Get the EVM schedule for the given block number.
	fn schedule(&self, block_number: BlockNumber) -> Schedule {
		self.machine().schedule(block_number)
	}

	/// Builtin-contracts for the chain..
	fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		self.machine().builtins()
	}

	/// Attempt to get a handle to a built-in contract.
	/// Only returns references to activated built-ins.
	fn builtin(&self, a: &Address, block_number: BlockNumber) -> Option<&Builtin> {
		self.machine().builtin(a, block_number)
	}

	/// Some intrinsic operation parameters; by default they take their value from the `spec()`'s `engine_params`.
	fn maximum_extra_data_size(&self) -> usize { self.params().maximum_extra_data_size }

	/// The nonce with which accounts begin at given block.
	fn account_start_nonce(&self, block: BlockNumber) -> U256 {
		self.machine().account_start_nonce(block)
	}

	/// The network ID that transactions should be signed with.
	fn signing_chain_id(&self, env_info: &EnvInfo) -> Option<u64> {
		self.machine().signing_chain_id(env_info)
	}

	/// Perform basic/cheap transaction verification.
	///
	/// This should include all cheap checks that can be done before
	/// actually checking the signature, like chain-replay protection.
	///
	/// NOTE This is done before the signature is recovered so avoid
	/// doing any state-touching checks that might be expensive.
	///
	/// TODO: Add flags for which bits of the transaction to check.
	/// TODO: consider including State in the params.
	fn verify_transaction_basic(&self, t: &UnverifiedTransaction, header: &Header) -> Result<(), transaction::Error> {
		self.machine().verify_transaction_basic(t, header)
	}

	/// Performs pre-validation of RLP decoded transaction before other processing
	fn decode_transaction(&self, transaction: &[u8]) -> Result<UnverifiedTransaction, transaction::Error> {
		self.machine().decode_transaction(transaction)
	}
}

/// Verifier for all blocks within an epoch with self-contained state.
pub trait EpochVerifier: Send + Sync {
	/// Lightly verify the next block header.
	/// This may not be a header belonging to a different epoch.
	fn verify_light(&self, _header: &Header) -> Result<(), Error> { Ok(()) }

	/// Perform potentially heavier checks on the next block header.
	fn verify_heavy(&self, header: &Header) -> Result<(), Error> {
		self.verify_light(header)
	}

	/// Check a finality proof against this epoch verifier.
	/// Returns `Some(hashes)` if the proof proves finality of these hashes.
	/// Returns `None` if the proof doesn't prove anything.
	fn check_finality_proof(&self, _proof: &[u8]) -> Option<Vec<H256>> {
		None
	}
}

/// Special "no-op" verifier for stateless, epoch-less engines.
pub struct NoOp;

impl EpochVerifier for NoOp {}
