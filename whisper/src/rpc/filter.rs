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

//! Abstraction over filters which works with polling and subscription.

use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::thread;

use bigint::hash::{H32, H256, H512};
use ethkey::Public;
use jsonrpc_macros::pubsub::Sink;
use parking_lot::{Mutex, RwLock};

use message::{self, Message, Topic};
use super::key_store::KeyStore;
use super::types::{self, FilterItem, HexEncode};

/// Kinds of filters,
#[derive(PartialEq, Eq)]
pub enum Kind {
	/// Polled filter only returns data upon request
	Poll,
	/// Subscription filter pushes data to subscriber immediately.
	Subscription,
}

pub type ItemBuffer = Arc<Mutex<Vec<FilterItem>>>;

enum FilterEntry {
	Poll(Arc<Filter>, ItemBuffer),
	Subscription(Arc<Filter>, Sink<FilterItem>),
}

/// Filter manager. Handles filters as well as a thread for
pub struct Manager {
	key_store: Arc<RwLock<KeyStore>>,
	filters: RwLock<HashMap<H256, FilterEntry>>,
	tx: Mutex<mpsc::Sender<Box<Fn() + Send>>>,
	join: Option<thread::JoinHandle<()>>,
}

impl Manager {
	/// Create a new filter manager that will dispatch decryption tasks onto
	/// the given thread pool and use given key store for key management.
	pub fn new(key_store: Arc<RwLock<KeyStore>>) ->
		::std::io::Result<Self>
	{
		let (tx, rx) = mpsc::channel::<Box<Fn() + Send>>();
		let join_handle = thread::Builder::new()
			.name("Whisper Decryption Worker".to_string())
			.spawn(move || for item in rx { (item)() })?;

		Ok(Manager {
			key_store: key_store,
			filters: RwLock::new(HashMap::new()),
			tx: Mutex::new(tx),
			join: Some(join_handle),
		})
	}

	/// Get filter kind if it's known.
	pub fn kind(&self, id: &H256) -> Option<Kind> {
		self.filters.read().get(id).map(|filter| match *filter {
			FilterEntry::Poll(_, _) => Kind::Poll,
			FilterEntry::Subscription(_, _) => Kind::Subscription,
		})
	}

	/// Remove filter by ID.
	pub fn remove(&self, id: &H256) {
		self.filters.write().remove(id);
	}

	/// Add a new polled filter.
	pub fn insert_polled(&self, id: H256, filter: Filter) -> ItemBuffer {
		let buffer = Arc::new(Mutex::new(Vec::new()));
		let entry = FilterEntry::Poll(Arc::new(filter), buffer.clone());

		self.filters.write().insert(id, entry);

		buffer
	}

	/// Add a new subscription filter.
	pub fn insert_subscription(&self, id: H256, filter: Filter, sink: Sink<FilterItem>) {
		let entry = FilterEntry::Subscription(Arc::new(filter), sink);
		self.filters.write().insert(id, entry);
	}
}

// machinery for attaching the manager to the network instance.
impl ::net::MessageHandler for Arc<Manager> {
	fn handle_messages(&self, messages: &[Message]) {
		let filters = self.filters.read();
		let filters_iter = filters
			.values()
			.flat_map(|filter| messages.iter().map(move |msg| (filter, msg))) ;

		for	(filter, message) in filters_iter {
			// if the message matches any of the possible bloom filters,
			// send to thread pool to attempt decryption and avoid
			// blocking the network thread for long.
			let failed_send = match *filter {
				FilterEntry::Poll(ref filter, _) | FilterEntry::Subscription(ref filter, _)
					if !filter.bloom_matches(message) => None,
				FilterEntry::Poll(ref filter, ref buffer) => {
					let (message, key_store) = (message.clone(), self.key_store.clone());
					let (filter, buffer) = (filter.clone(), buffer.clone());

					self.tx.lock().send(Box::new(move || {
						filter.handle_message(
							&message,
							&*key_store,
							|matched| buffer.lock().push(matched),
						)
					})).err().map(|x| x.0)
				}
				FilterEntry::Subscription(ref filter, ref sink) => {
					let (message, key_store) = (message.clone(), self.key_store.clone());
					let (filter, sink) = (filter.clone(), sink.clone());

					self.tx.lock().send(Box::new(move || {
						filter.handle_message(
							&message,
							&*key_store,
							|matched| { let _ = sink.notify(Ok(matched)); },
						)
					})).err().map(|x| x.0)
				}
			};

			// if we failed to send work, no option but to do it locally.
			if let Some(local_work) = failed_send {
				(local_work)()
			}
		}
	}
}

impl Drop for Manager {
	fn drop(&mut self) {
		if let Some(guard) = self.join.take() {
			let _ = guard.join();
		}
	}
}

/// Filter incoming messages by critera.
pub struct Filter {
	topics: Vec<(Vec<u8>, H512, Topic)>,
	from: Option<Public>,
	decrypt_with: H256,
}

impl Filter {
	/// Create a new filter from filter request.
	///
	/// Fails if the topics vector is empty.
	pub fn new(params: types::FilterRequest) -> Result<Self, &'static str> {
		if params.topics.is_empty() {
			return Err("no topics for filter");
		}

		let topics: Vec<_> = params.topics.into_iter()
			.map(|x| x.into_inner())
			.map(|topic| {
				let abridged = super::abridge_topic(&topic);
				(topic, abridged.bloom(), abridged)
			})
			.collect();

		Ok(Filter {
			topics: topics,
			from: params.from.map(|x| x.into_inner()),
			decrypt_with: params.decrypt_with.into_inner(),
		})
	}

	// whether the given message matches at least one of the topics of the
	// filter.
	fn bloom_matches(&self, message: &Message) -> bool {
		self.topics.iter().any(|&(_, ref bloom, _)| {
			&(bloom & message.bloom()) == bloom
		})
	}

	// handle a message that matches the bloom.
	fn handle_message<F: Fn(FilterItem) + Send + Sync>(
		&self,
		message: &Message,
		store: &RwLock<KeyStore>,
		on_match: F,
	) {
		let matched_topics: Vec<_> = self.topics.iter()
			.filter_map(|&(_, ref bloom, ref abridged)| {
				let contains_topic = &(bloom & message.bloom()) == bloom
					&& message.topics().contains(abridged);

				if contains_topic { Some(HexEncode(H32(abridged.0))) } else { None }
			})
			.collect();

		if matched_topics.is_empty() { return }
		let decrypt = match store.read().decryption_instance(&self.decrypt_with) {
			Some(d) => d,
			None => {
				warn!(target: "whisper", "Filter attempted to decrypt with destroyed identity {}",
					self.decrypt_with);

				return
			}
		};

		let decrypted = match decrypt.decrypt(message.data()) {
			Some(d) => d,
			None => {
				trace!(target: "whisper", "Failed to decrypt message with {} matching topics",
					matched_topics.len());

				return
			}
		};

		match ::rpc::payload::decode(&decrypted) {
			Ok(decoded) => {
				if decoded.from != self.from { return }

				on_match(FilterItem {
					from: decoded.from.map(HexEncode),
					recipient: HexEncode(self.decrypt_with),
					ttl: message.envelope().ttl,
					topics: matched_topics,
					timestamp: message.envelope().expiry - message.envelope().ttl,
					payload: HexEncode(decoded.message.to_vec()),
					padding: decoded.padding.map(|pad| HexEncode(pad.to_vec())),
				})
			}
			Err(reason) =>
				trace!(target: "whisper", "Bad payload in decrypted message with {} topics: {}",
					matched_topics.len(), reason),
		}
	}
}
