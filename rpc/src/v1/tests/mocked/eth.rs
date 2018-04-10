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

use std::str::FromStr;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

use ethereum_types::{H256, U256, Address};
use parking_lot::Mutex;
use ethcore::account_provider::AccountProvider;
use ethcore::client::{BlockChainClient, BlockId, EachBlockWith, Executed, TestBlockChainClient, TransactionId};
use ethcore::log_entry::{LocalizedLogEntry, LogEntry};
use ethcore::miner::MinerService;
use ethcore::receipt::{LocalizedReceipt, TransactionOutcome};
use ethkey::Secret;
use ethsync::SyncState;
use miner::external::ExternalMiner;
use rlp;
use rustc_hex::{FromHex, ToHex};
use transaction::{Transaction, Action};

use jsonrpc_core::IoHandler;
use v1::{Eth, EthClient, EthClientOptions, EthFilter, EthFilterClient, EthSigning, SigningUnsafeClient};
use v1::helpers::nonce;
use v1::helpers::dispatch::FullDispatcher;
use v1::tests::helpers::{TestSyncProvider, Config, TestMinerService, TestSnapshotService};
use v1::metadata::Metadata;
use v1::types::Origin;

fn blockchain_client() -> Arc<TestBlockChainClient> {
	let client = TestBlockChainClient::new();
	Arc::new(client)
}

fn accounts_provider() -> Arc<AccountProvider> {
	Arc::new(AccountProvider::transient_provider())
}

fn sync_provider() -> Arc<TestSyncProvider> {
	Arc::new(TestSyncProvider::new(Config {
		network_id: 3,
		num_peers: 120,
	}))
}

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

fn snapshot_service() -> Arc<TestSnapshotService> {
	Arc::new(TestSnapshotService::new())
}

struct EthTester {
	pub client: Arc<TestBlockChainClient>,
	pub sync: Arc<TestSyncProvider>,
	pub accounts_provider: Arc<AccountProvider>,
	pub miner: Arc<TestMinerService>,
	pub snapshot: Arc<TestSnapshotService>,
	hashrates: Arc<Mutex<HashMap<H256, (Instant, U256)>>>,
	pub io: IoHandler<Metadata>,
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
		let opt_ap = Some(ap.clone());
		let miner = miner_service();
		let snapshot = snapshot_service();
		let hashrates = Arc::new(Mutex::new(HashMap::new()));
		let external_miner = Arc::new(ExternalMiner::new(hashrates.clone()));
		let gas_price_percentile = options.gas_price_percentile;
		let eth = EthClient::new(&client, &snapshot, &sync, &opt_ap, &miner, &external_miner, options).to_delegate();
		let filter = EthFilterClient::new(client.clone(), miner.clone()).to_delegate();
		let reservations = Arc::new(Mutex::new(nonce::Reservations::new()));

		let dispatcher = FullDispatcher::new(client.clone(), miner.clone(), reservations, gas_price_percentile);
		let sign = SigningUnsafeClient::new(&opt_ap, dispatcher).to_delegate();
		let mut io: IoHandler<Metadata> = IoHandler::default();
		io.extend_with(eth);
		io.extend_with(sign);
		io.extend_with(filter);

