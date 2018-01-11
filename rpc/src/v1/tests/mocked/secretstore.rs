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

use std::sync::Arc;

use ethcore::account_provider::AccountProvider;
use ethkey::{KeyPair, Signature, verify_public};

use serde_json;
use jsonrpc_core::{IoHandler, Success};
use v1::metadata::Metadata;
use v1::SecretStoreClient;
use v1::traits::secretstore::SecretStore;
use v1::helpers::secretstore::ordered_servers_keccak;
use v1::types::H256;

struct Dependencies {
	pub accounts: Arc<AccountProvider>,
}

impl Dependencies {
	pub fn new() -> Self {
		Dependencies {
			accounts: Arc::new(AccountProvider::transient_provider()),
		}
	}

	pub fn client(&self) -> SecretStoreClient {
		SecretStoreClient::new(&Some(self.accounts.clone()))
	}

	fn default_client(&self) -> IoHandler<Metadata> {
		let mut io = IoHandler::default();
		io.extend_with(self.client().to_delegate());
		io
	}
}

#[test]
fn rpc_secretstore_encrypt_and_decrypt() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	// insert new account
	let secret = "c1f1cfe279a5c350d13795bce162941967340c8a228e6ba175489afc564a5bef".parse().unwrap();
	deps.accounts.insert_account(secret, "password").unwrap();

	// execute encryption request
	let encryption_request = r#"{"jsonrpc": "2.0", "method": "secretstore_encrypt", "params":[
		"0x5c2f3b4ec0c2234f8358697edc8b82a62e3ac995", "password",
		"0x0440262acc06f1e13cb11b34e792cdf698673a16bb812163cb52689ac34c94ae47047b58f58d8b596d21ac7b03a55896132d07a7dc028b2dad88f6c5a90623fa5b30ff4b1ba385a98c970432d13417cf6d7facd62f86faaef15ca993735890da0cb3e417e2740fc72de7501eef083a12dd5a9ebe513b592b1740848576a936a1eb88fc553fc624b1cae41a0a4e074e34e2aaae686709f08d70e505c5acba12ef96017e89be675a2adb07c72c4e95814fbf",
		"0xdeadbeef"
	], "id": 1}"#;
	let encryption_response = io.handle_request_sync(encryption_request).unwrap();
	let encryption_response: Success = serde_json::from_str(&encryption_response).unwrap();

	// execute decryption request
	let decryption_request_left = r#"{"jsonrpc": "2.0", "method": "secretstore_decrypt", "params":[
		"0x5c2f3b4ec0c2234f8358697edc8b82a62e3ac995", "password",
		"0x0440262acc06f1e13cb11b34e792cdf698673a16bb812163cb52689ac34c94ae47047b58f58d8b596d21ac7b03a55896132d07a7dc028b2dad88f6c5a90623fa5b30ff4b1ba385a98c970432d13417cf6d7facd62f86faaef15ca993735890da0cb3e417e2740fc72de7501eef083a12dd5a9ebe513b592b1740848576a936a1eb88fc553fc624b1cae41a0a4e074e34e2aaae686709f08d70e505c5acba12ef96017e89be675a2adb07c72c4e95814fbf",""#;
	let decryption_request_mid = encryption_response.result.as_str().unwrap();
	let decryption_request_right = r#""
		], "id": 2}"#;
	let decryption_request = decryption_request_left.to_owned() + decryption_request_mid + decryption_request_right;
	let decryption_response = io.handle_request_sync(&decryption_request).unwrap();
	assert_eq!(decryption_response, r#"{"jsonrpc":"2.0","result":"0xdeadbeef","id":2}"#);
}

#[test]
fn rpc_secretstore_shadow_decrypt() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	// insert new account
	let secret = "82758356bf46b42710d3946a8efa612b7bf5e125e4d49f28facf1139db4a46f4".parse().unwrap();
	deps.accounts.insert_account(secret, "password").unwrap();

	// execute decryption request
	let decryption_request = r#"{"jsonrpc": "2.0", "method": "secretstore_shadowDecrypt", "params":[
		"0x00dfE63B22312ab4329aD0d28CaD8Af987A01932", "password",
		"0x843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91",
		"0x07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3",
		["0x049ce50bbadb6352574f2c59742f78df83333975cbd5cbb151c6e8628749a33dc1fa93bb6dffae5994e3eb98ae859ed55ee82937538e6adb054d780d1e89ff140f121529eeadb1161562af9d3342db0008919ca280a064305e5a4e518e93279de7a9396fe5136a9658e337e8e276221248c381c5384cd1ad28e5921f46ff058d5fbcf8a388fc881d0dd29421c218d51761"],
		"0x2ddec1f96229efa2916988d8b2a82a47ef36f71c"
	], "id": 1}"#;
	let decryption_response = io.handle_request_sync(&decryption_request).unwrap();
	assert_eq!(decryption_response, r#"{"jsonrpc":"2.0","result":"0xdeadbeef","id":1}"#);
}

#[test]
fn rpc_secretstore_servers_set_hash() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	// execute hashing request
	let hashing_request = r#"{"jsonrpc": "2.0", "method": "secretstore_serversSetHash", "params":[
		["0x843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91",
		 "0x07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3"]
	], "id": 1}"#;
	let hashing_response = io.handle_request_sync(&hashing_request).unwrap();
	let hashing_response = hashing_response.replace(r#"{"jsonrpc":"2.0","result":"0x"#, "");
	let hashing_response = hashing_response.replace(r#"","id":1}"#, "");
	let hash: H256 = hashing_response.parse().unwrap();

	let servers_set_keccak = ordered_servers_keccak(vec![
		"843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91".parse().unwrap(),
		"07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3".parse().unwrap()
	].into_iter().collect());
	assert_eq!(hash, servers_set_keccak);
}

#[test]
fn rpc_secretstore_sign_raw_hash() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	// insert new account
	let secret = "82758356bf46b42710d3946a8efa612b7bf5e125e4d49f28facf1139db4a46f4".parse().unwrap();
	let key_pair = KeyPair::from_secret(secret).unwrap();
	deps.accounts.insert_account(key_pair.secret().clone(), "password").unwrap();

	// execute signing request
	let signing_request = r#"{"jsonrpc": "2.0", "method": "secretstore_signRawHash", "params":[
		"0x00dfE63B22312ab4329aD0d28CaD8Af987A01932", "password", "0x0000000000000000000000000000000000000000000000000000000000000001"
	], "id": 1}"#;
	let signing_response = io.handle_request_sync(&signing_request).unwrap();
	let signing_response = signing_response.replace(r#"{"jsonrpc":"2.0","result":"0x"#, "");
	let signing_response = signing_response.replace(r#"","id":1}"#, "");
	let signature: Signature = signing_response.parse().unwrap();

	let hash = "0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap();
	assert!(verify_public(key_pair.public(), &signature, &hash).unwrap());
}
