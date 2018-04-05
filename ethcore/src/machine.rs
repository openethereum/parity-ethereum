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

use std::collections::{BTreeMap, HashMap};
use std::cmp;
use std::sync::Arc;

use block::{ExecutedBlock, IsBlock};
use builtin::Builtin;
use client::{BlockInfo, CallContract};
use error::Error;
use executive::Executive;
use header::{BlockNumber, Header};
use spec::CommonParams;
use state::{CleanupMode, Substate};
use trace::{NoopTracer, NoopVMTracer, Tracer, ExecutiveTracer, RewardType, Tracing};
use transaction::{self, SYSTEM_ADDRESS, UnverifiedTransaction, SignedTransaction};
use tx_filter::TransactionFilter;

use ethereum_types::{U256, Address};
use bytes::BytesRef;
use vm::{CallType, ActionParams, ActionValue, ParamsType};
use vm::{EnvInfo, Schedule, CreateContractAddress};

/// Parity tries to round block.gas_limit to multiple of this constant
pub const PARITY_GAS_LIMIT_DETERMINANT: U256 = U256([37, 0, 0, 0]);

/// Ethash-specific extensions.
#[derive(Debug, Clone)]
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
}

impl From<::ethjson::spec::EthashParams> for EthashExtensions {
	fn from(p: ::ethjson::spec::EthashParams) -> Self {
		EthashExtensions {
			homestead_transition: p.homestead_transition.map_or(0, Into::into),
			eip150_transition: p.eip150_transition.map_or(0, Into::into),
			eip160_transition: p.eip160_transition.map_or(0, Into::into),
			eip161abc_transition: p.eip161abc_transition.map_or(0, Into::into),
			eip161d_transition: p.eip161d_transition.map_or(u64::max_value(), Into::into),
			dao_hardfork_transition: p.dao_hardfork_transition.map_or(u64::max_value(), Into::into),
			dao_hardfork_beneficiary: p.dao_hardfork_beneficiary.map_or_else(Address::new, Into::into),
			dao_hardfork_accounts: p.dao_hardfork_accounts.unwrap_or_else(Vec::new).into_iter().map(Into::into).collect(),
		}
	}
}

/// Special rules to be applied to the schedule.
pub type ScheduleCreationRules = Fn(&mut Schedule, BlockNumber) + Sync + Send;

/// An ethereum-like state machine.
pub struct EthereumMachine {
	params: CommonParams,
	builtins: Arc<BTreeMap<Address, Builtin>>,
	tx_filter: Option<Arc<TransactionFilter>>,
	ethash_extensions: Option<EthashExtensions>,
	schedule_rules: Option<Box<ScheduleCreationRules>>,
}

impl EthereumMachine {
	/// Regular ethereum machine.
	pub fn regular(params: CommonParams, builtins: BTreeMap<Address, Builtin>) -> EthereumMachine {
		let tx_filter = TransactionFilter::from_params(&params).map(Arc::new);
		EthereumMachine {
			params: params,
			builtins: Arc::new(builtins),
			tx_filter: tx_filter,
			ethash_extensions: None,
			schedule_rules: None,
		}
	}

	/// Ethereum machine with ethash extensions.
	// TODO: either unify or specify to mainnet specifically and include other specific-chain HFs?
	pub fn with_ethash_extensions(params: CommonParams, builtins: BTreeMap<Address, Builtin>, extensions: EthashExtensions) -> EthereumMachine {
		let mut machine = EthereumMachine::regular(params, builtins);
		machine.ethash_extensions = Some(extensions);
		machine
	}

	/// Attach special rules to the creation of schedule.
	pub fn set_schedule_creation_rules(&mut self, rules: Box<ScheduleCreationRules>) {
		self.schedule_rules = Some(rules);
	}

	/// Get a reference to the ethash-specific extensions.
	pub fn ethash_extensions(&self) -> Option<&EthashExtensions> {
		self.ethash_extensions.as_ref()
	}
}

