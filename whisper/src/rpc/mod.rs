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

//! JSONRPC interface for Whisper.
//!
//! Manages standard message format decoding, ephemeral identities, signing,
//! encryption, and decryption.
//!
//! Provides an interface for using whisper to transmit data securely.

use std::sync::Arc;

use jsonrpc_core::{Error, ErrorCode, Metadata};
use jsonrpc_derive::rpc;
use jsonrpc_pubsub::{Session, PubSubMetadata, SubscriptionId, typed::Subscriber};

use ethereum_types::H256;
use memzero::Memzero;
use parking_lot::RwLock;

use self::filter::Filter;
use self::key_store::{Key, KeyStore};
use self::types::HexEncode;

use message::{CreateParams, Message, Topic};

mod crypto;
mod filter;
mod key_store;
mod payload;
mod types;

pub use self::filter::Manager as FilterManager;

// create whisper RPC error.
fn whisper_error<T: Into<String>>(message: T) -> Error {
	const ERROR_CODE: i64 = -32085;

	Error {
		code: ErrorCode::ServerError(ERROR_CODE),
		message: message.into(),
		data: None,
	}
}

fn topic_hash(topic: &[u8]) -> H256 {
	H256(::tiny_keccak::keccak256(topic))
}

// abridge topic using first four bytes of hash.
fn abridge_topic(topic: &[u8]) -> Topic {
	let mut abridged = [0; 4];
	let hash = topic_hash(topic).0;
	abridged.copy_from_slice(&hash[..4]);
	abridged.into()
}

/// Whisper RPC interface.
#[rpc]
pub trait Whisper {
	/// Info about the node.
	#[rpc(name = "shh_info")]
	fn info(&self) -> Result<types::NodeInfo, Error>;

	/// Generate a new asymmetric key pair and return an identity.
	#[rpc(name = "shh_newKeyPair")]
	fn new_key_pair(&self) -> Result<types::Identity, Error>;

	/// Import the given SECP2561k private key and return an identity.
	#[rpc(name = "shh_addPrivateKey")]
	fn add_private_key(&self, types::Private) -> Result<types::Identity, Error>;

	/// Generate a new symmetric key and return an identity.
	#[rpc(name = "shh_newSymKey")]
	fn new_sym_key(&self) -> Result<types::Identity, Error>;

	/// Import the given symmetric key and return an identity.
	#[rpc(name = "shh_addSymKey")]
	fn add_sym_key(&self, types::Symmetric) -> Result<types::Identity, Error>;

	/// Get public key. Succeeds if identity is stored and asymmetric.
	#[rpc(name = "shh_getPublicKey")]
	fn get_public(&self, types::Identity) -> Result<types::Public, Error>;

	/// Get private key. Succeeds if identity is stored and asymmetric.
	#[rpc(name = "shh_getPrivateKey")]
	fn get_private(&self, types::Identity) -> Result<types::Private, Error>;

	#[rpc(name = "shh_getSymKey")]
	fn get_symmetric(&self, types::Identity) -> Result<types::Symmetric, Error>;

	/// Delete key pair denoted by given identity.
	///
	/// Return true if successfully removed, false if unknown,
	/// and error otherwise.
	#[rpc(name = "shh_deleteKey")]
	fn remove_key(&self, types::Identity) -> Result<bool, Error>;

	/// Post a message to the network with given parameters.
	#[rpc(name = "shh_post")]
	fn post(&self, types::PostRequest) -> Result<bool, Error>;

	/// Create a new polled filter.
	#[rpc(name = "shh_newMessageFilter")]
	fn new_filter(&self, types::FilterRequest) -> Result<types::Identity, Error>;

	/// Poll changes on a polled filter.
	#[rpc(name = "shh_getFilterMessages")]
	fn poll_changes(&self, types::Identity) -> Result<Vec<types::FilterItem>, Error>;

	/// Delete polled filter. Return bool indicating success.
	#[rpc(name = "shh_deleteMessageFilter")]
	fn delete_filter(&self, types::Identity) -> Result<bool, Error>;
}

/// Whisper RPC pubsub.
#[rpc]
pub trait WhisperPubSub {
	// RPC Metadata
	type Metadata;

	/// Subscribe to messages matching the filter.
	#[pubsub(subscription = "shh_subscription", subscribe, name = "shh_subscribe")]
	fn subscribe(&self, Self::Metadata, Subscriber<types::FilterItem>, types::FilterRequest);

