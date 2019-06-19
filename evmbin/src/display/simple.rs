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

//! Simple VM output.

use ethcore::trace;
use bytes::ToPretty;

use config::{logger};
use display;
use info as vm;

/// Simple formatting informant.
#[derive(Default)]
pub struct Informant;

#[derive(Serialize, Debug)]
pub struct MessageInitial<'a> {
	action: &'a str,
	test: &'a str,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageSuccess<'a> {
	output: &'a str,
	gas_used: &'a str,
	time: &'a u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageFailure<'a> {
	error: &'a str,
	time: &'a u64,
}

impl vm::Informant for Informant {

	type Sink = ();

	fn before_test(&mut self, name: &str, action: &str) {
		let message_init =
			MessageInitial {
				action,
				test: &name,
			}
		;

		let serialized_message_init = serde_json::to_string(&message_init).expect("serialization cannot fail; qed");
		info!("Message initial: {}", serialized_message_init);
	}

	fn clone_sink(&self) -> Self::Sink { () }

	fn finish(result: vm::RunResult<Self::Output>, _sink: &mut Self::Sink) {
		match result {
			Ok(success) => {
				let message_success =
					MessageSuccess {
						output: &format!("0x{}", success.output.to_hex()),
						gas_used: &format!("{:#x}", success.gas_used),
						time: &display::as_micros(&success.time),
					}
				;

				let serialized_message_success = serde_json::to_string(&message_success).expect("serialization cannot fail; qed");
				info!("Message success: {}", serialized_message_success);
			},
			Err(failure) => {
				let message_failure =
					MessageFailure {
						error: &failure.error.to_string(),
						time: &display::as_micros(&failure.time),
					}
				;

				let serialized_message_failure = serde_json::to_string(&message_failure).expect("serialization cannot fail; qed");
				error!("Message failure: {}", serialized_message_failure);
			},
		}
	}
}

impl trace::VMTracer for Informant {
	type Output = ();

	fn prepare_subtrace(&mut self, _code: &[u8]) { Default::default() }
	fn done_subtrace(&mut self) {}
	fn drain(self) -> Option<()> { None }
}
