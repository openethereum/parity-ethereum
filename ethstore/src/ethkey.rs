//! ethkey reexport to make documentation look pretty.
pub use _ethkey::{Address, Message, Signature, Public, Secret, Generator, sign, verify, Error, KeyPair, Random, Prefix};
use json;

impl Into<json::H160> for Address {
	fn into(self) -> json::H160 {
		let a: [u8; 20] = self.into();
		From::from(a)
	}
}

impl From<json::H160> for Address {
	fn from(json: json::H160) -> Self {
		let a: [u8; 20] = json.into();
		From::from(a)
	}
}
