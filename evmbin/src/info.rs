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
	params: ActionParams,
	mut informant: T,
) -> RunResult<T::Output> {
	informant.set_gas(params.gas);
	run(spec, params.gas, None, |mut client| {
		let result = client
			.call(params, &mut trace::NoopTracer, &mut informant)
			.map(|r| (0.into(), r.gas_left, r.return_data.to_vec()));
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
		let result = client.transact(env_info, transaction, trace::NoopTracer, informant);
		match result {
			TransactResult::Ok { state_root, .. } if state_root != post_root => {
				(Err(EvmTestError::PostCondition(format!(
					"State root mismatch (got: {}, expected: {})",
					state_root,
					post_root,
				))), None)
			},
			TransactResult::Ok { state_root, gas_left, output, vm_trace, .. } => {
				(Ok((state_root, gas_left, output)), vm_trace)
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
	F: FnOnce(EvmTestClient) -> (Result<(H256, U256, Vec<u8>), EvmTestError>, Option<X>),
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
		(Ok((state_root, gas_left, output)), traces) => Ok(Success {
			state_root,
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
pub mod tests {
	use std::sync::Arc;
	use rustc_hex::FromHex;
	use super::*;

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

		let spec = ::ethcore::ethereum::new_foundation(&::std::env::temp_dir());
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
}
