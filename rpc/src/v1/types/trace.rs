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

use util::{Address, U256, H256};
use ethcore::trace::trace;
use ethcore::trace::LocalizedTrace;
use v1::types::Bytes;

#[derive(Debug, Serialize)]
pub struct Create {
	from: Address,
	value: U256,
	gas: U256,
	init: Bytes,
}

impl From<trace::Create> for Create {
	fn from(c: trace::Create) -> Self {
		Create {
			from: c.from,
			value: c.value,
			gas: c.gas,
			init: Bytes::new(c.init),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct Call {
	from: Address,
	to: Address,
	value: U256,
	gas: U256,
	input: Bytes,
}

impl From<trace::Call> for Call {
	fn from(c: trace::Call) -> Self {
		Call {
			from: c.from,
			to: c.to,
			value: c.value,
			gas: c.gas,
			input: Bytes::new(c.input),
		}
	}
}

#[derive(Debug, Serialize)]
pub enum Action {
	#[serde(rename="call")]
	Call(Call),
	#[serde(rename="create")]
	Create(Create),
}

impl From<trace::Action> for Action {
	fn from(c: trace::Action) -> Self {
		match c {
			trace::Action::Call(call) => Action::Call(Call::from(call)),
			trace::Action::Create(create) => Action::Create(Create::from(create)),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct CallResult {
	#[serde(rename="gasUsed")]
	gas_used: U256,
	output: Bytes,
}

impl From<trace::CallResult> for CallResult {
	fn from(c: trace::CallResult) -> Self {
		CallResult {
			gas_used: c.gas_used,
			output: Bytes::new(c.output),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct CreateResult {
	#[serde(rename="gasUsed")]
	gas_used: U256,
	code: Bytes,
	address: Address,
}

impl From<trace::CreateResult> for CreateResult {
	fn from(c: trace::CreateResult) -> Self {
		CreateResult {
			gas_used: c.gas_used,
			code: Bytes::new(c.code),
			address: c.address,
		}
	}
}

#[derive(Debug, Serialize)]
pub enum Res {
	#[serde(rename="call")]
	Call(CallResult),
	#[serde(rename="create")]
	Create(CreateResult),
	#[serde(rename="failedCall")]
	FailedCall,
	#[serde(rename="failedCreate")]
	FailedCreate,
}

impl From<trace::Res> for Res {
	fn from(t: trace::Res) -> Self {
		match t {
			trace::Res::Call(call) => Res::Call(CallResult::from(call)),
			trace::Res::Create(create) => Res::Create(CreateResult::from(create)),
			trace::Res::FailedCall => Res::FailedCall,
			trace::Res::FailedCreate => Res::FailedCreate,
		}
	}
}

#[derive(Debug, Serialize)]
pub struct Trace {
	action: Action,
	result: Res,
	#[serde(rename="traceAddress")]
	trace_address: Vec<U256>,
	subtraces: U256,
	#[serde(rename="transactionPosition")]
	transaction_position: U256,
	#[serde(rename="transactionHash")]
	transaction_hash: H256,
	#[serde(rename="blockNumber")]
	block_number: U256,
	#[serde(rename="blockHash")]
	block_hash: H256,
}

impl From<LocalizedTrace> for Trace {
	fn from(t: LocalizedTrace) -> Self {
		Trace {
			action: From::from(t.action),
			result: From::from(t.result),
			trace_address: t.trace_address.into_iter().map(From::from).collect(),
			subtraces: From::from(t.subtraces),
			transaction_position: From::from(t.transaction_number),
			transaction_hash: t.transaction_hash,
			block_number: From::from(t.block_number),
			block_hash: t.block_hash,
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use util::{U256, H256, Address};
	use v1::types::Bytes;
	use super::*;

	#[test]
	fn test_trace_serialize() {
		let t = Trace {
			action: Action::Call(Call {
				from: Address::from(4),
				to: Address::from(5),
				value: U256::from(6),
				gas: U256::from(7),
				input: Bytes::new(vec![0x12, 0x34]),
			}),
			result: Res::Call(CallResult {
				gas_used: U256::from(8),
				output: Bytes::new(vec![0x56, 0x78]),
			}),
			trace_address: vec![U256::from(10)],
			subtraces: U256::from(1),
			transaction_position: U256::from(11),
			transaction_hash: H256::from(12),
			block_number: U256::from(13),
			block_hash: H256::from(14),
		};
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"action":{"call":{"from":"0x0000000000000000000000000000000000000004","to":"0x0000000000000000000000000000000000000005","value":"0x06","gas":"0x07","input":"0x1234"}},"result":{"call":{"gasUsed":"0x08","output":"0x5678"}},"traceAddress":["0x0a"],"subtraces":"0x01","transactionPosition":"0x0b","transactionHash":"0x000000000000000000000000000000000000000000000000000000000000000c","blockNumber":"0x0d","blockHash":"0x000000000000000000000000000000000000000000000000000000000000000e"}"#);
	}

	#[test]
	fn test_action_serialize() {
		let actions = vec![Action::Call(Call {
			from: Address::from(1),
			to: Address::from(2),
			value: U256::from(3),
			gas: U256::from(4),
			input: Bytes::new(vec![0x12, 0x34]),
		}), Action::Create(Create {
			from: Address::from(5),
			value: U256::from(6),
			gas: U256::from(7),
			init: Bytes::new(vec![0x56, 0x78]),
		})];

		let serialized = serde_json::to_string(&actions).unwrap();
		assert_eq!(serialized, r#"[{"call":{"from":"0x0000000000000000000000000000000000000001","to":"0x0000000000000000000000000000000000000002","value":"0x03","gas":"0x04","input":"0x1234"}},{"create":{"from":"0x0000000000000000000000000000000000000005","value":"0x06","gas":"0x07","init":"0x5678"}}]"#);
	}

	#[test]
	fn test_result_serialize() {
		let results = vec![
			Res::Call(CallResult {
				gas_used: U256::from(1),
				output: Bytes::new(vec![0x12, 0x34]),
			}),
			Res::Create(CreateResult {
				gas_used: U256::from(2),
				code: Bytes::new(vec![0x45, 0x56]),
				address: Address::from(3),
			}),
			Res::FailedCall,
			Res::FailedCreate,
		];

		let serialized = serde_json::to_string(&results).unwrap();
		assert_eq!(serialized, r#"[{"call":{"gasUsed":"0x01","output":"0x1234"}},{"create":{"gasUsed":"0x02","code":"0x4556","address":"0x0000000000000000000000000000000000000003"}},{"failedCall":[]},{"failedCreate":[]}]"#);
	}
}
