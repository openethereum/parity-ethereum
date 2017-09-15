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
use ethcore::account_provider::AccountProvider;
use ethcore::client::{TestBlockChainClient, Executed};
use ethcore::miner::LocalTransactionStatus;
use ethcore_logger::RotatingLogger;
use ethstore::ethkey::{Generator, Random};
use ethsync::ManageNetwork;
use node_health::{self, NodeHealth};
use parity_reactor;
use util::Address;

use jsonrpc_core::IoHandler;
use v1::{Parity, ParityClient};
use v1::metadata::Metadata;
use v1::helpers::{SignerService, NetworkSettings};
use v1::tests::helpers::{TestSyncProvider, Config, TestMinerService, TestUpdater};
use super::manage_network::TestManageNetwork;

pub type TestParityClient = ParityClient<TestBlockChainClient, TestMinerService, TestUpdater>;

pub struct Dependencies {
	pub miner: Arc<TestMinerService>,
	pub client: Arc<TestBlockChainClient>,
	pub sync: Arc<TestSyncProvider>,
	pub updater: Arc<TestUpdater>,
	pub health: NodeHealth,
	pub logger: Arc<RotatingLogger>,
	pub settings: Arc<NetworkSettings>,
	pub network: Arc<ManageNetwork>,
	pub accounts: Arc<AccountProvider>,
	pub dapps_address: Option<(String, u16)>,
	pub ws_address: Option<(String, u16)>,
}

impl Dependencies {
	pub fn new() -> Self {
		Dependencies {
			miner: Arc::new(TestMinerService::default()),
			client: Arc::new(TestBlockChainClient::default()),
			sync: Arc::new(TestSyncProvider::new(Config {
				network_id: 3,
				num_peers: 120,
			})),
			health: NodeHealth::new(
				Arc::new(FakeSync),
				node_health::TimeChecker::new::<String>(&[], node_health::CpuPool::new(1)),
				parity_reactor::Remote::new_sync(),
			),
			updater: Arc::new(TestUpdater::default()),
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
			accounts: Arc::new(AccountProvider::transient_provider()),
			dapps_address: Some(("127.0.0.1".into(), 18080)),
			ws_address: Some(("127.0.0.1".into(), 18546)),
		}
	}

	pub fn client(&self, signer: Option<Arc<SignerService>>) -> TestParityClient {
		let opt_accounts = Some(self.accounts.clone());

		ParityClient::new(
			self.client.clone(),
			self.miner.clone(),
			self.sync.clone(),
			self.updater.clone(),
			self.network.clone(),
			self.health.clone(),
			opt_accounts.clone(),
			self.logger.clone(),
			self.settings.clone(),
			signer,
			self.dapps_address.clone(),
			self.ws_address.clone(),
		)
	}

	fn default_client(&self) -> IoHandler<Metadata> {
		let mut io = IoHandler::default();
		io.extend_with(self.client(None).to_delegate());
		io
	}

	fn with_signer(&self, signer: SignerService) -> IoHandler<Metadata> {
		let mut io = IoHandler::default();
		io.extend_with(self.client(Some(Arc::new(signer))).to_delegate());
		io
	}
}

#[derive(Debug)]
struct FakeSync;
impl node_health::SyncStatus for FakeSync {
	fn is_major_importing(&self) -> bool { false }
	fn peers(&self) -> (usize, usize) { (4, 25) }
}

#[test]
fn rpc_parity_accounts_info() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	deps.accounts.new_account("").unwrap();
	let accounts = deps.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	deps.accounts.set_address_name(1.into(), "XX".into());
	deps.accounts.set_account_name(address.clone(), "Test".into()).unwrap();
	deps.accounts.set_account_meta(address.clone(), "{foo: 69}".into()).unwrap();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_accountsInfo", "params": [], "id": 1}"#;
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"0x{}\":{{\"name\":\"Test\"}}}},\"id\":1}}", address.hex());
	assert_eq!(io.handle_request_sync(request), Some(response));

	// Change the whitelist
	let address = Address::from(1);
	deps.accounts.set_new_dapps_addresses(Some(vec![address.clone()])).unwrap();
	let request = r#"{"jsonrpc": "2.0", "method": "parity_accountsInfo", "params": [], "id": 1}"#;
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"0x{}\":{{\"name\":\"XX\"}}}},\"id\":1}}", address.hex());
	assert_eq!(io.handle_request_sync(request), Some(response));
}

