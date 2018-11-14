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

//! Parity-specific PUB-SUB rpc implementation.

use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;

use jsonrpc_core::{self as core, Result, MetaIoHandler};
use jsonrpc_core::futures::{future, Future, Stream, Sink};
use jsonrpc_macros::Trailing;
use jsonrpc_macros::pubsub::Subscriber;
use jsonrpc_pubsub::SubscriptionId;
use tokio_timer;

use parity_runtime::Executor;
use v1::helpers::GenericPollManager;
use v1::metadata::Metadata;
use v1::traits::PubSub;

/// Parity PubSub implementation.
pub struct PubSubClient<S: core::Middleware<Metadata>> {
	poll_manager: Arc<RwLock<GenericPollManager<S>>>,
	executor: Executor,
}

impl<S: core::Middleware<Metadata>> PubSubClient<S> {
	/// Creates new `PubSubClient`.
	pub fn new(rpc: MetaIoHandler<Metadata, S>, executor: Executor) -> Self {
		let poll_manager = Arc::new(RwLock::new(GenericPollManager::new(rpc)));
		let pm2 = Arc::downgrade(&poll_manager);

		let timer = tokio_timer::wheel()
			.tick_duration(Duration::from_millis(500))
			.build();

		// Start ticking
		let interval = timer.interval(Duration::from_millis(1000));
		executor.spawn(interval
			.map_err(|e| warn!("Polling timer error: {:?}", e))
			.for_each(move |_| {
				if let Some(pm2) = pm2.upgrade() {
					pm2.read().tick()
				} else {
					Box::new(future::err(()))
				}
			})
		);

		PubSubClient {
			poll_manager,
			executor,
		}
	}
}

impl PubSubClient<core::NoopMiddleware> {
	/// Creates new `PubSubClient` with deterministic ids.
	#[cfg(test)]
	pub fn new_test(rpc: MetaIoHandler<Metadata, core::NoopMiddleware>, executor: Executor) -> Self {
		let client = Self::new(MetaIoHandler::with_middleware(Default::default()), executor);
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
				self.executor.spawn(receiver.forward(sink.sink_map_err(|e| {
					warn!("Cannot send notification: {:?}", e);
				})).map(|_| ()));
			},
			Err(_) => {
				poll_manager.unsubscribe(&id);
			},
		}
	}

	fn parity_unsubscribe(&self, id: SubscriptionId) -> Result<bool> {
		let res = self.poll_manager.write().unsubscribe(&id);
		Ok(res)
	}
}
