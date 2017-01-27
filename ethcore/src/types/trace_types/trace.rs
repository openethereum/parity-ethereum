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

//! Tracing datatypes.

use util::{U256, Bytes, Address};
use util::sha3::Hashable;
use util::bloom::Bloomable;
use rlp::*;

use action_params::ActionParams;
use basic_types::LogBloom;
use types::executed::CallType;
use super::error::Error;

/// `Call` result.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "ipc", binary)]
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
			gas_used: d.val_at(0)?,
			output: d.val_at(1)?,
		};

		Ok(res)
	}
}

/// `Create` result.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
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
			gas_used: d.val_at(0)?,
			code: d.val_at(1)?,
			address: d.val_at(2)?,
		};

		Ok(res)
	}
}

impl CreateResult {
	/// Returns bloom.
	pub fn bloom(&self) -> LogBloom {
		LogBloom::from_bloomed(&self.address.sha3())
	}
}

/// Description of a _call_ action, either a `CALL` operation or a message transction.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
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
	/// The type of the call.
	pub call_type: CallType,
}

impl From<ActionParams> for Call {
	fn from(p: ActionParams) -> Self {
		Call {
			from: p.sender,
			to: p.address,
			value: p.value.value(),
			gas: p.gas,
			input: p.data.unwrap_or_else(Vec::new),
			call_type: p.call_type,
		}
	}
}

impl Encodable for Call {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(6);
		s.append(&self.from);
		s.append(&self.to);
		s.append(&self.value);
		s.append(&self.gas);
		s.append(&self.input);
		s.append(&self.call_type);
	}
}

impl Decodable for Call {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = Call {
			from: d.val_at(0)?,
			to: d.val_at(1)?,
			value: d.val_at(2)?,
			gas: d.val_at(3)?,
			input: d.val_at(4)?,
			call_type: d.val_at(5)?,
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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
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
			init: p.code.map_or_else(Vec::new, |c| (*c).clone()),
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
			from: d.val_at(0)?,
			value: d.val_at(1)?,
			gas: d.val_at(2)?,
			init: d.val_at(3)?,
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

/// Suicide action.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct Suicide {
	/// Suicided address.
	pub address: Address,
	/// Suicided contract heir.
	pub refund_address: Address,
	/// Balance of the contract just before suicide.
	pub balance: U256,
}

impl Suicide {
	/// Return suicide action bloom.
	pub fn bloom(&self) -> LogBloom {
		LogBloom::from_bloomed(&self.address.sha3())
			.with_bloomed(&self.refund_address.sha3())
	}
}

impl Encodable for Suicide {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(3);
		s.append(&self.address);
		s.append(&self.refund_address);
		s.append(&self.balance);
	}
}

impl Decodable for Suicide {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let res = Suicide {
			address: d.val_at(0)?,
			refund_address: d.val_at(1)?,
			balance: d.val_at(2)?,
		};

		Ok(res)
	}
}


/// Description of an action that we trace; will be either a call or a create.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
pub enum Action {
	/// It's a call action.
	Call(Call),
	/// It's a create action.
	Create(Create),
	/// Suicide.
	Suicide(Suicide),
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
			},
			Action::Suicide(ref suicide) => {
				s.append(&2u8);
				s.append(suicide);
			}
		}
	}
}

impl Decodable for Action {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let action_type: u8 = d.val_at(0)?;
		match action_type {
			0 => d.val_at(1).map(Action::Call),
			1 => d.val_at(1).map(Action::Create),
			2 => d.val_at(1).map(Action::Suicide),
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
			Action::Suicide(ref suicide) => suicide.bloom(),
		}
	}
}

/// The result of the performed action.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
pub enum Res {
	/// Successful call action result.
	Call(CallResult),
	/// Successful create action result.
	Create(CreateResult),
	/// Failed call.
	FailedCall(Error),
	/// Failed create.
	FailedCreate(Error),
	/// None
	None,
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
			Res::FailedCall(ref err) => {
				s.begin_list(2);
				s.append(&2u8);
				s.append(err);
			},
			Res::FailedCreate(ref err) => {
				s.begin_list(2);
				s.append(&3u8);
				s.append(err);
			},
			Res::None => {
				s.begin_list(1);
				s.append(&4u8);
			}
		}
	}
}

impl Decodable for Res {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let action_type: u8 = d.val_at(0)?;
		match action_type {
			0 => d.val_at(1).map(Res::Call),
			1 => d.val_at(1).map(Res::Create),
			2 => d.val_at(1).map(Res::FailedCall),
			3 => d.val_at(1).map(Res::FailedCreate),
			4 => Ok(Res::None),
			_ => Err(DecoderError::Custom("Invalid result type.")),
		}
	}
}

impl Res {
	/// Returns result bloom.
	pub fn bloom(&self) -> LogBloom {
		match *self {
			Res::Create(ref create) => create.bloom(),
			Res::Call(_) | Res::FailedCall(_) | Res::FailedCreate(_) | Res::None => Default::default(),
		}
	}

	/// Did this call fail?
	pub fn succeeded(&self) -> bool {
		match *self {
			Res::Call(_) | Res::Create(_) => true,
			_ => false,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
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
			offset: d.val_at(0)?,
			data: d.val_at(1)?,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
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
			location: d.val_at(0)?,
			value: d.val_at(1)?,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
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
			gas_used: d.val_at(0)?,
			stack_push: d.val_at(1)?,
			mem_diff: d.val_at(2)?,
			store_diff: d.val_at(3)?,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "ipc", binary)]
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
			pc: d.val_at(0)?,
			instruction: d.val_at(1)?,
			gas_cost: d.val_at(2)?,
			executed: d.val_at(3)?,
		};

		Ok(res)
	}
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "ipc", binary)]
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
			parent_step: d.val_at(0)?,
			code: d.val_at(1)?,
			operations: d.val_at(2)?,
			subs: d.val_at(3)?,
		};

		Ok(res)
	}
}
