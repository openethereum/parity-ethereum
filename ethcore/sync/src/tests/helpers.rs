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

use std::collections::{VecDeque, HashSet, HashMap};
use std::sync::Arc;
use ethereum_types::H256;
use parking_lot::{RwLock, Mutex};
use bytes::Bytes;
use network::{self, PeerId, ProtocolId, PacketId, SessionInfo};
use tests::snapshot::*;
use ethcore::client::{TestBlockChainClient, BlockChainClient, Client as EthcoreClient,
	ClientConfig, ChainNotify, NewBlocks, ChainMessageType, ClientIoMessage};
use ethcore::snapshot::SnapshotService;
use ethcore::spec::Spec;
use ethcore::miner::Miner;
use ethcore::test_helpers;
use sync_io::SyncIo;
use io::{IoChannel, IoContext, IoHandler};
use api::WARP_SYNC_PROTOCOL_ID;
use chain::{ChainSync, ETH_PROTOCOL_VERSION_63, PAR_PROTOCOL_VERSION_3, PRIVATE_TRANSACTION_PACKET, SIGNED_PRIVATE_TRANSACTION_PACKET, SyncSupplier};
use SyncConfig;
use private_tx::SimplePrivateTxHandler;
use types::BlockNumber;

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
		ETH_PROTOCOL_VERSION_63.0
	}

	fn protocol_version(&self, protocol: &ProtocolId, peer_id: PeerId) -> u8 {
		if protocol == &WARP_SYNC_PROTOCOL_ID { PAR_PROTOCOL_VERSION_3.0 } else { self.eth_protocol_version(peer_id) }
	}

	fn chain_overlay(&self) -> &RwLock<HashMap<BlockNumber, Bytes>> {
		&self.overlay
	}

	fn payload_soft_limit(&self) -> usize {
		100_000
	}
}

/// Mock for emulution of async run of new blocks
struct NewBlockMessage {
	imported: Vec<H256>,
	invalid: Vec<H256>,
	enacted: Vec<H256>,
	retracted: Vec<H256>,
	sealed: Vec<H256>,
	proposed: Vec<Bytes>,
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

	/// Process the queue of pending io messages
	fn process_all_io_messages(&self);

	/// Process the queue of new block messages
	fn process_all_new_block_messages(&self);
}

pub struct EthPeer<C> where C: FlushingBlockChainClient {
	pub chain: Arc<C>,
	pub miner: Arc<Miner>,
	pub snapshot_service: Arc<TestSnapshotService>,
	pub sync: RwLock<ChainSync>,
	pub queue: RwLock<VecDeque<TestPacket>>,
	pub private_tx_handler: Arc<SimplePrivateTxHandler>,
	pub io_queue: RwLock<VecDeque<ChainMessageType>>,
	new_blocks_queue: RwLock<VecDeque<NewBlockMessage>>,
}

impl<C> EthPeer<C> where C: FlushingBlockChainClient {
	fn is_io_queue_empty(&self) -> bool {
		self.io_queue.read().is_empty()
	}

	fn is_new_blocks_queue_empty(&self) -> bool {
		self.new_blocks_queue.read().is_empty()
	}

	fn process_io_message(&self, message: ChainMessageType) {
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None);
		match message {
			ChainMessageType::Consensus(data) => self.sync.write().propagate_consensus_packet(&mut io, data),
			ChainMessageType::PrivateTransaction(transaction_hash, data) =>
				self.sync.write().propagate_private_transaction(&mut io, transaction_hash, PRIVATE_TRANSACTION_PACKET, data),
			ChainMessageType::SignedPrivateTransaction(transaction_hash, data) =>
				self.sync.write().propagate_private_transaction(&mut io, transaction_hash, SIGNED_PRIVATE_TRANSACTION_PACKET, data),
		}
	}

	fn process_new_block_message(&self, message: NewBlockMessage) {
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None);
		self.sync.write().chain_new_blocks(
			&mut io,
			&message.imported,
			&message.invalid,
			&message.enacted,
			&message.retracted,
			&message.sealed,
			&message.proposed
		);
	}
}

impl<C: FlushingBlockChainClient> Peer for EthPeer<C> {
	type Message = TestPacket;

	fn on_connect(&self, other: PeerId) {
		self.sync.write().update_targets(&*self.chain);
		self.sync.write().on_peer_connected(&mut TestIo::new(
			&*self.chain,
			&self.snapshot_service,
			&self.queue,
			Some(other)),
			other);
	}

