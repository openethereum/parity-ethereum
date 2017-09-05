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

use std::io;
use std::sync::Arc;
use std::collections::BTreeSet;
use futures::{Future, Poll, Async};
use tokio_io::{AsyncRead, AsyncWrite};
use ethkey::{Random, Generator, KeyPair, verify_public};
use bigint::hash::H256;
use key_server_cluster::{NodeId, Error, NodeKeyPair};
use key_server_cluster::message::{Message, ClusterMessage, NodePublicKey, NodePrivateKeySignature};
use key_server_cluster::io::{write_message, write_encrypted_message, WriteMessage, ReadMessage,
	read_message, read_encrypted_message, fix_shared_key};

/// Start handshake procedure with another node from the cluster.
pub fn handshake<A>(a: A, self_key_pair: Arc<NodeKeyPair>, trusted_nodes: BTreeSet<NodeId>) -> Handshake<A> where A: AsyncWrite + AsyncRead {
	let self_confirmation_plain = Random.generate().map(|kp| *kp.secret().clone()).map_err(Into::into);
	handshake_with_plain_confirmation(a, self_confirmation_plain, self_key_pair, trusted_nodes)
}

/// Start handshake procedure with another node from the cluster and given plain confirmation.
pub fn handshake_with_plain_confirmation<A>(a: A, self_confirmation_plain: Result<H256, Error>, self_key_pair: Arc<NodeKeyPair>, trusted_nodes: BTreeSet<NodeId>) -> Handshake<A> where A: AsyncWrite + AsyncRead {
	let (error, state) = match self_confirmation_plain.clone()
		.and_then(|c| Handshake::<A>::make_public_key_message(self_key_pair.public().clone(), c)) {
		Ok(message) => (None, HandshakeState::SendPublicKey(write_message(a, message))),
		Err(err) => (Some((a, Err(err))), HandshakeState::Finished),
	};

	Handshake {
		is_active: true,
		error: error,
		state: state,
		self_key_pair: self_key_pair,
		self_confirmation_plain: self_confirmation_plain.unwrap_or(Default::default()),
		trusted_nodes: Some(trusted_nodes),
		other_node_id: None,
		other_confirmation_plain: None,
		shared_key: None,
	}
}

/// Wait for handshake procedure to be started by another node from the cluster.
pub fn accept_handshake<A>(a: A, self_key_pair: Arc<NodeKeyPair>) -> Handshake<A> where A: AsyncWrite + AsyncRead {
	let self_confirmation_plain = Random.generate().map(|kp| *kp.secret().clone()).map_err(Into::into);
	let (error, state) = match self_confirmation_plain.clone() {
		Ok(_) => (None, HandshakeState::ReceivePublicKey(read_message(a))),
		Err(err) => (Some((a, Err(err))), HandshakeState::Finished),
	};

	Handshake {
		is_active: false,
		error: error,
		state: state,
		self_key_pair: self_key_pair,
		self_confirmation_plain: self_confirmation_plain.unwrap_or(Default::default()),
		trusted_nodes: None,
		other_node_id: None,
		other_confirmation_plain: None,
		shared_key: None,
	}
}

