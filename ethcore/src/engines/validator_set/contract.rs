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

/// Preconfigured validator list.

use util::*;
use client::chain_notify::ChainNotify;
use super::{ValidatorSet, SimpleList};

#[derive(Debug)]
pub struct ValidatorContract {
	address: Address,
	validators: RwLock<SimpleList>,
}

impl ValidatorContract {
	pub fn new(contract_address: Address) -> Self {
		ValidatorContract {
			address: contract_address,
			validators: Default::default(),
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
		_duration: u64)
	{
		//self.client.is_major_importing()
	}
}

impl ValidatorSet for ValidatorContract {
	fn contains(&self, address: &Address) -> bool {
		self.validators.read().contains(address)
	}

	fn get(&self, nonce: usize) -> Address {
		self.validators.read().get(nonce).clone()
	}
}
