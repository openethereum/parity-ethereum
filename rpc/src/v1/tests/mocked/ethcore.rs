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
use ethstore::ethkey::{Generator, Random};

use jsonrpc_core::IoHandler;
use v1::{Ethcore, EthcoreClient};
use v1::helpers::{SignerService, NetworkSettings};
use v1::tests::helpers::{TestSyncProvider, Config, TestMinerService, TestFetch};
use super::manage_network::TestManageNetwork;


pub type TestEthcoreClient = EthcoreClient<TestBlockChainClient, TestMinerService, TestSyncProvider, TestFetch>;

pub struct Dependencies {
	pub miner: Arc<TestMinerService>,
	pub client: Arc<TestBlockChainClient>,
	pub sync: Arc<TestSyncProvider>,
	pub logger: Arc<RotatingLogger>,
	pub settings: Arc<NetworkSettings>,
	pub network: Arc<ManageNetwork>,
	pub dapps_port: Option<u16>,
}

impl Dependencies {
	pub fn new() -> Self {
		Dependencies {
			miner: Arc::new(TestMinerService::default()),
			client: Arc::new(TestBlockChainClient::default()),
			sync: Arc::new(TestSyncProvider::new(Config {
				network_id: U256::from(3),
				num_peers: 120,
			})),
			logger: Arc::new(RotatingLogger::new("rpc=trace".to_owned())),
			settings: Arc::new(NetworkSettings {
				name: "mynode".to_owned(),
				chain: "testchain".to_owned(),
				network_port: 30303,
				rpc_enabled: true,
				rpc_interface: "all".to_owned(),
				rpc_port: 8545,
			}),
			network: Arc::new(TestManageNetwork),
			dapps_port: Some(18080),
		}
	}

	pub fn client(&self, signer: Option<Arc<SignerService>>) -> TestEthcoreClient {
		EthcoreClient::with_fetch(
			&self.client,
			&self.miner,
			&self.sync,
			&self.network,
			self.logger.clone(),
			self.settings.clone(),
			signer,
			self.dapps_port,
		)
	}

	fn default_client(&self) -> IoHandler {
		let io = IoHandler::new();
		io.add_delegate(self.client(None).to_delegate());
		io
	}

	fn with_signer(&self, signer: SignerService) -> IoHandler {
		let io = IoHandler::new();
		io.add_delegate(self.client(Some(Arc::new(signer))).to_delegate());
		io
	}
}

#[test]
fn rpc_ethcore_extra_data() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_extraData", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x01020304","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_default_extra_data() {
	use util::misc;
	use util::ToPretty;

	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_defaultExtraData", "params": [], "id": 1}"#;
	let response = format!(r#"{{"jsonrpc":"2.0","result":"0x{}","id":1}}"#, misc::version_data().to_hex());

	assert_eq!(io.handle_request_sync(request), Some(response));
}

#[test]
fn rpc_ethcore_gas_floor_target() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_gasFloorTarget", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x3039","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_min_gas_price() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_minGasPrice", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1312d00","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_dev_logs() {
	let deps = Dependencies::new();
	deps.logger.append("a".to_owned());
	deps.logger.append("b".to_owned());

	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_devLogs", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["b","a"],"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_dev_logs_levels() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_devLogsLevels", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"rpc=trace","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_transactions_limit() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_transactionsLimit", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":1024,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_chain() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netChain", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"testchain","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_peers() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netPeers", "params":[], "id": 1}"#;
	let response = "{\"jsonrpc\":\"2.0\",\"result\":{\"active\":0,\"connected\":120,\"max\":50,\"peers\":[{\"caps\":[\"eth/62\",\"eth/63\"],\
\"id\":\"node1\",\"name\":\"Parity/1\",\"network\":{\"localAddress\":\"127.0.0.1:8888\",\"remoteAddress\":\"127.0.0.1:7777\"}\
,\"protocols\":{\"eth\":{\"difficulty\":\"0x28\",\"head\":\"0000000000000000000000000000000000000000000000000000000000000032\"\
,\"version\":62}}},{\"caps\":[\"eth/63\",\"eth/64\"],\"id\":null,\"name\":\"Parity/2\",\"network\":{\"localAddress\":\
\"127.0.0.1:3333\",\"remoteAddress\":\"Handshake\"},\"protocols\":{\"eth\":{\"difficulty\":null,\"head\":\
\"000000000000000000000000000000000000000000000000000000000000003c\",\"version\":64}}}]},\"id\":1}";

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_net_port() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_netPort", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":30303,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_rpc_settings() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_rpcSettings", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"enabled":true,"interface":"all","port":8545},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_node_name() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_nodeName", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"mynode","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_unsigned_transactions_count() {
	let deps = Dependencies::new();
	let io = deps.with_signer(SignerService::new_test(Some(18180)));

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_unsigned_transactions_count_when_signer_disabled() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32030,"message":"Trusted Signer is disabled. This API is not available.","data":null},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_hash_content() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_hashContent", "params":["https://ethcore.io/assets/images/ethcore-black-horizontal.png"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_pending_transactions() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_pendingTransactions", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_encrypt() {
	let deps = Dependencies::new();
	let io = deps.default_client();
	let key = format!("{:?}", Random.generate().unwrap().public());

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_encryptMessage", "params":["0x"#.to_owned() + &key + r#"", "0x01"], "id": 1}"#;
	assert!(io.handle_request_sync(&request).unwrap().contains("result"), "Should return success.");
}

#[test]
fn rpc_ethcore_signer_port() {
	// given
	let deps = Dependencies::new();
	let io1 = deps.with_signer(SignerService::new_test(Some(18180)));
	let io2 = deps.default_client();

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_signerPort", "params": [], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":18180,"id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","error":{"code":-32030,"message":"Trusted Signer is disabled. This API is not available.","data":null},"id":1}"#;

	// then
	assert_eq!(io1.handle_request_sync(request), Some(response1.to_owned()));
	assert_eq!(io2.handle_request_sync(request), Some(response2.to_owned()));
}

#[test]
fn rpc_ethcore_dapps_port() {
	// given
	let mut deps = Dependencies::new();
	let io1 = deps.default_client();
	deps.dapps_port = None;
	let io2 = deps.default_client();

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_dappsPort", "params": [], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":18080,"id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","error":{"code":-32031,"message":"Dapps Server is disabled. This API is not available.","data":null},"id":1}"#;

	// then
	assert_eq!(io1.handle_request_sync(request), Some(response1.to_owned()));
	assert_eq!(io2.handle_request_sync(request), Some(response2.to_owned()));
}
