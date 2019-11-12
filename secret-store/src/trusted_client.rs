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
	filter::Filter as BlockchainFilter,
	log_entry::LocalizedLogEntry,
};
use parking_lot::RwLock;
use ethereum_types::{H256, Address, Public};
use ethcore::client::Client;
use ethabi::RawLog;
use crypto::publickey::{Signature, Error as EthKeyError};
use client_traits::BlockChainClient;
use call_contract::CallContract;
use client_traits::{ChainInfo, Nonce, ChainNotify};
use ethcore::miner::{Miner, MinerService};
use sync::SyncProvider;
use {Error, ContractAddress};
use registrar::RegistrarClient;

// TODO: Instead of a constant, make this based on consensus finality.
/// Number of confirmations required before request can be processed.
const REQUEST_CONFIRMATIONS_REQUIRED: u64 = 3;

/// Key pair with signing ability.
pub trait SigningKeyPair: Send + Sync {
	/// Public portion of key.
	fn public(&self) -> &Public;
	/// Address of key owner.
	fn address(&self) -> Address;
	/// Sign data with the key.
	fn sign(&self, data: &H256) -> Result<Signature, EthKeyError>;
}

/// Wrapps client ChainNotify in order to send signal about new blocks
pub trait NewBlocksNotify: Send + Sync {
	/// Fires when chain has new blocks.
	/// Sends this signal only, if contracts' update required
	fn new_blocks(&self, _new_enacted_len: usize) {
		// does nothing by default
	}
}

/// Blockchain logs Filter.
#[derive(Debug, PartialEq)]
pub struct Filter {
	/// Blockchain will be searched from this block.
	pub from_block: BlockId,

	/// Search addresses.
	///
	/// If None, match all.
	/// If specified, log must be produced by one of these addresses.
	pub address: Option<Vec<Address>>,

	/// Search topics.
	///
	/// If None, match all.
	/// If specified, log must contain one of these topics.
	pub topics: Vec<Option<Vec<H256>>>,
}

/// 'Trusted' client weak reference.
pub struct TrustedClient {
	/// This key server node key pair.
	self_key_pair: Arc<dyn SigningKeyPair>,
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
	pub fn new(self_key_pair: Arc<dyn SigningKeyPair>, client: Arc<Client>, sync: Arc<dyn SyncProvider>, miner: Arc<Miner>) -> Arc<Self> {
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
				self.get_confirmed_block_hash()
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
	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute> {
		if let Some(client) = self.get_trusted() {
			client.tree_route(from, to)
		} else {
			None
		}
	}

	/// Client's logs wrapper
	fn logs(&self, filter: BlockchainFilter) -> Option<Vec<LocalizedLogEntry>> {
		if let Some(client) = self.get_trusted() {
			client.logs(filter).ok()
		} else {
			None
		}
	}

	/// Retrieve last blockchain logs for the filter
	pub fn retrieve_last_logs(&self, filter: Filter) -> Option<Vec<RawLog>> {
		let confirmed_block = match self.get_confirmed_block_hash() {
			Some(confirmed_block) => confirmed_block,
			None => return None, // no block with enough confirmations
		};

		let from_block = self.block_hash(filter.from_block).unwrap_or_else(|| confirmed_block);
		let first_block = match self.tree_route(&from_block, &confirmed_block) {
			// if we have a route from last_log_block to confirmed_block => search for logs on this route
			//
			// potentially this could lead us to reading same logs twice when reorganizing to the fork, which
			// already has been canonical previosuly
			// the worst thing that can happen in this case is spending some time reading unneeded data from SS db
			Some(ref route) if route.index < route.blocks.len() => route.blocks[route.index],
			// else we care only about confirmed block
			_ => confirmed_block.clone(),
		};

		self.logs(BlockchainFilter {
			from_block: BlockId::Hash(first_block),
			to_block: BlockId::Hash(confirmed_block),
			address: filter.address,
			topics: filter.topics,
			limit: None,
		})
		.map(|blockchain_logs| {
			blockchain_logs
				.into_iter()
					.map(|log| {
						let raw_log: RawLog = (log.entry.topics.into_iter().map(|t| t.0.into()).collect(), log.entry.data).into();
						raw_log
					})
				.collect::<Vec<_>>()
		})
	}

	/// Get hash of the last block with at least n confirmations.
	pub fn get_confirmed_block_hash(&self) -> Option<H256> {
		self.block_number(BlockId::Latest)
			.map(|b| b.saturating_sub(REQUEST_CONFIRMATIONS_REQUIRED))
			.and_then(|b| self.block_hash(BlockId::Number(b)))
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

