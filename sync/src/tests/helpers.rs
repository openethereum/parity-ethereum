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

use std::collections::{VecDeque, HashSet, HashMap};
use std::sync::Arc;
use ethereum_types::H256;
use parking_lot::RwLock;
use bytes::Bytes;
use network::{self, PeerId, ProtocolId, PacketId, SessionInfo};
use tests::snapshot::*;
use ethcore::client::{TestBlockChainClient, BlockChainClient, Client as EthcoreClient, ClientConfig, ChainNotify};
use ethcore::header::BlockNumber;
use ethcore::snapshot::SnapshotService;
use ethcore::spec::Spec;
use ethcore::account_provider::AccountProvider;
use ethcore::miner::Miner;
use sync_io::SyncIo;
use io::IoChannel;
use api::WARP_SYNC_PROTOCOL_ID;
use chain::ChainSync;
use ::SyncConfig;

pub trait FlushingBlockChainClient: BlockChainClient {
	fn flush(&self) {}
}

impl FlushingBlockChainClient for EthcoreClient {
	fn flush(&self) {
		self.flush_queue();
	}
}

impl FlushingBlockChainClient for TestBlockChainClient {}

pub struct TestIo<'p, C> where C: FlushingBlockChainClient, C: 'p {
	pub chain: &'p C,
	pub snapshot_service: &'p TestSnapshotService,
	pub queue: &'p RwLock<VecDeque<TestPacket>>,
	pub sender: Option<PeerId>,
	pub to_disconnect: HashSet<PeerId>,
	pub packets: Vec<TestPacket>,
	pub peers_info: HashMap<PeerId, String>,
	overlay: RwLock<HashMap<BlockNumber, Bytes>>,
}

impl<'p, C> TestIo<'p, C> where C: FlushingBlockChainClient, C: 'p {
	pub fn new(chain: &'p C, ss: &'p TestSnapshotService, queue: &'p RwLock<VecDeque<TestPacket>>, sender: Option<PeerId>) -> TestIo<'p, C> {
		TestIo {
			chain: chain,
			snapshot_service: ss,
			queue: queue,
			sender: sender,
			to_disconnect: HashSet::new(),
			overlay: RwLock::new(HashMap::new()),
			packets: Vec::new(),
			peers_info: HashMap::new(),
		}
	}
}

impl<'p, C> Drop for TestIo<'p, C> where C: FlushingBlockChainClient, C: 'p {
	fn drop(&mut self) {
		self.queue.write().extend(self.packets.drain(..));
	}
}

impl<'p, C> SyncIo for TestIo<'p, C> where C: FlushingBlockChainClient, C: 'p {
	fn disable_peer(&mut self, peer_id: PeerId) {
		self.disconnect_peer(peer_id);
	}

	fn disconnect_peer(&mut self, peer_id: PeerId) {
		self.to_disconnect.insert(peer_id);
	}

	fn is_expired(&self) -> bool {
		false
	}

	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), network::Error> {
		self.packets.push(TestPacket {
			data: data,
			packet_id: packet_id,
			recipient: self.sender.unwrap()
		});
		Ok(())
	}

	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), network::Error> {
		self.packets.push(TestPacket {
			data: data,
			packet_id: packet_id,
			recipient: peer_id,
		});
		Ok(())
	}

	fn send_protocol(&mut self, _protocol: ProtocolId, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), network::Error> {
		self.send(peer_id, packet_id, data)
	}

	fn chain(&self) -> &BlockChainClient {
		&*self.chain
	}

	fn peer_info(&self, peer_id: PeerId) -> String {
		self.peers_info.get(&peer_id)
			.cloned()
			.unwrap_or_else(|| peer_id.to_string())
	}

	fn snapshot_service(&self) -> &SnapshotService {
		self.snapshot_service
	}

	fn peer_session_info(&self, _peer_id: PeerId) -> Option<SessionInfo> {
		None
	}

	fn eth_protocol_version(&self, _peer: PeerId) -> u8 {
		63
	}

	fn protocol_version(&self, protocol: &ProtocolId, peer_id: PeerId) -> u8 {
		if protocol == &WARP_SYNC_PROTOCOL_ID { 2 } else { self.eth_protocol_version(peer_id) }
	}

	fn chain_overlay(&self) -> &RwLock<HashMap<BlockNumber, Bytes>> {
		&self.overlay
	}
}

