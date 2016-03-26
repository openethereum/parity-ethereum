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
use v1::{Net, NetClient};
use v1::tests::helpers::{Config, TestSyncProvider};
use util::numbers::*;

fn sync_provider() -> Arc<TestSyncProvider> {
	Arc::new(TestSyncProvider::new(Config {
		network_id: U256::from(3),
		num_peers: 120,
	}))
}

#[test]
fn rpc_net_version() {
	let sync = sync_provider();
	let net = NetClient::new(&sync).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(net);

	let request = r#"{"jsonrpc": "2.0", "method": "net_version", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"3","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_net_peer_count() {
	let sync = sync_provider();
	let net = NetClient::new(&sync).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(net);

	let request = r#"{"jsonrpc": "2.0", "method": "net_peerCount", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x78","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_net_listening() {
	let sync = sync_provider();
	let net = NetClient::new(&sync).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(net);

	let request = r#"{"jsonrpc": "2.0", "method": "net_listening", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}
