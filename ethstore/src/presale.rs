// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::fs;
use std::path::Path;
use json;
use ethkey::{Address, Secret, KeyPair};
use crypto::{Keccak256, pbkdf2};
use {crypto, Error};

/// Pre-sale wallet.
pub struct PresaleWallet {
	iv: [u8; 16],
	ciphertext: Vec<u8>,
	address: Address,
}

impl From<json::PresaleWallet> for PresaleWallet {
	fn from(wallet: json::PresaleWallet) -> Self {
		let mut iv = [0u8; 16];
		iv.copy_from_slice(&wallet.encseed[..16]);

		let mut ciphertext = vec![];
		ciphertext.extend_from_slice(&wallet.encseed[16..]);

		PresaleWallet {
			iv: iv,
			ciphertext: ciphertext,
			address: Address::from(wallet.address),
		}
	}
}

impl PresaleWallet {
	/// Open a pre-sale wallet.
	pub fn open<P>(path: P) -> Result<Self, Error> where P: AsRef<Path> {
		let file = fs::File::open(path)?;
		let presale = json::PresaleWallet::load(file)
			.map_err(|e| Error::InvalidKeyFile(format!("{}", e)))?;
		Ok(PresaleWallet::from(presale))
	}

	/// Decrypt the wallet.
	pub fn decrypt(&self, password: &str) -> Result<KeyPair, Error> {
		let mut derived_key = [0u8; 32];
		let salt = pbkdf2::Salt(password.as_bytes());
		let sec = pbkdf2::Secret(password.as_bytes());
		pbkdf2::sha256(2000, salt, sec, &mut derived_key);

		let mut key = vec![0; self.ciphertext.len()];
		let len = crypto::aes::decrypt_128_cbc(&derived_key[0..16], &self.iv, &self.ciphertext, &mut key)
			.map_err(|_| Error::InvalidPassword)?;
		let unpadded = &key[..len];

		let secret = Secret::from_unsafe_slice(&unpadded.keccak256())?;
		if let Ok(kp) = KeyPair::from_secret(secret) {
			if kp.address() == self.address {
				return Ok(kp)
			}
		}

		Err(Error::InvalidPassword)
	}
}

#[cfg(test)]
mod tests {
	use super::PresaleWallet;
	use json;

	#[test]
	fn test() {
		let json = r#"
		{
			"encseed": "137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066",
			"ethaddr": "ede84640d1a1d3e06902048e67aa7db8d52c2ce1",
			"email": "123@gmail.com",
			"btcaddr": "1JvqEc6WLhg6GnyrLBe2ztPAU28KRfuseH"
		} "#;

		let wallet = json::PresaleWallet::load(json.as_bytes()).unwrap();
		let wallet = PresaleWallet::from(wallet);
		assert!(wallet.decrypt("123").is_ok());
		assert!(wallet.decrypt("124").is_err());
	}
}
