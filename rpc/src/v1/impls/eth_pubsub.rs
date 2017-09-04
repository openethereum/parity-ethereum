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

//! Eth PUB-SUB rpc implementation.

use std::sync::Arc;
use std::collections::BTreeMap;

use futures::{self, future, BoxFuture, Future};
use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use jsonrpc_macros::pubsub::{Sink, Subscriber};
use jsonrpc_pubsub::SubscriptionId;

use v1::helpers::{errors, limit_logs, Subscribers};
use v1::helpers::light_fetch::LightFetch;
use v1::metadata::Metadata;
use v1::traits::EthPubSub;
use v1::types::{pubsub, RichHeader, Log};

use ethcore::encoded;
use ethcore::filter::Filter as EthFilter;
use ethcore::client::{BlockChainClient, ChainNotify, BlockId};
use ethsync::LightSync;
use light::cache::Cache;
use light::on_demand::OnDemand;
use light::client::{LightChainClient, LightChainNotify};
use parity_reactor::Remote;
use bigint::hash::H256;
use util::Bytes;
use parking_lot::{RwLock, Mutex};

type Client = Sink<pubsub::Result>;

/// Eth PubSub implementation.
pub struct EthPubSubClient<C> {
	handler: Arc<ChainNotificationHandler<C>>,
	heads_subscribers: Arc<RwLock<Subscribers<Client>>>,
	logs_subscribers: Arc<RwLock<Subscribers<(Client, EthFilter)>>>,
}

impl<C> EthPubSubClient<C> {
	/// Creates new `EthPubSubClient`.
	pub fn new(client: Arc<C>, remote: Remote) -> Self {
		let heads_subscribers = Arc::new(RwLock::new(Subscribers::default()));
		let logs_subscribers = Arc::new(RwLock::new(Subscribers::default()));
		EthPubSubClient {
			handler: Arc::new(ChainNotificationHandler {
				client,
				remote,
				heads_subscribers: heads_subscribers.clone(),
				logs_subscribers: logs_subscribers.clone(),
			}),
			heads_subscribers,
			logs_subscribers,
		}
	}

	/// Creates new `EthPubSubCient` with deterministic subscription ids.
	#[cfg(test)]
	pub fn new_test(client: Arc<C>, remote: Remote) -> Self {
		let client = Self::new(client, remote);
		*client.heads_subscribers.write() = Subscribers::new_test();
		*client.logs_subscribers.write() = Subscribers::new_test();
		client
	}

	/// Returns a chain notification handler.
	pub fn handler(&self) -> Arc<ChainNotificationHandler<C>> {
		self.handler.clone()
	}
}

impl EthPubSubClient<LightFetch> {
	/// Creates a new `EthPubSubClient` for `LightClient`.
	pub fn light(
		client: Arc<LightChainClient>,
		on_demand: Arc<OnDemand>,
		sync: Arc<LightSync>,
		cache: Arc<Mutex<Cache>>,
		remote: Remote,
	) -> Self {
		let fetch = LightFetch {
			client,
			on_demand,
			sync,
			cache
		};
		EthPubSubClient::new(Arc::new(fetch), remote)
	}
}

/// PubSub Notification handler.
pub struct ChainNotificationHandler<C> {
	client: Arc<C>,
	remote: Remote,
	heads_subscribers: Arc<RwLock<Subscribers<Client>>>,
	logs_subscribers: Arc<RwLock<Subscribers<(Client, EthFilter)>>>,
}

impl<C> ChainNotificationHandler<C> {
	fn notify(remote: &Remote, subscriber: &Client, result: pubsub::Result) {
		remote.spawn(subscriber
			.notify(Ok(result))
			.map(|_| ())
			.map_err(|e| warn!(target: "rpc", "Unable to send notification: {}", e))
		);
	}

	fn notify_heads(&self, headers: &[(encoded::Header, BTreeMap<String, String>)]) {
		for subscriber in self.heads_subscribers.read().values() {
			for &(ref header, ref extra_info) in headers {
				Self::notify(&self.remote, subscriber, pubsub::Result::Header(RichHeader {
					inner: header.into(),
					extra_info: extra_info.clone(),
				}));
			}
		}
	}

