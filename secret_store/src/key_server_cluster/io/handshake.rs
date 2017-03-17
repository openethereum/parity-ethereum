// TODO: move unwraps to Ok(Err(aaa))

use std::{io, cmp};
use std::collections::BTreeSet;
use futures::{Future, Poll, Async};
use ethkey::{Random, Generator, KeyPair, Secret, sign, verify_public};
use util::H256;
use key_server_cluster::{NodeId, Error};
use key_server_cluster::message::{Message, ClusterMessage, NodePublicKey, NodePrivateKeySignature};
use key_server_cluster::io::{write_message, WriteMessage, ReadMessage, read_message, serialize_message, SerializedMessage};

/// Start handshake procedure with another node from the cluster.
pub fn handshake<A>(a: A, self_key_pair: KeyPair, trusted_nodes: BTreeSet<NodeId>) -> Handshake<A> where A: io::Write + io::Read {
	let self_confirmation_plain = *Random.generate().unwrap().secret().clone();
	handshake_with_plain_confirmation(a, self_confirmation_plain, self_key_pair, trusted_nodes)
}

pub fn handshake_with_plain_confirmation<A>(a: A, self_confirmation_plain: H256, self_key_pair: KeyPair, trusted_nodes: BTreeSet<NodeId>) -> Handshake<A> where A: io::Write + io::Read {
	let message = Handshake::<A>::make_public_key_message(self_key_pair.public().clone(), self_confirmation_plain.clone()).unwrap();
	Handshake {
		is_active: true,
		state: HandshakeState::SendPublicKey(write_message(a, message)),
		self_key_pair: self_key_pair,
		self_confirmation_plain: self_confirmation_plain,
		trusted_nodes: trusted_nodes,
		other_node_id: None,
		other_confirmation_plain: None,
	}
}

/// Wait for handshake procedure to be started by another node from the cluster.
pub fn accept_handshake<A>(a: A, self_key_pair: KeyPair, trusted_nodes: BTreeSet<NodeId>) -> Handshake<A> where A: io::Write + io::Read {
	let self_confirmation_plain = *Random.generate().unwrap().secret().clone();

	Handshake {
		is_active: false,
		state: HandshakeState::ReceivePublicKey(read_message(a)),
		self_key_pair: self_key_pair,
		self_confirmation_plain: self_confirmation_plain,
		trusted_nodes: trusted_nodes,
		other_node_id: None,
		other_confirmation_plain: None,
	}
}

#[derive(Debug, PartialEq)]
/// Result of handshake procedure.
pub struct HandshakeResult {
	/// Node id.
	pub node_id: NodeId,
}

/// Future handshake procedure.
pub struct Handshake<A> {
	is_active: bool,
	state: HandshakeState<A>,
	self_key_pair: KeyPair,
	self_confirmation_plain: H256,
	trusted_nodes: BTreeSet<NodeId>,
	other_node_id: Option<NodeId>,
	other_confirmation_plain: Option<H256>,
}

/// Active handshake state.
enum HandshakeState<A> {
	SendPublicKey(WriteMessage<A>),
	ReceivePublicKey(ReadMessage<A>),
	SendPrivateKeySignature(WriteMessage<A>),
	ReceivePrivateKeySignature(ReadMessage<A>),
	Finished,
}

impl<A> Handshake<A> where A: io::Read + io::Write {
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

	fn make_private_key_signature_message(secret: &Secret, confirmation_plain: &H256) -> Result<Message, Error> {
		Ok(Message::Cluster(ClusterMessage::NodePrivateKeySignature(NodePrivateKeySignature {
			confirmation_signed: sign(secret, confirmation_plain)?.into(),
		})))
	}
}

