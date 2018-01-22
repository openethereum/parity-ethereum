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

///! Given: two nodes each holding its own `self_key_pair`.
///!
///! Handshake process:
///! 1) both nodes are generating random `KeyPair` (`session_key_pair`), which will be used for channel encryption
///! 2) both nodes are generating random H256 (`confirmation_plain`)
///! 3) both nodes are signing `confirmation_plain` using `session_key_pair` to receive `confirmation_signed_session`
///! 4) nodes exchange with `NodePublicKey` messages, containing: `self_key_pair.public`, `confirmation_plain`, `confirmation_signed_session`
///! 5) both nodes are checking that they're configured to communicate to server with received `message.self_key_pair.public`. Connection is closed otherwise
///! 6) both nodes are recovering peer' `session_key_pair.public` from `message.confirmation_plain` and `message.confirmation_signed_session`
///! 7) both nodes are computing shared session key pair using self' `session_key_pair.secret` && peer' `session_key_pair.public`. All following messages are encrypted using this key_pair.
///! 8) both nodes are signing `message.confirmation_plain` with their own `self_key_pair.private` to receive `confirmation_signed`
///! 9) nodes exchange with `NodePrivateKeySignature` messages, containing `confirmation_signed`
///! 10) both nodes are checking that `confirmation_signed` is actually signed with the owner of peer' `self_key_pair.secret`
///!
///! Result of handshake is:
///! 1) belief, that we are connected to the KS from our KS-set
///! 2) session key pair, which is used to enrypt all connection messages

use std::io;
use std::sync::Arc;
use std::collections::BTreeSet;
use futures::{Future, Poll, Async};
use tokio_io::{AsyncRead, AsyncWrite};
use ethcrypto::ecdh::agree;
use ethkey::{Random, Generator, KeyPair, Public, Signature, verify_public, sign, recover};
use ethereum_types::H256;
use key_server_cluster::{NodeId, Error, NodeKeyPair};
use key_server_cluster::message::{Message, ClusterMessage, NodePublicKey, NodePrivateKeySignature};
use key_server_cluster::io::{write_message, write_encrypted_message, WriteMessage, ReadMessage,
	read_message, read_encrypted_message, fix_shared_key};

/// Start handshake procedure with another node from the cluster.
pub fn handshake<A>(a: A, self_key_pair: Arc<NodeKeyPair>, trusted_nodes: BTreeSet<NodeId>) -> Handshake<A> where A: AsyncWrite + AsyncRead {
	let init_data = Random.generate().map(|kp| *kp.secret().clone()).map_err(Into::into)
		.and_then(|cp| Random.generate().map(|kp| (cp, kp)).map_err(Into::into));
	handshake_with_init_data(a, init_data, self_key_pair, trusted_nodes)
}

/// Start handshake procedure with another node from the cluster and given plain confirmation + session key pair.
pub fn handshake_with_init_data<A>(a: A, init_data: Result<(H256, KeyPair), Error>, self_key_pair: Arc<NodeKeyPair>, trusted_nodes: BTreeSet<NodeId>) -> Handshake<A> where A: AsyncWrite + AsyncRead {
	let handshake_input_data = init_data
		.and_then(|(cp, kp)| sign(kp.secret(), &cp).map(|sp| (cp, kp, sp)).map_err(Into::into))
		.and_then(|(cp, kp, sp)| Handshake::<A>::make_public_key_message(self_key_pair.public().clone(), cp.clone(), sp).map(|msg| (cp, kp, msg)));

	let (error, cp, kp, state) = match handshake_input_data {
		Ok((cp, kp, msg)) => (None, cp, Some(kp), HandshakeState::SendPublicKey(write_message(a, msg))),
		Err(err) => (Some((a, Err(err))), Default::default(), None, HandshakeState::Finished),
	};

	Handshake {
		is_active: true,
		error: error,
		state: state,
		self_key_pair: self_key_pair,
		self_session_key_pair: kp,
		self_confirmation_plain: cp,
		trusted_nodes: Some(trusted_nodes),
		peer_node_id: None,
		peer_session_public: None,
		peer_confirmation_plain: None,
		shared_key: None,
	}
}