		EthTester {
			client: client,
			sync: sync,
			accounts_provider: ap,
			miner: miner,
			snapshot: snapshot,
			io: io,
			hashrates: hashrates,
		}
	}

	pub fn add_blocks(&self, count: usize, with: EachBlockWith) {
		self.client.add_blocks(count, with);
		self.sync.increase_imported_block_number(count as u64);
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
	use ethcore::snapshot::RestorationStatus;

	let request = r#"{"jsonrpc": "2.0", "method": "eth_syncing", "params": [], "id": 1}"#;

	let tester = EthTester::default();

	let false_res = r#"{"jsonrpc":"2.0","result":false,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(false_res.to_owned()));

	{
		let mut status = tester.sync.status.write();
		status.state = SyncState::Blocks;
		status.highest_block_number = Some(2500);
	}

	// "sync" to 1000 blocks.
	// causes TestBlockChainClient to return 1000 for its best block number.
	tester.add_blocks(1000, EachBlockWith::Nothing);


	let true_res = r#"{"jsonrpc":"2.0","result":{"currentBlock":"0x3e8","highestBlock":"0x9c4","startingBlock":"0x0","warpChunksAmount":null,"warpChunksProcessed":null},"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(true_res.to_owned()));

	*tester.client.ancient_block.write() = None;
	*tester.client.first_block.write() = None;

	let snap_res = r#"{"jsonrpc":"2.0","result":{"currentBlock":"0x3e8","highestBlock":"0x9c4","startingBlock":"0x0","warpChunksAmount":"0x32","warpChunksProcessed":"0x18"},"id":1}"#;
	tester.snapshot.set_status(RestorationStatus::Ongoing {
		state_chunks: 40,
		block_chunks: 10,
		state_chunks_done: 18,
		block_chunks_done: 6,
	});

	assert_eq!(tester.io.handle_request_sync(request), Some(snap_res.to_owned()));

	tester.snapshot.set_status(RestorationStatus::Inactive);

	// finish "syncing"
	tester.add_blocks(1500, EachBlockWith::Nothing);

	{
		let mut status = tester.sync.status.write();
		status.state = SyncState::Idle;
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
	tester.client.set_logs(vec![LocalizedLogEntry {
		block_number: 1,
		block_hash: H256::default(),
		entry: LogEntry {
			address: Address::default(),
			topics: vec![],
			data: vec![1,2,3],
		},
		transaction_index: 0,
		transaction_log_index: 0,
		transaction_hash: H256::default(),
		log_index: 0,
	}, LocalizedLogEntry {
		block_number: 1,
		block_hash: H256::default(),
		entry: LogEntry {
			address: Address::default(),
			topics: vec![],
			data: vec![1,2,3],
		},
		transaction_index: 0,
		transaction_log_index: 1,
		transaction_hash: H256::default(),
		log_index: 1,
	}]);


	let request1 = r#"{"jsonrpc": "2.0", "method": "eth_getLogs", "params": [{}], "id": 1}"#;
	let request2 = r#"{"jsonrpc": "2.0", "method": "eth_getLogs", "params": [{"limit":1}], "id": 1}"#;
	let request3 = r#"{"jsonrpc": "2.0", "method": "eth_getLogs", "params": [{"limit":0}], "id": 1}"#;

	let response1 = r#"{"jsonrpc":"2.0","result":[{"address":"0x0000000000000000000000000000000000000000","blockHash":"0x0000000000000000000000000000000000000000000000000000000000000000","blockNumber":"0x1","data":"0x010203","logIndex":"0x0","topics":[],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","transactionLogIndex":"0x0","type":"mined"},{"address":"0x0000000000000000000000000000000000000000","blockHash":"0x0000000000000000000000000000000000000000000000000000000000000000","blockNumber":"0x1","data":"0x010203","logIndex":"0x1","topics":[],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","transactionLogIndex":"0x1","type":"mined"}],"id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","result":[{"address":"0x0000000000000000000000000000000000000000","blockHash":"0x0000000000000000000000000000000000000000000000000000000000000000","blockNumber":"0x1","data":"0x010203","logIndex":"0x1","topics":[],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","transactionLogIndex":"0x1","type":"mined"}],"id":1}"#;
	let response3 = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request1), Some(response1.to_owned()));
	assert_eq!(tester.io.handle_request_sync(request2), Some(response2.to_owned()));
	assert_eq!(tester.io.handle_request_sync(request3), Some(response3.to_owned()));
}

