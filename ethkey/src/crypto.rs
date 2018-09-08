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

use secp256k1;
use std::io;
use parity_crypto::error::SymmError;



quick_error! {
	#[derive(Debug)]
	pub enum Error {
		Secp(e: secp256k1::Error) {
			display("secp256k1 error: {}", e)
			cause(e)
			from()
		}
		Io(e: io::Error) {
			display("i/o error: {}", e)
			cause(e)
			from()
		}
		InvalidMessage {
			display("invalid message")
		}
		Symm(e: SymmError) {
			cause(e)
			from()
		}
	}
}

/// ECDH functions
pub mod ecdh {
	use secp256k1::{self, ecdh, key};
	use super::Error;
	use {Secret, Public, SECP256K1};

	/// Agree on a shared secret
	pub fn agree(secret: &Secret, public: &Public) -> Result<Secret, Error> {
		let context = &SECP256K1;
		let pdata = {
			let mut temp = [4u8; 65];
			(&mut temp[1..65]).copy_from_slice(&public[0..64]);
			temp
		};

		let publ = key::PublicKey::from_slice(context, &pdata)?;
		let sec = key::SecretKey::from_slice(context, &secret)?;
		let shared = ecdh::SharedSecret::new_raw(context, &publ, &sec);

		Secret::from_unsafe_slice(&shared[0..32])
			.map_err(|_| Error::Secp(secp256k1::Error::InvalidSecretKey))
	}
}

/// ECIES function
pub mod ecies {
	use parity_crypto::{aes, digest, hmac, is_equal};
	use ethereum_types::H128;
	use super::{ecdh, Error};
	use {Random, Generator, Public, Secret};

	/// Encrypt a message with a public key, writing an HMAC covering both
	/// the plaintext and authenticated data.
	///
	/// Authenticated data may be empty.
	pub fn encrypt(public: &Public, auth_data: &[u8], plain: &[u8]) -> Result<Vec<u8>, Error> {
		let r = Random.generate()?;
		let z = ecdh::agree(r.secret(), public)?;
		let mut key = [0u8; 32];
		kdf(&z, &[0u8; 0], &mut key);

		let ekey = &key[0..16];
		let mkey = hmac::SigKey::sha256(&digest::sha256(&key[16..32]));

		let mut msg = vec![0u8; 1 + 64 + 16 + plain.len() + 32];
		msg[0] = 0x04u8;
		{
			let msgd = &mut msg[1..];
			msgd[0..64].copy_from_slice(r.public());
			let iv = H128::random();
			msgd[64..80].copy_from_slice(&iv);
			{
				let cipher = &mut msgd[(64 + 16)..(64 + 16 + plain.len())];
				aes::encrypt_128_ctr(ekey, &iv, plain, cipher)?;
			}
			let mut hmac = hmac::Signer::with(&mkey);
			{
				let cipher_iv = &msgd[64..(64 + 16 + plain.len())];
				hmac.update(cipher_iv);
			}
			hmac.update(auth_data);
			let sig = hmac.sign();
			msgd[(64 + 16 + plain.len())..].copy_from_slice(&sig);
		}
		Ok(msg)
	}

	/// Decrypt a message with a secret key, checking HMAC for ciphertext
	/// and authenticated data validity.
	pub fn decrypt(secret: &Secret, auth_data: &[u8], encrypted: &[u8]) -> Result<Vec<u8>, Error> {
		let meta_len = 1 + 64 + 16 + 32;
		if encrypted.len() < meta_len  || encrypted[0] < 2 || encrypted[0] > 4 {
			return Err(Error::InvalidMessage); //invalid message: publickey
		}

		let e = &encrypted[1..];
		let p = Public::from_slice(&e[0..64]);
		let z = ecdh::agree(secret, &p)?;
		let mut key = [0u8; 32];
		kdf(&z, &[0u8; 0], &mut key);

		let ekey = &key[0..16];
		let mkey = hmac::SigKey::sha256(&digest::sha256(&key[16..32]));

		let clen = encrypted.len() - meta_len;
		let cipher_with_iv = &e[64..(64+16+clen)];
		let cipher_iv = &cipher_with_iv[0..16];
		let cipher_no_iv = &cipher_with_iv[16..];
		let msg_mac = &e[(64+16+clen)..];

		// Verify tag
		let mut hmac = hmac::Signer::with(&mkey);
		hmac.update(cipher_with_iv);
		hmac.update(auth_data);
		let mac = hmac.sign();

		if !is_equal(&mac.as_ref()[..], msg_mac) {
			return Err(Error::InvalidMessage);
		}

		let mut msg = vec![0u8; clen];
		aes::decrypt_128_ctr(ekey, cipher_iv, cipher_no_iv, &mut msg[..])?;
		Ok(msg)
	}

