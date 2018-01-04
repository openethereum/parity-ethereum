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

//! Simple Client used for EVM tests.

use std::fmt;
use std::sync::Arc;
use ethereum_types::{H256, U256};
use journaldb;
use {trie, kvdb_memorydb, bytes};
use kvdb::{self, KeyValueDB};
use {state, state_db, client, executive, trace, transaction, db, spec, pod_state};
use factory::Factories;
use evm::{self, VMType, FinalizationResult};
use vm::{self, ActionParams};

/// EVM test Error.
#[derive(Debug)]
pub enum EvmTestError {
	/// Trie integrity error.
	Trie(trie::TrieError),
	/// EVM error.
	Evm(vm::Error),
	/// Initialization error.
	ClientError(::error::Error),
	/// Low-level database error.
	Database(kvdb::Error),
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
			Database(ref err) => write!(fmt, "DB: {}", err),
			PostCondition(ref err) => write!(fmt, "{}", err),
		}
	}
}

use ethereum;
use ethjson::state::test::ForkSpec;

lazy_static! {
	pub static ref FRONTIER: spec::Spec = ethereum::new_frontier_test();
	pub static ref HOMESTEAD: spec::Spec = ethereum::new_homestead_test();
	pub static ref EIP150: spec::Spec = ethereum::new_eip150_test();
	pub static ref EIP161: spec::Spec = ethereum::new_eip161_test();
	pub static ref BYZANTIUM: spec::Spec = ethereum::new_byzantium_test();
	pub static ref BYZANTIUM_TRANSITION: spec::Spec = ethereum::new_transition_test();
}

/// Simplified, single-block EVM test client.
pub struct EvmTestClient<'a> {
	state: state::State<state_db::StateDB>,
	spec: &'a spec::Spec,
}

impl<'a> EvmTestClient<'a> {
	/// Converts a json spec definition into spec.
	pub fn spec_from_json(spec: &ForkSpec) -> Option<&'static spec::Spec> {
		match *spec {
			ForkSpec::Frontier => Some(&*FRONTIER),
			ForkSpec::Homestead => Some(&*HOMESTEAD),
			ForkSpec::EIP150 => Some(&*EIP150),
			ForkSpec::EIP158 => Some(&*EIP161),
			ForkSpec::Byzantium => Some(&*BYZANTIUM),
			ForkSpec::EIP158ToByzantiumAt5 => Some(&BYZANTIUM_TRANSITION),
			ForkSpec::FrontierToHomesteadAt5 | ForkSpec::HomesteadToDaoAt5 | ForkSpec::HomesteadToEIP150At5 => None,
			_ => None,
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
			vm: evm::Factory::new(VMType::Interpreter, 5 * 1024),
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
			db.write(batch).map_err(EvmTestError::Database)?;
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

	/// Execute the VM given ActionParams and tracer.
	/// Returns amount of gas left and the output.
	pub fn call<T: trace::VMTracer>(&mut self, params: ActionParams, vm_tracer: &mut T)
		-> Result<FinalizationResult, EvmTestError>
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
		let mut substate = state::Substate::new();
		let mut tracer = trace::NoopTracer;
		let mut output = vec![];
		let mut executive = executive::Executive::new(&mut self.state, &info, self.spec.engine.machine());
		executive.call(
			params,
			&mut substate,
			bytes::BytesRef::Flexible(&mut output),
			&mut tracer,
			vm_tracer,
		).map_err(EvmTestError::Evm)
	}

	/// Executes a SignedTransaction within context of the provided state and `EnvInfo`.
	/// Returns the state root, gas left and the output.
	pub fn transact<T: trace::VMTracer>(
		&mut self,
		env_info: &client::EnvInfo,
		transaction: transaction::SignedTransaction,
		vm_tracer: T,
	) -> TransactResult<T::Output> {
		let initial_gas = transaction.gas;
		// Verify transaction
		let is_ok = transaction.verify_basic(true, None, env_info.number >= self.spec.engine.params().eip86_transition);
		if let Err(error) = is_ok {
			return TransactResult::Err {
				state_root: *self.state.root(),
				error,
			};
		}

		// Apply transaction
		let tracer = trace::NoopTracer;
		let result = self.state.apply_with_tracing(&env_info, self.spec.engine.machine(), &transaction, tracer, vm_tracer);

		match result {
			Ok(result) => {
				self.state.commit().ok();
				TransactResult::Ok {
					state_root: *self.state.root(),
					gas_left: initial_gas - result.receipt.gas_used,
					output: result.output,
					vm_trace: result.vm_trace,
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
pub enum TransactResult<T> {
	/// Successful execution
	Ok {
		/// State root
		state_root: H256,
		/// Amount of gas left
		gas_left: U256,
		/// Output
		output: Vec<u8>,
		/// VM Traces
		vm_trace: Option<T>,
	},
	/// Transaction failed to run
	Err {
		/// State root
		state_root: H256,
		/// Execution error
		error: ::error::Error,
	},
}