	/// Unsubscribe from filter matching given ID. Return
	/// true on success, error otherwise.
	#[pubsub(subscription = "shh_subscription", unsubscribe, name = "shh_unsubscribe")]
	fn unsubscribe(&self, Option<Self::Metadata>, SubscriptionId) -> Result<bool, Error>;
}

/// Something which can send messages to the network.
pub trait PoolHandle: Send + Sync {
	/// Give message to the whisper network for relay.
	/// Returns false if PoW too low.
	fn relay(&self, message: Message) -> bool;

	/// Number of messages and memory used by resident messages.
	fn pool_status(&self) -> ::net::PoolStatus;
}

/// Default, simple metadata implementation.
#[derive(Clone, Default)]
pub struct Meta {
	session: Option<Arc<Session>>,
}

impl Metadata for Meta {}
impl PubSubMetadata for Meta {
	fn session(&self) -> Option<Arc<Session>> {
		self.session.clone()
	}
}

/// Implementation of whisper RPC.
pub struct WhisperClient<P, M = Meta> {
	store: Arc<RwLock<KeyStore>>,
	pool: P,
	filter_manager: Arc<filter::Manager>,
	_meta: ::std::marker::PhantomData<M>,
}

impl<P> WhisperClient<P> {
	/// Create a new whisper client with basic metadata.
	pub fn with_simple_meta(pool: P, filter_manager: Arc<filter::Manager>) -> Self {
		WhisperClient::new(pool, filter_manager)
	}
}

impl<P, M> WhisperClient<P, M> {
	/// Create a new whisper client.
	pub fn new(pool: P, filter_manager: Arc<filter::Manager>) -> Self {
		WhisperClient {
			store: filter_manager.key_store(),
			pool: pool,
			filter_manager: filter_manager,
			_meta: ::std::marker::PhantomData,
		}
	}

	fn delete_filter_kind(&self, id: H256, kind: filter::Kind) -> bool {
		match self.filter_manager.kind(&id) {
			Some(k) if k == kind => {
				self.filter_manager.remove(&id);
				true
			}
			None | Some(_) => false,
		}
	}
}

impl<P: PoolHandle + 'static, M: Send + Sync + 'static> Whisper for WhisperClient<P, M> {
	fn info(&self) -> Result<types::NodeInfo, Error> {
		let status = self.pool.pool_status();

		Ok(types::NodeInfo {
			required_pow: status.required_pow,
			messages: status.message_count,
			memory: status.cumulative_size,
			target_memory: status.target_size,
		})
	}

	fn new_key_pair(&self) -> Result<types::Identity, Error> {
		let mut store = self.store.write();
		let key_pair = Key::new_asymmetric(store.rng());

		Ok(HexEncode(store.insert(key_pair)))
	}

	fn add_private_key(&self, private: types::Private) -> Result<types::Identity, Error> {
		let key_pair = Key::from_secret(private.into_inner().into())
			.ok_or_else(|| whisper_error("Invalid private key"))?;

		Ok(HexEncode(self.store.write().insert(key_pair)))
	}

	fn new_sym_key(&self) -> Result<types::Identity, Error> {
		let mut store = self.store.write();
		let key = Key::new_symmetric(store.rng());

		Ok(HexEncode(store.insert(key)))
	}

	fn add_sym_key(&self, raw_key: types::Symmetric) -> Result<types::Identity, Error> {
		let raw_key = raw_key.into_inner().0;
		let key = Key::from_raw_symmetric(raw_key);

		Ok(HexEncode(self.store.write().insert(key)))
	}

	fn get_public(&self, id: types::Identity) -> Result<types::Public, Error> {
		self.store.read().public(&id.into_inner())
			.cloned()
			.map(HexEncode)
			.ok_or_else(|| whisper_error("Unknown identity"))
	}

	fn get_private(&self, id: types::Identity) -> Result<types::Private, Error> {
		self.store.read().secret(&id.into_inner())
			.map(|x| (&**x).clone())
			.map(HexEncode)
			.ok_or_else(|| whisper_error("Unknown identity"))
	}

	fn get_symmetric(&self, id: types::Identity) -> Result<types::Symmetric, Error> {
		self.store.read().symmetric(&id.into_inner())
			.cloned()
			.map(H256)
			.map(HexEncode)
			.ok_or_else(|| whisper_error("Unknown identity"))
	}

