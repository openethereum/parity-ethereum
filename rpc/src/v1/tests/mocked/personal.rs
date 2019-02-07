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

use std::sync::Arc;
use std::str::FromStr;

use bytes::ToPretty;
use ethereum_types::{U256, Address};
use accounts::AccountProvider;
use ethcore::client::TestBlockChainClient;
use jsonrpc_core::IoHandler;
use parking_lot::Mutex;
use types::transaction::{Action, Transaction};
use parity_runtime::Runtime;
use hash::keccak;

use v1::{PersonalClient, Personal, Metadata};
use v1::helpers::{nonce, eip191};
use v1::helpers::dispatch::{eth_data_hash, FullDispatcher};
use v1::tests::helpers::TestMinerService;
use v1::types::{EIP191Version, PresignedTransaction, H520};
use rustc_hex::ToHex;
use serde_json::to_value;
use ethkey::Secret;

struct PersonalTester {
	_runtime: Runtime,
	accounts: Arc<AccountProvider>,
	io: IoHandler<Metadata>,
	miner: Arc<TestMinerService>,
}

fn blockchain_client() -> Arc<TestBlockChainClient> {
	let client = TestBlockChainClient::new();
	Arc::new(client)
}

fn accounts_provider() -> Arc<AccountProvider> {
	Arc::new(AccountProvider::transient_provider())
}

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

fn setup() -> PersonalTester {
	setup_with(Config {
		allow_experimental_rpcs: true
	})
}

struct Config {
	pub allow_experimental_rpcs: bool,
}

fn setup_with(c: Config) -> PersonalTester {
	let runtime = Runtime::with_thread_count(1);
	let accounts = accounts_provider();
	let client = blockchain_client();
	let miner = miner_service();
	let reservations = Arc::new(Mutex::new(nonce::Reservations::new(runtime.executor())));

	let dispatcher = FullDispatcher::new(client, miner.clone(), reservations, 50);
	let personal = PersonalClient::new(&accounts, dispatcher, false, c.allow_experimental_rpcs);

	let mut io = IoHandler::default();
	io.extend_with(personal.to_delegate());

	let tester = PersonalTester {
		_runtime: runtime,
		accounts: accounts,
		io: io,
		miner: miner,
	};

	tester
}

#[test]
fn accounts() {
	let tester = setup();
	let address = tester.accounts.new_account(&"".into()).unwrap();
	let request = r#"{"jsonrpc": "2.0", "method": "personal_listAccounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[""#.to_owned() + &format!("0x{:x}", address) + r#""],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn new_account() {
	let tester = setup();
	let request = r#"{"jsonrpc": "2.0", "method": "personal_newAccount", "params": ["pass"], "id": 1}"#;

	let res = tester.io.handle_request_sync(request);

	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"","id":1}"#;

	assert_eq!(res, Some(response));
}

fn invalid_password_test(method: &str)
{
	let tester = setup();
	let address = tester.accounts.new_account(&"password123".into()).unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": ""#.to_owned() + method + r#"",
		"params": [{
			"from": ""# + format!("0x{:x}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}, "password321"],
		"id": 1
	}"#;

	let response = r#"{"jsonrpc":"2.0","error":{"code":-32021,"message":"Account password is invalid or account does not exist.","data":"SStore(InvalidPassword)"},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response.into()));
}

#[test]
fn sign() {
	let tester = setup();
	let address = tester.accounts.new_account(&"password123".into()).unwrap();
	let data = vec![5u8];

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_sign",
		"params": [
			""#.to_owned() + format!("0x{}", data.to_hex()).as_ref() + r#"",
			""# + format!("0x{:x}", address).as_ref() + r#"",
			"password123"
		],
		"id": 1
	}"#;

	let hash = eth_data_hash(data);
	let signature = H520(tester.accounts.sign(address, Some("password123".into()), hash).unwrap().into_electrum());
	let signature = format!("0x{:?}", signature);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + &signature + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response));
}

#[test]
fn sign_with_invalid_password() {
	let tester = setup();
	let address = tester.accounts.new_account(&"password123".into()).unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_sign",
		"params": [
			"0x0000000000000000000000000000000000000000000000000000000000000005",
			""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			""
		],
		"id": 1
	}"#;

	let response = r#"{"jsonrpc":"2.0","error":{"code":-32021,"message":"Account password is invalid or account does not exist.","data":"SStore(InvalidPassword)"},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response.into()));
}

#[test]
fn sign_transaction_with_invalid_password() {
	invalid_password_test("personal_signTransaction");
}

