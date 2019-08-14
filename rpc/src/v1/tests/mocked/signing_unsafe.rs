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

use std::str::FromStr;
use std::sync::Arc;

use accounts::AccountProvider;
use ethcore::client::TestBlockChainClient;
use ethereum_types::{U256, Address};
use parity_runtime::Runtime;
use parking_lot::Mutex;
use rlp;
use rustc_hex::ToHex;
use types::transaction::{Transaction, Action};

use jsonrpc_core::IoHandler;
use v1::{EthClientOptions, EthSigning, SigningUnsafeClient};
use v1::helpers::nonce;
use v1::helpers::dispatch::{self, FullDispatcher};
use v1::tests::helpers::{TestMinerService};
use v1::metadata::Metadata;

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

struct EthTester {
	pub runtime: Runtime,
	pub client: Arc<TestBlockChainClient>,
	pub accounts_provider: Arc<AccountProvider>,
	pub miner: Arc<TestMinerService>,
	pub io: IoHandler<Metadata>,
}

impl Default for EthTester {
	fn default() -> Self {
		Self::new_with_options(Default::default())
	}
}

impl EthTester {
	pub fn new_with_options(options: EthClientOptions) -> Self {
		let runtime = Runtime::with_thread_count(1);
		let client = blockchain_client();
		let accounts_provider = accounts_provider();
		let ap = Arc::new(dispatch::Signer::new(accounts_provider.clone())) as _;
		let miner = miner_service();
		let gas_price_percentile = options.gas_price_percentile;
		let reservations = Arc::new(Mutex::new(nonce::Reservations::new(runtime.executor())));

		let dispatcher = FullDispatcher::new(client.clone(), miner.clone(), reservations, gas_price_percentile);
		let sign = SigningUnsafeClient::new(&ap, dispatcher).to_delegate();
		let mut io: IoHandler<Metadata> = IoHandler::default();
		io.extend_with(sign);

		EthTester {
			runtime,
			client,
			miner,
			io,
			accounts_provider,
		}
	}
}

#[test]
fn rpc_eth_send_transaction() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account(&"".into()).unwrap();
	tester.accounts_provider.unlock_account_permanently(address, "".into()).unwrap();
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

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	let signature = tester.accounts_provider.sign(address, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:x}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&request), Some(response));

	tester.miner.increment_nonce(&address);

	let t = Transaction {
		nonce: U256::one(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	let signature = tester.accounts_provider.sign(address, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:x}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&request), Some(response));
}

#[test]
fn rpc_eth_sign_transaction() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account(&"".into()).unwrap();
	tester.accounts_provider.unlock_account_permanently(address, "".into()).unwrap();
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
	let signature = tester.accounts_provider.sign(address, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);
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
		&format!("\"publicKey\":\"0x{:x}\",", t.recover_public().unwrap()) +
		&format!("\"r\":\"0x{:x}\",", U256::from(signature.r())) +
		&format!("\"raw\":\"0x{}\",", rlp.to_hex()) +
		&format!("\"s\":\"0x{:x}\",", U256::from(signature.s())) +
		&format!("\"standardV\":\"0x{:x}\",", U256::from(t.standard_v())) +
		r#""to":"0xd46e8dd67c5d32be8058bb8eb970870f07244567","transactionIndex":null,"# +
		&format!("\"v\":\"0x{:x}\",", U256::from(t.original_v())) +
		r#""value":"0x9184e72a""# +
		r#"}},"id":1}"#;

	tester.miner.increment_nonce(&address);

	assert_eq!(tester.io.handle_request_sync(&request), Some(response));
}

#[test]
fn rpc_eth_send_transaction_with_bad_to() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account(&"".into()).unwrap();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			"to": "",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;

	let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: prefix is missing."},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));
}

#[test]
fn rpc_eth_send_transaction_error() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account(&"".into()).unwrap();
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

	let response = r#"{"jsonrpc":"2.0","error":{"code":-32020,"message":"Your account is locked. Unlock the account via CLI, personal_unlockAccount or use Trusted Signer.","data":"NotUnlocked"},"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));
}
