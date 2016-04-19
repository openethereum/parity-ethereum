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
use jsonrpc_core::IoHandler;
use v1::{Ethcore, EthcoreClient};
use ethminer::MinerService;
use v1::tests::helpers::TestMinerService;
use util::numbers::*;
use rustc_serialize::hex::FromHex;


fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

#[test]
fn rpc_ethcore_extra_data() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_extraData", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x01020304","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}


#[test]
fn rpc_ethcore_gas_floor_target() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_gasFloorTarget", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x3039","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_min_gas_price() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_minGasPrice", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x01312d00","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_ethcore_set_min_gas_price() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_setMinGasPrice", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
	assert_eq!(miner.minimal_gas_price(), U256::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn rpc_ethcore_set_gas_floor_target() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_setGasFloorTarget", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
	assert_eq!(miner.gas_floor_target(), U256::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn rpc_ethcore_set_extra_data() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_setExtraData", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
	assert_eq!(miner.extra_data(), "cd1722f3947def4cf144679da39c4c32bdc35681".from_hex().unwrap());
}

#[test]
fn rpc_ethcore_set_author() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_setAuthor", "params":["0xcd1722f3947def4cf144679da39c4c32bdc35681"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
	assert_eq!(miner.author(), Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn rpc_ethcore_set_transactions_limit() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_setTransactionsLimit", "params":[10240240], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
	assert_eq!(miner.transactions_limit(), 10_240_240);
}


#[test]
fn rpc_ethcore_transactions_limit() {
	let miner = miner_service();
	let ethcore = EthcoreClient::new(&miner).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(ethcore);

	let request = r#"{"jsonrpc": "2.0", "method": "ethcore_transactionsLimit", "params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":1024,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}
