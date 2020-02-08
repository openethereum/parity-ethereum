// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Tracing data types.

// ================== NOTE ========================
// IF YOU'RE ADDING A FIELD TO A STRUCT WITH
// RLP ENCODING, MAKE SURE IT'S DONE IN A BACKWARDS
// COMPATIBLE WAY!
// ================== NOTE ========================

use std::convert::TryFrom;
use ethereum_types::{U256, Address, Bloom, BloomInput};
use parity_bytes::Bytes;
use rlp::{Rlp, RlpStream, Encodable, DecoderError, Decodable};
use rlp_derive::{RlpEncodable, RlpDecodable};
use vm::ActionParams;
use evm::ActionType;
use super::error::Error;

/// `Call` result.
#[derive(Debug, Clone, PartialEq, Default, RlpEncodable, RlpDecodable)]
pub struct CallResult {
	/// Gas used by call.
	pub gas_used: U256,
	/// Call Output.
	pub output: Bytes,
}

/// `Call` type. Distinguish between different types of contract interactions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CallType {
	/// Call
	Call,
	/// Call code
	CallCode,
	/// Delegate call
	DelegateCall,
	/// Static call
	StaticCall,
}

impl TryFrom<ActionType> for CallType {
	type Error = &'static str;
	fn try_from(action_type: ActionType) -> Result<Self, Self::Error> {
		match action_type {
			ActionType::Call => Ok(CallType::Call),
			ActionType::CallCode => Ok(CallType::CallCode),
			ActionType::DelegateCall => Ok(CallType::DelegateCall),
			ActionType::StaticCall => Ok(CallType::StaticCall),
			ActionType::Create => Err("Create cannot be converted to CallType"),
			ActionType::Create2 => Err("Create2 cannot be converted to CallType"),
		}
	}
}

/// `Create` result.
#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
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
	pub fn bloom(&self) -> Bloom {
		BloomInput::Raw(self.address.as_bytes()).into()
	}
}

/// `Create` method. Distinguish between use of `CREATE` and `CREATE2` opcodes in an action.
#[derive(Debug, Clone, PartialEq)]
pub enum CreationMethod {
	/// Create
	Create,
	/// Create2
	Create2,
}

impl TryFrom<ActionType> for CreationMethod {
	type Error = &'static str;
	fn try_from(action_type: ActionType) -> Result<Self, Self::Error> {
		match action_type {
			ActionType::Call => Err("Call cannot be converted to CreationMethod"),
			ActionType::CallCode => Err("CallCode cannot be converted to CreationMethod"),
			ActionType::DelegateCall => Err("DelegateCall cannot be converted to CreationMethod"),
			ActionType::StaticCall => Err("StaticCall cannot be converted to CreationMethod"),
			ActionType::Create => Ok(CreationMethod::Create),
			ActionType::Create2 => Ok(CreationMethod::Create2),
		}
	}
}

impl Encodable for CreationMethod {
	fn rlp_append(&self, s: &mut RlpStream) {
		let v = match *self {
			CreationMethod::Create => 0u32,
			CreationMethod::Create2 => 1,
		};
		Encodable::rlp_append(&v, s);
	}
}

impl Decodable for CreationMethod {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.as_val().and_then(|v| Ok(match v {
			0u32 => CreationMethod::Create,
			1 => CreationMethod::Create2,
			_ => return Err(DecoderError::Custom("Invalid value of CreationMethod item")),
		}))
	}
}

/// Description of a _call_ action, either a `CALL` operation or a message transaction.
#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
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
	pub call_type: BackwardsCompatibleCallType,
}

/// This is essentially an `Option<CallType>`, but with a custom
/// `rlp` en/de-coding which preserves backwards compatibility with
/// the older encodings used in parity-ethereum versions < 2.7 and 2.7.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackwardsCompatibleCallType(pub Option<CallType>);

impl From<Option<CallType>> for BackwardsCompatibleCallType {
	fn from(option: Option<CallType>) -> Self {
		BackwardsCompatibleCallType(option)
	}
}

// Encoding is the same as `CallType_v2_6_x`.
impl Encodable for BackwardsCompatibleCallType {
	fn rlp_append(&self, s: &mut RlpStream) {
		let v = match self.0 {
			None => 0u32,
			Some(CallType::Call) => 1,
			Some(CallType::CallCode) => 2,
			Some(CallType::DelegateCall) => 3,
			Some(CallType::StaticCall) => 4,
		};
		Encodable::rlp_append(&v, s);
	}
}

