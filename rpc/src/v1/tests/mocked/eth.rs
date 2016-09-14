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
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, Duration};
use jsonrpc_core::IoHandler;
use util::{Uint, U256, Address, H256, FixedHash, Mutex};
use ethcore::account_provider::AccountProvider;
use ethcore::client::{TestBlockChainClient, EachBlockWith, Executed, TransactionID};
use ethcore::log_entry::{LocalizedLogEntry, LogEntry};
use ethcore::receipt::LocalizedReceipt;
use ethcore::transaction::{Transaction, Action};
use ethcore::miner::{ExternalMiner, MinerService};
use ethsync::SyncState;
use v1::{Eth, EthClient, EthClientOptions, EthSigning, EthSigningUnsafeClient};
use v1::tests::helpers::{TestSyncProvider, Config, TestMinerService};
use rustc_serialize::hex::ToHex;
use time::get_time;

fn blockchain_client() -> Arc<TestBlockChainClient> {
	let client = TestBlockChainClient::new();
	Arc::new(client)
}

fn accounts_provider() -> Arc<AccountProvider> {
	Arc::new(AccountProvider::transient_provider())
}

fn sync_provider() -> Arc<TestSyncProvider> {
	Arc::new(TestSyncProvider::new(Config {
		network_id: U256::from(3),
		num_peers: 120,
	}))
}

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

struct EthTester {
	pub client: Arc<TestBlockChainClient>,
	pub sync: Arc<TestSyncProvider>,
	pub accounts_provider: Arc<AccountProvider>,
	pub miner: Arc<TestMinerService>,
	hashrates: Arc<Mutex<HashMap<H256, (Instant, U256)>>>,
	pub io: IoHandler,
}

impl Default for EthTester {
	fn default() -> Self {
		Self::new_with_options(Default::default())
	}
}

impl EthTester {
	pub fn new_with_options(options: EthClientOptions) -> Self {
		let client = blockchain_client();
		let sync = sync_provider();
		let ap = accounts_provider();
		let miner = miner_service();
		let hashrates = Arc::new(Mutex::new(HashMap::new()));
		let external_miner = Arc::new(ExternalMiner::new(hashrates.clone()));
		let eth = EthClient::new(&client, &sync, &ap, &miner, &external_miner, options).to_delegate();
		let sign = EthSigningUnsafeClient::new(&client, &ap, &miner).to_delegate();
		let io = IoHandler::new();
		io.add_delegate(eth);
		io.add_delegate(sign);

		EthTester {
			client: client,
			sync: sync,
			accounts_provider: ap,
			miner: miner,
			io: io,
			hashrates: hashrates,
		}
	}
}

#[test]
fn rpc_eth_protocol_version() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_protocolVersion", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"63","id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_syncing() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_syncing", "params": [], "id": 1}"#;

	let tester = EthTester::default();

	let false_res = r#"{"jsonrpc":"2.0","result":false,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(false_res.to_owned()));

	{
		let mut status = tester.sync.status.write();
		status.state = SyncState::Blocks;
		status.highest_block_number = Some(2500);

		// "sync" to 1000 blocks.
		// causes TestBlockChainClient to return 1000 for its best block number.
		let mut blocks = tester.client.blocks.write();
		for i in 0..1000 {
			blocks.insert(H256::from(i), Vec::new());
		}
	}

	let true_res = r#"{"jsonrpc":"2.0","result":{"currentBlock":"0x3e8","highestBlock":"0x9c4","startingBlock":"0x0"},"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(true_res.to_owned()));

	{
		// finish "syncing"
		let mut blocks = tester.client.blocks.write();
		for i in 0..1500 {
			blocks.insert(H256::from(i + 1000), Vec::new());
		}
	}

	assert_eq!(tester.io.handle_request_sync(request), Some(false_res.to_owned()));
}

