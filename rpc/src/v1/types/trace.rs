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

use std::collections::BTreeMap;
use serde::{Serialize, Serializer};
use ethcore::trace::trace;
use ethcore::trace::{FlatTrace, LocalizedTrace as EthLocalizedTrace};
use ethcore::trace as et;
use ethcore::state_diff;
use ethcore::account_diff;
use ethcore::executed;
use ethcore::client::Executed;
use util::Uint;
use v1::types::{Bytes, H160, H256, U256};

#[derive(Debug, Serialize)]
/// A diff of some chunk of memory.
pub struct MemoryDiff {
	/// Offset into memory the change begins.
	pub off: usize,
	/// The changed data.
	pub data: Vec<u8>,
}

impl From<et::MemoryDiff> for MemoryDiff {
	fn from(c: et::MemoryDiff) -> Self {
		MemoryDiff {
			off: c.offset,
			data: c.data,
		}
	}
}

#[derive(Debug, Serialize)]
/// A diff of some storage value.
pub struct StorageDiff {
	/// Which key in storage is changed.
	pub key: U256,
	/// What the value has been changed to.
	pub val: U256,
}

impl From<et::StorageDiff> for StorageDiff {
	fn from(c: et::StorageDiff) -> Self {
		StorageDiff {
			key: c.location.into(),
			val: c.value.into(),
		}
	}
}

#[derive(Debug, Serialize)]
/// A record of an executed VM operation.
pub struct VMExecutedOperation {
	/// The total gas used.
	#[serde(rename="used")]
	pub used: u64,
	/// The stack item placed, if any.
	pub push: Vec<U256>,
	/// If altered, the memory delta.
	#[serde(rename="mem")]
	pub mem: Option<MemoryDiff>,
	/// The altered storage value, if any.
	#[serde(rename="store")]
	pub store: Option<StorageDiff>,
}

impl From<et::VMExecutedOperation> for VMExecutedOperation {
	fn from(c: et::VMExecutedOperation) -> Self {
		VMExecutedOperation {
			used: c.gas_used.low_u64(),
			push: c.stack_push.into_iter().map(Into::into).collect(),
			mem: c.mem_diff.map(Into::into),
			store: c.store_diff.map(Into::into),
		}
	}
}

#[derive(Debug, Serialize)]
/// A record of the execution of a single VM operation.
pub struct VMOperation {
	/// The program counter.
	pub pc: usize,
	/// The gas cost for this instruction.
	pub cost: u64,
	/// Information concerning the execution of the operation.
	pub ex: Option<VMExecutedOperation>,
	/// Subordinate trace of the CALL/CREATE if applicable.
	pub sub: Option<VMTrace>,
}

impl From<(et::VMOperation, Option<et::VMTrace>)> for VMOperation {
	fn from(c: (et::VMOperation, Option<et::VMTrace>)) -> Self {
		VMOperation {
			pc: c.0.pc,
			cost: c.0.gas_cost.low_u64(),
			ex: c.0.executed.map(Into::into),
			sub: c.1.map(Into::into),
		}
	}
}

#[derive(Debug, Serialize)]
/// A record of a full VM trace for a CALL/CREATE.
pub struct VMTrace {
	/// The code to be executed.
	pub code: Vec<u8>,
	/// The operations executed.
	pub ops: Vec<VMOperation>,
}

impl From<et::VMTrace> for VMTrace {
	fn from(c: et::VMTrace) -> Self {
		let mut subs = c.subs.into_iter();
		let mut next_sub = subs.next();
		VMTrace {
			code: c.code,
			ops: c.operations
				.into_iter()
				.enumerate()
				.map(|(i, op)| (op, {
					let have_sub = next_sub.is_some() && next_sub.as_ref().unwrap().parent_step == i;
					if have_sub {
						let r = next_sub.clone();
						next_sub = subs.next();
						r
					} else { None }
				}).into())
				.collect(),
		}
	}
}