// Try to decode it as `CallType_v2_6_x` first, and then as `Option<CallType_v2_7_0>`.
impl Decodable for BackwardsCompatibleCallType {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		if rlp.is_data() {
			rlp.as_val().and_then(|v| Ok(match v {
				0u32 => None,
				1 => Some(CallType::Call),
				2 => Some(CallType::CallCode),
				3 => Some(CallType::DelegateCall),
				4 => Some(CallType::StaticCall),
				_ => return Err(DecoderError::Custom("Invalid value of CallType item")),
			}.into()))
		} else {
			#[allow(non_camel_case_types)]
			#[derive(Debug, Clone, Copy, PartialEq)]
			enum CallType_v2_7_0 {
				Call,
				CallCode,
				DelegateCall,
				StaticCall,
			}

			impl Decodable for CallType_v2_7_0 {
				fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
					rlp.as_val().and_then(|v| Ok(match v {
						0u32 => CallType_v2_7_0::Call,
						1 => CallType_v2_7_0::CallCode,
						2 => CallType_v2_7_0::DelegateCall,
						3 => CallType_v2_7_0::StaticCall,
						_ => return Err(DecoderError::Custom("Invalid value of CallType item")),
					}))
				}
			}

			impl From<CallType_v2_7_0> for CallType {
				fn from(old_call_type: CallType_v2_7_0) -> Self {
					match old_call_type {
						CallType_v2_7_0::Call => Self::Call,
						CallType_v2_7_0::CallCode => Self::CallCode,
						CallType_v2_7_0::DelegateCall => Self::DelegateCall,
						CallType_v2_7_0::StaticCall => Self::StaticCall,
					}
				}
			}

			let optional: Option<CallType_v2_7_0> = Decodable::decode(rlp)?;
			Ok(optional.map(Into::into).into())
		}
	}
}

impl From<ActionParams> for Call {
	fn from(p: ActionParams) -> Self {
		match p.action_type {
			ActionType::DelegateCall | ActionType::CallCode => Call {
				from: p.address,
				to: p.code_address,
				value: p.value.value(),
				gas: p.gas,
				input: p.data.unwrap_or_else(Vec::new),
				call_type: CallType::try_from(p.action_type).ok().into(),
			},
			_ => Call {
				from: p.sender,
				to: p.address,
				value: p.value.value(),
				gas: p.gas,
				input: p.data.unwrap_or_else(Vec::new),
				call_type: CallType::try_from(p.action_type).ok().into(),
			},
		}
	}
}

impl Call {
	/// Returns call action bloom.
	/// The bloom contains from and to addresses.
	pub fn bloom(&self) -> Bloom {
		let mut bloom = Bloom::default();
		bloom.accrue(BloomInput::Raw(self.from.as_bytes()));
		bloom.accrue(BloomInput::Raw(self.to.as_bytes()));
		bloom
	}
}

/// Description of a _create_ action, either a `CREATE` operation or a create transaction.
#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
pub struct Create {
	/// The address of the creator.
	pub from: Address,
	/// The value with which the new account is endowed.
	pub value: U256,
	/// The gas available for the creation init code.
	pub gas: U256,
	/// The init code.
	pub init: Bytes,
	/// Creation method (CREATE vs CREATE2).
	#[rlp(default)]
	pub creation_method: Option<CreationMethod>,
}

impl From<ActionParams> for Create {
	fn from(p: ActionParams) -> Self {
		Create {
			from: p.sender,
			value: p.value.value(),
			gas: p.gas,
			init: p.code.map_or_else(Vec::new, |c| (*c).clone()),
			creation_method: CreationMethod::try_from(p.action_type).ok().into(),
		}
	}
}

impl Create {
	/// Returns bloom create action bloom.
	/// The bloom contains only from address.
	pub fn bloom(&self) -> Bloom {
		BloomInput::Raw(self.from.as_bytes()).into()
	}
}

/// Reward type.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RewardType {
	/// Block
	Block,
	/// Uncle
	Uncle,
	/// Empty step (AuthorityRound)
	EmptyStep,
	/// A reward directly attributed by an external protocol (e.g. block reward contract)
	External,
}

impl Encodable for RewardType {
	fn rlp_append(&self, s: &mut RlpStream) {
		let v = match *self {
			RewardType::Block => 0u32,
			RewardType::Uncle => 1,
			RewardType::EmptyStep => 2,
			RewardType::External => 3,
		};
		Encodable::rlp_append(&v, s);
	}
}

impl Decodable for RewardType {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.as_val().and_then(|v| Ok(match v {
			0u32 => RewardType::Block,
			1 => RewardType::Uncle,
			2 => RewardType::EmptyStep,
			3 => RewardType::External,
			_ => return Err(DecoderError::Custom("Invalid value of RewardType item")),
		}))
	}
}

/// Reward action
#[derive(Debug, Clone, PartialEq)]
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
	pub fn bloom(&self) -> Bloom {
		BloomInput::Raw(self.author.as_bytes()).into()
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
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
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
	pub fn bloom(&self) -> Bloom {
		let mut bloom = Bloom::default();
		bloom.accrue(BloomInput::Raw(self.address.as_bytes()));
		bloom.accrue(BloomInput::Raw(self.refund_address.as_bytes()));
		bloom
	}
}

