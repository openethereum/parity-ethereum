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

//! Simple Client used for EVM tests.

use std::fmt;
use std::sync::Arc;
use ethereum_types::{H256, U256, H160};
use {trie_vm_factories, journaldb, trie, kvdb_memorydb};
use kvdb::{self, KeyValueDB};
use {state_db, trace, db, spec};
use pod::PodState;
use types::{
	errors::EthcoreError,
	log_entry,
	receipt,
	transaction
};
use ethjson::spec::ForkSpec;
use trie_vm_factories::Factories;
use evm::FinalizationResult;
use vm::{self, ActionParams, CreateContractAddress};
use ethtrie;
use account_state::{CleanupMode, State};
use machine::{
	executive,
	substate::Substate,
};

use executive_state::ExecutiveState;

/// EVM test Error.
#[derive(Debug)]
pub enum EvmTestError {
	/// Trie integrity error.
	Trie(Box<ethtrie::TrieError>),
	/// EVM error.
	Evm(vm::Error),
	/// Initialization error.
	ClientError(EthcoreError),
	/// Post-condition failure,
	PostCondition(String),
}

impl<E: Into<EthcoreError>> From<E> for EvmTestError {
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

/// Simplified, single-block EVM test client.
pub struct EvmTestClient<'a> {
	state: State<state_db::StateDB>,
	spec: &'a spec::Spec,
	dump_state: fn(&State<state_db::StateDB>) -> Option<PodState>,
}

fn no_dump_state(_: &State<state_db::StateDB>) -> Option<PodState> {
	None
}

