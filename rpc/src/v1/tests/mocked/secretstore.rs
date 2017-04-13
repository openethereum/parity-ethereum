use std::sync::Arc;

use ethcore::account_provider::AccountProvider;

use serde_json;
use jsonrpc_core::{IoHandler, Success};
use v1::metadata::Metadata;
use v1::SecretStoreClient;
use v1::traits::secretstore::SecretStore;

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

	// insert new account && unlock it
	let secret = "c1f1cfe279a5c350d13795bce162941967340c8a228e6ba175489afc564a5bef".parse().unwrap();
	let address = deps.accounts.insert_account(secret, "password").unwrap();
	deps.accounts.unlock_account_permanently(address, "password".into()).unwrap();

	// execute encryption request
	let encryption_request = r#"{"jsonrpc": "2.0", "method": "secretstore_encrypt", "params":[
		"0x5c2f3b4ec0c2234f8358697edc8b82a62e3ac995",
		"0x0440262acc06f1e13cb11b34e792cdf698673a16bb812163cb52689ac34c94ae47047b58f58d8b596d21ac7b03a55896132d07a7dc028b2dad88f6c5a90623fa5b30ff4b1ba385a98c970432d13417cf6d7facd62f86faaef15ca993735890da0cb3e417e2740fc72de7501eef083a12dd5a9ebe513b592b1740848576a936a1eb88fc553fc624b1cae41a0a4e074e34e2aaae686709f08d70e505c5acba12ef96017e89be675a2adb07c72c4e95814fbf",
		"0xdeadbeef"
	], "id": 1}"#;
	let encryption_response = io.handle_request_sync(encryption_request).unwrap();
	let encryption_response: Success = serde_json::from_str(&encryption_response).unwrap();

	// execute decryption request
	let decryption_request_left = r#"{"jsonrpc": "2.0", "method": "secretstore_decrypt", "params":[
		"0x5c2f3b4ec0c2234f8358697edc8b82a62e3ac995",
		"0x0440262acc06f1e13cb11b34e792cdf698673a16bb812163cb52689ac34c94ae47047b58f58d8b596d21ac7b03a55896132d07a7dc028b2dad88f6c5a90623fa5b30ff4b1ba385a98c970432d13417cf6d7facd62f86faaef15ca993735890da0cb3e417e2740fc72de7501eef083a12dd5a9ebe513b592b1740848576a936a1eb88fc553fc624b1cae41a0a4e074e34e2aaae686709f08d70e505c5acba12ef96017e89be675a2adb07c72c4e95814fbf",""#;
	let decryption_request_mid = encryption_response.result.as_str().unwrap();
	let decryption_request_right = r#""
		], "id": 2}"#;
	let decryption_request = decryption_request_left.to_owned() + decryption_request_mid + decryption_request_right;
	let decryption_response = io.handle_request_sync(&decryption_request).unwrap();
	assert_eq!(decryption_response, r#"{"jsonrpc":"2.0","result":"0xdeadbeef","id":2}"#);
}
