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

//! Encryption providers.

use std::io::Read;
use std::iter::repeat;
use rand::{Rng, OsRng};
use bigint::hash::H256;
use ethjson;
use ethkey::{sign, Random, Generator, Public, Secret};
use ethcrypto;
use futures::Future;
use fetch::{Fetch, Method as FetchMethod, Client as FetchClient};
use bytes::{Bytes, ToPretty};
use error::{Error as EthcoreError, PrivateTransactionError};
use util::Address;

/// Initialization vector length.
const INIT_VEC_LEN: usize = 16;

/// Trait for encryption/decryption operations.
pub trait Encryptor: Send + Sync + 'static {
	/// Generate unique contract key && encrypt passed data. Encryption can only be performed once.
	fn encrypt(&self, contract_address: &Address, plain_data: &[u8]) -> Result<Bytes, EthcoreError>;

	/// Decrypt data using previously generated contract key.
	fn decrypt(&self, requester: &Secret, contract_address: &Address, cypher: &[u8]) -> Result<Bytes, EthcoreError>;
}

/// SecretStore-based encryption/decryption operations.
pub struct SecretStoreEncryptor {
	client: FetchClient,
	base_url: String,
	threshold: u32,
}

impl SecretStoreEncryptor {
	/// Create new encryptor.
	pub fn new() -> Result<Self, EthcoreError> {
		Ok(SecretStoreEncryptor {
			client: FetchClient::new()
				.map_err(|e| EthcoreError::PrivateTransaction(PrivateTransactionError::Encrypt(format!("{}", e))))?,
			base_url: "http://localhost:8082".into(),
			threshold: 0,
		})
	}

	/// Ask secret store for key && decrypt the key.
	fn retrieve_key(&self, url_suffix: &str, method: FetchMethod, contract_address: &Address, requester: &Secret) -> Result<Bytes, EthcoreError> {
		// key id in SS is H256 && we have H160 here => expand with assitional zeros
		let mut contract_address_extended = Vec::with_capacity(32);
		contract_address_extended.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
		contract_address_extended.extend_from_slice(&**contract_address);

		// sign key id so that SS will know that we have an access to the key
		let contract_address_signature = sign(requester, &H256::from_slice(&contract_address_extended))?;

		// prepare request url
		let url = format!("{}/{}/{}{}",
				self.base_url,
				contract_address_extended.to_hex(),
				contract_address_signature,
				url_suffix,
			);

		// send HTTP request
		let mut response = self.client.fetch_with_abort(&url, method, Default::default()).wait()
			.map_err(|e| EthcoreError::PrivateTransaction(PrivateTransactionError::Encrypt(format!("{}", e))))?;
		if !response.is_success() {
			return Err(EthcoreError::PrivateTransaction(PrivateTransactionError::Encrypt(response.status().canonical_reason().unwrap_or("unknown").into())));
		}

		// read HTTP response
		let mut result = String::new();
		response.read_to_string(&mut result)?;

		// response is JSON string (which is, in turn, hex-encoded, encrypted Public)
		let result_len = result.len();
		if result_len == 0 || &result[0..1] != "\"" || &result[result_len - 1..result_len] != "\"" {
			return Err(EthcoreError::PrivateTransaction(PrivateTransactionError::Encrypt(format!("Invalid SecretStore response: {}", result))));
		}
		let encrypted_bytes: ethjson::bytes::Bytes = result[1..result_len-1].parse().map_err(|e| EthcoreError::PrivateTransaction(PrivateTransactionError::Encrypt(e)))?;

		// decrypt Public
		let decrypted_bytes = ethcrypto::ecies::decrypt(requester, &ethcrypto::DEFAULT_MAC, &encrypted_bytes).unwrap();
		let key = Public::from_slice(&decrypted_bytes);

		// and now take x coordinate of Public as a key
		Ok((*key)[..INIT_VEC_LEN].into())
	}

	/// Generate random initialization vector.
	fn initialization_vector() -> [u8; INIT_VEC_LEN] {
		let mut result = [0u8; INIT_VEC_LEN];
		let mut rng = OsRng::new().unwrap();
		rng.fill_bytes(&mut result);
		result
	}
}

impl Encryptor for SecretStoreEncryptor {
	fn encrypt(&self, contract_address: &Address, plain_data: &[u8]) -> Result<Bytes, EthcoreError> {
		// requester here is only used to encrypt response
		let requester = Random.generate()?;

		// generate new key
		let key = self.retrieve_key(&format!("/{}", self.threshold), FetchMethod::Post, contract_address, requester.secret())?;

		// encrypt data
		let iv = Self::initialization_vector();
		let mut cypher = Vec::with_capacity(plain_data.len() + iv.len());
		cypher.extend(repeat(0).take(plain_data.len()));
		ethcrypto::aes::encrypt(&key, &iv, plain_data, &mut cypher);
		cypher.extend_from_slice(&iv);

		Ok(cypher)
	}

	/// Decrypt data using previously generated contract key.
	fn decrypt(&self, requester: &Secret, contract_address: &Address, cypher: &[u8]) -> Result<Bytes, EthcoreError> {
		// initialization vector takes INIT_VEC_LEN bytes
		let cypher_len = cypher.len();
		if cypher_len < INIT_VEC_LEN {
			return Err(EthcoreError::PrivateTransaction(PrivateTransactionError::Decrypt("Invalid cypher".into())));
		}

		// retrieve existing key
		let key = self.retrieve_key("", FetchMethod::Get, contract_address, requester)?;

		// use symmetric decryption to decrypt document
		let (cypher, iv) = cypher.split_at(cypher_len - INIT_VEC_LEN);
		let mut plain_data = Vec::with_capacity(cypher_len - INIT_VEC_LEN);
		plain_data.extend(repeat(0).take(cypher_len - INIT_VEC_LEN));
		ethcrypto::aes::decrypt(&key, &iv, cypher, &mut plain_data);

		Ok(plain_data)
	}
}

/// Dummy encryptor.
#[derive(Default)]
pub struct DummyEncryptor;

impl Encryptor for DummyEncryptor {
	fn encrypt(&self, _contract_address: &Address, data: &[u8]) -> Result<Bytes, EthcoreError> {
		Ok(data.to_vec())
	}

	/// Decrypt data using previously generated contract key.
	fn decrypt(&self, _requester: &Secret, _contract_address: &Address, data: &[u8]) -> Result<Bytes, EthcoreError> {
		Ok(data.to_vec())
	}
}

#[cfg(test)]
pub mod tests {
	use ethkey::{Random, Generator};
	use super::{Encryptor, SecretStoreEncryptor};

	#[test]
	fn my_test() {
		let encryptor = SecretStoreEncryptor::new().unwrap();

		let plain_data = vec![42];
		let cypher = encryptor.encrypt(&Default::default(), &plain_data).unwrap();
		let _decrypted_data = encryptor.decrypt(Random.generate().unwrap().secret(), &Default::default(), &cypher).unwrap();
	}
}