impl EthereumMachine {
	/// Execute a call as the system address.
	pub fn execute_as_system(
		&self,
		block: &mut ExecutedBlock,
		contract_address: Address,
		gas: U256,
		data: Option<Vec<u8>>,
	) -> Result<Vec<u8>, Error> {
		let env_info = {
			let mut env_info = block.env_info();
			env_info.gas_limit = env_info.gas_used + gas;
			env_info
		};

		let mut state = block.state_mut();
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
			params_type: ParamsType::Separate,
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
	fn push_last_hash(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		let params = self.params();
		if block.header().number() == params.eip210_transition {
			let state = block.state_mut();
			state.init_code(&params.eip210_contract_address, params.eip210_contract_code.clone())?;
		}
		if block.header().number() >= params.eip210_transition {
			let parent_hash = block.header().parent_hash().clone();
			let _ = self.execute_as_system(
				block,
				params.eip210_contract_address,
				params.eip210_contract_gas,
				Some(parent_hash.to_vec()),
			)?;
		}
		Ok(())
	}

	/// Logic to perform on a new block: updating last hashes and the DAO
	/// fork, for ethash.
	pub fn on_new_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		self.push_last_hash(block)?;

		if let Some(ref ethash_params) = self.ethash_extensions {
			if block.header().number() == ethash_params.dao_hardfork_transition {
				let state = block.state_mut();
				for child in &ethash_params.dao_hardfork_accounts {
					let beneficiary = &ethash_params.dao_hardfork_beneficiary;
					state.balance(child)
						.and_then(|b| state.transfer_balance(child, beneficiary, &b, CleanupMode::NoEmpty))?;
				}
			}
		}

		Ok(())
	}

	/// Populate a header's fields based on its parent's header.
	/// Usually implements the chain scoring rule based on weight.
	/// The gas floor target must not be lower than the engine's minimum gas limit.
	pub fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256, gas_ceil_target: U256) {
		header.set_difficulty(parent.difficulty().clone());
		let gas_limit = parent.gas_limit().clone();
		assert!(!gas_limit.is_zero(), "Gas limit should be > 0");

		if let Some(ref ethash_params) = self.ethash_extensions {
			let gas_limit = {
				let bound_divisor = self.params().gas_limit_bound_divisor;
				let lower_limit = gas_limit - gas_limit / bound_divisor + 1.into();
				let upper_limit = gas_limit + gas_limit / bound_divisor - 1.into();
				let gas_limit = if gas_limit < gas_floor_target {
					let gas_limit = cmp::min(gas_floor_target, upper_limit);
					round_block_gas_limit(gas_limit, lower_limit, upper_limit)
				} else if gas_limit > gas_ceil_target {
					let gas_limit = cmp::max(gas_ceil_target, lower_limit);
					round_block_gas_limit(gas_limit, lower_limit, upper_limit)
				} else {
					let total_lower_limit = cmp::max(lower_limit, gas_floor_target);
					let total_upper_limit = cmp::min(upper_limit, gas_ceil_target);
					let gas_limit = cmp::max(gas_floor_target, cmp::min(total_upper_limit,
						lower_limit + (header.gas_used().clone() * 6u32 / 5.into()) / bound_divisor));
					round_block_gas_limit(gas_limit, total_lower_limit, total_upper_limit)
				};
				// ensure that we are not violating protocol limits
				debug_assert!(gas_limit >= lower_limit);
				debug_assert!(gas_limit <= upper_limit);
				gas_limit
			};

			header.set_gas_limit(gas_limit);
			if header.number() >= ethash_params.dao_hardfork_transition &&
				header.number() <= ethash_params.dao_hardfork_transition + 9 {
				header.set_extra_data(b"dao-hard-fork"[..].to_owned());
			}
			return
		}

		header.set_gas_limit({
			let bound_divisor = self.params().gas_limit_bound_divisor;
			if gas_limit < gas_floor_target {
				cmp::min(gas_floor_target, gas_limit + gas_limit / bound_divisor - 1.into())
			} else {
				cmp::max(gas_floor_target, gas_limit - gas_limit / bound_divisor + 1.into())
			}
		});
	}

	/// Get the general parameters of the chain.
	pub fn params(&self) -> &CommonParams {
		&self.params
	}

	/// Get the EVM schedule for the given block number.
	pub fn schedule(&self, block_number: BlockNumber) -> Schedule {
		let mut schedule = match self.ethash_extensions {
			None => self.params.schedule(block_number),
			Some(ref ext) => {
				if block_number < ext.homestead_transition {
					Schedule::new_frontier()
				} else if block_number < ext.eip150_transition {
					Schedule::new_homestead()
				} else {
					let max_code_size = self.params.max_code_size(block_number);
					let mut schedule = Schedule::new_post_eip150(
						max_code_size as _,
						block_number >= ext.eip160_transition,
						block_number >= ext.eip161abc_transition,
						block_number >= ext.eip161d_transition
					);

					self.params.update_schedule(block_number, &mut schedule);
					schedule
				}
			}
		};

		if let Some(ref rules) = self.schedule_rules {
			(rules)(&mut schedule, block_number)
		}

		schedule
	}

	/// Builtin-contracts for the chain..
	pub fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		&*self.builtins
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

	/// Verify a particular transaction is valid, regardless of order.
	pub fn verify_transaction_unordered(&self, t: UnverifiedTransaction, _header: &Header) -> Result<SignedTransaction, Error> {
		Ok(SignedTransaction::new(t)?)
	}

	/// Does basic verification of the transaction.
	pub fn verify_transaction_basic(&self, t: &UnverifiedTransaction, header: &Header) -> Result<(), Error> {
		let check_low_s = match self.ethash_extensions {
			Some(ref ext) => header.number() >= ext.homestead_transition,
			None => true,
		};

		let chain_id = if header.number() < self.params().validate_chain_id_transition {
			t.chain_id()
		} else if header.number() >= self.params().eip155_transition {
			Some(self.params().chain_id)
		} else {
			None
		};
		t.verify_basic(check_low_s, chain_id, false)?;

		Ok(())
	}

	/// Does verification of the transaction against the parent state.
	// TODO: refine the bound here to be a "state provider" or similar as opposed
	// to full client functionality.
	pub fn verify_transaction<C: BlockInfo + CallContract>(&self, t: &SignedTransaction, header: &Header, client: &C) -> Result<(), Error> {
		if let Some(ref filter) = self.tx_filter.as_ref() {
			if !filter.transaction_allowed(header.parent_hash(), t, client) {
				return Err(transaction::Error::NotAllowed.into())
			}
		}

		Ok(())
	}

	/// Additional params.
	pub fn additional_params(&self) -> HashMap<String, String> {
		hash_map![
			"registrar".to_owned() => format!("{:x}", self.params.registrar)
		]
	}
}

