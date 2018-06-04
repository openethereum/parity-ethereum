// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
use ethereum_types::{U256, Address};
use bytes::ToPretty;

use ethcore::account_provider::AccountProvider;
use ethcore::client::TestBlockChainClient;
use parity_reactor::EventLoop;
use parking_lot::Mutex;
use rlp::encode;
use transaction::{Transaction, Action, SignedTransaction};

use serde_json;
use jsonrpc_core::IoHandler;
use v1::{SignerClient, Signer, Origin};
use v1::metadata::Metadata;
use v1::tests::helpers::TestMinerService;
use v1::types::{Bytes as RpcBytes, H520};
use v1::helpers::{nonce, SigningQueue, SignerService, FilledTransactionRequest, ConfirmationPayload};
use v1::helpers::dispatch::{FullDispatcher, eth_data_hash};

struct SignerTester {
	signer: Arc<SignerService>,
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

fn signer_tester() -> SignerTester {
	let signer = Arc::new(SignerService::new_test(false));
	let accounts = accounts_provider();
	let client = blockchain_client();
	let miner = miner_service();
	let reservations = Arc::new(Mutex::new(nonce::Reservations::new()));
	let event_loop = EventLoop::spawn();

	let dispatcher = FullDispatcher::new(client, miner.clone(), reservations, 50);
	let mut io = IoHandler::default();
	io.extend_with(SignerClient::new(&accounts, dispatcher, &signer, event_loop.remote()).to_delegate());

	SignerTester {
		signer: signer,
		accounts: accounts,
		io: io,
		miner: miner,
	}
}

#[test]
fn should_return_list_of_items_to_confirm() {
	// given
	let tester = signer_tester();
	let _send_future = tester.signer.add_request(ConfirmationPayload::SendTransaction(FilledTransactionRequest {
		from: Address::from(1),
		used_default_from: false,
		to: Some(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: None,
		condition: None,
	}), Origin::Dapps("http://parity.io".into())).unwrap();
	let _sign_future = tester.signer.add_request(ConfirmationPayload::EthSignMessage(1.into(), vec![5].into()), Origin::Unknown).unwrap();

	// when
	let request = r#"{"jsonrpc":"2.0","method":"signer_requestsToConfirm","params":[],"id":1}"#;
	let response = concat!(
		r#"{"jsonrpc":"2.0","result":["#,
		r#"{"id":"0x1","origin":{"dapp":"http://parity.io"},"payload":{"sendTransaction":{"condition":null,"data":"0x","from":"0x0000000000000000000000000000000000000001","gas":"0x989680","gasPrice":"0x2710","nonce":null,"to":"0xd46e8dd67c5d32be8058bb8eb970870f07244567","value":"0x1"}}},"#,
		r#"{"id":"0x2","origin":"unknown","payload":{"sign":{"address":"0x0000000000000000000000000000000000000001","data":"0x05"}}}"#,
		r#"],"id":1}"#
	);

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn should_reject_transaction_from_queue_without_dispatching() {
	// given
	let tester = signer_tester();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::SendTransaction(FilledTransactionRequest {
		from: Address::from(1),
		used_default_from: false,
		to: Some(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: None,
		condition: None,
	}), Origin::Unknown).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);

	// when
	let request = r#"{"jsonrpc":"2.0","method":"signer_rejectRequest","params":["0x1"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().len(), 0);
}

#[test]
fn should_not_remove_transaction_if_password_is_invalid() {
	// given
	let tester = signer_tester();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::SendTransaction(FilledTransactionRequest {
		from: Address::from(1),
		used_default_from: false,
		to: Some(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: None,
		condition: None,
	}), Origin::Unknown).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);

	// when
	let request = r#"{"jsonrpc":"2.0","method":"signer_confirmRequest","params":["0x1",{},"xxx"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32021,"message":"Account password is invalid or account does not exist.","data":"SStore(InvalidAccount)"},"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 1);
}

#[test]
fn should_not_remove_sign_if_password_is_invalid() {
	// given
	let tester = signer_tester();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::EthSignMessage(0.into(), vec![5].into()), Origin::Unknown).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);

	// when
	let request = r#"{"jsonrpc":"2.0","method":"signer_confirmRequest","params":["0x1",{},"xxx"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32021,"message":"Account password is invalid or account does not exist.","data":"SStore(InvalidAccount)"},"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 1);
}

