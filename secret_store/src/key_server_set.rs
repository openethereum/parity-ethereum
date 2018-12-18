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
use std::net::SocketAddr;
use std::collections::{BTreeMap, HashSet};
use parking_lot::Mutex;
use ethabi::FunctionOutputDecoder;
use ethcore::client::{Client, BlockChainClient, BlockId, ChainNotify, NewBlocks, CallContract};
use ethereum_types::{H256, Address};
use ethkey::public_to_address;
use bytes::Bytes;
use types::{Error, Public, NodeAddress, NodeId};
use trusted_client::TrustedClient;
use {NodeKeyPair, ContractAddress};

use_contract!(key_server, "res/key_server_set.json");

/// Name of KeyServerSet contract in registry.
const KEY_SERVER_SET_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_server_set";
/// Number of blocks (since latest new_set change) required before actually starting migration.
const MIGRATION_CONFIRMATIONS_REQUIRED: u64 = 5;
/// Number of blocks before the same-migration transaction (be it start or confirmation) will be retried.
const TRANSACTION_RETRY_INTERVAL_BLOCKS: u64 = 30;

#[derive(Default, Debug, Clone, PartialEq)]
/// Key Server Set state.
pub struct KeyServerSetSnapshot {
	/// Current set of key servers.
	pub current_set: BTreeMap<NodeId, SocketAddr>,
	/// New set of key servers.
	pub new_set: BTreeMap<NodeId, SocketAddr>,
	/// Current migration data.
	pub migration: Option<KeyServerSetMigration>,
}

#[derive(Default, Debug, Clone, PartialEq)]
/// Key server set migration.
pub struct KeyServerSetMigration {
	/// Migration id.
	pub id: H256,
	/// Migration set of key servers. It is the new_set at the moment of migration start.
	pub set: BTreeMap<NodeId, SocketAddr>,
	/// Master node of the migration process.
	pub master: NodeId,
	/// Is migration confirmed by this node?
	pub is_confirmed: bool,
}

/// Key Server Set
pub trait KeyServerSet: Send + Sync {
	/// Is this node currently isolated from the set?
	fn is_isolated(&self) -> bool;
	/// Get server set state.
	fn snapshot(&self) -> KeyServerSetSnapshot;
	/// Start migration.
	fn start_migration(&self, migration_id: H256);
	/// Confirm migration.
	fn confirm_migration(&self, migration_id: H256);
}

/// On-chain Key Server set implementation.
pub struct OnChainKeyServerSet {
	/// Cached on-chain contract.
	contract: Mutex<CachedContract>,
}

#[derive(Default, Debug, Clone, PartialEq)]
/// Non-finalized new_set.
struct FutureNewSet {
	/// New servers set.
	pub new_set: BTreeMap<NodeId, SocketAddr>,
	/// Hash of block, when this set has appeared for first time.
	pub block: H256,
}

#[derive(Default, Debug, Clone, PartialEq)]
/// Migration-related transaction information.
struct PreviousMigrationTransaction {
	/// Migration id.
	pub migration_id: H256,
	/// Latest actual block number at the time this transaction has been sent.
	pub block: u64,
}

/// Cached on-chain Key Server set contract.
struct CachedContract {
	/// Blockchain client.
	client: TrustedClient,
	/// Contract address source.
	contract_address_source: Option<ContractAddress>,
	/// Current contract address.
	contract_address: Option<Address>,
	/// Is auto-migrate enabled?
	auto_migrate_enabled: bool,
	/// Current contract state.
	snapshot: KeyServerSetSnapshot,
	/// Scheduled contract state (if any).
	future_new_set: Option<FutureNewSet>,
	/// Previous start migration transaction.
	start_migration_tx: Option<PreviousMigrationTransaction>,
	/// Previous confirm migration transaction.
	confirm_migration_tx: Option<PreviousMigrationTransaction>,
	/// This node key pair.
	self_key_pair: Arc<NodeKeyPair>,
}