fn dump_state(state: &State<state_db::StateDB>) -> Option<PodState> {
	state.to_pod_full().ok()
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
	pub fn fork_spec_from_json(spec: &ForkSpec) -> Option<spec::Spec> {
		match *spec {
			ForkSpec::Frontier => Some(spec::new_frontier_test()),
			ForkSpec::Homestead => Some(spec::new_homestead_test()),
			ForkSpec::EIP150 => Some(spec::new_eip150_test()),
			ForkSpec::EIP158 => Some(spec::new_eip161_test()),
			ForkSpec::Byzantium => Some(spec::new_byzantium_test()),
			ForkSpec::Constantinople => Some(spec::new_constantinople_test()),
			ForkSpec::ConstantinopleFix => Some(spec::new_constantinople_fix_test()),
			ForkSpec::Istanbul => Some(spec::new_istanbul_test()),
			ForkSpec::EIP158ToByzantiumAt5 => Some(spec::new_transition_test()),
			ForkSpec::FrontierToHomesteadAt5 | ForkSpec::HomesteadToDaoAt5 | ForkSpec::HomesteadToEIP150At5 => None,
		}
	}

	/// Change default function for dump state (default does not dump)
	pub fn set_dump_state(&mut self) {
		self.dump_state = dump_state;
	}

	/// Creates new EVM test client with in-memory DB initialized with genesis of given Spec.
	/// Takes a `TrieSpec` to set the type of trie.
	pub fn new_with_trie(spec: &'a spec::Spec, trie_spec: trie::TrieSpec) -> Result<Self, EvmTestError> {
		let factories = Self::factories(trie_spec);
		let state =	Self::state_from_spec(spec, &factories)?;

		Ok(EvmTestClient {
			state,
			spec,
			dump_state: no_dump_state,
		})
	}

	/// Creates new EVM test client with an in-memory DB initialized with genesis of given chain Spec.
	pub fn new(spec: &'a spec::Spec) -> Result<Self, EvmTestError> {
		Self::new_with_trie(spec, trie::TrieSpec::Secure)
	}

	/// Creates new EVM test client with an in-memory DB initialized with given PodState.
	/// Takes a `TrieSpec` to set the type of trie.
	pub fn from_pod_state_with_trie(spec: &'a spec::Spec, pod_state: PodState, trie_spec: trie::TrieSpec) -> Result<Self, EvmTestError> {
		let factories = Self::factories(trie_spec);
		let state =	Self::state_from_pod(spec, &factories, pod_state)?;

		Ok(EvmTestClient {
			state,
			spec,
			dump_state: no_dump_state,
		})
	}

	/// Creates new EVM test client with an in-memory DB initialized with given PodState.
	pub fn from_pod_state(spec: &'a spec::Spec, pod_state: PodState) -> Result<Self, EvmTestError> {
		Self::from_pod_state_with_trie(spec, pod_state, trie::TrieSpec::Secure)
	}

	fn factories(trie_spec: trie::TrieSpec) -> Factories {
		Factories {
			vm: trie_vm_factories::VmFactory::new(5 * 1024),
			trie: trie::TrieFactory::new(trie_spec, ethtrie::Layout),
			accountdb: Default::default(),
		}
	}

	fn state_from_spec(spec: &'a spec::Spec, factories: &Factories) -> Result<State<state_db::StateDB>, EvmTestError> {
		let db = Arc::new(kvdb_memorydb::create(db::NUM_COLUMNS));
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

		State::from_existing(
			state_db,
			*genesis.state_root(),
			spec.engine.account_start_nonce(0),
			factories.clone()
		).map_err(EvmTestError::Trie)
	}

	fn state_from_pod(spec: &'a spec::Spec, factories: &Factories, pod_state: PodState) -> Result<State<state_db::StateDB>, EvmTestError> {
		let db = Arc::new(kvdb_memorydb::create(db::NUM_COLUMNS));
		let journal_db = journaldb::new(db.clone(), journaldb::Algorithm::EarlyMerge, db::COL_STATE);
		let state_db = state_db::StateDB::new(journal_db, 5 * 1024 * 1024);
		let mut state = State::new(
			state_db,
			spec.engine.account_start_nonce(0),
			factories.clone(),
		);
		state.populate_from(pod_state);
		state.commit()?;
		Ok(state)
	}

	/// Return current state.
	pub fn state(&self) -> &State<state_db::StateDB> {
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
		let info = vm::EnvInfo {
			number: genesis.number(),
			author: *genesis.author(),
			timestamp: genesis.timestamp(),
			difficulty: *genesis.difficulty(),
			last_hashes: Arc::new([H256::zero(); 256].to_vec()),
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
		info: vm::EnvInfo,
	) -> Result<FinalizationResult, EvmTestError>
	{
		let mut substate = Substate::new();
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
		env_info: &vm::EnvInfo,
		transaction: transaction::SignedTransaction,
		tracer: T,
		vm_tracer: V,
	) -> std::result::Result<TransactSuccess<T::Output, V::Output>, TransactErr> {
		let initial_gas = transaction.gas;
		// Verify transaction
		let is_ok = transaction.verify_basic(true, None);
		if let Err(error) = is_ok {
			return Err(
				TransactErr{
					state_root: *self.state.root(),
					error: error.into(),
					end_state: (self.dump_state)(&self.state),
				});
		}

		// Apply transaction
		let result = self.state.apply_with_tracing(&env_info, self.spec.engine.machine(), &transaction, tracer, vm_tracer);
		let scheme = CreateContractAddress::FromSenderAndNonce;

		// Touch the coinbase at the end of the test to simulate
		// miner reward.
		// Details: https://github.com/paritytech/parity-ethereum/issues/9431
		let schedule = self.spec.engine.machine().schedule(env_info.number);
		self.state.add_balance(&env_info.author, &0.into(), if schedule.no_empty {
			CleanupMode::NoEmpty
		} else {
			CleanupMode::ForceCreate
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

		let state_root = *self.state.root();

		let end_state = (self.dump_state)(&self.state);

		match result {
			Ok(result) => {
				Ok(TransactSuccess {
					state_root,
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
					},
					end_state,
				}
			)},
			Err(e) => Err(TransactErr {state_root, error: e.into(), end_state}),
		}
	}
}

/// To be returned inside a std::result::Result::Ok after a successful
/// transaction completed.
#[allow(dead_code)]
pub struct TransactSuccess<T, V> {
	/// State root
	pub state_root: H256,
	/// Amount of gas left
	pub gas_left: U256,
	/// Output
	pub output: Vec<u8>,
	/// Traces
	pub trace: Vec<T>,
	/// VM Traces
	pub vm_trace: Option<V>,
	/// Created contract address (if any)
	pub contract_address: Option<H160>,
	/// Generated logs
	pub logs: Vec<log_entry::LogEntry>,
	/// outcome
	pub outcome: receipt::TransactionOutcome,
	/// end state if needed
	pub end_state: Option<PodState>,
}

/// To be returned inside a std::result::Result::Err after a failed
/// transaction.
#[allow(dead_code)]
pub struct TransactErr {
	/// State root
	pub state_root: H256,
	/// Execution error
	pub error: EthcoreError,
	/// end state if needed
	pub end_state: Option<PodState>,
}
