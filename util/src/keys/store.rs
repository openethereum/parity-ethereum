// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Secret Store

use keys::directory::*;
use common::*;
use rcrypto::pbkdf2::*;
use rcrypto::scrypt::*;
use rcrypto::hmac::*;
use crypto;

const KEY_LENGTH: u32 = 32;
const KEY_ITERATIONS: u32 = 10240;
const KEY_LENGTH_AES: u32 = KEY_LENGTH/2;

const KEY_LENGTH_USIZE: usize = KEY_LENGTH as usize;
const KEY_LENGTH_AES_USIZE: usize = KEY_LENGTH_AES as usize;

/// Encrypted hash-map, each request should contain password
pub trait EncryptedHashMap<Key: Hash + Eq> {
	/// Returns existing value for the key, if any
	fn get<Value: FromRawBytes + BytesConvertable>(&self, key: &Key, password: &str) ->  Result<Value, EncryptedHashMapError>;
	/// Insert new encrypted key-value and returns previous if there was any
	fn insert<Value: FromRawBytes + BytesConvertable>(&mut self, key: Key, value: Value, password: &str) -> Option<Value>;
	/// Removes key-value by key and returns the removed one, if any exists and password was provided
	fn remove<Value: FromRawBytes + BytesConvertable> (&mut self, key: &Key, password: Option<&str>) -> Option<Value>;
	/// Deletes key-value by key and returns if the key-value existed
	fn delete(&mut self, key: &Key) -> bool {
		self.remove::<Bytes>(key, None).is_some()
	}
}

/// Error retrieving value from encrypted hashmap
#[derive(Debug)]
pub enum EncryptedHashMapError {
	/// Encryption failed
	InvalidPassword,
	/// No key in the hashmap
	UnknownIdentifier,
	/// Stored value is not well formed for the requested type
	InvalidValueFormat(FromBytesError),
}

/// Represent service for storing encrypted arbitrary data
pub struct SecretStore {
	directory: KeyDirectory
}

impl SecretStore {
	/// new instance of Secret Store in default home directory
	pub fn new() -> SecretStore {
		let mut path = ::std::env::home_dir().expect("Failed to get home dir");
		path.push("keystore");
		Self::new_in(&path)
	}

	/// new instance of Secret Store in specific directory
	pub fn new_in(path: &Path) -> SecretStore {
		SecretStore {
			directory: KeyDirectory::new(path)
		}
	}

	/// Lists all accounts and corresponding key ids
	pub fn accounts(&self) -> Result<Vec<(Address, H128)>, ::std::io::Error> {
		let accounts = try!(self.directory.list()).iter().map(|key_id| self.directory.get(key_id))
			.filter(|key| key.is_some())
			.map(|key| { let some_key = key.unwrap(); (some_key.account, some_key.id) })
			.filter(|&(ref account, _)| account.is_some())
			.map(|(account, id)| (account.unwrap(), id))
			.collect::<Vec<(Address, H128)>>();
		Ok(accounts)
	}

	/// Resolves key_id by account address
	pub fn account(&self, account: &Address) -> Option<H128> {
		let mut accounts = match self.accounts() {
			Ok(accounts) => accounts,
			Err(e) => { warn!(target: "sstore", "Failed to load accounts: {}", e); return None; }
		};
		accounts.retain(|&(ref store_account, _)| account == store_account);
		accounts.first().and_then(|&(_, ref key_id)| Some(key_id.clone()))
	}

	/// Imports pregenerated key, returns error if not saved correctly
	pub fn import_key(&mut self, key_file: KeyFileContent) -> Result<(), ::std::io::Error> {
		try!(self.directory.save(key_file));
		Ok(())
	}

	#[cfg(test)]
	fn new_test(path: &::devtools::RandomTempPath) -> SecretStore {
		SecretStore {
			directory: KeyDirectory::new(path.as_path())
		}
	}
}

fn derive_key_iterations(password: &str, salt: &H256, c: u32) -> (Bytes, Bytes) {
	let mut h_mac = Hmac::new(::rcrypto::sha2::Sha256::new(), password.as_bytes());
	let mut derived_key = vec![0u8; KEY_LENGTH_USIZE];
	pbkdf2(&mut h_mac, &salt.as_slice(), c, &mut derived_key);
	let derived_right_bits = &derived_key[0..KEY_LENGTH_AES_USIZE];
	let derived_left_bits = &derived_key[KEY_LENGTH_AES_USIZE..KEY_LENGTH_USIZE];
	(derived_right_bits.to_vec(), derived_left_bits.to_vec())
}

fn derive_key(password: &str, salt: &H256) -> (Bytes, Bytes) {
	derive_key_iterations(password, salt, KEY_ITERATIONS)
}

fn derive_key_scrypt(password: &str, salt: &H256, n: u32, p: u32, r: u32) -> (Bytes, Bytes) {
	let mut derived_key = vec![0u8; KEY_LENGTH_USIZE];
	let scrypt_params = ScryptParams::new(n.trailing_zeros() as u8, r, p);
	scrypt(password.as_bytes(), &salt.as_slice(), &scrypt_params, &mut derived_key);
	let derived_right_bits = &derived_key[0..KEY_LENGTH_AES_USIZE];
	let derived_left_bits = &derived_key[KEY_LENGTH_AES_USIZE..KEY_LENGTH_USIZE];
	(derived_right_bits.to_vec(), derived_left_bits.to_vec())
}