impl OnChainKeyServerSet {
	pub fn new(trusted_client: TrustedClient, contract_address_source: Option<ContractAddress>, self_key_pair: Arc<NodeKeyPair>, auto_migrate_enabled: bool, key_servers: BTreeMap<Public, NodeAddress>) -> Result<Arc<Self>, Error> {
		let client = trusted_client.get_untrusted();
		let key_server_set = Arc::new(OnChainKeyServerSet {
			contract: Mutex::new(CachedContract::new(trusted_client, contract_address_source, self_key_pair, auto_migrate_enabled, key_servers)?),
		});
		client
			.ok_or_else(|| Error::Internal("Constructing OnChainKeyServerSet without active Client".into()))?
			.add_notify(key_server_set.clone());
		Ok(key_server_set)
	}
}

impl KeyServerSet for OnChainKeyServerSet {
	fn is_isolated(&self) -> bool {
		self.contract.lock().is_isolated()
	}

	fn snapshot(&self) -> KeyServerSetSnapshot {
		self.contract.lock().snapshot()
	}

	fn start_migration(&self, migration_id: H256) {
		self.contract.lock().start_migration(migration_id)
	}

	fn confirm_migration(&self, migration_id: H256) {
		self.contract.lock().confirm_migration(migration_id);
	}
}

impl ChainNotify for OnChainKeyServerSet {
	fn new_blocks(&self, new_blocks: NewBlocks) {
		if new_blocks.has_more_blocks_to_import { return }
		let (enacted, retracted) = new_blocks.route.into_enacted_retracted();

		if !enacted.is_empty() || !retracted.is_empty() {
			self.contract.lock().update(enacted, retracted)
		}
	}
}

trait KeyServerSubset<F: Fn(Vec<u8>) -> Result<Vec<u8>, String>> {
	fn read_list(&self, f: &F) -> Result<Vec<Address>, String>;

	fn read_public(&self, address: Address, f: &F) -> Result<Bytes, String>;

	fn read_address(&self, address: Address, f: &F) -> Result<String, String>;
}

struct CurrentKeyServerSubset;

impl <F: Fn(Vec<u8>) -> Result<Vec<u8>, String>> KeyServerSubset<F> for CurrentKeyServerSubset {
	fn read_list(&self, f: &F) -> Result<Vec<Address>, String> {
		let (encoded, decoder) = key_server::functions::get_current_key_servers::call();
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}

	fn read_public(&self, address: Address, f: &F) -> Result<Bytes, String> {
		let (encoded, decoder) = key_server::functions::get_current_key_server_public::call(address);
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}

	fn read_address(&self, address: Address, f: &F) -> Result<String, String> {
		let (encoded, decoder) = key_server::functions::get_current_key_server_address::call(address);
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}
}

struct MigrationKeyServerSubset;

impl <F: Fn(Vec<u8>) -> Result<Vec<u8>, String>> KeyServerSubset<F> for MigrationKeyServerSubset {
	fn read_list(&self, f: &F) -> Result<Vec<Address>, String> {
		let (encoded, decoder) = key_server::functions::get_migration_key_servers::call();
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}

	fn read_public(&self, address: Address, f: &F) -> Result<Bytes, String> {
		let (encoded, decoder) = key_server::functions::get_migration_key_server_public::call(address);
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}

	fn read_address(&self, address: Address, f: &F) -> Result<String, String> {
		let (encoded, decoder) = key_server::functions::get_migration_key_server_address::call(address);
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}
}

struct NewKeyServerSubset;

impl <F: Fn(Vec<u8>) -> Result<Vec<u8>, String>> KeyServerSubset<F> for NewKeyServerSubset {
	fn read_list(&self, f: &F) -> Result<Vec<Address>, String> {
		let (encoded, decoder) = key_server::functions::get_new_key_servers::call();
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}

