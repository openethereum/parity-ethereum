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

use std::sync::{Arc, Weak};
use std::net::SocketAddr;
use std::collections::BTreeMap;
use futures::{future, Future};
use parking_lot::Mutex;
use ethcore::filter::Filter;
use ethcore::client::{Client, BlockChainClient, BlockId, ChainNotify};
use native_contracts::KeyServerSet as KeyServerSetContract;
use util::{H256, Address, Bytes, Hashable};
use types::all::Public;

const KEY_SERVER_SET_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_server_set";

// TODO: ethabi should be able to generate this.
const ADDED_EVENT_NAME: &'static [u8] = &*b"KeyServerAdded(address)";
const REMOVED_EVENT_NAME: &'static [u8] = &*b"KeyServerRemoved(address)";

lazy_static! {
	static ref ADDED_EVENT_NAME_HASH: H256 = ADDED_EVENT_NAME.sha3();
	static ref REMOVED_EVENT_NAME_HASH: H256 = REMOVED_EVENT_NAME.sha3();
}

/// Key Server set
pub trait KeyServerSet: Send + Sync {
	/// Get set of configured key servers
	fn get(&self) -> BTreeMap<Public, SocketAddr>;
}

/// On-chain Key Server set implementation.
pub struct OnChainKeyServerSet {
	/// Cached on-chain contract.
	contract: Mutex<CachedContract>,
}

/// Cached on-chain Key Server set contract.
struct CachedContract {
	/// Blockchain client.
	client: Weak<Client>,
	/// Contract address.
	contract_addr: Option<Address>,
	/// Active set of key servers.
	key_servers: BTreeMap<Public, SocketAddr>,
}

impl OnChainKeyServerSet {
	pub fn new(client: &Arc<Client>, key_servers: BTreeMap<Public, SocketAddr>) -> Arc<Self> {
		let mut cached_contract = CachedContract::new(client, key_servers);
		let key_server_contract_address = client.registry_address(KEY_SERVER_SET_CONTRACT_REGISTRY_NAME.to_owned());
		// only initialize from contract if it is installed. otherwise - use default nodes
		if key_server_contract_address.is_some() {
			cached_contract.read_from_registry(&*client, key_server_contract_address);
		}

		let key_server_set = Arc::new(OnChainKeyServerSet {
			contract: Mutex::new(cached_contract),
		});
		client.add_notify(key_server_set.clone());
		key_server_set
	}
}

impl KeyServerSet for OnChainKeyServerSet {
	fn get(&self) -> BTreeMap<Public, SocketAddr> {
		self.contract.lock().get()
	}
}

impl ChainNotify for OnChainKeyServerSet {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, enacted: Vec<H256>, retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: u64) {
		self.contract.lock().update(enacted, retracted)
	}
}

impl CachedContract {
	pub fn new(client: &Arc<Client>, key_servers: BTreeMap<Public, SocketAddr>) -> Self {
		CachedContract {
			client: Arc::downgrade(client),
			contract_addr: None,
			key_servers: key_servers,
		}
	}

	pub fn update(&mut self, enacted: Vec<H256>, retracted: Vec<H256>) {
		if let Some(client) = self.client.upgrade() {
			let new_contract_addr = client.registry_address(KEY_SERVER_SET_CONTRACT_REGISTRY_NAME.to_owned());

			// new contract installed
			if self.contract_addr.as_ref() != new_contract_addr.as_ref() {
				self.read_from_registry(&*client, new_contract_addr);
				return;
			}

			// check for contract events
			let is_set_changed = self.contract_addr.is_some() && enacted.iter()
				.chain(retracted.iter())
				.any(|block_hash| !client.logs(Filter {
					from_block: BlockId::Hash(block_hash.clone()),
					to_block: BlockId::Hash(block_hash.clone()),
					address: self.contract_addr.clone().map(|a| vec![a]),
					topics: vec![
						Some(vec![*ADDED_EVENT_NAME_HASH, *REMOVED_EVENT_NAME_HASH]),
						None,
						None,
						None,
					],
					limit: Some(1),
				}).is_empty());
			if is_set_changed {
				self.read_from_registry(&*client, new_contract_addr);
			}
		}
	}

	pub fn get(&self) -> BTreeMap<Public, SocketAddr> {
		self.key_servers.clone()
	}

	fn read_from_registry(&mut self, client: &Client, new_contract_address: Option<Address>) {
		self.key_servers = new_contract_address.map(|contract_addr| {
			trace!(target: "secretstore", "Configuring for key server set contract from {}", contract_addr);

			KeyServerSetContract::new(contract_addr)
		})
		.map(|contract| {
			let mut key_servers = BTreeMap::new();
			let do_call = |a, d| future::done(client.call_contract(BlockId::Latest, a, d));
			let key_servers_list = contract.get_key_servers(do_call).wait()
				.map_err(|err| { trace!(target: "secretstore", "Error {} reading list of key servers from contract", err); err })
				.unwrap_or_default();
			for key_server in key_servers_list {
				let key_server_public = contract.get_key_server_public(
					|a, d| future::done(client.call_contract(BlockId::Latest, a, d)), key_server).wait()
					.and_then(|p| if p.len() == 64 { Ok(Public::from_slice(&p)) } else { Err(format!("Invalid public length {}", p.len())) });
				let key_server_ip = contract.get_key_server_address(
					|a, d| future::done(client.call_contract(BlockId::Latest, a, d)), key_server).wait()
					.and_then(|a| a.parse().map_err(|e| format!("Invalid ip address: {}", e)));
				if let (Ok(key_server_public), Ok(key_server_ip)) = (key_server_public, key_server_ip) {
					key_servers.insert(key_server_public, key_server_ip);
				}
			}
			key_servers
		})
		.unwrap_or_default();
		self.contract_addr = new_contract_address;
	}
}
