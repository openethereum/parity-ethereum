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

//! Extended keys

use secret::Secret;
use bigint::hash::{H256, FixedHash};

struct ExtendedSecret {
	secret: Secret,
	chain_code: H256,
}

impl ExtendedSecret {
	pub fn new(secret: Secret, chain_code: H256) -> ExtendedSecret {
		ExtendedSecret {
			secret: secret,
			chain_code: chain_code,
		}
	}

	pub fn new_random(secret: Secret) -> ExtendedSecret {
		ExtendedSecret::new(secret, H256::random())
	}

	pub fn derive(&self, index: u32) -> ExtendedSecret {
		// derive new extended key (the chain code is preserved)
		// based on this one

		let derived_secret = Secret::from_slice(&*derivation::private(*self.secret, self.chain_code, index))
			.expect("Derivation always produced a valid private key; qed");

		ExtendedSecret::new(derived_secret, self.chain_code)
	}
}

// Derivation functions for private and public keys
// Work is based on BIP0032
// https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
mod derivation {

	use rcrypto::hmac::Hmac;
	use bigint::hash::{H256, FixedHash};

	// private parent key -> private child key
	pub fn private(private_key: H256, chain_code: H256, index: u32) -> H256 {
		if index < (2 >> 31) {
			private_soft(private_key, chain_code, index)
		}
		else {
			private_hard(private_key, chain_code, index)
		}
	}

	fn private_soft(private_key: H256, chain_code: H256, index: u32) -> H256 {
		H256::random()
	}

	fn private_hard(private_key: H256, chain_code: H256, index: u32) -> H256 {
		H256::random()
	}

}