#[derive(Debug, Serialize)]
/// Aux type for Diff::Changed.
pub struct ChangedType<T> where T: Serialize {
	from: T,
	to: T,
}

#[derive(Debug, Serialize)]
/// Serde-friendly `Diff` shadow.
pub enum Diff<T> where T: Serialize {
	#[serde(rename="=")]
	Same,
	#[serde(rename="+")]
	Born(T),
	#[serde(rename="-")]
	Died(T),
	#[serde(rename="*")]
	Changed(ChangedType<T>),
}

impl<T, U> From<account_diff::Diff<T>> for Diff<U> where T: Eq + ::ethcore_ipc::BinaryConvertable, U: Serialize + From<T> {
	fn from(c: account_diff::Diff<T>) -> Self {
		match c {
			account_diff::Diff::Same => Diff::Same,
			account_diff::Diff::Born(t) => Diff::Born(t.into()),
			account_diff::Diff::Died(t) => Diff::Died(t.into()),
			account_diff::Diff::Changed(t, u) => Diff::Changed(ChangedType{from: t.into(), to: u.into()}),
		}
	}
}

#[derive(Debug, Serialize)]
/// Serde-friendly `AccountDiff` shadow.
pub struct AccountDiff {
	pub balance: Diff<U256>,
	pub nonce: Diff<U256>,
	pub code: Diff<Bytes>,
	pub storage: BTreeMap<H256, Diff<H256>>,
}

impl From<account_diff::AccountDiff> for AccountDiff {
	fn from(c: account_diff::AccountDiff) -> Self {
		AccountDiff {
			balance: c.balance.into(),
			nonce: c.nonce.into(),
			code: c.code.into(),
			storage: c.storage.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
		}
	}
}

#[derive(Debug)]
/// Serde-friendly `StateDiff` shadow.
pub struct StateDiff(BTreeMap<H160, AccountDiff>);

impl Serialize for StateDiff {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		Serialize::serialize(&self.0, serializer)
	}
}

