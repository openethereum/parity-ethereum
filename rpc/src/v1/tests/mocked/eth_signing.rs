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

use std::str::FromStr;
use std::sync::Arc;
use jsonrpc_core::{IoHandler, to_value, Success};
use v1::impls::EthSigningQueueClient;
use v1::traits::{EthSigning, Ethcore};
use v1::helpers::{SignerService, SigningQueue};
use v1::types::{H256 as RpcH256, H520 as RpcH520, Bytes};
use v1::tests::helpers::TestMinerService;
use v1::tests::mocked::ethcore;

use util::{Address, FixedHash, Uint, U256, H256, H520};
use ethcore::account_provider::AccountProvider;
use ethcore::client::TestBlockChainClient;
use ethcore::transaction::{Transaction, Action};
use ethstore::ethkey::{Generator, Random};
use serde_json;

struct EthSigningTester {
	pub signer: Arc<SignerService>,
	pub client: Arc<TestBlockChainClient>,
	pub miner: Arc<TestMinerService>,
	pub accounts: Arc<AccountProvider>,
	pub io: IoHandler,
}

impl Default for EthSigningTester {
	fn default() -> Self {
		let signer = Arc::new(SignerService::new_test(None));
		let client = Arc::new(TestBlockChainClient::default());
		let miner = Arc::new(TestMinerService::default());
		let accounts = Arc::new(AccountProvider::transient_provider());
		let io = IoHandler::new();
		io.add_delegate(EthSigningQueueClient::new(&signer, &client, &miner, &accounts).to_delegate());

		EthSigningTester {
			signer: signer,
			client: client,
			miner: miner,
			accounts: accounts,
			io: io,
		}
	}
}

fn eth_signing() -> EthSigningTester {
	EthSigningTester::default()
}

#[test]
fn should_add_sign_to_queue() {
	// given
	let tester = eth_signing();
	let address = Address::random();
	assert_eq!(tester.signer.requests().len(), 0);

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sign",
		"params": [
			""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"0x0000000000000000000000000000000000000000000000000000000000000005"
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","id":1}"#;

	// then
	let async_result = tester.io.handle_request(&request).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);
	// respond
	tester.signer.request_confirmed(U256::from(1), Ok(to_value(&RpcH520::from(H520::default()))));
	assert!(async_result.on_result(move |res| {
		assert_eq!(res, response.to_owned());
	}));
}

#[test]
fn should_post_sign_to_queue() {
	// given
	let tester = eth_signing();
	let address = Address::random();
	assert_eq!(tester.signer.requests().len(), 0);

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_postSign",
		"params": [
			""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"0x0000000000000000000000000000000000000000000000000000000000000005"
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 1);
}

#[test]
fn should_check_status_of_request() {
	// given
	let tester = eth_signing();
	let address = Address::random();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_postSign",
		"params": [
			""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"0x0000000000000000000000000000000000000000000000000000000000000005"
		],
		"id": 1
	}"#;
	tester.io.handle_request_sync(&request).expect("Sent");

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_checkRequest",
		"params": ["0x1"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn should_check_status_of_request_when_its_resolved() {
	// given
	let tester = eth_signing();
	let address = Address::random();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_postSign",
		"params": [
			""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"0x0000000000000000000000000000000000000000000000000000000000000005"
		],
		"id": 1
	}"#;
	tester.io.handle_request_sync(&request).expect("Sent");
	tester.signer.request_confirmed(U256::from(1), Ok(to_value(&"Hello World!")));

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_checkRequest",
		"params": ["0x1"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"Hello World!","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn should_sign_if_account_is_unlocked() {
	// given
	let tester = eth_signing();
	let hash: H256 = 5.into();
	let acc = tester.accounts.new_account("test").unwrap();
	tester.accounts.unlock_account_permanently(acc, "test".into()).unwrap();

	let signature = tester.accounts.sign(acc, None, hash).unwrap();

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sign",
		"params": [
			""#.to_owned() + format!("0x{:?}", acc).as_ref() + r#"",
			""# + format!("0x{:?}", hash).as_ref() + r#""
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{}", signature).as_ref() + r#"","id":1}"#;
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 0);
}

#[test]
fn should_add_transaction_to_queue() {
	// given
	let tester = eth_signing();
	let address = Address::random();
	assert_eq!(tester.signer.requests().len(), 0);

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000","id":1}"#;

	// then
	let async_result = tester.io.handle_request(&request).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);
	// respond
	tester.signer.request_confirmed(U256::from(1), Ok(to_value(&RpcH256::from(H256::default()))));
	assert!(async_result.on_result(move |res| {
		assert_eq!(res, response.to_owned());
	}));
}

#[test]
fn should_dispatch_transaction_if_account_is_unlock() {
	// given
	let tester = eth_signing();
	let acc = tester.accounts.new_account("test").unwrap();
	tester.accounts.unlock_account_permanently(acc, "test".into()).unwrap();

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	let signature = tester.accounts.sign(acc, None, t.hash()).unwrap();
	let t = t.with_signature(signature);

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:?}", acc).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn should_decrypt_message_if_account_is_unlocked() {
	// given
	let tester = eth_signing();
	let ethcore = ethcore::Dependencies::new();
	tester.io.add_delegate(ethcore.client(None).to_delegate());
	let (address, public) = tester.accounts.new_account_and_public("test").unwrap();
	tester.accounts.unlock_account_permanently(address, "test".into()).unwrap();


	// First encrypt message
	let request = format!("{}0x{:?}{}",
		r#"{"jsonrpc": "2.0", "method": "ethcore_encryptMessage", "params":[""#,
		public,
		r#"", "0x01020304"], "id": 1}"#
	);
	let encrypted: Success = serde_json::from_str(&tester.io.handle_request_sync(&request).unwrap()).unwrap();

	// then call decrypt
	let request = format!("{}{:?}{}{:?}{}",
		r#"{"jsonrpc": "2.0", "method": "ethcore_decryptMessage", "params":["0x"#,
		address,
		r#"","#,
		encrypted.result,
		r#"], "id": 1}"#
	);
	println!("Request: {:?}", request);
	let response = r#"{"jsonrpc":"2.0","result":"0x01020304","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));
}

#[test]
fn should_add_decryption_to_the_queue() {
	// given
	let tester = eth_signing();
	let acc = Random.generate().unwrap();
	assert_eq!(tester.signer.requests().len(), 0);

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "ethcore_decryptMessage",
		"params": ["0x"#.to_owned() + &format!("{:?}", acc.address()) + r#"",
		"0x012345"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0102","id":1}"#;

	// then
	let async_result = tester.io.handle_request(&request).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);
	// respond
	tester.signer.request_confirmed(U256::from(1), Ok(to_value(Bytes(vec![0x1, 0x2]))));
	assert!(async_result.on_result(move |res| {
		assert_eq!(res, response.to_owned());
	}));
}