	fn read_public(&self, address: Address, f: &F) -> Result<Bytes, String> {
		let (encoded, decoder) = key_server::functions::get_new_key_server_public::call(address);
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}

	fn read_address(&self, address: Address, f: &F) -> Result<String, String> {
		let (encoded, decoder) = key_server::functions::get_new_key_server_address::call(address);
		decoder.decode(&f(encoded)?).map_err(|e| e.to_string())
	}
}

impl CachedContract {
	pub fn new(client: TrustedClient, contract_address_source: Option<ContractAddress>, self_key_pair: Arc<NodeKeyPair>, auto_migrate_enabled: bool, key_servers: BTreeMap<Public, NodeAddress>) -> Result<Self, Error> {
		let server_set = match contract_address_source.is_none() {
			true => key_servers.into_iter()
				.map(|(p, addr)| {
					let addr = format!("{}:{}", addr.address, addr.port).parse()
						.map_err(|err| Error::Internal(format!("error parsing node address: {}", err)))?;
					Ok((p, addr))
				})
				.collect::<Result<BTreeMap<_, _>, Error>>()?,
			false => Default::default(),
		};

		let mut contract =  CachedContract {
			client: client,
			contract_address_source: contract_address_source,
			contract_address: None,
			auto_migrate_enabled: auto_migrate_enabled,
			future_new_set: None,
			confirm_migration_tx: None,
			start_migration_tx: None,
			snapshot: KeyServerSetSnapshot {
				current_set: server_set.clone(),
				new_set: server_set,
				..Default::default()
			},
			self_key_pair: self_key_pair,
		};
		contract.update_contract_address();

		Ok(contract)
	}

	pub fn update_contract_address(&mut self) {
		if let Some(ref contract_address_source) = self.contract_address_source {
			let contract_address = self.client.read_contract_address(KEY_SERVER_SET_CONTRACT_REGISTRY_NAME.into(), contract_address_source);
			if contract_address != self.contract_address {
				trace!(target: "secretstore", "{}: Configuring for key server set contract from address {:?}",
					self.self_key_pair.public(), contract_address);

				self.contract_address = contract_address;
			}
		}
	}

	pub fn update(&mut self, enacted: Vec<H256>, retracted: Vec<H256>) {
		// no need to update when servers set is hardcoded
		if self.contract_address_source.is_none() {
			return;
		}

		if let Some(client) = self.client.get() {
			// read new snapshot from reqistry (if something has chnaged)
			if !enacted.is_empty() || !retracted.is_empty() {
				self.update_contract_address();
				self.read_from_registry(&*client);
			}

			// update number of confirmations (if there's future new set)
			self.update_number_of_confirmations_if_required(&*client);
		}
	}

	fn is_isolated(&self) -> bool {
		!self.snapshot.current_set.contains_key(self.self_key_pair.public())
	}

	fn snapshot(&self) -> KeyServerSetSnapshot {
		self.snapshot.clone()
	}

	fn start_migration(&mut self, migration_id: H256) {
		// trust is not needed here, because it is the reaction to the read of the trusted client
		if let (Some(client), Some(contract_address)) = (self.client.get_untrusted(), self.contract_address.as_ref()) {
			// check if we need to send start migration transaction
			if !update_last_transaction_block(&*client, &migration_id, &mut self.start_migration_tx) {
				return;
			}

			// prepare transaction data
			let transaction_data = key_server::functions::start_migration::encode_input(migration_id);

			// send transaction
			match self.client.transact_contract(*contract_address, transaction_data) {
				Ok(_) => trace!(target: "secretstore_net", "{}: sent auto-migration start transaction",
					self.self_key_pair.public()),
				Err(error) => warn!(target: "secretstore_net", "{}: failed to submit auto-migration start transaction: {}",
					self.self_key_pair.public(), error),
			}
		}
	}