	fn on_disconnect(&self, other: PeerId) {
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, Some(other));
		self.sync.write().on_peer_aborting(&mut io, other);
	}

	fn receive_message(&self, from: PeerId, msg: TestPacket) -> HashSet<PeerId> {
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, Some(from));
		SyncSupplier::dispatch_packet(&self.sync, &mut io, from, msg.packet_id, &msg.data);
		self.chain.flush();
		io.to_disconnect.clone()
	}

	fn pending_message(&self) -> Option<TestPacket> {
		self.chain.flush();
		self.queue.write().pop_front()
	}

	fn is_done(&self) -> bool {
		self.queue.read().is_empty() && self.is_io_queue_empty() && self.is_new_blocks_queue_empty()
	}

	fn sync_step(&self) {
		let mut io = TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None);
		self.chain.flush();
		self.sync.write().maintain_peers(&mut io);
		self.sync.write().maintain_sync(&mut io);
		self.sync.write().continue_sync(&mut io);
		self.sync.write().propagate_new_transactions(&mut io);
	}

	fn restart_sync(&self) {
		self.sync.write().restart(&mut TestIo::new(&*self.chain, &self.snapshot_service, &self.queue, None));
	}

	fn process_all_io_messages(&self) {
		if !self.is_io_queue_empty() {
			while let Some(message) = self.io_queue.write().pop_front() {
				self.process_io_message(message);
			}
		}
	}

	fn process_all_new_block_messages(&self) {
		if !self.is_new_blocks_queue_empty() {
			while let Some(message) = self.new_blocks_queue.write().pop_front() {
				self.process_new_block_message(message);
			}
		}
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
			let private_tx_handler = Arc::new(SimplePrivateTxHandler::default());
			let sync = ChainSync::new(config.clone(), &chain, Some(private_tx_handler.clone()));
			net.peers.push(Arc::new(EthPeer {
				sync: RwLock::new(sync),
				snapshot_service: ss,
				chain: Arc::new(chain),
				miner: Arc::new(Miner::new_for_tests(&Spec::new_test(), None)),
				queue: RwLock::new(VecDeque::new()),
				private_tx_handler,
				io_queue: RwLock::new(VecDeque::new()),
				new_blocks_queue: RwLock::new(VecDeque::new()),
			}));
		}
		net
	}

	// relies on Arc uniqueness, which is only true when we haven't registered a ChainNotify.
	pub fn peer_mut(&mut self, i: usize) -> &mut EthPeer<TestBlockChainClient> {
		Arc::get_mut(&mut self.peers[i]).expect("Arc never exposed externally")
	}
}

impl TestNet<EthPeer<EthcoreClient>> {
	pub fn with_spec<F>(
		n: usize,
		config: SyncConfig,
		spec_factory: F,
	) -> Self
		where F: Fn() -> Spec
	{
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
			disconnect_events: Vec::new(),
		};
		for _ in 0..n {
			net.add_peer_with_private_config(config.clone(), spec_factory());
		}
		net
	}

	pub fn add_peer_with_private_config(&mut self, config: SyncConfig, spec: Spec) {
		let channel = IoChannel::disconnected();
		let miner = Arc::new(Miner::new_for_tests(&spec, None));
		let client = EthcoreClient::new(
			ClientConfig::default(),
			&spec,
			test_helpers::new_db(),
			miner.clone(),
			channel.clone()
		).unwrap();

		let private_tx_handler = Arc::new(SimplePrivateTxHandler::default());
		let ss = Arc::new(TestSnapshotService::new());
		let sync = ChainSync::new(config, &*client, Some(private_tx_handler.clone()));
		let peer = Arc::new(EthPeer {
			sync: RwLock::new(sync),
			snapshot_service: ss,
			chain: client,
			miner,
			queue: RwLock::new(VecDeque::new()),
			private_tx_handler,
			io_queue: RwLock::new(VecDeque::new()),
			new_blocks_queue: RwLock::new(VecDeque::new()),
		});
		peer.chain.add_notify(peer.clone());
		//private_provider.add_notify(peer.clone());
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
			self.deliver_io_messages();
			self.deliver_new_block_messages();
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

	pub fn deliver_io_messages(&mut self) {
		for peer in self.peers.iter() {
			peer.process_all_io_messages();
		}
	}

	pub fn deliver_new_block_messages(&mut self) {
		for peer in self.peers.iter() {
			peer.process_all_new_block_messages();
		}
	}

	pub fn done(&self) -> bool {
		self.peers.iter().all(|p| p.is_done())
	}
}

impl<C: FlushingBlockChainClient> TestNet<EthPeer<C>> {
	pub fn trigger_chain_new_blocks(&mut self, peer_id: usize) {
		let peer = &mut self.peers[peer_id];
		peer.sync.write().chain_new_blocks(&mut TestIo::new(&*peer.chain, &peer.snapshot_service, &peer.queue, None), &[], &[], &[], &[], &[], &[]);
	}
}

pub struct TestIoHandler {
	pub client: Arc<EthcoreClient>,
	pub private_tx_queued: Mutex<usize>,
}

impl TestIoHandler {
	pub fn new(client: Arc<EthcoreClient>) -> Self {
		TestIoHandler {
			client,
			private_tx_queued: Mutex::default(),
		}
	}
}

impl IoHandler<ClientIoMessage> for TestIoHandler {
	fn message(&self, _io: &IoContext<ClientIoMessage>, net_message: &ClientIoMessage) {
		match *net_message {
			ClientIoMessage::Execute(ref exec) => {
				*self.private_tx_queued.lock() += 1;
				(*exec.0)(&self.client);
			},
			_ => {} // ignore other messages
		}
	}
}

impl ChainNotify for EthPeer<EthcoreClient> {
	fn new_blocks(&self, new_blocks: NewBlocks)
	{
		if new_blocks.has_more_blocks_to_import { return }
		let (enacted, retracted) = new_blocks.route.into_enacted_retracted();

		self.new_blocks_queue.write().push_back(NewBlockMessage {
			imported: new_blocks.imported,
			invalid: new_blocks.invalid,
			enacted,
			retracted,
			sealed: new_blocks.sealed,
			proposed: new_blocks.proposed,
		});
	}

	fn start(&self) {}

	fn stop(&self) {}

	fn broadcast(&self, message_type: ChainMessageType) {
		self.io_queue.write().push_back(message_type)
	}
}
