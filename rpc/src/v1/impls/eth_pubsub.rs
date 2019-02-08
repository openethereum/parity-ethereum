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

//! Eth PUB-SUB rpc implementation.

use std::sync::{Arc, Weak};
use std::collections::BTreeMap;

use jsonrpc_core::{BoxFuture, Result, Error};
use jsonrpc_core::futures::{self, Future, IntoFuture};
use jsonrpc_pubsub::{SubscriptionId, typed::{Sink, Subscriber}};

use v1::helpers::{errors, limit_logs, Subscribers};
use v1::helpers::light_fetch::LightFetch;
use v1::helpers::block_import::is_major_importing;
use v1::metadata::Metadata;
use v1::traits::EthPubSub;
use v1::types::{pubsub, RichHeader, Log, SyncStatus, SyncInfo};

use ethcore::client::{BlockChainClient, ChainNotify, NewBlocks, ChainRouteType, BlockId};
use ethcore::snapshot::SnapshotService;
use ethereum_types::{H256,U256};
use light::cache::Cache;
use light::client::{LightChainClient, LightChainNotify};
use light::on_demand::OnDemand;
use parity_runtime::Executor;
use parking_lot::{RwLock, Mutex};
use sync::{LightSync, SyncProvider};
use types::encoded;
use types::filter::Filter as EthFilter;

type Client = Sink<pubsub::Result>;

/// Eth PubSub implementation.
pub struct EthPubSubClient<CNH>
{
	handler: Arc<CNH>,
	ethpubsub_notifier: Arc<EthPubSubNotifier>
}

impl<CNH> EthPubSubClient<CNH>
{
	/// Returns a chain notification handler.
	pub fn handler(&self) -> Weak<CNH> {
		Arc::downgrade(&self.handler)
	}
}

impl<C> EthPubSubClient<ChainNotificationHandlerFull<C>>
{
	/// Creates new `EthPubSubClient` for full node
	pub fn new_full(client: Arc<C>, snapshot: Arc<SnapshotService>, sync: Arc<SyncProvider>, executor: Executor) -> Self {
		let ethpubsub_notifier = Arc::new(EthPubSubNotifier::new(executor));

		EthPubSubClient {
			ethpubsub_notifier: ethpubsub_notifier.clone(),
			handler: Arc::new(ChainNotificationHandlerFull {
				client,
				snapshot,
				sync,
				ethpubsub_notifier,
			})
		}
	}

	/// Creates new `EthPubSubClient` with deterministic subscription ids.
	#[cfg(test)]
	pub fn new_test<C>(client: Arc<C>, snapshot: Arc<SnapshotService>, executor: Executor) -> Self {

		let ethpubsub_notifier = Arc::new(EthPubSubNotifier::new(executor));

		EthPubSubClient {
			ethpubsub_notifier: ethpubsub_notifier.clone(),
			handler: Arc::new(ChainNotificationHandlerFull {
				ethpubsub_notifier,
				client,
				snapshot,
				sync,
			})
		}
	}
}

impl EthPubSubClient<ChainNotificationHandlerLight>
{
	/// Creates new `EthPubSubClient` for `LightClient`.
	pub fn new_light(
		client: Arc<LightChainClient>,
		on_demand: Arc<OnDemand>,
		sync: Arc<LightSync>,
		cache: Arc<Mutex<Cache>>,
		executor: Executor,
		gas_price_percentile: usize,
	) -> Self {
		let fetch = LightFetch {
			client,
			on_demand,
			sync: sync.clone(),
			cache,
			gas_price_percentile,
		};

		let ethpubsub_notifier = Arc::new(EthPubSubNotifier::new(executor));

		EthPubSubClient {
			ethpubsub_notifier: ethpubsub_notifier.clone(),
			handler: Arc::new(ChainNotificationHandlerLight {
				client: Arc::new(fetch),
				sync,
				ethpubsub_notifier,
			})
		}
	}
}

/// Stores and manages EthPubSub subscriptions
pub struct EthPubSubNotifier {
	executor: Executor,
	heads_subscribers: Arc<RwLock<Subscribers<Client>>>,
	syncing_subscribers: Arc<RwLock<Subscribers<Client>>>,
	logs_subscribers: Arc<RwLock<Subscribers<(Client, EthFilter)>>>,
	transactions_subscribers: Arc<RwLock<Subscribers<Client>>>, 
}

impl EthPubSubNotifier {
	fn new(executor: Executor) -> Self {
		let heads_subscribers = Arc::new(RwLock::new(Subscribers::default()));
		let syncing_subscribers = Arc::new(RwLock::new(Subscribers::default()));
		let logs_subscribers = Arc::new(RwLock::new(Subscribers::default()));
		let transactions_subscribers = Arc::new(RwLock::new(Subscribers::default()));

		EthPubSubNotifier {
			executor,
			heads_subscribers,
			syncing_subscribers,
			logs_subscribers,
			transactions_subscribers
		}
	}

