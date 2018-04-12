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

use std::thread;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use rlp;

use jsonrpc_core::{IoHandler, Success};
use jsonrpc_core::futures::Future;
use v1::impls::SigningQueueClient;
use v1::metadata::Metadata;
use v1::traits::{EthSigning, ParitySigning, Parity};
use v1::helpers::{nonce, SignerService, SigningQueue, FullDispatcher};
use v1::types::{ConfirmationResponse, RichRawTransaction};
use v1::tests::helpers::TestMinerService;
use v1::tests::mocked::parity;

use ethereum_types::{U256, Address};
use bytes::ToPretty;
use ethcore::account_provider::AccountProvider;
use ethcore::client::TestBlockChainClient;
use ethkey::Secret;
use ethstore::ethkey::{Generator, Random};
use parking_lot::Mutex;
use serde_json;
use transaction::{Transaction, Action, SignedTransaction};

use parity_reactor::Remote;

struct SigningTester {
	pub signer: Arc<SignerService>,
	pub client: Arc<TestBlockChainClient>,
	pub miner: Arc<TestMinerService>,
	pub accounts: Arc<AccountProvider>,
	pub io: IoHandler<Metadata>,
}

impl Default for SigningTester {
	fn default() -> Self {
		let signer = Arc::new(SignerService::new_test(false));
		let client = Arc::new(TestBlockChainClient::default());
		let miner = Arc::new(TestMinerService::default());
		let accounts = Arc::new(AccountProvider::transient_provider());
		let opt_accounts = Some(accounts.clone());
		let reservations = Arc::new(Mutex::new(nonce::Reservations::new()));
		let mut io = IoHandler::default();

		let dispatcher = FullDispatcher::new(client.clone(), miner.clone(), reservations, 50);

		let remote = Remote::new_thread_per_future();

		let rpc = SigningQueueClient::new(&signer, dispatcher.clone(), remote.clone(), &opt_accounts);
		io.extend_with(EthSigning::to_delegate(rpc));
		let rpc = SigningQueueClient::new(&signer, dispatcher, remote, &opt_accounts);
		io.extend_with(ParitySigning::to_delegate(rpc));

		SigningTester {
			signer: signer,
			client: client,
			miner: miner,
			accounts: accounts,
			io: io,
		}
	}
}

