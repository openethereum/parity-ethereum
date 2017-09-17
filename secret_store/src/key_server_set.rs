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
use hash::keccak;
use bigint::hash::H256;
use util::Address;
use bytes::Bytes;
use types::all::{Error, Public, NodeAddress};

const KEY_SERVER_SET_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_server_set";

/// Key server has been added to the set.
const ADDED_EVENT_NAME: &'static [u8] = &*b"KeyServerAdded(address)";
/// Key server has been removed from the set.
const REMOVED_EVENT_NAME: &'static [u8] = &*b"KeyServerRemoved(address)";

lazy_static! {
	static ref ADDED_EVENT_NAME_HASH: H256 = keccak(ADDED_EVENT_NAME);
	static ref REMOVED_EVENT_NAME_HASH: H256 = keccak(REMOVED_EVENT_NAME);
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
	pub fn new(client: &Arc<Client>, key_servers: BTreeMap<Public, NodeAddress>) -> Result<Arc<Self>, Error> {
		let mut cached_contract = CachedContract::new(client, key_servers)?;
		let key_server_contract_address = client.registry_address(KEY_SERVER_SET_CONTRACT_REGISTRY_NAME.to_owned());
		// only initialize from contract if it is installed. otherwise - use default nodes
		// once the contract is installed, all default nodes are lost (if not in the contract' set)
		if key_server_contract_address.is_some() {
			cached_contract.read_from_registry(&*client, key_server_contract_address);
		}

		let key_server_set = Arc::new(OnChainKeyServerSet {
			contract: Mutex::new(cached_contract),
		});
		client.add_notify(key_server_set.clone());
		Ok(key_server_set)
	}
}

impl KeyServerSet for OnChainKeyServerSet {
	fn get(&self) -> BTreeMap<Public, SocketAddr> {
		self.contract.lock().get()
	}
}

impl ChainNotify for OnChainKeyServerSet {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, enacted: Vec<H256>, retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: u64) {
		if !enacted.is_empty() || !retracted.is_empty() {
			self.contract.lock().update(enacted, retracted)
		}
	}
}

impl CachedContract {
	pub fn new(client: &Arc<Client>, key_servers: BTreeMap<Public, NodeAddress>) -> Result<Self, Error> {
		Ok(CachedContract {
			client: Arc::downgrade(client),
			contract_addr: None,
			key_servers: key_servers.into_iter()
				.map(|(p, addr)| {
					let addr = format!("{}:{}", addr.address, addr.port).parse()
						.map_err(|err| Error::Internal(format!("error parsing node address: {}", err)))?;
					Ok((p, addr))
				})
				.collect::<Result<BTreeMap<_, _>, Error>>()?,
		})
	}

	pub fn update(&mut self, enacted: Vec<H256>, retracted: Vec<H256>) {
		if let Some(client) = self.client.upgrade() {
			let new_contract_addr = client.registry_address(KEY_SERVER_SET_CONTRACT_REGISTRY_NAME.to_owned());

			// new contract installed => read nodes set from the contract
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
			// to simplify processing - just re-read the whole nodes set from the contract
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

				// only add successfully parsed nodes
				match (key_server_public, key_server_ip) {
					(Ok(key_server_public), Ok(key_server_ip)) => { key_servers.insert(key_server_public, key_server_ip); },
					(Err(public_err), _) => warn!(target: "secretstore_net", "received invalid public from key server set contract: {}", public_err),
					(_, Err(ip_err)) => warn!(target: "secretstore_net", "received invalid IP from key server set contract: {}", ip_err),
				}
			}
			key_servers
		})
		.unwrap_or_default();
		self.contract_addr = new_contract_address;
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::BTreeMap;
	use std::net::SocketAddr;
	use ethkey::Public;
	use super::KeyServerSet;

	#[derive(Default)]
	pub struct MapKeyServerSet {
		nodes: BTreeMap<Public, SocketAddr>,
	}

	impl MapKeyServerSet {
		pub fn new(nodes: BTreeMap<Public, SocketAddr>) -> Self {
			MapKeyServerSet {
				nodes: nodes,
			}
		}
	}

	impl KeyServerSet for MapKeyServerSet {
		fn get(&self) -> BTreeMap<Public, SocketAddr> {
			self.nodes.clone()
		}
	}
}
