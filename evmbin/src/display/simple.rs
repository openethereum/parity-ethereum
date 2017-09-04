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

//! Simple VM output.

use ethcore::trace;
use util::ToPretty;

use display;
use info as vm;

/// Simple formatting informant.
#[derive(Default)]
pub struct Informant;

impl vm::Informant for Informant {
	fn before_test(&self, name: &str, action: &str) {
		println!("Test: {} ({})", name, action);
	}

	fn finish(result: Result<vm::Success, vm::Failure>) {
		match result {
			Ok(success) => {
				println!("Output: 0x{}", success.output.to_hex());
				println!("Gas used: {:x}", success.gas_used);
				println!("Time: {}", display::format_time(&success.time));
			},
			Err(failure) => {
				println!("Error: {}", failure.error);
				println!("Time: {}", display::format_time(&failure.time));
			},
		}
	}
}

impl trace::VMTracer for Informant {
	fn prepare_subtrace(&self, _code: &[u8]) -> Self where Self: Sized { Default::default() }
	fn done_subtrace(&mut self, _sub: Self) {}
	fn drain(self) -> Option<trace::VMTrace> { None }
}