/// Wait for handshake procedure to be started by another node from the cluster.
pub fn accept_handshake<A>(a: A, self_key_pair: Arc<NodeKeyPair>) -> Handshake<A> where A: AsyncWrite + AsyncRead {
	let self_confirmation_plain = Random.generate().map(|kp| *kp.secret().clone()).map_err(Into::into);
	let handshake_input_data = self_confirmation_plain
		.and_then(|cp| Random.generate().map(|kp| (cp, kp)).map_err(Into::into));

	let (error, cp, kp, state) = match handshake_input_data {
		Ok((cp, kp)) => (None, cp, Some(kp), HandshakeState::ReceivePublicKey(read_message(a))),
		Err(err) => (Some((a, Err(err))), Default::default(), None, HandshakeState::Finished),
	};

	Handshake {
		is_active: false,
		error: error,
		state: state,
		self_key_pair: self_key_pair,
		self_session_key_pair: kp,
		self_confirmation_plain: cp,
		trusted_nodes: None,
		peer_node_id: None,
		peer_session_public: None,
		peer_confirmation_plain: None,
		shared_key: None,
	}
}

/// Result of handshake procedure.
#[derive(Debug, PartialEq)]
pub struct HandshakeResult {
	/// Node id.
	pub node_id: NodeId,
	/// Shared key.
	pub shared_key: KeyPair,
}

/// Future handshake procedure.
pub struct Handshake<A> {
	is_active: bool,
	error: Option<(A, Result<HandshakeResult, Error>)>,
	state: HandshakeState<A>,
	self_key_pair: Arc<NodeKeyPair>,
	self_session_key_pair: Option<KeyPair>,
	self_confirmation_plain: H256,
	trusted_nodes: Option<BTreeSet<NodeId>>,
	peer_node_id: Option<NodeId>,
	peer_session_public: Option<Public>,
	peer_confirmation_plain: Option<H256>,
	shared_key: Option<KeyPair>,
}

/// Active handshake state.
enum HandshakeState<A> {
	SendPublicKey(WriteMessage<A>),
	ReceivePublicKey(ReadMessage<A>),
	SendPrivateKeySignature(WriteMessage<A>),
	ReceivePrivateKeySignature(ReadMessage<A>),
	Finished,
}

impl<A> Handshake<A> where A: AsyncRead + AsyncWrite {
	#[cfg(test)]
	pub fn set_self_confirmation_plain(&mut self, self_confirmation_plain: H256) {
		self.self_confirmation_plain = self_confirmation_plain;
	}

	#[cfg(test)]
	pub fn set_self_session_key_pair(&mut self, self_session_key_pair: KeyPair) {
		self.self_session_key_pair = Some(self_session_key_pair);
	}

	pub fn make_public_key_message(self_node_id: NodeId, confirmation_plain: H256, confirmation_signed_session: Signature) -> Result<Message, Error> {
		Ok(Message::Cluster(ClusterMessage::NodePublicKey(NodePublicKey {
			node_id: self_node_id.into(),
			confirmation_plain: confirmation_plain.into(),
			confirmation_signed_session: confirmation_signed_session.into(),
		})))
	}

	fn make_private_key_signature_message(self_key_pair: &NodeKeyPair, confirmation_plain: &H256) -> Result<Message, Error> {
		Ok(Message::Cluster(ClusterMessage::NodePrivateKeySignature(NodePrivateKeySignature {
			confirmation_signed: self_key_pair.sign(confirmation_plain)?.into(),
		})))
	}

	fn compute_shared_key(self_session_key_pair: &KeyPair, peer_session_public: &Public) -> Result<KeyPair, Error> {
		agree(self_session_key_pair.secret(), peer_session_public)
			.map_err(Into::into)
			.and_then(|s| fix_shared_key(&s))
	}
}

impl<A> Future for Handshake<A> where A: AsyncRead + AsyncWrite {
	type Item = (A, Result<HandshakeResult, Error>);
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		if let Some(error_result) = self.error.take() {
			return Ok(error_result.into());
		}