impl From<state_diff::StateDiff> for StateDiff {
	fn from(c: state_diff::StateDiff) -> Self {
		StateDiff(c.raw.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
	}
}

/// Create response
#[derive(Debug, Serialize)]
pub struct Create {
	/// Sender
	from: H160,
	/// Value
	value: U256,
	/// Gas
	gas: U256,
	/// Initialization code
	init: Bytes,
}

impl From<trace::Create> for Create {
	fn from(c: trace::Create) -> Self {
		Create {
			from: c.from.into(),
			value: c.value.into(),
			gas: c.gas.into(),
			init: Bytes::new(c.init),
		}
	}
}

/// Call type.
#[derive(Debug, Serialize)]
pub enum CallType {
	/// None
	#[serde(rename="none")]
	None,
	/// Call
	#[serde(rename="call")]
	Call,
	/// Call code
	#[serde(rename="callcode")]
	CallCode,
	/// Delegate call
	#[serde(rename="delegatecall")]
	DelegateCall,
}

impl From<executed::CallType> for CallType {
	fn from(c: executed::CallType) -> Self {
		match c {
			executed::CallType::None => CallType::None,
			executed::CallType::Call => CallType::Call,
			executed::CallType::CallCode => CallType::CallCode,
			executed::CallType::DelegateCall => CallType::DelegateCall,
		}
	}
}

/// Call response
#[derive(Debug, Serialize)]
pub struct Call {
	/// Sender
	from: H160,
	/// Recipient
	to: H160,
	/// Transfered Value
	value: U256,
	/// Gas
	gas: U256,
	/// Input data
	input: Bytes,
	/// The type of the call.
	#[serde(rename="callType")]
	call_type: CallType,
}

impl From<trace::Call> for Call {
	fn from(c: trace::Call) -> Self {
		Call {
			from: c.from.into(),
			to: c.to.into(),
			value: c.value.into(),
			gas: c.gas.into(),
			input: c.input.into(),
			call_type: c.call_type.into(),
		}
	}
}

/// Suicide
#[derive(Debug, Serialize)]
pub struct Suicide {
	/// Address.
	pub address: H160,
	/// Refund address.
	#[serde(rename="refundAddress")]
	pub refund_address: H160,
	/// Balance.
	pub balance: U256,
}

impl From<trace::Suicide> for Suicide {
	fn from(s: trace::Suicide) -> Self {
		Suicide {
			address: s.address.into(),
			refund_address: s.refund_address.into(),
			balance: s.balance.into(),
		}
	}
}

/// Action
#[derive(Debug, Serialize)]
pub enum Action {
	/// Call
	#[serde(rename="call")]
	Call(Call),
	/// Create
	#[serde(rename="create")]
	Create(Create),
	/// Suicide
	#[serde(rename="suicide")]
	Suicide(Suicide),
}

impl From<trace::Action> for Action {
	fn from(c: trace::Action) -> Self {
		match c {
			trace::Action::Call(call) => Action::Call(call.into()),
			trace::Action::Create(create) => Action::Create(create.into()),
			trace::Action::Suicide(suicide) => Action::Suicide(suicide.into()),
		}
	}
}

/// Call Result
#[derive(Debug, Serialize)]
pub struct CallResult {
	/// Gas used
	#[serde(rename="gasUsed")]
	gas_used: U256,
	/// Output bytes
	output: Bytes,
}

impl From<trace::CallResult> for CallResult {
	fn from(c: trace::CallResult) -> Self {
		CallResult {
			gas_used: c.gas_used.into(),
			output: c.output.into(),
		}
	}
}

/// Craete Result
#[derive(Debug, Serialize)]
pub struct CreateResult {
	/// Gas used
	#[serde(rename="gasUsed")]
	gas_used: U256,
	/// Code
	code: Bytes,
	/// Assigned address
	address: H160,
}

impl From<trace::CreateResult> for CreateResult {
	fn from(c: trace::CreateResult) -> Self {
		CreateResult {
			gas_used: c.gas_used.into(),
			code: c.code.into(),
			address: c.address.into(),
		}
	}
}

/// Response
#[derive(Debug, Serialize)]
pub enum Res {
	/// Call
	#[serde(rename="call")]
	Call(CallResult),
	/// Create
	#[serde(rename="create")]
	Create(CreateResult),
	/// Call failure
	#[serde(rename="failedCall")]
	FailedCall,
	/// Creation failure
	#[serde(rename="failedCreate")]
	FailedCreate,
	/// None
	#[serde(rename="none")]
	None,
}

impl From<trace::Res> for Res {
	fn from(t: trace::Res) -> Self {
		match t {
			trace::Res::Call(call) => Res::Call(CallResult::from(call)),
			trace::Res::Create(create) => Res::Create(CreateResult::from(create)),
			trace::Res::FailedCall => Res::FailedCall,
			trace::Res::FailedCreate => Res::FailedCreate,
			trace::Res::None => Res::None,
		}
	}
}

/// Trace
#[derive(Debug, Serialize)]
pub struct LocalizedTrace {
	/// Action
	action: Action,
	/// Result
	result: Res,
	/// Trace address
	#[serde(rename="traceAddress")]
	trace_address: Vec<U256>,
	/// Subtraces
	subtraces: U256,
	/// Transaction position
	#[serde(rename="transactionPosition")]
	transaction_position: U256,
	/// Transaction hash
	#[serde(rename="transactionHash")]
	transaction_hash: H256,
	/// Block Number
	#[serde(rename="blockNumber")]
	block_number: U256,
	/// Block Hash
	#[serde(rename="blockHash")]
	block_hash: H256,
}

impl From<EthLocalizedTrace> for LocalizedTrace {
	fn from(t: EthLocalizedTrace) -> Self {
		LocalizedTrace {
			action: t.action.into(),
			result: t.result.into(),
			trace_address: t.trace_address.into_iter().map(Into::into).collect(),
			subtraces: t.subtraces.into(),
			transaction_position: t.transaction_number.into(),
			transaction_hash: t.transaction_hash.into(),
			block_number: t.block_number.into(),
			block_hash: t.block_hash.into(),
		}
	}
}

/// Trace
#[derive(Debug, Serialize)]
pub struct Trace {
	/// Trace address
	#[serde(rename="traceAddress")]
	trace_address: Vec<U256>,
	/// Subtraces
	subtraces: U256,
	/// Action
	action: Action,
	/// Result
	result: Res,
}

impl From<FlatTrace> for Trace {
	fn from(t: FlatTrace) -> Self {
		Trace {
			trace_address: t.trace_address.into_iter().map(Into::into).collect(),
			subtraces: t.subtraces.into(),
			action: t.action.into(),
			result: t.result.into(),
		}
	}
}

#[derive(Debug, Serialize)]
/// A diff of some chunk of memory.
pub struct TraceResults {
	/// The output of the call/create
	pub output: Vec<u8>,
	/// The transaction trace.
	pub trace: Vec<Trace>,
	/// The transaction trace.
	#[serde(rename="vmTrace")]
	pub vm_trace: Option<VMTrace>,
	/// The transaction trace.
	#[serde(rename="stateDiff")]
	pub state_diff: Option<StateDiff>,
}

impl From<Executed> for TraceResults {
	fn from(t: Executed) -> Self {
		TraceResults {
			output: t.output.into(),
			trace: t.trace.into_iter().map(Into::into).collect(),
			vm_trace: t.vm_trace.map(Into::into),
			state_diff: t.state_diff.map(Into::into),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use std::collections::BTreeMap;
	use v1::types::{Bytes, U256, H256, H160};
	use super::*;

	#[test]
	fn should_serialize_trace_results() {
		let r = TraceResults {
			output: vec![0x60],
			trace: vec![],
			vm_trace: None,
			state_diff: None,
		};
		let serialized = serde_json::to_string(&r).unwrap();
		assert_eq!(serialized, r#"{"output":[96],"trace":[],"vmTrace":null,"stateDiff":null}"#);
	}

	#[test]
	fn test_trace_serialize() {
		let t = LocalizedTrace {
			action: Action::Call(Call {
				from: H160::from(4),
				to: H160::from(5),
				value: U256::from(6),
				gas: U256::from(7),
				input: Bytes::new(vec![0x12, 0x34]),
				call_type: CallType::Call,
			}),
			result: Res::Call(CallResult {
				gas_used: U256::from(8),
				output: vec![0x56, 0x78].into(),
			}),
			trace_address: vec![U256::from(10)],
			subtraces: U256::from(1),
			transaction_position: U256::from(11),
			transaction_hash: H256::from(12),
			block_number: U256::from(13),
			block_hash: H256::from(14),
		};
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"action":{"call":{"from":"0x0000000000000000000000000000000000000004","to":"0x0000000000000000000000000000000000000005","value":"0x06","gas":"0x07","input":"0x1234","callType":{"call":[]}}},"result":{"call":{"gasUsed":"0x08","output":"0x5678"}},"traceAddress":["0x0a"],"subtraces":"0x01","transactionPosition":"0x0b","transactionHash":"0x000000000000000000000000000000000000000000000000000000000000000c","blockNumber":"0x0d","blockHash":"0x000000000000000000000000000000000000000000000000000000000000000e"}"#);
	}

	#[test]
	fn test_vmtrace_serialize() {
		let t = VMTrace {
			code: vec![0, 1, 2, 3],
			ops: vec![
				VMOperation {
					pc: 0,
					cost: 10,
					ex: None,
					sub: None,
				},
				VMOperation {
					pc: 1,
					cost: 11,
					ex: Some(VMExecutedOperation {
						used: 10,
						push: vec![69.into()],
						mem: None,
						store: None,
					}),
					sub: Some(VMTrace {
						code: vec![0],
						ops: vec![
							VMOperation {
								pc: 0,
								cost: 0,
								ex: Some(VMExecutedOperation {
									used: 10,
									push: vec![42.into()],
									mem: Some(MemoryDiff {off: 42, data: vec![1, 2, 3]}),
									store: Some(StorageDiff {key: 69.into(), val: 42.into()}),
								}),
								sub: None,
							}
						]
					}),
				}
			]
		};
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"code":[0,1,2,3],"ops":[{"pc":0,"cost":10,"ex":null,"sub":null},{"pc":1,"cost":11,"ex":{"used":10,"push":["0x45"],"mem":null,"store":null},"sub":{"code":[0],"ops":[{"pc":0,"cost":0,"ex":{"used":10,"push":["0x2a"],"mem":{"off":42,"data":[1,2,3]},"store":{"key":"0x45","val":"0x2a"}},"sub":null}]}}]}"#);
	}

	#[test]
	fn test_statediff_serialize() {
		let t = StateDiff(map![
			42.into() => AccountDiff {
				balance: Diff::Same,
				nonce: Diff::Born(1.into()),
				code: Diff::Same,
				storage: map![
					42.into() => Diff::Same
				]
			},
			69.into() => AccountDiff {
				balance: Diff::Same,
				nonce: Diff::Changed(ChangedType { from: 1.into(), to: 0.into() }),
				code: Diff::Died(vec![96].into()),
				storage: map![],
			}
		]);
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"0x000000000000000000000000000000000000002a":{"balance":{"=":[]},"nonce":{"+":"0x01"},"code":{"=":[]},"storage":{"0x000000000000000000000000000000000000000000000000000000000000002a":{"=":[]}}},"0x0000000000000000000000000000000000000045":{"balance":{"=":[]},"nonce":{"*":{"from":"0x01","to":"0x00"}},"code":{"-":"0x60"},"storage":{}}}"#);
	}

