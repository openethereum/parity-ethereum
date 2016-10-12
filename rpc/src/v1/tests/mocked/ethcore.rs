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

use std::sync::Arc;
use util::log::RotatingLogger;
use util::U256;
use ethsync::ManageNetwork;
use ethcore::client::{TestBlockChainClient};

use jsonrpc_core::IoHandler;
use v1::{Ethcore, EthcoreClient};
use v1::helpers::{SignerService, NetworkSettings};
use v1::tests::helpers::{TestSyncProvider, Config, TestMinerService, TestFetch};
use super::manage_network::TestManageNetwork;

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

fn client_service() -> Arc<TestBlockChainClient> {
	Arc::new(TestBlockChainClient::default())
}

fn sync_provider() -> Arc<TestSyncProvider> {
	Arc::new(TestSyncProvider::new(Config {
		network_id: U256::from(3),
		num_peers: 120,
	}))
}

fn logger() -> Arc<RotatingLogger> {
	Arc::new(RotatingLogger::new("rpc=trace".to_owned()))
}

fn settings() -> Arc<NetworkSettings> {
	Arc::new(NetworkSettings {
		name: "mynode".to_owned(),
		chain: "testchain".to_owned(),
		network_port: 30303,
		rpc_enabled: true,
		rpc_interface: "all".to_owned(),
		rpc_port: 8545,
	})
}

fn network_service() -> Arc<ManageNetwork> {
	Arc::new(TestManageNetwork)
}

type TestEthcoreClient = EthcoreClient<TestBlockChainClient, TestMinerService, TestSyncProvider, TestFetch>;

fn ethcore_client(
	client: &Arc<TestBlockChainClient>,
	miner: &Arc<TestMinerService>,
	sync: &Arc<TestSyncProvider>,
	net: &Arc<ManageNetwork>)
	-> TestEthcoreClient {
	EthcoreClient::with_fetch(client, miner, sync, net, logger(), settings(), None)
}

#[test]
fn rpc_ethcore_extra_data() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_extraData", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x01020304","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_default_extra_data() {
	use util::misc;
	use util::ToPretty;

	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_defaultExtraData", "params": [], "id": 1}"#;
	let response = format!(r#"{{"jsonrpc":"2.0","result":"0x{}","id":1}}"#, misc::version_data().to_hex());

	assert_eq!(io.handle_request_sync(request), Some(response));
}

#[test]
fn rpc_ethcore_gas_floor_target() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_gasFloorTarget", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x3039","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_min_gas_price() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_minGasPrice", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1312d00","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_dev_logs() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let logger = logger();
	logger.append("a".to_owned());
	logger.append("b".to_owned());
	let ethcore: TestEthcoreClient = EthcoreClient::with_fetch(&client, &miner, &sync, &net, logger.clone(), settings(), None);
	let io = IoHandler::new();
	io.add_delegate(ethcore.to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_devLogs", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["b","a"],"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_dev_logs_levels() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_devLogsLevels", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"rpc=trace","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_transactions_limit() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_transactionsLimit", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":1024,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_chain() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netChain", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"testchain","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_peers() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netPeers", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"active":0,"connected":120,"max":50},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_port() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netPort", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":30303,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_rpc_settings() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_rpcSettings", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"enabled":true,"interface":"all","port":8545},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_node_name() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_nodeName", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"mynode","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_unsigned_transactions_count() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	let signer = Arc::new(SignerService::new_test());
	let ethcore: TestEthcoreClient = EthcoreClient::with_fetch(&client, &miner, &sync, &net, logger(), settings(), Some(signer));
	io.add_delegate(ethcore.to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_unsigned_transactions_count_when_signer_disabled() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32030,"message":"Trusted Signer is disabled. This API is not available.","data":null},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_hash_content() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_hashContent", "params":["https://ethcore.io/assets/images/ethcore-black-horizontal.png"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_pending_transactions() {
	let miner = miner_service();
	let client = client_service();
	let sync = sync_provider();
	let net = network_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner, &sync, &net).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_pendingTransactions", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}
