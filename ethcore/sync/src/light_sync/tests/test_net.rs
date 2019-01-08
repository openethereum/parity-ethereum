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

//! TestNet peer definition.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use light_sync::*;
use tests::helpers::{TestNet, Peer as PeerLike, TestPacket};

use ethcore::client::TestBlockChainClient;
use ethcore::spec::Spec;
use io::IoChannel;
use kvdb_memorydb;
use light::client::fetch::{self, Unavailable};
use light::net::{LightProtocol, IoContext, Capabilities, Params as LightParams};
use light::provider::LightProvider;
use network::{NodeId, PeerId};
use parking_lot::RwLock;

use std::time::Duration;
use light::cache::Cache;

const NETWORK_ID: u64 = 0xcafebabe;

pub type LightClient = ::light::client::Client<Unavailable>;

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
	fn is_reserved_peer(&self, _peer: PeerId) -> bool { false }
}

// peer-specific data.
enum PeerData {
	Light(Arc<LightSync<LightClient>>, Arc<LightClient>),
	Full(Arc<TestBlockChainClient>)
}

// test peer type.
// Either a full peer or a light peer.
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
			config: Default::default(),
			capabilities: Capabilities {
				serve_headers: true,
				serve_chain_since: None,
				serve_state_since: None,
				tx_relay: true,
			},
			sample_store: None,
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
		let sync = Arc::new(LightSync::new(chain.clone()).unwrap());
		let params = LightParams {
			network_id: NETWORK_ID,
			config: Default::default(),
			capabilities: Capabilities {
				serve_headers: false,
				serve_chain_since: None,
				serve_state_since: None,
				tx_relay: false,
			},
			sample_store: None,
		};

		let provider = LightProvider::new(chain.clone(), Arc::new(RwLock::new(Default::default())));
		let mut proto = LightProtocol::new(Arc::new(provider), params);
		proto.add_handler(sync.clone());
		Peer {
			proto: proto,
			queue: RwLock::new(VecDeque::new()),
			data: PeerData::Light(sync, chain),
		}
	}

	// get the chain from the client, asserting that it is a full node.
	pub fn chain(&self) -> &TestBlockChainClient {
		match self.data {
			PeerData::Full(ref chain) => &*chain,
			_ => panic!("Attempted to access full chain on light peer."),
		}
	}

	// get the light chain from the peer, asserting that it is a light node.
	pub fn light_chain(&self) -> &LightClient {
		match self.data {
			PeerData::Light(_, ref chain) => &*chain,
			_ => panic!("Attempted to access light chain on full peer."),
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
		self.proto.on_connect(other, &io);
	}

	fn on_disconnect(&self, other: PeerId){
		let io = self.io(Some(other));
		self.proto.on_disconnect(other, &io);
	}

	fn receive_message(&self, from: PeerId, msg: TestPacket) -> HashSet<PeerId> {
		let io = self.io(Some(from));
		self.proto.handle_packet(&io, from, msg.packet_id, &msg.data);
		io.to_disconnect.into_inner()
	}

	fn pending_message(&self) -> Option<TestPacket> {
		self.queue.write().pop_front()
	}

	fn is_done(&self) -> bool {
		self.queue.read().is_empty() && match self.data {
			PeerData::Light(_, ref client) => {
				// should create a test light client which just imports
				// headers directly and doesn't have a queue to drain.
				client.import_verified();
				client.queue_info().is_empty()
			}
			_ => true,
		}
	}

	fn sync_step(&self) {
		if let PeerData::Light(_, ref client) = self.data {
			client.flush_queue();

			while !client.queue_info().is_empty() {
				client.import_verified()
			}
		}
	}

	fn restart_sync(&self) { }

	fn process_all_io_messages(&self) { }

	fn process_all_new_block_messages(&self) { }
}

impl TestNet<Peer> {
	/// Create a new `TestNet` for testing light synchronization.
	/// The first parameter is the number of light nodes,
	/// the second is the number of full nodes.
	pub fn light(n_light: usize, n_full: usize) -> Self {
		let mut peers = Vec::with_capacity(n_light + n_full);
		for _ in 0..n_light {
			let mut config = ::light::client::Config::default();

			// skip full verification because the blocks are bad.
			config.verify_full = false;
			let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));
			let db = kvdb_memorydb::create(0);
			let client = LightClient::new(
				config,
				Arc::new(db),
				None,
				&Spec::new_test(),
				fetch::unavailable(), // TODO: allow fetch from full nodes.
				IoChannel::disconnected(),
				cache
			).expect("New DB creation infallible; qed");

			peers.push(Arc::new(Peer::new_light(Arc::new(client))))
		}

		for _ in 0..n_full {
			peers.push(Arc::new(Peer::new_full(Arc::new(TestBlockChainClient::new()))))
		}

		TestNet {
			peers: peers,
			started: false,
			disconnect_events: Vec::new(),
		}
	}
}
