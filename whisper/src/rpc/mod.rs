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

//! JSONRPC interface for Whisper.
//!
//! Manages standard message format decoding, ephemeral identities, signing,
//! encryption, and decryption.
//!
//! Provides an interface for using whisper to transmit data securely.

use std::sync::Arc;

use jsonrpc_core::{Error, ErrorCode};
use parking_lot::{Mutex, RwLock};
use rand::{Rng, SeedableRng, XorShiftRng};

use self::filter::Filter;
use self::key_store::{Key, KeyStore};
use self::types::HexEncode;

use message::{CreateParams, Message, Topic};

mod crypto;
mod filter;
mod key_store;
mod payload;
mod types;

// create whisper RPC error.
fn whisper_error<T: Into<String>>(message: T) -> Error {
	const ERROR_CODE: i64 = -32085;

	Error {
		code: ErrorCode::ServerError(ERROR_CODE),
		message: message.into(),
		data: None,
	}
}

// abridge topic using first four bytes of hash.
fn abridge_topic(topic: &[u8]) -> Topic {
	let mut abridged = [0; 4];
	let hash = ::tiny_keccak::keccak256(topic);
	abridged.copy_from_slice(&hash[..4]);
	abridged.into()
}

build_rpc_trait! {
	/// Whisper RPC interface.
	pub trait Whisper {
		/// Generate a new asymmetric key pair and return an identity.
		#[rpc(name = "shh_newKeyPair")]
		fn new_key_pair(&self) -> Result<types::Identity, Error>;

		/// Import the given SECP2561k private key and return an identity.
		#[rpc(name = "shh_addPrivateKey")]
		fn add_private_key(&self, types::Private) -> Result<types::Identity, Error>;

		/// Get public key. Succeeds if identity is stored and asymmetric.
		#[rpc(name = "shh_getPublicKey")]
		fn get_public(&self, types::Identity) -> Result<types::Public, Error>;

		/// Get private key. Succeeds if identity is stored and asymmetric.
		#[rpc(name = "shh_getPrivateKey")]
		fn get_private(&self, types::Identity) -> Result<types::Private, Error>;

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
}

/// Something which can send messages to the network.
pub trait MessageSender: Send + Sync {
	/// Give message to the whisper network for relay.
	fn relay(&self, message: Message);
}

impl MessageSender for ::net::MessagePoster {
	fn relay(&self, message: Message) {
		self.post_message(message)
	}
}

/// Implementation of whisper RPC.
pub struct WhisperClient<S> {
	store: RwLock<key_store::KeyStore>,
	sender: S,
	filter_manager: Arc<filter::Manager>,
	filter_ids_rng: Mutex<XorShiftRng>,
}

impl<S> WhisperClient<S> {
	/// Create a new whisper client.
	///
	/// This spawns a thread for handling
	/// asynchronous work like performing PoW on messages or handling
	/// subscriptions.
	pub fn new(sender: S, filter_manager: Arc<filter::Manager>) -> ::std::io::Result<Self> {
		let filter_ids_rng = {
			let mut rng = ::rand::thread_rng();
			XorShiftRng::from_seed(rng.gen())
		};

		Ok(WhisperClient {
			store: RwLock::new(KeyStore::new()?),
			sender: sender,
			filter_manager: filter_manager,
			filter_ids_rng: Mutex::new(filter_ids_rng),
		})
	}
}

impl<S: MessageSender + 'static> Whisper for WhisperClient<S> {
	fn new_key_pair(&self) -> Result<types::Identity, Error> {
		let mut store = self.store.write();
		let key_pair = Key::new_asymmetric(store.rng());

		Ok(HexEncode(store.insert(key_pair)))
	}

	fn add_private_key(&self, private: types::Private) -> Result<types::Identity, Error> {
		let key_pair = Key::from_secret(private.into_inner().into())
			.map_err(|_| whisper_error("Invalid private key"))?;

		Ok(HexEncode(self.store.write().insert(key_pair)))
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

	fn remove_key(&self, id: types::Identity) -> Result<bool, Error> {
		Ok(self.store.write().remove(&id.into_inner()))
	}

	fn post(&self, req: types::PostRequest) -> Result<bool, Error> {
		use self::crypto::EncryptionInstance;

		let encryption = match req.to {
			types::Receiver::Public(public) => EncryptionInstance::ecies(public.into_inner())
				.map_err(whisper_error)?,
			types::Receiver::Identity(id) => self.store.read().encryption_instance(&id.into_inner())
				.map_err(whisper_error)?,
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

			encryption.encrypt(&payload)
		};

		// mining the packet is the heaviest item of work by far.
		// there may be a benefit to dispatching this onto the CPU pool
		// and returning a future. but then things get _less_ efficient
		//
		// if the server infrastructure has more threads than the CPU pool.
		let message = Message::create(CreateParams {
			ttl: req.ttl,
			payload: encrypted,
			topics: req.topics.into_iter().map(|x| abridge_topic(&x.into_inner())).collect(),
			work: req.priority,
		});

		self.sender.relay(message);

		Ok(true)
	}

	fn new_filter(&self, req: types::FilterRequest) -> Result<types::Identity, Error> {
		let filter = Filter::new(req).map_err(whisper_error)?;
		let id = self.filter_ids_rng.lock().gen();

		self.filter_manager.insert_polled(id, filter);
		Ok(HexEncode(id))
	}

	fn poll_changes(&self, id: types::Identity) -> Result<Vec<types::FilterItem>, Error> {
		match self.filter_manager.poll_changes(&id.into_inner()) {
			None => Err(whisper_error("no such message filter")),
			Some(items) => Ok(items),
		}
	}

	fn delete_filter(&self, id: types::Identity) -> Result<bool, Error> {
		let id = id.into_inner();
		Ok(match self.filter_manager.kind(&id) {
			Some(filter::Kind::Poll) => {
				self.filter_manager.remove(&id);
				true
			}
			None | Some(_) => false,
		})
	}
}

// TODO: pub-sub in a way that keeps it easy to integrate with main Parity RPC.
