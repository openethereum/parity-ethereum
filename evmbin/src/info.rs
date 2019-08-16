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

//! EVM runner.

use std::time::{Instant, Duration};

use common_types::transaction;
use ethcore::{
	client::{EvmTestClient, EvmTestError, TransactErr, TransactSuccess},
	TrieSpec,
};
use ethereum_types::{H256, U256};
use ethjson;
use pod::PodState;
use spec;
use trace;
use vm::ActionParams;

/// EVM execution informant.
pub trait Informant: trace::VMTracer {
	/// Sink to use with finish
	type Sink;
	/// Display a single run init message
	fn before_test(&mut self, test: &str, action: &str);
	/// Set initial gas.
	fn set_gas(&mut self, _gas: U256) {}
	/// Clone sink.
	fn clone_sink(&self) -> Self::Sink;
	/// Display final result.
	fn finish(result: RunResult<Self::Output>, _: &mut Self::Sink);
}

/// Execution finished correctly.
#[derive(Debug)]
pub struct Success<T> {
	/// State root.
	pub state_root: H256,
	/// Used gas.
	pub gas_used: U256,
	/// Output as bytes.
	pub output: Vec<u8>,
	/// Time taken.
	pub time: Duration,
	/// Traces.
	pub traces: Option<T>,
	/// Optional end state dump
	pub end_state: Option<PodState>,
}

/// Execution failed.
#[derive(Debug)]
pub struct Failure<T> {
	/// State root.
	pub state_root: H256,
	/// Used gas.
	pub gas_used: U256,
	/// Internal error.
	pub error: EvmTestError,
	/// Duration.
	pub time: Duration,
	/// Traces.
	pub traces: Option<T>,
	/// Optional end state dump
	pub end_state: Option<PodState>,
}

/// EVM execution result.
pub type RunResult<T> = Result<Success<T>, Failure<T>>;

/// Execute given `ActionParams` and return the result.
pub fn run_action<T: Informant>(
	spec: &spec::Spec,
	mut params: ActionParams,
	mut informant: T,
	trie_spec: TrieSpec,
) -> RunResult<T::Output> {
	informant.set_gas(params.gas);

	// if the code is not overwritten from CLI, use code from spec file.
	if params.code.is_none() {
		if let Some(acc) = spec.genesis_state.get().get(&params.code_address) {
			params.code = acc.code.clone().map(::std::sync::Arc::new);
			params.code_hash = None;
		}
	}
	run(spec, trie_spec, params.gas, &spec.genesis_state, |mut client| {
		let result = match client.call(params, &mut trace::NoopTracer, &mut informant) {
			Ok(r) => (Ok(r.return_data.to_vec()), Some(r.gas_left)),
			Err(err) => (Err(err), None),
		};
		(result.0, H256::from_low_u64_be(0), None, result.1, informant.drain())
	})
}

/// Input data to run transaction.
#[derive(Debug)]
pub struct TxInput<'a, T> {
	/// State test name associated with the transaction.
	pub state_test_name: &'a str,
	/// Transaction index from list of transactions within a state root hash corresponding to a chain.
	pub tx_index: usize,
	/// Fork specification (i.e. Constantinople, EIP150, EIP158, etc).
	pub fork_spec_name: &'a ethjson::spec::ForkSpec,
	/// State of all accounts in the system that is a binary tree mapping of each account address to account data
	/// that is expressed as Plain Old Data containing the account balance, account nonce, account code in bytes,
	/// and the account storage binary tree map.
	pub pre_state: &'a PodState,
	/// State root hash associated with the transaction.
	pub post_root: H256,
	/// Client environment information associated with the transaction's chain specification.
	pub env_info: &'a vm::EnvInfo,
	/// Signed transaction accompanied by a signature that may be unverified and a successfully recovered
	/// sender address. The unverified transaction contains a recoverable ECDSA signature that has been encoded
	/// as RSV components and includes replay protection for the specified chain. Verification of the signed transaction
	/// with a valid secret of an account's keypair and a specific chain may be used to recover the sender's public key
	/// and their associated address by applying the Keccak-256 hash function.
	pub transaction: transaction::SignedTransaction,
	/// JSON formatting informant.
	pub informant: T,
	/// Trie specification (i.e. Generic trie, Secure trie, Secure with fat database).
	pub trie_spec: TrieSpec,
}

/// Execute given transaction and verify resulting state root.
/// Returns true if the transaction executes successfully.
pub fn run_transaction<T: Informant>(
	tx_input: TxInput<T>
) -> bool {
	let TxInput {
		state_test_name, tx_index, fork_spec_name, pre_state, post_root, env_info, transaction, mut informant, trie_spec, ..
	} = tx_input;
	let fork_spec_name_formatted = format!("{:?}", fork_spec_name).to_lowercase();
	let fork_spec = match EvmTestClient::fork_spec_from_json(&fork_spec_name) {
		Some(spec) => {
			informant.before_test(
				&format!("{}:{}:{}", &state_test_name, &fork_spec_name_formatted, tx_index), "starting");
			spec
		},
		None => {
			informant.before_test(&format!("{}:{}:{}",
				&state_test_name, fork_spec_name_formatted, &tx_index), "skipping because of missing fork specification");
			return false;
		},
	};

	informant.set_gas(env_info.gas_limit);

	let mut sink = informant.clone_sink();
	let result = run(&fork_spec, trie_spec, transaction.gas, &pre_state, |mut client| {
		let result = client.transact(&env_info, transaction, trace::NoopTracer, informant);
		match result {
			Ok(TransactSuccess { state_root, gas_left, output, vm_trace, end_state, .. }) => {
				if state_root != post_root {
					(Err(EvmTestError::PostCondition(format!(
						"State root mismatch (got: {:#x}, expected: {:#x})",
						state_root,
						post_root,
					))), state_root, end_state, Some(gas_left), None)
				} else {
					(Ok(output), state_root, end_state, Some(gas_left), vm_trace)
				}
			},
			Err(TransactErr { state_root, error, end_state }) => {
				(Err(EvmTestError::PostCondition(format!(
					"Unexpected execution error: {:?}", error
				))), state_root, end_state, None, None)
			},
		}
	});

	let ok = result.is_ok();
	T::finish(result, &mut sink);
	ok
}

