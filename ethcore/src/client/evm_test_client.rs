// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Simple Client used for EVM tests.

use std::fmt;
use std::sync::Arc;
use ethereum_types::{H256, U256, H160};
use {factory, journaldb, trie, kvdb_memorydb};
use kvdb::{self, KeyValueDB};
use {state, state_db, client, executive, trace, transaction, db, spec, pod_state, log_entry, receipt};
use factory::Factories;
use evm::{VMType, FinalizationResult};
use vm::{self, ActionParams};
use ethtrie;

/// EVM test Error.
#[derive(Debug)]
pub enum EvmTestError {
	/// Trie integrity error.
	Trie(Box<ethtrie::TrieError>),
	/// EVM error.
	Evm(vm::Error),
	/// Initialization error.
	ClientError(::error::Error),
	/// Post-condition failure,
	PostCondition(String),
}

impl<E: Into<::error::Error>> From<E> for EvmTestError {
	fn from(err: E) -> Self {
		EvmTestError::ClientError(err.into())
	}
}

impl fmt::Display for EvmTestError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use self::EvmTestError::*;

		match *self {
			Trie(ref err) => write!(fmt, "Trie: {}", err),
			Evm(ref err) => write!(fmt, "EVM: {}", err),
			ClientError(ref err) => write!(fmt, "{}", err),
			PostCondition(ref err) => write!(fmt, "{}", err),
		}
	}
}

use ethereum;
use ethjson::spec::ForkSpec;

/// Simplified, single-block EVM test client.
pub struct EvmTestClient<'a> {
	state: state::State<state_db::StateDB>,
	spec: &'a spec::Spec,
}

impl<'a> fmt::Debug for EvmTestClient<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("EvmTestClient")
			.field("state", &self.state)
			.field("spec", &self.spec.name)
			.finish()
	}
}

impl<'a> EvmTestClient<'a> {
	/// Converts a json spec definition into spec.
	pub fn spec_from_json(spec: &ForkSpec) -> Option<spec::Spec> {
		match *spec {
			ForkSpec::Frontier => Some(ethereum::new_frontier_test()),
			ForkSpec::Homestead => Some(ethereum::new_homestead_test()),
			ForkSpec::EIP150 => Some(ethereum::new_eip150_test()),
			ForkSpec::EIP158 => Some(ethereum::new_eip161_test()),
			ForkSpec::Byzantium => Some(ethereum::new_byzantium_test()),
			ForkSpec::Constantinople => Some(ethereum::new_constantinople_test()),
			ForkSpec::EIP158ToByzantiumAt5 => Some(ethereum::new_transition_test()),
			ForkSpec::FrontierToHomesteadAt5 | ForkSpec::HomesteadToDaoAt5 | ForkSpec::HomesteadToEIP150At5 => None,
		}
	}

	/// Creates new EVM test client with in-memory DB initialized with genesis of given Spec.
	pub fn new(spec: &'a spec::Spec) -> Result<Self, EvmTestError> {
		let factories = Self::factories();
		let state =	Self::state_from_spec(spec, &factories)?;

		Ok(EvmTestClient {
			state,
			spec,
		})
	}

	/// Creates new EVM test client with in-memory DB initialized with given PodState.
	pub fn from_pod_state(spec: &'a spec::Spec, pod_state: pod_state::PodState) -> Result<Self, EvmTestError> {
		let factories = Self::factories();
		let state =	Self::state_from_pod(spec, &factories, pod_state)?;

		Ok(EvmTestClient {
			state,
			spec,
		})
	}

	fn factories() -> Factories {
		Factories {
			vm: factory::VmFactory::new(VMType::Interpreter, 5 * 1024),
			trie: trie::TrieFactory::new(trie::TrieSpec::Secure),
			accountdb: Default::default(),
		}
	}

	fn state_from_spec(spec: &'a spec::Spec, factories: &Factories) -> Result<state::State<state_db::StateDB>, EvmTestError> {
		let db = Arc::new(kvdb_memorydb::create(db::NUM_COLUMNS.expect("We use column-based DB; qed")));
		let journal_db = journaldb::new(db.clone(), journaldb::Algorithm::EarlyMerge, db::COL_STATE);
		let mut state_db = state_db::StateDB::new(journal_db, 5 * 1024 * 1024);
		state_db = spec.ensure_db_good(state_db, factories)?;

		let genesis = spec.genesis_header();
		// Write DB
		{
			let mut batch = kvdb::DBTransaction::new();
			state_db.journal_under(&mut batch, 0, &genesis.hash())?;
			db.write(batch)?;
		}

		state::State::from_existing(
			state_db,
			*genesis.state_root(),
			spec.engine.account_start_nonce(0),
			factories.clone()
		).map_err(EvmTestError::Trie)
	}

