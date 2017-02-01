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
use Public;
use bigint::hash::{H256, FixedHash};
pub use self::derivation::Error as DerivationError;

/// Extended secret key, allows deterministic derivation of subsequent keys.
pub struct ExtendedSecret {
	secret: Secret,
	chain_code: H256,
}

impl ExtendedSecret {
	/// New extended key from given secret and chain code.
	pub fn with_code(secret: Secret, chain_code: H256) -> ExtendedSecret {
		ExtendedSecret {
			secret: secret,
			chain_code: chain_code,
		}
	}

	/// New extended key from given secret with the random chain code.
	pub fn new_random(secret: Secret) -> ExtendedSecret {
		ExtendedSecret::with_code(secret, H256::random())
	}

	/// New extended key from given secret.
	/// Chain code will be derived from this secret (in a deterministic way).
	pub fn new(secret: Secret) -> ExtendedSecret {
		let chain_code = derivation::chain_code(*secret);
		ExtendedSecret::with_code(secret, chain_code)
	}

	/// Derive new private key
	pub fn derive(&self, index: u32) -> ExtendedSecret {
		let (derived_key, next_chain_code) = derivation::private(*self.secret, self.chain_code, index);

		let derived_secret = Secret::from_slice(&*derived_key)
			.expect("Derivation always produced a valid private key; qed");

		ExtendedSecret::with_code(derived_secret, next_chain_code)
	}

	/// Private key component of the extended key.
	pub fn secret(&self) -> &Secret {
		&self.secret
	}
}

/// Extended public key, allows deterministic derivation of subsequent keys.
pub struct ExtendedPublic {
	public: Public,
	chain_code: H256,
}

impl ExtendedPublic {
	/// New extended public key from known parent and chain code
	pub fn new(public: Public, chain_code: H256) -> Self {
		ExtendedPublic { public: public, chain_code: chain_code }
	}

	/// Create new extended public key from known secret
	pub fn from_secret(secret: &ExtendedSecret) -> Result<Self, DerivationError> {
		Ok(
			ExtendedPublic::new(
				derivation::point(**secret.secret())?,
				secret.chain_code.clone(),
			)
		)
	}

	/// Derive new public key
	/// Operation is defined only for index belongs [0..2^31)
	pub fn derive(self, index: u32) -> Result<Self, DerivationError> {
		let (derived_key, next_chain_code) = derivation::public(self.public, self.chain_code, index)?;
		Ok(ExtendedPublic::new(derived_key, next_chain_code))
	}

	pub fn public(&self) -> &Public {
		&self.public
	}
}

// Derivation functions for private and public keys
// Work is based on BIP0032
// https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
mod derivation {

	use rcrypto::hmac::Hmac;
	use rcrypto::mac::Mac;
	use rcrypto::sha2::Sha512;
	use bigint::hash::{H512, H256, FixedHash};
	use bigint::prelude::{U256, U512, Uint};
	use byteorder::{BigEndian, ByteOrder};
	use secp256k1;
	use secp256k1::key::{SecretKey, PublicKey};
	use SECP256K1;
	use keccak;

	#[derive(Debug)]
	pub enum Error {
		InvalidHardenedUse,
		InvalidPoint,
		MissingIndex,
	}

	// Deterministic derivation of the key using elliptic curve.
	// Derivation can be either hardened or not.
	// For hardened derivation, pass index at least 2^31
	pub fn private(private_key: H256, chain_code: H256, index: u32) -> (H256, H256) {
		if index < (2 << 30) {
			private_soft(private_key, chain_code, index)
		}
		else {
			private_hard(private_key, chain_code, index)
		}
	}

