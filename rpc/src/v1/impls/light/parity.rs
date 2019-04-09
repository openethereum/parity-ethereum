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

//! Parity-specific rpc implementation.
use std::sync::Arc;
use std::collections::BTreeMap;

use version::version_data;

use crypto::DEFAULT_MAC;
use ethkey::{crypto::ecies, Brain, Generator};
use ethstore::random_phrase;
use sync::{LightSyncInfo, LightSyncProvider, LightNetworkDispatcher, ManageNetwork};
use updater::VersionInfo as UpdaterVersionInfo;
use ethereum_types::{H64, H160, H256, H512, U64, U256};
use ethcore_logger::RotatingLogger;

use jsonrpc_core::{Result, BoxFuture};
use jsonrpc_core::futures::{future, Future};
use light::on_demand::OnDemandRequester;
use v1::helpers::{self, errors, ipfs, NetworkSettings, verify_signature};
use v1::helpers::external_signer::{SignerService, SigningQueue};
use v1::helpers::dispatch::LightDispatcher;
use v1::helpers::light_fetch::{LightFetch, light_all_transactions};
use v1::metadata::Metadata;
use v1::traits::Parity;
use v1::types::{
	Bytes, CallRequest,
	Peers, Transaction, RpcSettings, Histogram,
	TransactionStats, LocalTransactionStatus,
	LightBlockNumber, ChainStatus, Receipt,
	BlockNumber, ConsensusCapability, VersionInfo,
	OperationsInfo, Header, RichHeader, RecoveredAccount,
	Log, Filter,
};
use Host;

/// Parity implementation for light client.
pub struct ParityClient<S, OD>
where
	S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static,
	OD: OnDemandRequester + 'static
{
	light_dispatch: Arc<LightDispatcher<S, OD>>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
	signer: Option<Arc<SignerService>>,
	ws_address: Option<Host>,
	gas_price_percentile: usize,
}

impl<S, OD> ParityClient<S, OD>
where
	S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static,
	OD: OnDemandRequester + 'static
{
	/// Creates new `ParityClient`.
	pub fn new(
		light_dispatch: Arc<LightDispatcher<S, OD>>,
		logger: Arc<RotatingLogger>,
		settings: Arc<NetworkSettings>,
		signer: Option<Arc<SignerService>>,
		ws_address: Option<Host>,
		gas_price_percentile: usize,
	) -> Self {
		ParityClient {
			light_dispatch,
			logger,
			settings,
			signer,
			ws_address,
			gas_price_percentile,
		}
	}

	/// Create a light blockchain data fetcher.
	fn fetcher(&self) -> LightFetch<S, OD>
	{
		LightFetch {
			client: self.light_dispatch.client.clone(),
			on_demand: self.light_dispatch.on_demand.clone(),
			sync: self.light_dispatch.sync.clone(),
			cache: self.light_dispatch.cache.clone(),
			gas_price_percentile: self.gas_price_percentile,
		}
	}
}

