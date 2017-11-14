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

//! Parity-specific rpc implementation.
use std::sync::Arc;
use std::collections::{BTreeMap, HashSet};

use util::misc::version_data;

use crypto::{ecies, DEFAULT_MAC};
use ethkey::{Brain, Generator};
use ethstore::random_phrase;
use ethsync::LightSyncProvider;
use ethcore::account_provider::AccountProvider;
use ethcore_logger::RotatingLogger;
use node_health::{NodeHealth, Health};

use light::client::LightChainClient;

use jsonrpc_core::{Result, BoxFuture};
use jsonrpc_core::futures::Future;
use jsonrpc_macros::Trailing;
use v1::helpers::{self, errors, ipfs, SigningQueue, SignerService, NetworkSettings};
use v1::helpers::dispatch::LightDispatcher;
use v1::helpers::light_fetch::LightFetch;
use v1::metadata::Metadata;
use v1::traits::Parity;
use v1::types::{
	Bytes, U256, U64, H160, H256, H512, CallRequest,
	Peers, Transaction, RpcSettings, Histogram,
	TransactionStats, LocalTransactionStatus,
	BlockNumber, ConsensusCapability, VersionInfo,
	OperationsInfo, DappId, ChainStatus,
	AccountInfo, HwAccountInfo, Header, RichHeader,
};
use Host;

/// Parity implementation for light client.
pub struct ParityClient {
	client: Arc<LightChainClient>,
	light_dispatch: Arc<LightDispatcher>,
	accounts: Arc<AccountProvider>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
	health: NodeHealth,
	signer: Option<Arc<SignerService>>,
	dapps_address: Option<Host>,
	ws_address: Option<Host>,
	eip86_transition: u64,
}

impl ParityClient {
	/// Creates new `ParityClient`.
	pub fn new(
		client: Arc<LightChainClient>,
		light_dispatch: Arc<LightDispatcher>,
		accounts: Arc<AccountProvider>,
		logger: Arc<RotatingLogger>,
		settings: Arc<NetworkSettings>,
		health: NodeHealth,
		signer: Option<Arc<SignerService>>,
		dapps_address: Option<Host>,
		ws_address: Option<Host>,
	) -> Self {
		ParityClient {
			light_dispatch,
			accounts,
			logger,
			settings,
			health,
			signer,
			dapps_address,
			ws_address,
			eip86_transition: client.eip86_transition(),
			client: client,
		}
	}

	/// Create a light blockchain data fetcher.
	fn fetcher(&self) -> LightFetch {
		LightFetch {
			client: self.light_dispatch.client.clone(),
			on_demand: self.light_dispatch.on_demand.clone(),
			sync: self.light_dispatch.sync.clone(),
			cache: self.light_dispatch.cache.clone(),
		}
	}
}

impl Parity for ParityClient {
	type Metadata = Metadata;

	fn accounts_info(&self, dapp: Trailing<DappId>) -> Result<BTreeMap<H160, AccountInfo>> {
		let dapp = dapp.unwrap_or_default();

		let store = &self.accounts;
		let dapp_accounts = store
			.note_dapp_used(dapp.clone().into())
			.and_then(|_| store.dapp_addresses(dapp.into()))
			.map_err(|e| errors::account("Could not fetch accounts.", e))?
			.into_iter().collect::<HashSet<_>>();

		let info = store.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		let other = store.addresses_info();

		Ok(info
			.into_iter()
			.chain(other.into_iter())
			.filter(|&(ref a, _)| dapp_accounts.contains(a))
			.map(|(a, v)| (H160::from(a), AccountInfo { name: v.name }))
			.collect()
		)
	}

	fn hardware_accounts_info(&self) -> Result<BTreeMap<H160, HwAccountInfo>> {
		let store = &self.accounts;
		let info = store.hardware_accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		Ok(info
			.into_iter()
			.map(|(a, v)| (H160::from(a), HwAccountInfo { name: v.name, manufacturer: v.meta }))
			.collect()
		)
	}

	fn locked_hardware_accounts_info(&self) -> Result<Vec<String>> {
		let store = &self.accounts;
		Ok(store.locked_hardware_accounts().map_err(|e| errors::account("Error communicating with hardware wallet.", e))?)
	}

	fn default_account(&self, meta: Self::Metadata) -> Result<H160> {
		let dapp_id = meta.dapp_id();
		Ok(self.accounts
			.dapp_addresses(dapp_id.into())
			.ok()
			.and_then(|accounts| accounts.get(0).cloned())
			.map(|acc| acc.into())
			.unwrap_or_default())
	}

