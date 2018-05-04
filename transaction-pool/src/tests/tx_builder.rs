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

use super::{Transaction, U256, Address};

#[derive(Debug, Default, Clone)]
pub struct TransactionBuilder {
	nonce: U256,
	gas_price: U256,
	gas: U256,
	sender: Address,
	mem_usage: usize,
}

impl TransactionBuilder {
	pub fn tx(&self) -> Self {
		self.clone()
	}

	pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
		self.nonce = nonce.into();
		self
	}

	pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
		self.gas_price = gas_price.into();
		self
	}

	pub fn sender<T: Into<Address>>(mut self, sender: T) -> Self {
		self.sender = sender.into();
		self
	}

	pub fn mem_usage(mut self, mem_usage: usize) -> Self {
		self.mem_usage = mem_usage;
		self
	}

	pub fn new(self) -> Transaction {
		let hash = self.nonce ^ (U256::from(100) * self.gas_price) ^ (U256::from(100_000) * U256::from(self.sender.low_u64()));
		Transaction {
			hash: hash.into(),
			nonce: self.nonce,
			gas_price: self.gas_price,
			gas: 21_000.into(),
			sender: self.sender,
			mem_usage: self.mem_usage,
		}
	}
}
