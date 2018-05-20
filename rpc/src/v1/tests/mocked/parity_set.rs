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

use std::sync::Arc;
use std::str::FromStr;
use rustc_hex::FromHex;
use ethereum_types::{U256, Address};

use ethcore::miner::MinerService;
use ethcore::client::TestBlockChainClient;
use sync::ManageNetwork;
use futures_cpupool::CpuPool;

use jsonrpc_core::IoHandler;
use v1::{ParitySet, ParitySetClient};
use v1::tests::helpers::{TestMinerService, TestUpdater, TestDappsService};
use super::manage_network::TestManageNetwork;

use fake_fetch::FakeFetch;

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

fn client_service() -> Arc<TestBlockChainClient> {
	Arc::new(TestBlockChainClient::default())
}

fn network_service() -> Arc<TestManageNetwork> {
	Arc::new(TestManageNetwork)
}

fn updater_service() -> Arc<TestUpdater> {
	Arc::new(TestUpdater::default())
}

pub type TestParitySetClient = ParitySetClient<TestBlockChainClient, TestMinerService, TestUpdater, FakeFetch<usize>>;

fn parity_set_client(
	client: &Arc<TestBlockChainClient>,
	miner: &Arc<TestMinerService>,
	updater: &Arc<TestUpdater>,
	net: &Arc<TestManageNetwork>,
) -> TestParitySetClient {
	let dapps_service = Arc::new(TestDappsService);
	let pool = CpuPool::new(1);
	ParitySetClient::new(client, miner, updater, &(net.clone() as Arc<ManageNetwork>), Some(dapps_service), FakeFetch::new(Some(1)), pool)
}

#[test]
fn rpc_parity_execute_upgrade() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();
	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_executeUpgrade", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));

	let request = r#"{"jsonrpc": "2.0", "method": "parity_executeUpgrade", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":false,"id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_upgrade_ready() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();
	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_upgradeReady", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"binary":"0x00000000000000000000000000000000000000000000000000000000000005e6","fork":15100,"is_critical":true,"version":{"hash":"0x0000000000000000000000000000000000000097","track":"beta","version":{"major":1,"minor":5,"patch":1}}},"id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));

	updater.set_updated(true);

	let request = r#"{"jsonrpc": "2.0", "method": "parity_upgradeReady", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;
	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_set_min_gas_price() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();

	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setMinGasPrice", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":false,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_set_gas_floor_target() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();

	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setGasFloorTarget", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.authoring_params().gas_range_target.0, U256::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn rpc_parity_set_extra_data() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();

	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setExtraData", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.authoring_params().extra_data, "cd1722f3947def4cf144679da39c4c32bdc35681".from_hex().unwrap());
}

#[test]
fn rpc_parity_set_author() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();
	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setAuthor", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.authoring_params().author, Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn rpc_parity_set_engine_signer() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();
	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setEngineSigner", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681", "password"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.authoring_params().author, Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
	assert_eq!(*miner.password.read(), "password".to_string());
}

#[test]
fn rpc_parity_set_transactions_limit() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();
	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setTransactionsLimit", "params":[10240240], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":false,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_set_hash_content() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();
	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_hashContent", "params":["https://parity.io/assets/images/ethcore-black-horizontal.png"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_remove_transaction() {
	use transaction::{Transaction, Action};

	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();
	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let tx = Transaction {
		nonce: 1.into(),
		gas_price: 0x9184e72a000u64.into(),
		gas: 0x76c0.into(),
		action: Action::Call(5.into()),
		value: 0x9184e72au64.into(),
		data: vec![]
	};
	let signed = tx.fake_sign(2.into());
	let hash = signed.hash();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_removeTransaction", "params":[""#.to_owned() + &format!("0x{:x}", hash) + r#""], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"blockHash":null,"blockNumber":null,"chainId":null,"condition":null,"creates":null,"from":"0x0000000000000000000000000000000000000002","gas":"0x76c0","gasPrice":"0x9184e72a000","hash":"0xa2e0da8a8064e0b9f93e95a53c2db6d01280efb8ac72a708d25487e67dd0f8fc","input":"0x","nonce":"0x1","publicKey":null,"r":"0x1","raw":"0xe9018609184e72a0008276c0940000000000000000000000000000000000000005849184e72a80800101","s":"0x1","standardV":"0x4","to":"0x0000000000000000000000000000000000000005","transactionIndex":null,"v":"0x0","value":"0x9184e72a"},"id":1}"#;

	miner.pending_transactions.lock().insert(hash, signed);
	assert_eq!(io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_set_dapps_list() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let updater = updater_service();
	let mut io = IoHandler::new();
	io.extend_with(parity_set_client(&client, &miner, &updater, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_dappsList", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[{"author":"Parity Technologies Ltd","description":"A skeleton dapp","iconUrl":"title.png","id":"skeleton","localUrl":null,"name":"Skeleton","version":"0.1"}],"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}
