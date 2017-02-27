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

use super::{SECP256K1, Public, Secret, Error};
use secp256k1::key;

/// Inplace multiply public key by secret key (EC point * scalar)
pub fn public_mul_secret(public: &mut Public, secret: &Secret) -> Result<(), Error> {
	let key_secret = key::SecretKey::from_slice(&SECP256K1, &secret[..])?;
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

/// Inplace add one secret key to another (scalar + scalar)
pub fn secret_add(secret: &mut Secret, other: &Secret) -> Result<(), Error> {
	let mut key_secret = to_secp256k1_secret(secret)?;
	let other_secret = to_secp256k1_secret(other)?;
	key_secret.add_assign(&SECP256K1, &other_secret)?;

	*secret = key_secret.into();
	Ok(())
}

/// Inplace subtract one secret key from another (scalar - scalar)
pub fn secret_sub(secret: &mut Secret, other: &Secret) -> Result<(), Error> {
	let mut key_secret = to_secp256k1_secret(secret)?;
	let mut other_secret = to_secp256k1_secret(other)?;
	other_secret.mul_assign(&SECP256K1, &key::MINUS_ONE_KEY)?;
	key_secret.add_assign(&SECP256K1, &other_secret)?;

	*secret = key_secret.into();
	Ok(())
}

/// Inplace multiply one secret key to another (scalar * scalar)
pub fn secret_mul(secret: &mut Secret, other: &Secret) -> Result<(), Error> {
	let mut key_secret = to_secp256k1_secret(secret)?;
	let other_secret = to_secp256k1_secret(other)?;
	key_secret.mul_assign(&SECP256K1, &other_secret)?;

	*secret = key_secret.into();
	Ok(())
}

/// Inplace inverse secret key (1 / scalar)
pub fn secret_inv(secret: &mut Secret) -> Result<(), Error> {
	let mut key_secret = to_secp256k1_secret(secret)?;
	key_secret.inv_assign(&SECP256K1)?;

	*secret = key_secret.into();
	Ok(())
}

/// Compute power of secret key inplace (secret ^ pow).
/// This function is not intended to be used with large powers.
pub fn secret_pow(secret: &mut Secret, pow: usize) -> Result<(), Error> {
	if pow == 0 {
		*secret = key::ONE_KEY.into();
		return Ok(());
	}
	if pow == 1 {
		return Ok(());
	}

	let secret_copy = secret.clone();
	for _ in 1..pow {
		secret_mul(secret, &secret_copy)?;
	}

	Ok(())
}

/// Return base point of secp256k1
pub fn generation_point() -> Public {
	let key_public = key::PublicKey::from_secret_key(&SECP256K1, &key::ONE_KEY).expect("predefined");
	let mut public = Public::default();
	set_public(&mut public, &key_public);
	public
}

fn to_secp256k1_public(public: &Public) -> Result<key::PublicKey, Error> {
	let public_data = {
		let mut temp = [4u8; 65];
		(&mut temp[1..65]).copy_from_slice(&public[0..64]);
		temp
	};

	Ok(key::PublicKey::from_slice(&SECP256K1, &public_data)?)
}

fn to_secp256k1_secret(secret: &Secret) -> Result<key::SecretKey, Error> {
	Ok(key::SecretKey::from_slice(&SECP256K1, &secret[..])?)
}

fn set_public(public: &mut Public, key_public: &key::PublicKey) {
	let key_public_serialized = key_public.serialize_vec(&SECP256K1, false);
	public.copy_from_slice(&key_public_serialized[1..65]);
}
