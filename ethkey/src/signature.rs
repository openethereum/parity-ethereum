use std::ops::{Deref, DerefMut};
use std::{mem, fmt};
use std::str::FromStr;
use secp256k1::{Message as SecpMessage, RecoverableSignature, RecoveryId, Error as SecpError};
use secp256k1::key::{SecretKey, PublicKey};
use rustc_serialize::hex::{ToHex, FromHex};
use {Secret, Public, SECP256K1, Error, Message};

#[repr(C)]
#[derive(Eq)]
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
}

// manual implementation large arrays don't have trait impls by default.
// remove when integer generics exist
impl ::std::cmp::PartialEq for Signature {
	fn eq(&self, other: &Self) -> bool {
		&self.0[..] == &other.0[..]
	}
}

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
	// no way to create from raw byte array.
	let sec: &SecretKey = unsafe { mem::transmute(secret) };
	let s = try!(context.sign_recoverable(&try!(SecpMessage::from_slice(&message[..])), sec));
	let (rec_id, data) = s.serialize_compact(context);
	let mut data_arr = [0; 65];

	// no need to check if s is low, it always is
	data_arr[0..64].copy_from_slice(&data[0..64]);
	data_arr[64] = rec_id.to_i32() as u8;
	Ok(Signature(data_arr))
}

pub fn verify(public: &Public, signature: &Signature, message: &Message) -> Result<bool, Error> {
	let context = &SECP256K1;
	let rsig = try!(RecoverableSignature::from_compact(context, &signature[0..64], try!(RecoveryId::from_i32(signature[64] as i32))));
	let sig = rsig.to_standard(context);

	let pdata: [u8; 65] = {
		let mut temp = [4u8; 65];
		temp[1..65].copy_from_slice(public.deref());
		temp
	};

	let publ = try!(PublicKey::from_slice(context, &pdata));
	match context.verify(&try!(SecpMessage::from_slice(&message[..])), &sig, &publ) {
		Ok(_) => Ok(true),
		Err(SecpError::IncorrectSignature) => Ok(false),
		Err(x) => Err(Error::from(x))
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use {Generator, Random, Message};
	use super::{sign, verify, Signature};

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
	fn sign_and_verify() {
		let keypair = Random.generate().unwrap();
		let message = Message::default();
		let signature = sign(keypair.secret(), &message).unwrap();
		assert!(verify(keypair.public(), &signature, &message).unwrap());
	}
}
