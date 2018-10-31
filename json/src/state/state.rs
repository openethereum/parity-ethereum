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

//! State test deserialization.

use bytes::Bytes;
use hash::H256;
use state::{Env, AccountState, Transaction, Log};

/// State test deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
	/// Environment.
	pub env: Env,
	/// Output.
	#[serde(rename = "out")]
	pub output: Bytes,
	/// Pre state.
	#[serde(rename = "pre")]
	pub pre_state: AccountState,
	/// Post state.
	#[serde(rename = "post")]
	pub post_state: AccountState,
	/// Post state root.
	pub post_state_root: H256,
	/// Transaction.
	pub transaction: Transaction,
	/// Logs.
	pub logs: Vec<Log>
}

#[cfg(test)]
mod tests {
	use serde_json;
	use state::State;

	#[test]
	fn state_deserialization() {
		let s = r#"{
			"env" : {
				"currentCoinbase" : "2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
				"currentDifficulty" : "0x0100",
				"currentGasLimit" : "0x01c9c380",
				"currentNumber" : "0x00",
				"currentTimestamp" : "0x01",
				"previousHash" : "5e20a0453cecd065ea59c37ac63e079ee08998b6045136a8ce6635c7912ec0b6"
			},
			"logs" : [
			],
			"out" : "0x",
			"post" : {
				"1000000000000000000000000000000000000000" : {
					"balance" : "0x0de0b6b3a763ffff",
					"code" : "0x6040600060406000600173100000000000000000000000000000000000000162055730f1600055",
					"nonce" : "0x00",
					"storage" : {
						"0x00" : "0x01"
					}
				},
				"1000000000000000000000000000000000000001" : {
					"balance" : "0x0de0b6b3a763ffff",
					"code" : "0x604060006040600060027310000000000000000000000000000000000000026203d090f1600155",
					"nonce" : "0x00",
					"storage" : {
						"0x01" : "0x01"
					}
				},
				"1000000000000000000000000000000000000002" : {
					"balance" : "0x02",
					"code" : "0x600160025533600455346007553060e6553260e8553660ec553860ee553a60f055",
					"nonce" : "0x00",
					"storage" : {
						"0x02" : "0x01",
						"0x04" : "0x1000000000000000000000000000000000000001",
						"0x07" : "0x02",
						"0xe6" : "0x1000000000000000000000000000000000000002",
						"0xe8" : "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b",
						"0xec" : "0x40",
						"0xee" : "0x21",
						"0xf0" : "0x01"
					}
				},
				"2adc25665018aa1fe0e6bc666dac8fc2697ff9ba" : {
					"balance" : "0x039455",
					"code" : "0x",
					"nonce" : "0x00",
					"storage" : {
					}
				},
				"a94f5374fce5edbc8e2a8697c15331677e6ebf0b" : {
					"balance" : "0x0de0b6b3a7606bab",
					"code" : "0x",
					"nonce" : "0x01",
					"storage" : {
					}
				}
			},
			"postStateRoot" : "8f8ed2aed2973e159fa5486f47c6ebf15c5058f8e2350286b84b569bc6ce2d25",
			"pre" : {
				"1000000000000000000000000000000000000000" : {
					"balance" : "0x0de0b6b3a7640000",
					"code" : "0x6040600060406000600173100000000000000000000000000000000000000162055730f1600055",
					"nonce" : "0x00",
					"storage" : {
					}
				},
				"1000000000000000000000000000000000000001" : {
					"balance" : "0x0de0b6b3a7640000",
					"code" : "0x604060006040600060027310000000000000000000000000000000000000026203d090f1600155",
					"nonce" : "0x00",
					"storage" : {
					}
				},
				"1000000000000000000000000000000000000002" : {
					"balance" : "0x00",
					"code" : "0x600160025533600455346007553060e6553260e8553660ec553860ee553a60f055",
					"nonce" : "0x00",
					"storage" : {
					}
				},
				"a94f5374fce5edbc8e2a8697c15331677e6ebf0b" : {
					"balance" : "0x0de0b6b3a7640000",
					"code" : "0x",
					"nonce" : "0x00",
					"storage" : {
					}
				}
			},
			"transaction" : {
				"data" : "",
				"gasLimit" : "0x2dc6c0",
				"gasPrice" : "0x01",
				"nonce" : "0x00",
				"secretKey" : "45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8",
				"to" : "1000000000000000000000000000000000000000",
				"value" : "0x00"
			}
		}"#;
		let _deserialized: State = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