	fn hmac_pair(data: [u8; 37], private_key: H256, chain_code: H256) -> (H256, H256) {
		let private: U256 = private_key.into();

		// produces 512-bit derived hmac (I)
		let mut hmac = Hmac::new(Sha512::new(), &*chain_code);
		let mut i_512 = [0u8; 64];
		hmac.input(&data[..]);
		hmac.raw_result(&mut i_512);

		// left most 256 bits are later added to original private key
		let hmac_key: U256 = H256::from_slice(&i_512[0..32]).into();
		// right most 256 bits are new chain code for later derivations
		let next_chain_code = H256::from(&i_512[32..64]);

		let child_key = private_add(hmac_key, private).into();
		(child_key, next_chain_code)
	}

	fn private_soft(private_key: H256, chain_code: H256, index: u32) -> (H256, H256) {
		let mut data = [0u8; 37];

		let sec_private = SecretKey::from_slice(&SECP256K1, &*private_key)
			.expect("Caller should provide valid private key");
		let sec_public = PublicKey::from_secret_key(&SECP256K1, &sec_private)
			.expect("Caller should provide valid private key");
		let public_serialized = sec_public.serialize_vec(&SECP256K1, true);

		// curve point (compressed public key) --  index
		//             0.33                    --  33..37
		data[0..33].copy_from_slice(&public_serialized);
		BigEndian::write_u32(&mut data[33..37], index);

		hmac_pair(data, private_key, chain_code)
	}

	// Deterministic derivation of the key using elliptic curve
	// This is hardened derivation and does not allow to associate
	// corresponding public keys of the original and derived private keys
	fn private_hard(private_key: H256, chain_code: H256, index: u32) -> (H256, H256) {
		let mut data = [0u8; 37];
		let private: U256 = private_key.into();

		// 0x00 (padding) -- private_key --  index
		//  0             --    1..33    -- 33..37
		private.to_big_endian(&mut data[1..33]);
		BigEndian::write_u32(&mut data[33..37], index);

		hmac_pair(data, private_key, chain_code)
	}

	fn private_add(k1: U256, k2: U256) -> U256 {
		let sum = U512::from(k1) + U512::from(k2);
		modulo(sum, curve_n())
	}

	// todo: surely can be optimized
	fn modulo(u1: U512, u2: U256) -> U256 {
		let dv = u1 / U512::from(u2);
		let md = u1 - (dv * U512::from(u2));
		md.into()
	}

	// returns n (for mod(n)) for the secp256k1 elliptic curve
	// todo: maybe lazy static
	fn curve_n() -> U256 {
		H256::from_slice(&secp256k1::constants::CURVE_ORDER).into()
	}

	pub fn public(public_key: H512, chain_code: H256, index: u32) -> Result<(H512, H256), Error> {
		if index >= (2 << 30) {
			// public derivation is only defined on 'soft' index space [0..2^31)
			return Err(Error::InvalidHardenedUse)
		}

		let mut public_sec_raw = [0u8; 65];
		public_sec_raw[0] = 4;
		public_sec_raw[1..65].copy_from_slice(&*public_key);
		let public_sec = PublicKey::from_slice(&SECP256K1, &public_sec_raw).map_err(|_| Error::InvalidPoint)?;
		let public_serialized = public_sec.serialize_vec(&SECP256K1, true);

		let mut data = [0u8; 37];
		// curve point (compressed public key) --  index
		//             0.33                    --  33..37
		data[0..33].copy_from_slice(&public_serialized);
		BigEndian::write_u32(&mut data[33..37], index);

		// HMAC512SHA produces [derived private(256); new chain code(256)]
		let mut hmac = Hmac::new(Sha512::new(), &*chain_code);
		let mut i_512 = [0u8; 64];
		hmac.input(&data[..]);
		hmac.raw_result(&mut i_512);

		let new_private = H256::from(&i_512[0..32]);
		let new_chain_code = H256::from(&i_512[32..64]);

		// Generated private key can (extremely rarely) be out of secp256k1 key field
		if curve_n() <= new_private.clone().into() { return Err(Error::MissingIndex); }
		let new_private_sec = SecretKey::from_slice(&SECP256K1, &*new_private)
			.expect("Private key belongs to the field [0..CURVE_ORDER) (checked above); So initializing can never fail; qed");
		let mut new_public = PublicKey::from_secret_key(&SECP256K1, &new_private_sec)
			.expect("Valid private key produces valid public key");

		// Adding two points on the elliptic curves (combining two public keys)
		new_public.add_assign(&SECP256K1, &public_sec)
			.expect("Addition of two valid points produce valid point");

		let serialized = new_public.serialize_vec(&SECP256K1, false);

		Ok((
			H512::from(&serialized[1..65]),
			new_chain_code,
		))
	}

