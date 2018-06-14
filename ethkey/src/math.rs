// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use super::{SECP256K1, Public, Secret, Error};
use secp256k1::key;
use secp256k1::constants::{GENERATOR_X, GENERATOR_Y, CURVE_ORDER};
use ethereum_types::{U256, H256};

/// Whether the public key is valid.
pub fn public_is_valid(public: &Public) -> bool {
	to_secp256k1_public(public).ok()
		.map_or(false, |p| p.is_valid())
}

/// Inplace multiply public key by secret key (EC point * scalar)
pub fn public_mul_secret(public: &mut Public, secret: &Secret) -> Result<(), Error> {
	let key_secret = secret.to_secp256k1_secret()?;
	let mut key_public = to_secp256k1_public(public)?;
	key_public.mul_assign(&SECP256K1, &key_secret)?;
	set_public(public, &key_public);
	Ok(())
}

/// Inplace add one public key to another (EC point + EC point)
pub fn public_add(public: &mut Public, other: &Public) -> Result<(), Error> {
	let mut key_public = to_secp256k1_public(public)?;
	let other_public = to_secp256k1_public(other)?;
	key_public.add_assign(&SECP256K1, &other_public)?;
	set_public(public, &key_public);
	Ok(())
}

/// Inplace sub one public key from another (EC point - EC point)
pub fn public_sub(public: &mut Public, other: &Public) -> Result<(), Error> {
	let mut key_neg_other = to_secp256k1_public(other)?;
	key_neg_other.mul_assign(&SECP256K1, &key::MINUS_ONE_KEY)?;

	let mut key_public = to_secp256k1_public(public)?;
	key_public.add_assign(&SECP256K1, &key_neg_other)?;
	set_public(public, &key_public);
	Ok(())
}

/// Replace public key with its negation (EC point = - EC point)
pub fn public_negate(public: &mut Public) -> Result<(), Error> {
	let mut key_public = to_secp256k1_public(public)?;
	key_public.mul_assign(&SECP256K1, &key::MINUS_ONE_KEY)?;
	set_public(public, &key_public);
	Ok(())
}

/// Return base point of secp256k1
pub fn generation_point() -> Public {
	let mut public_sec_raw = [0u8; 65];
	public_sec_raw[0] = 4;
	public_sec_raw[1..33].copy_from_slice(&GENERATOR_X);
	public_sec_raw[33..65].copy_from_slice(&GENERATOR_Y);

	let public_key = key::PublicKey::from_slice(&SECP256K1, &public_sec_raw)
		.expect("constructing using predefined constants; qed");
	let mut public = Public::default();
	set_public(&mut public, &public_key);
	public
}

/// Return secp256k1 elliptic curve order
pub fn curve_order() -> U256 {
	H256::from_slice(&CURVE_ORDER).into()
}

fn to_secp256k1_public(public: &Public) -> Result<key::PublicKey, Error> {
	let public_data = {
		let mut temp = [4u8; 65];
		(&mut temp[1..65]).copy_from_slice(&public[0..64]);
		temp
	};

	Ok(key::PublicKey::from_slice(&SECP256K1, &public_data)?)
}

fn set_public(public: &mut Public, key_public: &key::PublicKey) {
	let key_public_serialized = key_public.serialize_vec(&SECP256K1, false);
	public.copy_from_slice(&key_public_serialized[1..65]);
}

#[cfg(test)]
mod tests {
	use super::super::{Random, Generator};
	use super::{public_add, public_sub};

	#[test]
	fn public_addition_is_commutative() {
		let public1 = Random.generate().unwrap().public().clone();
		let public2 = Random.generate().unwrap().public().clone();

		let mut left = public1.clone();
		public_add(&mut left, &public2).unwrap();

		let mut right = public2.clone();
		public_add(&mut right, &public1).unwrap();

		assert_eq!(left, right);
	}

	#[test]
	fn public_addition_is_reversible_with_subtraction() {
		let public1 = Random.generate().unwrap().public().clone();
		let public2 = Random.generate().unwrap().public().clone();

		let mut sum = public1.clone();
		public_add(&mut sum, &public2).unwrap();
		public_sub(&mut sum, &public2).unwrap();

		assert_eq!(sum, public1);
	}
}
