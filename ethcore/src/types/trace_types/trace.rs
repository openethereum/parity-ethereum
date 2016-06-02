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

//! Tracing datatypes.

use util::{U256, Bytes, Address, FixedHash};
use util::rlp::*;
use util::sha3::Hashable;
use action_params::ActionParams;
use basic_types::LogBloom;
use ipc::binary::BinaryConvertError;
use std::mem;
use std::collections::VecDeque;

/// `Call` result.
#[derive(Debug, Clone, PartialEq, Default, Binary)]
pub struct CallResult {
	/// Gas used by call.
	pub gas_used: U256,
	/// Call Output.
	pub output: Bytes,
}

impl Encodable for CallResult {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.gas_used);
		s.append(&self.output);
	}
}

impl Decodable for CallResult {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = CallResult {
			gas_used: try!(d.val_at(0)),
			output: try!(d.val_at(1)),
		};

		Ok(res)
	}
}

/// `Create` result.
#[derive(Debug, Clone, PartialEq, Binary)]
pub struct CreateResult {
	/// Gas used by create.
	pub gas_used: U256,
	/// Code of the newly created contract.
	pub code: Bytes,
	/// Address of the newly created contract.
	pub address: Address,
}

impl Encodable for CreateResult {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(3);
		s.append(&self.gas_used);
		s.append(&self.code);
		s.append(&self.address);
	}
}

impl Decodable for CreateResult {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = CreateResult {
			gas_used: try!(d.val_at(0)),
			code: try!(d.val_at(1)),
			address: try!(d.val_at(2)),
		};

		Ok(res)
	}
}

/// Description of a _call_ action, either a `CALL` operation or a message transction.
#[derive(Debug, Clone, PartialEq, Binary)]
pub struct Call {
	/// The sending account.
	pub from: Address,
	/// The destination account.
	pub to: Address,
	/// The value transferred to the destination account.
	pub value: U256,
	/// The gas available for executing the call.
	pub gas: U256,
	/// The input data provided to the call.
	pub input: Bytes,
}

impl From<ActionParams> for Call {
	fn from(p: ActionParams) -> Self {
		Call {
			from: p.sender,
			to: p.address,
			value: p.value.value(),
			gas: p.gas,
			input: p.data.unwrap_or_else(Vec::new),
		}
	}
}

impl Encodable for Call {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(5);
		s.append(&self.from);
		s.append(&self.to);
		s.append(&self.value);
		s.append(&self.gas);
		s.append(&self.input);
	}
}

impl Decodable for Call {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = Call {
			from: try!(d.val_at(0)),
			to: try!(d.val_at(1)),
			value: try!(d.val_at(2)),
			gas: try!(d.val_at(3)),
			input: try!(d.val_at(4)),
		};

		Ok(res)
	}
}

impl Call {
	/// Returns call action bloom.
	/// The bloom contains from and to addresses.
	pub fn bloom(&self) -> LogBloom {
		LogBloom::from_bloomed(&self.from.sha3())
			.with_bloomed(&self.to.sha3())
	}
}

/// Description of a _create_ action, either a `CREATE` operation or a create transction.
#[derive(Debug, Clone, PartialEq, Binary)]
pub struct Create {
	/// The address of the creator.
	pub from: Address,
	/// The value with which the new account is endowed.
	pub value: U256,
	/// The gas available for the creation init code.
	pub gas: U256,
	/// The init code.
	pub init: Bytes,
}

impl From<ActionParams> for Create {
	fn from(p: ActionParams) -> Self {
		Create {
			from: p.sender,
			value: p.value.value(),
			gas: p.gas,
			init: p.code.unwrap_or_else(Vec::new),
		}
	}
}

impl Encodable for Create {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.from);
		s.append(&self.value);
		s.append(&self.gas);
		s.append(&self.init);
	}
}

impl Decodable for Create {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = Create {
			from: try!(d.val_at(0)),
			value: try!(d.val_at(1)),
			gas: try!(d.val_at(2)),
			init: try!(d.val_at(3)),
		};

		Ok(res)
	}
}

impl Create {
	/// Returns bloom create action bloom.
	/// The bloom contains only from address.
	pub fn bloom(&self) -> LogBloom {
		LogBloom::from_bloomed(&self.from.sha3())
	}
}

/// Description of an action that we trace; will be either a call or a create.
#[derive(Debug, Clone, PartialEq, Binary)]
pub enum Action {
	/// It's a call action.
	Call(Call),
	/// It's a create action.
	Create(Create),
}