fn eth_signing() -> SigningTester {
	SigningTester::default()
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
			""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			"0x0000000000000000000000000000000000000000000000000000000000000005"
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","id":1}"#;

	// then
	let promise = tester.io.handle_request(&request);

	// the future must be polled at least once before request is queued.
	let signer = tester.signer.clone();
	::std::thread::spawn(move || loop {
		if signer.requests().len() == 1 {
			// respond
			signer.request_confirmed(1.into(), Ok(ConfirmationResponse::Signature(0.into())));
			break
		}
		::std::thread::sleep(Duration::from_millis(100))
	});

	let res = promise.wait().unwrap();
	assert_eq!(res, Some(response.to_owned()));
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
		"method": "parity_postSign",
		"params": [
			""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
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
		"method": "parity_postSign",
		"params": [
			""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			"0x0000000000000000000000000000000000000000000000000000000000000005"
		],
		"id": 1
	}"#;
	tester.io.handle_request_sync(&request).expect("Sent");

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "parity_checkRequest",
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
		"method": "parity_postSign",
		"params": [
			""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			"0x0000000000000000000000000000000000000000000000000000000000000005"
		],
		"id": 1
	}"#;
	tester.io.handle_request_sync(&request).expect("Sent");
	tester.signer.request_confirmed(1.into(), Ok(ConfirmationResponse::Signature(1.into())));

	// This is not ideal, but we need to give futures some time to be executed, and they need to run in a separate thread
	thread::sleep(Duration::from_millis(20));

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "parity_checkRequest",
		"params": ["0x1"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn should_sign_if_account_is_unlocked() {
	// given
	let tester = eth_signing();
	let data = vec![5u8];
	let acc = tester.accounts.insert_account(Secret::from([69u8; 32]), "test").unwrap();
	tester.accounts.unlock_account_permanently(acc, "test".into()).unwrap();

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sign",
		"params": [
			""#.to_owned() + format!("0x{:x}", acc).as_ref() + r#"",
			""# + format!("0x{}", data.to_hex()).as_ref() + r#""
		],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xdb53b32e56cf3e9735377b7664d6de5a03e125b1bf8ec55715d253668b4238503b4ac931fe6af90add73e72a585e952665376b2b9afc5b6b239b7df74c734e121b","id":1}"#;
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
			"from": ""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000","id":1}"#;

	// then
	let promise = tester.io.handle_request(&request);

	// the future must be polled at least once before request is queued.
	let signer = tester.signer.clone();
	::std::thread::spawn(move || loop {
		if signer.requests().len() == 1 {
			// respond
			signer.request_confirmed(1.into(), Ok(ConfirmationResponse::SendTransaction(0.into())));
			break
		}
		::std::thread::sleep(Duration::from_millis(100))
	});

	let res = promise.wait().unwrap();
	assert_eq!(res, Some(response.to_owned()));
}

#[test]
fn should_add_sign_transaction_to_the_queue() {
	// given
	let tester = eth_signing();
	let address = tester.accounts.new_account("test").unwrap();

	assert_eq!(tester.signer.requests().len(), 0);

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_signTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;

	let t = Transaction {
		nonce: U256::one(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	let signature = tester.accounts.sign(address, Some("test".into()), t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);
	let t = SignedTransaction::new(t).unwrap();
	let signature = t.signature();
	let rlp = rlp::encode(&t);

	let response = r#"{"jsonrpc":"2.0","result":{"#.to_owned() +
		r#""raw":"0x"# + &rlp.to_hex() + r#"","# +
		r#""tx":{"# +
		r#""blockHash":null,"blockNumber":null,"# +
		&format!("\"chainId\":{},", t.chain_id().map_or("null".to_owned(), |n| format!("{}", n))) +
		r#""condition":null,"creates":null,"# +
		&format!("\"from\":\"0x{:x}\",", &address) +
		r#""gas":"0x76c0","gasPrice":"0x9184e72a000","# +
		&format!("\"hash\":\"0x{:x}\",", t.hash()) +
		r#""input":"0x","# +
		r#""nonce":"0x1","# +
		&format!("\"publicKey\":\"0x{:x}\",", t.public_key().unwrap()) +
		&format!("\"r\":\"0x{:x}\",", U256::from(signature.r())) +
		&format!("\"raw\":\"0x{}\",", rlp.to_hex()) +
		&format!("\"s\":\"0x{:x}\",", U256::from(signature.s())) +
		&format!("\"standardV\":\"0x{:x}\",", U256::from(t.standard_v())) +
		r#""to":"0xd46e8dd67c5d32be8058bb8eb970870f07244567","transactionIndex":null,"# +
		&format!("\"v\":\"0x{:x}\",", U256::from(t.original_v())) +
		r#""value":"0x9184e72a""# +
		r#"}},"id":1}"#;

	// then
	tester.miner.increment_nonce(&address);
	let promise = tester.io.handle_request(&request);

	// the future must be polled at least once before request is queued.
	let signer = tester.signer.clone();
	::std::thread::spawn(move || loop {
		if signer.requests().len() == 1 {
			// respond
			signer.request_confirmed(1.into(), Ok(ConfirmationResponse::SignTransaction(
				RichRawTransaction::from_signed(t.into(), 0x0, u64::max_value())
			)));
			break
		}
		::std::thread::sleep(Duration::from_millis(100))
	});

	let res = promise.wait().unwrap();
	assert_eq!(res, Some(response.to_owned()));
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
	let signature = tester.accounts.sign(acc, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:x}", acc).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:x}", t.hash()).as_ref() + r#"","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn should_decrypt_message_if_account_is_unlocked() {
	// given
	let mut tester = eth_signing();
	let parity = parity::Dependencies::new();
	tester.io.extend_with(parity.client(None).to_delegate());
	let (address, public) = tester.accounts.new_account_and_public("test").unwrap();
	tester.accounts.unlock_account_permanently(address, "test".into()).unwrap();


	// First encrypt message
	let request = format!("{}0x{:x}{}",
		r#"{"jsonrpc": "2.0", "method": "parity_encryptMessage", "params":[""#,
		public,
		r#"", "0x01020304"], "id": 1}"#
	);
	let encrypted: Success = serde_json::from_str(&tester.io.handle_request_sync(&request).unwrap()).unwrap();

	// then call decrypt
	let request = format!("{}{:x}{}{}{}",
		r#"{"jsonrpc": "2.0", "method": "parity_decryptMessage", "params":["0x"#,
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
		"method": "parity_decryptMessage",
		"params": ["0x"#.to_owned() + &format!("{:x}", acc.address()) + r#"",
		"0x012345"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0102","id":1}"#;

	// then
	let promise = tester.io.handle_request(&request);

	// the future must be polled at least once before request is queued.
	let signer = tester.signer.clone();
	::std::thread::spawn(move || loop {
		if signer.requests().len() == 1 {
			// respond
			signer.request_confirmed(1.into(), Ok(ConfirmationResponse::Decrypt(vec![0x1, 0x2].into())));
			break
		}
		::std::thread::sleep(Duration::from_millis(10))
	});

	// check response: will deadlock if unsuccessful.
	let res = promise.wait().unwrap();
	assert_eq!(res, Some(response.to_owned()));
}

#[test]
fn should_compose_transaction() {
	// given
	let tester = eth_signing();
	let acc = Random.generate().unwrap();
	assert_eq!(tester.signer.requests().len(), 0);
	let from = format!("{:x}", acc.address());

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "parity_composeTransaction",
		"params": [{"from":"0x"#.to_owned() + &from + r#"","value":"0x5"}],
		"id": 1
	}"#;

	let response = r#"{"jsonrpc":"2.0","result":{"condition":null,"data":"0x","from":"0x"#.to_owned()
		+ &from
		+ r#"","gas":"0x5208","gasPrice":"0x4a817c800","nonce":"0x0","to":null,"value":"0x5"},"id":1}"#;


	// then
	let res = tester.io.handle_request(&request).wait().unwrap();
	assert_eq!(res, Some(response.to_owned()));
}