#[test]
fn rpc_eth_hashrate() {
	let tester = EthTester::default();
	tester.hashrates.lock().insert(H256::from(0), (Instant::now() + Duration::from_secs(2), U256::from(0xfffa)));
	tester.hashrates.lock().insert(H256::from(0), (Instant::now() + Duration::from_secs(2), U256::from(0xfffb)));
	tester.hashrates.lock().insert(H256::from(1), (Instant::now() + Duration::from_secs(2), U256::from(0x1)));

	let request = r#"{"jsonrpc": "2.0", "method": "eth_hashrate", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xfffc","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_logs() {
	let tester = EthTester::default();

	let request = r#"{"jsonrpc": "2.0", "method": "eth_getLogs", "params": [{}], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_logs_with_limit() {
	let tester = EthTester::default();

	let request1 = r#"{"jsonrpc": "2.0", "method": "eth_getLogs", "params": [{}, 1], "id": 1}"#;
	let request2 = r#"{"jsonrpc": "2.0", "method": "eth_getLogs", "params": [{}, 0], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request1), Some(response.to_owned()));
	assert_eq!(tester.io.handle_request_sync(request2), Some(response.to_owned()));
}

#[test]
fn rpc_eth_submit_hashrate() {
	let tester = EthTester::default();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_submitHashrate",
		"params": [
			"0x0000000000000000000000000000000000000000000000000000000000500000",
			"0x59daa26581d0acd1fce254fb7e85952f4c09d0915afd33d3886cd914bc7d283c"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(tester.hashrates.lock().get(&H256::from("0x59daa26581d0acd1fce254fb7e85952f4c09d0915afd33d3886cd914bc7d283c")).cloned().unwrap().1,
		U256::from(0x500_000));
}

#[test]
fn rpc_eth_sign() {
	let tester = EthTester::default();

	let account = tester.accounts_provider.new_account("abcd").unwrap();
	tester.accounts_provider.unlock_account_permanently(account, "abcd".into()).unwrap();
	let message = H256::from("0x0cc175b9c0f1b6a831c399e26977266192eb5ffee6ae2fec3ad71c777531578f");
	let signed = tester.accounts_provider.sign(account, message).unwrap();

	let req = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sign",
		"params": [
			""#.to_owned() + &format!("0x{:?}", account) + r#"",
			"0x0cc175b9c0f1b6a831c399e26977266192eb5ffee6ae2fec3ad71c777531578f"
		],
		"id": 1
	}"#;
	let res = r#"{"jsonrpc":"2.0","result":""#.to_owned() + &format!("0x{}", signed) + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&req), Some(res));
}

#[test]
fn rpc_eth_author() {
	let make_res = |addr| r#"{"jsonrpc":"2.0","result":""#.to_owned() + &format!("0x{:?}", addr) + r#"","id":1}"#;
	let tester = EthTester::default();

	let req = r#"{
		"jsonrpc": "2.0",
		"method": "eth_coinbase",
		"params": [],
		"id": 1
	}"#;

	assert_eq!(tester.io.handle_request_sync(req), Some(make_res(Address::zero())));

	for i in 0..20 {
		let addr = tester.accounts_provider.new_account(&format!("{}", i)).unwrap();
		tester.miner.set_author(addr.clone());

		assert_eq!(tester.io.handle_request_sync(req), Some(make_res(addr)));
	}
}