/// Abstract messages between peers.
pub trait Message {
	/// The intended recipient of this message.
	fn recipient(&self) -> PeerId;
}

/// Mock subprotocol packet
pub struct TestPacket {
	pub data: Bytes,
	pub packet_id: PacketId,
	pub recipient: PeerId,
}

impl Message for TestPacket {
	fn recipient(&self) -> PeerId { self.recipient }
}

/// A peer which can be a member of the `TestNet`.
pub trait Peer {
	type Message: Message;

	/// Called on connection to other indicated peer.
	fn on_connect(&self, other: PeerId);

	/// Called on disconnect from other indicated peer.
	fn on_disconnect(&self, other: PeerId);

	/// Receive a message from another peer. Return a set of peers to disconnect.
	fn receive_message(&self, from: PeerId, msg: Self::Message) -> HashSet<PeerId>;

	/// Produce the next pending message to send to another peer.
	fn pending_message(&self) -> Option<Self::Message>;

	/// Whether this peer is done syncing (has no messages to send).
	fn is_done(&self) -> bool;

	/// Execute a "sync step". This is called for each peer after it sends a packet.
	fn sync_step(&self);

	/// Restart sync for a peer.
	fn restart_sync(&self);
}

pub struct EthPeer<C> where C: FlushingBlockChainClient {
	pub chain: Arc<C>,
	pub snapshot_service: Arc<TestSnapshotService>,
	pub sync: RwLock<ChainSync>,
	pub queue: RwLock<VecDeque<TestPacket>>,
}

impl<C: FlushingBlockChainClient> Peer for EthPeer<C> {
	type Message = TestPacket;

	fn on_connect(&self, other: PeerId) {
		self.sync.write().update_targets(&*self.chain);
		self.sync.write().on_peer_connected(&mut TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, Some(other)), other);
	}

	fn on_disconnect(&self, other: PeerId) {
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, Some(other));
		self.sync.write().on_peer_aborting(&mut io, other);
	}

	fn receive_message(&self, from: PeerId, msg: TestPacket) -> HashSet<PeerId> {
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, Some(from));
		ChainSync::dispatch_packet(&self.sync, &mut io, from, msg.packet_id, &msg.data);
		self.chain.flush();
		io.to_disconnect.clone()
	}

	fn pending_message(&self) -> Option<TestPacket> {
		self.chain.flush();
		self.queue.write().pop_front()
	}

	fn is_done(&self) -> bool {
		self.queue.read().is_empty()
	}

	fn sync_step(&self) {
		self.chain.flush();
		self.sync.write().maintain_peers(&mut TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None));
		self.sync.write().maintain_sync(&mut TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None));
		self.sync.write().propagate_new_transactions(&mut TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None));
	}

	fn restart_sync(&self) {
		self.sync.write().restart(&mut TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None));
	}
}

pub struct TestNet<P> {
	pub peers: Vec<Arc<P>>,
	pub started: bool,
	pub disconnect_events: Vec<(PeerId, PeerId)>, //disconnected (initiated by, to)
}

impl TestNet<EthPeer<TestBlockChainClient>> {
	pub fn new(n: usize) -> Self {
		Self::new_with_config(n, SyncConfig::default())
	}

	pub fn new_with_fork(n: usize, fork: Option<(BlockNumber, H256)>) -> Self {
		let mut config = SyncConfig::default();
		config.fork_block = fork;
		Self::new_with_config(n, config)
	}

	pub fn new_with_config(n: usize, config: SyncConfig) -> Self {
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
			disconnect_events: Vec::new(),
		};
		for _ in 0..n {
			let chain = TestBlockChainClient::new();
			let ss = Arc::new(TestSnapshotService::new());
			let sync = ChainSync::new(config.clone(), &chain);
			net.peers.push(Arc::new(EthPeer {
				sync: RwLock::new(sync),
				snapshot_service: ss,
				chain: Arc::new(chain),
				queue: RwLock::new(VecDeque::new()),
			}));
		}
		net
	}
}