	#[test]
	fn test_action_serialize() {
		let actions = vec![Action::Call(Call {
			from: H160::from(1),
			to: H160::from(2),
			value: U256::from(3),
			gas: U256::from(4),
			input: vec![0x12, 0x34].into(),
			call_type: CallType::Call,
		}), Action::Create(Create {
			from: H160::from(5),
			value: U256::from(6),
			gas: U256::from(7),
			init: vec![0x56, 0x78].into(),
		})];

		let serialized = serde_json::to_string(&actions).unwrap();
		assert_eq!(serialized, r#"[{"call":{"from":"0x0000000000000000000000000000000000000001","to":"0x0000000000000000000000000000000000000002","value":"0x03","gas":"0x04","input":"0x1234","callType":{"call":[]}}},{"create":{"from":"0x0000000000000000000000000000000000000005","value":"0x06","gas":"0x07","init":"0x5678"}}]"#);
	}

	#[test]
	fn test_result_serialize() {
		let results = vec![
			Res::Call(CallResult {
				gas_used: U256::from(1),
				output: vec![0x12, 0x34].into(),
			}),
			Res::Create(CreateResult {
				gas_used: U256::from(2),
				code: vec![0x45, 0x56].into(),
				address: H160::from(3),
			}),
			Res::FailedCall,
			Res::FailedCreate,
		];

		let serialized = serde_json::to_string(&results).unwrap();
		assert_eq!(serialized, r#"[{"call":{"gasUsed":"0x01","output":"0x1234"}},{"create":{"gasUsed":"0x02","code":"0x4556","address":"0x0000000000000000000000000000000000000003"}},{"failedCall":[]},{"failedCreate":[]}]"#);
	}
}