impl Encodable for Action {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		match *self {
			Action::Call(ref call) => {
				s.append(&0u8);
				s.append(call);
			},
			Action::Create(ref create) => {
				s.append(&1u8);
				s.append(create);
			}
		}
	}
}

impl Decodable for Action {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let action_type: u8 = try!(d.val_at(0));
		match action_type {
			0 => d.val_at(1).map(Action::Call),
			1 => d.val_at(1).map(Action::Create),
			_ => Err(DecoderError::Custom("Invalid action type.")),
		}
	}
}

impl Action {
	/// Returns action bloom.
	pub fn bloom(&self) -> LogBloom {
		match *self {
			Action::Call(ref call) => call.bloom(),
			Action::Create(ref create) => create.bloom(),
		}
	}
}

/// The result of the performed action.
#[derive(Debug, Clone, PartialEq, Binary)]
pub enum Res {
	/// Successful call action result.
	Call(CallResult),
	/// Successful create action result.
	Create(CreateResult),
	/// Failed call.
	FailedCall,
	/// Failed create.
	FailedCreate,
}

impl Encodable for Res {
	fn rlp_append(&self, s: &mut RlpStream) {
		match *self {
			Res::Call(ref call) => {
				s.begin_list(2);
				s.append(&0u8);
				s.append(call);
			},
			Res::Create(ref create) => {
				s.begin_list(2);
				s.append(&1u8);
				s.append(create);
			},
			Res::FailedCall => {
				s.begin_list(1);
				s.append(&2u8);
			},
			Res::FailedCreate => {
				s.begin_list(1);
				s.append(&3u8);
			}
		}
	}
}

impl Decodable for Res {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let action_type: u8 = try!(d.val_at(0));
		match action_type {
			0 => d.val_at(1).map(Res::Call),
			1 => d.val_at(1).map(Res::Create),
			2 => Ok(Res::FailedCall),
			3 => Ok(Res::FailedCreate),
			_ => Err(DecoderError::Custom("Invalid result type.")),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Binary)]
/// A trace; includes a description of the action being traced and sub traces of each interior action.
pub struct Trace {
	/// The number of EVM execution environments active when this action happened; 0 if it's
	/// the outer action of the transaction.
	pub depth: usize,
	/// The action being performed.
	pub action: Action,
	/// The sub traces for each interior action performed as part of this call.
	pub subs: Vec<Trace>,
	/// The result of the performed action.
	pub result: Res,
}

impl Encodable for Trace {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.depth);
		s.append(&self.action);
		s.append(&self.subs);
		s.append(&self.result);
	}
}

impl Decodable for Trace {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = Trace {
			depth: try!(d.val_at(0)),
			action: try!(d.val_at(1)),
			subs: try!(d.val_at(2)),
			result: try!(d.val_at(3)),
		};

		Ok(res)
	}
}

impl Trace {
	/// Returns trace bloom.
	pub fn bloom(&self) -> LogBloom {
		self.subs.iter().fold(self.action.bloom(), |b, s| b | s.bloom())
	}
}

#[derive(Debug, Clone, PartialEq, Binary)]
/// A diff of some chunk of memory.
pub struct MemoryDiff {
	/// Offset into memory the change begins.
	pub offset: usize,
	/// The changed data.
	pub data: Bytes,
}

impl Encodable for MemoryDiff {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.offset);
		s.append(&self.data);
	}
}

impl Decodable for MemoryDiff {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		Ok(MemoryDiff {
			offset: try!(d.val_at(0)),
			data: try!(d.val_at(1)),
		})
	}
}

#[derive(Debug, Clone, PartialEq, Binary)]
/// A diff of some storage value.
pub struct StorageDiff {
	/// Which key in storage is changed.
	pub location: U256,
	/// What the value has been changed to.
	pub value: U256,
}

impl Encodable for StorageDiff {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.location);
		s.append(&self.value);
	}
}

impl Decodable for StorageDiff {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		Ok(StorageDiff {
			location: try!(d.val_at(0)),
			value: try!(d.val_at(1)),
		})
	}
}

#[derive(Debug, Clone, PartialEq, Binary)]
/// A record of an executed VM operation.
pub struct VMExecutedOperation {
	/// The total gas used.
	pub gas_used: U256,
	/// The stack item placed, if any.
	pub stack_push: Vec<U256>,
	/// If altered, the memory delta.
	pub mem_diff: Option<MemoryDiff>,
	/// The altered storage value, if any.
	pub store_diff: Option<StorageDiff>,
}

