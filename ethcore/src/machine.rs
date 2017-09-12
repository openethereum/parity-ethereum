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

//! Ethereum-like state machine definition.

use std::collections::BTreeMap;
use std::sync::Arc;

use block::ExecutedBlock;
use builtin::Builtin;
use error::Error;
use executive::Executive;
use header::{BlockNumber, Header};
use spec::CommonParams;
use state::{CleanupMode, Substate};
use trace::{NoopTracer, NoopVMTracer, Tracer, ExecutiveTracer, RewardType};
use transaction::{SYSTEM_ADDRESS, UnverifiedTransaction, SignedTransaction};

use util::{Address, BytesRef, U256, H256};
use vm::{CallType, ActionParams, ActionValue, LastHashes};
use vm::{EnvInfo, Schedule, CreateContractAddress};

pub struct EthashExtensions {
	/// Homestead transition block number.
	pub homestead_transition: BlockNumber,
	/// EIP150 transition block number.
	pub eip150_transition: BlockNumber,
	/// Number of first block where EIP-160 rules begin.
	pub eip160_transition: u64,
	/// Number of first block where EIP-161.abc begin.
	pub eip161abc_transition: u64,
	/// Number of first block where EIP-161.d begins.
	pub eip161d_transition: u64,
	/// DAO hard-fork transition block (X).
	pub dao_hardfork_transition: u64,
	/// DAO hard-fork refund contract address (C).
	pub dao_hardfork_beneficiary: Address,
	/// DAO hard-fork DAO accounts list (L)
	pub dao_hardfork_accounts: Vec<Address>,
	/// Total block number for one ECIP-1017 era.
	pub ecip1017_era_rounds: u64,
}

/// An ethereum-like state machine.
pub enum EthereumMachine {
	/// Regular ethereum-like state machine
	Regular(CommonParams, BTreeMap<Address, Builtin>),
	/// A machine with Ethash extensions.
	// TODO: unify with regular. at the time of writing, we've only had to do
	// significant hard forks for ethash engines, but we don't want to end up
	// with a variant for each consensus mode.
	WithEthashExtensions(CommonParams, BTreeMap<Address, Builtin>, EthashExtensions),
}

impl EthereumMachine {
	/// Execute a call as the system address.
	pub fn execute_as_system(
		&self,
		block: &mut ExecutedBlock,
		last_hashes: Arc<LastHashes>,
		contract_address: Address,
		gas: U256,
		data: Option<Vec<u8>>,
	) -> Result<Vec<u8>, Error> {
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
		let mut ex = Executive::new(&mut state, &env_info, self);
		let mut substate = Substate::new();
		let mut output = Vec::new();
		if let Err(e) = ex.call(params, &mut substate, BytesRef::Flexible(&mut output), &mut NoopTracer, &mut NoopVMTracer) {
			warn!("Encountered error on making system call: {}", e);
		}

		Ok(output)
	}

	/// Push last known block hash to the state.
	pub fn push_last_hash(&self, block: &mut ExecutedBlock, last_hashes: Arc<LastHashes>, hash: &H256) -> Result<(), Error> {
		let params = self.params();
		if block.fields().header.number() == params.eip210_transition {
			let state = block.fields_mut().state;
			state.init_code(&params.eip210_contract_address, params.eip210_contract_code.clone())?;
		}
		if block.fields().header.number() >= params.eip210_transition {
			let _ = self.execute_as_system(
				block,
				last_hashes,
				params.eip210_contract_address,
				params.eip210_contract_gas,
				Some(hash.to_vec()),
			)?;
		}
		Ok(())
	}

	/// Get the general parameters of the chain.
	pub fn params(&self) -> &CommonParams {
		match *self {
			EthereumMachine::Regular(ref params, _) => params,
			EthereumMachine::WithEthashExtensions(ref params, _, _) => params,
		}
	}

