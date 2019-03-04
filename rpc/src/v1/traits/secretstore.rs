// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! SecretStore-specific rpc interface.

use std::collections::BTreeSet;

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use ethereum_types::{H160, H256, H512};
use ethkey::Password;
use v1::types::{Bytes, EncryptedDocumentKey};

/// Parity-specific rpc interface.
#[rpc]
pub trait SecretStore {
	/// Generate document key to store in secret store.
	/// Arguments: `account`, `password`, `server_key_public`.
	#[rpc(name = "secretstore_generateDocumentKey")]
	fn generate_document_key(&self, H160, Password, H512) -> Result<EncryptedDocumentKey>;

	/// Encrypt data with key, received from secret store.
	/// Arguments: `account`, `password`, `key`, `data`.
	#[rpc(name = "secretstore_encrypt")]
	fn encrypt(&self, H160, Password, Bytes, Bytes) -> Result<Bytes>;

	/// Decrypt data with key, received from secret store.
	/// Arguments: `account`, `password`, `key`, `data`.
	#[rpc(name = "secretstore_decrypt")]
	fn decrypt(&self, H160, Password, Bytes, Bytes) -> Result<Bytes>;

	/// Decrypt data with shadow key, received from secret store.
	/// Arguments: `account`, `password`, `decrypted_secret`, `common_point`, `decrypt_shadows`, `data`.
	#[rpc(name = "secretstore_shadowDecrypt")]
	fn shadow_decrypt(&self, H160, Password, H512, H512, Vec<Bytes>, Bytes) -> Result<Bytes>;

	/// Calculates the hash (keccak256) of servers set for using in ServersSetChange session.
	/// Returned hash must be signed later by using `secretstore_signRawHash` method.
	/// Arguments: `servers_set`.
	#[rpc(name = "secretstore_serversSetHash")]
	fn servers_set_hash(&self, BTreeSet<H512>) -> Result<H256>;

	/// Generate recoverable ECDSA signature of raw hash.
	/// Passed hash is treated as an input to the `sign` function (no prefixes added, no hash function is applied).
	/// Arguments: `account`, `password`, `raw_hash`.
	#[rpc(name = "secretstore_signRawHash")]
	fn sign_raw_hash(&self, H160, Password, H256) -> Result<Bytes>;
}