	fn confirm_migration(&mut self, migration_id: H256) {
		// trust is not needed here, because we have already completed the action
		if let (Some(client), Some(contract_address)) = (self.client.get(), self.contract_address) {
			// check if we need to send start migration transaction
			if !update_last_transaction_block(&*client, &migration_id, &mut self.confirm_migration_tx) {
				return;
			}

			// prepare transaction data
			let transaction_data = key_server::functions::confirm_migration::encode_input(migration_id);

			// send transaction
			match self.client.transact_contract(contract_address, transaction_data) {
				Ok(_) => trace!(target: "secretstore_net", "{}: sent auto-migration confirm transaction",
					self.self_key_pair.public()),
				Err(error) => warn!(target: "secretstore_net", "{}: failed to submit auto-migration confirmation transaction: {}",
					self.self_key_pair.public(), error),
			}
		}
	}

	fn read_from_registry(&mut self, client: &Client) {
		let contract_address = match self.contract_address {
			Some(contract_address) => contract_address,
			None => {
				// no contract installed => empty snapshot
				// WARNING: after restart current_set will be reset to the set from configuration file
				// even though we have reset to empty set here. We are not considering this as an issue
				// because it is actually the issue of administrator.
				self.snapshot = Default::default();
				self.future_new_set = None;
				return;
			},
		};

		let do_call = |data| client.call_contract(BlockId::Latest, contract_address, data);

		let current_set = Self::read_key_server_set(CurrentKeyServerSubset, &do_call);

		// read migration-related data if auto migration is enabled
		let (new_set, migration) = match self.auto_migrate_enabled {
			true => {
				let new_set = Self::read_key_server_set(NewKeyServerSubset, &do_call);
				let migration_set = Self::read_key_server_set(MigrationKeyServerSubset, &do_call);

				let migration_id = match migration_set.is_empty() {
					false => {
						let (encoded, decoder) = key_server::functions::get_migration_id::call();
						do_call(encoded)
							.map_err(|e| e.to_string())
							.and_then(|data| decoder.decode(&data).map_err(|e| e.to_string()))
							.map_err(|err| { trace!(target: "secretstore", "Error {} reading migration id from contract", err); err })
							.ok()
					},
					true => None,
				};

				let migration_master = match migration_set.is_empty() {
					false => {
						let (encoded, decoder) = key_server::functions::get_migration_master::call();
						do_call(encoded)
							.map_err(|e| e.to_string())
							.and_then(|data| decoder.decode(&data).map_err(|e| e.to_string()))
							.map_err(|err| { trace!(target: "secretstore", "Error {} reading migration master from contract", err); err })
							.ok()
							.and_then(|address| current_set.keys().chain(migration_set.keys())
								.find(|public| public_to_address(public) == address)
								.cloned())
					},
					true => None,
				};

				let is_migration_confirmed = match migration_set.is_empty() {
					false if current_set.contains_key(self.self_key_pair.public()) || migration_set.contains_key(self.self_key_pair.public()) => {
						let (encoded, decoder) = key_server::functions::is_migration_confirmed::call(self.self_key_pair.address());
						do_call(encoded)
							.map_err(|e| e.to_string())
							.and_then(|data| decoder.decode(&data).map_err(|e| e.to_string()))
							.map_err(|err| { trace!(target: "secretstore", "Error {} reading migration confirmation from contract", err); err })
							.ok()
					},
					_ => None,
				};

				let migration = match (migration_set.is_empty(), migration_id, migration_master, is_migration_confirmed) {
					(false, Some(migration_id), Some(migration_master), Some(is_migration_confirmed)) =>
						Some(KeyServerSetMigration {
							id: migration_id,
							master: migration_master,
							set: migration_set,
							is_confirmed: is_migration_confirmed,
						}),
					_ => None,
				};

				(new_set, migration)
			}
			false => (current_set.clone(), None),
		};

		let mut new_snapshot = KeyServerSetSnapshot {
			current_set: current_set,
			new_set: new_set,
			migration: migration,
		};

		// we might want to adjust new_set if auto migration is enabled
		if self.auto_migrate_enabled {
			let block = client.block_hash(BlockId::Latest).unwrap_or_default();
			update_future_set(&mut self.future_new_set, &mut new_snapshot, block);
		}

		self.snapshot = new_snapshot;
	}

