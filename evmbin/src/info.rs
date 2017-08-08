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
use util::U256;
use ethcore::{trace, spec};
use ethcore::client::{EvmTestClient, EvmTestError};
use vm::ActionParams;

/// VM execution informant
pub trait Informant: trace::VMTracer {
	/// Set initial gas.
	fn set_gas(&mut self, _gas: U256) {}
	/// Display final result.
	fn finish(&mut self, result: Result<Success, Failure>);
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

/// Execute VM with given `ActionParams`
pub fn run<T: trace::VMTracer>(vm_tracer: &mut T, spec: spec::Spec, params: ActionParams) -> Result<Success, Failure> {
	let mut test_client = EvmTestClient::new(spec).map_err(|error| Failure {
		gas_used: 0.into(),
		error,
		time: Duration::from_secs(0)
	})?;

	let initial_gas = params.gas;
	let start = Instant::now();
	let result = test_client.call(params, vm_tracer);
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