impl Encodable for VMExecutedOperation {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.gas_used);
		s.append(&self.stack_push);
		s.append(&self.mem_diff);
		s.append(&self.store_diff);
	}
}

impl Decodable for VMExecutedOperation {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		Ok(VMExecutedOperation {
			gas_used: try!(d.val_at(0)),
			stack_push: try!(d.val_at(1)),
			mem_diff: try!(d.val_at(2)),
			store_diff: try!(d.val_at(3)),
		})
	}
}

#[derive(Debug, Clone, PartialEq, Binary)]
/// A record of the execution of a single VM operation.
pub struct VMOperation {
	/// The program counter.
	pub pc: usize,
	/// The instruction executed.
	pub instruction: u8,
	/// The gas cost for this instruction.
	pub gas_cost: U256,
	/// Information concerning the execution of the operation.
	pub executed: Option<VMExecutedOperation>,
}

impl Encodable for VMOperation {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.pc);
		s.append(&self.instruction);
		s.append(&self.gas_cost);
		s.append(&self.executed);
	}
}

impl Decodable for VMOperation {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = VMOperation {
			pc: try!(d.val_at(0)),
			instruction: try!(d.val_at(1)),
			gas_cost: try!(d.val_at(2)),
			executed: try!(d.val_at(3)),
		};

		Ok(res)
	}
}

#[derive(Debug, Clone, PartialEq, Binary, Default)]
/// A record of a full VM trace for a CALL/CREATE.
pub struct VMTrace {
	/// The step (i.e. index into operations) at which this trace corresponds.
	pub parent_step: usize,
	/// The code to be executed.
	pub code: Bytes,
	/// The operations executed.
	pub operations: Vec<VMOperation>,
	/// The sub traces for each interior action performed as part of this call/create.
	/// Thre is a 1:1 correspondance between these and a CALL/CREATE/CALLCODE/DELEGATECALL instruction.
	pub subs: Vec<VMTrace>,
}

impl Encodable for VMTrace {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.parent_step);
		s.append(&self.code);
		s.append(&self.operations);
		s.append(&self.subs);
	}
}

impl Decodable for VMTrace {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = VMTrace {
			parent_step: try!(d.val_at(0)),
			code: try!(d.val_at(1)),
			operations: try!(d.val_at(2)),
			subs: try!(d.val_at(3)),
		};

		Ok(res)
	}
}

#[cfg(test)]
mod tests {
	use util::{Address, U256, FixedHash};
	use util::rlp::{encode, decode};
	use util::sha3::Hashable;
	use trace::trace::{Call, CallResult, Create, Res, Action, Trace};

	#[test]
	fn traces_rlp() {
		let trace = Trace {
			depth: 2,
			action: Action::Call(Call {
				from: Address::from(1),
				to: Address::from(2),
				value: U256::from(3),
				gas: U256::from(4),
				input: vec![0x5]
			}),
			subs: vec![
				Trace {
					depth: 3,
					action: Action::Create(Create {
						from: Address::from(6),
						value: U256::from(7),
						gas: U256::from(8),
						init: vec![0x9]
					}),
					subs: vec![],
					result: Res::FailedCreate
				}
			],
			result: Res::Call(CallResult {
				gas_used: U256::from(10),
				output: vec![0x11, 0x12]
			})
		};

		let encoded = encode(&trace);
		let decoded: Trace = decode(&encoded);
		assert_eq!(trace, decoded);
	}

	#[test]
	fn traces_bloom() {
		let trace = Trace {
			depth: 2,
			action: Action::Call(Call {
				from: Address::from(1),
				to: Address::from(2),
				value: U256::from(3),
				gas: U256::from(4),
				input: vec![0x5]
			}),
			subs: vec![
				Trace {
					depth: 3,
					action: Action::Create(Create {
						from: Address::from(6),
						value: U256::from(7),
						gas: U256::from(8),
						init: vec![0x9]
					}),
					subs: vec![],
					result: Res::FailedCreate
				}
			],
			result: Res::Call(CallResult {
				gas_used: U256::from(10),
				output: vec![0x11, 0x12]
			})
		};

		let bloom = trace.bloom();

		// right now only addresses are bloomed
		assert!(bloom.contains_bloomed(&Address::from(1).sha3()));
		assert!(bloom.contains_bloomed(&Address::from(2).sha3()));
		assert!(!bloom.contains_bloomed(&Address::from(20).sha3()));
		assert!(bloom.contains_bloomed(&Address::from(6).sha3()));
	}
}
