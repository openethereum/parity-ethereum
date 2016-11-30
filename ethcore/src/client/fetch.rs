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

use std::sync::Weak;
use std::str::FromStr;
use util::{Bytes, Address};

use client::{Client, BlockChainClient};
use fetch;

/// Client wrapper implementing `fetch::urlhint::ContractClient`
pub struct FetchHandler {
	client: Weak<Client>,
}

impl FetchHandler {
	/// Creates new wrapper
	pub fn new(client: Weak<Client>) -> Self {
		FetchHandler { client: client }
	}
}

impl fetch::urlhint::ContractClient for FetchHandler {
	fn registrar(&self) -> Result<Address, String> {
		self.client.upgrade().ok_or_else(|| "Client not available".to_owned())?
			.additional_params()
			.get("registrar")
			.and_then(|s| Address::from_str(s).ok())
			.ok_or_else(|| "Registrar not available".into())
	}

	fn call(&self, address: Address, data: Bytes) -> Result<Bytes, String> {
		self.client.upgrade().ok_or_else(|| "Client not available".to_owned())?
			.call_contract(address, data)
	}
}