	fn transactions_limit(&self) -> Result<usize> {
		Ok(usize::max_value())
	}

	fn min_gas_price(&self) -> Result<U256> {
		Ok(U256::default())
	}

	fn extra_data(&self) -> Result<Bytes> {
		Ok(Bytes::default())
	}

	fn gas_floor_target(&self) -> Result<U256> {
		Ok(U256::default())
	}

	fn gas_ceil_target(&self) -> Result<U256> {
		Ok(U256::default())
	}

	fn dev_logs(&self) -> Result<Vec<String>> {
		let logs = self.logger.logs();
		Ok(logs.as_slice().to_owned())
	}

	fn dev_logs_levels(&self) -> Result<String> {
		Ok(self.logger.levels().to_owned())
	}

	fn net_chain(&self) -> Result<String> {
		Ok(self.settings.chain.clone())
	}

	fn net_peers(&self) -> Result<Peers> {
		let peers = self.light_dispatch.sync.peers().into_iter().map(Into::into).collect();
		let peer_numbers = self.light_dispatch.sync.peer_numbers();

		Ok(Peers {
			active: peer_numbers.active,
			connected: peer_numbers.connected,
			max: peer_numbers.max as u32,
			peers: peers,
		})
	}

	fn net_port(&self) -> Result<u16> {
		Ok(self.settings.network_port)
	}

	fn node_name(&self) -> Result<String> {
		Ok(self.settings.name.clone())
	}

	fn registry_address(&self) -> Result<Option<H160>> {
		Err(errors::light_unimplemented(None))
	}

	fn rpc_settings(&self) -> Result<RpcSettings> {
		Ok(RpcSettings {
			enabled: self.settings.rpc_enabled,
			interface: self.settings.rpc_interface.clone(),
			port: self.settings.rpc_port as u64,
		})
	}

	fn default_extra_data(&self) -> Result<Bytes> {
		Ok(Bytes::new(version_data()))
	}

	fn gas_price_histogram(&self) -> BoxFuture<Histogram> {
		Box::new(self.light_dispatch.gas_price_corpus()
			.and_then(|corpus| corpus.histogram(10).ok_or_else(errors::not_enough_data))
			.map(Into::into))
	}

	fn unsigned_transactions_count(&self) -> Result<usize> {
		match self.signer {
			None => Err(errors::signer_disabled()),
			Some(ref signer) => Ok(signer.len()),
		}
	}

	fn generate_secret_phrase(&self) -> Result<String> {
		Ok(random_phrase(12))
	}

	fn phrase_to_address(&self, phrase: String) -> Result<H160> {
		Ok(Brain::new(phrase).generate().unwrap().address().into())
	}

	fn list_accounts(&self, _: u64, _: Option<H160>, _: Trailing<BlockNumber>) -> Result<Option<Vec<H160>>> {
		Err(errors::light_unimplemented(None))
	}

	fn list_storage_keys(&self, _: H160, _: u64, _: Option<H256>, _: Trailing<BlockNumber>) -> Result<Option<Vec<H256>>> {
		Err(errors::light_unimplemented(None))
	}

	fn encrypt_message(&self, key: H512, phrase: Bytes) -> Result<Bytes> {
		ecies::encrypt(&key.into(), &DEFAULT_MAC, &phrase.0)
			.map_err(errors::encryption)
			.map(Into::into)
	}

	fn pending_transactions(&self) -> Result<Vec<Transaction>> {
		let txq = self.light_dispatch.transaction_queue.read();
		let chain_info = self.light_dispatch.client.chain_info();
		Ok(
			txq.ready_transactions(chain_info.best_block_number, chain_info.best_block_timestamp)
				.into_iter()
				.map(|tx| Transaction::from_pending(tx, chain_info.best_block_number, self.eip86_transition))
				.collect::<Vec<_>>()
		)
	}

	fn future_transactions(&self) -> Result<Vec<Transaction>> {
		let txq = self.light_dispatch.transaction_queue.read();
		let chain_info = self.light_dispatch.client.chain_info();
		Ok(
			txq.future_transactions(chain_info.best_block_number, chain_info.best_block_timestamp)
				.into_iter()
				.map(|tx| Transaction::from_pending(tx, chain_info.best_block_number, self.eip86_transition))
				.collect::<Vec<_>>()
		)
	}

	fn pending_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>> {
		let stats = self.light_dispatch.sync.transactions_stats();
		Ok(stats.into_iter()
		   .map(|(hash, stats)| (hash.into(), stats.into()))
		   .collect()
		)
	}

