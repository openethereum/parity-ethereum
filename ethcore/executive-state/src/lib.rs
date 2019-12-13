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

//! Execute transactions and modify State. This is glue code between the `ethcore` and
//! `account-state` crates and contains everything that requires `Machine` or `Executive` (or types
//! thereof).

use account_state::{
	backend::{self, Backend},
	state::State,
};
use bytes::Bytes;
use common_types::{
	engines::machine::Executed as RawExecuted,
	errors::{ExecutionError, EthcoreError as Error},
	transaction::SignedTransaction,
	receipt::{TransactionOutcome, Receipt},
};
use ethereum_types::H256;
use hash_db::AsHashDB;
use keccak_hasher::KeccakHasher;
use kvdb::DBValue;
use log::trace;
use machine::{
	machine::Machine,
	executive::{Executive, TransactOptions},
	executed::Executed,
};
use trace::{FlatTrace, VMTrace};
use trie_vm_factories::Factories;
use vm::EnvInfo;

/// Return type of proof validity check.
#[derive(Debug, Clone)]
pub enum ProvedExecution {
	/// Proof wasn't enough to complete execution.
	BadProof,
	/// The transaction failed, but not due to a bad proof.
	Failed(ExecutionError),
	/// The transaction successfully completed with the given proof.
	Complete(Box<Executed>),
}

/// Used to return information about an `State::apply` operation.
pub struct ApplyOutcome<T, V> {
	/// The receipt for the applied transaction.
	pub receipt: Receipt,
	/// The output of the applied transaction.
	pub output: Bytes,
	/// The trace for the applied transaction, empty if tracing was not produced.
	pub trace: Vec<T>,
	/// The VM trace for the applied transaction, None if tracing was not produced.
	pub vm_trace: Option<V>
}

/// Result type for the execution ("application") of a transaction.
pub type ApplyResult<T, V> = Result<ApplyOutcome<T, V>, Error>;

/// Check the given proof of execution.
/// `Err(ExecutionError::Internal)` indicates failure, everything else indicates
/// a successful proof (as the transaction itself may be poorly chosen).
pub fn check_proof(
	proof: &[DBValue],
	root: H256,
	transaction: &SignedTransaction,
	machine: &Machine,
	env_info: &EnvInfo,
) -> ProvedExecution {
	let backend = self::backend::ProofCheck::new(proof);
	let mut factories = Factories::default();
	factories.accountdb = account_db::Factory::Plain;

	let res = State::from_existing(
		backend,
		root,
		machine.account_start_nonce(env_info.number),
		factories
	);

	let mut state = match res {
		Ok(state) => state,
		Err(_) => return ProvedExecution::BadProof,
	};

	let options = TransactOptions::with_no_tracing().save_output_from_contract();
	match execute(&mut state, env_info, machine, transaction, options, true) {
		Ok(executed) => ProvedExecution::Complete(Box::new(executed)),
		Err(ExecutionError::Internal(_)) => ProvedExecution::BadProof,
		Err(e) => ProvedExecution::Failed(e),
	}
}

/// Prove a `virtual` transaction on the given state.
/// Returns `None` when the transaction could not be proved,
/// and a proof otherwise.
pub fn prove_transaction_virtual<H: AsHashDB<KeccakHasher, DBValue> + Send + Sync>(
	db: H,
	root: H256,
	transaction: &SignedTransaction,
	machine: &Machine,
	env_info: &EnvInfo,
	factories: Factories,
) -> Option<(Bytes, Vec<DBValue>)> {
	use account_state::backend::Proving;

	let backend = Proving::new(db);
	let res = State::from_existing(
		backend,
		root,
		machine.account_start_nonce(env_info.number),
		factories,
	);

	let mut state = match res {
		Ok(state) => state,
		Err(_) => return None,
	};

	let options = TransactOptions::with_no_tracing().dont_check_nonce().save_output_from_contract();
	match execute(&mut state, env_info, machine, transaction, options, true) {
		Err(ExecutionError::Internal(_)) => None,
		Err(e) => {
			trace!(target: "state", "Proved call failed: {}", e);
			Some((Vec::new(), state.drop().1.extract_proof()))
		}
		Ok(res) => Some((res.output, state.drop().1.extract_proof())),
	}
}

/// Collects code that needs a Machine and/or Executive
pub trait ExecutiveState {
	/// Execute a given transaction, producing a receipt and an optional trace.
	/// This will change the state accordingly.
	fn apply(
		&mut self,
		env_info: &EnvInfo,
		machine: &Machine,
		t: &SignedTransaction,
		tracing: bool
	) -> ApplyResult<FlatTrace, VMTrace>;

	/// Execute a given transaction with given tracer and VM tracer producing a receipt and an optional trace.
	/// This will change the state accordingly.
	fn apply_with_tracing<V, T>(
		&mut self,
		env_info: &EnvInfo,
		machine: &Machine,
		t: &SignedTransaction,
		tracer: T,
		vm_tracer: V,
	) -> ApplyResult<T::Output, V::Output>
		where
			T: trace::Tracer,
			V: trace::VMTracer;
}

