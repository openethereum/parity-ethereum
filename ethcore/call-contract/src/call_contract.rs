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

use bytes::Bytes;
use ethereum_types::Address;
use ethabi::{decode, ParamType, Token};
use types::ids::BlockId;

/// Provides `call_contract` method
pub trait CallContract {
	/// Like `call`, but with various defaults. Designed to be used for calling contracts.
	/// Returns an error in case of vm exception. Tries to decode Solidity revert message if there is a revert exception.
	fn call_contract(&self, id: BlockId, address: Address, data: Bytes) -> Result<Bytes, String>;

	/// Like `call`, but with various defaults. Designed to be used for calling contracts with specified sender.
	/// Returns an error in case of vm exception. Tries to decode Solidity revert message if there is a revert exception.
	fn call_contract_from_sender(&self, block_id: BlockId, address: Address, sender: Address, data: Bytes) -> Result<Bytes, String>;

	/// Try to decode Solidity revert string
	fn try_decode_solidity_revert_msg(&self, data: &Bytes) -> Option<String> {
		let mut result = None;
		if data.len() > 4 {
			let (error_selector, enc_string) = data.split_at(4);
			// Error(string) selector. Details: https://solidity.readthedocs.io/en/v0.5.8/control-structures.html#error-handling-assert-require-revert-and-exceptions
			if error_selector == [0x08, 0xc3, 0x79, 0xa0] {
				result = decode(&[ParamType::String], enc_string)
					.as_ref()
					.map(|d| if let Token::String(str) = &d[0] { Some(str.as_str().to_string()) } else { None })
					.unwrap_or_else(|_| None);
			}
		}
		result
	}
}

/// Provides information on a blockchain service and it's registry
pub trait RegistryInfo {
	/// Get the address of a particular blockchain service, if available.
	fn registry_address(&self, name: String, block: BlockId) -> Option<Address>;
}
