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
use std::{sync::{Arc, atomic, atomic::AtomicBool, mpsc}, thread};

use ethereum_types::{H256, H512};
use ethkey::Public;
use jsonrpc_macros::pubsub::{Subscriber, Sink};
use parking_lot::{Mutex, RwLock};
use rand::{Rng, OsRng};

use message::{Message, Topic};
use super::{key_store::KeyStore, types::{self, FilterItem, HexEncode}};

/// Kinds of filters,
#[derive(PartialEq, Eq, Clone, Copy)]
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

/// Filter manager. Handles filters as well as a thread for doing decryption
/// and payload decoding.
pub struct Manager {
	key_store: Arc<RwLock<KeyStore>>,
	filters: RwLock<HashMap<H256, FilterEntry>>,
	tx: Mutex<mpsc::Sender<Box<Fn() + Send>>>,
	join: Option<thread::JoinHandle<()>>,
	exit: Arc<AtomicBool>,
}

impl Manager {
	/// Create a new filter manager that will dispatch decryption tasks onto
	/// the given thread pool.
	pub fn new() -> ::std::io::Result<Self> {
		let (tx, rx) = mpsc::channel::<Box<Fn() + Send>>();
		let exit = Arc::new(AtomicBool::new(false));
		let e = exit.clone();

		let join_handle = thread::Builder::new()
			.name("Whisper Decryption Worker".to_string())
			.spawn(move || {
				trace!(target: "parity_whisper", "Start decryption worker");
				loop {
					if exit.load(atomic::Ordering::Acquire) {
						break;
					}
					if let Ok(item) = rx.try_recv() {
						item();
					}
				}
			})?;

		Ok(Manager {
			key_store: Arc::new(RwLock::new(KeyStore::new()?)),
			filters: RwLock::new(HashMap::new()),
			tx: Mutex::new(tx),
			join: Some(join_handle),
			exit: e,
		})
	}

	/// Get a handle to the key store.
	pub fn key_store(&self) -> Arc<RwLock<KeyStore>> {
		self.key_store.clone()
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
	pub fn insert_polled(&self, filter: Filter) -> Result<H256, &'static str> {
		let buffer = Arc::new(Mutex::new(Vec::new()));
		let entry = FilterEntry::Poll(Arc::new(filter), buffer);
		let id = OsRng::new()
			.map_err(|_| "unable to acquire secure randomness")?
			.gen();

		self.filters.write().insert(id, entry);
		Ok(id)
	}

	/// Insert new subscription filter. Generates a secure ID and sends it to
	/// the subscriber
	pub fn insert_subscription(&self, filter: Filter, sub: Subscriber<FilterItem>)
		-> Result<(), &'static str>
	{
		let id: H256 = OsRng::new()
			.map_err(|_| "unable to acquire secure randomness")?
			.gen();

		sub.assign_id(::jsonrpc_pubsub::SubscriptionId::String(format!("{:x}", id)))
			.map(move |sink| {
				let entry = FilterEntry::Subscription(Arc::new(filter), sink);
				self.filters.write().insert(id, entry);
			})
			.map_err(|_| "subscriber disconnected")
	}

	/// Poll changes on filter identified by ID.
	pub fn poll_changes(&self, id: &H256) -> Option<Vec<FilterItem>> {
		self.filters.read().get(id).and_then(|filter| match *filter {
			FilterEntry::Subscription(_, _) => None,
			FilterEntry::Poll(_, ref changes)
				=> Some(::std::mem::replace(&mut *changes.lock(), Vec::new())),
		})
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
					if !filter.basic_matches(message) => None,
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
		trace!(target: "parity_whisper", "waiting to drop FilterManager");
		self.exit.store(true, atomic::Ordering::Release);
		if let Some(guard) = self.join.take() {
			let _ = guard.join();
		}
		trace!(target: "parity_whisper", "FilterManager dropped");
	}
}

