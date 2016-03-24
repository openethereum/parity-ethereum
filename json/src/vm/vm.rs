// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Vm execution env.

use bytes::Bytes;
use uint::Uint;
use blockchain::State;
use vm::{Transaction, Log, Call, Env};

/// Reporesents vm execution environment before and after exeuction of transaction.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Vm {
	/// Contract calls made internaly by executed transaction.
	#[serde(rename="callcreates")]
	pub calls: Option<Vec<Call>>,
	/// Env info.
	pub env: Env,
	/// Executed transaction
	#[serde(rename="exec")]
	pub transaction: Transaction,
	/// Gas left after transaction execution.
	#[serde(rename="gas")]
	pub gas_left: Option<Uint>,
	/// Logs created during execution of transaction.
	pub logs: Option<Vec<Log>>,
	/// Transaction output.
	#[serde(rename="out")]
	pub output: Option<Bytes>,
	/// Post execution vm state.
	#[serde(rename="post")]
	pub post_state: Option<State>,
	/// Pre execution vm state.
	#[serde(rename="pre")]
	pub pre_state: State,
}

impl Vm {
	/// Returns true if transaction execution run out of gas.
	pub fn out_of_gas(&self) -> bool {
		self.calls.is_none()
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use vm::Vm;

	#[test]
	fn vm_deserialization() {
		let s = r#"{
			"callcreates" : [
			],
			"env" : {
				"currentCoinbase" : "2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
				"currentDifficulty" : "0x0100",
				"currentGasLimit" : "0x0f4240",
				"currentNumber" : "0x00",
				"currentTimestamp" : "0x01"
			},
			"exec" : {
				"address" : "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6",
				"caller" : "cd1722f2947def4cf144679da39c4c32bdc35681",
				"code" : "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055",
				"data" : "0x",
				"gas" : "0x0186a0",
				"gasPrice" : "0x5af3107a4000",
				"origin" : "cd1722f2947def4cf144679da39c4c32bdc35681",
				"value" : "0x0de0b6b3a7640000"
			},
			"gas" : "0x013874",
			"logs" : [
			],
			"out" : "0x",
			"post" : {
				"0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6" : {
					"balance" : "0x0de0b6b3a7640000",
					"code" : "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055",
					"nonce" : "0x00",
					"storage" : {
						"0x00" : "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"
					}
				}
			},
			"pre" : {
				"0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6" : {
					"balance" : "0x0de0b6b3a7640000",
					"code" : "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055",
					"nonce" : "0x00",
					"storage" : {
					}
				}
			}
		}"#;
		let _deserialized: Vm = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
