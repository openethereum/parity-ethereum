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
use jsonrpc_core::IoHandler;
use v1::{Ethcore, EthcoreClient};
use v1::tests::helpers::TestMinerService;
use v1::helpers::ConfirmationsQueue;
use ethcore::client::{TestBlockChainClient};
use util::log::RotatingLogger;
use util::network_settings::NetworkSettings;

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

fn client_service() -> Arc<TestBlockChainClient> {
	Arc::new(TestBlockChainClient::default())
}

fn logger() -> Arc<RotatingLogger> {
	Arc::new(RotatingLogger::new("rpc=trace".to_owned()))
}

fn settings() -> Arc<NetworkSettings> {
	Arc::new(NetworkSettings {
		name: "mynode".to_owned(),
		chain: "testchain".to_owned(),
		max_peers: 25,
		network_port: 30303,
		rpc_enabled: true,
		rpc_interface: "all".to_owned(),
		rpc_port: 8545,
	})
}

fn ethcore_client(client: &Arc<TestBlockChainClient>, miner: &Arc<TestMinerService>) -> EthcoreClient<TestBlockChainClient, TestMinerService> {
	EthcoreClient::new(client, miner, logger(), settings(), None)
}

#[test]
fn rpc_ethcore_extra_data() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_extraData", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x01020304","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_default_extra_data() {
	use util::misc;
	use util::ToPretty;

	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_defaultExtraData", "params": [], "id": 1}"#;
	let response = format!(r#"{{"jsonrpc":"2.0","result":"0x{}","id":1}}"#, misc::version_data().to_hex());

	assert_eq!(io.handle_request(request), Some(response));
}

#[test]
fn rpc_ethcore_gas_floor_target() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_gasFloorTarget", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x3039","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_min_gas_price() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_minGasPrice", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x01312d00","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_dev_logs() {
	let miner = miner_service();
	let client = client_service();
	let logger = logger();
	logger.append("a".to_owned());
	logger.append("b".to_owned());
	let ethcore = EthcoreClient::new(&client, &miner, logger.clone(), settings(), None).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_devLogs", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["b","a"],"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_dev_logs_levels() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_devLogsLevels", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"rpc=trace","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_transactions_limit() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_transactionsLimit", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":1024,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_chain() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netChain", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"testchain","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_max_peers() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netMaxPeers", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":25,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_port() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netPort", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":30303,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_rpc_settings() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_rpcSettings", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"enabled":true,"interface":"all","port":8545},"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_node_name() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_nodeName", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"mynode","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_unsigned_transactions_count() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	let queue = Arc::new(ConfirmationsQueue::default());
	let ethcore = EthcoreClient::new(&client, &miner, logger(), settings(), Some(queue)).to_delegate();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_unsigned_transactions_count_when_signer_disabled() {
	let miner = miner_service();
	let client = client_service();
	let io = IoHandler::new();
	io.add_delegate(ethcore_client(&client, &miner).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32030,"message":"Trusted Signer is disabled. This API is not available.","data":null},"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}
