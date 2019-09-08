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

//! Provides CallContract and RegistryInfo traits

use std::fmt;
use bytes::Bytes;
use derive_builder::Builder;
use ethereum_types::{Address, U256};
use types::ids::BlockId;

#[derive(Debug)]
/// Transaction call error
pub enum CallError {
	/// Call reverted
	Reverted(String),
	/// Other error
	Other(String),
}

impl fmt::Display for CallError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

#[derive(Builder, Default, Clone)]
#[builder(default)]
/// Options to make a call to contract
pub struct CallOptions {
	/// Contract address
	pub contract_address: Address,
	/// Transaction sender
	pub sender: Address,
	/// Transaction data
	pub data: Bytes,
	/// Value in wei
	pub value: U256,
	/// Provided gas
	#[builder(default = "self.default_gas()")]
	pub gas: U256,
	/// Gas price
	pub gas_price: U256,
}

impl CallOptions {
	/// Convenience method for creating the most common use case.
	pub fn new(contract: Address, data: Bytes) -> Self {
		CallOptionsBuilder::default().contract_address(contract).data(data).build().unwrap()
	}
}

impl CallOptionsBuilder {
    fn default_gas(&self) -> U256 {
        U256::from(50_000_000)
    }
}

/// Provides `call_contract` method
pub trait CallContract {
	/// Executes a transient call to a contract at given `BlockId`.
	///
	/// The constructed transaction must be executed on top of state at block `id`,
	/// any changes introduce by the call must be discarded.
	/// Returns:
	/// - A return data from the contract if the call was successful,
	/// - A `CallError::Reverted(msg)` error in case the call was reverted with an exception and message.
	/// - A `CallError::Other(msg)` in case the call did not succeed for other reasons.
	fn call_contract(&self, id: BlockId, call_options: CallOptions) -> Result<Bytes, CallError>;
}

/// Provides information on a blockchain service and it's registry
pub trait RegistryInfo {
	/// Get the address of a particular blockchain service, if available.
	fn registry_address(&self, name: String, block: BlockId) -> Option<Address>;
}