	fn notify(executor: &Executor, subscriber: &Client, result: pubsub::Result) {
		executor.spawn(subscriber
			.notify(Ok(result))
			.map(|_| ())
			.map_err(|e| warn!(target: "rpc", "Unable to send notification: {}", e))
		);
	}

	fn notify_heads(&self, headers: &[(encoded::Header, BTreeMap<String, String>)]) {
		for subscriber in self.heads_subscribers.read().values() {
			for &(ref header, ref extra_info) in headers {
				Self::notify(&self.executor, subscriber, pubsub::Result::Header(RichHeader {
					inner: header.into(),
					extra_info: extra_info.clone(),
				}));
			}
		}
	}

	fn notify_syncing(&self, sync_status: SyncStatus) {
		for subscriber in self.syncing_subscribers.read().values() {
			Self::notify(&self.executor, subscriber, pubsub::Result::SyncStatus(sync_status.clone()));
		}
	}

	fn notify_logs<F, T, Ex>(&self, enacted: &[(H256, Ex)], logs: F) where
		F: Fn(EthFilter, &Ex) -> T,
		Ex: Send,
		T: IntoFuture<Item = Vec<Log>, Error = Error>,
		T::Future: Send + 'static,
	{
		for &(ref subscriber, ref filter) in self.logs_subscribers.read().values() {
			let logs = futures::future::join_all(enacted
				.iter()
				.map(|&(hash, ref ex)| {
					let mut filter = filter.clone();
					filter.from_block = BlockId::Hash(hash);
					filter.to_block = filter.from_block.clone();
					logs(filter, ex).into_future()
				})
				.collect::<Vec<_>>()
			);
			let limit = filter.limit;
			let executor = self.executor.clone();
			let subscriber = subscriber.clone();
			self.executor.spawn(logs
				.map(move |logs| {
					let logs = logs.into_iter().flat_map(|log| log).collect();

					for log in limit_logs(logs, limit) {
						Self::notify(&executor, &subscriber, pubsub::Result::Log(log))
					}
				})
				.map_err(|e| warn!("Unable to fetch latest logs: {:?}", e))
			);
		}
	}

	/// Notify all subscribers about new transaction hashes.
	pub fn notify_new_transactions(&self, hashes: &[H256]) {
		for subscriber in self.transactions_subscribers.read().values() {
			for hash in hashes {
				Self::notify(&self.executor, subscriber, pubsub::Result::TransactionHash((*hash).into()));
			}
		}
	}

	pub fn no_new_blocks_listeners(&self) -> bool {
		self.heads_subscribers.read().is_empty() && self.logs_subscribers.read().is_empty() && self.syncing_subscribers.read().is_empty()
	}
}

/// Receives blockchain notifications and sends them to EthPubSubNotifier
/// (full node)
pub struct ChainNotificationHandlerFull<C>
{
	client: Arc<C>,
	snapshot: Arc<SnapshotService>,
	sync: Arc<SyncProvider>,
	pub ethpubsub_notifier: Arc<EthPubSubNotifier>,
}

impl<C: BlockChainClient> ChainNotify for ChainNotificationHandlerFull<C>
{
	fn new_blocks(&self, new_blocks: NewBlocks) {
		if self.ethpubsub_notifier.no_new_blocks_listeners() { return; }
		const EXTRA_INFO_PROOF: &'static str = "Object exists in in blockchain (fetched earlier), extra_info is always available if object exists; qed";
		let headers = new_blocks.route.route()
			.iter()
			.filter_map(|&(hash, ref typ)| {
				match typ {
					&ChainRouteType::Retracted => None,
					&ChainRouteType::Enacted => self.client.block_header(BlockId::Hash(hash))
				}
			})
			.map(|header| {
				let hash = header.hash();
				(header, self.client.block_extra_info(BlockId::Hash(hash)).expect(EXTRA_INFO_PROOF))
			})
			.collect::<Vec<_>>();

		// Headers
		self.ethpubsub_notifier.notify_heads(&headers);

		// We notify logs enacting and retracting as the order in route.
		self.ethpubsub_notifier.notify_logs(new_blocks.route.route(), |filter, ex| {
			match ex {
				&ChainRouteType::Enacted =>
					Ok(self.client.logs(filter).unwrap_or_default().into_iter().map(Into::into).collect()),
				&ChainRouteType::Retracted =>
					Ok(self.client.logs(filter).unwrap_or_default().into_iter().map(Into::into).map(|mut log: Log| {
						log.log_type = "removed".into();
						log.removed = true;
						log
					}).collect()),
			}
		});

		// Sync status
		let sync_status = {
			use ethcore::snapshot::RestorationStatus;

			let status = self.sync.status();
			let client = &self.client;
			let snapshot_status = self.snapshot.status();

			let (warping, warp_chunks_amount, warp_chunks_processed) = match snapshot_status {
				RestorationStatus::Ongoing { state_chunks, block_chunks, state_chunks_done, block_chunks_done } =>
					(true, Some(block_chunks + state_chunks), Some(block_chunks_done + state_chunks_done)),
				_ => (false, None, None),
			};

			if warping || is_major_importing(Some(status.state), client.queue_info()) {
				let chain_info = client.chain_info();
				let current_block = U256::from(chain_info.best_block_number);
				let highest_block = U256::from(status.highest_block_number.unwrap_or(status.start_block_number));

				let info = SyncInfo {
					starting_block: status.start_block_number.into(),
					current_block: current_block.into(),
					highest_block: highest_block.into(),
					warp_chunks_amount: warp_chunks_amount.map(|x| U256::from(x as u64)).map(Into::into),
					warp_chunks_processed: warp_chunks_processed.map(|x| U256::from(x as u64)).map(Into::into),
				};
				SyncStatus::Info(info)
			} else {
				SyncStatus::None
			}
		};

		self.ethpubsub_notifier.notify_syncing(sync_status);
	}
}

