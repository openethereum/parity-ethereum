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

use crypto::publickey::{KeyPair, Public, Signature, Error as EthKeyError, sign, public_to_address};
use ethereum_types::{H256, Address};
use blockchain::SigningKeyPair;

pub struct PlainNodeKeyPair {
	key_pair: KeyPair,
}

impl PlainNodeKeyPair {
	pub fn new(key_pair: KeyPair) -> Self {
		PlainNodeKeyPair {
			key_pair: key_pair,
		}
	}

	#[cfg(test)]
	pub fn key_pair(&self) -> &KeyPair {
		&self.key_pair
	}
}

impl SigningKeyPair for PlainNodeKeyPair {
	fn public(&self) -> &Public {
		self.key_pair.public()
	}

	fn address(&self) -> Address {
		public_to_address(self.key_pair.public())
	}

	fn sign(&self, data: &H256) -> Result<Signature, EthKeyError> {
		sign(self.key_pair.secret(), data)
	}
}