	fn remove_key(&self, id: types::Identity) -> Result<bool, Error> {
		Ok(self.store.write().remove(&id.into_inner()))
	}

	fn post(&self, req: types::PostRequest) -> Result<bool, Error> {
		use self::crypto::EncryptionInstance;

		let encryption = match req.to {
			Some(types::Receiver::Public(public)) => EncryptionInstance::ecies(public.into_inner())
				.map_err(whisper_error)?,
			Some(types::Receiver::Identity(id)) => self.store.read().encryption_instance(&id.into_inner())
				.map_err(whisper_error)?,
			None => {
				use rand::{Rng, OsRng};

				// broadcast mode: use fixed nonce and fresh key each time.

				let mut rng = OsRng::new()
					.map_err(|_| whisper_error("unable to acquire secure randomness"))?;

				let key = Memzero::from(rng.gen::<[u8; 32]>());
				if req.topics.is_empty() {
					return Err(whisper_error("must supply at least one topic for broadcast message"));
				}

				EncryptionInstance::broadcast(
					key,
					req.topics.iter().map(|x| topic_hash(&x)).collect()
				)
			}
		};

		let sign_with = match req.from {
			Some(from) => {
				Some(
					self.store.read().secret(&from.into_inner())
						.cloned()
						.ok_or_else(|| whisper_error("Unknown identity `from`"))?
				)
			}
			None => None,
		};

		let encrypted = {
			let payload = payload::encode(payload::EncodeParams {
				message: &req.payload.into_inner(),
				padding: req.padding.map(|p| p.into_inner()).as_ref().map(|x| &x[..]),
				sign_with: sign_with.as_ref(),
			}).map_err(whisper_error)?;

			encryption.encrypt(&payload).ok_or(whisper_error("encryption error"))?
		};

		// mining the packet is the heaviest item of work by far.
		// there may be a benefit to dispatching this onto the CPU pool
		// and returning a future. but then things get _less_ efficient
		// if the server infrastructure has more threads than the CPU pool.
		let message = Message::create(CreateParams {
			ttl: req.ttl,
			payload: encrypted,
			topics: req.topics.into_iter().map(|x| abridge_topic(&x.into_inner())).collect(),
			work: req.priority,
		}).map_err(|_| whisper_error("Empty topics"))?;

		if !self.pool.relay(message) {
			Err(whisper_error("PoW too low to compete with other messages"))
		} else {
			Ok(true)
		}
	}

	fn new_filter(&self, req: types::FilterRequest) -> Result<types::Identity, Error> {
		let filter = Filter::new(req).map_err(whisper_error)?;

		self.filter_manager.insert_polled(filter)
			.map(HexEncode)
			.map_err(whisper_error)
	}

	fn poll_changes(&self, id: types::Identity) -> Result<Vec<types::FilterItem>, Error> {
		match self.filter_manager.poll_changes(&id.into_inner()) {
			None => Err(whisper_error("no such message filter")),
			Some(items) => Ok(items),
		}
	}

	fn delete_filter(&self, id: types::Identity) -> Result<bool, Error> {
		Ok(self.delete_filter_kind(id.into_inner(), filter::Kind::Poll))
	}
}

impl<P: PoolHandle + 'static, M: Send + Sync + PubSubMetadata> WhisperPubSub for WhisperClient<P, M> {
	type Metadata = M;

	fn subscribe(
		&self,
		_meta: Self::Metadata,
		subscriber: Subscriber<types::FilterItem>,
		req: types::FilterRequest,
	) {
		match Filter::new(req) {
			Ok(filter) => {
				if let Err(e) = self.filter_manager.insert_subscription(filter, subscriber) {
					debug!(target: "whisper", "Failed to add subscription: {}", e);
				}
			}
			Err(reason) => { let _ = subscriber.reject(whisper_error(reason)); }
		}
	}

	fn unsubscribe(&self, _: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool, Error> {
		use std::str::FromStr;

		let res = match id {
			SubscriptionId::String(s) => H256::from_str(&s)
				.map_err(|_| "unrecognized ID")
				.map(|id| self.delete_filter_kind(id, filter::Kind::Subscription)),
			SubscriptionId::Number(_) => Err("unrecognized ID"),
		};

		res.map_err(whisper_error)
	}
}
