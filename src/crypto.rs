use hash::*;
use secp256k1::Secp256k1;
use secp256k1::key;
use rand::os::OsRng;

pub type Secret = H256;
pub type Public = H512;
pub type Signature = H520;

impl Signature {
	/// Create a new signature from the R, S and V componenets.
	pub fn from_rsv(r: &H256, s: &H256, v: u8) -> Signature {
		use std::ptr;
		let mut ret: Signature = Signature::new();
		unsafe {
			let retslice: &mut [u8] = &mut ret;
			ptr::copy(r.as_ptr(), retslice.as_mut_ptr(), 32);
			ptr::copy(s.as_ptr(), retslice.as_mut_ptr().offset(32), 32);
		}
		ret[64] = v;
		ret
	}
}

#[derive(Debug)]
pub enum CryptoError {
	InvalidSecret,
	InvalidPublic,
	InvalidSignature,
	InvalidMessage,
	Io(::std::io::Error),
}

impl From<::secp256k1::Error> for CryptoError {
	fn from(e: ::secp256k1::Error) -> CryptoError {
		match e {
			::secp256k1::Error::InvalidMessage => CryptoError::InvalidMessage,
			::secp256k1::Error::InvalidPublicKey => CryptoError::InvalidPublic,
			::secp256k1::Error::InvalidSignature => CryptoError::InvalidSignature,
			::secp256k1::Error::InvalidSecretKey => CryptoError::InvalidSecret,
			_ => panic!("Crypto error: {:?}", e),
		}
	}
}

impl From<::std::io::Error> for CryptoError {
	fn from(err: ::std::io::Error) -> CryptoError {
		CryptoError::Io(err)
	}
}

#[derive(Debug, PartialEq, Eq)]
/// secp256k1 Key pair
///
/// Use `create()` to create a new random key pair. 
/// 
/// # Example
/// ```rust
/// extern crate ethcore_util;
/// use ethcore_util::crypto::*;
/// use ethcore_util::hash::*;
/// fn main() {
///   let pair = KeyPair::create().unwrap();
///   let message = H256::random();
///   let signature = sign(pair.secret(), &message).unwrap();
///
///   assert!(verify(pair.public(), &signature, &message).unwrap());
///   assert_eq!(recover(&signature, &message).unwrap(), *pair.public());
/// }
/// ```
pub struct KeyPair {
	secret: Secret,
	public: Public,
}

impl KeyPair {
	/// Create a pair from secret key
	pub fn from_secret(secret: Secret) -> Result<KeyPair, CryptoError> {
		let context = Secp256k1::new();
		let s: key::SecretKey = try!(key::SecretKey::from_slice(&context, &secret));
		let pub_key = try!(key::PublicKey::from_secret_key(&context, &s));
		let serialized = pub_key.serialize_vec(&context, false);
		let p: Public = Public::from_slice(&serialized[1..65]);
		Ok(KeyPair {
			secret: secret,
			public: p,
		})
	}
	/// Create a new random key pair
	pub fn create() -> Result<KeyPair, CryptoError> {
		let context = Secp256k1::new();
		let mut rng = try!(OsRng::new());
		let (sec, publ) = try!(context.generate_keypair(&mut rng));
		let serialized = publ.serialize_vec(&context, false);
		let p: Public = Public::from_slice(&serialized[1..65]);
		let s: Secret = unsafe { ::std::mem::transmute(sec) };
		Ok(KeyPair {
			secret: s,
			public: p,
		})
	}
	/// Returns public key
	pub fn public(&self) -> &Public {
		&self.public
	}
	/// Returns private key
	pub fn secret(&self) -> &Secret {
		&self.secret
	}

	/// Sign a message with our secret key.
	pub fn sign(&self, message: &H256) -> Result<Signature, CryptoError> { sign(&self.secret, message) }
}