fn derive_mac(derived_left_bits: &[u8], cipher_text: &[u8]) -> Bytes {
	let mut mac = vec![0u8; KEY_LENGTH_AES_USIZE + cipher_text.len()];
	mac[0..KEY_LENGTH_AES_USIZE].clone_from_slice(derived_left_bits);
	mac[KEY_LENGTH_AES_USIZE..cipher_text.len()+KEY_LENGTH_AES_USIZE].clone_from_slice(cipher_text);
	mac
}

impl EncryptedHashMap<H128> for SecretStore {
	fn get<Value: FromRawBytes + BytesConvertable>(&self, key: &H128, password: &str) -> Result<Value, EncryptedHashMapError> {
		match self.directory.get(key) {
			Some(key_file) => {
				let (derived_left_bits, derived_right_bits) = match key_file.crypto.kdf {
					KeyFileKdf::Pbkdf2(ref params) => derive_key_iterations(password, &params.salt, params.c),
					KeyFileKdf::Scrypt(ref params) => derive_key_scrypt(password, &params.salt, params.n, params.p, params.r)
				};

				if derive_mac(&derived_right_bits, &key_file.crypto.cipher_text)
					.sha3() != key_file.crypto.mac { return Err(EncryptedHashMapError::InvalidPassword); }

				let mut val = vec![0u8; key_file.crypto.cipher_text.len()];
				match key_file.crypto.cipher_type {
					CryptoCipherType::Aes128Ctr(ref iv) => {
						crypto::aes::decrypt(&derived_left_bits, &iv.as_slice(), &key_file.crypto.cipher_text, &mut val);
					}
				};

				match Value::from_bytes(&val) {
					Ok(value) => Ok(value),
					Err(bytes_error) => Err(EncryptedHashMapError::InvalidValueFormat(bytes_error))
				}
			},
			None => Err(EncryptedHashMapError::UnknownIdentifier)
		}
	}

	fn insert<Value: FromRawBytes + BytesConvertable>(&mut self, key: H128, value: Value, password: &str) -> Option<Value> {
		let previous = if let Ok(previous_value) = self.get(&key, password) { Some(previous_value) } else { None };

		// crypto random initiators
		let salt = H256::random();
		let iv = H128::random();

		// two parts of derived key
		// DK = [ DK[0..15] DK[16..31] ] = [derived_left_bits, derived_right_bits]
		let (derived_left_bits, derived_right_bits) = derive_key(password, &salt);

		let mut cipher_text = vec![0u8; value.as_slice().len()];
		// aes-128-ctr with initial vector of iv
		crypto::aes::encrypt(&derived_left_bits, &iv.clone(), &value.as_slice(), &mut cipher_text);

		// KECCAK(DK[16..31] ++ <ciphertext>), where DK[16..31] - derived_right_bits
		let mac = derive_mac(&derived_right_bits, &cipher_text.clone()).sha3();

		let mut key_file = KeyFileContent::new(
			KeyFileCrypto::new_pbkdf2(
				cipher_text,
				iv,
				salt,
				mac,
				KEY_ITERATIONS,
				KEY_LENGTH));
		key_file.id = key;
		if let Err(io_error) = self.directory.save(key_file) {
			warn!("Error saving key file: {:?}", io_error);
		}
		previous
	}

	fn remove<Value: FromRawBytes + BytesConvertable>(&mut self, key: &H128, password: Option<&str>) -> Option<Value> {
		let previous = if let Some(pass) = password {
			if let Ok(previous_value) = self.get(&key, pass) { Some(previous_value) } else { None }
		}
		else { None };

		if let Err(io_error) = self.directory.delete(key) {
			warn!("Error saving key file: {:?}", io_error);
		}
		previous
	}

}

#[cfg(test)]
mod vector_tests {
	use super::{derive_mac,derive_key_iterations};
	use common::*;


