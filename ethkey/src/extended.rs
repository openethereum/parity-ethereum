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

		let (derived_key, next_chain_code) = derivation::private(*self.secret, self.chain_code, index);

		let derived_secret = Secret::from_slice(&*derived_key)
				.expect("Derivation always produced a valid private key; qed");

		ExtendedSecret::new(derived_secret, next_chain_code)
	}

	pub fn secret(&self) -> &Secret {
		&self.secret
	}
}

// Derivation functions for private and public keys
// Work is based on BIP0032
// https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
mod derivation {

	use rcrypto::hmac::Hmac;
	use rcrypto::mac::Mac;
	use rcrypto::sha2::Sha512;
	use bigint::hash::{H256, FixedHash};
	use bigint::prelude::{U256, U512, Uint};
	use byteorder::{BigEndian, ByteOrder};
	use secp256k1;

	// private parent key -> private child key
	pub fn private(private_key: H256, chain_code: H256, index: u32) -> (H256, H256) {
		if index < (2 >> 31) {
			private_soft(private_key, chain_code, index)
		}
		else {
			private_hard(private_key, chain_code, index)
		}
	}

	fn private_soft(_private_key: H256, chain_code: H256, _index: u32) -> (H256, H256) {
		(H256::random(), chain_code)
	}

	fn private_hard(private_key: H256, chain_code: H256, index: u32) -> (H256, H256) {
		let mut data = [0u8; 37];
		let private: U256 = private_key.into();
		// 0x00 (padding) -- chain_code --  index
		//  0             --    1..33   -- 33..37
		private.to_big_endian(&mut data[1..33]);
		BigEndian::write_u32(&mut data[33..37], index);

		// produces 512-bytes (I)
		let mut hmac = Hmac::new(Sha512::new(), &*chain_code);
		let mut i_512 = [0u8; 64];
		hmac.input(&data[..]);
		hmac.raw_result(&mut i_512);

		let hmac_key: U256 = H256::from_slice(&i_512[0..32]).into();
		let next_chain_code = H256::from(&i_512[32..64]);
		let child_key = private_add(hmac_key, private).into();

		(child_key, next_chain_code)
	}

	fn private_add(k1: U256, k2: U256) -> U256 {
		let sum = U512::from(k1) + U512::from(k2);

		// N (curve order) of secp256k1
		// todo: maybe lazy static
		let order = H256::from_slice(&secp256k1::constants::CURVE_ORDER);

		modulo(sum, U256::from(order))
	}

	// todo: surely can be optimized
	fn modulo(u1: U512, u2: U256) -> U256 {
		let dv = u1 / U512::from(u2);
		let md = u1 - (dv * U512::from(u2));
		md.into()
	}
}

#[cfg(test)]
mod tests {

	use super::ExtendedSecret;
	use secret::Secret;
	use std::str::FromStr;

	#[test]
	fn smoky() {
		let secret = Secret::from_str("a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65").unwrap();
		let extended_secret = ExtendedSecret::new(secret.clone(), 0u64.into());

		assert_eq!(&**extended_secret.secret(), &*secret);
		assert_eq!(&**extended_secret.derive(0).secret(), &"196d2f31973452e74fb68ba167a5e74cc08bce18491648e543b150deb2217e34".into());
	}
}
