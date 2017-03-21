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

extern crate rand;
#[macro_use]
extern crate lazy_static;
extern crate tiny_keccak;
extern crate secp256k1;
extern crate rustc_serialize;
extern crate ethcore_bigint as bigint;
extern crate crypto as rcrypto;
extern crate byteorder;

mod brain;
mod error;
mod keypair;
mod keccak;
mod prefix;
mod random;
mod signature;
mod secret;
mod extended;

lazy_static! {
	pub static ref SECP256K1: secp256k1::Secp256k1 = secp256k1::Secp256k1::new();
}

/// Generates new keypair.
pub trait Generator {
	/// Should be called to generate new keypair.
	fn generate(self) -> Result<KeyPair, Error>;
}

pub mod math;

pub use self::brain::Brain;
pub use self::error::Error;
pub use self::keypair::{KeyPair, public_to_address};
pub use self::prefix::Prefix;
pub use self::random::Random;
pub use self::signature::{sign, verify_public, verify_address, recover, Signature};
pub use self::secret::Secret;
pub use self::extended::{ExtendedPublic, ExtendedSecret, ExtendedKeyPair, DerivationError, Derivation};

use bigint::hash::{H160, H256, H512};

pub type Address = H160;
pub type Message = H256;
pub type Public = H512;
