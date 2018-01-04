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

//! VM runner.

use std::time::{Instant, Duration};
use ethereum_types::{H256, U256};
use ethcore::{trace, spec, transaction, pod_state};
use ethcore::client::{self, EvmTestClient, EvmTestError, TransactResult};
use ethjson;
use vm::ActionParams;

/// VM execution informant
pub trait Informant: trace::VMTracer {
	/// Display a single run init message
	fn before_test(&self, test: &str, action: &str);
	/// Set initial gas.
	fn set_gas(&mut self, _gas: U256) {}
	/// Display final result.
	fn finish(result: RunResult<Self::Output>);
}

/// Execution finished correctly
#[derive(Debug)]
pub struct Success<T> {
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
	params: ActionParams,
	mut informant: T,
) -> RunResult<T::Output> {
	informant.set_gas(params.gas);
	run(spec, params.gas, None, |mut client| {
		let result = client
			.call(params, &mut informant)
			.map(|r| (r.gas_left, r.return_data.to_vec()));
		(result, informant.drain())
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

	let result = run(spec, env_info.gas_limit, pre_state, |mut client| {
		let result = client.transact(env_info, transaction, informant);
		match result {
			TransactResult::Ok { state_root, .. } if state_root != post_root => {
				(Err(EvmTestError::PostCondition(format!(
					"State root mismatch (got: {}, expected: {})",
					state_root,
					post_root,
				))), None)
			},
			TransactResult::Ok { gas_left, output, vm_trace, .. } => {
				(Ok((gas_left, output)), vm_trace)
			},
			TransactResult::Err { error, .. } => {
				(Err(EvmTestError::PostCondition(format!(
					"Unexpected execution error: {:?}", error
				))), None)
			},
		}
	});

	T::finish(result)
}

/// Execute VM with given `ActionParams`
pub fn run<'a, F, T, X>(
	spec: &'a spec::Spec,
	initial_gas: U256,
	pre_state: T,
	run: F,
) -> RunResult<X> where
	F: FnOnce(EvmTestClient) -> (Result<(U256, Vec<u8>), EvmTestError>, Option<X>),
	T: Into<Option<&'a pod_state::PodState>>,
{
	let test_client = match pre_state.into() {
		Some(pre_state) => EvmTestClient::from_pod_state(spec, pre_state.clone()),
		None => EvmTestClient::new(spec),
	}.map_err(|error| Failure {
		gas_used: 0.into(),
		error,
		time: Duration::from_secs(0),
		traces: None,
	})?;

	let start = Instant::now();
	let result = run(test_client);
	let time = start.elapsed();

	match result {
		(Ok((gas_left, output)), traces) => Ok(Success {
			gas_used: initial_gas - gas_left,
			output,
			time,
			traces,
		}),
		(Err(error), traces) => Err(Failure {
			gas_used: initial_gas,
			error,
			time,
			traces,
		}),
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use rustc_hex::FromHex;
	use super::*;

	#[test]
	fn should_trace_failure() {
		run_test(
			"60F8d6",
			0xffff,
			r#"
{"pc":0,"op":96,"opName":"PUSH1","gas":"0xffff","gasCost":"0x3","memory":"0x","stack":[],"storage":{},"depth":1}
{"pc":2,"op":214,"opName":"","gas":"0xfffc","gasCost":"0x0","memory":"0x","stack":["0xf8"],"storage":{},"depth":1}
			"#,
		);

		run_test(
			"F8d6",
			0xffff,
			r#"
{"pc":0,"op":248,"opName":"","gas":"0xffff","gasCost":"0x0","memory":"0x","stack":[],"storage":{},"depth":1}
			"#,
		);
	}

	#[test]
	fn should_trace_create_correctly() {
		run_test(
			"32343434345830f138343438323439f0",
			0xffff,
			r#"
{"pc":0,"op":50,"opName":"ORIGIN","gas":"0xffff","gasCost":"0x2","memory":"0x","stack":[],"storage":{},"depth":1}
{"pc":1,"op":52,"opName":"CALLVALUE","gas":"0xfffd","gasCost":"0x2","memory":"0x","stack":["0x0"],"storage":{},"depth":1}
{"pc":2,"op":52,"opName":"CALLVALUE","gas":"0xfffb","gasCost":"0x2","memory":"0x","stack":["0x0","0x0"],"storage":{},"depth":1}
{"pc":3,"op":52,"opName":"CALLVALUE","gas":"0xfff9","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":4,"op":52,"opName":"CALLVALUE","gas":"0xfff7","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":5,"op":88,"opName":"PC","gas":"0xfff5","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":6,"op":48,"opName":"ADDRESS","gas":"0xfff3","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0","0x5"],"storage":{},"depth":1}
{"pc":7,"op":241,"opName":"CALL","gas":"0xfff1","gasCost":"0x61d0","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0","0x5","0x0"],"storage":{},"depth":1}
{"pc":8,"op":56,"opName":"CODESIZE","gas":"0x9e21","gasCost":"0x2","memory":"0x","stack":["0x1"],"storage":{},"depth":1}
{"pc":9,"op":52,"opName":"CALLVALUE","gas":"0x9e1f","gasCost":"0x2","memory":"0x","stack":["0x1","0x10"],"storage":{},"depth":1}
{"pc":10,"op":52,"opName":"CALLVALUE","gas":"0x9e1d","gasCost":"0x2","memory":"0x","stack":["0x1","0x10","0x0"],"storage":{},"depth":1}
{"pc":11,"op":56,"opName":"CODESIZE","gas":"0x9e1b","gasCost":"0x2","memory":"0x","stack":["0x1","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":12,"op":50,"opName":"ORIGIN","gas":"0x9e19","gasCost":"0x2","memory":"0x","stack":["0x1","0x10","0x0","0x0","0x10"],"storage":{},"depth":1}
{"pc":13,"op":52,"opName":"CALLVALUE","gas":"0x9e17","gasCost":"0x2","memory":"0x","stack":["0x1","0x10","0x0","0x0","0x10","0x0"],"storage":{},"depth":1}
{"pc":14,"op":57,"opName":"CODECOPY","gas":"0x9e15","gasCost":"0x9","memory":"0x","stack":["0x1","0x10","0x0","0x0","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":15,"op":240,"opName":"CREATE","gas":"0x9e0c","gasCost":"0x9e0c","memory":"0x32343434345830f138343438323439f0","stack":["0x1","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":0,"op":50,"opName":"ORIGIN","gas":"0x210c","gasCost":"0x2","memory":"0x","stack":[],"storage":{},"depth":2}
{"pc":1,"op":52,"opName":"CALLVALUE","gas":"0x210a","gasCost":"0x2","memory":"0x","stack":["0x0"],"storage":{},"depth":2}
{"pc":2,"op":52,"opName":"CALLVALUE","gas":"0x2108","gasCost":"0x2","memory":"0x","stack":["0x0","0x0"],"storage":{},"depth":2}
{"pc":3,"op":52,"opName":"CALLVALUE","gas":"0x2106","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":4,"op":52,"opName":"CALLVALUE","gas":"0x2104","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":5,"op":88,"opName":"PC","gas":"0x2102","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":6,"op":48,"opName":"ADDRESS","gas":"0x2100","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0","0x5"],"storage":{},"depth":2}
{"pc":7,"op":241,"opName":"CALL","gas":"0x20fe","gasCost":"0x0","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0","0x5","0xbd770416a3345f91e4b34576cb804a576fa48eb1"],"storage":{},"depth":2}
			"#,
		)
	}

	fn run_test<T: Into<U256>>(
		code: &str,
		gas: T,
		expected: &str,
	) {
		let mut params = ActionParams::default();
		params.code = Some(Arc::new(code.from_hex().unwrap()));
		params.gas = gas.into();

		let spec = ::ethcore::ethereum::new_foundation(&::std::env::temp_dir());
		let informant = ::display::json::Informant::default();
		let result = run_action(&spec, params, informant);
		let expected = expected.split("\n")
			.map(|x| x.trim())
			.map(|x| x.to_owned())
			.filter(|x| !x.is_empty())
			.collect::<Vec<_>>();
		match result {
			Ok(Success { traces, .. }) => {
				assert_traces_eq(&traces.unwrap(), &expected);
			},
			Err(Failure { traces, .. }) => {
				assert_traces_eq(&traces.unwrap(), &expected);
			},
		}
	}

	fn assert_traces_eq(
		a: &[String],
		b: &[String],
	) {
		let mut ita = a.iter();
		let mut itb = b.iter();

		loop {
			match (ita.next(), itb.next()) {
				(Some(a), Some(b)) => {
					assert_eq!(a, b);
					println!("{}", a);
				},
				(None, None) => return,
				e => {
					panic!("Traces mismatch: {:?}", e);
				}
			}
		}
	}
}