	fn state_from_pod(spec: &'a spec::Spec, factories: &Factories, pod_state: pod_state::PodState) -> Result<state::State<state_db::StateDB>, EvmTestError> {
		let db = Arc::new(kvdb_memorydb::create(db::NUM_COLUMNS.expect("We use column-based DB; qed")));
		let journal_db = journaldb::new(db.clone(), journaldb::Algorithm::EarlyMerge, db::COL_STATE);
		let state_db = state_db::StateDB::new(journal_db, 5 * 1024 * 1024);
		let mut state = state::State::new(
			state_db,
			spec.engine.account_start_nonce(0),
			factories.clone(),
		);
		state.populate_from(pod_state);
		state.commit()?;
		Ok(state)
	}

	/// Return current state.
	pub fn state(&self) -> &state::State<state_db::StateDB> {
		&self.state
	}

	/// Execute the VM given ActionParams and tracer.
	/// Returns amount of gas left and the output.
	pub fn call<T: trace::Tracer, V: trace::VMTracer>(
		&mut self,
		params: ActionParams,
		tracer: &mut T,
		vm_tracer: &mut V,
	) -> Result<FinalizationResult, EvmTestError>
	{
		let genesis = self.spec.genesis_header();
		let info = client::EnvInfo {
			number: genesis.number(),
			author: *genesis.author(),
			timestamp: genesis.timestamp(),
			difficulty: *genesis.difficulty(),
			last_hashes: Arc::new([H256::default(); 256].to_vec()),
			gas_used: 0.into(),
			gas_limit: *genesis.gas_limit(),
		};
		self.call_envinfo(params, tracer, vm_tracer, info)
	}

	/// Execute the VM given envinfo, ActionParams and tracer.
	/// Returns amount of gas left and the output.
	pub fn call_envinfo<T: trace::Tracer, V: trace::VMTracer>(
		&mut self,
		params: ActionParams,
		tracer: &mut T,
		vm_tracer: &mut V,
		info: client::EnvInfo,
	) -> Result<FinalizationResult, EvmTestError>
	{
		let mut substate = state::Substate::new();
		let machine = self.spec.engine.machine();
		let schedule = machine.schedule(info.number);
		let mut executive = executive::Executive::new(&mut self.state, &info, &machine, &schedule);
		executive.call(
			params,
			&mut substate,
			tracer,
			vm_tracer,
		).map_err(EvmTestError::Evm)
	}

	/// Executes a SignedTransaction within context of the provided state and `EnvInfo`.
	/// Returns the state root, gas left and the output.
	pub fn transact<T: trace::Tracer, V: trace::VMTracer>(
		&mut self,
		env_info: &client::EnvInfo,
		transaction: transaction::SignedTransaction,
		tracer: T,
		vm_tracer: V,
	) -> TransactResult<T::Output, V::Output> {
		let initial_gas = transaction.gas;
		// Verify transaction
		let is_ok = transaction.verify_basic(true, None, false);
		if let Err(error) = is_ok {
			return TransactResult::Err {
				state_root: *self.state.root(),
				error: error.into(),
			};
		}

		// Apply transaction
		let result = self.state.apply_with_tracing(&env_info, self.spec.engine.machine(), &transaction, tracer, vm_tracer);
		let scheme = self.spec.engine.machine().create_address_scheme(env_info.number);

		// Touch the coinbase at the end of the test to simulate
		// miner reward.
		// Details: https://github.com/paritytech/parity-ethereum/issues/9431
		let schedule = self.spec.engine.machine().schedule(env_info.number);
		self.state.add_balance(&env_info.author, &0.into(), if schedule.no_empty {
			state::CleanupMode::NoEmpty
		} else {
			state::CleanupMode::ForceCreate
		}).ok();
		// Touching also means that we should remove the account if it's within eip161
		// conditions.
		self.state.kill_garbage(
			&vec![env_info.author].into_iter().collect(),
			schedule.kill_empty,
			&None,
			false
		).ok();
		self.state.commit().ok();

		match result {
			Ok(result) => {
				TransactResult::Ok {
					state_root: *self.state.root(),
					gas_left: initial_gas - result.receipt.gas_used,
					outcome: result.receipt.outcome,
					output: result.output,
					trace: result.trace,
					vm_trace: result.vm_trace,
					logs: result.receipt.logs,
					contract_address: if let transaction::Action::Create = transaction.action {
						Some(executive::contract_address(scheme, &transaction.sender(), &transaction.nonce, &transaction.data).0)
					} else {
						None
					}
				}
			},
			Err(error) => TransactResult::Err {
				state_root: *self.state.root(),
				error,
			},
		}
	}
}

/// A result of applying transaction to the state.
#[derive(Debug)]
pub enum TransactResult<T, V> {
	/// Successful execution
	Ok {
		/// State root
		state_root: H256,
		/// Amount of gas left
		gas_left: U256,
		/// Output
		output: Vec<u8>,
		/// Traces
		trace: Vec<T>,
		/// VM Traces
		vm_trace: Option<V>,
		/// Created contract address (if any)
		contract_address: Option<H160>,
		/// Generated logs
		logs: Vec<log_entry::LogEntry>,
		/// outcome
		outcome: receipt::TransactionOutcome,
	},
	/// Transaction failed to run
	Err {
		/// State root
		state_root: H256,
		/// Execution error
		error: ::error::Error,
	},
}