#[test]
fn rpc_parity_default_account() {
	let deps = Dependencies::new();
	let io = deps.default_client();


	// Check empty
	let address = Address::default();
	let request = r#"{"jsonrpc": "2.0", "method": "parity_defaultAccount", "params": [], "id": 1}"#;
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":\"0x{}\",\"id\":1}}", address.hex());
	assert_eq!(io.handle_request_sync(request), Some(response));

	// With account
	deps.accounts.new_account("").unwrap();
	let accounts = deps.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	let request = r#"{"jsonrpc": "2.0", "method": "parity_defaultAccount", "params": [], "id": 1}"#;
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":\"0x{}\",\"id\":1}}", address.hex());
	assert_eq!(io.handle_request_sync(request), Some(response));
}

#[test]
fn rpc_parity_consensus_capability() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_consensusCapability", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"capableUntil":15100},"id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));

	deps.updater.set_current_block(15101);

	let request = r#"{"jsonrpc": "2.0", "method": "parity_consensusCapability", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"incapableSince":15100},"id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));

	deps.updater.set_updated(true);

	let request = r#"{"jsonrpc": "2.0", "method": "parity_consensusCapability", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"capable","id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_version_info() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_versionInfo", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"hash":"0x0000000000000000000000000000000000000096","track":"beta","version":{"major":1,"minor":5,"patch":0}},"id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_releases_info() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_releasesInfo", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"fork":15100,"minor":null,"this_fork":15000,"track":{"binary":"0x00000000000000000000000000000000000000000000000000000000000005e6","fork":15100,"is_critical":true,"version":{"hash":"0x0000000000000000000000000000000000000097","track":"beta","version":{"major":1,"minor":5,"patch":1}}}},"id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_extra_data() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_extraData", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x01020304","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_default_extra_data() {
	use util::misc;
	use bytes::ToPretty;

	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_defaultExtraData", "params": [], "id": 1}"#;
	let response = format!(r#"{{"jsonrpc":"2.0","result":"0x{}","id":1}}"#, misc::version_data().to_hex());

	assert_eq!(io.handle_request_sync(request), Some(response));
}

#[test]
fn rpc_parity_gas_floor_target() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_gasFloorTarget", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x3039","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_min_gas_price() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_minGasPrice", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1312d00","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_dev_logs() {
	let deps = Dependencies::new();
	deps.logger.append("a".to_owned());
	deps.logger.append("b".to_owned());

	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_devLogs", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["b","a"],"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_dev_logs_levels() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_devLogsLevels", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"rpc=trace","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_transactions_limit() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_transactionsLimit", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":1024,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_net_chain() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_netChain", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"testchain","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_chain() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_chain", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"foundation","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_net_peers() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_netPeers", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"active":0,"connected":120,"max":50,"peers":[{"caps":["eth/62","eth/63"],"id":"node1","name":"Parity/1","network":{"localAddress":"127.0.0.1:8888","remoteAddress":"127.0.0.1:7777"},"protocols":{"eth":{"difficulty":"0x28","head":"0000000000000000000000000000000000000000000000000000000000000032","version":62},"pip":null}},{"caps":["eth/63","eth/64"],"id":null,"name":"Parity/2","network":{"localAddress":"127.0.0.1:3333","remoteAddress":"Handshake"},"protocols":{"eth":{"difficulty":null,"head":"000000000000000000000000000000000000000000000000000000000000003c","version":64},"pip":null}}]},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_net_port() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_netPort", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":30303,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_rpc_settings() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_rpcSettings", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"enabled":true,"interface":"all","port":8545},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_node_name() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_nodeName", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"mynode","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_unsigned_transactions_count() {
	let deps = Dependencies::new();
	let io = deps.with_signer(SignerService::new_test(true));

	let request = r#"{"jsonrpc": "2.0", "method": "parity_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_unsigned_transactions_count_when_signer_disabled() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Trusted Signer is disabled. This API is not available."},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_pending_transactions() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_pendingTransactions", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_encrypt() {
	let deps = Dependencies::new();
	let io = deps.default_client();
	let key = format!("{:?}", Random.generate().unwrap().public());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_encryptMessage", "params":["0x"#.to_owned() + &key + r#"", "0x01"], "id": 1}"#;
	assert!(io.handle_request_sync(&request).unwrap().contains("result"), "Should return success.");
}

