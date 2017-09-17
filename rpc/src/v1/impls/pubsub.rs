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

//! Parity-specific PUB-SUB rpc implementation.

use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;

use futures::{self, BoxFuture, Future, Stream, Sink};
use jsonrpc_core::{self as core, Error, MetaIoHandler};
use jsonrpc_macros::Trailing;
use jsonrpc_macros::pubsub::Subscriber;
use jsonrpc_pubsub::SubscriptionId;
use tokio_timer;

use parity_reactor::Remote;
use v1::helpers::GenericPollManager;
use v1::metadata::Metadata;
use v1::traits::PubSub;

/// Parity PubSub implementation.
pub struct PubSubClient<S: core::Middleware<Metadata>> {
	poll_manager: Arc<RwLock<GenericPollManager<S>>>,
	remote: Remote,
}

impl<S: core::Middleware<Metadata>> PubSubClient<S> {
	/// Creates new `PubSubClient`.
	pub fn new(rpc: MetaIoHandler<Metadata, S>, remote: Remote) -> Self {
		let poll_manager = Arc::new(RwLock::new(GenericPollManager::new(rpc)));
		let pm2 = poll_manager.clone();

		let timer = tokio_timer::wheel()
			.tick_duration(Duration::from_millis(500))
			.build();

		// Start ticking
		let interval = timer.interval(Duration::from_millis(1000));
		remote.spawn(interval
			.map_err(|e| warn!("Polling timer error: {:?}", e))
			.for_each(move |_| pm2.read().tick())
		);

		PubSubClient {
			poll_manager,
			remote,
		}
	}
}

impl PubSubClient<core::NoopMiddleware> {
	/// Creates new `PubSubClient` with deterministic ids.
	#[cfg(test)]
	pub fn new_test(rpc: MetaIoHandler<Metadata, core::NoopMiddleware>, remote: Remote) -> Self {
		let client = Self::new(MetaIoHandler::with_middleware(Default::default()), remote);
		*client.poll_manager.write() = GenericPollManager::new_test(rpc);
		client
	}
}

impl<S: core::Middleware<Metadata>> PubSub for PubSubClient<S> {
	type Metadata = Metadata;

	fn parity_subscribe(&self, mut meta: Metadata, subscriber: Subscriber<core::Value>, method: String, params: Trailing<core::Params>) {
		let params = params.unwrap_or(core::Params::Array(vec![]));
		// Make sure to get rid of PubSub session otherwise it will never be dropped.
		meta.session = None;

		let mut poll_manager = self.poll_manager.write();
		let (id, receiver) = poll_manager.subscribe(meta, method, params);
		match subscriber.assign_id(id.clone()) {
			Ok(sink) => {
				self.remote.spawn(receiver.forward(sink.sink_map_err(|e| {
					warn!("Cannot send notification: {:?}", e);
				})).map(|_| ()));
			},
			Err(_) => {
				poll_manager.unsubscribe(&id);
			},
		}
	}

	fn parity_unsubscribe(&self, id: SubscriptionId) -> BoxFuture<bool, Error> {
		let res = self.poll_manager.write().unsubscribe(&id);
		futures::future::ok(res).boxed()
	}
}