impl<B: Backend> ExecutiveState for State<B> {
	/// Execute a given transaction, producing a receipt and an optional trace.
	/// This will change the state accordingly.
	fn apply(
		&mut self,
		env_info: &EnvInfo,
		machine: &Machine,
		t: &SignedTransaction,
		tracing: bool
	) -> ApplyResult<FlatTrace, VMTrace> {
		if tracing {
			let options = TransactOptions::with_tracing();
			self.apply_with_tracing(env_info, machine, t, options.tracer, options.vm_tracer)
		} else {
			let options = TransactOptions::with_no_tracing();
			self.apply_with_tracing(env_info, machine, t, options.tracer, options.vm_tracer)
		}
	}

	/// Execute a given transaction with given tracer and VM tracer producing a receipt and an optional trace.
	/// This will change the state accordingly.
	fn apply_with_tracing<V, T>(
		&mut self,
		env_info: &EnvInfo,
		machine: &Machine,
		t: &SignedTransaction,
		tracer: T,
		vm_tracer: V,
	) -> ApplyResult<T::Output, V::Output>
		where
			T: trace::Tracer,
			V: trace::VMTracer,
	{
		let options = TransactOptions::new(tracer, vm_tracer);
		let e = execute(self, env_info, machine, t, options, false)?;
		let params = machine.params();

		let eip658 = env_info.number >= params.eip658_transition;
		let no_intermediate_commits =
			eip658 ||
				(env_info.number >= params.eip98_transition && env_info.number >= params.validate_receipts_transition);

		let outcome = if no_intermediate_commits {
			if eip658 {
				TransactionOutcome::StatusCode(if e.exception.is_some() { 0 } else { 1 })
			} else {
				TransactionOutcome::Unknown
			}
		} else {
			self.commit()?;
			TransactionOutcome::StateRoot(self.root().clone())
		};

		let output = e.output;
		let receipt = Receipt::new(outcome, e.cumulative_gas_used, e.logs);
		trace!(target: "state", "Transaction receipt: {:?}", receipt);

		Ok(ApplyOutcome {
			receipt,
			output,
			trace: e.trace,
			vm_trace: e.vm_trace,
		})
	}
}