#[test]
fn rpc_parity_ws_address() {
	// given
	let mut deps = Dependencies::new();
	let io1 = deps.default_client();
	deps.ws_address = None;
	let io2 = deps.default_client();

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "parity_wsUrl", "params": [], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":"127.0.0.1:18546","id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"WebSockets Server is disabled. This API is not available."},"id":1}"#;

	// then
	assert_eq!(io1.handle_request_sync(request), Some(response1.to_owned()));
	assert_eq!(io2.handle_request_sync(request), Some(response2.to_owned()));
}

#[test]
fn rpc_parity_dapps_address() {
	// given
	let mut deps = Dependencies::new();
	let io1 = deps.default_client();
	deps.dapps_address = None;
	let io2 = deps.default_client();

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "parity_dappsUrl", "params": [], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":"127.0.0.1:18080","id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Dapps Server is disabled. This API is not available."},"id":1}"#;

	// then
	assert_eq!(io1.handle_request_sync(request), Some(response1.to_owned()));
	assert_eq!(io2.handle_request_sync(request), Some(response2.to_owned()));
}

#[test]
fn rpc_parity_next_nonce() {
	let deps = Dependencies::new();
	let address = Address::default();
	let io1 = deps.default_client();
	let deps = Dependencies::new();
	deps.miner.last_nonces.write().insert(address.clone(), 2.into());
	let io2 = deps.default_client();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "parity_nextNonce",
		"params": [""#.to_owned() + &format!("0x{:?}", address) + r#""],
		"id": 1
	}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":"0x0","id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","result":"0x3","id":1}"#;

	assert_eq!(io1.handle_request_sync(&request), Some(response1.to_owned()));
	assert_eq!(io2.handle_request_sync(&request), Some(response2.to_owned()));
}

#[test]
fn rpc_parity_transactions_stats() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_pendingTransactionsStats", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"0x0000000000000000000000000000000000000000000000000000000000000001":{"firstSeen":10,"propagatedTo":{"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080":16}},"0x0000000000000000000000000000000000000000000000000000000000000005":{"firstSeen":16,"propagatedTo":{"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010":1}}},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_local_transactions() {
	let deps = Dependencies::new();
	let io = deps.default_client();
	deps.miner.local_transactions.lock().insert(10.into(), LocalTransactionStatus::Pending);
	deps.miner.local_transactions.lock().insert(15.into(), LocalTransactionStatus::Future);

	let request = r#"{"jsonrpc": "2.0", "method": "parity_localTransactions", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"0x000000000000000000000000000000000000000000000000000000000000000a":{"status":"pending"},"0x000000000000000000000000000000000000000000000000000000000000000f":{"status":"future"}},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_chain_status() {
	use bigint::prelude::U256;
	use bigint::hash::H256;

	let deps = Dependencies::new();
	let io = deps.default_client();

	*deps.client.ancient_block.write() = Some((H256::default(), 5));
	*deps.client.first_block.write() = Some((H256::from(U256::from(1234)), 3333));

	let request = r#"{"jsonrpc": "2.0", "method": "parity_chainStatus", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"blockGap":["0x6","0xd05"]},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_node_kind() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_nodeKind", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"availability":"personal","capability":"full"},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_cid() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_cidV0", "params":["0x414243"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"QmSF59MAENc8ZhM4aM1thuAE8w5gDmyfzkAvNoyPea7aDz","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_call() {
	use bigint::prelude::U256;

	let deps = Dependencies::new();
	deps.client.set_execution_result(Ok(Executed {
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
	let io = deps.default_client();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "parity_call",
		"params": [[{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		}],
		"latest"],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x1234ff"],"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_node_health() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_nodeHealth", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"peers":{"details":[4,25],"message":"","status":"ok"},"sync":{"details":false,"message":"","status":"ok"},"time":{"details":0,"message":"","status":"ok"}},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}