	fn read_key_server_set<T, F>(subset: T, do_call: F) -> BTreeMap<Public, SocketAddr>
		where
			T: KeyServerSubset<F>,
			F: Fn(Vec<u8>) -> Result<Vec<u8>, String> {
		let mut key_servers = BTreeMap::new();
		let mut key_servers_addresses = HashSet::new();
		let key_servers_list = subset.read_list(&do_call)
			.map_err(|err| { warn!(target: "secretstore_net", "error {} reading list of key servers from contract", err); err })
			.unwrap_or_default();
		for key_server in key_servers_list {
			let key_server_public = subset.read_public(key_server, &do_call)
				.and_then(|p| if p.len() == 64 { Ok(Public::from_slice(&p)) } else { Err(format!("Invalid public length {}", p.len())) });
			let key_server_address: Result<SocketAddr, _> = subset.read_address(key_server, &do_call)
				.and_then(|a| a.parse().map_err(|e| format!("Invalid ip address: {}", e)));

			// only add successfully parsed nodes
			match (key_server_public, key_server_address) {
				(Ok(key_server_public), Ok(key_server_address)) => {
					if !key_servers_addresses.insert(key_server_address.clone()) {
						warn!(target: "secretstore_net", "the same address ({}) specified twice in list of contracts. Ignoring server {}",
							key_server_address, key_server_public);
						continue;
					}

					key_servers.insert(key_server_public, key_server_address);
				},
				(Err(public_err), _) => warn!(target: "secretstore_net", "received invalid public from key server set contract: {}", public_err),
				(_, Err(ip_err)) => warn!(target: "secretstore_net", "received invalid IP from key server set contract: {}", ip_err),
			}
		}
		key_servers
	}

	fn update_number_of_confirmations_if_required(&mut self, client: &BlockChainClient) {
		if !self.auto_migrate_enabled {
			return;
		}

		update_number_of_confirmations(
			&|| latest_block_hash(&*client),
			&|block| block_confirmations(&*client, block),
			&mut self.future_new_set, &mut self.snapshot);
	}
}

/// Check if two sets are equal (in terms of migration requirements). We do not need migration if only
/// addresses are changed - simply adjusting connections is enough in this case.
pub fn is_migration_required(current_set: &BTreeMap<NodeId, SocketAddr>, new_set: &BTreeMap<NodeId, SocketAddr>) -> bool {
	let no_nodes_removed = current_set.keys().all(|n| new_set.contains_key(n));
	let no_nodes_added = new_set.keys().all(|n| current_set.contains_key(n));
	!no_nodes_removed || !no_nodes_added
}

fn update_future_set(future_new_set: &mut Option<FutureNewSet>, new_snapshot: &mut KeyServerSetSnapshot, block: H256) {
	// migration has already started => no need to delay visibility
	if new_snapshot.migration.is_some() {
		*future_new_set = None;
		return;
	}

	// no migration is required => no need to delay visibility
	if !is_migration_required(&new_snapshot.current_set, &new_snapshot.new_set) {
		*future_new_set = None;
		return;
	}

	// when auto-migrate is enabled, we do not want to start migration right after new_set is changed, because of:
	// 1) there could be a fork && we could start migration to forked version (and potentially lose secrets)
	// 2) there must be some period for new_set changes finalization (i.e. adding/removing more servers)
	let mut new_set = new_snapshot.current_set.clone();
	::std::mem::swap(&mut new_set, &mut new_snapshot.new_set);

	// if nothing has changed in future_new_set, then we want to preserve previous block hash
	let block = match Some(&new_set) == future_new_set.as_ref().map(|f| &f.new_set) {
		true => future_new_set.as_ref().map(|f| &f.block).cloned().unwrap_or_else(|| block),
		false => block,
	};

	*future_new_set = Some(FutureNewSet {
		new_set: new_set,
		block: block,
	});
}

