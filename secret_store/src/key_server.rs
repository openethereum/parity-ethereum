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

use ethcrypto;
use ethkey;
use super::acl_storage::AclStorage;
use super::key_storage::KeyStorage;
use traits::KeyServer;
use types::all::{Error, RequestSignature, DocumentAddress, DocumentEncryptedKey};

/// Secret store key server implementation
pub struct KeyServerImpl<T: AclStorage, U: KeyStorage> {
	acl_storage: T,
	key_storage: U,
}

impl<T, U> KeyServerImpl<T, U> where T: AclStorage, U: KeyStorage {
	/// Create new key server instance
	pub fn new(acl_storage: T, key_storage: U) -> Self {
		KeyServerImpl {
			acl_storage: acl_storage,
			key_storage: key_storage,
		}
	}
}

impl<T, U> KeyServer for KeyServerImpl<T, U> where T: AclStorage, U: KeyStorage {
	fn document_key(&self, signature: &RequestSignature, document: &DocumentAddress) -> Result<DocumentEncryptedKey, Error> {
		// recover requestor' public key from signature
		let public = ethkey::recover(signature, document)
			.map_err(|_| Error::BadSignature)?;

		// check that requestor has access to the document
		if !self.acl_storage.check(&public, document)? {
			return Err(Error::AccessDenied);
		}

		// read unencrypted document key
		let document_key = self.key_storage.get(document)?;
		// encrypt document key with requestor public key
		let document_key = ethcrypto::ecies::encrypt_single_message(&public, &document_key)
			.map_err(|err| Error::Internal(format!("Error encrypting document key: {}", err)))?;
		Ok(document_key)
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use ethcrypto;
	use ethkey::{self, Secret};
	use acl_storage::DummyAclStorage;
	use key_storage::KeyStorage;
	use key_storage::tests::DummyKeyStorage;
	use super::super::{Error, RequestSignature, DocumentAddress};
	use super::{KeyServer, KeyServerImpl};

	const DOCUMENT1: &'static str = "0000000000000000000000000000000000000000000000000000000000000001";
	const DOCUMENT2: &'static str = "0000000000000000000000000000000000000000000000000000000000000002";
	const KEY1: &'static str = "key1";
	const PRIVATE1: &'static str = "03055e18a8434dcc9061cc1b81c4ef84dc7cf4574d755e52cdcf0c8898b25b11";
	const PUBLIC2: &'static str = "dfe62f56bb05fbd85b485bac749f3410309e24b352bac082468ce151e9ddb94fa7b5b730027fe1c7c5f3d5927621d269f91aceb5caa3c7fe944677a22f88a318";
	const PRIVATE2: &'static str = "0eb3816f4f705fa0fd952fb27b71b8c0606f09f4743b5b65cbc375bd569632f2";

	fn create_key_server() -> KeyServerImpl<DummyAclStorage, DummyKeyStorage> {
		let acl_storage = DummyAclStorage::default();
		let key_storage = DummyKeyStorage::default();
		key_storage.insert(DOCUMENT1.into(), KEY1.into()).unwrap();
		acl_storage.prohibit(PUBLIC2.into(), DOCUMENT1.into());
		KeyServerImpl::new(acl_storage, key_storage)
	}

	fn make_signature(secret: &str, document: &'static str) -> RequestSignature {
		let secret = Secret::from_str(secret).unwrap();
		let document: DocumentAddress = document.into();
		ethkey::sign(&secret, &document).unwrap()
	}

	#[test]
	fn document_key_succeeds() {
		let key_server = create_key_server();
		let signature = make_signature(PRIVATE1, DOCUMENT1);
		let document_key = key_server.document_key(&signature, &DOCUMENT1.into()).unwrap();
		let document_key = ethcrypto::ecies::decrypt_single_message(&Secret::from_str(PRIVATE1).unwrap(), &document_key);
		assert_eq!(document_key, Ok(KEY1.into()));
	}

	#[test]
	fn document_key_fails_when_bad_signature() {
		let key_server = create_key_server();
		let signature = RequestSignature::default();
		let document_key = key_server.document_key(&signature, &DOCUMENT1.into());
		assert_eq!(document_key, Err(Error::BadSignature));
	}

	#[test]
	fn document_key_fails_when_acl_check_fails() {
		let key_server = create_key_server();
		let signature = make_signature(PRIVATE2, DOCUMENT1);
		let document_key = key_server.document_key(&signature, &DOCUMENT1.into());
		assert_eq!(document_key, Err(Error::AccessDenied));
	}

	#[test]
	fn document_key_fails_when_document_not_found() {
		let key_server = create_key_server();
		let signature = make_signature(PRIVATE1, DOCUMENT2);
		let document_key = key_server.document_key(&signature, &DOCUMENT2.into());
		assert_eq!(document_key, Err(Error::DocumentNotFound));
	}
}
