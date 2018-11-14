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

use ethereum_types::{U256, Address};
use bytes::Bytes;

use v1::types::{Origin, TransactionCondition};
use ethereum_types::H256;

/// Transaction request coming from RPC
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct TransactionRequest {
	/// Sender
	pub from: Option<Address>,
	/// Recipient
	pub to: Option<Address>,
	/// Gas Price
	pub gas_price: Option<U256>,
	/// Gas
	pub gas: Option<U256>,
	/// Value of transaction in wei
	pub value: Option<U256>,
	/// Additional data sent with transaction
	pub data: Option<Bytes>,
	/// Transaction's nonce
	pub nonce: Option<U256>,
	/// Delay until this condition is met.
	pub condition: Option<TransactionCondition>,
}

/// Transaction request coming from RPC with default values filled in.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct FilledTransactionRequest {
	/// Sender
	pub from: Address,
	/// Indicates if the sender was filled by default value.
	pub used_default_from: bool,
	/// Recipient
	pub to: Option<Address>,
	/// Gas Price
	pub gas_price: U256,
	/// Gas
	pub gas: U256,
	/// Value of transaction in wei
	pub value: U256,
	/// Additional data sent with transaction
	pub data: Bytes,
	/// Transaction's nonce
	pub nonce: Option<U256>,
	/// Delay until this condition is met.
	pub condition: Option<TransactionCondition>,
}

impl From<FilledTransactionRequest> for TransactionRequest {
	fn from(r: FilledTransactionRequest) -> Self {
		TransactionRequest {
			from: Some(r.from),
			to: r.to,
			gas_price: Some(r.gas_price),
			gas: Some(r.gas),
			value: Some(r.value),
			data: Some(r.data),
			nonce: r.nonce,
			condition: r.condition,
		}
	}
}

/// Call request
#[derive(Debug, Default, PartialEq)]
pub struct CallRequest {
	/// From
	pub from: Option<Address>,
	/// To
	pub to: Option<Address>,
	/// Gas Price
	pub gas_price: Option<U256>,
	/// Gas
	pub gas: Option<U256>,
	/// Value
	pub value: Option<U256>,
	/// Data
	pub data: Option<Vec<u8>>,
	/// Nonce
	pub nonce: Option<U256>,
}

/// Confirmation object
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConfirmationRequest {
	/// Id of this confirmation
	pub id: U256,
	/// Payload to confirm
	pub payload: ConfirmationPayload,
	/// Request origin
	pub origin: Origin,
}

/// Payload to confirm in Trusted Signer
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ConfirmationPayload {
	/// Transaction
	SendTransaction(FilledTransactionRequest),
	/// Sign Transaction
	SignTransaction(FilledTransactionRequest),
	/// Sign a message with an Ethereum specific security prefix.
	EthSignMessage(Address, Bytes),
	/// Sign a message
	SignMessage(Address, H256),
	/// Decrypt request
	Decrypt(Address, Bytes),
}

impl ConfirmationPayload {
	pub fn sender(&self) -> Address {
		match *self {
			ConfirmationPayload::SendTransaction(ref request) => request.from,
			ConfirmationPayload::SignTransaction(ref request) => request.from,
			ConfirmationPayload::EthSignMessage(ref address, _) => *address,
			ConfirmationPayload::SignMessage(ref address, _) => *address,
			ConfirmationPayload::Decrypt(ref address, _) => *address,
		}
	}
}
