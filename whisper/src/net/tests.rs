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

//! Tests for the whisper network module.

use std::collections::HashSet;
use std::sync::mpsc;

use parking_lot::Mutex;
use network::{NodeId, PeerId};

use message::{CreateParams, Message};
use super::*;

struct TestHandler(Mutex<mpsc::Sender<Message>>);

impl MessageHandler for TestHandler {
	fn handle_messages(&self, messages: &[Message]) {
		let tx = self.0.lock();
		for message in messages {
			let _ = tx.send(message.clone());
		}
	}
}

struct TestPeer {
	network: Network<TestHandler>,
	recv: mpsc::Receiver<Message>,
	disconnected: Mutex<HashSet<PeerId>>,
}

impl TestPeer {
	fn create() -> Self {
		let (tx, rx) = mpsc::channel();

		TestPeer {
			network: Network::new(10 * 1024 * 1024, TestHandler(Mutex::new(tx))),
			recv: rx,
			disconnected: Mutex::new(HashSet::new()),
		}
	}
}

struct TestNetwork {
	peers: Vec<TestPeer>,
}

impl TestNetwork {
	fn new(n_peers: usize) -> Self {
		let unconnected_peers: Vec<_> = (0..n_peers).map(|_| TestPeer::create()).collect();
		for i in 0..n_peers {
			for j in (i + 1)..n_peers {
				let (peer1, peer2) = (&unconnected_peers[i], &unconnected_peers[j]);
				let ctx1 = TestContext::new(&unconnected_peers, i);
				let ctx2 = TestContext::new(&unconnected_peers, j);

				peer1.network.on_connect(&ctx1, &j);
				peer2.network.on_connect(&ctx2, &i);
			}
		}

		TestNetwork {
			peers: unconnected_peers,
		}
	}

	fn post_message_from(&self, id: PeerId, msg: Message) {
		self.peers[id].network.post_message(msg, &TestContext::new(&self.peers, id));
	}
}

enum Event {
	Disconnect(PeerId, PeerId),
	Send(PeerId, PeerId, u8, Vec<u8>),
}

struct TestContext<'a> {
	peers: &'a [TestPeer],
	local_id: PeerId,
	events: Mutex<Vec<Event>>,
}

impl<'a> TestContext<'a> {
	fn new(peers: &'a [TestPeer], local_id: PeerId) -> Self {
		TestContext {
			peers,
			local_id,
			events: Mutex::new(Vec::new()),
		}
	}
}

impl<'a> Context for TestContext<'a> {
	fn disconnect_peer(&self, id: PeerId) {
		self.events.lock().push(Event::Disconnect(self.local_id, id));
	}

	fn disable_peer(&self, id: PeerId) {
		self.events.lock().push(Event::Disconnect(self.local_id, id));
	}

	fn node_key(&self, peer: PeerId) -> Option<NodeId> {
		let mut id = NodeId::default();
		id[0] = peer as _;
		Some(id)
	}

	fn protocol_version(&self, id: ::network::ProtocolId, _peer: PeerId) -> Option<u8> {
		if &id == b"shh" || &id == b"pwh" {
			Some(PROTOCOL_VERSION as _)
		} else {
			None
		}
	}

	fn send(&self, peer: PeerId, packet: u8, data: Vec<u8>) {
		self.events.lock().push(Event::Send(self.local_id, peer, packet, data));
	}
}

impl<'a> Drop for TestContext<'a> {
	fn drop(&mut self) {
		let events = self.events.get_mut();
		while !events.is_empty() {
			let mut deferred = Vec::new();
			for event in events.drain(..) {
				match event {
					Event::Disconnect(from, target) => {
						self.peers[from].network.on_disconnect(&target);
						self.peers[target].network.on_disconnect(&from);

						self.peers[from].disconnected.lock().insert(target);
						self.peers[target].disconnected.lock().insert(from);
					}
					Event::Send(from, target, packet, data) => {
						if self.peers[from].disconnected.lock().contains(&target) {
							continue;
						}

						let mut inner_ctx = TestContext::new(self.peers, target);

						self.peers[target].network.on_packet(
							&inner_ctx,
							&from,
							packet,
							&data[..]
						);

						// don't recursively apply disconnects or new messages
						// from the receiver's actions yet.
						let inner_events = ::std::mem::replace(inner_ctx.events.get_mut(), Vec::new());
						deferred.extend(inner_events);
					}
				}
			}

			events.extend(deferred);
		}
	}
}

#[test]
fn message_gets_relayed() {
	let network = TestNetwork::new(5);
	let message = Message::create(CreateParams {
		ttl: 500,
		payload: b"this is my payload, pal".to_vec(),
		topics: vec![[0, 1, 2, 3].into()],
		work: 25,
	}).unwrap();

	network.post_message_from(0, message.clone());

	assert!(network.peers[0].recv.try_recv().is_err());

	for i in 1..5 {
		assert_eq!(network.peers[i].recv.try_recv().unwrap(), message);
	}
}