impl<A> Future for Handshake<A> where A: io::Read + io::Write {
	type Item = (A, Result<HandshakeResult, Error>);
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let (next, result) = match self.state {
			HandshakeState::SendPublicKey(ref mut future) => {
				let (stream, _) = try_ready!(future.poll());

				if self.is_active {
					(HandshakeState::ReceivePublicKey(
						read_message(stream)
					), Async::NotReady)
				} else {
					let message = Handshake::<A>::make_private_key_signature_message(
						self.self_key_pair.secret(),
						self.other_confirmation_plain.as_ref().expect("we are in passive mode; in passive mode SendPublicKey follows ReceivePublicKey; other_confirmation_plain is filled in ReceivePublicKey; qed")
					).unwrap();
					(HandshakeState::SendPrivateKeySignature(write_message(stream, message)), Async::NotReady)
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

				if !self.trusted_nodes.contains(&*message.node_id) {
					return Ok((stream, Err(Error::InvalidNodeId)).into());
				}

				self.other_node_id = Some(message.node_id.into());
				self.other_confirmation_plain = Some(message.confirmation_plain.into());
				if self.is_active {
					let message = Handshake::<A>::make_private_key_signature_message(
						self.self_key_pair.secret(),
						self.other_confirmation_plain.as_ref().expect("filled couple of lines above; qed")
					).unwrap();
					(HandshakeState::SendPrivateKeySignature(write_message(stream, message)), Async::NotReady)
				} else {
					(HandshakeState::SendPublicKey(
						write_message(stream, Handshake::<A>::make_public_key_message(self.self_key_pair.public().clone(), self.self_confirmation_plain.clone()).unwrap())
					), Async::NotReady)
				}
			},
			HandshakeState::SendPrivateKeySignature(ref mut future) => {
				let (stream, _) = try_ready!(future.poll());

				(HandshakeState::ReceivePrivateKeySignature(
					read_message(stream)
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
					node_id: self.other_node_id.expect("other_node_id is filled in ReceivePublicKey; ReceivePrivateKeySignature follows ReceivePublicKey; qed")
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
	use std::io;
	use std::collections::BTreeSet;
	use futures::Future;
	use ethkey::{Random, Generator, sign};
	use util::H256;
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
		let peer_confirmation_signed = sign(self_key_pair.secret(), &peer_confirmation_plain).unwrap();

		io.add_input_message(Message::Cluster(ClusterMessage::NodePublicKey(NodePublicKey {
			node_id: peer_key_pair.public().clone().into(),
			confirmation_plain: peer_confirmation_plain.into(),
		})));
		io.add_input_message(Message::Cluster(ClusterMessage::NodePrivateKeySignature(NodePrivateKeySignature {
			confirmation_signed: self_confirmation_signed.into(),
		})));

		io.add_output_message(Message::Cluster(ClusterMessage::NodePublicKey(NodePublicKey {
			node_id: self_key_pair.public().clone().into(),
			confirmation_plain: self_confirmation_plain.clone().into(),
		})));
		io.add_output_message(Message::Cluster(ClusterMessage::NodePrivateKeySignature(NodePrivateKeySignature {
			confirmation_signed: peer_confirmation_signed.into(),
		})));

		(self_confirmation_plain, io)
	}

	#[test]
	fn active_handshake_works() {
		let (self_confirmation_plain, io) = prepare_test_io();
		let self_key_pair = io.self_key_pair().clone();
		let trusted_nodes: BTreeSet<_> = vec![io.peer_public().clone()].into_iter().collect();

		let handshake = handshake_with_plain_confirmation(io, self_confirmation_plain, self_key_pair, trusted_nodes);
		let handshake_result = handshake.wait().unwrap();
		assert_eq!(handshake_result.1, Ok(HandshakeResult {
			node_id: handshake_result.0.peer_public().clone(),
		}));
		handshake_result.0.assert_output();
	}

	#[test]
	fn passive_handshake_works() {
		let (self_confirmation_plain, io) = prepare_test_io();
		let self_key_pair = io.self_key_pair().clone();
		let trusted_nodes: BTreeSet<_> = vec![io.peer_public().clone()].into_iter().collect();

		let mut handshake = accept_handshake(io, self_key_pair, trusted_nodes);
		handshake.set_self_confirmation_plain(self_confirmation_plain);

		let handshake_result = handshake.wait().unwrap();
		assert_eq!(handshake_result.1, Ok(HandshakeResult {
			node_id: handshake_result.0.peer_public().clone(),
		}));
		handshake_result.0.assert_output();
	}
}
