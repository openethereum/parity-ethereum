// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use util::*;
use network::*;
use tests::snapshot::*;
use ethcore::client::{TestBlockChainClient, BlockChainClient};
use ethcore::header::BlockNumber;
use ethcore::snapshot::SnapshotService;
use sync_io::SyncIo;
use chain::ChainSync;
use ::SyncConfig;

pub struct TestIo<'p> {
	pub chain: &'p mut TestBlockChainClient,
	pub snapshot_service: &'p TestSnapshotService,
	pub queue: &'p mut VecDeque<TestPacket>,
	pub sender: Option<PeerId>,
	pub to_disconnect: HashSet<PeerId>,
	overlay: RwLock<HashMap<BlockNumber, Bytes>>,
}

impl<'p> TestIo<'p> {
	pub fn new(chain: &'p mut TestBlockChainClient, ss: &'p TestSnapshotService, queue: &'p mut VecDeque<TestPacket>, sender: Option<PeerId>) -> TestIo<'p> {
		TestIo {
			chain: chain,
			snapshot_service: ss,
			queue: queue,
			sender: sender,
			to_disconnect: HashSet::new(),
			overlay: RwLock::new(HashMap::new()),
		}
	}
}

impl<'p> SyncIo for TestIo<'p> {
	fn disable_peer(&mut self, peer_id: PeerId) {
		self.disconnect_peer(peer_id);
	}

	fn disconnect_peer(&mut self, peer_id: PeerId) {
		self.to_disconnect.insert(peer_id);
	}

	fn is_expired(&self) -> bool {
		false
	}

	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), NetworkError> {
		self.queue.push_back(TestPacket {
			data: data,
			packet_id: packet_id,
			recipient: self.sender.unwrap()
		});
		Ok(())
	}

	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), NetworkError> {
		self.queue.push_back(TestPacket {
			data: data,
			packet_id: packet_id,
			recipient: peer_id,
		});
		Ok(())
	}

	fn chain(&self) -> &BlockChainClient {
		self.chain
	}

	fn snapshot_service(&self) -> &SnapshotService {
		self.snapshot_service
	}

	fn peer_session_info(&self, _peer_id: PeerId) -> Option<SessionInfo> {
		None
	}

	fn eth_protocol_version(&self, _peer: PeerId) -> u8 {
		64
	}

	fn chain_overlay(&self) -> &RwLock<HashMap<BlockNumber, Bytes>> {
		&self.overlay
	}
}

pub struct TestPacket {
	pub data: Bytes,
	pub packet_id: PacketId,
	pub recipient: PeerId,
}

pub struct TestPeer {
	pub chain: TestBlockChainClient,
	pub snapshot_service: Arc<TestSnapshotService>,
	pub sync: RwLock<ChainSync>,
	pub queue: VecDeque<TestPacket>,
}

pub struct TestNet {
	pub peers: Vec<TestPeer>,
	pub started: bool,
}

impl TestNet {
	pub fn new(n: usize) -> TestNet {
		Self::new_with_fork(n, None)
	}

	pub fn new_with_fork(n: usize, fork: Option<(BlockNumber, H256)>) -> TestNet {
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
		};
		for _ in 0..n {
			let chain = TestBlockChainClient::new();
			let mut config = SyncConfig::default();
			config.fork_block = fork;
			let ss = Arc::new(TestSnapshotService::new());
			let sync = ChainSync::new(config, &chain);
			net.peers.push(TestPeer {
				sync: RwLock::new(sync),
				snapshot_service: ss,
				chain: chain,
				queue: VecDeque::new(),
			});
		}
		net
	}

	pub fn peer(&self, i: usize) -> &TestPeer {
		self.peers.get(i).unwrap()
	}

	pub fn peer_mut(&mut self, i: usize) -> &mut TestPeer {
		self.peers.get_mut(i).unwrap()
	}

	pub fn start(&mut self) {
		for peer in 0..self.peers.len() {
			for client in 0..self.peers.len() {
				if peer != client {
					let mut p = self.peers.get_mut(peer).unwrap();
					p.sync.write().restart(&mut TestIo::new(&mut p.chain, &p.snapshot_service, &mut p.queue, Some(client as PeerId)));
					p.sync.write().on_peer_connected(&mut TestIo::new(&mut p.chain, &p.snapshot_service, &mut p.queue, Some(client as PeerId)), client as PeerId);
				}
			}
		}
	}

	pub fn sync_step(&mut self) {
		for peer in 0..self.peers.len() {
			if let Some(packet) = self.peers[peer].queue.pop_front() {
				let disconnecting = {
					let mut p = self.peers.get_mut(packet.recipient).unwrap();
					trace!("--- {} -> {} ---", peer, packet.recipient);
					let to_disconnect = {
						let mut io = TestIo::new(&mut p.chain, &p.snapshot_service, &mut p.queue, Some(peer as PeerId));
						ChainSync::dispatch_packet(&p.sync, &mut io, peer as PeerId, packet.packet_id, &packet.data);
						io.to_disconnect
					};
					for d in &to_disconnect {
						// notify this that disconnecting peers are disconnecting
						let mut io = TestIo::new(&mut p.chain, &p.snapshot_service, &mut p.queue, Some(*d));
						p.sync.write().on_peer_aborting(&mut io, *d);
					}
					to_disconnect
				};
				for d in &disconnecting {
					// notify other peers that this peer is disconnecting
					let mut p = self.peers.get_mut(*d).unwrap();
					let mut io = TestIo::new(&mut p.chain, &p.snapshot_service, &mut p.queue, Some(peer as PeerId));
					p.sync.write().on_peer_aborting(&mut io, peer as PeerId);
				}
			}

			self.sync_step_peer(peer);
		}
	}

	pub fn sync_step_peer(&mut self, peer_num: usize) {
		let mut peer = self.peer_mut(peer_num);
		peer.sync.write().maintain_sync(&mut TestIo::new(&mut peer.chain, &peer.snapshot_service, &mut peer.queue, None));
	}

	pub fn restart_peer(&mut self, i: usize) {
		let peer = self.peer_mut(i);
		peer.sync.write().restart(&mut TestIo::new(&mut peer.chain, &peer.snapshot_service, &mut peer.queue, None));
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
		if !self.started {
			self.start();
			self.started = true;
		}
		for _ in 0..count {
			self.sync_step();
		}
	}

	pub fn done(&self) -> bool {
		self.peers.iter().all(|p| p.queue.is_empty())
	}

	pub fn trigger_chain_new_blocks(&mut self, peer_id: usize) {
		let mut peer = self.peer_mut(peer_id);
		peer.sync.write().chain_new_blocks(&mut TestIo::new(&mut peer.chain, &peer.snapshot_service, &mut peer.queue, None), &[], &[], &[], &[], &[]);
	}
}
