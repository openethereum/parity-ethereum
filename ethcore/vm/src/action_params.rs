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

//! Evm input params.
use util::{Address, Bytes, U256};
use util::hash::{H256};
use util::sha3::{Hashable, SHA3_EMPTY};
use ethjson;

use call_type::CallType;

use std::sync::Arc;

/// Transaction value
#[derive(Clone, Debug)]
pub enum ActionValue {
	/// Value that should be transfered
	Transfer(U256),
	/// Apparent value for transaction (not transfered)
	Apparent(U256)
}

impl ActionValue {
	/// Returns action value as U256.
	pub fn value(&self) -> U256 {
		match *self {
			ActionValue::Transfer(x) | ActionValue::Apparent(x) => x
		}
	}

	/// Returns the transfer action value of the U256-convertable raw value
	pub fn transfer<T: Into<U256>>(transfer_value: T) -> ActionValue {
		ActionValue::Transfer(transfer_value.into())
	}

	/// Returns the apparent action value of the U256-convertable raw value
	pub fn apparent<T: Into<U256>>(apparent_value: T) -> ActionValue {
		ActionValue::Apparent(apparent_value.into())
	}
}

// TODO: should be a trait, possible to avoid cloning everything from a Transaction(/View).
/// Action (call/create) input params. Everything else should be specified in Externalities.
#[derive(Clone, Debug)]
pub struct ActionParams {
	/// Address of currently executed code.
	pub code_address: Address,
	/// Hash of currently executed code.
	pub code_hash: Option<H256>,
	/// Receive address. Usually equal to code_address,
	/// except when called using CALLCODE.
	pub address: Address,
	/// Sender of current part of the transaction.
	pub sender: Address,
	/// Transaction initiator.
	pub origin: Address,
	/// Gas paid up front for transaction execution
	pub gas: U256,
	/// Gas price.
	pub gas_price: U256,
	/// Transaction value.
	pub value: ActionValue,
	/// Code being executed.
	pub code: Option<Arc<Bytes>>,
	/// Input data.
	pub data: Option<Bytes>,
	/// Type of call
	pub call_type: CallType,

}

impl Default for ActionParams {
	/// Returns default ActionParams initialized with zeros
	fn default() -> ActionParams {
		ActionParams {
			code_address: Address::new(),
			code_hash: Some(SHA3_EMPTY),
			address: Address::new(),
			sender: Address::new(),
			origin: Address::new(),
			gas: U256::zero(),
			gas_price: U256::zero(),
			value: ActionValue::Transfer(U256::zero()),
			code: None,
			data: None,
			call_type: CallType::None,
		}
	}
}

impl From<ethjson::vm::Transaction> for ActionParams {
	fn from(t: ethjson::vm::Transaction) -> Self {
		let address: Address = t.address.into();
		ActionParams {
			code_address: Address::new(),
			code_hash: Some((&*t.code).sha3()),
			address: address,
			sender: t.sender.into(),
			origin: t.origin.into(),
			code: Some(Arc::new(t.code.into())),
			data: Some(t.data.into()),
			gas: t.gas.into(),
			gas_price: t.gas_price.into(),
			value: ActionValue::Transfer(t.value.into()),
			call_type: match address.is_zero() { true => CallType::None, false => CallType::Call },	// TODO @debris is this correct?
		}
	}
}