fn update_number_of_confirmations<F1: Fn() -> H256, F2: Fn(H256) -> Option<u64>>(latest_block: &F1, confirmations: &F2, future_new_set: &mut Option<FutureNewSet>, snapshot: &mut KeyServerSetSnapshot) {
	match future_new_set.as_mut() {
		// no future new set is scheduled => do nothing,
		None => return,
		// else we should calculate number of confirmations for future new set
		Some(future_new_set) => match confirmations(future_new_set.block.clone()) {
			// we have enough confirmations => should move new_set from future to snapshot
			Some(confirmations) if confirmations >= MIGRATION_CONFIRMATIONS_REQUIRED => (),
			// not enough confirmations => do nothing
			Some(_) => return,
			// if number of confirmations is None, then reorg has happened && we need to reset block
			// (some more intelligent strategy is possible, but let's stick to simplest one)
			None => {
				future_new_set.block = latest_block();
				return;
			}
		}
	}

	let future_new_set = future_new_set.take()
		.expect("we only pass through match above when future_new_set is some; qed");
	snapshot.new_set = future_new_set.new_set;
}

fn update_last_transaction_block(client: &Client, migration_id: &H256, previous_transaction: &mut Option<PreviousMigrationTransaction>) -> bool {
	let last_block = client.block_number(BlockId::Latest).unwrap_or_default();
	match previous_transaction.as_ref() {
		// no previous transaction => send immediately
		None => (),
		// previous transaction has been sent for other migration process => send immediately
		Some(tx) if tx.migration_id != *migration_id => (),
		// if we have sent the same type of transaction recently => do nothing (hope it will be mined eventually)
		// if we have sent the same transaction some time ago =>
		//   assume that our tx queue was full
		//   or we didn't have enough eth fot this tx
		//   or the transaction has been removed from the queue (and never reached any miner node)
		// if we have restarted after sending tx => assume we have never sent it
		Some(tx) => {
			if tx.block > last_block || last_block - tx.block < TRANSACTION_RETRY_INTERVAL_BLOCKS {
				return false;
			}
		},
	}

	*previous_transaction = Some(PreviousMigrationTransaction {
		migration_id: migration_id.clone(),
		block: last_block,
	});

	true
}

fn latest_block_hash(client: &BlockChainClient) -> H256 {
	client.block_hash(BlockId::Latest).unwrap_or_default()
}

fn block_confirmations(client: &BlockChainClient, block: H256) -> Option<u64> {
	client.block_number(BlockId::Hash(block))
		.and_then(|block| client.block_number(BlockId::Latest).map(|last_block| (block, last_block)))
		.map(|(block, last_block)| last_block - block)
}

#[cfg(test)]
pub mod tests {
	use std::collections::BTreeMap;
	use std::net::SocketAddr;
	use ethereum_types::H256;
	use ethkey::Public;
	use super::{update_future_set, update_number_of_confirmations, FutureNewSet,
		KeyServerSet, KeyServerSetSnapshot, MIGRATION_CONFIRMATIONS_REQUIRED};

	#[derive(Default)]
	pub struct MapKeyServerSet {
		is_isolated: bool,
		nodes: BTreeMap<Public, SocketAddr>,
	}

	impl MapKeyServerSet {
		pub fn new(is_isolated: bool, nodes: BTreeMap<Public, SocketAddr>) -> Self {
			MapKeyServerSet {
				is_isolated: is_isolated,
				nodes: nodes,
			}
		}
	}

	impl KeyServerSet for MapKeyServerSet {
		fn is_isolated(&self) -> bool {
			self.is_isolated
		}

