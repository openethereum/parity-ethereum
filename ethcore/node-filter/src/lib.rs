// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Smart contract based node filter.

extern crate common_types;
extern crate ethabi;
extern crate ethcore;
extern crate ethcore_network as network;
extern crate ethcore_network_devp2p as devp2p;
extern crate ethereum_types;
extern crate lru_cache;
extern crate parking_lot;

#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;
#[cfg(test)]
extern crate ethcore_io as io;
#[cfg(test)]
extern crate kvdb_memorydb;
#[cfg(test)]
extern crate tempdir;
#[macro_use]
extern crate log;

use std::collections::{HashMap, VecDeque};
use std::sync::Weak;

use common_types::ids::BlockId;
use ethcore::client::{BlockChainClient, ChainNotify, NewBlocks};
use ethereum_types::{H256, Address};
use ethabi::FunctionOutputDecoder;
use network::{ConnectionFilter, ConnectionDirection};
use devp2p::NodeId;
use devp2p::MAX_NODES_IN_TABLE;
use parking_lot::RwLock;

use_contract!(peer_set, "res/peer_set.json");

/// Connection filter that uses a contract to manage permissions.
pub struct NodeFilter {
	client: Weak<BlockChainClient>,
	contract_address: Address,
	cache: RwLock<Cache>
}

struct Cache {
	cache: HashMap<NodeId, bool>,
	order: VecDeque<NodeId>
}

// Increase cache size due to possible reserved peers, which do not count in the node table size
pub const CACHE_SIZE: usize = MAX_NODES_IN_TABLE + 1024;

impl NodeFilter {
	/// Create a new instance. Accepts a contract address.
	pub fn new(client: Weak<dyn BlockChainClient>, contract_address: Address) -> NodeFilter {
		NodeFilter {
			client,
			contract_address,
			cache: RwLock::new(Cache{
				cache: HashMap::with_capacity(CACHE_SIZE),
				order: VecDeque::with_capacity(CACHE_SIZE)
			})
		}
	}
}

impl ConnectionFilter for NodeFilter {
	fn connection_allowed(&self, own_id: &NodeId, connecting_id: &NodeId, _direction: ConnectionDirection) -> bool {
		let client = match self.client.upgrade() {
			Some(client) => client,
			None => return false,
		};

		if let Some(allowed) = self.cache.read().cache.get(connecting_id) {
			return *allowed;
		}

		let address = self.contract_address;
		let own_low = H256::from_slice(&own_id[0..32]);
		let own_high = H256::from_slice(&own_id[32..64]);
		let id_low = H256::from_slice(&connecting_id[0..32]);
		let id_high = H256::from_slice(&connecting_id[32..64]);

		let (data, decoder) = peer_set::functions::connection_allowed::call(own_low, own_high, id_low, id_high);
		let allowed = client.call_contract(BlockId::Latest, address, data)
			.and_then(|value| decoder.decode(&value).map_err(|e| e.to_string()))
			.unwrap_or_else(|e| {
				debug!("Error callling peer set contract: {:?}", e);
				false
			});
		let mut cache = self.cache.write();
		if cache.cache.len() == CACHE_SIZE {
			let popped = cache.order.pop_front().expect("the cache is full so there's at least one item we can pop; qed");
			cache.cache.remove(&popped);
		};
		if cache.cache.insert(*connecting_id, allowed).is_none() {
			cache.order.push_back(*connecting_id);
		}
		allowed
	}
}

impl ChainNotify for NodeFilter {
	fn new_blocks(&self, _new_blocks: NewBlocks)	{
		let mut cache = self.cache.write();
		cache.cache.clear();
		cache.order.clear();
	}
}

#[cfg(test)]
mod test {
	use std::sync::{Arc, Weak};
	use ethcore::spec::Spec;
	use ethcore::client::{BlockChainClient, Client, ClientConfig};
	use ethcore::miner::Miner;
	use ethcore::test_helpers;
	use network::{ConnectionDirection, ConnectionFilter, NodeId};
	use io::IoChannel;
	use super::NodeFilter;
	use tempdir::TempDir;
	use ethereum_types::Address;
	use std::str::FromStr;

	/// Contract code: https://gist.github.com/arkpar/467dbcc73cbb85b0997a7a10ffa0695f
	#[test]
	fn node_filter() {
		let contract_addr = Address::from_str("0000000000000000000000000000000000000005").unwrap();
		let data = include_bytes!("../res/node_filter.json");
		let tempdir = TempDir::new("").unwrap();
		let spec = Spec::load(&tempdir.path(), &data[..]).unwrap();
		let client_db = test_helpers::new_db();

		let client = Client::new(
			ClientConfig::default(),
			&spec,
			client_db,
			Arc::new(Miner::new_for_tests(&spec, None)),
			IoChannel::disconnected(),
		).unwrap();
		let filter = NodeFilter::new(Arc::downgrade(&client) as Weak<BlockChainClient>, contract_addr);
		let self1 = NodeId::from_str("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002").unwrap();
		let self2 = NodeId::from_str("00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003").unwrap();
		let node1 = NodeId::from_str("00000000000000000000000000000000000000000000000000000000000000110000000000000000000000000000000000000000000000000000000000000012").unwrap();
		let node2 = NodeId::from_str("00000000000000000000000000000000000000000000000000000000000000210000000000000000000000000000000000000000000000000000000000000022").unwrap();
		let nodex = NodeId::from_str("77000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

		assert!(filter.connection_allowed(&self1, &node1, ConnectionDirection::Inbound));
		assert!(filter.connection_allowed(&self1, &nodex, ConnectionDirection::Inbound));
		assert!(filter.connection_allowed(&self2, &node1, ConnectionDirection::Inbound));
		assert!(filter.connection_allowed(&self2, &node2, ConnectionDirection::Inbound));
	}
}
