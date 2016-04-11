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
use v1::tests::helpers::{TestMinerService};
use util::numbers::*;


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

