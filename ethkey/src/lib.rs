extern crate rand;
#[macro_use]
extern crate lazy_static;
extern crate tiny_keccak;
extern crate secp256k1;
extern crate rustc_serialize;

mod brain;
mod error;
mod keypair;
mod keccak;
mod prefix;
mod primitive;
mod random;
mod signature;

lazy_static! {
	static ref SECP256K1: secp256k1::Secp256k1 = secp256k1::Secp256k1::new();
}

/// Generates new keypair.
pub trait Generator {
	/// Should be called to generate new keypair.
	fn generate(self) -> Result<KeyPair, Error>;
}

pub use self::brain::Brain;
pub use self::error::Error;
pub use self::keypair::KeyPair;
pub use self::primitive::{Secret, Public, Address, Message};
pub use self::prefix::Prefix;
pub use self::random::Random;
pub use self::signature::{sign, verify, Signature};
