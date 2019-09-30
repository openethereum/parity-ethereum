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

use futures::{Future, future, IntoFuture};
use call_contract::CallContract;
use ethabi::{Address, Bytes};
use std::sync::Arc;
use keccak_hash::keccak;
use types::ids::BlockId;

use_contract!(registrar, "res/registrar.json");

// Maps a domain name to an Ethereum address
const DNS_A_RECORD: &'static str = "A";

/// Registrar contract interface
/// Should execute transaction using current blockchain state.
pub trait RegistrarClient: CallContract + Send + Sync {
	/// Get registrar address
	fn registrar_address(&self) -> Result<Address, String>;

	fn get_address<'a>(&self, key: &'a str, block: BlockId) -> Box<dyn Future<Item = Address, Error = String> + Send> {
		// Address of the registrar itself
		let registrar_address = match self.registrar_address() {
			Ok(a) => a,
			Err(e) => return Box::new(future::err(e)),
		};

		let hashed_key: [u8; 32] = keccak(key).into();
		let id = registrar::functions::get_address::encode_input(hashed_key, DNS_A_RECORD);

		let future = self.call_contract(block, registrar_address, id)
			.into_future()
			.and_then(move |address| registrar::functions::get_address::decode_output(&address).map_err(|e| e.to_string()));

		Box::new(future)
	}
}