#[test]
fn should_confirm_transaction_and_dispatch() {
	//// given
	let tester = signer_tester();
	let address = tester.accounts.new_account("test").unwrap();
	let recipient = Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::SendTransaction(FilledTransactionRequest {
		from: address,
		used_default_from: false,
		to: Some(recipient),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: None,
		condition: None,
	}), Origin::Unknown).unwrap();

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x1000),
		gas: U256::from(0x50505),
		action: Action::Call(recipient),
		value: U256::from(0x1),
		data: vec![]
	};
	tester.accounts.unlock_account_temporarily(address, "test".into()).unwrap();
	let signature = tester.accounts.sign(address, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	assert_eq!(tester.signer.requests().len(), 1);

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_confirmRequest",
		"params":["0x1", {"gasPrice":"0x1000","gas":"0x50505"}, "test"],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:x}", t.hash()).as_ref() + r#"","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().len(), 1);
}

#[test]
fn should_alter_the_sender_and_nonce() {
	//// given
	let tester = signer_tester();
	let recipient = Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::SendTransaction(FilledTransactionRequest {
		from: 0.into(),
		used_default_from: false,
		to: Some(recipient),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: Some(10.into()),
		condition: None,
	}), Origin::Unknown).unwrap();

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x1000),
		gas: U256::from(0x50505),
		action: Action::Call(recipient),
		value: U256::from(0x1),
		data: vec![]
	};

	let address = tester.accounts.new_account("test").unwrap();
	let signature = tester.accounts.sign(address, Some("test".into()), t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	assert_eq!(tester.signer.requests().len(), 1);

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_confirmRequest",
		"params":["0x1", {"sender":""#.to_owned()
		+ &format!("0x{:x}", address)
		+ r#"","gasPrice":"0x1000","gas":"0x50505"}, "test"],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + &format!("0x{:x}", t.hash()) + r#"","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().len(), 1);
}

#[test]
fn should_confirm_transaction_with_token() {
	// given
	let tester = signer_tester();
	let address = tester.accounts.new_account("test").unwrap();
	let recipient = Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::SendTransaction(FilledTransactionRequest {
		from: address,
		used_default_from: false,
		to: Some(recipient),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: None,
		condition: None,
	}), Origin::Unknown).unwrap();

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x1000),
		gas: U256::from(10_000_000),
		action: Action::Call(recipient),
		value: U256::from(0x1),
		data: vec![]
	};
	let (signature, token) = tester.accounts.sign_with_token(address, "test".into(), t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	assert_eq!(tester.signer.requests().len(), 1);

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_confirmRequestWithToken",
		"params":["0x1", {"gasPrice":"0x1000"}, ""#.to_owned() + &token + r#""],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"result":""#.to_owned() +
		format!("0x{:x}", t.hash()).as_ref() +
		r#"","token":""#;

	// then
	let result = tester.io.handle_request_sync(&request).unwrap();
	assert!(result.starts_with(&response), "Should return correct result. Expected: {:?}, Got: {:?}", response, result);
	assert_eq!(tester.signer.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().len(), 1);
}

#[test]
fn should_confirm_transaction_with_rlp() {
	// given
	let tester = signer_tester();
	let address = tester.accounts.new_account("test").unwrap();
	let recipient = Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::SendTransaction(FilledTransactionRequest {
		from: address,
		used_default_from: false,
		to: Some(recipient),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: None,
		condition: None,
	}), Origin::Unknown).unwrap();

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x1000),
		gas: U256::from(10_000_000),
		action: Action::Call(recipient),
		value: U256::from(0x1),
		data: vec![]
	};
	let signature = tester.accounts.sign(address, Some("test".into()), t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);
	let rlp = encode(&t);

	assert_eq!(tester.signer.requests().len(), 1);

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_confirmRequestRaw",
		"params":["0x1", "0x"#.to_owned() + &rlp.to_hex() + r#""],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:x}", t.hash()).as_ref() + r#"","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().len(), 1);
}

#[test]
fn should_return_error_when_sender_does_not_match() {
	// given
	let tester = signer_tester();
	let address = tester.accounts.new_account("test").unwrap();
	let recipient = Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::SendTransaction(FilledTransactionRequest {
		from: Address::default(),
		used_default_from: false,
		to: Some(recipient),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: None,
		condition: None,
	}), Origin::Unknown).unwrap();

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x1000),
		gas: U256::from(10_000_000),
		action: Action::Call(recipient),
		value: U256::from(0x1),
		data: vec![]
	};
	tester.accounts.unlock_account_temporarily(address, "test".into()).unwrap();
	let signature = tester.accounts.sign(address, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);
	let rlp = encode(&t);

	assert_eq!(tester.signer.requests().len(), 1);

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_confirmRequestRaw",
		"params":["0x1", "0x"#.to_owned() + &rlp.to_hex() + r#""],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Couldn't parse parameters: Sent transaction does not match the request.","data":"[\"from\"]"},"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 1);
}

