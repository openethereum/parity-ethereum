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

pub mod http_listener;
pub mod service_contract;
pub mod service_contract_listener;

use std::collections::BTreeSet;
use std::sync::Arc;
use traits::{ServerKeyGenerator, DocumentKeyServer, MessageSigner, AdminSessionsServer, KeyServer};
use types::all::{Error, Public, MessageHash, EncryptedMessageSignature, RequestSignature, ServerKeyId,
	EncryptedDocumentKey, EncryptedDocumentKeyShadow, NodeId};

pub struct Listener {
	key_server: Arc<KeyServer>,
	_http: Option<http_listener::KeyServerHttpListener>,
	_contract: Option<Arc<service_contract_listener::ServiceContractListener>>,
}

impl Listener {
	pub fn new(key_server: Arc<KeyServer>, http: Option<http_listener::KeyServerHttpListener>, contract: Option<Arc<service_contract_listener::ServiceContractListener>>) -> Self {
		Self {
			key_server: key_server,
			_http: http,
			_contract: contract,
		}
	}
}

impl KeyServer for Listener {}

impl ServerKeyGenerator for Listener {
	fn generate_key(&self, key_id: &ServerKeyId, signature: &RequestSignature, threshold: usize) -> Result<Public, Error> {
		self.key_server.generate_key(key_id, signature, threshold)
	}
}

impl DocumentKeyServer for Listener {
	fn store_document_key(&self, key_id: &ServerKeyId, signature: &RequestSignature, common_point: Public, encrypted_document_key: Public) -> Result<(), Error> {
		self.key_server.store_document_key(key_id, signature, common_point, encrypted_document_key)
	}

	fn generate_document_key(&self, key_id: &ServerKeyId, signature: &RequestSignature, threshold: usize) -> Result<EncryptedDocumentKey, Error> {
		self.key_server.generate_document_key(key_id, signature, threshold)
	}

	fn restore_document_key(&self, key_id: &ServerKeyId, signature: &RequestSignature) -> Result<EncryptedDocumentKey, Error> {
		self.key_server.restore_document_key(key_id, signature)
	}

	fn restore_document_key_shadow(&self, key_id: &ServerKeyId, signature: &RequestSignature) -> Result<EncryptedDocumentKeyShadow, Error> {
		self.key_server.restore_document_key_shadow(key_id, signature)
	}
}

impl MessageSigner for Listener {
	fn sign_message(&self, key_id: &ServerKeyId, signature: &RequestSignature, message: MessageHash) -> Result<EncryptedMessageSignature, Error> {
		self.key_server.sign_message(key_id, signature, message)
	}
}

impl AdminSessionsServer for Listener {
	fn change_servers_set(&self, old_set_signature: RequestSignature, new_set_signature: RequestSignature, new_servers_set: BTreeSet<NodeId>) -> Result<(), Error> {
		self.key_server.change_servers_set(old_set_signature, new_set_signature, new_servers_set)
	}
}