#[test]
fn rpc_eth_mining() {
	let tester = EthTester::default();
	tester.miner.set_author(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap());

	let request = r#"{"jsonrpc": "2.0", "method": "eth_mining", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":false,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_gas_price() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_gasPrice", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x4a817c800","id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_accounts() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account("").unwrap();

	let request = r#"{"jsonrpc": "2.0", "method": "eth_accounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[""#.to_owned() + &format!("0x{:?}", address) + r#""],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_block_number() {
	let tester = EthTester::default();
	tester.client.add_blocks(10, EachBlockWith::Nothing);

	let request = r#"{"jsonrpc": "2.0", "method": "eth_blockNumber", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xa","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_balance() {
	let tester = EthTester::default();
	tester.client.set_balance(Address::from(1), U256::from(5));

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getBalance",
		"params": ["0x0000000000000000000000000000000000000001", "latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x5","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_balance_pending() {
	let tester = EthTester::default();
	tester.client.set_balance(Address::from(1), U256::from(5));

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getBalance",
		"params": ["0x0000000000000000000000000000000000000001", "pending"],
		"id": 1
	}"#;

	// the TestMinerService doesn't communicate with the the TestBlockChainClient in any way.
	// if this returns zero, we know that the "pending" call is being properly forwarded to the
	// miner.
	let response = r#"{"jsonrpc":"2.0","result":"0x0","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_storage_at() {
	let tester = EthTester::default();
	tester.client.set_storage(Address::from(1), H256::from(4), H256::from(7));

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getStorageAt",
		"params": ["0x0000000000000000000000000000000000000001", "0x4", "latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000007","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_transaction_count() {
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionCount",
		"params": ["0x0000000000000000000000000000000000000001", "latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0","id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_block_transaction_count_by_hash() {
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getBlockTransactionCountByHash",
		"params": ["0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_transaction_count_by_number() {
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getBlockTransactionCountByNumber",
		"params": ["latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0","id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_transaction_count_by_number_pending() {
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getBlockTransactionCountByNumber",
		"params": ["pending"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1","id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_pending_transaction_by_hash() {
	use util::*;
	use ethcore::transaction::*;

	let tester = EthTester::default();
	{
		let tx: SignedTransaction = ::rlp::decode(&FromHex::from_hex("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap());
		tester.miner.pending_transactions.lock().insert(H256::zero(), tx);
	}

	let response = r#"{"jsonrpc":"2.0","result":{"blockHash":null,"blockNumber":null,"creates":null,"from":"0x0f65fe9276bc9a24ae7083ae28e2660ef72df99e","gas":"0x5208","gasPrice":"0x1","hash":"0x41df922fd0d4766fcc02e161f8295ec28522f329ae487f14d811e4b64c8d6e31","input":"0x","nonce":"0x0","raw":"0xf85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804","to":"0x095e7baea6a6c7c4c2dfeb977efac326af552d87","transactionIndex":null,"value":"0xa"},"id":1}"#;
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionByHash",
		"params": ["0x0000000000000000000000000000000000000000000000000000000000000000"],
		"id": 1
	}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}


#[test]
fn rpc_eth_uncle_count_by_block_hash() {
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getUncleCountByBlockHash",
		"params": ["0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_uncle_count_by_block_number() {
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getUncleCountByBlockNumber",
		"params": ["latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0","id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_code() {
	let tester = EthTester::default();
	tester.client.set_code(Address::from(1), vec![0xff, 0x21]);

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getCode",
		"params": ["0x0000000000000000000000000000000000000001", "latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xff21","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_call_latest() {
	let tester = EthTester::default();
	tester.client.set_execution_result(Ok(Executed {
		gas: U256::zero(),
		gas_used: U256::from(0xff30),
		refunded: U256::from(0x5),
		cumulative_gas_used: U256::zero(),
		logs: vec![],
		contracts_created: vec![],
		output: vec![0x12, 0x34, 0xff],
		trace: vec![],
		vm_trace: None,
		state_diff: None,
	}));

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_call",
		"params": [{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		},
		"latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1234ff","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_call() {
	let tester = EthTester::default();
	tester.client.set_execution_result(Ok(Executed {
		gas: U256::zero(),
		gas_used: U256::from(0xff30),
		refunded: U256::from(0x5),
		cumulative_gas_used: U256::zero(),
		logs: vec![],
		contracts_created: vec![],
		output: vec![0x12, 0x34, 0xff],
		trace: vec![],
		vm_trace: None,
		state_diff: None,
	}));

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_call",
		"params": [{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		},
		"0x0"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1234ff","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_call_default_block() {
	let tester = EthTester::default();
	tester.client.set_execution_result(Ok(Executed {
		gas: U256::zero(),
		gas_used: U256::from(0xff30),
		refunded: U256::from(0x5),
		cumulative_gas_used: U256::zero(),
		logs: vec![],
		contracts_created: vec![],
		output: vec![0x12, 0x34, 0xff],
		trace: vec![],
		vm_trace: None,
		state_diff: None,
	}));

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_call",
		"params": [{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		}],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1234ff","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_estimate_gas() {
	let tester = EthTester::default();
	tester.client.set_execution_result(Ok(Executed {
		gas: U256::zero(),
		gas_used: U256::from(0xff30),
		refunded: U256::from(0x5),
		cumulative_gas_used: U256::zero(),
		logs: vec![],
		contracts_created: vec![],
		output: vec![0x12, 0x34, 0xff],
		trace: vec![],
		vm_trace: None,
		state_diff: None,
	}));

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_estimateGas",
		"params": [{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		},
		"latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xff35","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_estimate_gas_default_block() {
	let tester = EthTester::default();
	tester.client.set_execution_result(Ok(Executed {
		gas: U256::zero(),
		gas_used: U256::from(0xff30),
		refunded: U256::from(0x5),
		cumulative_gas_used: U256::zero(),
		logs: vec![],
		contracts_created: vec![],
		output: vec![0x12, 0x34, 0xff],
		trace: vec![],
		vm_trace: None,
		state_diff: None,
	}));

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_estimateGas",
		"params": [{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		}],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xff35","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_send_transaction() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account("").unwrap();
	tester.accounts_provider.unlock_account_permanently(address, "".into()).unwrap();
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

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	let signature = tester.accounts_provider.sign(address, t.hash()).unwrap();
	let t = t.with_signature(signature);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&request), Some(response));

	tester.miner.last_nonces.write().insert(address.clone(), U256::zero());

	let t = Transaction {
		nonce: U256::one(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	let signature = tester.accounts_provider.sign(address, t.hash()).unwrap();
	let t = t.with_signature(signature);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&request), Some(response));
}
#[test]
fn rpc_eth_send_transaction_with_bad_to() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account("").unwrap();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"to": "",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;

	let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params","data":null},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));
}


#[test]
fn rpc_eth_send_transaction_error() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account("").unwrap();
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

	let response = r#"{"jsonrpc":"2.0","error":{"code":-32020,"message":"Your account is locked. Unlock the account via CLI, personal_unlockAccount or use Trusted Signer.","data":"NotUnlocked"},"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(&request), Some(response.into()));
}

#[test]
fn rpc_eth_send_raw_transaction() {
	let tester = EthTester::default();
	let address = tester.accounts_provider.new_account("abcd").unwrap();
	tester.accounts_provider.unlock_account_permanently(address, "abcd".into()).unwrap();

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	let signature = tester.accounts_provider.sign(address, t.hash()).unwrap();
	let t = t.with_signature(signature);

	let rlp = ::rlp::encode(&t).to_vec().to_hex();

	let req = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendRawTransaction",
		"params": [
			"0x"#.to_owned() + &rlp + r#""
		],
		"id": 1
	}"#;

	let res = r#"{"jsonrpc":"2.0","result":""#.to_owned() + &format!("0x{:?}", t.hash()) + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&req), Some(res));
}

#[test]
fn rpc_eth_transaction_receipt() {
	let receipt = LocalizedReceipt {
		transaction_hash: H256::zero(),
		transaction_index: 0,
		block_hash: H256::from_str("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5").unwrap(),
		block_number: 0x4510c,
		cumulative_gas_used: U256::from(0x20),
		gas_used: U256::from(0x10),
		contract_address: None,
		logs: vec![LocalizedLogEntry {
			entry: LogEntry {
				address: Address::from_str("33990122638b9132ca29c723bdf037f1a891a70c").unwrap(),
				topics: vec![
					H256::from_str("a6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc").unwrap(),
					H256::from_str("4861736852656700000000000000000000000000000000000000000000000000").unwrap()
				],
				data: vec![],
			},
			block_hash: H256::from_str("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5").unwrap(),
			block_number: 0x4510c,
			transaction_hash: H256::new(),
			transaction_index: 0,
			log_index: 1,
		}]
	};

	let hash = H256::from_str("b903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238").unwrap();
	let tester = EthTester::default();
	tester.client.set_transaction_receipt(TransactionID::Hash(hash), receipt);

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionReceipt",
		"params": ["0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x4510c","contractAddress":null,"cumulativeGasUsed":"0x20","gasUsed":"0x10","logs":[{"address":"0x33990122638b9132ca29c723bdf037f1a891a70c","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x4510c","data":"0x","logIndex":"0x1","topics":["0xa6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc","0x4861736852656700000000000000000000000000000000000000000000000000"],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","type":"mined"}],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0"},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_transaction_receipt_null() {
	let tester = EthTester::default();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionReceipt",
		"params": ["0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

// These tests are incorrect: their output is undefined as long as eth_getCompilers is [].
// Will ignore for now, but should probably be replaced by more substantial tests which check
// the output of eth_getCompilers to determine whether to test. CI systems can then be preinstalled
// with solc/serpent/lllc and they'll be proper again.
#[ignore]
#[test]
fn rpc_eth_compilers() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getCompilers", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[ignore]
#[test]
fn rpc_eth_compile_lll() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_compileLLL", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error","data":null},"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[ignore]
#[test]
fn rpc_eth_compile_solidity() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_compileSolidity", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error","data":null},"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[ignore]
#[test]
fn rpc_eth_compile_serpent() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_compileSerpent", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error","data":null},"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_get_work_returns_no_work_if_cant_mine() {
	let eth_tester = EthTester::default();
	eth_tester.client.set_queue_size(10);

	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32001,"message":"Still syncing.","data":null},"id":1}"#;

	assert_eq!(eth_tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_get_work_returns_correct_work_package() {
	let eth_tester = EthTester::default();
	eth_tester.miner.set_author(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap());

	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x3bbe93f74e7b97ae00784aeff8819c5cb600dd87e8b282a5d3446f3f871f0347","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000800000000000000000000000000000000000000000000000000000000000","0x1"],"id":1}"#;

	assert_eq!(eth_tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_get_work_should_not_return_block_number() {
	let eth_tester = EthTester::new_with_options(EthClientOptions {
		allow_pending_receipt_query: true,
		send_block_number_in_get_work: false,
	});
	eth_tester.miner.set_author(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap());

	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x3bbe93f74e7b97ae00784aeff8819c5cb600dd87e8b282a5d3446f3f871f0347","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000800000000000000000000000000000000000000000000000000000000000"],"id":1}"#;

	assert_eq!(eth_tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_get_work_should_timeout() {
	let eth_tester = EthTester::default();
	eth_tester.miner.set_author(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap());
	eth_tester.client.set_latest_block_timestamp(get_time().sec as u64 - 1000);  // Set latest block to 1000 seconds ago
	let hash = eth_tester.miner.map_sealing_work(&*eth_tester.client, |b| b.hash()).unwrap();

	// Request without providing timeout. This should work since we're disabling timeout.
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [], "id": 1}"#;
	let work_response = format!(
		r#"{{"jsonrpc":"2.0","result":["0x{:?}","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000800000000000000000000000000000000000000000000000000000000000","0x1"],"id":1}}"#,
		hash,
	);
	assert_eq!(eth_tester.io.handle_request_sync(request), Some(work_response.to_owned()));

	// Request with timeout of 0 seconds. This should work since we're disabling timeout.
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": ["0"], "id": 1}"#;
	let work_response = format!(
		r#"{{"jsonrpc":"2.0","result":["0x{:?}","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000800000000000000000000000000000000000000000000000000000000000","0x1"],"id":1}}"#,
		hash,
	);
	assert_eq!(eth_tester.io.handle_request_sync(request), Some(work_response.to_owned()));

	// Request with timeout of 10K seconds. This should work.
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": ["10000"], "id": 1}"#;
	assert_eq!(eth_tester.io.handle_request_sync(request), Some(work_response.to_owned()));

	// Request with timeout of 10 seconds. This should fail.
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": ["10"], "id": 1}"#;
	let err_response = r#"{"jsonrpc":"2.0","error":{"code":-32003,"message":"Work has not changed.","data":null},"id":1}"#;
	assert_eq!(eth_tester.io.handle_request_sync(request), Some(err_response.to_owned()));
}