/// Recovers Public key from signed message hash.
pub fn recover(signature: &Signature, message: &H256) -> Result<Public, CryptoError> {
	use secp256k1::*;
	let context = Secp256k1::new();
	let rsig = try!(RecoverableSignature::from_compact(&context, &signature[0..64], try!(RecoveryId::from_i32(signature[64] as i32))));
	let publ = try!(context.recover(&try!(Message::from_slice(&message)), &rsig));
	let serialized = publ.serialize_vec(&context, false);
	let p: Public = Public::from_slice(&serialized[1..65]);
	//TODO: check if it's the zero key and fail if so.

	Ok(p)
}

/// Returns siganture of message hash.
pub fn sign(secret: &Secret, message: &H256) -> Result<Signature, CryptoError> {
	use secp256k1::*;
	let context = Secp256k1::new();
	let sec: &key::SecretKey = unsafe { ::std::mem::transmute(secret) };
	let s = try!(context.sign_recoverable(&try!(Message::from_slice(&message)), sec));
	let (rec_id, data) = s.serialize_compact(&context);
	let mut signature: ::crypto::Signature = unsafe { ::std::mem::uninitialized() };
	signature.clone_from_slice(&data);
	signature[64] = rec_id.to_i32() as u8;
	Ok(signature)
}

/// Check if each component of the signature is in range.
pub fn is_valid(sig: &Signature) -> bool {
	sig[64] <= 1 &&
		H256::from_slice(&sig[0..32]) < h256_from_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141") &&
		H256::from_slice(&sig[32..64]) < h256_from_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141") &&
		H256::from_slice(&sig[32..64]) >= h256_from_u64(1) &&
		H256::from_slice(&sig[0..32]) >= h256_from_u64(1)
}

/// Verify signature.
pub fn verify(public: &Public, signature: &Signature, message: &H256) -> Result<bool, CryptoError> {
	use secp256k1::*;
	let context = Secp256k1::new();
	let rsig = try!(RecoverableSignature::from_compact(&context, &signature[0..64], try!(RecoveryId::from_i32(signature[64] as i32))));
	let sig = rsig.to_standard(&context);

	let mut pdata: [u8; 65] = [4u8; 65];
	let ptr = pdata[1..].as_mut_ptr();
	let src = public.as_ptr();
	unsafe { ::std::ptr::copy_nonoverlapping(src, ptr, 64) };
	let publ = try!(key::PublicKey::from_slice(&context, &pdata));
	match context.verify(&try!(Message::from_slice(&message)), &sig, &publ) {
		Ok(_) => Ok(true),
		Err(Error::IncorrectSignature) => Ok(false),
		Err(x) => Err(<CryptoError as From<Error>>::from(x))
	}
}

#[cfg(test)]
mod tests {
	use hash::*;
	use crypto::*;

	// TODO: tests for sign/recover roundtrip, at least.

	#[test]
	fn test_signature() {
		let pair = KeyPair::create().unwrap();
		let message = H256::random();
		let signature = sign(pair.secret(), &message).unwrap();

		assert!(verify(pair.public(), &signature, &message).unwrap());
		assert_eq!(recover(&signature, &message).unwrap(), *pair.public());
	}

	#[test]
	fn test_invalid_key() {
		assert!(KeyPair::from_secret(h256_from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")).is_err());
		assert!(KeyPair::from_secret(h256_from_hex("0000000000000000000000000000000000000000000000000000000000000000")).is_err());
		assert!(KeyPair::from_secret(h256_from_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141")).is_err());
	}

	#[test]
	fn test_key() {
		let pair = KeyPair::from_secret(h256_from_hex("6f7b0d801bc7b5ce7bbd930b84fd0369b3eb25d09be58d64ba811091046f3aa2")).unwrap();
		assert_eq!(pair.public().hex(), "101b3ef5a4ea7a1c7928e24c4c75fd053c235d7b80c22ae5c03d145d0ac7396e2a4ffff9adee3133a7b05044a5cee08115fd65145e5165d646bde371010d803c");
	}
}
