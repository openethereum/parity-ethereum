// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! TestNet peer definition.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use light_sync::*;
use tests::helpers::{Message, TestNet, Peer as PeerLike, TestPacket};

use ethcore::client::TestBlockChainClient;
use light::client::Client as LightClient;
use light::net::{LightProtocol, IoContext, Capabilities, Params as LightParams};
use light::net::buffer_flow::FlowParams;
use network::{NodeId, PeerId};
use util::RwLock;

const NETWORK_ID: u64 = 0xcafebabe;

struct TestIoContext<'a> {
	queue: &'a RwLock<VecDeque<TestPacket>>,
	sender: Option<PeerId>,
	to_disconnect: RwLock<HashSet<PeerId>>,
}

impl<'a> IoContext for TestIoContext<'a> {
	fn send(&self, peer: PeerId, packet_id: u8, packet_body: Vec<u8>) {
		self.queue.write().push_back(TestPacket {
			data: packet_body,
			packet_id: packet_id,
			recipient: peer,
		})
	}

	fn respond(&self, packet_id: u8, packet_body: Vec<u8>) {
		if let Some(sender) = self.sender {
			self.send(sender, packet_id, packet_body);
		}
	}

	fn disconnect_peer(&self, peer: PeerId) {
		self.to_disconnect.write().insert(peer);
	}

	fn disable_peer(&self, peer: PeerId) { self.disconnect_peer(peer) }
	fn protocol_version(&self, _peer: PeerId) -> Option<u8> { Some(::light::net::MAX_PROTOCOL_VERSION) }

	fn persistent_peer_id(&self, _peer: PeerId) -> Option<NodeId> { unimplemented!() }
}

// peer-specific data.
enum PeerData {
	Light(LightSync<LightClient>, Arc<LightClient>),
	Full(Arc<TestBlockChainClient>)
}

// test peer type.
// Either a full peer or a LES peer.
pub struct Peer {
	proto: LightProtocol,
	queue: RwLock<VecDeque<TestPacket>>,
	data: PeerData,
}

impl Peer {
	// create a new full-client peer for light client peers to sync to.
	// buffer flow is made negligible.
	pub fn new_full(chain: Arc<TestBlockChainClient>) -> Self {
		let params = LightParams {
			network_id: NETWORK_ID,
			flow_params: FlowParams::free(),
			capabilities: Capabilities {
				serve_headers: true,
				serve_chain_since: None,
				serve_state_since: None,
				tx_relay: true,
			},
		};

		let proto = LightProtocol::new(chain.clone(), params);
		Peer {
			proto: proto,
			queue: RwLock::new(VecDeque::new()),
			data: PeerData::Full(chain),
		}
	}

	// create a new light-client peer to sync to full peers.
	pub fn new_light(chain: Arc<LightClient>) -> Self {
		let sync = LightSync::new(chain.clone()).unwrap();
		let params = LightParams {
			network_id: NETWORK_ID,
			flow_params: FlowParams::default(),
			capabilities: Capabilities {
				serve_headers: false,
				serve_chain_since: None,
				serve_state_since: None,
				tx_relay: false,
			},
		};

		let proto = LightProtocol::new(chain.clone(), params);
		Peer {
			proto: proto,
			queue: RwLock::new(VecDeque::new()),
			data: PeerData::Light(sync, chain),
		}
	}

	// get a test Io context based on
	fn io(&self, sender: Option<PeerId>) -> TestIoContext {
		TestIoContext {
			queue: &self.queue,
			sender: sender,
			to_disconnect: RwLock::new(HashSet::new()),
		}
	}
}

impl PeerLike for Peer {
	type Message = TestPacket;

	fn on_connect(&self, other: PeerId) {
		let io = self.io(Some(other));
		self.proto.on_connect(&other, &io);
	}

	fn on_disconnect(&self, other: PeerId){
		let io = self.io(Some(other));
		self.proto.on_disconnect(other, &io);
	}

	fn receive_message(&self, from: PeerId, msg: TestPacket) -> HashSet<PeerId> {
		let io = self.io(Some(from));
		self.proto.handle_packet(&io, &from, msg.packet_id, &msg.data);
		io.to_disconnect.into_inner()
	}

	fn pending_message(&self) -> Option<TestPacket> {
		self.queue.write().pop_front()
	}

	fn is_done(&self) -> bool {
		self.queue.read().is_empty()
	}

	fn sync_step(&self) {
		if let PeerData::Light(ref sync, _) = self.data {
			let io = self.io(None);
			self.proto.with_context(&io, |ctx| sync.maintain_sync(ctx))
		}
	}

	fn restart_sync(&self) { }
}

impl TestNet<Peer> {
	/// Create a new `TestNet` for testing light synchronization.
	pub fn light() -> Self {
		TestNet {
			peers: Vec::new(),
			started: false,
			disconnect_events: Vec::new(),
		}
	}

	/// Add a peer.
	pub fn add_peer(&mut self, peer: Peer) {
		self.peers.push(Arc::new(peer))
	}
}
