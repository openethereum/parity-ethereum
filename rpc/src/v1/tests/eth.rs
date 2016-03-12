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

use std::collections::HashMap;
use std::sync::Arc;
use jsonrpc_core::IoHandler;
use util::hash::{Address};
use ethcore::client::{TestBlockChainClient, EachBlockWith};
use v1::{Eth, EthClient};
use v1::tests::helpers::{TestAccount, TestAccountProvider, TestSyncProvider, Config};

fn blockchain_client() -> Arc<TestBlockChainClient> {
	let mut client = TestBlockChainClient::new();
	client.add_blocks(10, EachBlockWith::Nothing);
	Arc::new(client)
}

fn accounts_provider() -> Arc<TestAccountProvider> {
	let mut accounts = HashMap::new();
	accounts.insert(Address::from(1), TestAccount::new("test"));
	let ap = TestAccountProvider::new(accounts);
	Arc::new(ap)
}

fn sync_provider() -> Arc<TestSyncProvider> {
	Arc::new(TestSyncProvider::new(Config {
		protocol_version: 65,
		num_peers: 120,
	}))
}

#[test]
fn rpc_eth_accounts() {
	let client = blockchain_client();
	let sync = sync_provider();
	let ap = accounts_provider();

	let eth = EthClient::new(&client, &sync, &ap).to_delegate();
	let io = IoHandler::new();
	io.add_delegate(eth);

	let request = r#"{"jsonrpc": "2.0", "method": "eth_accounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x0000000000000000000000000000000000000001"],"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}