	fn kdf(secret: &Secret, s1: &[u8], dest: &mut [u8]) {
		// SEC/ISO/Shoup specify counter size SHOULD be equivalent
		// to size of hash output, however, it also notes that
		// the 4 bytes is okay. NIST specifies 4 bytes.
		let mut ctr = 1u32;
		let mut written = 0usize;
		while written < dest.len() {
			let mut hasher = digest::Hasher::sha256();
			let ctrs = [(ctr >> 24) as u8, (ctr >> 16) as u8, (ctr >> 8) as u8, ctr as u8];
			hasher.update(&ctrs);
			hasher.update(secret);
			hasher.update(s1);
			let d = hasher.finish();
			&mut dest[written..(written + 32)].copy_from_slice(&d);
			written += 32;
			ctr += 1;
		}
	}
}

pub mod eip1024{

////****  Imports for mod eip1024   *****////
use sodiumoxide;
use serde_json;
use sodiumoxide::crypto::box_;
use serde_json::Error as serdeError;
/////***  End imports for mod eip1024  ***//////



	static VERSION: &'static str = "x25519-xsalsa20-poly1305";

	type JsonStr = String;

	#[derive(Serialize)]
	struct Digest <'a>{
	    version: &'a str,
	    ciphertext: &'a[u8],
	    ephemPublicKey: &'a[u8],
	    nonce: &'a[u8]
	}

	#[derive(Deserialize, Debug)]
	struct Undigest{
	    version: String,
	    ciphertext: Vec<u8>,
	    ephemPublicKey: Vec<u8>,
	    nonce: Vec<u8>
	}



	pub fn encrypt(receiver_public_key: &[u8], msg: &[u8]) -> Result<JsonStr, serdeError>{

	    //generate my Ephemeral key pair
	    let ( ephem_pub_key ,ephem_priv_key ) = box_::gen_keypair();



	    //convert received public key to sodiumoxide type
	    let rpubk = box_::PublicKey::from_slice(receiver_public_key)
	            .expect("receiver_public_key has  invalid length");


	    //generate a nonce
	    let nonce = box_::gen_nonce();


	    let ciphertext = box_::seal(msg, &nonce, &rpubk, &ephem_priv_key);

	    
	    //serialized output
	    //{ version, ciphertext, nonce, ephemPublicKey }
	    let ephem_pub_key = ephem_pub_key.as_ref();
	    let nonce = nonce.as_ref();
	    let encrypted = Digest {
	                    version: VERSION,
	                    ciphertext: &ciphertext,
	                    ephemPublicKey: ephem_pub_key,
	                    //signature: &signature
	                    nonce: nonce
	    };


	    let serialized = serde_json::to_string(&encrypted)?; 
	    
	    

	    Ok(serialized)

	}



	pub fn decrypt(encrypted_blob: &JsonStr, my_private_key: &[u8] ) -> Result<Vec<u8>, ()>{

	    
	    let private_key = box_::SecretKey::from_slice(my_private_key)
	                        .expect("my_private_key has invalid length");
	            



	    //Assuming encrypted_blob: json string   { version, ciphertext, nonce, ephemPublicKey }
	    let deserialized: Undigest = match serde_json::from_str(encrypted_blob){
	                    Ok(i) => i,
	                    Err(e) => return Err(())
	    };


	    let sender_public_key = box_::PublicKey::from_slice(&deserialized.ephemPublicKey)
	            .expect("ephemPublicKey appears to be invalid length");

	    let nonce = box_::Nonce::from_slice(&deserialized.nonce).unwrap();


	    match box_::open(&deserialized.ciphertext, &nonce, &sender_public_key,&private_key) {
	        Ok(i) => Ok(i),
	        _ => Err(())
	    }

	    

	}


}

#[cfg(test)]
mod tests {
	use super::ecies;
	use {Random, Generator};
	use super::eip1024;
	use sodiumoxide::crypto::box_;

	#[test]
	fn ecies_shared() {
		let kp = Random.generate().unwrap();
		let message = b"So many books, so little time";

		let shared = b"shared";
		let wrong_shared = b"incorrect";
		let encrypted = ecies::encrypt(kp.public(), shared, message).unwrap();
		assert!(encrypted[..] != message[..]);
		assert_eq!(encrypted[0], 0x04);

		assert!(ecies::decrypt(kp.secret(), wrong_shared, &encrypted).is_err());
		let decrypted = ecies::decrypt(kp.secret(), shared, &encrypted).unwrap();
		assert_eq!(decrypted[..message.len()], message[..]);
	}

	#[test]
    fn encrypt_decrypt_eip1024(){
      let plaintext = b"E pluribus unum";
      let (senderpubk, senderprivk) = box_::gen_keypair();

      let senderpubk = &senderpubk[..];
      let senderprivk = &senderprivk[..];
      
      let encr_json = eip1024::encrypt(senderpubk, plaintext).unwrap();
      
      let decr_plaintext = eip1024::decrypt(&encr_json, senderprivk).unwrap();

      assert_eq!(plaintext, decr_plaintext.as_slice()); 
    }
}