/// Auxiliary data fetcher for an Ethereum machine. In Ethereum-like machines
/// there are two kinds of auxiliary data: bodies and receipts.
#[derive(Default, Clone)]
pub struct AuxiliaryData<'a> {
	/// The full block bytes, including the header.
	pub bytes: Option<&'a [u8]>,
	/// The block receipts.
	pub receipts: Option<&'a [::receipt::Receipt]>,
}

/// Type alias for a function we can make calls through synchronously.
/// Returns the call result and state proof for each call.
pub type Call<'a> = Fn(Address, Vec<u8>) -> Result<(Vec<u8>, Vec<Vec<u8>>), String> + 'a;

/// Request for auxiliary data of a block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuxiliaryRequest {
	/// Needs the body.
	Body,
	/// Needs the receipts.
	Receipts,
	/// Needs both body and receipts.
	Both,
}

impl ::parity_machine::Machine for EthereumMachine {
	type Header = Header;

	type LiveBlock = ExecutedBlock;
	type EngineClient = ::client::EngineClient;
	type AuxiliaryRequest = AuxiliaryRequest;

	type Error = Error;
}

impl<'a> ::parity_machine::LocalizedMachine<'a> for EthereumMachine {
	type StateContext = Call<'a>;
	type AuxiliaryData = AuxiliaryData<'a>;
}

impl ::parity_machine::WithBalances for EthereumMachine {
	fn balance(&self, live: &ExecutedBlock, address: &Address) -> Result<U256, Error> {
		live.state().balance(address).map_err(Into::into)
	}

	fn add_balance(&self, live: &mut ExecutedBlock, address: &Address, amount: &U256) -> Result<(), Error> {
		live.state_mut().add_balance(address, amount, CleanupMode::NoEmpty).map_err(Into::into)
	}

