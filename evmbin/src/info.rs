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
use util::{U256, H256};
use ethcore::{trace, spec, transaction, pod_state};
use ethcore::client::{self, EvmTestClient, EvmTestError};

/// VM execution informant
pub trait Informant: trace::VMTracer {
	/// Set initial gas.
	fn set_gas(&mut self, _gas: U256) {}
	/// Display final result.
	fn finish(result: Result<Success, Failure>);
}

/// Execution finished correctly
pub struct Success {
	/// Used gas
	pub gas_used: U256,
	/// Output as bytes
	pub output: Vec<u8>,
	/// Time Taken
	pub time: Duration,
}

/// Execution failed
pub struct Failure {
	/// Used gas
	pub gas_used: U256,
	/// Internal error
	pub error: EvmTestError,
	/// Duration
	pub time: Duration,
}

/// Execute given Transaction and verify resulting state root.
pub fn run_transaction<T: Informant>(
	spec: spec::Spec,
	pre_state: &pod_state::PodState,
	post_root: H256,
	env_info: &client::EnvInfo,
	transaction: transaction::SignedTransaction,
	mut informant: T,
) {
	informant.set_gas(env_info.gas_limit);
	let result = run(spec, env_info.gas_limit, Some(pre_state), |mut client| {
		let (root, gas, out) = client.transact(env_info, transaction, informant)?;
		if root != post_root {
			return Err(EvmTestError::PostCondition(format!(
				"State root mismatch (got: {}, expected: {})",
				root,
				post_root,
			)));
		}

		Ok((gas, out))
	});
	T::finish(result)
}

/// Execute VM with given `ActionParams`
pub fn run<F>(spec: spec::Spec, initial_gas: U256, pre_state: Option<&pod_state::PodState>,  run: F) -> Result<Success, Failure> where
	F: FnOnce(EvmTestClient) -> Result<(U256, Vec<u8>), EvmTestError>
{
	let test_client = EvmTestClient::with_pre_state(spec, pre_state).map_err(|error| Failure {
		gas_used: 0.into(),
		error,
		time: Duration::from_secs(0)
	})?;

	let start = Instant::now();
	let result = run(test_client);
	let duration = start.elapsed();

	match result {
		Ok((gas_left, output)) => Ok(Success {
			gas_used: initial_gas - gas_left,
			output: output,
			time: duration,
		}),
		Err(e) => Err(Failure {
			gas_used: initial_gas,
			error: e,
			time: duration,
		}),
	}
}
