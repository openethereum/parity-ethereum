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
// TODO: find a way to unify
pub enum EthereumMachine {
	Regular(CommonParams, BTreeMap<Address, Builtin>),
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

	/// Bestow a block reward.
	// TODO: move back to engines since lock rewards are relevant to security
	// in permissionless systems.
	pub fn bestow_block_reward(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		if let EthereumMachine::WithEthashExtensions(ref params, _, ref ext) = *self {
			return apply_ethash_reward(block, params, ext);
		}

		let fields = block.fields_mut();

		// Bestow block reward
		let reward = self.params().block_reward;
		let res = fields.state.add_balance(fields.header.author(), &reward, CleanupMode::NoEmpty)
			.map_err(::error::Error::from)
			.and_then(|_| fields.state.commit());

		let block_author = fields.header.author().clone();
		fields.traces.as_mut().map(move |mut traces| {
  			let mut tracer = ExecutiveTracer::default();
  			tracer.trace_reward(block_author, reward, RewardType::Block);
  			traces.push(tracer.drain())
		});

		// Commit state so that we can actually figure out the state root.
		if let Err(ref e) = res {
			warn!("Encountered error on bestowing reward: {}", e);
		}
		res
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
	type State = ();
	type Error = ();
}

fn apply_ethash_reward(block: &mut ExecutedBlock, params: &CommonParams, ethash_params: &EthashExtensions) -> Result<(), Error> {
	use std::ops::Shr;
	use block::IsBlock;

	let reward = params.block_reward;
	let tracing_enabled = block.tracing_enabled();
	let fields = block.fields_mut();
	let eras_rounds = ethash_params.ecip1017_era_rounds;
	let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, reward, fields.header.number());
	let mut tracer = ExecutiveTracer::default();

	// Bestow block reward
	let result_block_reward = reward + reward.shr(5) * U256::from(fields.uncles.len());
	fields.state.add_balance(
		fields.header.author(),
		&result_block_reward,
		CleanupMode::NoEmpty
	)?;

	if tracing_enabled {
		let block_author = fields.header.author().clone();
		tracer.trace_reward(block_author, result_block_reward, RewardType::Block);
	}

	// Bestow uncle rewards
	// TODO: find a way to bring uncle rewards out of this function without breaking
	// certain PoA networks where uncles were not disallowed or rewarded.
	let current_number = fields.header.number();
	for u in fields.uncles.iter() {
		let uncle_author = u.author().clone();
		let result_uncle_reward: U256;

		if eras == 0 {
			result_uncle_reward = (reward * U256::from(8 + u.number() - current_number)).shr(3);
			fields.state.add_balance(
				u.author(),
				&result_uncle_reward,
				CleanupMode::NoEmpty
			)
		} else {
			result_uncle_reward = reward.shr(5);
			fields.state.add_balance(
				u.author(),
				&result_uncle_reward,
				CleanupMode::NoEmpty
			)
		}?;

		// Trace uncle rewards
		if tracing_enabled {
			tracer.trace_reward(uncle_author, result_uncle_reward, RewardType::Uncle);
		}
	}

	Ok(())
}

fn ecip1017_eras_block_reward(era_rounds: u64, mut reward: U256, block_number:u64) -> (u64, U256) {
	let eras = if block_number != 0 && block_number % era_rounds == 0 {
		block_number / era_rounds - 1
	} else {
		block_number / era_rounds
	};
	for _ in 0..eras {
		reward = reward / U256::from(5) * U256::from(4);
	}
	(eras, reward)
}

#[cfg(test)]
mod tests {
	use util::U256;
	use super::ecip1017_eras_block_reward;

	#[test]
	fn has_valid_ecip1017_eras_block_reward() {
		let eras_rounds = 5000000;

		let start_reward: U256 = "4563918244F40000".parse().unwrap();

		let block_number = 0;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(0, eras);
		assert_eq!(U256::from_str("4563918244F40000").unwrap(), reward);

		let block_number = 5000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(0, eras);
		assert_eq!(U256::from_str("4563918244F40000").unwrap(), reward);

		let block_number = 10000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(1, eras);
		assert_eq!(U256::from_str("3782DACE9D900000").unwrap(), reward);

		let block_number = 20000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(3, eras);
		assert_eq!(U256::from_str("2386F26FC1000000").unwrap(), reward);

		let block_number = 80000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(15, eras);
		assert_eq!(U256::from_str("271000000000000").unwrap(), reward);
	}
}
