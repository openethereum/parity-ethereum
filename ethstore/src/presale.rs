use std::fs;
use std::path::Path;
use rcrypto::pbkdf2::pbkdf2;
use rcrypto::sha2::Sha256;
use rcrypto::hmac::Hmac;
use json;
use ethkey::{Address, Secret, KeyPair};
use crypto::Keccak256;
use {crypto, Error};

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
	pub fn open<P>(path: P) -> Result<Self, Error> where P: AsRef<Path> {
		let file = fs::File::open(path)?;
		let presale = json::PresaleWallet::load(file)
			.map_err(|e| Error::InvalidKeyFile(format!("{}", e)))?;
		Ok(PresaleWallet::from(presale))
	}

	pub fn decrypt(&self, password: &str) -> Result<KeyPair, Error> {
		let mut h_mac = Hmac::new(Sha256::new(), password.as_bytes());
		let mut derived_key = vec![0u8; 16];
		pbkdf2(&mut h_mac, password.as_bytes(), 2000, &mut derived_key);

		let mut key = vec![0; self.ciphertext.len()];
		let len = crypto::aes::decrypt_cbc(&derived_key, &self.iv, &self.ciphertext, &mut key).map_err(|_| Error::InvalidPassword)?;
		let unpadded = &key[..len];

		let secret = Secret::from_slice(&unpadded.keccak256())?;
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