impl<S, OD> Parity for ParityClient<S, OD>
where
	S: LightSyncInfo + LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static,
	OD: OnDemandRequester + 'static
{
	type Metadata = Metadata;

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
			peers,
		})
	}

	fn net_port(&self) -> Result<u16> {
		Ok(self.settings.network_port)
	}

	fn node_name(&self) -> Result<String> {
		Ok(self.settings.name.clone())
	}

	fn registry_address(&self) -> Result<Option<H160>> {
		let reg = self.light_dispatch.client.engine().params().registrar;
		if reg == Default::default() {
			Ok(None)
		} else {
			Ok(Some(reg))
		}
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
		Ok(Brain::new(phrase).generate().expect("Brain::generate always returns Ok; qed").address())
	}

	fn list_accounts(&self, _: u64, _: Option<H160>, _: Option<BlockNumber>) -> Result<Option<Vec<H160>>> {
		Err(errors::light_unimplemented(None))
	}

	fn list_storage_keys(&self, _: H160, _: u64, _: Option<H256>, _: Option<BlockNumber>) -> Result<Option<Vec<H256>>> {
		Err(errors::light_unimplemented(None))
	}

	fn encrypt_message(&self, key: H512, phrase: Bytes) -> Result<Bytes> {
		ecies::encrypt(&key, &DEFAULT_MAC, &phrase.0)
			.map_err(errors::encryption)
			.map(Into::into)
	}

	fn pending_transactions(&self, limit: Option<usize>) -> Result<Vec<Transaction>> {
		let txq = self.light_dispatch.transaction_queue.read();
		let chain_info = self.light_dispatch.client.chain_info();
		Ok(
			txq.ready_transactions(chain_info.best_block_number, chain_info.best_block_timestamp)
				.into_iter()
				.take(limit.unwrap_or_else(usize::max_value))
				.map(Transaction::from_pending)
				.collect::<Vec<_>>()
		)
	}

	fn all_transactions(&self) -> Result<Vec<Transaction>> {
		Ok(
			light_all_transactions(&self.light_dispatch)
				.map(Transaction::from_pending)
				.collect()
		)
	}

	fn all_transaction_hashes(&self) -> Result<Vec<H256>> {
		Ok(
			light_all_transactions(&self.light_dispatch)
				.map(|tx| tx.transaction.hash())
				.collect()
		)
	}

	fn future_transactions(&self) -> Result<Vec<Transaction>> {
		let txq = self.light_dispatch.transaction_queue.read();
		let chain_info = self.light_dispatch.client.chain_info();
		Ok(
			txq.future_transactions(chain_info.best_block_number, chain_info.best_block_timestamp)
				.into_iter()
				.map(Transaction::from_pending)
				.collect::<Vec<_>>()
		)
	}

	fn pending_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>> {
		let stats = self.light_dispatch.sync.transactions_stats();
		Ok(stats.into_iter()
			.map(|(hash, stats)| (hash, stats.into()))
			.collect()
		)
	}

	fn local_transactions(&self) -> Result<BTreeMap<H256, LocalTransactionStatus>> {
		let mut map = BTreeMap::new();
		let chain_info = self.light_dispatch.client.chain_info();
		let (best_num, best_tm) = (chain_info.best_block_number, chain_info.best_block_timestamp);
		let txq = self.light_dispatch.transaction_queue.read();

		for pending in txq.ready_transactions(best_num, best_tm) {
			map.insert(pending.hash(), LocalTransactionStatus::Pending);
		}

		for future in txq.future_transactions(best_num, best_tm) {
			map.insert(future.hash(), LocalTransactionStatus::Future);
		}

		// TODO: other types?

		Ok(map)
	}

	fn ws_url(&self) -> Result<String> {
		helpers::to_url(&self.ws_address)
			.ok_or_else(errors::ws_disabled)
	}

	fn next_nonce(&self, address: H160) -> BoxFuture<U256> {
		Box::new(self.light_dispatch.next_nonce(address))
	}

	fn mode(&self) -> Result<String> {
		Err(errors::light_unimplemented(None))
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
		Ok(UpdaterVersionInfo::this().into())
	}

	fn releases_info(&self) -> Result<Option<OperationsInfo>> {
		Err(errors::light_unimplemented(None))
	}

	fn chain_status(&self) -> Result<ChainStatus> {
		let chain_info = self.light_dispatch.client.chain_info();

		let gap = chain_info.ancient_block_number.map(|x| U256::from(x + 1))
			.and_then(|first| chain_info.first_block_number.map(|last| (first, U256::from(last))));

		Ok(ChainStatus {
			block_gap: gap,
		})
	}

	fn node_kind(&self) -> Result<::v1::types::NodeKind> {
		use ::v1::types::{NodeKind, Availability, Capability};

		Ok(NodeKind {
			availability: Availability::Personal,
			capability: Capability::Light,
		})
	}

	fn block_header(&self, number: Option<BlockNumber>) -> BoxFuture<RichHeader> {
		use types::encoded;

		let engine = self.light_dispatch.client.engine().clone();
		let from_encoded = move |encoded: encoded::Header| {
			let header = encoded.decode().map_err(errors::decode)?;
			let extra_info = engine.extra_info(&header);
			Ok(RichHeader {
				inner: Header {
					hash: Some(header.hash()),
					size: Some(encoded.rlp().as_raw().len().into()),
					parent_hash: *header.parent_hash(),
					uncles_hash: *header.uncles_hash(),
					author: *header.author(),
					miner: *header.author(),
					state_root: *header.state_root(),
					transactions_root: *header.transactions_root(),
					receipts_root: *header.receipts_root(),
					number: Some(header.number().into()),
					gas_used: *header.gas_used(),
					gas_limit: *header.gas_limit(),
					logs_bloom: *header.log_bloom(),
					timestamp: header.timestamp().into(),
					difficulty: *header.difficulty(),
					seal_fields: header.seal().iter().cloned().map(Into::into).collect(),
					extra_data: Bytes::new(header.extra_data().clone()),
				},
				extra_info,
			})
		};
		let id = number.unwrap_or_default().to_block_id();
		Box::new(self.fetcher().header(id).and_then(from_encoded))
	}

	fn block_receipts(&self, number: Option<BlockNumber>) -> BoxFuture<Vec<Receipt>> {
		let id = number.unwrap_or_default().to_block_id();
		Box::new(self.fetcher().receipts(id).and_then(|receipts| Ok(receipts.into_iter().map(Into::into).collect())))
	}

	fn ipfs_cid(&self, content: Bytes) -> Result<String> {
		ipfs::cid(content)
	}

	fn call(&self, _requests: Vec<CallRequest>, _block: Option<BlockNumber>) -> Result<Vec<Bytes>> {
		Err(errors::light_unimplemented(None))
	}

	fn submit_work_detail(&self, _nonce: H64, _pow_hash: H256, _mix_hash: H256) -> Result<H256> {
		Err(errors::light_unimplemented(None))
	}

	fn status(&self) -> Result<()> {
		let has_peers = self.settings.is_dev_chain || self.light_dispatch.sync.peer_numbers().connected > 0;
		let is_importing = (*self.light_dispatch.sync).is_major_importing();

		if has_peers && !is_importing {
			Ok(())
		} else {
			Err(errors::status_error(has_peers))
		}
	}

	fn logs_no_tx_hash(&self, filter: Filter) -> BoxFuture<Vec<Log>> {
		let filter = match filter.try_into() {
			Ok(value) => value,
			Err(err) => return Box::new(future::err(err)),
		};
		Box::new(self.fetcher().logs_no_tx_hash(filter)) as BoxFuture<_>
	}

	fn verify_signature(&self, is_prefixed: bool, message: Bytes, r: H256, s: H256, v: U64) -> Result<RecoveredAccount> {
		verify_signature(is_prefixed, message, r, s, v, self.light_dispatch.client.signing_chain_id())
	}
}
