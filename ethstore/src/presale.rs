use rcrypto::pbkdf2::pbkdf2;
use rcrypto::sha2::Sha256;
use rcrypto::hmac::Hmac;
use json::PresaleWallet;
use ethkey::{Address, Secret, KeyPair};
use crypto::Keccak256;
use {crypto, Error};

fn decrypt_presale_wallet(wallet: &PresaleWallet, password: &str) -> Result<KeyPair, Error> {
	let mut iv = [0u8; 16];
	iv.copy_from_slice(&wallet.encseed[..16]);

	let mut ciphertext = [0u8; 80];
	ciphertext.copy_from_slice(&wallet.encseed[16..]);

	let mut h_mac = Hmac::new(Sha256::new(), password.as_bytes());
	let mut derived_key = vec![0u8; 16];
	pbkdf2(&mut h_mac, password.as_bytes(), 2000, &mut derived_key);

	let mut key = [0u8; 64];
	crypto::aes::decrypt_cbc(&derived_key, &iv, &ciphertext, &mut key);

	let secret = Secret::from(key.keccak256());
	if let Ok(kp) = KeyPair::from_secret(secret) {
		if kp.address() == Address::from(&wallet.address) {
			return Ok(kp)
		}
	}

	Err(Error::InvalidPassword)
}

#[cfg(test)]
mod tests {
	use ethkey::{KeyPair, Address};
	use super::decrypt_presale_wallet;
	use json::PresaleWallet;

	#[test]
	fn test() {
		let json = r#"
		{
			"encseed": "137103c28caeebbcea5d7f95edb97a289ded151b72159137cb7b2671f394f54cff8c121589dcb373e267225547b3c71cbdb54f6e48ec85cd549f96cf0dedb3bc0a9ac6c79b9c426c5878ca2c9d06ff42a23cb648312fc32ba83649de0928e066",
			"ethaddr": "ede84640d1a1d3e06902048e67aa7db8d52c2ce1",
			"email": "123@gmail.com",
			"btcaddr": "1JvqEc6WLhg6GnyrLBe2ztPAU28KRfuseH"
		} "#;

		let wallet = PresaleWallet::load(json.as_bytes()).unwrap();
		let kp = decrypt_presale_wallet(&wallet, "123").unwrap();
		assert_eq!(kp.address(), Address::from(wallet.address));
	}
}