		let (next, result) = match self.state {
			HandshakeState::SendPublicKey(ref mut future) => {
				let (stream, _) = try_ready!(future.poll());

				if self.is_active {
					(HandshakeState::ReceivePublicKey(
						read_message(stream)
					), Async::NotReady)
				} else {
					let shared_key = Self::compute_shared_key(
						self.self_session_key_pair.as_ref().expect(
							"self_session_key_pair is not filled only when initialization has failed; if initialization has failed, self.error.is_some(); qed"),
						self.peer_session_public.as_ref().expect(
							"we are in passive mode; in passive mode SendPublicKey follows ReceivePublicKey; peer_session_public is filled in ReceivePublicKey; qed"),
					);

					self.shared_key = match shared_key {
						Ok(shared_key) => Some(shared_key),
						Err(err) => return Ok((stream, Err(err)).into()),
					};

					let peer_confirmation_plain = self.peer_confirmation_plain.as_ref()
						.expect("we are in passive mode; in passive mode SendPublicKey follows ReceivePublicKey; peer_confirmation_plain is filled in ReceivePublicKey; qed");
					let message = match Handshake::<A>::make_private_key_signature_message(&*self.self_key_pair, peer_confirmation_plain) {
						Ok(message) => message,
						Err(err) => return Ok((stream, Err(err)).into()),
					};

					(HandshakeState::SendPrivateKeySignature(write_encrypted_message(stream,
						self.shared_key.as_ref().expect("filled couple of lines above; qed"),
					message)), Async::NotReady)
				}
			},
			HandshakeState::ReceivePublicKey(ref mut future) => {
				let (stream, message) = try_ready!(future.poll());

				let message = match message {
					Ok(message) => match message {
						Message::Cluster(ClusterMessage::NodePublicKey(message)) => message,
						_ => return Ok((stream, Err(Error::InvalidMessage)).into()),
					},
					Err(err) => return Ok((stream, Err(err.into())).into()),
				};

				if !self.trusted_nodes.as_ref().map(|tn| tn.contains(&*message.node_id)).unwrap_or(true) {
					return Ok((stream, Err(Error::InvalidNodeId)).into());
				}

				self.peer_node_id = Some(message.node_id.into());
				self.peer_session_public = Some(match recover(&message.confirmation_signed_session, &message.confirmation_plain) {
					Ok(peer_session_public) => peer_session_public,
					Err(err) => return Ok((stream, Err(err.into())).into()),
				});
				self.peer_confirmation_plain = Some(message.confirmation_plain.into());
				if self.is_active {
					let shared_key = Self::compute_shared_key(
						self.self_session_key_pair.as_ref().expect(
							"self_session_key_pair is not filled only when initialization has failed; if initialization has failed, self.error.is_some(); qed"),
						self.peer_session_public.as_ref().expect(
							"we are in passive mode; in passive mode SendPublicKey follows ReceivePublicKey; peer_session_public is filled in ReceivePublicKey; qed"),
					);

					self.shared_key = match shared_key {
						Ok(shared_key) => Some(shared_key),
						Err(err) => return Ok((stream, Err(err)).into()),
					};

					let peer_confirmation_plain = self.peer_confirmation_plain.as_ref()
						.expect("filled couple of lines above; qed");
					let message = match Handshake::<A>::make_private_key_signature_message(&*self.self_key_pair, peer_confirmation_plain) {
						Ok(message) => message,
						Err(err) => return Ok((stream, Err(err)).into()),
					};

					(HandshakeState::SendPrivateKeySignature(write_encrypted_message(stream,
						self.shared_key.as_ref().expect("filled couple of lines above; qed"),
					message)), Async::NotReady)
				} else {
					let self_session_key_pair = self.self_session_key_pair.as_ref()
						.expect("self_session_key_pair is not filled only when initialization has failed; if initialization has failed, self.error.is_some(); qed");
					let confirmation_signed_session = match sign(self_session_key_pair.secret(), &self.self_confirmation_plain).map_err(Into::into) {
						Ok(confirmation_signed_session) => confirmation_signed_session,
						Err(err) => return Ok((stream, Err(err)).into()),
					};

					let message = match Handshake::<A>::make_public_key_message(self.self_key_pair.public().clone(), self.self_confirmation_plain.clone(), confirmation_signed_session) {
						Ok(message) => message,
						Err(err) => return Ok((stream, Err(err)).into()),
					};
					(HandshakeState::SendPublicKey(write_message(stream, message)), Async::NotReady)
				}
			},
			HandshakeState::SendPrivateKeySignature(ref mut future) => {
				let (stream, _) = try_ready!(future.poll());

				(HandshakeState::ReceivePrivateKeySignature(
					read_encrypted_message(stream,
						self.shared_key.as_ref().expect("shared_key is filled in Send/ReceivePublicKey; SendPrivateKeySignature follows Send/ReceivePublicKey; qed").clone()
					)
				), Async::NotReady)
			},
			HandshakeState::ReceivePrivateKeySignature(ref mut future) => {
				let (stream, message) = try_ready!(future.poll());

				let message = match message {
					Ok(message) => match message {
						Message::Cluster(ClusterMessage::NodePrivateKeySignature(message)) => message,
						_ => return Ok((stream, Err(Error::InvalidMessage)).into()),
					},
					Err(err) => return Ok((stream, Err(err.into())).into()),
				};

				let peer_public = self.peer_node_id.as_ref().expect("peer_node_id is filled in ReceivePublicKey; ReceivePrivateKeySignature follows ReceivePublicKey; qed");
				if !verify_public(peer_public, &*message.confirmation_signed, &self.self_confirmation_plain).unwrap_or(false) {
					return Ok((stream, Err(Error::InvalidMessage)).into());
				}

				(HandshakeState::Finished, Async::Ready((stream, Ok(HandshakeResult {
					node_id: self.peer_node_id.expect("peer_node_id is filled in ReceivePublicKey; ReceivePrivateKeySignature follows ReceivePublicKey; qed"),
					shared_key: self.shared_key.clone().expect("shared_key is filled in Send/ReceivePublicKey; ReceivePrivateKeySignature follows Send/ReceivePublicKey; qed"),
				}))))
			},
			HandshakeState::Finished => panic!("poll Handshake after it's done"),
		};

