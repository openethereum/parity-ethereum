// Copyright 2016 Ethcore (UK) Ltd.
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

//! rpc integration tests.
use std::collections::HashMap;
use std::sync::Arc;

use ethjson::blockchain::test::Test;
use ethcore::client::{BlockChainClient, Client, ClientConfig};
use ethcore::spec::Genesis;
use ethcore::block::Block;
use ethcore::ethereum;
use ethminer::ExternalMiner;
use devtools::RandomTempPath;
use util::io::IoChannel;
use util::hash::{Address, FixedHash};
use util::numbers::U256;
use util::keys::{TestAccount, TestAccountProvider};
use jsonrpc_core::IoHandler;

use v1::traits::eth::Eth;
use v1::impls::EthClient;
use v1::tests::helpers::{TestSyncProvider, Config, TestMinerService};

use super::RPC_CHAIN;

#[test]
fn harness_works() {
	eth_harness(|_| {});
}

fn account_provider() -> Arc<TestAccountProvider> {
	let mut accounts = HashMap::new();
	accounts.insert(Address::from(1), TestAccount::new("test"));
	let ap = TestAccountProvider::new(accounts);
	Arc::new(ap)
}

fn sync_provider() -> Arc<TestSyncProvider> {
	Arc::new(TestSyncProvider::new(Config {
		network_id: U256::from(3),
		num_peers: 120,
	}))
}

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

// this harness will create a handler which tests can send specially-crafted
// JSONRPC requests to.
fn eth_harness<F, U>(mut cb: F) -> U
	where F: FnMut(&IoHandler) -> U {
	let chains = Test::load(RPC_CHAIN).unwrap();
	let chain = chains.into_iter().next().unwrap().1;
	let genesis = Genesis::from(chain.genesis());
	let mut spec = ethereum::new_frontier_test();
	let state = chain.pre_state.clone().into();
	spec.set_genesis_state(state);
	spec.overwrite_genesis_params(genesis);
	assert!(spec.is_state_root_valid());

	let dir = RandomTempPath::new();
	let client = Client::new(ClientConfig::default(), spec, dir.as_path(), IoChannel::disconnected()).unwrap();
	let sync_provider = sync_provider();
	let miner_service = miner_service();
	let account_provider = account_provider();
	let external_miner = Arc::new(ExternalMiner::default());

	for b in &chain.blocks_rlp() {
		if Block::is_good(&b) {
			let _ = client.import_block(b.clone());
			client.flush_queue();
			client.import_verified_blocks(&IoChannel::disconnected());
		}
	}

	assert!(client.chain_info().best_block_hash == chain.best_block.into());

	let eth_client = EthClient::new(&client, &sync_provider, &account_provider,
		&miner_service, &external_miner);

	let handler = IoHandler::new();
	let delegate = eth_client.to_delegate();
	handler.add_delegate(delegate);
	cb(&handler)
}