#[test]
fn rpc_logs_filter() {
	let tester = EthTester::default();
	// Set some logs
	tester.client.set_logs(vec![LocalizedLogEntry {
		block_number: 1,
		block_hash: H256::default(),
		entry: LogEntry {
			address: Address::default(),
			topics: vec![],
			data: vec![1,2,3],
		},
		transaction_index: 0,
		transaction_log_index: 0,
		transaction_hash: H256::default(),
		log_index: 0,
	}, LocalizedLogEntry {
		block_number: 1,
		block_hash: H256::default(),
		entry: LogEntry {
			address: Address::default(),
			topics: vec![],
			data: vec![1,2,3],
		},
		transaction_index: 0,
		transaction_log_index: 1,
		transaction_hash: H256::default(),
		log_index: 1,
	}]);

	// Register filters first
	let request_default = r#"{"jsonrpc": "2.0", "method": "eth_newFilter", "params": [{}], "id": 1}"#;
	let request_limit = r#"{"jsonrpc": "2.0", "method": "eth_newFilter", "params": [{"limit":1}], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":"0x0","id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","result":"0x1","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request_default), Some(response1.to_owned()));
	assert_eq!(tester.io.handle_request_sync(request_limit), Some(response2.to_owned()));

	let request_changes1 = r#"{"jsonrpc": "2.0", "method": "eth_getFilterChanges", "params": ["0x0"], "id": 1}"#;
	let request_changes2 = r#"{"jsonrpc": "2.0", "method": "eth_getFilterChanges", "params": ["0x1"], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":[{"address":"0x0000000000000000000000000000000000000000","blockHash":"0x0000000000000000000000000000000000000000000000000000000000000000","blockNumber":"0x1","data":"0x010203","logIndex":"0x0","topics":[],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","transactionLogIndex":"0x0","type":"mined"},{"address":"0x0000000000000000000000000000000000000000","blockHash":"0x0000000000000000000000000000000000000000000000000000000000000000","blockNumber":"0x1","data":"0x010203","logIndex":"0x1","topics":[],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","transactionLogIndex":"0x1","type":"mined"}],"id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","result":[{"address":"0x0000000000000000000000000000000000000000","blockHash":"0x0000000000000000000000000000000000000000000000000000000000000000","blockNumber":"0x1","data":"0x010203","logIndex":"0x1","topics":[],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","transactionLogIndex":"0x1","type":"mined"}],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request_changes1), Some(response1.to_owned()));
	assert_eq!(tester.io.handle_request_sync(request_changes2), Some(response2.to_owned()));
}

#[test]
fn rpc_blocks_filter() {
	let tester = EthTester::default();
	let request_filter = r#"{"jsonrpc": "2.0", "method": "eth_newBlockFilter", "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request_filter), Some(response.to_owned()));

	let request_changes = r#"{"jsonrpc": "2.0", "method": "eth_getFilterChanges", "params": ["0x0"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request_changes), Some(response.to_owned()));

	tester.client.add_blocks(2, EachBlockWith::Nothing);

	let response = format!(
		r#"{{"jsonrpc":"2.0","result":["0x{:x}","0x{:x}"],"id":1}}"#,
		tester.client.block_hash(BlockId::Number(1)).unwrap(),
		tester.client.block_hash(BlockId::Number(2)).unwrap());

	assert_eq!(tester.io.handle_request_sync(request_changes), Some(response.to_owned()));
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

	let account = tester.accounts_provider.insert_account(Secret::from_slice(&[69u8; 32]), "abcd").unwrap();
	tester.accounts_provider.unlock_account_permanently(account, "abcd".into()).unwrap();
	let _message = "0cc175b9c0f1b6a831c399e26977266192eb5ffee6ae2fec3ad71c777531578f".from_hex().unwrap();

	let req = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sign",
		"params": [
			""#.to_owned() + &format!("0x{:x}", account) + r#"",
			"0x0cc175b9c0f1b6a831c399e26977266192eb5ffee6ae2fec3ad71c777531578f"
		],
		"id": 1
	}"#;
	let res = r#"{"jsonrpc":"2.0","result":"0xa2870db1d0c26ef93c7b72d2a0830fa6b841e0593f7186bc6c7cc317af8cf3a42fda03bd589a49949aa05db83300cdb553116274518dbe9d90c65d0213f4af491b","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&req), Some(res.into()));
}