	fn sha3(slc: &[u8]) -> H256 {
		keccak::Keccak256::keccak256(slc).into()
	}

	pub fn chain_code(secret: H256) -> H256 {
		// 10,000 rounds of sha3
		let mut running_sha3 = sha3(&*secret);
		for _ in 0..99999 { running_sha3 = sha3(&*running_sha3); }
		running_sha3
	}

	pub fn point(secret: H256) -> Result<H512, Error> {
		let sec = SecretKey::from_slice(&SECP256K1, &*secret)
			.map_err(|_| Error::InvalidPoint)?;
		let public_sec = PublicKey::from_secret_key(&SECP256K1, &sec)
			.map_err(|_| Error::InvalidPoint)?;
		let serialized = public_sec.serialize_vec(&SECP256K1, false);
		Ok(H512::from(&serialized[1..65]))
	}
}

#[cfg(test)]
mod tests {

	use super::{ExtendedSecret, ExtendedPublic};
	use secret::Secret;
	use std::str::FromStr;

	#[test]
	fn smoky() {
		let secret = Secret::from_str("a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65").unwrap();
		let extended_secret = ExtendedSecret::with_code(secret.clone(), 0u64.into());

		// hardened
		assert_eq!(&**extended_secret.secret(), &*secret);
		assert_eq!(&**extended_secret.derive(2147483648).secret(), &"0927453daed47839608e414a3738dfad10aed17c459bbd9ab53f89b026c834b6".into());
		assert_eq!(&**extended_secret.derive(2147483649).secret(), &"44238b6a29c6dcbe9b401364141ba11e2198c289a5fed243a1c11af35c19dc0f".into());

		// normal
		assert_eq!(&**extended_secret.derive(0).secret(), &"bf6a74e3f7b36fc4c96a1e12f31abc817f9f5904f5a8fc27713163d1f0b713f6".into());
		assert_eq!(&**extended_secret.derive(1).secret(), &"bd4fca9eb1f9c201e9448c1eecd66e302d68d4d313ce895b8c134f512205c1bc".into());
		assert_eq!(&**extended_secret.derive(2).secret(), &"86932b542d6cab4d9c65490c7ef502d89ecc0e2a5f4852157649e3251e2a3268".into());

		let extended_public = ExtendedPublic::from_secret(&extended_secret).expect("Extended public should be created");
		let derived_public = extended_public.derive(0).expect("First derivation of public should succeed");
		assert_eq!(&*derived_public.public(), &"f7b3244c96688f92372bfd4def26dc4151529747bab9f188a4ad34e141d47bd66522ff048bc6f19a0a4429b04318b1a8796c000265b4fa200dae5f6dda92dd94".into());
	}

	#[test]
	fn match_() {
		let secret = Secret::from_str("a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65").unwrap();
		let extended_secret = ExtendedSecret::with_code(secret.clone(), 0u64.into());
		let extended_public = ExtendedPublic::from_secret(&extended_secret).expect("Extended public should be created");

		let derived_secret0 = extended_secret.derive(0);
		let derived_public0 = extended_public.derive(0).expect("First derivation of public should succeed");

		let public_from_secret0 = ExtendedPublic::from_secret(&derived_secret0).expect("Extended public should be created");

		assert_eq!(public_from_secret0.public(), derived_public0.public());
	}
}