#[test]
fn should_confirm_sign_transaction_with_rlp() {
	// given
	let tester = signer_tester();
	let address = tester.accounts.new_account("test").unwrap();
	let recipient = Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::SignTransaction(FilledTransactionRequest {
		from: address,
		used_default_from: false,
		to: Some(recipient),
		gas_price: U256::from(10_000),
		gas: U256::from(10_000_000),
		value: U256::from(1),
		data: vec![],
		nonce: None,
		condition: None,
	}), Origin::Unknown).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x1000),
		gas: U256::from(10_000_000),
		action: Action::Call(recipient),
		value: U256::from(0x1),
		data: vec![]
	};
	let signature = tester.accounts.sign(address, Some("test".into()), t.hash(None)).unwrap();
	let t = SignedTransaction::new(t.with_signature(signature.clone(), None)).unwrap();
	let rlp = encode(&t);

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_confirmRequestRaw",
		"params":["0x1", "0x"#.to_owned() + &rlp.to_hex() + r#""],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"#.to_owned() +
		r#""raw":"0x"# + &rlp.to_hex() + r#"","# +
		r#""tx":{"# +
		r#""blockHash":null,"blockNumber":null,"# +
		&format!("\"chainId\":{},", t.chain_id().map_or("null".to_owned(), |n| format!("{}", n))) +
		r#""condition":null,"creates":null,"# +
		&format!("\"from\":\"0x{:x}\",", &address) +
		r#""gas":"0x989680","gasPrice":"0x1000","# +
		&format!("\"hash\":\"0x{:x}\",", t.hash()) +
		r#""input":"0x","# +
		r#""nonce":"0x0","# +
		&format!("\"publicKey\":\"0x{:x}\",", t.public_key().unwrap()) +
		&format!("\"r\":\"0x{:x}\",", U256::from(signature.r())) +
		&format!("\"raw\":\"0x{}\",", rlp.to_hex()) +
		&format!("\"s\":\"0x{:x}\",", U256::from(signature.s())) +
		&format!("\"standardV\":\"0x{:x}\",", U256::from(t.standard_v())) +
		r#""to":"0xd46e8dd67c5d32be8058bb8eb970870f07244567","transactionIndex":null,"# +
		&format!("\"v\":\"0x{:x}\",", U256::from(t.original_v())) +
		r#""value":"0x1""# +
		r#"}},"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().len(), 0);
}

#[test]
fn should_confirm_data_sign_with_signature() {
	// given
	let tester = signer_tester();
	let address = tester.accounts.new_account("test").unwrap();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::EthSignMessage(
		address,
		vec![1, 2, 3, 4].into(),
	), Origin::Unknown).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);

	let data_hash = eth_data_hash(vec![1, 2, 3, 4].into());
	let signature = H520(tester.accounts.sign(address, Some("test".into()), data_hash).unwrap().into_electrum());
	let signature = format!("0x{:?}", signature);

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_confirmRequestRaw",
		"params":["0x1", ""#.to_owned() + &signature + r#""],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + &signature + r#"","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().len(), 0);
}

#[test]
fn should_confirm_decrypt_with_phrase() {
	// given
	let tester = signer_tester();
	let address = tester.accounts.new_account("test").unwrap();
	let _confirmation_future = tester.signer.add_request(ConfirmationPayload::Decrypt(
		address,
		vec![1, 2, 3, 4].into(),
	), Origin::Unknown).unwrap();
	assert_eq!(tester.signer.requests().len(), 1);

	let decrypted = serde_json::to_string(&RpcBytes::new(b"phrase".to_vec())).unwrap();

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_confirmRequestRaw",
		"params":["0x1", "#.to_owned() + &decrypted + r#"],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"#.to_owned() + &decrypted + r#","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
	assert_eq!(tester.signer.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().len(), 0);
}

#[test]
fn should_generate_new_token() {
	// given
	let tester = signer_tester();

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_generateAuthorizationToken",
		"params":[],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"new_token","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn should_generate_new_web_proxy_token() {
	use jsonrpc_core::{Response, Output, Value};
	// given
	let tester = signer_tester();

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"signer_generateWebProxyAccessToken",
		"params":["https://parity.io"],
		"id":1
	}"#;
	let response = tester.io.handle_request_sync(&request).unwrap();
	let result = serde_json::from_str(&response).unwrap();

	if let Response::Single(Output::Success(ref success)) = result {
		if let Value::String(ref token) = success.result {
			assert_eq!(tester.signer.web_proxy_access_token_domain(&token), Some("https://parity.io".into()));
			return;
		}
	}

	assert!(false, "Expected successful response, got: {:?}", result);
}
