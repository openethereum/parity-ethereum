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
use std::str::FromStr;
use rustc_serialize::hex::FromHex;
use util::{U256, Address};

use ethcore::miner::MinerService;
use ethcore::client::TestBlockChainClient;
use ethsync::ManageNetwork;

use jsonrpc_core::IoHandler;
use v1::{ParitySet, ParitySetClient};
use v1::tests::helpers::{TestMinerService, TestFetch};
use super::manage_network::TestManageNetwork;

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

fn client_service() -> Arc<TestBlockChainClient> {
	Arc::new(TestBlockChainClient::default())
}

fn network_service() -> Arc<TestManageNetwork> {
	Arc::new(TestManageNetwork)
}

pub type TestParitySetClient = ParitySetClient<TestBlockChainClient, TestMinerService, TestFetch>;

fn parity_set_client(client: &Arc<TestBlockChainClient>, miner: &Arc<TestMinerService>, net: &Arc<TestManageNetwork>) -> TestParitySetClient {
	ParitySetClient::with_fetch(client, miner, &(net.clone() as Arc<ManageNetwork>))
}

#[test]
fn rpc_parity_set_min_gas_price() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let io = IoHandler::new();
	io.add_delegate(parity_set_client(&client, &miner, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setMinGasPrice", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.minimal_gas_price(), U256::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn rpc_parity_set_gas_floor_target() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let io = IoHandler::new();
	io.add_delegate(parity_set_client(&client, &miner, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setGasFloorTarget", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.gas_floor_target(), U256::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn rpc_parity_set_extra_data() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let io = IoHandler::new();
	io.add_delegate(parity_set_client(&client, &miner, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setExtraData", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.extra_data(), "cd1722f3947def4cf144679da39c4c32bdc35681".from_hex().unwrap());
}

#[test]
fn rpc_parity_set_author() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let io = IoHandler::new();
	io.add_delegate(parity_set_client(&client, &miner, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setAuthor", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.author(), Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn rpc_parity_set_transactions_limit() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let io = IoHandler::new();
	io.add_delegate(parity_set_client(&client, &miner, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_setTransactionsLimit", "params":[10240240], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
	assert_eq!(miner.transactions_limit(), 10_240_240);
}

#[test]
fn rpc_parity_set_hash_content() {
	let miner = miner_service();
	let client = client_service();
	let network = network_service();
	let io = IoHandler::new();
	io.add_delegate(parity_set_client(&client, &miner, &network).to_delegate());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_hashContent", "params":["https://ethcore.io/assets/images/ethcore-black-horizontal.png"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e","id":1}"#;

	assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
}