	/// Get the EVM schedule for the given block number.
	pub fn schedule(&self, block_number: BlockNumber) -> Schedule {
		match *self {
			EthereumMachine::Regular(ref params, _) => params.schedule(block_number),
			EthereumMachine::WithEthashExtensions(ref params, _, ref ext) => {
				if block_number < ext.homestead_transition {
					Schedule::new_frontier()
				} else if block_number < ext.eip150_transition {
					Schedule::new_homestead()
				} else {
					let mut schedule = Schedule::new_post_eip150(
						params.max_code_size as _,
						block_number >= ext.eip160_transition,
						block_number >= ext.eip161abc_transition,
						block_number >= ext.eip161d_transition
					);

					params.update_schedule(block_number, &mut schedule);
					schedule
				}
			}
		}
	}

	/// Builtin-contracts for the chain..
	pub fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		match *self {
			EthereumMachine::Regular(_, ref builtins) => builtins,
			EthereumMachine::WithEthashExtensions(_, ref builtins, _) => builtins,
		}
	}

	/// Attempt to get a handle to a built-in contract.
	/// Only returns references to activated built-ins.
	// TODO: builtin contract routing - to do this properly, it will require removing the built-in configuration-reading logic
	// from Spec into here and removing the Spec::builtins field.
	pub fn builtin(&self, a: &Address, block_number: BlockNumber) -> Option<&Builtin> {
		self.builtins()
			.get(a)
			.and_then(|b| if b.is_active(block_number) { Some(b) } else { None })
	}

	/// Some intrinsic operation parameters; by default they take their value from the `spec()`'s `engine_params`.
	pub fn maximum_extra_data_size(&self) -> usize { self.params().maximum_extra_data_size }

	/// The nonce with which accounts begin at given block.
	pub fn account_start_nonce(&self, block: u64) -> U256 {
		let params = self.params();

		if block >= params.dust_protection_transition {
			U256::from(params.nonce_cap_increment) * U256::from(block)
		} else {
			params.account_start_nonce
		}
	}

	/// The network ID that transactions should be signed with.
	pub fn signing_chain_id(&self, env_info: &EnvInfo) -> Option<u64> {
		let params = self.params();

		if env_info.number >= params.eip155_transition {
			Some(params.chain_id)
		} else {
			None
		}
	}

	/// Returns new contract address generation scheme at given block number.
	pub fn create_address_scheme(&self, number: BlockNumber) -> CreateContractAddress {
		if number >= self.params().eip86_transition {
			CreateContractAddress::FromCodeHash
		} else {
			CreateContractAddress::FromSenderAndNonce
		}
	}

	/// Verify a particular transaction is valid.
	pub fn verify_transaction(&self, t: UnverifiedTransaction, _header: &Header) -> Result<SignedTransaction, Error> {
		SignedTransaction::new(t)
	}

	pub fn verify_transaction_basic(&self, t: &UnverifiedTransaction, header: &Header) -> Result<(), Error> {
		use error::TransactionError;

		t.check_low_s()?;

		if let Some(n) = t.chain_id() {
			if header.number() >= self.params().eip155_transition && n != self.params().chain_id {
				return Err(TransactionError::InvalidChainId.into());
			}
		}

		Ok(())
	}

	/// If this machine supports wasm.
	pub fn supports_wasm(&self) -> bool {
		self.params().wasm
	}
}

impl ::parity_machine::Machine for EthereumMachine {
	type Header = Header;
	type LiveBlock = ExecutedBlock;
	type Error = Error;
}

impl ::parity_machine::WithBalances for EthereumMachine {
	fn balance(&self, live: &ExecutedBlock, address: &Address) -> Result<U256, Error> {
		live.fields().state.balance(address).map_err(Into::into)
	}

	fn add_balance(&self, live: &mut ExecutedBlock, address: &Address, amount: &U256) -> Result<(), Error> {
		live.fields_mut().state.add_balance(address, amount, CleanupMode::NoEmpty).map_err(Into::into)
	}

	fn note_rewards(
		&self,
		live: &mut Self::LiveBlock,
		direct: &[(Address, U256)],
		indirect: &[(Address, U256)],
	) -> Result<(), Self::Error> {
		use block::IsBlock;

		if !live.tracing_enabled() { return Ok(()) }

		let mut tracer = ExecutiveTracer::default();

		for &(address, amount) in direct {
			tracer.trace_reward(address, amount, RewardType::Block);
		}

		for &(address, amount) in indirect {
			tracer.trace_reward(address, amount, RewardType::Uncle);
		}

		live.fields_mut().push_traces(tracer);

		Ok(())
	}
}