/// Execute EVM with given `ActionParams`.
pub fn run<'a, F, X>(
	spec: &'a spec::Spec,
	trie_spec: TrieSpec,
	initial_gas: U256,
	pre_state: &'a PodState,
	run: F,
) -> RunResult<X> where
	F: FnOnce(EvmTestClient) -> (Result<Vec<u8>, EvmTestError>, H256, Option<PodState>, Option<U256>, Option<X>),
{
	let do_dump = trie_spec == TrieSpec::Fat;

	let mut test_client = EvmTestClient::from_pod_state_with_trie(spec, pre_state.clone(), trie_spec)
		.map_err(|error| Failure {
			gas_used: 0.into(),
			error,
			time: Duration::from_secs(0),
			traces: None,
			state_root: H256::zero(),
			end_state: None,
		})?;

	if do_dump {
		test_client.set_dump_state();
	}

	let start = Instant::now();
	let result = run(test_client);
	let time = start.elapsed();

	match result {
		(Ok(output), state_root, end_state, gas_left, traces) => Ok(Success {
			state_root,
			gas_used: gas_left.map(|gas_left| initial_gas - gas_left).unwrap_or(initial_gas),
			output,
			time,
			traces,
			end_state,
		}),
		(Err(error), state_root, end_state, gas_left, traces) => Err(Failure {
			gas_used: gas_left.map(|gas_left| initial_gas - gas_left).unwrap_or(initial_gas),
			error,
			time,
			traces,
			state_root,
			end_state,
		}),
	}
}

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use rustc_hex::FromHex;
	use super::*;
	use tempdir::TempDir;
	use ethereum_types::Address;
	use spec::{self, Spec};

	pub fn run_test<T, I, F>(
		informant: I,
		compare: F,
		code: &str,
		gas: T,
		expected: &str,
	) where
		T: Into<U256>,
		I: Informant,
		F: FnOnce(Option<I::Output>, &str),
	{
		let mut params = ActionParams::default();
		params.code = Some(Arc::new(code.from_hex().unwrap()));
		params.gas = gas.into();

		let tempdir = TempDir::new("").unwrap();
		let spec = spec::new_foundation(&tempdir.path());
		let result = run_action(&spec, params, informant, TrieSpec::Secure);
		match result {
			Ok(Success { traces, .. }) => {
				compare(traces, expected)
			},
			Err(Failure { traces, .. }) => {
				compare(traces, expected)
			},
		}
	}

	#[test]
	fn should_call_account_from_spec() {
		use crate::display::std_json::tests::informant;

		let (inf, res, _) = informant();
		let mut params = ActionParams::default();
		params.code_address = Address::from_low_u64_be(0x20);
		params.gas = 0xffff.into();

		let tempdir = TempDir::new("").unwrap();
		let spec = Spec::load(&tempdir.path(), include_bytes!("../res/testchain.json") as &[u8]).unwrap();
		let _result = run_action(&spec, params, inf, TrieSpec::Secure);

		assert_eq!(
			&String::from_utf8_lossy(&**res.0.lock().unwrap()),
r#"{"pc":0,"op":98,"opName":"PUSH3","gas":"0xffff","stack":[],"storage":{},"depth":1}
{"pc":4,"op":96,"opName":"PUSH1","gas":"0xfffc","stack":["0xaaaaaa"],"storage":{},"depth":1}
{"pc":6,"op":96,"opName":"PUSH1","gas":"0xfff9","stack":["0xaaaaaa","0xaa"],"storage":{},"depth":1}
{"pc":8,"op":80,"opName":"POP","gas":"0xfff6","stack":["0xaaaaaa","0xaa","0xaa"],"storage":{},"depth":1}
{"pc":9,"op":96,"opName":"PUSH1","gas":"0xfff4","stack":["0xaaaaaa","0xaa"],"storage":{},"depth":1}
{"pc":11,"op":96,"opName":"PUSH1","gas":"0xfff1","stack":["0xaaaaaa","0xaa","0xaa"],"storage":{},"depth":1}
{"pc":13,"op":96,"opName":"PUSH1","gas":"0xffee","stack":["0xaaaaaa","0xaa","0xaa","0xaa"],"storage":{},"depth":1}
{"pc":15,"op":96,"opName":"PUSH1","gas":"0xffeb","stack":["0xaaaaaa","0xaa","0xaa","0xaa","0xaa"],"storage":{},"depth":1}
{"pc":17,"op":96,"opName":"PUSH1","gas":"0xffe8","stack":["0xaaaaaa","0xaa","0xaa","0xaa","0xaa","0xaa"],"storage":{},"depth":1}
{"pc":19,"op":96,"opName":"PUSH1","gas":"0xffe5","stack":["0xaaaaaa","0xaa","0xaa","0xaa","0xaa","0xaa","0xaa"],"storage":{},"depth":1}
"#);
	}
}
