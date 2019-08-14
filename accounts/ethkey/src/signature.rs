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

use std::ops::{Deref, DerefMut};
use std::cmp::PartialEq;
use std::fmt;
use std::str::FromStr;
use std::hash::{Hash, Hasher};
use secp256k1::{Message as SecpMessage, RecoverableSignature, RecoveryId, Error as SecpError};
use secp256k1::key::{SecretKey, PublicKey};
use rustc_hex::{ToHex, FromHex};
use ethereum_types::{H520, H256};
use {Secret, Public, SECP256K1, Error, Message, public_to_address, Address};

/// Signature encoded as RSV components
#[repr(C)]
pub struct Signature([u8; 65]);

impl Signature {
	/// Get a slice into the 'r' portion of the data.
	pub fn r(&self) -> &[u8] {
		&self.0[0..32]
	}

	/// Get a slice into the 's' portion of the data.
	pub fn s(&self) -> &[u8] {
		&self.0[32..64]
	}

	/// Get the recovery byte.
	pub fn v(&self) -> u8 {
		self.0[64]
	}

	/// Encode the signature into RSV array (V altered to be in "Electrum" notation).
	pub fn into_electrum(mut self) -> [u8; 65] {
		self.0[64] += 27;
		self.0
	}

	/// Parse bytes as a signature encoded as RSV (V in "Electrum" notation).
	/// May return empty (invalid) signature if given data has invalid length.
	pub fn from_electrum(data: &[u8]) -> Self {
		if data.len() != 65 || data[64] < 27 {
			// fallback to empty (invalid) signature
			return Signature::default();
		}

		let mut sig = [0u8; 65];
		sig.copy_from_slice(data);
		sig[64] -= 27;
		Signature(sig)
	}

	/// Create a signature object from the sig.
	pub fn from_rsv(r: &H256, s: &H256, v: u8) -> Self {
		let mut sig = [0u8; 65];
		sig[0..32].copy_from_slice(r.as_ref());
		sig[32..64].copy_from_slice(s.as_ref());
		sig[64] = v;
		Signature(sig)
	}

	/// Check if this is a "low" signature.
	pub fn is_low_s(&self) -> bool {
		// "7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0"
		const MASK: H256 = H256([
			0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
			0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
			0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D,
			0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
		]);
		H256::from_slice(self.s()) <= MASK
	}

	/// Check if each component of the signature is in range.
	pub fn is_valid(&self) -> bool {
		// "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141"
		const MASK: H256 = H256([
			0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
			0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b,
			0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
		]);
		const ONE: H256 = H256([
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
		]);
		let r = H256::from_slice(self.r());
		let s = H256::from_slice(self.s());
		self.v() <= 1 &&
			r < MASK && r >= ONE &&
			s < MASK && s >= ONE
	}
}

// manual implementation large arrays don't have trait impls by default.
// remove when integer generics exist
impl PartialEq for Signature {
	fn eq(&self, other: &Self) -> bool {
		&self.0[..] == &other.0[..]
	}
}

// manual implementation required in Rust 1.13+, see `std::cmp::AssertParamIsEq`.
impl Eq for Signature { }

// also manual for the same reason, but the pretty printing might be useful.
impl fmt::Debug for Signature {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		f.debug_struct("Signature")
			.field("r", &self.0[0..32].to_hex())
			.field("s", &self.0[32..64].to_hex())
			.field("v", &self.0[64..65].to_hex())
		.finish()
	}
}

impl fmt::Display for Signature {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", self.to_hex())
	}
}

impl FromStr for Signature {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.from_hex() {
			Ok(ref hex) if hex.len() == 65 => {
				let mut data = [0; 65];
				data.copy_from_slice(&hex[0..65]);
				Ok(Signature(data))
			},
			_ => Err(Error::InvalidSignature)
		}
	}
}

impl Default for Signature {
	fn default() -> Self {
		Signature([0; 65])
	}
}

impl Hash for Signature {
    fn hash<H: Hasher>(&self, state: &mut H) {
    	H520::from(self.0).hash(state);
    }
}

impl Clone for Signature {
    fn clone(&self) -> Self {
		Signature(self.0)
    }
}

