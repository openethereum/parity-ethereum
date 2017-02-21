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
use util::log::RotatingLogger;
use util::Address;
use ethsync::ManageNetwork;
use ethcore::account_provider::AccountProvider;
use ethcore::client::{TestBlockChainClient};
use ethcore::miner::LocalTransactionStatus;
use ethstore::ethkey::{Generator, Random};

use jsonrpc_core::IoHandler;
use v1::{Parity, ParityClient};
use v1::metadata::Metadata;
use v1::helpers::{SignerService, NetworkSettings};
use v1::tests::helpers::{TestSyncProvider, Config, TestMinerService, TestUpdater};
use super::manage_network::TestManageNetwork;

pub type TestParityClient = ParityClient<TestBlockChainClient, TestMinerService, TestSyncProvider, TestUpdater>;

pub struct Dependencies {
	pub miner: Arc<TestMinerService>,
	pub client: Arc<TestBlockChainClient>,
	pub sync: Arc<TestSyncProvider>,
	pub updater: Arc<TestUpdater>,
	pub logger: Arc<RotatingLogger>,
	pub settings: Arc<NetworkSettings>,
	pub network: Arc<ManageNetwork>,
	pub accounts: Arc<AccountProvider>,
	pub dapps_interface: Option<String>,
	pub dapps_port: Option<u16>,
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
			dapps_interface: Some("127.0.0.1".into()),
			dapps_port: Some(18080),
		}
	}

	pub fn client(&self, signer: Option<Arc<SignerService>>) -> TestParityClient {
		ParityClient::new(
			&self.client,
			&self.miner,
			&self.sync,
			&self.updater,
			&self.network,
			&self.accounts,
			self.logger.clone(),
			self.settings.clone(),
			signer,
			self.dapps_interface.clone(),
			self.dapps_port,
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
	use util::ToPretty;

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
fn rpc_parity_net_peers() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_netPeers", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"active":0,"connected":120,"max":50,"peers":[{"caps":["eth/62","eth/63"],"id":"node1","name":"Parity/1","network":{"localAddress":"127.0.0.1:8888","remoteAddress":"127.0.0.1:7777"},"protocols":{"eth":{"difficulty":"0x28","head":"0000000000000000000000000000000000000000000000000000000000000032","version":62},"les":null}},{"caps":["eth/63","eth/64"],"id":null,"name":"Parity/2","network":{"localAddress":"127.0.0.1:3333","remoteAddress":"Handshake"},"protocols":{"eth":{"difficulty":null,"head":"000000000000000000000000000000000000000000000000000000000000003c","version":64},"les":null}}]},"id":1}"#;

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
	let io = deps.with_signer(SignerService::new_test(Some(("127.0.0.1".into(), 18180))));

	let request = r#"{"jsonrpc": "2.0", "method": "parity_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_unsigned_transactions_count_when_signer_disabled() {
	let deps = Dependencies::new();
	let io = deps.default_client();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_unsignedTransactionsCount", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32030,"message":"Trusted Signer is disabled. This API is not available.","data":null},"id":1}"#;

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
fn rpc_parity_signer_port() {
	// given
	let deps = Dependencies::new();
	let io1 = deps.with_signer(SignerService::new_test(Some(("127.0.0.1".into(), 18180))));
	let io2 = deps.default_client();

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "parity_signerPort", "params": [], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":18180,"id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","error":{"code":-32030,"message":"Trusted Signer is disabled. This API is not available.","data":null},"id":1}"#;

	// then
	assert_eq!(io1.handle_request_sync(request), Some(response1.to_owned()));
	assert_eq!(io2.handle_request_sync(request), Some(response2.to_owned()));
}

#[test]
fn rpc_parity_dapps_port() {
	// given
	let mut deps = Dependencies::new();
	let io1 = deps.default_client();
	deps.dapps_port = None;
	let io2 = deps.default_client();

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "parity_dappsPort", "params": [], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":18080,"id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","error":{"code":-32031,"message":"Dapps Server is disabled. This API is not available.","data":null},"id":1}"#;

	// then
	assert_eq!(io1.handle_request_sync(request), Some(response1.to_owned()));
	assert_eq!(io2.handle_request_sync(request), Some(response2.to_owned()));
}

#[test]
fn rpc_parity_dapps_interface() {
	// given
	let mut deps = Dependencies::new();
	let io1 = deps.default_client();
	deps.dapps_interface = None;
	let io2 = deps.default_client();

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "parity_dappsInterface", "params": [], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":"127.0.0.1","id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","error":{"code":-32031,"message":"Dapps Server is disabled. This API is not available.","data":null},"id":1}"#;

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
	use util::{H256, U256};

	let deps = Dependencies::new();
	let io = deps.default_client();

	*deps.client.ancient_block.write() = Some((H256::default(), 5));
	*deps.client.first_block.write() = Some((H256::from(U256::from(1234)), 3333));

	let request = r#"{"jsonrpc": "2.0", "method": "parity_chainStatus", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"blockGap":["0x6","0xd05"]},"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}
