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

//! SecretStore-specific rpc interface.

use std::collections::BTreeSet;
use jsonrpc_core::Result;

use v1::types::{H160, H256, H512, Bytes};

build_rpc_trait! {
	/// Parity-specific rpc interface.
	pub trait SecretStore {
		/// Encrypt data with key, received from secret store.
		/// Arguments: `account`, `password`, `key`, `data`.
		#[rpc(name = "secretstore_encrypt")]
		fn encrypt(&self, H160, String, Bytes, Bytes) -> Result<Bytes>;

		/// Decrypt data with key, received from secret store.
		/// Arguments: `account`, `password`, `key`, `data`.
		#[rpc(name = "secretstore_decrypt")]
		fn decrypt(&self, H160, String, Bytes, Bytes) -> Result<Bytes>;

		/// Decrypt data with shadow key, received from secret store.
		/// Arguments: `account`, `password`, `decrypted_secret`, `common_point`, `decrypt_shadows`, `data`.
		#[rpc(name = "secretstore_shadowDecrypt")]
		fn shadow_decrypt(&self, H160, String, H512, H512, Vec<Bytes>, Bytes) -> Result<Bytes>;

		/// Calculates the hash (keccak256) of servers set for using in ServersSetChange session.
		/// Returned hash must be signed later by using `secretstore_signRawHash` method.
		/// Arguments: `servers_set`.
		#[rpc(name = "secretstore_serversSetHash")]
		fn servers_set_hash(&self, BTreeSet<H512>) -> Result<H256>;

		/// Generate recoverable ECDSA signature of raw hash.
		/// Passed hash is treated as an input to the `sign` function (no prefixes added, no hash function is applied).
		/// Arguments: `account`, `password`, `raw_hash`.
		#[rpc(name = "secretstore_signRawHash")]
		fn sign_raw_hash(&self, H160, String, H256) -> Result<Bytes>;
	}
}