#[test]
fn rpc_eth_author() {
	let make_res = |addr| r#"{"jsonrpc":"2.0","result":""#.to_owned() + &format!("0x{:x}", addr) + r#"","id":1}"#;
	let tester = EthTester::default();

	let req = r#"{
		"jsonrpc": "2.0",
		"method": "eth_coinbase",
		"params": [],
		"id": 1
	}"#;

	// No accounts - returns zero
	assert_eq!(tester.io.handle_request_sync(req), Some(make_res(Address::zero())));

	// Account set - return first account
	let addr = tester.accounts_provider.new_account("123").unwrap();
	assert_eq!(tester.io.handle_request_sync(req), Some(make_res(addr)));

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
	tester.accounts_provider.set_new_dapps_addresses(None).unwrap();
	tester.accounts_provider.set_address_name(1.into(), "1".into());
	tester.accounts_provider.set_address_name(10.into(), "10".into());

	// with current policy it should return the account
	let request = r#"{"jsonrpc": "2.0", "method": "eth_accounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[""#.to_owned() + &format!("0x{:x}", address) + r#""],"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	tester.accounts_provider.set_new_dapps_addresses(Some(vec![1.into()])).unwrap();
	// even with some account it should return empty list (no dapp detected)
	let request = r#"{"jsonrpc": "2.0", "method": "eth_accounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x0000000000000000000000000000000000000001"],"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// when we add visible address it should return that.
	tester.accounts_provider.set_dapp_addresses("app1".into(), Some(vec![10.into()])).unwrap();
	let request = r#"{"jsonrpc": "2.0", "method": "eth_accounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x000000000000000000000000000000000000000a"],"id":1}"#;
	let mut meta = Metadata::default();
	meta.origin = Origin::Dapps("app1".into());
	assert_eq!((*tester.io).handle_request_sync(request, meta), Some(response.to_owned()));
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

	let response = r#"{"jsonrpc":"2.0","result":"0x5","id":1}"#;

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
fn rpc_eth_transaction_count_next_nonce() {
	let tester = EthTester::new_with_options(EthClientOptions::with(|options| {
		options.pending_nonce_from_queue = true;
	}));
	tester.miner.increment_last_nonce(1.into());

	let request1 = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionCount",
		"params": ["0x0000000000000000000000000000000000000001", "pending"],
		"id": 1
	}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":"0x1","id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request1), Some(response1.to_owned()));

	let request2 = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionCount",
		"params": ["0x0000000000000000000000000000000000000002", "pending"],
		"id": 1
	}"#;
	let response2 = r#"{"jsonrpc":"2.0","result":"0x0","id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request2), Some(response2.to_owned()));
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
	use ethereum_types::H256;
	use rlp;
	use transaction::SignedTransaction;

	let tester = EthTester::default();
	{
		let tx = rlp::decode(&FromHex::from_hex("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap());
		let tx = SignedTransaction::new(tx).unwrap();
		tester.miner.pending_transactions.lock().insert(H256::zero(), tx);
	}

	let response = r#"{"jsonrpc":"2.0","result":{"blockHash":null,"blockNumber":null,"chainId":null,"condition":null,"creates":null,"from":"0x0f65fe9276bc9a24ae7083ae28e2660ef72df99e","gas":"0x5208","gasPrice":"0x1","hash":"0x41df922fd0d4766fcc02e161f8295ec28522f329ae487f14d811e4b64c8d6e31","input":"0x","nonce":"0x0","publicKey":"0x7ae46da747962c2ee46825839c1ef9298e3bd2e70ca2938495c3693a485ec3eaa8f196327881090ff64cf4fbb0a48485d4f83098e189ed3b7a87d5941b59f789","r":"0x48b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353","raw":"0xf85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804","s":"0xefffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804","standardV":"0x0","to":"0x095e7baea6a6c7c4c2dfeb977efac326af552d87","transactionIndex":null,"v":"0x1b","value":"0xa"},"id":1}"#;
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
		exception: None,
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
		exception: None,
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
		exception: None,
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
		exception: None,
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
	let response = r#"{"jsonrpc":"2.0","result":"0x5208","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_eth_estimate_gas_default_block() {
	let tester = EthTester::default();
	tester.client.set_execution_result(Ok(Executed {
		exception: None,
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
	let response = r#"{"jsonrpc":"2.0","result":"0x5208","id":1}"#;

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

	tester.miner.last_nonces.write().insert(address.clone(), U256::zero());

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
	let address = tester.accounts_provider.new_account("").unwrap();
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

	tester.miner.last_nonces.write().insert(address.clone(), U256::zero());

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
			"from": ""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			"to": "",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;

	let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: expected a hex-encoded hash with 0x prefix."},"id":1}"#;

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

#[test]
fn rpc_eth_send_raw_transaction_error() {
	let tester = EthTester::default();

	let req = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendRawTransaction",
		"params": [
			"0x0123"
		],
		"id": 1
	}"#;
	let res = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid RLP.","data":"RlpExpectedToBeList"},"id":1}"#.into();

	assert_eq!(tester.io.handle_request_sync(&req), Some(res));
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
	let signature = tester.accounts_provider.sign(address, None, t.hash(None)).unwrap();
	let t = t.with_signature(signature, None);

	let rlp = rlp::encode(&t).into_vec().to_hex();

	let req = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendRawTransaction",
		"params": [
			"0x"#.to_owned() + &rlp + r#""
		],
		"id": 1
	}"#;

	let res = r#"{"jsonrpc":"2.0","result":""#.to_owned() + &format!("0x{:x}", t.hash()) + r#"","id":1}"#;

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
			transaction_log_index: 0,
			log_index: 1,
		}],
		log_bloom: 0.into(),
		outcome: TransactionOutcome::StateRoot(0.into()),
	};

	let hash = H256::from_str("b903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238").unwrap();
	let tester = EthTester::default();
	tester.client.set_transaction_receipt(TransactionId::Hash(hash), receipt);

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionReceipt",
		"params": ["0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x4510c","contractAddress":null,"cumulativeGasUsed":"0x20","gasUsed":"0x10","logs":[{"address":"0x33990122638b9132ca29c723bdf037f1a891a70c","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x4510c","data":"0x","logIndex":"0x1","topics":["0xa6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc","0x4861736852656700000000000000000000000000000000000000000000000000"],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","transactionLogIndex":"0x0","type":"mined"}],"logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","root":"0x0000000000000000000000000000000000000000000000000000000000000000","status":null,"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0"},"id":1}"#;

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
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32070,"message":"Method deprecated","data":"Compilation functionality is deprecated."},"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[ignore]
#[test]
fn rpc_eth_compile_lll() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_compileLLL", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32070,"message":"Method deprecated","data":"Compilation of LLL via RPC is deprecated"},"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[ignore]
#[test]
fn rpc_eth_compile_solidity() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_compileSolidity", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32070,"message":"Method deprecated","data":"Compilation of Solidity via RPC is deprecated"},"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[ignore]
#[test]
fn rpc_eth_compile_serpent() {
	let request = r#"{"jsonrpc": "2.0", "method": "eth_compileSerpent", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32070,"message":"Method deprecated","data":"Compilation of Serpent via RPC is deprecated"},"id":1}"#;

	assert_eq!(EthTester::default().io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_get_work_returns_no_work_if_cant_mine() {
	let eth_tester = EthTester::default();
	eth_tester.client.set_queue_size(10);

	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32001,"message":"Still syncing."},"id":1}"#;

	assert_eq!(eth_tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_get_work_returns_correct_work_package() {
	let eth_tester = EthTester::default();
	eth_tester.miner.set_author(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap());

	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x76c7bd86693aee93d1a80a408a09a0585b1a1292afcb56192f171d925ea18e2d","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000800000000000000000000000000000000000000000000000000000000000","0x1"],"id":1}"#;

	assert_eq!(eth_tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_get_work_should_not_return_block_number() {
	let eth_tester = EthTester::new_with_options(EthClientOptions::with(|options| {
		options.send_block_number_in_get_work = false;
	}));
	eth_tester.miner.set_author(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap());

	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x76c7bd86693aee93d1a80a408a09a0585b1a1292afcb56192f171d925ea18e2d","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000800000000000000000000000000000000000000000000000000000000000"],"id":1}"#;

	assert_eq!(eth_tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_get_work_should_timeout() {
	let eth_tester = EthTester::default();
	eth_tester.miner.set_author(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap());
	let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 1000;  // Set latest block to 1000 seconds ago
	eth_tester.client.set_latest_block_timestamp(timestamp);
	let hash = eth_tester.miner.map_sealing_work(&*eth_tester.client, |b| b.hash()).unwrap();

	// Request without providing timeout. This should work since we're disabling timeout.
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [], "id": 1}"#;
	let work_response = format!(
		r#"{{"jsonrpc":"2.0","result":["0x{:x}","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000800000000000000000000000000000000000000000000000000000000000","0x1"],"id":1}}"#,
		hash,
	);
	assert_eq!(eth_tester.io.handle_request_sync(request), Some(work_response.to_owned()));

	// Request with timeout of 0 seconds. This should work since we're disabling timeout.
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [0], "id": 1}"#;
	let work_response = format!(
		r#"{{"jsonrpc":"2.0","result":["0x{:x}","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000800000000000000000000000000000000000000000000000000000000000","0x1"],"id":1}}"#,
		hash,
	);
	assert_eq!(eth_tester.io.handle_request_sync(request), Some(work_response.to_owned()));

	// Request with timeout of 10K seconds. This should work.
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [10000], "id": 1}"#;
	assert_eq!(eth_tester.io.handle_request_sync(request), Some(work_response.to_owned()));

	// Request with timeout of 10 seconds. This should fail.
	let request = r#"{"jsonrpc": "2.0", "method": "eth_getWork", "params": [10], "id": 1}"#;
	let err_response = r#"{"jsonrpc":"2.0","error":{"code":-32003,"message":"Work has not changed."},"id":1}"#;
	assert_eq!(eth_tester.io.handle_request_sync(request), Some(err_response.to_owned()));
}
