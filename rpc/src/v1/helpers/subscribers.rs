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

//! A map of subscribers.

use std::ops;
use std::collections::HashMap;
use jsonrpc_macros::pubsub::{Subscriber, Sink, SubscriptionId};

#[derive(Clone, Debug)]
pub struct Subscribers<T> {
	next_id: u64,
	subscriptions: HashMap<u64, T>,
}

impl<T> Default for Subscribers<T> {
	fn default() -> Self {
		Subscribers {
			next_id: 0,
			subscriptions: HashMap::new(),
		}
	}
}

impl<T> Subscribers<T> {
	fn next_id(&mut self) -> u64 {
		self.next_id += 1;
		self.next_id
	}

	/// Insert new subscription and return assigned id.
	pub fn insert(&mut self, val: T) -> SubscriptionId {
		let id = self.next_id();
		debug!(target: "pubsub", "Adding subscription id={}", id);
		self.subscriptions.insert(id, val);
		SubscriptionId::Number(id)
	}

	/// Removes subscription with given id and returns it (if any).
	pub fn remove(&mut self, id: &SubscriptionId) -> Option<T> {
		trace!(target: "pubsub", "Removing subscription id={:?}", id);
		match *id {
			SubscriptionId::Number(id) => {
				self.subscriptions.remove(&id)
			},
			_ => None,
		}
	}
}

impl <T> Subscribers<Sink<T>> {
	/// Assigns id and adds a subscriber to the list.
	pub fn push(&mut self, sub: Subscriber<T>) {
		let id = self.next_id();
		match sub.assign_id(SubscriptionId::Number(id)) {
			Ok(sink) => {
				debug!(target: "pubsub", "Adding subscription id={:?}", id);
				self.subscriptions.insert(id, sink);
			},
			Err(_) => {
				self.next_id -= 1;
			},
		}
	}
}

impl<T> ops::Deref for Subscribers<T> {
	type Target = HashMap<u64, T>;

	fn deref(&self) -> &Self::Target {
		&self.subscriptions
	}
}