		fn snapshot(&self) -> KeyServerSetSnapshot {
			KeyServerSetSnapshot {
				current_set: self.nodes.clone(),
				new_set: self.nodes.clone(),
				..Default::default()
			}
		}

		fn start_migration(&self, _migration_id: H256) {
			unimplemented!("test-only")
		}

		fn confirm_migration(&self, _migration_id: H256) {
			unimplemented!("test-only")
		}
	}

	#[test]
	fn future_set_is_updated_to_none_when_migration_has_already_started() {
		let mut future_new_set = Some(Default::default());
		let mut new_snapshot = KeyServerSetSnapshot {
			migration: Some(Default::default()),
			..Default::default()
		};
		let new_snapshot_copy = new_snapshot.clone();
		update_future_set(&mut future_new_set, &mut new_snapshot, Default::default());
		assert_eq!(future_new_set, None);
		assert_eq!(new_snapshot, new_snapshot_copy);
	}

	#[test]
	fn future_set_is_updated_to_none_when_no_migration_is_required() {
		let node_id = Default::default();
		let address1 = "127.0.0.1:12000".parse().unwrap();
		let address2 = "127.0.0.1:12001".parse().unwrap();

		// addresses are different, but node set is the same => no migration is required
		let mut future_new_set = Some(Default::default());
		let mut new_snapshot = KeyServerSetSnapshot {
			current_set: vec![(node_id, address1)].into_iter().collect(),
			new_set: vec![(node_id, address2)].into_iter().collect(),
			..Default::default()
		};
		let new_snapshot_copy = new_snapshot.clone();
		update_future_set(&mut future_new_set, &mut new_snapshot, Default::default());
		assert_eq!(future_new_set, None);
		assert_eq!(new_snapshot, new_snapshot_copy);

		// everything is the same => no migration is required
		let mut future_new_set = Some(Default::default());
		let mut new_snapshot = KeyServerSetSnapshot {
			current_set: vec![(node_id, address1)].into_iter().collect(),
			new_set: vec![(node_id, address1)].into_iter().collect(),
			..Default::default()
		};
		let new_snapshot_copy = new_snapshot.clone();
		update_future_set(&mut future_new_set, &mut new_snapshot, Default::default());
		assert_eq!(future_new_set, None);
		assert_eq!(new_snapshot, new_snapshot_copy);
	}