#[test]
fn sign_and_send_transaction_with_invalid_password() {
	invalid_password_test("personal_sendTransaction");
}

#[test]
fn send_transaction() {
	sign_and_send_test("personal_sendTransaction");
}

#[test]
fn sign_and_send_transaction() {
	sign_and_send_test("personal_signAndSendTransaction");
}

fn sign_and_send_test(method: &str) {
	let tester = setup();
	let address = tester.accounts.new_account(&"password123".into()).unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": ""#.to_owned() + method + r#"",
		"params": [{
			"from": ""# + format!("0x{:x}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}, "password123"],
		"id": 1
	}"#;

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	tester.accounts.unlock_account_temporarily(address, "password123".into()).unwrap();
	let signature = tester.accounts.sign(address, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:x}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response));

	tester.miner.increment_nonce(&address);

	let t = Transaction {
		nonce: U256::one(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	tester.accounts.unlock_account_temporarily(address, "password123".into()).unwrap();
	let signature = tester.accounts.sign(address, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:x}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response));
}

#[test]
fn ec_recover() {
	let tester = setup();
	let address = tester.accounts.new_account(&"password123".into()).unwrap();
	let data = vec![5u8];

	let hash = eth_data_hash(data.clone());
	let signature = H520(tester.accounts.sign(address, Some("password123".into()), hash).unwrap().into_electrum());
	let signature = format!("0x{:?}", signature);

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_ecRecover",
		"params": [
			""#.to_owned() + format!("0x{}", data.to_hex()).as_ref() + r#"",
			""# + &signature + r#""
		],
		"id": 1
	}"#;

	let address = format!("0x{:x}", address);
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + &address + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response.into()));
}

#[test]
fn ec_recover_invalid_signature() {
	let tester = setup();
	let data = vec![5u8];

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_ecRecover",
		"params": [
			""#.to_owned() + format!("0x{}", data.to_hex()).as_ref() + r#"",
			"0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
		],
		"id": 1
	}"#;

	let response = r#"{"jsonrpc":"2.0","error":{"code":-32055,"message":"Encryption error.","data":"InvalidSignature"},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response.into()));
}

#[test]
fn should_not_unlock_account_temporarily_if_allow_perm_is_disabled() {
	let tester = setup();
	let address = tester.accounts.new_account(&"password123".into()).unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_unlockAccount",
		"params": [
			""#.to_owned() + &format!("0x{:x}", address) + r#"",
			"password123",
			"0x100"
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Time-unlocking is not supported when permanent unlock is disabled.","data":"Use personal_sendTransaction or enable permanent unlocking, instead."},"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));

	assert!(tester.accounts.sign(address, None, Default::default()).is_err(), "Should not unlock account.");
}

#[test]
fn should_unlock_account_permanently() {
	let tester = setup();
	let address = tester.accounts.new_account(&"password123".into()).unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_unlockAccount",
		"params": [
			""#.to_owned() + &format!("0x{:x}", address) + r#"",
			"password123",
			null
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));
	assert!(tester.accounts.sign(address, None, Default::default()).is_ok(), "Should unlock account.");
}

#[test]
fn sign_eip191_with_validator() {
	let tester = setup();
	let address = tester.accounts.new_account(&"password123".into()).unwrap();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_sign191",
		"params": [
			"0x00",
			{
				"validator": ""#.to_owned() + &format!("0x{:x}", address) + r#"",
				"data": ""# + &format!("0x{:x}", keccak("hello world")) + r#""
			},
			""# + &format!("0x{:x}", address) + r#"",
			"password123"
		],
		"id": 1
	}"#;
	let with_validator = to_value(PresignedTransaction {
		validator: address.into(),
		data: keccak("hello world").to_vec().into()
	}).unwrap();
	let result = eip191::hash_message(EIP191Version::PresignedTransaction, with_validator).unwrap();
	let result = tester.accounts.sign(address, Some("password123".into()), result).unwrap().into_electrum();
	let expected = r#"{"jsonrpc":"2.0","result":""#.to_owned() +  &format!("0x{}", result.to_hex()) + r#"","id":1}"#;
	let response = tester.io.handle_request_sync(&request).unwrap();
	assert_eq!(response, expected)
}