impl TestNet<EthPeer<EthcoreClient>> {
	pub fn with_spec_and_accounts<F>(n: usize, config: SyncConfig, spec_factory: F, accounts: Option<Arc<AccountProvider>>) -> Self
		where F: Fn() -> Spec
	{
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
			disconnect_events: Vec::new(),
		};
		for _ in 0..n {
			net.add_peer(config.clone(), spec_factory(), accounts.clone());
		}
		net
	}

	pub fn add_peer(&mut self, config: SyncConfig, spec: Spec, accounts: Option<Arc<AccountProvider>>) {
		let client = EthcoreClient::new(
			ClientConfig::default(),
			&spec,
			Arc::new(::kvdb_memorydb::create(::ethcore::db::NUM_COLUMNS.unwrap_or(0))),
			Arc::new(Miner::with_spec_and_accounts(&spec, accounts)),
			IoChannel::disconnected(),
		).unwrap();

		let ss = Arc::new(TestSnapshotService::new());
		let sync = ChainSync::new(config, &*client);
		let peer = Arc::new(EthPeer {
			sync: RwLock::new(sync),
			snapshot_service: ss,
			chain: client,
			queue: RwLock::new(VecDeque::new()),
		});
		peer.chain.add_notify(peer.clone());
		self.peers.push(peer);
	}
}

impl<P> TestNet<P> where P: Peer {
	pub fn peer(&self, i: usize) -> &P {
		&self.peers[i]
	}

	pub fn start(&mut self) {
		if self.started {
			return;
		}
		for peer in 0..self.peers.len() {
			for client in 0..self.peers.len() {
				if peer != client {
					self.peers[peer].on_connect(client as PeerId);
				}
			}
		}
		self.started = true;
	}

	pub fn sync_step(&mut self) {
		for peer in 0..self.peers.len() {
			let packet = self.peers[peer].pending_message();
			if let Some(packet) = packet {
				let disconnecting = {
					let recipient = packet.recipient();
					trace!("--- {} -> {} ---", peer, recipient);
					let to_disconnect = self.peers[recipient].receive_message(peer as PeerId, packet);
					for d in &to_disconnect {
						// notify this that disconnecting peers are disconnecting
						self.peers[recipient].on_disconnect(*d as PeerId);
						self.disconnect_events.push((peer, *d));
					}
					to_disconnect
				};
				for d in &disconnecting {
					// notify other peers that this peer is disconnecting
					self.peers[*d].on_disconnect(peer as PeerId);
				}
			}

			self.sync_step_peer(peer);
		}
	}

	pub fn sync_step_peer(&mut self, peer_num: usize) {
		self.peers[peer_num].sync_step();
	}

	pub fn restart_peer(&mut self, i: usize) {
		self.peers[i].restart_sync();
	}

	pub fn sync(&mut self) -> u32 {
		self.start();
		let mut total_steps = 0;
		while !self.done() {
			self.sync_step();
			total_steps += 1;
		}
		total_steps
	}

	pub fn sync_steps(&mut self, count: usize) {
		self.start();
		for _ in 0..count {
			self.sync_step();
		}
	}

	pub fn done(&self) -> bool {
		self.peers.iter().all(|p| p.is_done())
	}
}

impl TestNet<EthPeer<TestBlockChainClient>> {
	// relies on Arc uniqueness, which is only true when we haven't registered a ChainNotify.
	pub fn peer_mut(&mut self, i: usize) -> &mut EthPeer<TestBlockChainClient> {
		Arc::get_mut(&mut self.peers[i]).expect("Arc never exposed externally")
	}
}

impl<C: FlushingBlockChainClient> TestNet<EthPeer<C>> {
	pub fn trigger_chain_new_blocks(&mut self, peer_id: usize) {
		let peer = &mut self.peers[peer_id];
		peer.sync.write().chain_new_blocks(&mut TestIo::new(&*peer.chain, &peer.snapshot_service, &peer.queue, None), &[], &[], &[], &[], &[], &[]);
	}
}

impl ChainNotify for EthPeer<EthcoreClient> {
	fn new_blocks(&self,
		imported: Vec<H256>,
		invalid: Vec<H256>,
		enacted: Vec<H256>,
		retracted: Vec<H256>,
		sealed: Vec<H256>,
		proposed: Vec<Bytes>,
		_duration: u64)
	{
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None);
		self.sync.write().chain_new_blocks(
			&mut io,
			&imported,
			&invalid,
			&enacted,
			&retracted,
			&sealed,
			&proposed);
	}

	fn start(&self) {}

	fn stop(&self) {}

	fn broadcast(&self, message: Vec<u8>) {
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None);
		self.sync.write().propagate_consensus_packet(&mut io, message.clone());
	}
}