/// Filter incoming messages by critera.
pub struct Filter {
	topics: Vec<(Vec<u8>, H512, Topic)>,
	from: Option<Public>,
	decrypt_with: Option<H256>,
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
			decrypt_with: params.decrypt_with.map(|x| x.into_inner()),
		})
	}

	// does basic matching:
	// whether the given message matches at least one of the topics of the
	// filter.
	// TODO: minimum PoW heuristic.
	fn basic_matches(&self, message: &Message) -> bool {
		self.topics.iter().any(|&(_, ref bloom, _)| {
			&(bloom & message.bloom()) == bloom
		})
	}

	// handle a message that matches the bloom.
	fn handle_message<F: Fn(FilterItem)>(
		&self,
		message: &Message,
		store: &RwLock<KeyStore>,
		on_match: F,
	) {
		use rpc::crypto::DecryptionInstance;
		use tiny_keccak::keccak256;

		let matched_indices: Vec<_> = self.topics.iter()
			.enumerate()
			.filter_map(|(i, &(_, ref bloom, ref abridged))| {
				let contains_topic = &(bloom & message.bloom()) == bloom
					&& message.topics().contains(abridged);

				if contains_topic { Some(i) } else { None }
			})
			.collect();

		if matched_indices.is_empty() { return }

		let decrypt = match self.decrypt_with {
			Some(ref id) => match store.read().decryption_instance(id) {
				Some(d) => d,
				None => {
					warn!(target: "whisper", "Filter attempted to decrypt with destroyed identity {}",
						id);

					return
				}
			},
			None => {
				let known_idx = matched_indices[0];
				let known_topic = H256(keccak256(&self.topics[0].0));

				DecryptionInstance::broadcast(message.topics().len(), known_idx, known_topic)
					.expect("known idx is within the range 0..message.topics.len(); qed")
			}
		};

		let decrypted = match decrypt.decrypt(message.data()) {
			Some(d) => d,
			None => {
				trace!(target: "whisper", "Failed to decrypt message with {} matching topics",
					matched_indices.len());

				return
			}
		};

		match ::rpc::payload::decode(&decrypted) {
			Ok(decoded) => {
				if decoded.from != self.from { return }

				let matched_topics = matched_indices
					.into_iter()
					.map(|i| self.topics[i].0.clone())
					.map(HexEncode)
					.collect();

				on_match(FilterItem {
					from: decoded.from.map(HexEncode),
					recipient: self.decrypt_with.map(HexEncode),
					ttl: message.envelope().ttl,
					topics: matched_topics,
					timestamp: message.envelope().expiry - message.envelope().ttl,
					payload: HexEncode(decoded.message.to_vec()),
					padding: decoded.padding.map(|pad| HexEncode(pad.to_vec())),
				})
			}
			Err(reason) =>
				trace!(target: "whisper", "Bad payload in decrypted message with {} topics: {}",
					matched_indices.len(), reason),
		}
	}
}

#[cfg(test)]
mod tests {
	use message::{CreateParams, Message, Topic};
	use rpc::types::{FilterRequest, HexEncode};
	use rpc::abridge_topic;
	use super::*;

	#[test]
	fn rejects_empty_topics() {
		let req = FilterRequest {
			decrypt_with: Default::default(),
			from: None,
			topics: Vec::new(),
		};

		assert!(Filter::new(req).is_err());
	}

	#[test]
	fn basic_match() {
		let topics = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
		let abridged_topics: Vec<_> = topics.iter().map(|x| abridge_topic(&x)).collect();

		let req = FilterRequest {
			decrypt_with: Default::default(),
			from: None,
			topics: topics.into_iter().map(HexEncode).collect(),
		};

		let filter = Filter::new(req).unwrap();
		let message = Message::create(CreateParams {
			ttl: 100,
			payload: vec![1, 3, 5, 7, 9],
			topics: abridged_topics.clone(),
			work: 0,
		}).unwrap();

		assert!(filter.basic_matches(&message));

		let message = Message::create(CreateParams {
			ttl: 100,
			payload: vec![1, 3, 5, 7, 9],
			topics: abridged_topics.clone(),
			work: 0,
		}).unwrap();

		assert!(filter.basic_matches(&message));

		let message = Message::create(CreateParams {
			ttl: 100,
			payload: vec![1, 3, 5, 7, 9],
			topics: vec![Topic([1, 8, 3, 99])],
			work: 0,
		}).unwrap();

		assert!(!filter.basic_matches(&message));
	}

	#[test]
	fn decrypt_and_decode() {
		use rpc::payload::{self, EncodeParams};
		use rpc::key_store::{Key, KeyStore};

		let topics = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
		let abridged_topics: Vec<_> = topics.iter().map(|x| abridge_topic(&x)).collect();

		let mut store = KeyStore::new().unwrap();
		let signing_pair = Key::new_asymmetric(store.rng());
		let encrypting_key = Key::new_symmetric(store.rng());

		let decrypt_id = store.insert(encrypting_key);
		let encryption_instance = store.encryption_instance(&decrypt_id).unwrap();

		let store = ::parking_lot::RwLock::new(store);

		let payload = payload::encode(EncodeParams {
			message: &[1, 2, 3],
			padding: Some(&[4, 5, 4, 5]),
			sign_with: Some(signing_pair.secret().unwrap())
		}).unwrap();

		let encrypted = encryption_instance.encrypt(&payload);

		let message = Message::create(CreateParams {
			ttl: 100,
			payload: encrypted,
			topics: abridged_topics.clone(),
			work: 0,
		}).unwrap();

		let message2 = Message::create(CreateParams {
			ttl: 100,
			payload: vec![3, 5, 7, 9],
			topics: abridged_topics,
			work: 0,
		}).unwrap();

		let filter = Filter::new(FilterRequest {
			decrypt_with: Some(HexEncode(decrypt_id)),
			from: Some(HexEncode(signing_pair.public().unwrap().clone())),
			topics: topics.into_iter().map(HexEncode).collect(),
		}).unwrap();

		assert!(filter.basic_matches(&message));
		assert!(filter.basic_matches(&message2));

		let items = ::std::cell::Cell::new(0);
		let on_match = |_| { items.set(items.get() + 1); };

		filter.handle_message(&message, &store, &on_match);
		filter.handle_message(&message2, &store, &on_match);

		assert_eq!(items.get(), 1);
	}
}
