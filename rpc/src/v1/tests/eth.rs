// Copyright 2016 Ethcore (UK) Ltd.
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

//! rpc integration tests.
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;

use ethcore::client::{MiningBlockChainClient, Client, ClientConfig};
use ethcore::spec::Genesis;
use ethcore::block::Block;
use ethcore::ethereum;
use ethcore::transaction::{Transaction, Action};
use ethcore::miner::{MinerService, ExternalMiner};
use devtools::RandomTempPath;
use util::io::IoChannel;
use util::hash::Address;
use util::numbers::{Uint, U256};
use util::keys::{AccountProvider, TestAccount, TestAccountProvider};
use jsonrpc_core::IoHandler;
use ethjson::blockchain::BlockChain;

use v1::traits::eth::Eth;
use v1::impls::EthClient;
use v1::tests::helpers::{TestSyncProvider, Config, TestMinerService};

struct EthTester {
	_client: Arc<MiningBlockChainClient>,
	_miner: Arc<MinerService>,
	accounts: Arc<TestAccountProvider>,
	handler: IoHandler,
}

#[test]
fn harness_works() {
	let chain: BlockChain = extract_chain!("BlockchainTests/bcUncleTest");
	chain_harness(chain, |_| {});
}

#[test]
fn eth_get_balance() {
	let chain = extract_chain!("BlockchainTests/bcWalletTest", "wallet2outOf3txs");
	chain_harness(chain, |tester| {
		// final account state
		let req_latest = r#"{
			"jsonrpc": "2.0",
			"method": "eth_getBalance",
			"params": ["0xaaaf5374fce5edbc8e2a8697c15331677e6ebaaa", "latest"],
			"id": 1
		}"#;
		let res_latest = r#"{"jsonrpc":"2.0","result":"0x09","id":1}"#.to_owned();
		assert_eq!(tester.handler.handle_request(req_latest).unwrap(), res_latest);

		// non-existant account
		let req_new_acc = r#"{
			"jsonrpc": "2.0",
			"method": "eth_getBalance",
			"params": ["0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
			"id": 3
		}"#;

		let res_new_acc = r#"{"jsonrpc":"2.0","result":"0x00","id":3}"#.to_owned();
		assert_eq!(tester.handler.handle_request(req_new_acc).unwrap(), res_new_acc);
	});
}

#[test]
fn eth_block_number() {
	let chain = extract_chain!("BlockchainTests/bcRPC_API_Test");
	chain_harness(chain, |tester| {
		let req_number = r#"{
			"jsonrpc": "2.0",
			"method": "eth_blockNumber",
			"params": [],
			"id": 1
		}"#;

		let res_number = r#"{"jsonrpc":"2.0","result":"0x20","id":1}"#.to_owned();
		assert_eq!(tester.handler.handle_request(req_number).unwrap(), res_number);
	});
}

#[cfg(test)]
#[test]
fn eth_transaction_count() {
	let chain = extract_chain!("BlockchainTests/bcRPC_API_Test");
	chain_harness(chain, |tester| {
		let address = tester.accounts.new_account("123").unwrap();
		let secret = tester.accounts.account_secret(&address).unwrap();

		let req_before = r#"{
			"jsonrpc": "2.0",
			"method": "eth_getTransactionCount",
			"params": [""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"", "latest"],
			"id": 15
		}"#;

		let res_before = r#"{"jsonrpc":"2.0","result":"0x00","id":15}"#;

		assert_eq!(tester.handler.handle_request(&req_before).unwrap(), res_before);

		let t = Transaction {
			nonce: U256::zero(),
			gas_price: U256::from(0x9184e72a000u64),
			gas: U256::from(0x76c0),
			action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
			value: U256::from(0x9184e72au64),
			data: vec![]
		}.sign(&secret);

		let req_send_trans = r#"{
			"jsonrpc": "2.0",
			"method": "eth_sendTransaction",
			"params": [{
				"from": ""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
				"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
				"gas": "0x76c0",
				"gasPrice": "0x9184e72a000",
				"value": "0x9184e72a"
			}],
			"id": 16
		}"#;

		let res_send_trans = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":16}"#;

		// dispatch the transaction.
		assert_eq!(tester.handler.handle_request(&req_send_trans).unwrap(), res_send_trans);

		// we have submitted the transaction -- but this shouldn't be reflected in a "latest" query.
		let req_after_latest = r#"{
			"jsonrpc": "2.0",
			"method": "eth_getTransactionCount",
			"params": [""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"", "latest"],
			"id": 17
		}"#;

		let res_after_latest = r#"{"jsonrpc":"2.0","result":"0x00","id":17}"#;

		assert_eq!(&tester.handler.handle_request(&req_after_latest).unwrap(), res_after_latest);

		// the pending transactions should have been updated.
		let req_after_pending = r#"{
			"jsonrpc": "2.0",
			"method": "eth_getTransactionCount",
			"params": [""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"", "pending"],
			"id": 18
		}"#;

		let res_after_pending = r#"{"jsonrpc":"2.0","result":"0x01","id":18}"#;

		assert_eq!(&tester.handler.handle_request(&req_after_pending).unwrap(), res_after_pending);
	});
}

fn account_provider() -> Arc<TestAccountProvider> {
	let mut accounts = HashMap::new();
	accounts.insert(Address::from(1), TestAccount::new("test"));
	let ap = TestAccountProvider::new(accounts);
	Arc::new(ap)
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

// given a blockchain, this harness will create an EthClient wrapping it
// which tests can pass specially crafted requests to.
fn chain_harness<F, U>(chain: BlockChain, mut cb: F) -> U
	where F: FnMut(&EthTester) -> U {
	let genesis = Genesis::from(chain.genesis());
	let mut spec = ethereum::new_frontier_test();
	let state = chain.pre_state.clone().into();
	spec.set_genesis_state(state);
	spec.overwrite_genesis_params(genesis);
	assert!(spec.is_state_root_valid());

	let dir = RandomTempPath::new();
	let client = Client::new(ClientConfig::default(), spec, dir.as_path(), IoChannel::disconnected()).unwrap();
	let sync_provider = sync_provider();
	let miner_service = miner_service();
	let account_provider = account_provider();
	let external_miner = Arc::new(ExternalMiner::default());

	for b in &chain.blocks_rlp() {
		if Block::is_good(&b) {
			let _ = client.import_block(b.clone());
			client.flush_queue();
			client.import_verified_blocks(&IoChannel::disconnected());
		}
	}

	assert!(client.chain_info().best_block_hash == chain.best_block.into());

	let eth_client = EthClient::new(&client, &sync_provider, &account_provider,
		&miner_service, &external_miner);

	let handler = IoHandler::new();
	let delegate = eth_client.to_delegate();
	handler.add_delegate(delegate);

	let tester = EthTester {
		_miner: miner_service,
		_client: client,
		accounts: account_provider,
		handler: handler,
	};

	cb(&tester)
}
