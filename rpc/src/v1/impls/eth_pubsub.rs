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

use futures::{self, BoxFuture, Future};
use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use jsonrpc_macros::pubsub::{Sink, Subscriber};
use jsonrpc_pubsub::SubscriptionId;

use v1::helpers::{errors, Subscribers};
use v1::metadata::Metadata;
use v1::traits::EthPubSub;
use v1::types::{pubsub, RichHeader};

use ethcore::encoded;
use ethcore::client::{BlockChainClient, ChainNotify, BlockId};
use light::client::{LightChainClient, LightChainNotify};
use parity_reactor::Remote;
use util::{Mutex, H256, Bytes};

/// Eth PubSub implementation.
pub struct EthPubSubClient<C> {
	handler: Arc<ChainNotificationHandler<C>>,
	heads_subscribers: Arc<Mutex<Subscribers<Sink<pubsub::Result>>>>,
}

impl<C> EthPubSubClient<C> {
	/// Creates new `EthPubSubClient`.
	pub fn new(client: Arc<C>, remote: Remote) -> Self {
		let heads_subscribers = Arc::new(Mutex::new(Subscribers::default()));
		EthPubSubClient {
			handler: Arc::new(ChainNotificationHandler {
				client: client,
				remote: remote,
				heads_subscribers: heads_subscribers.clone(),
			}),
			heads_subscribers: heads_subscribers,
		}
	}

	/// Returns a chain notification handler.
	pub fn handler(&self) -> Arc<ChainNotificationHandler<C>> {
		self.handler.clone()
	}
}

/// PubSub Notification handler.
pub struct ChainNotificationHandler<C> {
	client: Arc<C>,
	remote: Remote,
	heads_subscribers: Arc<Mutex<Subscribers<Sink<pubsub::Result>>>>,
}

impl<C> ChainNotificationHandler<C> {
	fn notify(&self, blocks: Vec<(encoded::Header, BTreeMap<String, String>)>) {
		for subscriber in self.heads_subscribers.lock().values() {
			for &(ref block, ref extra_info) in &blocks {
				self.remote.spawn(subscriber
					.notify(Ok(pubsub::Result::Header(RichHeader {
						inner: block.into(),
						extra_info: extra_info.clone(),
					})))
					.map(|_| ())
					.map_err(|e| warn!(target: "rpc", "Unable to send notification: {}", e))
				);
			}
		}
	}
}

impl<C: LightChainClient> LightChainNotify for ChainNotificationHandler<C> {
	fn new_headers(
		&self,
		headers: &[H256],
	) {
		let blocks = headers
			.iter()
			.filter_map(|hash| self.client.block_header(BlockId::Hash(*hash)))
			.map(|header| (header, Default::default()))
			.collect();

		self.notify(blocks);
	}
}

impl<C: BlockChainClient> ChainNotify for ChainNotificationHandler<C> {
	fn new_blocks(
		&self,
		_imported: Vec<H256>,
		_invalid: Vec<H256>,
		enacted: Vec<H256>,
		_retracted: Vec<H256>,
		_sealed: Vec<H256>,
		// Block bytes.
		_proposed: Vec<Bytes>,
		_duration: u64,
	) {
		const EXTRA_INFO_PROOF: &'static str = "Object exists in in blockchain (fetched earlier), extra_info is always available if object exists; qed";
		let blocks = enacted
			.into_iter()
			.filter_map(|hash| self.client.block_header(BlockId::Hash(hash)))
			.map(|header| {
				let hash = header.hash();
				(header, self.client.block_extra_info(BlockId::Hash(hash)).expect(EXTRA_INFO_PROOF))
			})
			.collect();
		self.notify(blocks);
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
		match (kind, params.0) {
			(pubsub::Kind::NewHeads, pubsub::Params::None) => {
				self.heads_subscribers.lock().push(subscriber)
			},
			_ => {
				let _ = subscriber.reject(errors::unimplemented(None));
			},
		}
	}

	fn unsubscribe(&self, id: SubscriptionId) -> BoxFuture<bool, Error> {
		let res = self.heads_subscribers.lock().remove(&id).is_some();
		futures::future::ok(res).boxed()
	}
}
