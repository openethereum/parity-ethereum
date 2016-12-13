// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

/// Validator set maintained in a contract.

use std::sync::Weak;
use util::*;
use client::{Client, BlockChainClient};
use client::chain_notify::ChainNotify;
use super::ValidatorSet;
use super::simple_list::SimpleList;

pub struct ValidatorContract {
	address: Address,
	validators: RwLock<SimpleList>,
	client: RwLock<Weak<Client>>,
}

impl ValidatorContract {
	pub fn new(contract_address: Address) -> Self {
		ValidatorContract {
			address: contract_address,
			validators: Default::default(),
			client: RwLock::new(Weak::new()),
		}
	}
}

impl ChainNotify for ValidatorContract {
	fn new_blocks(
		&self,
		imported: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_duration: u64) {
		if let Some(client) = self.client.read().upgrade() {

			// We rely on a secure state. Bail if we're unsure about it.
			if !client.chain_info().security_level().is_full() {
				return;
			}

			
		}
	}
}

impl ValidatorSet for ValidatorContract {
	fn contains(&self, address: &Address) -> bool {
		self.validators.read().contains(address)
	}

	fn get(&self, nonce: usize) -> Address {
		self.validators.read().get(nonce).clone()
	}

	fn register_client(&self, client: Weak<Client>) {
		let mut guard = self.client.write();
		guard.clone_from(&client);
	}
}