	#[test]
	fn future_set_is_initialized() {
		let address = "127.0.0.1:12000".parse().unwrap();

		let mut future_new_set = None;
		let mut new_snapshot = KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(2.into(), address)].into_iter().collect(),
			..Default::default()
		};
		update_future_set(&mut future_new_set, &mut new_snapshot, Default::default());
		assert_eq!(future_new_set, Some(FutureNewSet {
			new_set: vec![(2.into(), address)].into_iter().collect(),
			block: Default::default(),
		}));
		assert_eq!(new_snapshot, KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(1.into(), address)].into_iter().collect(),
			..Default::default()
		});
	}

	#[test]
	fn future_set_is_updated_when_set_differs() {
		let address = "127.0.0.1:12000".parse().unwrap();

		let mut future_new_set = Some(FutureNewSet {
			new_set: vec![(2.into(), address)].into_iter().collect(),
			block: Default::default(),
		});
		let mut new_snapshot = KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(3.into(), address)].into_iter().collect(),
			..Default::default()
		};
		update_future_set(&mut future_new_set, &mut new_snapshot, 1.into());
		assert_eq!(future_new_set, Some(FutureNewSet {
			new_set: vec![(3.into(), address)].into_iter().collect(),
			block: 1.into(),
		}));
		assert_eq!(new_snapshot, KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(1.into(), address)].into_iter().collect(),
			..Default::default()
		});
	}

	#[test]
	fn future_set_is_not_updated_when_set_is_the_same() {
		let address = "127.0.0.1:12000".parse().unwrap();

		let mut future_new_set = Some(FutureNewSet {
			new_set: vec![(2.into(), address)].into_iter().collect(),
			block: Default::default(),
		});
		let mut new_snapshot = KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(2.into(), address)].into_iter().collect(),
			..Default::default()
		};
		update_future_set(&mut future_new_set, &mut new_snapshot, 1.into());
		assert_eq!(future_new_set, Some(FutureNewSet {
			new_set: vec![(2.into(), address)].into_iter().collect(),
			block: Default::default(),
		}));
		assert_eq!(new_snapshot, KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(1.into(), address)].into_iter().collect(),
			..Default::default()
		});
	}

	#[test]
	fn when_updating_confirmations_nothing_is_changed_if_no_future_set() {
		let address = "127.0.0.1:12000".parse().unwrap();

		let mut future_new_set = None;
		let mut snapshot = KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(1.into(), address)].into_iter().collect(),
			..Default::default()
		};
		let snapshot_copy = snapshot.clone();
		update_number_of_confirmations(
			&|| 1.into(),
			&|_| Some(MIGRATION_CONFIRMATIONS_REQUIRED),
			&mut future_new_set, &mut snapshot);
		assert_eq!(future_new_set, None);
		assert_eq!(snapshot, snapshot_copy);
	}

	#[test]
	fn when_updating_confirmations_migration_is_scheduled() {
		let address = "127.0.0.1:12000".parse().unwrap();

		let mut future_new_set = Some(FutureNewSet {
			new_set: vec![(2.into(), address)].into_iter().collect(),
			block: Default::default(),
		});
		let mut snapshot = KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(1.into(), address)].into_iter().collect(),
			..Default::default()
		};
		update_number_of_confirmations(
			&|| 1.into(),
			&|_| Some(MIGRATION_CONFIRMATIONS_REQUIRED),
			&mut future_new_set, &mut snapshot);
		assert_eq!(future_new_set, None);
		assert_eq!(snapshot, KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(2.into(), address)].into_iter().collect(),
			..Default::default()
		});
	}

	#[test]
	fn when_updating_confirmations_migration_is_not_scheduled_when_not_enough_confirmations() {
		let address = "127.0.0.1:12000".parse().unwrap();

		let mut future_new_set = Some(FutureNewSet {
			new_set: vec![(2.into(), address)].into_iter().collect(),
			block: Default::default(),
		});
		let mut snapshot = KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(1.into(), address)].into_iter().collect(),
			..Default::default()
		};
		let future_new_set_copy = future_new_set.clone();
		let snapshot_copy = snapshot.clone();
		update_number_of_confirmations(
			&|| 1.into(),
			&|_| Some(MIGRATION_CONFIRMATIONS_REQUIRED - 1),
			&mut future_new_set, &mut snapshot);
		assert_eq!(future_new_set, future_new_set_copy);
		assert_eq!(snapshot, snapshot_copy);
	}

	#[test]
	fn when_updating_confirmations_migration_is_reset_when_reorganized() {
		let address = "127.0.0.1:12000".parse().unwrap();

		let mut future_new_set = Some(FutureNewSet {
			new_set: vec![(2.into(), address)].into_iter().collect(),
			block: 1.into(),
		});
		let mut snapshot = KeyServerSetSnapshot {
			current_set: vec![(1.into(), address)].into_iter().collect(),
			new_set: vec![(1.into(), address)].into_iter().collect(),
			..Default::default()
		};
		let snapshot_copy = snapshot.clone();
		update_number_of_confirmations(
			&|| 2.into(),
			&|_| None,
			&mut future_new_set, &mut snapshot);
		assert_eq!(future_new_set, Some(FutureNewSet {
			new_set: vec![(2.into(), address)].into_iter().collect(),
			block: 2.into(),
		}));
		assert_eq!(snapshot, snapshot_copy);
	}
}
