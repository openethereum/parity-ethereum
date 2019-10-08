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

//! A map of subscribers.

use std::{ops, str};
use std::collections::HashMap;
use rand::RngCore;
use jsonrpc_pubsub::{typed::{Subscriber, Sink}, SubscriptionId};
use ethereum_types::H64;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Id(H64);
impl str::FromStr for Id {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.starts_with("0x") {
			Ok(Id(s[2..].parse().map_err(|e| format!("{}", e))?))
		} else {
			Err("The id must start with 0x".into())
		}
	}
}
impl Id {
	// TODO: replace `format!` see [#10412](https://github.com/paritytech/parity-ethereum/issues/10412)
	pub fn as_string(&self) -> String {
		format!("{:?}", self.0)
	}
}

#[cfg(not(test))]
mod random {
	use rand;

	pub type Rng = rand::rngs::OsRng;

	pub fn new() -> Rng { Rng::new().expect("Valid random source is required.") }
}

#[cfg(test)]
mod random {
	use rand::SeedableRng;
	use rand_xorshift::XorShiftRng;

	const RNG_SEED: [u8; 16] = [0u8; 16];

	pub type Rng = XorShiftRng;

	pub fn new() -> Rng { Rng::from_seed(RNG_SEED) }
}

pub struct Subscribers<T> {
	rand: random::Rng,
	subscriptions: HashMap<Id, T>,
}

impl<T> Default for Subscribers<T> {
	fn default() -> Self {
		Subscribers {
			rand: random::new(),
			subscriptions: HashMap::new(),
		}
	}
}

impl<T> Subscribers<T> {
	fn next_id(&mut self) -> Id {
		let mut data = H64::default();
		// TODO [grbIzl] rework with proper H64::random_using with rand 0.7
		self.rand.fill_bytes(&mut data.as_bytes_mut());
		Id(data)
	}

	/// Insert new subscription and return assigned id.
	pub fn insert(&mut self, val: T) -> SubscriptionId {
		let id = self.next_id();
		debug!(target: "pubsub", "Adding subscription id={:?}", id);
		let s = id.as_string();
		self.subscriptions.insert(id, val);
		SubscriptionId::String(s)
	}

	/// Removes subscription with given id and returns it (if any).
	pub fn remove(&mut self, id: &SubscriptionId) -> Option<T> {
		trace!(target: "pubsub", "Removing subscription id={:?}", id);
		match *id {
			SubscriptionId::String(ref id) => match id.parse() {
				Ok(id) => self.subscriptions.remove(&id),
				Err(_) => None,
			},
			_ => None,
		}
	}
}

impl<T> Subscribers<Sink<T>> {
	/// Assigns id and adds a subscriber to the list.
	pub fn push(&mut self, sub: Subscriber<T>) {
		let id = self.next_id();
		if let Ok(sink) = sub.assign_id(SubscriptionId::String(id.as_string())) {
			debug!(target: "pubsub", "Adding subscription id={:?}", id);
			self.subscriptions.insert(id, sink);
		}
	}
}

impl<T, V> Subscribers<(Sink<T>, V)> {
	/// Assigns id and adds a subscriber to the list.
	pub fn push(&mut self, sub: Subscriber<T>, val: V) {
		let id = self.next_id();
		if let Ok(sink) = sub.assign_id(SubscriptionId::String(id.as_string())) {
			debug!(target: "pubsub", "Adding subscription id={:?}", id);
			self.subscriptions.insert(id, (sink, val));
		}
	}
}

impl<T> ops::Deref for Subscribers<T> {
	type Target = HashMap<Id, T>;

	fn deref(&self) -> &Self::Target {
		&self.subscriptions
	}
}
