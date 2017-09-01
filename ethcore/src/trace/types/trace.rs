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
use hash::keccak;
use bloomable::Bloomable;
use rlp::*;

use vm::ActionParams;
use basic_types::LogBloom;
use evm::CallType;
use super::error::Error;

/// `Call` result.
#[derive(Debug, Clone, PartialEq, Default, RlpEncodable, RlpDecodable)]
#[cfg_attr(feature = "ipc", binary)]
pub struct CallResult {
	/// Gas used by call.
	pub gas_used: U256,
	/// Call Output.
	pub output: Bytes,
}

/// `Create` result.
#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
#[cfg_attr(feature = "ipc", binary)]
pub struct CreateResult {
	/// Gas used by create.
	pub gas_used: U256,
	/// Code of the newly created contract.
	pub code: Bytes,
	/// Address of the newly created contract.
	pub address: Address,
}

impl CreateResult {
	/// Returns bloom.
	pub fn bloom(&self) -> LogBloom {
		LogBloom::from_bloomed(&keccak(&self.address))
	}
}

/// Description of a _call_ action, either a `CALL` operation or a message transction.
#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
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

impl Call {
	/// Returns call action bloom.
	/// The bloom contains from and to addresses.
	pub fn bloom(&self) -> LogBloom {
		LogBloom::from_bloomed(&keccak(&self.from))
			.with_bloomed(&keccak(&self.to))
	}
}

/// Description of a _create_ action, either a `CREATE` operation or a create transction.
#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
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

impl Create {
	/// Returns bloom create action bloom.
	/// The bloom contains only from address.
	pub fn bloom(&self) -> LogBloom {
		LogBloom::from_bloomed(&keccak(&self.from))
	}
}

/// Reward type.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "ipc", binary)]
pub enum RewardType {
	/// Block
	Block,
	/// Uncle
	Uncle,
}

impl Encodable for RewardType {
	fn rlp_append(&self, s: &mut RlpStream) {
		let v = match *self {
			RewardType::Block => 0u32,
			RewardType::Uncle => 1,
		};
		Encodable::rlp_append(&v, s);
	}
}

impl Decodable for RewardType {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		rlp.as_val().and_then(|v| Ok(match v {
			0u32 => RewardType::Block,
			1 => RewardType::Uncle,
			_ => return Err(DecoderError::Custom("Invalid value of RewardType item")),
		}))
	}
}

/// Reward action
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct Reward {
	/// Author's address.
	pub author: Address,
	/// Reward amount.
	pub value: U256,
	/// Reward type.
	pub reward_type: RewardType,
}

impl Reward {
	/// Return reward action bloom.
	pub fn bloom(&self) -> LogBloom {
		LogBloom::from_bloomed(&keccak(&self.author))
	}
}

impl Encodable for Reward {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(3);
		s.append(&self.author);
		s.append(&self.value);
		s.append(&self.reward_type);
	}
}

impl Decodable for Reward {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		let res = Reward {
			author: rlp.val_at(0)?,
			value: rlp.val_at(1)?,
			reward_type: rlp.val_at(2)?,
		};

		Ok(res)
	}
}


/// Suicide action.
#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
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
		LogBloom::from_bloomed(&keccak(self.address))
			.with_bloomed(&keccak(self.refund_address))
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
	/// Reward
	Reward(Reward),
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
			},
			Action::Reward(ref reward) => {
				s.append(&3u8);
				s.append(reward);
			}

		}
	}
}

impl Decodable for Action {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		let action_type: u8 = rlp.val_at(0)?;
		match action_type {
			0 => rlp.val_at(1).map(Action::Call),
			1 => rlp.val_at(1).map(Action::Create),
			2 => rlp.val_at(1).map(Action::Suicide),
			3 => rlp.val_at(1).map(Action::Reward),
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
			Action::Reward(ref reward) => reward.bloom(),
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
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		let action_type: u8 = rlp.val_at(0)?;
		match action_type {
			0 => rlp.val_at(1).map(Res::Call),
			1 => rlp.val_at(1).map(Res::Create),
			2 => rlp.val_at(1).map(Res::FailedCall),
			3 => rlp.val_at(1).map(Res::FailedCreate),
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

#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
#[cfg_attr(feature = "ipc", binary)]
/// A diff of some chunk of memory.
pub struct MemoryDiff {
	/// Offset into memory the change begins.
	pub offset: usize,
	/// The changed data.
	pub data: Bytes,
}

#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
#[cfg_attr(feature = "ipc", binary)]
/// A diff of some storage value.
pub struct StorageDiff {
	/// Which key in storage is changed.
	pub location: U256,
	/// What the value has been changed to.
	pub value: U256,
}

#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
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

#[derive(Debug, Clone, PartialEq, Default, RlpEncodable, RlpDecodable)]
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

#[derive(Debug, Clone, PartialEq, Default, RlpEncodable, RlpDecodable)]
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
