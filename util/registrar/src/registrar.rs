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

use call_contract::CallContract;
use ethabi::Address;
use keccak_hash::keccak;
use types::ids::BlockId;

use_contract!(registrar, "res/registrar.json");

// Maps a domain name to an Ethereum address
const DNS_A_RECORD: &'static str = "A";

/// Registrar contract interface
pub trait RegistrarClient: CallContract + Send + Sync {
	/// Get address of the registrar itself
	fn registrar_address(&self) -> Option<Address>;

	/// Get address from registrar for the specified key.
	fn get_address(&self, key: &str, block: BlockId) -> Result<Option<Address>, String> {
		use registrar::registrar::functions::get_address::{encode_input, decode_output};

		let registrar_address = match self.registrar_address() {
			Some(address) => address,
			None => return Err("Registrar address not defined.".to_owned())
		};

		let hashed_key: [u8; 32] = keccak(key).into();
		let id = encode_input(hashed_key, DNS_A_RECORD);

		let address_bytes = self.call_contract(block, registrar_address, id)?;

		let address = decode_output(&address_bytes).map_err(|e| e.to_string())?;

		if address.is_zero() {
			Ok(None)
		} else {
			Ok(Some(address))
		}
	}
}