		self.state = next;
		match result {
			// by polling again, we register new future
			Async::NotReady => self.poll(),
			result => Ok(result)
		}
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::BTreeSet;
	use futures::Future;
	use ethkey::{Random, Generator, sign};
	use ethereum_types::H256;
	use key_server_cluster::PlainNodeKeyPair;
	use key_server_cluster::io::message::tests::TestIo;
	use key_server_cluster::message::{Message, ClusterMessage, NodePublicKey, NodePrivateKeySignature};
	use super::{handshake_with_init_data, accept_handshake, HandshakeResult};

	fn prepare_test_io() -> (H256, TestIo) {
		let mut io = TestIo::new();

		let self_confirmation_plain = *Random.generate().unwrap().secret().clone();
		let peer_confirmation_plain = *Random.generate().unwrap().secret().clone();

		let self_confirmation_signed = sign(io.peer_key_pair().secret(), &self_confirmation_plain).unwrap();
		let peer_confirmation_signed = sign(io.peer_session_key_pair().secret(), &peer_confirmation_plain).unwrap();

		let peer_public = io.peer_key_pair().public().clone();
		io.add_input_message(Message::Cluster(ClusterMessage::NodePublicKey(NodePublicKey {
			node_id: peer_public.into(),
			confirmation_plain: peer_confirmation_plain.into(),
			confirmation_signed_session: peer_confirmation_signed.into(),
		})));
		io.add_encrypted_input_message(Message::Cluster(ClusterMessage::NodePrivateKeySignature(NodePrivateKeySignature {
			confirmation_signed: self_confirmation_signed.into(),
		})));

		(self_confirmation_plain, io)
	}

	#[test]
	fn active_handshake_works() {
		let (self_confirmation_plain, io) = prepare_test_io();
		let trusted_nodes: BTreeSet<_> = vec![io.peer_key_pair().public().clone()].into_iter().collect();
		let self_session_key_pair = io.self_session_key_pair().clone();
		let self_key_pair = Arc::new(PlainNodeKeyPair::new(io.self_key_pair().clone()));
		let shared_key = io.shared_key_pair().clone();

		let handshake = handshake_with_init_data(io, Ok((self_confirmation_plain, self_session_key_pair)), self_key_pair, trusted_nodes);
		let handshake_result = handshake.wait().unwrap();
		assert_eq!(handshake_result.1, Ok(HandshakeResult {
			node_id: handshake_result.0.peer_key_pair().public().clone(),
			shared_key: shared_key,
		}));
	}

	#[test]
	fn passive_handshake_works() {
		let (self_confirmation_plain, io) = prepare_test_io();
		let self_key_pair = Arc::new(PlainNodeKeyPair::new(io.self_key_pair().clone()));
		let self_session_key_pair = io.self_session_key_pair().clone();
		let shared_key = io.shared_key_pair().clone();

		let mut handshake = accept_handshake(io, self_key_pair);
		handshake.set_self_confirmation_plain(self_confirmation_plain);
		handshake.set_self_session_key_pair(self_session_key_pair);

		let handshake_result = handshake.wait().unwrap();
		assert_eq!(handshake_result.1, Ok(HandshakeResult {
			node_id: handshake_result.0.peer_key_pair().public().clone(),
			shared_key: shared_key,
		}));
	}
}