#[derive(Debug, PartialEq)]
/// Result of handshake procedure.
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
	self_confirmation_plain: H256,
	trusted_nodes: Option<BTreeSet<NodeId>>,
	other_node_id: Option<NodeId>,
	other_confirmation_plain: Option<H256>,
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

	pub fn make_public_key_message(self_node_id: NodeId, confirmation_plain: H256) -> Result<Message, Error> {
		Ok(Message::Cluster(ClusterMessage::NodePublicKey(NodePublicKey {
			node_id: self_node_id.into(),
			confirmation_plain: confirmation_plain.into(),
		})))
	}

	fn make_private_key_signature_message(self_key_pair: &NodeKeyPair, confirmation_plain: &H256) -> Result<Message, Error> {
		Ok(Message::Cluster(ClusterMessage::NodePrivateKeySignature(NodePrivateKeySignature {
			confirmation_signed: self_key_pair.sign(confirmation_plain)?.into(),
		})))
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
					self.shared_key = match self.self_key_pair.compute_shared_key(
						self.other_node_id.as_ref().expect("we are in passive mode; in passive mode SendPublicKey follows ReceivePublicKey; other_node_id is filled in ReceivePublicKey; qed")
					).map_err(Into::into).and_then(|sk| fix_shared_key(sk.secret())) {
						Ok(shared_key) => Some(shared_key),
						Err(err) => return Ok((stream, Err(err.into())).into()),
					};

					let message = match Handshake::<A>::make_private_key_signature_message(
						&*self.self_key_pair,
						self.other_confirmation_plain.as_ref().expect("we are in passive mode; in passive mode SendPublicKey follows ReceivePublicKey; other_confirmation_plain is filled in ReceivePublicKey; qed")
					) {
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

				self.other_node_id = Some(message.node_id.into());
				self.other_confirmation_plain = Some(message.confirmation_plain.into());
				if self.is_active {
					self.shared_key = match self.self_key_pair.compute_shared_key(
						self.other_node_id.as_ref().expect("filled couple of lines above; qed")
					).map_err(Into::into).and_then(|sk| fix_shared_key(sk.secret())) {
						Ok(shared_key) => Some(shared_key),
						Err(err) => return Ok((stream, Err(err.into())).into()),
					};

					let message = match Handshake::<A>::make_private_key_signature_message(
						&*self.self_key_pair,
						self.other_confirmation_plain.as_ref().expect("filled couple of lines above; qed")
					) {
						Ok(message) => message,
						Err(err) => return Ok((stream, Err(err)).into()),
					};
					(HandshakeState::SendPrivateKeySignature(write_encrypted_message(stream,
						self.shared_key.as_ref().expect("filled couple of lines above; qed"),
					message)), Async::NotReady)
				} else {
					let message = match Handshake::<A>::make_public_key_message(self.self_key_pair.public().clone(), self.self_confirmation_plain.clone()) {
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

				let other_node_public = self.other_node_id.as_ref().expect("other_node_id is filled in ReceivePublicKey; ReceivePrivateKeySignature follows ReceivePublicKey; qed");
				if !verify_public(other_node_public, &*message.confirmation_signed, &self.self_confirmation_plain).unwrap_or(false) {
					return Ok((stream, Err(Error::InvalidMessage)).into());
				}

				(HandshakeState::Finished, Async::Ready((stream, Ok(HandshakeResult {
					node_id: self.other_node_id.expect("other_node_id is filled in ReceivePublicKey; ReceivePrivateKeySignature follows ReceivePublicKey; qed"),
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
	use ethcrypto::ecdh::agree;
	use bigint::hash::H256;
	use key_server_cluster::PlainNodeKeyPair;
	use key_server_cluster::io::message::fix_shared_key;
	use key_server_cluster::io::message::tests::TestIo;
	use key_server_cluster::message::{Message, ClusterMessage, NodePublicKey, NodePrivateKeySignature};
	use super::{handshake_with_plain_confirmation, accept_handshake, HandshakeResult};

	fn prepare_test_io() -> (H256, TestIo) {
		let self_key_pair = Random.generate().unwrap();
		let peer_key_pair = Random.generate().unwrap();
		let mut io = TestIo::new(self_key_pair.clone(), peer_key_pair.public().clone());

		let self_confirmation_plain = *Random.generate().unwrap().secret().clone();
		let peer_confirmation_plain = *Random.generate().unwrap().secret().clone();

		let self_confirmation_signed = sign(peer_key_pair.secret(), &self_confirmation_plain).unwrap();

		io.add_input_message(Message::Cluster(ClusterMessage::NodePublicKey(NodePublicKey {
			node_id: peer_key_pair.public().clone().into(),
			confirmation_plain: peer_confirmation_plain.into(),
		})));
		io.add_encrypted_input_message(Message::Cluster(ClusterMessage::NodePrivateKeySignature(NodePrivateKeySignature {
			confirmation_signed: self_confirmation_signed.into(),
		})));

		(self_confirmation_plain, io)
	}

	#[test]
	fn active_handshake_works() {
		let (self_confirmation_plain, io) = prepare_test_io();
		let self_key_pair = io.self_key_pair().clone();
		let trusted_nodes: BTreeSet<_> = vec![io.peer_public().clone()].into_iter().collect();
		let shared_key = fix_shared_key(&agree(self_key_pair.secret(), trusted_nodes.iter().nth(0).unwrap()).unwrap()).unwrap();

		let handshake = handshake_with_plain_confirmation(io, Ok(self_confirmation_plain), Arc::new(PlainNodeKeyPair::new(self_key_pair)), trusted_nodes);
		let handshake_result = handshake.wait().unwrap();
		assert_eq!(handshake_result.1, Ok(HandshakeResult {
			node_id: handshake_result.0.peer_public().clone(),
			shared_key: shared_key,
		}));
	}

	#[test]
	fn passive_handshake_works() {
		let (self_confirmation_plain, io) = prepare_test_io();
		let self_key_pair = io.self_key_pair().clone();
		let trusted_nodes: BTreeSet<_> = vec![io.peer_public().clone()].into_iter().collect();
		let shared_key = fix_shared_key(&agree(self_key_pair.secret(), trusted_nodes.iter().nth(0).unwrap()).unwrap()).unwrap();

		let mut handshake = accept_handshake(io, Arc::new(PlainNodeKeyPair::new(self_key_pair)));
		handshake.set_self_confirmation_plain(self_confirmation_plain);

		let handshake_result = handshake.wait().unwrap();
		assert_eq!(handshake_result.1, Ok(HandshakeResult {
			node_id: handshake_result.0.peer_public().clone(),
			shared_key: shared_key,
		}));
	}
}
