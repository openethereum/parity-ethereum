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
use util::{self, U256, H256, journaldb, trie};
use util::kvdb::{self, KeyValueDB};
use {state, state_db, client, executive, trace, transaction, db, spec, pod_state};
use factory::Factories;
use evm::{self, VMType};
use vm::{self, ActionParams};

/// EVM test Error.
#[derive(Debug)]
pub enum EvmTestError {
	/// Trie integrity error.
	Trie(util::TrieError),
	/// EVM error.
	Evm(vm::Error),
	/// Initialization error.
	Initialization(::error::Error),
	/// Error during transaction execution.
	Execution(::error::ExecutionError),
	/// Low-level database error.
	Database(String),
	/// Post-condition failure,
	PostCondition(String),
}

impl fmt::Display for EvmTestError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use self::EvmTestError::*;

		match *self {
			Trie(ref err) => write!(fmt, "Trie: {}", err),
			Evm(ref err) => write!(fmt, "EVM: {}", err),
			Initialization(ref err) => write!(fmt, "Initialization: {}", err),
			Execution(ref err) => write!(fmt, "Execution: {}", err),
			Database(ref err) => write!(fmt, "DB: {}", err),
			PostCondition(ref err) => write!(fmt, "{}", err),
		}
	}
}

/// Simplified, single-block EVM test client.
pub struct EvmTestClient {
	state: state::State<state_db::StateDB>,
	factories: Factories,
	spec: spec::Spec,
}

impl EvmTestClient {
	/// Creates new EVM test client with in-memory DB initialized with genesis of given Spec.
	pub fn new(spec: spec::Spec) -> Result<Self, EvmTestError> {
		Self::with_pre_state(spec, None)
	}

	pub fn with_pre_state<'a, T>(spec: spec::Spec, pre: T) -> Result<Self, EvmTestError> where
		T: Into<Option<&'a pod_state::PodState>>,
	{
		let factories = Factories {
			vm: evm::Factory::new(VMType::Interpreter, 5 * 1024),
			trie: trie::TrieFactory::new(trie::TrieSpec::Secure),
			accountdb: Default::default(),
		};
		let genesis = spec.genesis_header();
		let db = Arc::new(kvdb::in_memory(db::NUM_COLUMNS.expect("We use column-based DB; qed")));
		let journal_db = journaldb::new(db.clone(), journaldb::Algorithm::EarlyMerge, db::COL_STATE);
		let mut state_db = state_db::StateDB::new(journal_db, 5 * 1024 * 1024);
		let pre_state = pre.into();
		let state = match pre_state {
			None => {
				state_db = spec.ensure_db_good(state_db, &factories).map_err(EvmTestError::Initialization)?;
				// Write DB
				{
					let mut batch = kvdb::DBTransaction::new();
					state_db.journal_under(&mut batch, 0, &genesis.hash()).map_err(|e| EvmTestError::Initialization(e.into()))?;
					db.write(batch).map_err(EvmTestError::Database)?;
				}
				state::State::from_existing(
					state_db.boxed_clone(),
					*genesis.state_root(),
					spec.engine.account_start_nonce(0),
					factories.clone()
				).map_err(EvmTestError::Trie)?
			},
			Some(pre_state) => {
				let mut state = state::State::new(
					state_db.boxed_clone(),
					spec.engine.account_start_nonce(0),
					factories.clone(),
				);
				state.populate_from(pre_state.clone());
				state.commit().map_err(EvmTestError::Initialization)?;
				state
			}
		};

		Ok(EvmTestClient {
			state,
			factories,
			spec,
		})
	}

	/// Execute the VM given ActionParams and tracer.
	/// Returns amount of gas left and the output.
	pub fn call<T: trace::VMTracer>(&mut self, params: ActionParams, vm_tracer: &mut T)
		-> Result<(U256, Vec<u8>), EvmTestError>
	{
		let genesis = self.spec.genesis_header();
		let info = client::EnvInfo {
			number: genesis.number(),
			author: *genesis.author(),
			timestamp: genesis.timestamp(),
			difficulty: *genesis.difficulty(),
			last_hashes: Arc::new([util::H256::default(); 256].to_vec()),
			gas_used: 0.into(),
			gas_limit: *genesis.gas_limit(),
		};
		let mut substate = state::Substate::new();
		let mut tracer = trace::NoopTracer;
		let mut output = vec![];
		let mut executive = executive::Executive::new(&mut self.state, &info, &*self.spec.engine);
		let (gas_left, _) = executive.call(
			params,
			&mut substate,
			util::BytesRef::Flexible(&mut output),
			&mut tracer,
			vm_tracer,
		).map_err(EvmTestError::Evm)?;

		Ok((gas_left, output))
	}

	/// Executes a SignedTransaction within context of the provided state and `EnvInfo`.
	/// Returns the state root, gas left and the output.
	pub fn transact<T: trace::VMTracer>(&mut self, env_info: &client::EnvInfo, transaction: transaction::SignedTransaction, vm_tracer: T)
		-> Result<(H256, U256, Vec<u8>), EvmTestError>
	{
		let tracer = trace::NoopTracer;
		let executed = {
			let mut executive = executive::Executive::new(&mut self.state, env_info, &*self.spec.engine);
			executive.transact_with_tracer(
				&transaction,
				true,
				tracer,
				vm_tracer,
			).map_err(EvmTestError::Execution)
		}?;
		self.state.commit().map_err(EvmTestError::Initialization)?;
		Ok((*self.state.root(), executed.gas_used, executed.output))
	}
}