/// Description of an action that we trace; will be either a call or a create.
#[derive(Debug, Clone, PartialEq)]
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
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
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
	pub fn bloom(&self) -> Bloom {
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
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
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
	pub fn bloom(&self) -> Bloom {
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
/// A diff of some chunk of memory.
pub struct MemoryDiff {
	/// Offset into memory the change begins.
	pub offset: usize,
	/// The changed data.
	pub data: Bytes,
}

#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
/// A diff of some storage value.
pub struct StorageDiff {
	/// Which key in storage is changed.
	pub location: U256,
	/// What the value has been changed to.
	pub value: U256,
}

#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
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

#[cfg(test)]
mod tests {
	use rlp::{RlpStream, Encodable};
	use rlp_derive::{RlpEncodable, RlpDecodable};
	use super::{Address, Bytes, Call, CallType, Create, CreationMethod, U256};

	#[test]
	fn test_call_type_backwards_compatibility() {
		// Call type in version < 2.7.
		#[derive(Debug, Clone, PartialEq, RlpEncodable)]
		struct OldCall {
			from: Address,
			to: Address,
			value: U256,
			gas: U256,
			input: Bytes,
			call_type: OldCallType,
		}

		// CallType type in version < 2.7.
		#[allow(dead_code)]
		#[derive(Debug, PartialEq, Clone)]
		enum OldCallType {
			None,
			Call,
			CallCode,
			DelegateCall,
			StaticCall,
		}

		// CallType rlp encoding in version < 2.7.
		impl Encodable for OldCallType {
			fn rlp_append(&self, s: &mut RlpStream) {
				let v = match *self {
					OldCallType::None => 0u32,
					OldCallType::Call => 1,
					OldCallType::CallCode => 2,
					OldCallType::DelegateCall => 3,
					OldCallType::StaticCall => 4,
				};
				Encodable::rlp_append(&v, s);
			}
		}

		let old_call = OldCall {
			from: Address::from_low_u64_be(1),
			to: Address::from_low_u64_be(2),
			value: U256::from(3),
			gas: U256::from(4),
			input: vec![5],
			call_type: OldCallType::DelegateCall,
		};

		let old_encoded = rlp::encode(&old_call);

		let new_call = Call {
			from: Address::from_low_u64_be(1),
			to: Address::from_low_u64_be(2),
			value: U256::from(3),
			gas: U256::from(4),
			input: vec![5],
			call_type: Some(CallType::DelegateCall).into(),
		};

		// `old_call` should be deserialized successfully into `new_call`
		assert_eq!(rlp::decode(&old_encoded), Ok(new_call.clone()));
		// test a roundtrip with `Some` `call_type`
		let new_encoded = rlp::encode(&new_call);
		assert_eq!(rlp::decode(&new_encoded), Ok(new_call));

		// test a roundtrip with `None` `call_type`
		let none_call = Call {
			from: Address::from_low_u64_be(1),
			to: Address::from_low_u64_be(2),
			value: U256::from(3),
			gas: U256::from(4),
			input: vec![5],
			call_type: None.into(),
		};
		let none_encoded = rlp::encode(&none_call);
		assert_eq!(rlp::decode(&none_encoded), Ok(none_call));
	}

	#[test]
	fn test_creation_method_backwards_compatibility() {
		// Create type in version < 2.7.
		#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
		struct OldCreate {
			from: Address,
			value: U256,
			gas: U256,
			init: Bytes,
		}

		let old_create = OldCreate {
			from: Address::from_low_u64_be(1),
			value: U256::from(3),
			gas: U256::from(4),
			init: vec![5],
		};

		let old_encoded = rlp::encode(&old_create);
		let new_create = Create {
			from: Address::from_low_u64_be(1),
			value: U256::from(3),
			gas: U256::from(4),
			init: vec![5],
			creation_method: None,
		};

		// `old_create` should be deserialized successfully into `new_create`
		assert_eq!(rlp::decode(&old_encoded), Ok(new_create.clone()));
		// test a roundtrip with `None` `creation_method`
		let new_encoded = rlp::encode(&new_create);
		assert_eq!(rlp::decode(&new_encoded), Ok(new_create));

		// test a roundtrip with `Some` `creation_method`
		let some_create = Create {
			from: Address::from_low_u64_be(1),
			value: U256::from(3),
			gas: U256::from(4),
			init: vec![5],
			creation_method: Some(CreationMethod::Create2),
		};
		let some_encoded = rlp::encode(&some_create);
		assert_eq!(rlp::decode(&some_encoded), Ok(some_create));
	}
}