impl From<[u8; 65]> for Signature {
	fn from(s: [u8; 65]) -> Self {
		Signature(s)
	}
}

impl Into<[u8; 65]> for Signature {
	fn into(self) -> [u8; 65] {
		self.0
	}
}

impl From<Signature> for H520 {
	fn from(s: Signature) -> Self {
		H520::from(s.0)
	}
}

impl From<H520> for Signature {
	fn from(bytes: H520) -> Self {
		Signature(bytes.into())
	}
}

impl Deref for Signature {
	type Target = [u8; 65];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Signature {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub fn sign(secret: &Secret, message: &Message) -> Result<Signature, Error> {
	let context = &SECP256K1;
	let sec = SecretKey::from_slice(context, secret.as_ref())?;
	let s = context.sign_recoverable(&SecpMessage::from_slice(&message[..])?, &sec)?;
	let (rec_id, data) = s.serialize_compact(context);
	let mut data_arr = [0; 65];

	// no need to check if s is low, it always is
	data_arr[0..64].copy_from_slice(&data[0..64]);
	data_arr[64] = rec_id.to_i32() as u8;
	Ok(Signature(data_arr))
}

pub fn verify_public(public: &Public, signature: &Signature, message: &Message) -> Result<bool, Error> {
	let context = &SECP256K1;
	let rsig = RecoverableSignature::from_compact(context, &signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
	let sig = rsig.to_standard(context);

	let pdata: [u8; 65] = {
		let mut temp = [4u8; 65];
		temp[1..65].copy_from_slice(public.as_bytes());
		temp
	};

	let publ = PublicKey::from_slice(context, &pdata)?;
	match context.verify(&SecpMessage::from_slice(&message[..])?, &sig, &publ) {
		Ok(_) => Ok(true),
		Err(SecpError::IncorrectSignature) => Ok(false),
		Err(x) => Err(Error::from(x))
	}
}

pub fn verify_address(address: &Address, signature: &Signature, message: &Message) -> Result<bool, Error> {
	let public = recover(signature, message)?;
	let recovered_address = public_to_address(&public);
	Ok(address == &recovered_address)
}

pub fn recover(signature: &Signature, message: &Message) -> Result<Public, Error> {
	let context = &SECP256K1;
	let rsig = RecoverableSignature::from_compact(context, &signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
	let pubkey = context.recover(&SecpMessage::from_slice(&message[..])?, &rsig)?;
	let serialized = pubkey.serialize_vec(context, false);

	let mut public = Public::default();
	public.as_bytes_mut().copy_from_slice(&serialized[1..65]);
	Ok(public)
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use {Generator, Random, Message};
	use super::{sign, verify_public, verify_address, recover, Signature};

	#[test]
	fn vrs_conversion() {
		// given
		let keypair = Random.generate().unwrap();
		let message = Message::default();
		let signature = sign(keypair.secret(), &message).unwrap();

		// when
		let vrs = signature.clone().into_electrum();
		let from_vrs = Signature::from_electrum(&vrs);

		// then
		assert_eq!(signature, from_vrs);
	}

	#[test]
	fn signature_to_and_from_str() {
		let keypair = Random.generate().unwrap();
		let message = Message::default();
		let signature = sign(keypair.secret(), &message).unwrap();
		let string = format!("{}", signature);
		let deserialized = Signature::from_str(&string).unwrap();
		assert_eq!(signature, deserialized);
	}

	#[test]
	fn sign_and_recover_public() {
		let keypair = Random.generate().unwrap();
		let message = Message::default();
		let signature = sign(keypair.secret(), &message).unwrap();
		assert_eq!(keypair.public(), &recover(&signature, &message).unwrap());
	}

	#[test]
	fn sign_and_verify_public() {
		let keypair = Random.generate().unwrap();
		let message = Message::default();
		let signature = sign(keypair.secret(), &message).unwrap();
		assert!(verify_public(keypair.public(), &signature, &message).unwrap());
	}

	#[test]
	fn sign_and_verify_address() {
		let keypair = Random.generate().unwrap();
		let message = Message::default();
		let signature = sign(keypair.secret(), &message).unwrap();
		assert!(verify_address(&keypair.address(), &signature, &message).unwrap());
	}
}
