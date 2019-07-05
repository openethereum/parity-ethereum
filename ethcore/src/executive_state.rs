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
//! `state-account` crates and contains everything that requires `Machine` or `Executive` (or types
//! thereof).

use machine::Machine;
use vm::EnvInfo;
use executive::{Executive, TransactOptions};
use executed::{Executed, ExecutionError};
use types::{
	transaction::SignedTransaction,
	receipt::{TransactionOutcome, Receipt},
};
use trace::{FlatTrace, VMTrace};
use state_account::{
	backend::{self, Backend},
	state::State,
};
use ethereum_types::H256;
use factories::Factories;
use bytes::Bytes;
use keccak_hasher::KeccakHasher;
use kvdb::DBValue;
use hash_db::AsHashDB;

use error::Error;

// TODO: is there a better place for this?
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
	factories.accountdb = ::account_db::Factory::Plain;

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
	use state_account::backend::Proving;

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
pub trait ExecutiveStateWithMachineZomgBetterName {
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

impl<B: Backend> ExecutiveStateWithMachineZomgBetterName for State<B> {
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
) -> Result<Executed<T::Output, V::Output>, ExecutionError>
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