	#[test]
	fn mac_vector() {
		let password = "testpassword";
		let salt = H256::from_str("ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd").unwrap();
		let cipher_text = FromHex::from_hex("5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46").unwrap();
		let iterations = 262144u32;

		let (derived_left_bits, derived_right_bits) = derive_key_iterations(password, &salt, iterations);
		assert_eq!("f06d69cdc7da0faffb1008270bca38f5", derived_left_bits.to_hex());
		assert_eq!("e31891a3a773950e6d0fea48a7188551", derived_right_bits.to_hex());

		let mac_body = derive_mac(&derived_right_bits, &cipher_text);
		assert_eq!("e31891a3a773950e6d0fea48a71885515318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46", mac_body.to_hex());

		let mac = mac_body.sha3();
		assert_eq!("517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2", format!("{:?}", mac));
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use devtools::*;
	use common::*;

	#[test]
	fn can_insert() {
		let temp = RandomTempPath::create_dir();
		let mut sstore = SecretStore::new_test(&temp);

		let id = H128::random();
		sstore.insert(id.clone(), "Cat".to_owned(), "pass");

		assert!(sstore.get::<String>(&id, "pass").is_ok());
	}

	#[test]
	fn can_get_fail() {
		let temp = RandomTempPath::create_dir();
		{
			use keys::directory::{KeyFileContent, KeyFileCrypto};
			let mut write_sstore = SecretStore::new_test(&temp);
			write_sstore.directory.save(
				KeyFileContent::new(
					KeyFileCrypto::new_pbkdf2(
						FromHex::from_hex("5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46").unwrap(),
						H128::from_str("6087dab2f9fdbbfaddc31a909735c1e6").unwrap(),
						H256::from_str("ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd").unwrap(),
						H256::from_str("517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2").unwrap(),
						262144,
						32)))
				.unwrap();
		}
		let sstore = SecretStore::new_test(&temp);
		if let Ok(_) = sstore.get::<Bytes>(&H128::from_str("3198bc9c66725ab3d9954942343ae5b6").unwrap(), "testpassword") {
			panic!("should be error loading key,  we requested the wrong key");
		}
	}

	fn pregenerate_keys(temp: &RandomTempPath, count: usize) -> Vec<H128> {
		use keys::directory::{KeyFileContent, KeyFileCrypto};
		let mut write_sstore = SecretStore::new_test(&temp);
		let mut result = Vec::new();
		for _ in 0..count {
			result.push(write_sstore.directory.save(
				KeyFileContent::new(
					KeyFileCrypto::new_pbkdf2(
						FromHex::from_hex("5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46").unwrap(),
						H128::from_str("6087dab2f9fdbbfaddc31a909735c1e6").unwrap(),
						H256::from_str("ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd").unwrap(),
						H256::from_str("517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2").unwrap(),
						262144,
						32)))
				.unwrap());
		}
		result
	}

	fn pregenerate_accounts(temp: &RandomTempPath, count: usize) -> Vec<H128> {
		use keys::directory::{KeyFileContent, KeyFileCrypto};
		let mut write_sstore = SecretStore::new_test(&temp);
		let mut result = Vec::new();
		for i in 0..count {
			let mut key_file =
				KeyFileContent::new(
					KeyFileCrypto::new_pbkdf2(
						FromHex::from_hex("5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46").unwrap(),
						H128::from_str("6087dab2f9fdbbfaddc31a909735c1e6").unwrap(),
						H256::from_str("ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd").unwrap(),
						H256::from_str("517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2").unwrap(),
						262144,
						32));
			key_file.account = Some(x!(i as u64));
			result.push(key_file.id.clone());
			write_sstore.import_key(key_file).unwrap();
		}
		result
	}

	#[test]
	fn can_get() {
		let temp = RandomTempPath::create_dir();
		let key_id = {
			use keys::directory::{KeyFileContent, KeyFileCrypto};
			let mut write_sstore = SecretStore::new_test(&temp);
			write_sstore.directory.save(
				KeyFileContent::new(
					KeyFileCrypto::new_pbkdf2(
						FromHex::from_hex("5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46").unwrap(),
						H128::from_str("6087dab2f9fdbbfaddc31a909735c1e6").unwrap(),
						H256::from_str("ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd").unwrap(),
						H256::from_str("517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2").unwrap(),
						262144,
						32)))
				.unwrap()
		};
		let sstore = SecretStore::new_test(&temp);
		if let Err(e) = sstore.get::<Bytes>(&key_id, "testpassword") {
			panic!("got no key: {:?}", e);
		}
	}

	#[test]
	fn can_delete() {
		let temp = RandomTempPath::create_dir();
		let keys = pregenerate_keys(&temp, 5);

		let mut sstore = SecretStore::new_test(&temp);
		sstore.delete(&keys[2]);

		assert_eq!(4, sstore.directory.list().unwrap().len())
	}

	#[test]
	fn can_import_account() {
		use keys::directory::{KeyFileContent, KeyFileCrypto};
		let temp = RandomTempPath::create_dir();
		let mut key_file =
			KeyFileContent::new(
				KeyFileCrypto::new_pbkdf2(
					FromHex::from_hex("5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46").unwrap(),
					H128::from_str("6087dab2f9fdbbfaddc31a909735c1e6").unwrap(),
					H256::from_str("ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd").unwrap(),
					H256::from_str("517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2").unwrap(),
					262144,
					32));
		key_file.account = Some(Address::from_str("3f49624084b67849c7b4e805c5988c21a430f9d9").unwrap());

		let mut sstore = SecretStore::new_test(&temp);

		sstore.import_key(key_file).unwrap();

		assert_eq!(1, sstore.accounts().unwrap().len());
		assert!(sstore.account(&Address::from_str("3f49624084b67849c7b4e805c5988c21a430f9d9").unwrap()).is_some());
	}

	#[test]
	fn can_list_accounts() {
		let temp = RandomTempPath::create_dir();
		pregenerate_accounts(&temp, 30);
		let sstore = SecretStore::new_test(&temp);
		let accounts = sstore.accounts().unwrap();
		assert_eq!(30, accounts.len());
	}
}
