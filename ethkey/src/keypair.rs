use std::fmt;
use secp256k1::key;
use rustc_serialize::hex::ToHex;
use keccak::Keccak256;
use super::{Secret, Public, Address, SECP256K1, Error};

pub fn public_to_address(public: &Public) -> Address {
	let hash = public.keccak256();
	let mut result = Address::default();
	result.copy_from_slice(&hash[12..]);
	result
}

/// secp256k1 key pair
pub struct KeyPair {
	secret: Secret,
	public: Public,
}

impl fmt::Display for KeyPair {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		try!(writeln!(f, "secret:  {}", self.secret.to_hex()));
		try!(writeln!(f, "public:  {}", self.public.to_hex()));
		write!(f, "address: {}", self.address().to_hex())
	}
}

impl KeyPair {
	/// Create a pair from secret key
	pub fn from_secret(secret: Secret) -> Result<KeyPair, Error> {
		let context = &SECP256K1;
		let s: key::SecretKey = try!(key::SecretKey::from_slice(context, &secret[..]));
		let pub_key = try!(key::PublicKey::from_secret_key(context, &s));
		let serialized = pub_key.serialize_vec(context, false);

		let mut public = Public::default();
		public.copy_from_slice(&serialized[1..65]);

		let keypair = KeyPair {
			secret: secret,
			public: public,
		};

		Ok(keypair)
	}

	pub fn from_keypair(sec: key::SecretKey, publ: key::PublicKey) -> Self {
		let context = &SECP256K1;
		let serialized = publ.serialize_vec(context, false);
		let mut secret = Secret::default();
		secret.copy_from_slice(&sec[0..32]);
		let mut public = Public::default();
		public.copy_from_slice(&serialized[1..65]);

		KeyPair {
			secret: secret,
			public: public,
		}
	}

	pub fn secret(&self) -> &Secret {
		&self.secret
	}

	pub fn public(&self) -> &Public {
		&self.public
	}

	pub fn address(&self) -> Address {
		public_to_address(&self.public)
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use {KeyPair, Secret};

	#[test]
	fn from_secret() {
		let secret = Secret::from_str("a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65").unwrap();
		let _ = KeyPair::from_secret(secret).unwrap();
	}

	#[test]
	fn keypair_display() {
		let expected =
"secret:  a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65
public:  8ce0db0b0359ffc5866ba61903cc2518c3675ef2cf380a7e54bde7ea20e6fa1ab45b7617346cd11b7610001ee6ae5b0155c41cad9527cbcdff44ec67848943a4
address: 5b073e9233944b5e729e46d618f0d8edf3d9c34a".to_owned();
		let secret = Secret::from_str("a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65").unwrap();
		let kp = KeyPair::from_secret(secret).unwrap();
		assert_eq!(format!("{}", kp), expected);
	}
}