	fn local_transactions(&self) -> Result<BTreeMap<H256, LocalTransactionStatus>> {
		let mut map = BTreeMap::new();
		let chain_info = self.light_dispatch.client.chain_info();
		let (best_num, best_tm) = (chain_info.best_block_number, chain_info.best_block_timestamp);
		let txq = self.light_dispatch.transaction_queue.read();

		for pending in txq.ready_transactions(best_num, best_tm) {
			map.insert(pending.hash().into(), LocalTransactionStatus::Pending);
		}

		for future in txq.future_transactions(best_num, best_tm) {
			map.insert(future.hash().into(), LocalTransactionStatus::Future);
		}

		// TODO: other types?

		Ok(map)
	}

	fn dapps_url(&self) -> Result<String> {
		helpers::to_url(&self.dapps_address)
			.ok_or_else(|| errors::dapps_disabled())
	}

	fn ws_url(&self) -> Result<String> {
		helpers::to_url(&self.ws_address)
			.ok_or_else(|| errors::ws_disabled())
	}

	fn next_nonce(&self, address: H160) -> BoxFuture<U256> {
		Box::new(self.light_dispatch.next_nonce(address.into()).map(Into::into))
	}

	fn mode(&self) -> Result<String> {
		Err(errors::light_unimplemented(None))
	}

	fn chain_id(&self) -> Result<Option<U64>> {
		Ok(self.client.signing_chain_id().map(U64::from))
	}

	fn chain(&self) -> Result<String> {
		Ok(self.settings.chain.clone())
	}

	fn enode(&self) -> Result<String> {
		self.light_dispatch.sync.enode().ok_or_else(errors::network_disabled)
	}

	fn consensus_capability(&self) -> Result<ConsensusCapability> {
		Err(errors::light_unimplemented(None))
	}

	fn version_info(&self) -> Result<VersionInfo> {
		Err(errors::light_unimplemented(None))
	}

	fn releases_info(&self) -> Result<Option<OperationsInfo>> {
		Err(errors::light_unimplemented(None))
	}

	fn chain_status(&self) -> Result<ChainStatus> {
		let chain_info = self.light_dispatch.client.chain_info();

		let gap = chain_info.ancient_block_number.map(|x| U256::from(x + 1))
			.and_then(|first| chain_info.first_block_number.map(|last| (first, U256::from(last))));

		Ok(ChainStatus {
			block_gap: gap.map(|(x, y)| (x.into(), y.into())),
		})
	}

	fn node_kind(&self) -> Result<::v1::types::NodeKind> {
		use ::v1::types::{NodeKind, Availability, Capability};

		Ok(NodeKind {
			availability: Availability::Personal,
			capability: Capability::Light,
		})
	}

	fn block_header(&self, number: Trailing<BlockNumber>) -> BoxFuture<RichHeader> {
		use ethcore::encoded;

		let engine = self.light_dispatch.client.engine().clone();
		let from_encoded = move |encoded: encoded::Header| {
			let header = encoded.decode();
			let extra_info = engine.extra_info(&header);
			RichHeader {
				inner: Header {
					hash: Some(header.hash().into()),
					size: Some(encoded.rlp().as_raw().len().into()),
					parent_hash: header.parent_hash().clone().into(),
					uncles_hash: header.uncles_hash().clone().into(),
					author: header.author().clone().into(),
					miner: header.author().clone().into(),
					state_root: header.state_root().clone().into(),
					transactions_root: header.transactions_root().clone().into(),
					receipts_root: header.receipts_root().clone().into(),
					number: Some(header.number().into()),
					gas_used: header.gas_used().clone().into(),
					gas_limit: header.gas_limit().clone().into(),
					logs_bloom: header.log_bloom().clone().into(),
					timestamp: header.timestamp().into(),
					difficulty: header.difficulty().clone().into(),
					seal_fields: header.seal().iter().cloned().map(Into::into).collect(),
					extra_data: Bytes::new(header.extra_data().clone()),
				},
				extra_info: extra_info,
			}
		};

		Box::new(self.fetcher().header(number.unwrap_or_default().into()).map(from_encoded))
	}

	fn ipfs_cid(&self, content: Bytes) -> Result<String> {
		ipfs::cid(content)
	}

	fn call(&self, _meta: Self::Metadata, _requests: Vec<CallRequest>, _block: Trailing<BlockNumber>) -> Result<Vec<Bytes>> {
		Err(errors::light_unimplemented(None))
	}

	fn node_health(&self) -> BoxFuture<Health> {
		Box::new(self.health.health()
			.map_err(|err| errors::internal("Health API failure.", err)))
	}
}
