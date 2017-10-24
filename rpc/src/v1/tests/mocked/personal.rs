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
use std::str::FromStr;

use bigint::prelude::U256;
use ethcore::account_provider::AccountProvider;
use ethcore::client::TestBlockChainClient;
use ethcore::transaction::{Action, Transaction};
use jsonrpc_core::IoHandler;
use parking_lot::Mutex;
use util::Address;

use v1::{PersonalClient, Personal, Metadata};
use v1::helpers::nonce;
use v1::helpers::dispatch::FullDispatcher;
use v1::tests::helpers::TestMinerService;

struct PersonalTester {
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
	let accounts = accounts_provider();
	let opt_accounts = Some(accounts.clone());
	let client = blockchain_client();
	let miner = miner_service();
	let reservations = Arc::new(Mutex::new(nonce::Reservations::new()));

	let dispatcher = FullDispatcher::new(client, miner.clone(), reservations);
	let personal = PersonalClient::new(opt_accounts, dispatcher, false);

	let mut io = IoHandler::default();
	io.extend_with(personal.to_delegate());

	let tester = PersonalTester {
		accounts: accounts,
		io: io,
		miner: miner,
	};

	tester
}

#[test]
fn accounts() {
	let tester = setup();
	let address = tester.accounts.new_account("").unwrap();
	let request = r#"{"jsonrpc": "2.0", "method": "personal_listAccounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[""#.to_owned() + &format!("0x{:?}", address) + r#""],"id":1}"#;

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
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"","id":1}"#;

	assert_eq!(res, Some(response));
}

#[test]
fn sign_and_send_transaction_with_invalid_password() {
	let tester = setup();
	let address = tester.accounts.new_account("password123").unwrap();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_sendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
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
fn send_transaction() {
	sign_and_send_test("personal_sendTransaction");
}

#[test]
fn sign_and_send_transaction() {
	sign_and_send_test("personal_signAndSendTransaction");
}

fn sign_and_send_test(method: &str) {
	let tester = setup();
	let address = tester.accounts.new_account("password123").unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": ""#.to_owned() + method + r#"",
		"params": [{
			"from": ""# + format!("0x{:?}", address).as_ref() + r#"",
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

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response));

	tester.miner.last_nonces.write().insert(address.clone(), U256::zero());

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

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request.as_ref()), Some(response));
}

#[test]
fn should_unlock_not_account_temporarily_if_allow_perm_is_disabled() {
	let tester = setup();
	let address = tester.accounts.new_account("password123").unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_unlockAccount",
		"params": [
			""#.to_owned() + &format!("0x{:?}", address) + r#"",
			"password123",
			"0x100"
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Time-unlocking is only supported in --geth compatibility mode.","data":"Restart your client with --geth flag or use personal_sendTransaction instead."},"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));

	assert!(tester.accounts.sign(address, None, Default::default()).is_err(), "Should not unlock account.");
}

#[test]
fn should_unlock_account_permanently() {
	let tester = setup();
	let address = tester.accounts.new_account("password123").unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_unlockAccount",
		"params": [
			""#.to_owned() + &format!("0x{:?}", address) + r#"",
			"password123",
			null
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));
	assert!(tester.accounts.sign(address, None, Default::default()).is_ok(), "Should unlock account.");
}
