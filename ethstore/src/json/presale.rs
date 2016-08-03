use std::io::Read;
use std::ops::Deref;
use serde_json;
use serde::{Deserialize, Deserializer, Error};
use rustc_serialize::hex::FromHex;
use super::{H160};

#[derive(Debug, PartialEq)]
pub struct Encseed(Vec<u8>);

impl Deref for Encseed {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Deserialize for Encseed {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
		where D: Deserializer
	{
		let s = try!(String::deserialize(deserializer));
		let data = try!(s.from_hex().map_err(|e| Error::custom(format!("Invalid hex value {}", e))));
		Ok(Encseed(data))
	}
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct PresaleWallet {
	pub encseed: Encseed,
	#[serde(rename = "ethaddr")]
	pub address: H160,
}

impl PresaleWallet {
	pub fn load<R>(reader: R) -> Result<Self, serde_json::Error> where R: Read {
		serde_json::from_reader(reader)
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use serde_json;
	use rustc_serialize::hex::FromHex;
	use json::{PresaleWallet, H160, Encseed};

	#[test]
	fn presale_wallet() {
		let json = r#"
		{
			"encseed": "137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066",
			"ethaddr": "ede84640d1a1d3e06902048e67aa7db8d52c2ce1",
			"email": "123@gmail.com",
			"btcaddr": "1JvqEc6WLhg6GnyrLBe2ztPAU28KRfuseH"
		} "#;

		let expected = PresaleWallet {
			encseed: Encseed("137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066".from_hex().unwrap()),
			address: H160::from_str("ede84640d1a1d3e06902048e67aa7db8d52c2ce1").unwrap(),
		};

		let wallet: PresaleWallet = serde_json::from_str(json).unwrap();
		assert_eq!(expected, wallet);
	}

	#[test]
	fn long_presale_wallet() {
		let json = r#"
		{
			"encseed":
			"137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0d",
			"ethaddr": "ede84640d1a1d3e06902048e67aa7db8d52c2ce1",
			"email": "123@gmail.com",
			"btcaddr": "1JvqEc6WLhg6GnyrLBe2ztPAU28KRfuseH"
		} "#;

		let expected = PresaleWallet {
			encseed: Encseed("137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0dcf0d".from_hex().unwrap()),
			address: H160::from_str("ede84640d1a1d3e06902048e67aa7db8d52c2ce1").unwrap(),
		};

		let wallet: PresaleWallet = serde_json::from_str(json).unwrap();
		assert_eq!(expected, wallet);
	}
}