	fn notify_logs<F>(&self, enacted: &[H256], logs: F) where
		F: Fn(EthFilter) -> BoxFuture<Vec<Log>, Error>,
	{
		for &(ref subscriber, ref filter) in self.logs_subscribers.read().values() {
			let logs = futures::future::join_all(enacted
				.iter()
				.map(|hash| {
					let mut filter = filter.clone();
					filter.from_block = BlockId::Hash(*hash);
					filter.to_block = filter.from_block.clone();
					logs(filter)
				})
				.collect::<Vec<_>>()
			);
			let limit = filter.limit;
			let remote = self.remote.clone();
			let subscriber = subscriber.clone();
			self.remote.spawn(logs
				.map(move |logs| {
					let logs = logs.into_iter().flat_map(|log| log).collect();
					let logs = limit_logs(logs, limit);
					if !logs.is_empty() {
						Self::notify(&remote, &subscriber, pubsub::Result::Logs(logs));
					}
				})
				.map_err(|e| warn!("Unable to fetch latest logs: {:?}", e))
			);
		}
	}
}

/// A light client wrapper struct.
pub trait LightClient: Send + Sync {
	/// Get a recent block header.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

	/// Fetch logs.
	fn logs(&self, filter: EthFilter) -> BoxFuture<Vec<Log>, Error>;
}

impl LightClient for LightFetch {
	fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		self.client.block_header(id)
	}

	fn logs(&self, filter: EthFilter) -> BoxFuture<Vec<Log>, Error> {
		LightFetch::logs(self, filter)
	}
}

impl<C: LightClient> LightChainNotify for ChainNotificationHandler<C> {
	fn new_headers(
		&self,
		enacted: &[H256],
	) {
		let headers = enacted
			.iter()
			.filter_map(|hash| self.client.block_header(BlockId::Hash(*hash)))
			.map(|header| (header, Default::default()))
			.collect::<Vec<_>>();

		self.notify_heads(&headers);
		self.notify_logs(&enacted, |filter| self.client.logs(filter))
	}
}

impl<C: BlockChainClient> ChainNotify for ChainNotificationHandler<C> {
	fn new_blocks(
		&self,
		_imported: Vec<H256>,
		_invalid: Vec<H256>,
		enacted: Vec<H256>,
		retracted: Vec<H256>,
		_sealed: Vec<H256>,
		// Block bytes.
		_proposed: Vec<Bytes>,
		_duration: u64,
	) {
		const EXTRA_INFO_PROOF: &'static str = "Object exists in in blockchain (fetched earlier), extra_info is always available if object exists; qed";
		let headers = enacted
			.iter()
			.filter_map(|hash| self.client.block_header(BlockId::Hash(*hash)))
			.map(|header| {
				let hash = header.hash();
				(header, self.client.block_extra_info(BlockId::Hash(hash)).expect(EXTRA_INFO_PROOF))
			})
			.collect::<Vec<_>>();

		// Headers
		self.notify_heads(&headers);

		// Enacted logs
		self.notify_logs(&enacted, |filter| {
			future::ok(self.client.logs(filter).into_iter().map(Into::into).collect()).boxed()
		});

		// Retracted logs
		self.notify_logs(&retracted, |filter| {
			future::ok(self.client.logs(filter).into_iter().map(Into::into).map(|mut log: Log| {
				log.log_type = "removed".into();
				log
			}).collect()).boxed()
		});
	}
}

impl<C: Send + Sync + 'static> EthPubSub for EthPubSubClient<C> {
	type Metadata = Metadata;

	fn subscribe(
		&self,
		_meta: Metadata,
		subscriber: Subscriber<pubsub::Result>,
		kind: pubsub::Kind,
		params: Trailing<pubsub::Params>,
	) {
		let error = match (kind, params.into()) {
			(pubsub::Kind::NewHeads, None) => {
				self.heads_subscribers.write().push(subscriber);
				return;
			},
			(pubsub::Kind::Logs, Some(pubsub::Params::Logs(filter))) => {
				self.logs_subscribers.write().push(subscriber, filter.into());
				return;
			},
			(pubsub::Kind::NewHeads, _) => {
				errors::invalid_params("newHeads", "Expected no parameters.")
			},
			(pubsub::Kind::Logs, _) => {
				errors::invalid_params("logs", "Expected a filter object.")
			},
			_ => {
				errors::unimplemented(None)
			},
		};

		let _ = subscriber.reject(error);
	}

	fn unsubscribe(&self, id: SubscriptionId) -> BoxFuture<bool, Error> {
		let res = self.heads_subscribers.write().remove(&id).is_some();
		let res2 = self.logs_subscribers.write().remove(&id).is_some();

		future::ok(res || res2).boxed()
	}
}