	fn note_rewards(
		&self,
		live: &mut Self::LiveBlock,
		direct: &[(Address, U256)],
		indirect: &[(Address, U256)],
	) -> Result<(), Self::Error> {
		if let Tracing::Enabled(ref mut traces) = *live.traces_mut() {
			let mut tracer = ExecutiveTracer::default();

			for &(address, amount) in direct {
				tracer.trace_reward(address, amount, RewardType::Block);
			}

			for &(address, amount) in indirect {
				tracer.trace_reward(address, amount, RewardType::Uncle);
			}

			traces.push(tracer.drain().into());
		}

		Ok(())
	}
}

// Try to round gas_limit a bit so that:
// 1) it will still be in desired range
// 2) it will be a nearest (with tendency to increase) multiple of PARITY_GAS_LIMIT_DETERMINANT
fn round_block_gas_limit(gas_limit: U256, lower_limit: U256, upper_limit: U256) -> U256 {
	let increased_gas_limit = gas_limit + (PARITY_GAS_LIMIT_DETERMINANT - gas_limit % PARITY_GAS_LIMIT_DETERMINANT);
	if increased_gas_limit > upper_limit {
		let decreased_gas_limit = increased_gas_limit - PARITY_GAS_LIMIT_DETERMINANT;
		if decreased_gas_limit < lower_limit {
			gas_limit
		} else {
			decreased_gas_limit
		}
	} else {
		increased_gas_limit
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	fn get_default_ethash_extensions() -> EthashExtensions {
		EthashExtensions {
			homestead_transition: 1150000,
			eip150_transition: u64::max_value(),
			eip160_transition: u64::max_value(),
			eip161abc_transition: u64::max_value(),
			eip161d_transition: u64::max_value(),
			dao_hardfork_transition: u64::max_value(),
			dao_hardfork_beneficiary: "0000000000000000000000000000000000000001".into(),
			dao_hardfork_accounts: Vec::new(),
		}
	}

	#[test]
	fn ethash_gas_limit_is_multiple_of_determinant() {
		use ethereum_types::U256;

		let spec = ::ethereum::new_homestead_test();
		let ethparams = get_default_ethash_extensions();

		let machine = EthereumMachine::with_ethash_extensions(
			spec.params().clone(),
			Default::default(),
			ethparams,
		);

		let mut parent = ::header::Header::new();
		let mut header = ::header::Header::new();
		header.set_number(1);

		// this test will work for this constant only
		assert_eq!(PARITY_GAS_LIMIT_DETERMINANT, U256::from(37));

		// when parent.gas_limit < gas_floor_target:
		parent.set_gas_limit(U256::from(50_000));
		machine.populate_from_parent(&mut header, &parent, U256::from(100_000), U256::from(200_000));
		assert_eq!(*header.gas_limit(), U256::from(50_024));

		// when parent.gas_limit > gas_ceil_target:
		parent.set_gas_limit(U256::from(250_000));
		machine.populate_from_parent(&mut header, &parent, U256::from(100_000), U256::from(200_000));
		assert_eq!(*header.gas_limit(), U256::from(249_787));

		// when parent.gas_limit is in miner's range
		header.set_gas_used(U256::from(150_000));
		parent.set_gas_limit(U256::from(150_000));
		machine.populate_from_parent(&mut header, &parent, U256::from(100_000), U256::from(200_000));
		assert_eq!(*header.gas_limit(), U256::from(150_035));

		// when parent.gas_limit is in miner's range
		// && we can NOT increase it to be multiple of constant
		header.set_gas_used(U256::from(150_000));
		parent.set_gas_limit(U256::from(150_000));
		machine.populate_from_parent(&mut header, &parent, U256::from(100_000), U256::from(150_002));
		assert_eq!(*header.gas_limit(), U256::from(149_998));

		// when parent.gas_limit is in miner's range
		// && we can NOT increase it to be multiple of constant
		// && we can NOT decrease it to be multiple of constant
		header.set_gas_used(U256::from(150_000));
		parent.set_gas_limit(U256::from(150_000));
		machine.populate_from_parent(&mut header, &parent, U256::from(150_000), U256::from(150_002));
		assert_eq!(*header.gas_limit(), U256::from(150_002));
	}
}
