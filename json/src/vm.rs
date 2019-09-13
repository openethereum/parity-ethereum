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

//! Vm json deserialization

use crate::{
	bytes::Bytes,
	hash::{Address, H256},
	maybe::MaybeEmpty,
	spec::State,
	uint::Uint,
};
use serde::Deserialize;

/// Represents vm execution environment before and after execution of transaction.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Vm {
	/// Contract calls made internaly by executed transaction.
	#[serde(rename = "callcreates")]
	pub calls: Option<Vec<Call>>,
	/// Env info.
	pub env: Env,
	/// Executed transaction
	#[serde(rename = "exec")]
	pub transaction: Transaction,
	/// Gas left after transaction execution.
	#[serde(rename = "gas")]
	pub gas_left: Option<Uint>,
	/// Hash of logs created during execution of transaction.
	pub logs: Option<H256>,
	/// Transaction output.
	#[serde(rename = "out")]
	pub output: Option<Bytes>,
	/// Post execution vm state.
	#[serde(rename = "post")]
	pub post_state: Option<State>,
	/// Pre execution vm state.
	#[serde(rename = "pre")]
	pub pre_state: State,
}

impl Vm {
	/// Returns true if transaction execution run out of gas.
	pub fn out_of_gas(&self) -> bool {
		self.calls.is_none()
	}
}

/// Call deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Call {
	/// Call data.
	pub data: Bytes,
	/// Call destination.
	pub destination: MaybeEmpty<Address>,
	/// Gas limit.
	pub gas_limit: Uint,
	/// Call value.
	pub value: Uint,
}

/// Executed transaction.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
	/// Contract address.
	pub address: Address,
	/// Transaction sender.
	#[serde(rename = "caller")]
	pub sender: Address,
	/// Contract code.
	pub code: Bytes,
	/// Input data.
	pub data: Bytes,
	/// Gas.
	pub gas: Uint,
	/// Gas price.
	pub gas_price: Uint,
	/// Transaction origin.
	pub origin: Address,
	/// Sent value.
	pub value: Uint,
	/// Contract code version.
	#[serde(default)]
	pub code_version: Uint,
}

/// Environment.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Env {
	/// Address.
	#[serde(rename = "currentCoinbase")]
	pub author: Address,
	/// Difficulty
	#[serde(rename = "currentDifficulty")]
	pub difficulty: Uint,
	/// Gas limit.
	#[serde(rename = "currentGasLimit")]
	pub gas_limit: Uint,
	/// Number.
	#[serde(rename = "currentNumber")]
	pub number: Uint,
	/// Timestamp.
	#[serde(rename = "currentTimestamp")]
	pub timestamp: Uint,
}

#[cfg(test)]
mod tests {
	use std::{
		collections::BTreeMap,
		str::FromStr
	};
	use super::{Address, Bytes, Call, Env, H256, MaybeEmpty, State, Transaction, Uint, Vm};

	use crate::spec::Account;
	use ethereum_types::{U256, H160 as Hash160, H256 as Hash256};
	use macros::map;
	use rustc_hex::FromHex;

	const TEST_CODE: &str = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055";

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
			"logs" : "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
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
		let vm: Vm = serde_json::from_str(s).unwrap();
		assert_eq!(vm.calls, Some(Vec::new()));
		assert_eq!(vm.env, Env {
			author: Address(Hash160::from_str("2adc25665018aa1fe0e6bc666dac8fc2697ff9ba").unwrap()),
			difficulty: Uint(0x0100.into()),
			gas_limit: Uint(0x0f4240.into()),
			number: Uint(0.into()),
			timestamp: Uint(1.into())
		});
		assert_eq!(vm.transaction, Transaction {
			address: Address(Hash160::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap()),
			sender: Address(Hash160::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap()),
			code: Bytes::new(TEST_CODE.from_hex().unwrap()),
			code_version: Uint(0.into()),
			data: Bytes::new(Vec::new()),
			gas: Uint(0x0186a0.into()),
			gas_price: Uint(0x5af3107a4000_u64.into()),
			origin: Address(Hash160::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap()),
			value: Uint(0x0de0b6b3a7640000_u64.into())
		});
		assert_eq!(vm.gas_left, Some(Uint(0x013874.into())));
		assert_eq!(
			vm.logs,
			Some(H256(Hash256::from_str("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").unwrap()))
		);
		assert_eq!(vm.output, Some(Bytes::new(Vec::new())));
		assert_eq!(vm.pre_state, State(map![
			Address(Hash160::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap()) => Account {
				builtin: None,
				balance: Some(Uint(0x0de0b6b3a7640000_u64.into())),
				code: Some(Bytes::new(TEST_CODE.from_hex().unwrap())),
				constructor: None,
				nonce: Some(Uint(0.into())),
				storage: Some(map![]),
				version: None,
			}])
		);
		assert_eq!(vm.post_state, Some(
				State(map![
					Address(Hash160::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap()) => Account {
						builtin: None,
						balance: Some(Uint(0x0de0b6b3a7640000_u64.into())),
						code: Some(Bytes::new(TEST_CODE.from_hex().unwrap())),
						constructor: None,
						nonce: Some(Uint(0.into())),
						storage: Some(map![
							Uint(0.into()) => Uint(U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap())
						]),
						version: None,
				}])
			)
		);
	}

	#[test]
	fn call_deserialization_empty_dest() {
		let s = r#"{
			"data" : "0x1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff",
			"destination" : "",
			"gasLimit" : "0x1748766aa5",
			"value" : "0x00"
		}"#;
		let call: Call = serde_json::from_str(s).unwrap();

		assert_eq!(&call.data[..],
			&[0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44, 0x55, 0x55, 0x66, 0x66, 0x77, 0x77,
			  0x88, 0x88, 0x99, 0x99, 0x00, 0x00, 0xaa, 0xaa, 0xbb, 0xbb, 0xcc, 0xcc, 0xdd, 0xdd,
			  0xee, 0xee, 0xff, 0xff]);

		assert_eq!(call.destination, MaybeEmpty::None);
		assert_eq!(call.gas_limit, Uint(U256::from(0x1748766aa5u64)));
		assert_eq!(call.value, Uint(U256::from(0)));
	}

	#[test]
	fn call_deserialization_full_dest() {
		let s = r#"{
			"data" : "0x1234",
			"destination" : "5a39ed1020c04d4d84539975b893a4e7c53eab6c",
			"gasLimit" : "0x1748766aa5",
			"value" : "0x00"
		}"#;

		let call: Call = serde_json::from_str(s).unwrap();

		assert_eq!(&call.data[..], &[0x12, 0x34]);
		assert_eq!(call.destination, MaybeEmpty::Some(Address(Hash160::from_str("5a39ed1020c04d4d84539975b893a4e7c53eab6c").unwrap())));
		assert_eq!(call.gas_limit, Uint(U256::from(0x1748766aa5u64)));
		assert_eq!(call.value, Uint(U256::from(0)));
	}
}
