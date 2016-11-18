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
use util::misc::code_hash;
use util::Address;
use client::operations::Operations;
use client::client::Client;

pub struct Updater {
	operations: Operations,
}

impl Updater {
	pub fn new(client: Weak<Client>, operations: Address) -> Self {
		Updater {
			operations: Operations::new(operations, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))),
		}
	}

	pub fn tick(&mut self) {
		match self.operations.is_latest("par", &code_hash().into()) {
			Ok(res) => {
				info!("isLatest returned {}", res);
			},
			Err(e) => {
				warn!(target: "dapps", "Error while calling Operations.isLatest: {:?}", e);
			}
		}
	}
}
