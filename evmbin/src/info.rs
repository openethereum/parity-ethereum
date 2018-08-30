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

//! VM runner.

use std::time::{Instant, Duration};
use ethereum_types::{H256, U256};
use ethcore::client::{self, EvmTestClient, EvmTestError, TransactResult};
use ethcore::{trace, spec, pod_state};
use ethjson;
use transaction;
use vm::ActionParams;

/// VM execution informant
pub trait Informant: trace::VMTracer {
	/// Display a single run init message
	fn before_test(&mut self, test: &str, action: &str);
	/// Set initial gas.
	fn set_gas(&mut self, _gas: U256) {}
	/// Display final result.
	fn finish(result: RunResult<Self::Output>);
}

/// Execution finished correctly
#[derive(Debug)]
pub struct Success<T> {
	/// State root
	pub state_root: H256,
	/// Used gas
	pub gas_used: U256,
	/// Output as bytes
	pub output: Vec<u8>,
	/// Time Taken
	pub time: Duration,
	/// Traces
	pub traces: Option<T>,
}

/// Execution failed
#[derive(Debug)]
pub struct Failure<T> {
	/// Used gas
	pub gas_used: U256,
	/// Internal error
	pub error: EvmTestError,
	/// Duration
	pub time: Duration,
	/// Traces
	pub traces: Option<T>,
}

/// EVM Execution result
pub type RunResult<T> = Result<Success<T>, Failure<T>>;

/// Execute given `ActionParams` and return the result.
pub fn run_action<T: Informant>(
	spec: &spec::Spec,
	mut params: ActionParams,
	mut informant: T,
) -> RunResult<T::Output> {
	informant.set_gas(params.gas);

	// if the code is not overwritten from CLI, use code from spec file.
	if params.code.is_none() {
		if let Some(acc) = spec.genesis_state().get().get(&params.code_address) {
			params.code = acc.code.clone().map(::std::sync::Arc::new);
			params.code_hash = None;
		}
	}
	run(spec, params.gas, spec.genesis_state(), |mut client| {
		let result = match client.call(params, &mut trace::NoopTracer, &mut informant) {
			Ok(r) => (Ok((0.into(), r.return_data.to_vec())), Some(r.gas_left)),
			Err(err) => (Err(err), None),
		};
		(result.0, result.1, informant.drain())
	})
}

/// Execute given Transaction and verify resulting state root.
pub fn run_transaction<T: Informant>(
	name: &str,
	idx: usize,
	spec: &ethjson::state::test::ForkSpec,
	pre_state: &pod_state::PodState,
	post_root: H256,
	env_info: &client::EnvInfo,
	transaction: transaction::SignedTransaction,
	mut informant: T,
) {
	let spec_name = format!("{:?}", spec).to_lowercase();
	let spec = match EvmTestClient::spec_from_json(spec) {
		Some(spec) => {
			informant.before_test(&format!("{}:{}:{}", name, spec_name, idx), "starting");
			spec
		},
		None => {
			informant.before_test(&format!("{}:{}:{}", name, spec_name, idx), "skipping because of missing spec");
			return;
		},
	};

	informant.set_gas(env_info.gas_limit);

	let result = run(&spec, env_info.gas_limit, pre_state, |mut client| {
		let result = client.transact(env_info, transaction, trace::NoopTracer, informant);
		match result {
			TransactResult::Ok { state_root, gas_left, .. } if state_root != post_root => {
				(Err(EvmTestError::PostCondition(format!(
					"State root mismatch (got: 0x{:x}, expected: 0x{:x})",
					state_root,
					post_root,
				))), Some(gas_left), None)
			},
			TransactResult::Ok { state_root, gas_left, output, vm_trace, .. } => {
				(Ok((state_root, output)), Some(gas_left), vm_trace)
			},
			TransactResult::Err { error, .. } => {
				(Err(EvmTestError::PostCondition(format!(
					"Unexpected execution error: {:?}", error
				))), None, None)
			},
		}
	});

	T::finish(result)
}

/// Execute VM with given `ActionParams`
pub fn run<'a, F, X>(
	spec: &'a spec::Spec,
	initial_gas: U256,
	pre_state: &'a pod_state::PodState,
	run: F,
) -> RunResult<X> where
	F: FnOnce(EvmTestClient) -> (Result<(H256, Vec<u8>), EvmTestError>, Option<U256>, Option<X>),
{
	let test_client = EvmTestClient::from_pod_state(spec, pre_state.clone())
		.map_err(|error| Failure {
			gas_used: 0.into(),
			error,
			time: Duration::from_secs(0),
			traces: None,
		})?;

	let start = Instant::now();
	let result = run(test_client);
	let time = start.elapsed();

	match result {
		(Ok((state_root, output)), gas_left, traces) => Ok(Success {
			state_root,
			gas_used: gas_left.map(|gas_left| initial_gas - gas_left).unwrap_or(initial_gas),
			output,
			time,
			traces,
		}),
		(Err(error), gas_left, traces) => Err(Failure {
			gas_used: gas_left.map(|gas_left| initial_gas - gas_left).unwrap_or(initial_gas),
			error,
			time,
			traces,
		}),
	}
}

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use rustc_hex::FromHex;
	use super::*;
	use tempdir::TempDir;

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
		let spec = ::ethcore::ethereum::new_foundation(&tempdir.path());
		let result = run_action(&spec, params, informant);
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
		use display::std_json::tests::informant;

		let (inf, res) = informant();
		let mut params = ActionParams::default();
		params.code_address = 0x20.into();
		params.gas = 0xffff.into();

		let spec = ::ethcore::ethereum::load(None, include_bytes!("../res/testchain.json"));
		let _result = run_action(&spec, params, inf);

		assert_eq!(
			&String::from_utf8_lossy(&**res.lock().unwrap()),
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