// Execute a given transaction without committing changes.
//
// `virt` signals that we are executing outside of a block set and restrictions like
// gas limits and gas costs should be lifted.
fn execute<B, T, V>(
	state: &mut State<B>,
	env_info: &EnvInfo,
	machine: &Machine,
	t: &SignedTransaction,
	options: TransactOptions<T, V>,
	virt: bool
) -> Result<RawExecuted<T::Output, V::Output>, ExecutionError>
	where
		B: Backend,
		T: trace::Tracer,
		V: trace::VMTracer,
{
	let schedule = machine.schedule(env_info.number);
	let mut e = Executive::new(state, env_info, machine, &schedule);

	match virt {
		true => e.transact_virtual(t, options),
		false => e.transact(t, options),
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::str::FromStr;
	use std::collections::HashSet;

	use super::*;

	use account_state::{Account, CleanupMode};
	use common_types::transaction::*;
	use keccak_hash::{keccak, KECCAK_NULL_RLP};
	use parity_crypto::publickey::Secret;
	use ethereum_types::{H256, U256, Address, BigEndianHash};
	use ethcore::{
		test_helpers::{get_temp_state, get_temp_state_db}
	};
	use ethtrie;
	use machine::Machine;
	use pod::{self, PodAccount, PodState};
	use rustc_hex::FromHex;
	use spec;
	use ::trace::{FlatTrace, TraceError, trace};
	use trie_db::{TrieFactory, TrieSpec};
	use vm::EnvInfo;

	fn secret() -> Secret {
		keccak("").into()
	}

	fn make_frontier_machine(max_depth: usize) -> Machine {
		let mut machine = spec::new_frontier_test_machine();
		machine.set_schedule_creation_rules(Box::new(move |s, _| s.max_depth = max_depth));
		machine
	}

	#[test]
	fn should_apply_create_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 100.into(),
			data: FromHex::from_hex("601080600c6000396000f3006000355415600957005b60203560003555").unwrap(),
		}.sign(&secret(), None);

		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 0,
			action: trace::Action::Create(trace::Create {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				value: 100.into(),
				gas: 77412.into(),
				init: vec![96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85],
				creation_method: Some(trace::CreationMethod::Create),
			}),
			result: trace::Res::Create(trace::CreateResult {
				gas_used: U256::from(3224),
				address: Address::from_str("8988167e088c87cd314df6d3c2b83da5acb93ace").unwrap(),
				code: vec![96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53]
			}),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_work_when_cloned() {
		let _ = env_logger::try_init();

		let a = Address::zero();

		let mut state = {
			let mut state = get_temp_state();
			assert_eq!(state.exists(&a).unwrap(), false);
			state.inc_nonce(&a).unwrap();
			state.commit().unwrap();
			state.clone()
		};

		state.inc_nonce(&a).unwrap();
		state.commit().unwrap();
	}

	#[test]
	fn should_trace_failed_create_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 100.into(),
			data: FromHex::from_hex("5b600056").unwrap(),
		}.sign(&secret(), None);

		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			action: trace::Action::Create(trace::Create {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				value: 100.into(),
				gas: 78792.into(),
				init: vec![91, 96, 0, 86],
				creation_method: Some(trace::CreationMethod::Create),
			}),
			result: trace::Res::FailedCreate(TraceError::OutOfGas),
			subtraces: 0
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_call_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("6000").unwrap()).unwrap();
		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(3),
				output: vec![]
			}),
			subtraces: 0,
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_basic_call_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(0),
				output: vec![]
			}),
			subtraces: 0,
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_call_transaction_to_builtin() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = spec::new_test_machine();

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0x1)),
			value: 0.into(),
			data: vec![],
		}.sign(&secret(), None);

		let result = state.apply(&info, &machine, &t, true).unwrap();

		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_str("0000000000000000000000000000000000000001").unwrap(),
				value: 0.into(),
				gas: 79_000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(3000),
				output: vec![]
			}),
			subtraces: 0,
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_not_trace_subcall_transaction_to_builtin() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = spec::new_test_machine();

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 0.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("600060006000600060006001610be0f1").unwrap()).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();

		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 0.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(3_721), // in post-eip150
				output: vec![]
			}),
			subtraces: 0,
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_callcode_properly() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = spec::new_test_machine();

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 0.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("60006000600060006000600b611000f2").unwrap()).unwrap();
		state.init_code(&Address::from_low_u64_be(0xb), FromHex::from_hex("6000").unwrap()).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();

		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 0.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: 724.into(), // in post-eip150
				output: vec![]
			}),
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xa),
				to: Address::from_low_u64_be(0xb),
				value: 0.into(),
				gas: 4096.into(),
				input: vec![],
				call_type: Some(trace::CallType::CallCode),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: 3.into(),
				output: vec![],
			}),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_delegatecall_properly() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		info.number = 0x789b0;
		let machine = spec::new_test_machine();

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 0.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("6000600060006000600b618000f4").unwrap()).unwrap();
		state.init_code(&Address::from_low_u64_be(0xb), FromHex::from_hex("60056000526001601ff3").unwrap()).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();

		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 0.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(736), // in post-eip150
				output: vec![]
			}),
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xa),
				to: Address::from_low_u64_be(0xb),
				value: 0.into(),
				gas: 32768.into(),
				input: vec![],
				call_type: Some(trace::CallType::DelegateCall),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: 18.into(),
				output: vec![5],
			}),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_failed_call_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("5b600056").unwrap()).unwrap();
		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::FailedCall(TraceError::OutOfGas),
			subtraces: 0,
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_call_with_subcall_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap()).unwrap();
		state.init_code(&Address::from_low_u64_be(0xb), FromHex::from_hex("6000").unwrap()).unwrap();
		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();

		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(69),
				output: vec![]
			}),
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xa),
				to: Address::from_low_u64_be(0xb),
				value: 0.into(),
				gas: 78934.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(3),
				output: vec![]
			}),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_call_with_basic_subcall_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("60006000600060006045600b6000f1").unwrap()).unwrap();
		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(31761),
				output: vec![]
			}),
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xa),
				to: Address::from_low_u64_be(0xb),
				value: 69.into(),
				gas: 2300.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult::default()),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_not_trace_call_with_invalid_basic_subcall_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("600060006000600060ff600b6000f1").unwrap()).unwrap();	// not enough funds.
		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 0,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(31761),
				output: vec![]
			}),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_failed_subcall_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],//600480600b6000396000f35b600056
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap()).unwrap();
		state.init_code(&Address::from_low_u64_be(0xb), FromHex::from_hex("5b600056").unwrap()).unwrap();
		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(79_000),
				output: vec![]
			}),
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xa),
				to: Address::from_low_u64_be(0xb),
				value: 0.into(),
				gas: 78934.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::FailedCall(TraceError::OutOfGas),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_call_with_subcall_with_subcall_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap()).unwrap();
		state.init_code(&Address::from_low_u64_be(0xb), FromHex::from_hex("60006000600060006000600c602b5a03f1").unwrap()).unwrap();
		state.init_code(&Address::from_low_u64_be(0xc), FromHex::from_hex("6000").unwrap()).unwrap();
		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(135),
				output: vec![]
			}),
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xa),
				to: Address::from_low_u64_be(0xb),
				value: 0.into(),
				gas: 78934.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(69),
				output: vec![]
			}),
		}, FlatTrace {
			trace_address: vec![0, 0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xb),
				to: Address::from_low_u64_be(0xc),
				value: 0.into(),
				gas: 78868.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(3),
				output: vec![]
			}),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_failed_subcall_with_subcall_transaction() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],//600480600b6000396000f35b600056
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap()).unwrap();
		state.init_code(&Address::from_low_u64_be(0xb), FromHex::from_hex("60006000600060006000600c602b5a03f1505b601256").unwrap()).unwrap();
		state.init_code(&Address::from_low_u64_be(0xc), FromHex::from_hex("6000").unwrap()).unwrap();
		state.add_balance(&t.sender(), &(100.into()), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();

		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(79_000),
				output: vec![]
			})
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xa),
				to: Address::from_low_u64_be(0xb),
				value: 0.into(),
				gas: 78934.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::FailedCall(TraceError::OutOfGas),
		}, FlatTrace {
			trace_address: vec![0, 0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Call(trace::Call {
				from: Address::from_low_u64_be(0xb),
				to: Address::from_low_u64_be(0xc),
				value: 0.into(),
				gas: 78868.into(),
				call_type: Some(trace::CallType::Call),
				input: vec![],
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(3),
				output: vec![]
			}),
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn should_trace_suicide() {
		let _ = env_logger::try_init();

		let mut state = get_temp_state();

		let mut info = EnvInfo::default();
		info.gas_limit = 1_000_000.into();
		let machine = make_frontier_machine(5);

		let t = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Call(Address::from_low_u64_be(0xa)),
			value: 100.into(),
			data: vec![],
		}.sign(&secret(), None);

		state.init_code(&Address::from_low_u64_be(0xa), FromHex::from_hex("73000000000000000000000000000000000000000bff").unwrap()).unwrap();
		state.add_balance(&Address::from_low_u64_be(0xa), &50.into(), CleanupMode::NoEmpty).unwrap();
		state.add_balance(&t.sender(), &100.into(), CleanupMode::NoEmpty).unwrap();
		let result = state.apply(&info, &machine, &t, true).unwrap();
		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: Address::from_str("9cce34f7ab185c7aba1b7c8140d620b4bda941d6").unwrap(),
				to: Address::from_low_u64_be(0xa),
				value: 100.into(),
				gas: 79000.into(),
				input: vec![],
				call_type: Some(trace::CallType::Call),
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: 3.into(),
				output: vec![]
			}),
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Suicide(trace::Suicide {
				address: Address::from_low_u64_be(0xa),
				refund_address: Address::from_low_u64_be(0xb),
				balance: 150.into(),
			}),
			result: trace::Res::None,
		}];

		assert_eq!(result.trace, expected_trace);
	}

	#[test]
	fn code_from_database() {
		let a = Address::zero();
		let (root, db) = {
			let mut state = get_temp_state();
			state.require_or_from(&a, false, || Account::new_contract(42.into(), 0.into(), 0.into(), KECCAK_NULL_RLP), |_|{}).unwrap();
			state.init_code(&a, vec![1, 2, 3]).unwrap();
			assert_eq!(state.code(&a).unwrap(), Some(Arc::new(vec![1u8, 2, 3])));
			state.commit().unwrap();
			assert_eq!(state.code(&a).unwrap(), Some(Arc::new(vec![1u8, 2, 3])));
			state.drop()
		};

		let state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		assert_eq!(state.code(&a).unwrap(), Some(Arc::new(vec![1u8, 2, 3])));
	}

	#[test]
	fn storage_at_from_database() {
		let a = Address::zero();
		let (root, db) = {
			let mut state = get_temp_state();
			state.set_storage(&a, BigEndianHash::from_uint(&U256::from(1u64)), BigEndianHash::from_uint(&U256::from(69u64))).unwrap();
			state.commit().unwrap();
			state.drop()
		};

		let s = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		let h1 = BigEndianHash::from_uint(&U256::from(1u64));
		let h2 = BigEndianHash::from_uint(&U256::from(69u64));
		assert_eq!(s.storage_at(&a, &h1).unwrap(), h2);
	}

	#[test]
	fn get_from_database() {
		let a = Address::zero();
		let (root, db) = {
			let mut state = get_temp_state();
			state.inc_nonce(&a).unwrap();
			state.add_balance(&a, &U256::from(69u64), CleanupMode::NoEmpty).unwrap();
			state.commit().unwrap();
			assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
			state.drop()
		};

		let state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
		assert_eq!(state.nonce(&a).unwrap(), U256::from(1u64));
	}

	#[test]
	fn remove() {
		let a = Address::zero();
		let mut state = get_temp_state();
		assert_eq!(state.exists(&a).unwrap(), false);
		assert_eq!(state.exists_and_not_null(&a).unwrap(), false);
		state.inc_nonce(&a).unwrap();
		assert_eq!(state.exists(&a).unwrap(), true);
		assert_eq!(state.exists_and_not_null(&a).unwrap(), true);
		assert_eq!(state.nonce(&a).unwrap(), U256::from(1u64));
		state.kill_account(&a);
		assert_eq!(state.exists(&a).unwrap(), false);
		assert_eq!(state.exists_and_not_null(&a).unwrap(), false);
		assert_eq!(state.nonce(&a).unwrap(), U256::from(0u64));
	}

	#[test]
	fn empty_account_is_not_created() {
		let a = Address::zero();
		let db = get_temp_state_db();
		let (root, db) = {
			let mut state = State::new(db, U256::from(0), Default::default());
			state.add_balance(&a, &U256::default(), CleanupMode::NoEmpty).unwrap(); // create an empty account
			state.commit().unwrap();
			state.drop()
		};
		let state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		assert!(!state.exists(&a).unwrap());
		assert!(!state.exists_and_not_null(&a).unwrap());
	}

	#[test]
	fn empty_account_exists_when_creation_forced() {
		let a = Address::zero();
		let db = get_temp_state_db();
		let (root, db) = {
			let mut state = State::new(db, U256::from(0), Default::default());
			state.add_balance(&a, &U256::default(), CleanupMode::ForceCreate).unwrap(); // create an empty account
			state.commit().unwrap();
			state.drop()
		};
		let state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		assert!(state.exists(&a).unwrap());
		assert!(!state.exists_and_not_null(&a).unwrap());
	}

	#[test]
	fn remove_from_database() {
		let a = Address::zero();
		let (root, db) = {
			let mut state = get_temp_state();
			state.inc_nonce(&a).unwrap();
			state.commit().unwrap();
			assert_eq!(state.exists(&a).unwrap(), true);
			assert_eq!(state.nonce(&a).unwrap(), U256::from(1u64));
			state.drop()
		};

		let (root, db) = {
			let mut state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
			assert_eq!(state.exists(&a).unwrap(), true);
			assert_eq!(state.nonce(&a).unwrap(), U256::from(1u64));
			state.kill_account(&a);
			state.commit().unwrap();
			assert_eq!(state.exists(&a).unwrap(), false);
			assert_eq!(state.nonce(&a).unwrap(), U256::from(0u64));
			state.drop()
		};

		let state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		assert_eq!(state.exists(&a).unwrap(), false);
		assert_eq!(state.nonce(&a).unwrap(), U256::from(0u64));
	}

	#[test]
	fn alter_balance() {
		let mut state = get_temp_state();
		let a = Address::zero();
		let b = Address::from_low_u64_be(1u64);
		state.add_balance(&a, &U256::from(69u64), CleanupMode::NoEmpty).unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
		state.commit().unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
		state.sub_balance(&a, &U256::from(42u64), &mut CleanupMode::NoEmpty).unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(27u64));
		state.commit().unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(27u64));
		state.transfer_balance(&a, &b, &U256::from(18u64), CleanupMode::NoEmpty).unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(9u64));
		assert_eq!(state.balance(&b).unwrap(), U256::from(18u64));
		state.commit().unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(9u64));
		assert_eq!(state.balance(&b).unwrap(), U256::from(18u64));
	}

	#[test]
	fn alter_nonce() {
		let mut state = get_temp_state();
		let a = Address::zero();
		state.inc_nonce(&a).unwrap();
		assert_eq!(state.nonce(&a).unwrap(), U256::from(1u64));
		state.inc_nonce(&a).unwrap();
		assert_eq!(state.nonce(&a).unwrap(), U256::from(2u64));
		state.commit().unwrap();
		assert_eq!(state.nonce(&a).unwrap(), U256::from(2u64));
		state.inc_nonce(&a).unwrap();
		assert_eq!(state.nonce(&a).unwrap(), U256::from(3u64));
		state.commit().unwrap();
		assert_eq!(state.nonce(&a).unwrap(), U256::from(3u64));
	}

	#[test]
	fn balance_nonce() {
		let mut state = get_temp_state();
		let a = Address::zero();
		assert_eq!(state.balance(&a).unwrap(), U256::from(0u64));
		assert_eq!(state.nonce(&a).unwrap(), U256::from(0u64));
		state.commit().unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(0u64));
		assert_eq!(state.nonce(&a).unwrap(), U256::from(0u64));
	}

	#[test]
	fn ensure_cached() {
		let mut state = get_temp_state();
		let a = Address::zero();
		state.require(&a, false).unwrap();
		state.commit().unwrap();
		assert_eq!(*state.root(), H256::from_str("0ce23f3c809de377b008a4a3ee94a0834aac8bec1f86e28ffe4fdb5a15b0c785").unwrap());
	}

	#[test]
	fn checkpoint_basic() {
		let mut state = get_temp_state();
		let a = Address::zero();
		state.checkpoint();
		state.add_balance(&a, &U256::from(69u64), CleanupMode::NoEmpty).unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
		state.discard_checkpoint();
		assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
		state.checkpoint();
		state.add_balance(&a, &U256::from(1u64), CleanupMode::NoEmpty).unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(70u64));
		state.revert_to_checkpoint();
		assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
	}

	#[test]
	fn checkpoint_nested() {
		let mut state = get_temp_state();
		let a = Address::zero();
		state.checkpoint();
		state.checkpoint();
		state.add_balance(&a, &U256::from(69u64), CleanupMode::NoEmpty).unwrap();
		assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
		state.discard_checkpoint();
		assert_eq!(state.balance(&a).unwrap(), U256::from(69u64));
		state.revert_to_checkpoint();
		assert_eq!(state.balance(&a).unwrap(), U256::from(0));
	}

	#[test]
	fn checkpoint_revert_to_get_storage_at() {
		let mut state = get_temp_state();
		let a = Address::zero();
		let k = BigEndianHash::from_uint(&U256::from(0));

		let c0 = state.checkpoint();
		let c1 = state.checkpoint();
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(1))).unwrap();

		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(1)));

		state.revert_to_checkpoint(); // Revert to c1.
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(0)));
	}

	#[test]
	fn checkpoint_from_empty_get_storage_at() {
		let mut state = get_temp_state();
		let a = Address::zero();
		let k = BigEndianHash::from_uint(&U256::from(0));
		let k2 = BigEndianHash::from_uint(&U256::from(1));

		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(0)));
		state.clear();

		let c0 = state.checkpoint();
		state.new_contract(&a, U256::zero(), U256::zero(), U256::zero()).unwrap();
		let c1 = state.checkpoint();
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(1))).unwrap();
		let c2 = state.checkpoint();
		let c3 = state.checkpoint();
		state.set_storage(&a, k2, BigEndianHash::from_uint(&U256::from(3))).unwrap();
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(3))).unwrap();
		let c4 = state.checkpoint();
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(4))).unwrap();
		let c5 = state.checkpoint();

		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c2, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c3, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c4, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(3))));
		assert_eq!(state.checkpoint_storage_at(c5, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(4))));

		state.discard_checkpoint(); // Commit/discard c5.
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c2, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c3, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c4, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(3))));

		state.revert_to_checkpoint(); // Revert to c4.
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c2, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c3, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));

		state.discard_checkpoint(); // Commit/discard c3.
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c2, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));

		state.revert_to_checkpoint(); // Revert to c2.
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));

		state.discard_checkpoint(); // Commit/discard c1.
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
	}

	#[test]
	fn checkpoint_get_storage_at() {
		let mut state = get_temp_state();
		let a = Address::zero();
		let k = BigEndianHash::from_uint(&U256::from(0));
		let k2 = BigEndianHash::from_uint(&U256::from(1));

		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(0xffff))).unwrap();
		state.commit().unwrap();
		state.clear();

		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(0xffff)));
		state.clear();

		let cm1 = state.checkpoint();
		let c0 = state.checkpoint();
		state.new_contract(&a, U256::zero(), U256::zero(), U256::zero()).unwrap();
		let c1 = state.checkpoint();
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(1))).unwrap();
		let c2 = state.checkpoint();
		let c3 = state.checkpoint();
		state.set_storage(&a, k2, BigEndianHash::from_uint(&U256::from(3))).unwrap();
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(3))).unwrap();
		let c4 = state.checkpoint();
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(4))).unwrap();
		let c5 = state.checkpoint();

		assert_eq!(state.checkpoint_storage_at(cm1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c2, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c3, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c4, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(3))));
		assert_eq!(state.checkpoint_storage_at(c5, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(4))));

		state.discard_checkpoint(); // Commit/discard c5.
		assert_eq!(state.checkpoint_storage_at(cm1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c2, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c3, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c4, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(3))));

		state.revert_to_checkpoint(); // Revert to c4.
		assert_eq!(state.checkpoint_storage_at(cm1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c2, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));
		assert_eq!(state.checkpoint_storage_at(c3, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));

		state.discard_checkpoint(); // Commit/discard c3.
		assert_eq!(state.checkpoint_storage_at(cm1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));
		assert_eq!(state.checkpoint_storage_at(c2, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(1))));

		state.revert_to_checkpoint(); // Revert to c2.
		assert_eq!(state.checkpoint_storage_at(cm1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0))));

		state.discard_checkpoint(); // Commit/discard c1.
		assert_eq!(state.checkpoint_storage_at(cm1, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
		assert_eq!(state.checkpoint_storage_at(c0, &a, &k).unwrap(), Some(BigEndianHash::from_uint(&U256::from(0xffff))));
	}

	#[test]
	fn kill_account_with_checkpoints() {
		let mut state = get_temp_state();
		let a = Address::zero();
		let k = BigEndianHash::from_uint(&U256::from(0));
		state.checkpoint();
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(1))).unwrap();
		state.checkpoint();
		state.kill_account(&a);

		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(0)));
		state.revert_to_checkpoint();
		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(1)));
	}

	#[test]
	fn create_contract_fail() {
		let mut state = get_temp_state();
		let orig_root = state.root().clone();
		let a = Address::from_low_u64_be(1000);

		state.checkpoint(); // c1
		state.new_contract(&a, U256::zero(), U256::zero(), U256::zero()).unwrap();
		state.add_balance(&a, &U256::from(1), CleanupMode::ForceCreate).unwrap();
		state.checkpoint(); // c2
		state.add_balance(&a, &U256::from(1), CleanupMode::ForceCreate).unwrap();
		state.discard_checkpoint(); // discard c2
		state.revert_to_checkpoint(); // revert to c1
		assert_eq!(state.exists(&a).unwrap(), false);

		state.commit().unwrap();
		assert_eq!(orig_root, state.root().clone());
	}

	#[test]
	fn create_contract_fail_previous_storage() {
		let mut state = get_temp_state();
		let a = Address::from_low_u64_be(1000);
		let k = BigEndianHash::from_uint(&U256::from(0));

		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(0xffff))).unwrap();
		state.commit().unwrap();
		state.clear();

		let orig_root = state.root().clone();
		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(0xffff)));
		state.clear();

		state.checkpoint(); // c1
		state.new_contract(&a, U256::zero(), U256::zero(), U256::zero()).unwrap();
		state.checkpoint(); // c2
		state.set_storage(&a, k, BigEndianHash::from_uint(&U256::from(2))).unwrap();
		state.revert_to_checkpoint(); // revert to c2
		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(0)));
		state.revert_to_checkpoint(); // revert to c1
		assert_eq!(state.storage_at(&a, &k).unwrap(), BigEndianHash::from_uint(&U256::from(0xffff)));

		state.commit().unwrap();
		assert_eq!(orig_root, state.root().clone());
	}

	#[test]
	fn create_empty() {
		let mut state = get_temp_state();
		state.commit().unwrap();
		assert_eq!(*state.root(), H256::from_str("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").unwrap());
	}

	#[test]
	fn should_not_panic_on_state_diff_with_storage() {
		let mut state = get_temp_state();
		let a = Address::from_low_u64_be(0xa);
		state.init_code(&a, b"abcdefg".to_vec()).unwrap();
		state.add_balance(&a, &256.into(), CleanupMode::NoEmpty).unwrap();
		state.set_storage(&a, H256::from_low_u64_be(0xb), H256::from_low_u64_be(0xc).into()).unwrap();

		let mut new_state = state.clone();
		new_state.set_storage(&a, H256::from_low_u64_be(0xb), H256::from_low_u64_be(0xd).into()).unwrap();

		new_state.diff_from(state).unwrap();
	}

	#[test]
	fn should_kill_garbage() {
		let a = Address::from_low_u64_be(10);
		let b = Address::from_low_u64_be(20);
		let c = Address::from_low_u64_be(30);
		let d = Address::from_low_u64_be(40);
		let e = Address::from_low_u64_be(50);
		let x = Address::from_low_u64_be(0);
		let db = get_temp_state_db();
		let (root, db) = {
			let mut state = State::new(db, U256::from(0), Default::default());
			state.add_balance(&a, &U256::default(), CleanupMode::ForceCreate).unwrap(); // create an empty account
			state.add_balance(&b, &100.into(), CleanupMode::ForceCreate).unwrap(); // create a dust account
			state.add_balance(&c, &101.into(), CleanupMode::ForceCreate).unwrap(); // create a normal account
			state.add_balance(&d, &99.into(), CleanupMode::ForceCreate).unwrap(); // create another dust account
			state.new_contract(&e, 100.into(), 1.into(), 0.into()).unwrap(); // create a contract account
			state.init_code(&e, vec![0x00]).unwrap();
			state.commit().unwrap();
			state.drop()
		};

		let mut state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		let mut touched = HashSet::new();
		state.add_balance(&a, &U256::default(), CleanupMode::TrackTouched(&mut touched)).unwrap(); // touch an account
		state.transfer_balance(&b, &x, &1.into(), CleanupMode::TrackTouched(&mut touched)).unwrap(); // touch an account decreasing its balance
		state.transfer_balance(&c, &x, &1.into(), CleanupMode::TrackTouched(&mut touched)).unwrap(); // touch an account decreasing its balance
		state.transfer_balance(&e, &x, &1.into(), CleanupMode::TrackTouched(&mut touched)).unwrap(); // touch an account decreasing its balance
		state.kill_garbage(&touched, true, &None, false).unwrap();
		assert!(!state.exists(&a).unwrap());
		assert!(state.exists(&b).unwrap());
		state.kill_garbage(&touched, true, &Some(100.into()), false).unwrap();
		assert!(!state.exists(&b).unwrap());
		assert!(state.exists(&c).unwrap());
		assert!(state.exists(&d).unwrap());
		assert!(state.exists(&e).unwrap());
		state.kill_garbage(&touched, true, &Some(100.into()), true).unwrap();
		assert!(state.exists(&c).unwrap());
		assert!(state.exists(&d).unwrap());
		assert!(!state.exists(&e).unwrap());
	}

	#[test]
	fn should_trace_diff_suicided_accounts() {
		let a = Address::from_low_u64_be(10);
		let db = get_temp_state_db();
		let (root, db) = {
			let mut state = State::new(db, U256::from(0), Default::default());
			state.add_balance(&a, &100.into(), CleanupMode::ForceCreate).unwrap();
			state.commit().unwrap();
			state.drop()
		};

		let mut state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		let original = state.clone();
		state.kill_account(&a);

		let diff = state.diff_from(original).unwrap();
		let diff_map = diff.raw;
		assert_eq!(diff_map.len(), 1);
		assert!(diff_map.get(&a).is_some());
		assert_eq!(diff_map.get(&a),
			pod::account::diff_pod(
				Some(&PodAccount {
					balance: U256::from(100),
					nonce: U256::zero(),
					code: Some(Default::default()),
					storage: Default::default(),
					version: U256::zero(),
				}), None).as_ref());
	}

	#[test]
	fn should_trace_diff_unmodified_storage() {
		let a = Address::from_low_u64_be(10);
		let db = get_temp_state_db();

		let (root, db) = {
			let mut state = State::new(db, U256::from(0), Default::default());
			state.set_storage(&a, BigEndianHash::from_uint(&U256::from(1u64)), BigEndianHash::from_uint(&U256::from(20u64))).unwrap();
			state.commit().unwrap();
			state.drop()
		};

		let mut state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		let original = state.clone();
		state.set_storage(&a, BigEndianHash::from_uint(&U256::from(1u64)), BigEndianHash::from_uint(&U256::from(100u64))).unwrap();

		let diff = state.diff_from(original).unwrap();
		let diff_map = diff.raw;
		assert_eq!(diff_map.len(), 1);
		assert!(diff_map.get(&a).is_some());
		assert_eq!(diff_map.get(&a),
			pod::account::diff_pod(
				Some(&PodAccount {
						balance: U256::zero(),
						nonce: U256::zero(),
						code: Some(Default::default()),
						storage: vec![(BigEndianHash::from_uint(&U256::from(1u64)), BigEndianHash::from_uint(&U256::from(20u64)))].into_iter().collect(),
						version: U256::zero(),
					}),
					Some(&PodAccount {
						balance: U256::zero(),
						nonce: U256::zero(),
						code: Some(Default::default()),
						storage: vec![(BigEndianHash::from_uint(&U256::from(1u64)), BigEndianHash::from_uint(&U256::from(100u64)))].into_iter().collect(),
						version: U256::zero(),
					})).as_ref());
	}

	#[test]
	fn should_get_full_pod_storage_values() {
		let a = Address::from_low_u64_be(10);
		let db = get_temp_state_db();

		let factories = Factories {
			vm: Default::default(),
			trie: TrieFactory::new(TrieSpec::Fat, ethtrie::Layout),
			accountdb: Default::default(),
		};

		let get_pod_state_val = |pod_state : &PodState, ak, k| {
			pod_state.get().get(ak).unwrap().storage.get(&k).unwrap().clone()
		};

		let storage_address: H256 = BigEndianHash::from_uint(&U256::from(1u64));

		let (root, db) = {
			let mut state = State::new(db, U256::from(0), factories.clone());
			state.set_storage(&a, storage_address.clone(), BigEndianHash::from_uint(&U256::from(20u64))).unwrap();
			let dump = state.to_pod_full().unwrap();
			assert_eq!(get_pod_state_val(&dump, &a, storage_address.clone()), BigEndianHash::from_uint(&U256::from(20u64)));
			state.commit().unwrap();
			let dump = state.to_pod_full().unwrap();
			assert_eq!(get_pod_state_val(&dump, &a, storage_address.clone()), BigEndianHash::from_uint(&U256::from(20u64)));
			state.drop()
		};

		let mut state = State::from_existing(db, root, U256::from(0u8), factories).unwrap();
		let dump = state.to_pod_full().unwrap();
		assert_eq!(get_pod_state_val(&dump, &a, storage_address.clone()), BigEndianHash::from_uint(&U256::from(20u64)));
		state.set_storage(&a, storage_address.clone(), BigEndianHash::from_uint(&U256::from(21u64))).unwrap();
		let dump = state.to_pod_full().unwrap();
		assert_eq!(get_pod_state_val(&dump, &a, storage_address.clone()), BigEndianHash::from_uint(&U256::from(21u64)));
		state.commit().unwrap();
		state.set_storage(&a, storage_address.clone(), BigEndianHash::from_uint(&U256::from(0u64))).unwrap();
		let dump = state.to_pod_full().unwrap();
		assert_eq!(get_pod_state_val(&dump, &a, storage_address.clone()), BigEndianHash::from_uint(&U256::from(0u64)));
	}
}
