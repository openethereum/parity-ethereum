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

use std::sync::{Arc, Weak};
use bytes::Bytes;
use common_types::{
	ids::BlockId,
	BlockNumber,
	transaction::{Transaction, SignedTransaction, Action},
	chain_notify::NewBlocks,
	tree_route::TreeRoute,
	filter::Filter,
	log_entry::LocalizedLogEntry,
};
use parking_lot::RwLock;
use ethereum_types::{H256, Address};
use ethcore::client::Client;
use client_traits::BlockChainClient;
use call_contract::CallContract;
use client_traits::ChainNotify;
use client_traits::{ChainInfo, Nonce};
use ethcore::miner::{Miner, MinerService};
use sync::SyncProvider;
use helpers::{get_confirmed_block_hash, REQUEST_CONFIRMATIONS_REQUIRED};
use {Error, NodeKeyPair, ContractAddress};
use registrar::RegistrarClient;

/// Wrapps client ChainNotify in order to send signal about new blocks
pub trait NewBlocksNotify: Send + Sync {
	/// Fires when chain has new blocks.
	/// Sends this signal only, if contracts' update required
	fn new_blocks(&self, _new_enacted_len: usize) {
		// does nothing by default
	}
}

/// 'Trusted' client weak reference.
pub struct TrustedClient {
	/// This key server node key pair.
	self_key_pair: Arc<dyn NodeKeyPair>,
	/// Blockchain client.
	client: Weak<Client>,
	/// Sync provider.
	sync: Weak<dyn SyncProvider>,
	/// Miner service.
	miner: Weak<Miner>,
	/// Chain new blocks listeners
	listeners: RwLock<Vec<Weak<dyn NewBlocksNotify>>>,
}

impl TrustedClient {
	/// Create new trusted client.
	pub fn new(self_key_pair: Arc<dyn NodeKeyPair>, client: Arc<Client>, sync: Arc<dyn SyncProvider>, miner: Arc<Miner>) -> Arc<Self> {
		let trusted_client = Arc::new(TrustedClient {
			self_key_pair,
			client: Arc::downgrade(&client),
			sync: Arc::downgrade(&sync),
			miner: Arc::downgrade(&miner),
			listeners: RwLock::default(),
		});
		client.add_notify(trusted_client.clone());
		trusted_client
	}

	/// Adds listener for chain's NewBlocks event
	pub fn add_listener(&self, target: Arc<dyn NewBlocksNotify>) {
		self.listeners.write().push(Arc::downgrade(&target));
	}

	fn notify_listeners(&self, new_enacted_len: usize) {
		for np in self.listeners.read().iter() {
			if let Some(n) = np.upgrade() {
				n.new_blocks(new_enacted_len);
			}
		}
	}

	/// Check if the underlying client is in the trusted state
	pub fn is_trusted(&self) -> bool {
		self.get_trusted().is_some()
	}

	/// Get 'trusted' `Client` reference only if it is synchronized && trusted.
	fn get_trusted(&self) -> Option<Arc<Client>> {
		self.client.upgrade()
			.and_then(|client| self.sync.upgrade().map(|sync| (client, sync)))
			.and_then(|(client, sync)| {
				let is_synced = !sync.is_major_syncing();
				let is_trusted = client.chain_info().security_level().is_full();
				match is_synced && is_trusted {
					true => Some(client),
					false => None,
				}
			})
	}

	/// Transact contract.
	pub fn transact_contract(&self, contract: Address, tx_data: Bytes) -> Result<(), Error> {
		let client = self.client.upgrade().ok_or_else(|| Error::Internal("cannot submit tx when client is offline".into()))?;
		let miner = self.miner.upgrade().ok_or_else(|| Error::Internal("cannot submit tx when miner is offline".into()))?;
		let engine = client.engine();
		let transaction = Transaction {
			nonce: client.latest_nonce(&self.self_key_pair.address()),
			action: Action::Call(contract),
			gas: miner.authoring_params().gas_range_target.0,
			gas_price: miner.sensible_gas_price(),
			value: Default::default(),
			data: tx_data,
		};
		let chain_id = engine.signing_chain_id(&client.latest_env_info());
		let signature = self.self_key_pair.sign(&transaction.hash(chain_id))?;
		let signed = SignedTransaction::new(transaction.with_signature(signature, chain_id))?;
		miner.import_own_transaction(&*client, signed.into())
			.map_err(|e| Error::Internal(format!("failed to import tx: {}", e)))
	}

	/// Read contract address. If address source is registry, address only returned if current client state is
	/// trusted. Address from registry is read from registry from block latest block with
	/// REQUEST_CONFIRMATIONS_REQUIRED confirmations.
	pub fn read_contract_address(
		&self,
		registry_name: &str,
		address: &ContractAddress
	) -> Option<Address> {
		match *address {
			ContractAddress::Address(ref address) => Some(address.clone()),
			ContractAddress::Registry => self.get_trusted().and_then(|client|
				get_confirmed_block_hash(&*self, REQUEST_CONFIRMATIONS_REQUIRED)
					.and_then(|block| {
						client.get_address(registry_name, BlockId::Hash(block))
							.unwrap_or(None)
					})
			),
		}
	}

	/// Client's call_contract wrapper
	pub fn call_contract(&self, block_id: BlockId, contract_address: Address, data: Bytes) -> Result<Bytes, String> {
		if let Some(client) = self.get_trusted() {
			client.call_contract(block_id, contract_address, data)
		} else {
			Err("Calling ACL contract without trusted blockchain client".into())
		}
	}

	/// Client's block_hash wrapper
	pub fn block_hash(&self, id: BlockId) -> Option<H256> {
		if let Some(client) = self.get_trusted() {
			client.block_hash(id)
		} else {
			None
		}
	}

	/// Client's block_number wrapper
	pub fn block_number(&self, id: BlockId) -> Option<BlockNumber> {
		if let Some(client) = self.get_trusted() {
			client.block_number(id)
		} else {
			None
		}
	}

	/// Client's tree_route wrapper
	pub fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute> {
		if let Some(client) = self.get_trusted() {
			client.tree_route(from, to)
		} else {
			None
		}
	}

	/// Client's logs wrapper
	pub fn logs(&self, filter: Filter) -> Option<Vec<LocalizedLogEntry>> {
		if let Some(client) = self.get_trusted() {
			client.logs(filter).ok()
		} else {
			None
		}
	}
}

impl ChainNotify for TrustedClient {
	fn new_blocks(&self, new_blocks: NewBlocks) {
		if new_blocks.has_more_blocks_to_import { return }
		if !new_blocks.route.enacted().is_empty() || !new_blocks.route.retracted().is_empty() {
			let enacted_len = new_blocks.route.enacted().len();
			self.notify_listeners(enacted_len);
		}
	}
}