#[test]
fn sign_eip191_structured_data() {
	let tester = setup();
	let secret: Secret = keccak("cow").into();
	let address = tester.accounts.insert_account(secret, &"lol".into()).unwrap();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_sign191",
		"params": [
			"0x01",
			{
				"primaryType": "Mail",
				"domain": {
					"name": "Ether Mail",
					"version": "1",
					"chainId": "0x1",
					"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
				},
				"message": {
					"from": {
						"name": "Cow",
						"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
					},
					"to": {
						"name": "Bob",
						"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
					},
					"contents": "Hello, Bob!"
				},
				"types": {
					"EIP712Domain": [
						{ "name": "name", "type": "string" },
						{ "name": "version", "type": "string" },
						{ "name": "chainId", "type": "uint256" },
						{ "name": "verifyingContract", "type": "address" }
					],
					"Person": [
						{ "name": "name", "type": "string" },
						{ "name": "wallet", "type": "address" }
					],
					"Mail": [
						{ "name": "from", "type": "Person" },
						{ "name": "to", "type": "Person" },
						{ "name": "contents", "type": "string" }
					]
				}
			},
			""#.to_owned() + &format!("0x{:x}", address) + r#"",
			"lol"
		],
		"id": 1
	}"#;
	let expected = r#"{"jsonrpc":"2.0","result":"0x4355c47d63924e8a72e509b65029052eb6c299d53a04e167c5775fd466751c9d07299936d304c153f6443dfa05f40ff007d72911b6f72307f996231605b915621c","id":1}"#;
	let response = tester.io.handle_request_sync(&request).unwrap();
	assert_eq!(response, expected)
}

#[test]
fn sign_structured_data() {
	let tester = setup();
	let secret: Secret = keccak("cow").into();
	let address = tester.accounts.insert_account(secret, &"lol".into()).unwrap();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_signTypedData",
		"params": [
			{
				"primaryType": "Mail",
				"domain": {
					"name": "Ether Mail",
					"version": "1",
					"chainId": "0x1",
					"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
				},
				"message": {
					"from": {
						"name": "Cow",
						"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
					},
					"to": {
						"name": "Bob",
						"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
					},
					"contents": "Hello, Bob!"
				},
				"types": {
					"EIP712Domain": [
						{ "name": "name", "type": "string" },
						{ "name": "version", "type": "string" },
						{ "name": "chainId", "type": "uint256" },
						{ "name": "verifyingContract", "type": "address" }
					],
					"Person": [
						{ "name": "name", "type": "string" },
						{ "name": "wallet", "type": "address" }
					],
					"Mail": [
						{ "name": "from", "type": "Person" },
						{ "name": "to", "type": "Person" },
						{ "name": "contents", "type": "string" }
					]
				}
			},
			""#.to_owned() + &format!("0x{:x}", address) + r#"",
			"lol"
		],
		"id": 1
	}"#;
	let expected = r#"{"jsonrpc":"2.0","result":"0x4355c47d63924e8a72e509b65029052eb6c299d53a04e167c5775fd466751c9d07299936d304c153f6443dfa05f40ff007d72911b6f72307f996231605b915621c","id":1}"#;
	let response = tester.io.handle_request_sync(&request).unwrap();
	assert_eq!(response, expected)
}

#[test]
fn should_disable_experimental_apis() {
	// given
	let tester = setup_with(Config {
		allow_experimental_rpcs: false,
	});

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_sign191",
		"params": [
			"0x01",
			{},
			"0x1234567891234567891234567891234567891234",
			"lol"
		],
		"id": 1
	}"#;
	let r1 = tester.io.handle_request_sync(&request).unwrap();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_signTypedData",
		"params": [
			{
				"types": {},
				"message": {},
				"domain": {
					"name": "",
					"version": "1",
					"chainId": "0x1",
					"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
				},
				"primaryType": ""
			},
			"0x1234567891234567891234567891234678912344",
			"lol"
		],
		"id": 1
	}"#;
	let r2 = tester.io.handle_request_sync(&request).unwrap();

	// then
	let expected = r#"{"jsonrpc":"2.0","error":{"code":-32071,"message":"This method is not part of the official RPC API yet (EIP-191). Run with `--jsonrpc-experimental` to enable it.","data":"See EIP: https://eips.ethereum.org/EIPS/eip-191"},"id":1}"#;
	assert_eq!(r1, expected);

	let expected = r#"{"jsonrpc":"2.0","error":{"code":-32071,"message":"This method is not part of the official RPC API yet (EIP-712). Run with `--jsonrpc-experimental` to enable it.","data":"See EIP: https://eips.ethereum.org/EIPS/eip-712"},"id":1}"#;
	assert_eq!(r2, expected);
}
