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

//! Generic poll manager for Pub-Sub.

use std::sync::Arc;
use std::collections::HashMap;
use util::Mutex;

use jsonrpc_core::futures::{Sink, Future, BoxFuture};
use jsonrpc_core::futures::future::{self, Either};
use jsonrpc_core::futures::sync::mpsc;
use jsonrpc_core::{self as core, MetaIoHandler};

use v1::metadata::Metadata;

#[derive(Debug)]
struct Subscription {
	metadata: Metadata,
	method: String,
	params: core::Params,
	sink: mpsc::Sender<core::Output>,
	last_result: Arc<Mutex<Option<core::Output>>>,
}

/// A struct managing all subscriptions.
/// TODO [ToDr] Depending on the method decide on poll interval.
/// For most of the methods it will be enough to poll on new block instead of time-interval.
pub struct GenericPollManager<S: core::Middleware<Metadata>> {
	next_id: usize,
	poll_subscriptions: HashMap<usize, Subscription>,
	rpc: MetaIoHandler<Metadata, S>,
}

impl<S: core::Middleware<Metadata>> GenericPollManager<S> {
	/// Creates new poll manager
	pub fn new(rpc: MetaIoHandler<Metadata, S>) -> Self {
		GenericPollManager {
			next_id: 1,
			poll_subscriptions: Default::default(),
			rpc: rpc,
		}
	}

	/// Subscribes to update from polling given method.
	pub fn subscribe(&mut self, metadata: Metadata, method: String, params: core::Params)
		-> (usize, mpsc::Receiver<core::Output>)
	{
		let id = self.next_id;
		self.next_id += 1;

		let (sink, stream) = mpsc::channel(1);

		let subscription = Subscription {
			metadata: metadata,
			method: method,
			params: params,
			sink: sink,
			last_result: Default::default(),
		};

		debug!(target: "pubsub", "Adding subscription id={:?}, {:?}", id, subscription);
		self.poll_subscriptions.insert(id, subscription);
		(id, stream)
	}

	pub fn unsubscribe(&mut self, id: usize) -> bool {
		debug!(target: "pubsub", "Removing subscription: {:?}", id);
		self.poll_subscriptions.remove(&id).is_some()
	}

	pub fn tick(&self) -> BoxFuture<(), ()> {
		let mut futures = Vec::new();
		// poll all subscriptions
		for (id, subscription) in self.poll_subscriptions.iter() {
			let call = core::MethodCall {
				jsonrpc: Some(core::Version::V2),
				id: core::Id::Num(*id as u64),
				method: subscription.method.clone(),
				params: Some(subscription.params.clone()),
			};
			trace!(target: "pubsub", "Polling method: {:?}", call);
			let result = self.rpc.handle_call(call.into(), subscription.metadata.clone());

			let last_result = subscription.last_result.clone();
			let sender = subscription.sink.clone();

			let result = result.and_then(move |response| {
				let mut last_result = last_result.lock();
				if *last_result != response && response.is_some() {
					let output = response.expect("Existence proved by the condition.");
					*last_result = Some(output.clone());
					debug!(target: "pubsub", "Got new response, sending: {:?}", output);
					Either::A(sender.send(output).map(|_| ()).map_err(|_| ()))
				} else {
					trace!(target: "pubsub", "Response was not changed: {:?}", response);
					Either::B(future::ok(()))
				}
			});

			futures.push(result)
		}

		// return a future represeting all polls
		future::join_all(futures).map(|_| ()).boxed()
	}
}