/// A light client wrapper struct.
pub trait LightClient: Send + Sync {
	/// Get a recent block header.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

	/// Fetch logs.
	fn logs(&self, filter: EthFilter) -> BoxFuture<Vec<Log>>;
}

impl LightClient for LightFetch {
	fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		self.client.block_header(id)
	}

	fn logs(&self, filter: EthFilter) -> BoxFuture<Vec<Log>> {
		Box::new(LightFetch::logs(self, filter)) as BoxFuture<_>
	}
}

/// Receives blockchain notifications and sends them to ethpubsub_notifier
/// (light node)
pub struct ChainNotificationHandlerLight
{
	client: Arc<LightFetch>,
	sync: Arc<LightSync>,
	pub ethpubsub_notifier: Arc<EthPubSubNotifier>,
}

impl LightChainNotify for ChainNotificationHandlerLight
{
	fn new_headers(
		&self,
		enacted: &[H256],
	) {
		let headers = enacted
			.iter()
			.filter_map(|hash| self.client.block_header(BlockId::Hash(*hash)))
			.map(|header| (header, Default::default()))
			.collect::<Vec<_>>();

		self.ethpubsub_notifier.notify_heads(&headers);
		self.ethpubsub_notifier.notify_logs(&enacted.iter().map(|h| (*h, ())).collect::<Vec<_>>(), |filter, _| self.client.logs(filter));

		// Update sync status
		let sync_status = {
			if self.sync.is_major_importing() {
				let chain_info = self.client.client.chain_info();
				let current_block = U256::from(chain_info.best_block_number);
				let highest_block = self.sync.highest_block().map(U256::from)
					.unwrap_or_else(|| current_block);

				SyncStatus::Info(SyncInfo {
					starting_block: U256::from(self.sync.start_block()).into(),
					current_block: current_block.into(),
					highest_block: highest_block.into(),
					warp_chunks_amount: None,
					warp_chunks_processed: None,
				})
			} else {
				SyncStatus::None
			}
		};

		self.ethpubsub_notifier.notify_syncing(sync_status);
	}
}

impl<CNH> EthPubSub for EthPubSubClient<CNH>
where
	CNH: Send + Sync + 'static
	{
	type Metadata = Metadata;

	fn subscribe(
		&self,
		_meta: Metadata,
		subscriber: Subscriber<pubsub::Result>,
		kind: pubsub::Kind,
		params: Option<pubsub::Params>,
	) {
		let error = match (kind, params.into()) {
			(pubsub::Kind::NewHeads, None) => {
				self.ethpubsub_notifier.heads_subscribers.write().push(subscriber);
				return;
			},
			(pubsub::Kind::NewHeads, _) => {
				errors::invalid_params("newHeads", "Expected no parameters.")
			},
			(pubsub::Kind::Logs, Some(pubsub::Params::Logs(filter))) => {
				match filter.try_into() {
					Ok(filter) => {
						self.ethpubsub_notifier.logs_subscribers.write().push(subscriber, filter);
						return;
					},
					Err(err) => err,
				}
			},
			(pubsub::Kind::Logs, _) => {
				errors::invalid_params("logs", "Expected a filter object.")
			},
			(pubsub::Kind::NewPendingTransactions, None) => {
				self.ethpubsub_notifier.transactions_subscribers.write().push(subscriber);
				return;
			},
			(pubsub::Kind::NewPendingTransactions, _) => {
				errors::invalid_params("newPendingTransactions", "Expected no parameters.")
			},
			(pubsub::Kind::Syncing, None) => {
				self.ethpubsub_notifier.syncing_subscribers.write().push(subscriber);
				return;
			},
			_ => {
				errors::unimplemented(None)
			},
		};

		let _ = subscriber.reject(error);
	}

	fn unsubscribe(&self, _: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool> {
		let res = self.ethpubsub_notifier.heads_subscribers.write().remove(&id).is_some();
		let res2 = self.ethpubsub_notifier.syncing_subscribers.write().remove(&id).is_some();
		let res3 = self.ethpubsub_notifier.logs_subscribers.write().remove(&id).is_some();
		let res4 = self.ethpubsub_notifier.transactions_subscribers.write().remove(&id).is_some();

		Ok(res || res2 || res3 || res4)
	}
}
