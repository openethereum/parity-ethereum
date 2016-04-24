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
	parent: Option<U256>,
	children: Vec<U256>,
	depth: U256,
	action: Action,
	result: Res,
	trace_number: U256,
	transaction_number: U256,
	transaction_hash: H256,
	block_number: U256,
	block_hash: H256,
}

impl From<LocalizedTrace> for Trace {
	fn from(t: LocalizedTrace) -> Self {
		Trace {
			parent: t.parent.map(From::from),
			children: t.children.into_iter().map(From::from).collect(),
			depth: From::from(t.depth),
			action: From::from(t.action),
			result: From::from(t.result),
			trace_number: From::from(t.trace_number),
			transaction_number: From::from(t.transaction_number),
			transaction_hash: t.transaction_hash,
			block_number: From::from(t.block_number),
			block_hash: t.block_hash,
		}
	}
}

#[cfg(test)]
mod tests {

}
