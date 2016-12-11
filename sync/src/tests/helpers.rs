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

use util::*;
use network::*;
use tests::snapshot::*;
use ethcore::client::{TestBlockChainClient, BlockChainClient, Client as EthcoreClient, ClientConfig, ChainNotify};
use ethcore::header::BlockNumber;
use ethcore::snapshot::SnapshotService;
use ethcore::spec::Spec;
use ethcore::miner::Miner;
use ethcore::db::NUM_COLUMNS;
use sync_io::SyncIo;
use io::IoChannel;
use api::WARP_SYNC_PROTOCOL_ID;
use chain::ChainSync;
use ::SyncConfig;
use devtools::{self, GuardedTempResult};

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
	pub queue: &'p mut VecDeque<TestPacket>,
	pub sender: Option<PeerId>,
	pub to_disconnect: HashSet<PeerId>,
	overlay: RwLock<HashMap<BlockNumber, Bytes>>,
}

impl<'p, C> TestIo<'p, C> where C: FlushingBlockChainClient, C: 'p {
	pub fn new(chain: &'p C, ss: &'p TestSnapshotService, queue: &'p mut VecDeque<TestPacket>, sender: Option<PeerId>) -> TestIo<'p, C> {
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

	fn send_protocol(&mut self, _protocol: ProtocolId, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), NetworkError> {
		self.send(peer_id, packet_id, data)
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
		63
	}

	fn protocol_version(&self, protocol: &ProtocolId, peer_id: PeerId) -> u8 {
		if protocol == &WARP_SYNC_PROTOCOL_ID { 2 } else { self.eth_protocol_version(peer_id) }
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

pub struct TestPeer<C> where C: FlushingBlockChainClient {
	pub chain: C,
	pub snapshot_service: Arc<TestSnapshotService>,
	pub sync: RwLock<ChainSync>,
	pub queue: RwLock<VecDeque<TestPacket>>,
}

pub struct TestNet<C> where C: FlushingBlockChainClient {
	pub peers: Vec<Arc<TestPeer<C>>>,
	pub started: bool,
	pub disconnect_events: Vec<(PeerId, PeerId)>, //disconnected (initiated by, to)
}

impl TestNet<TestBlockChainClient> {
	pub fn new(n: usize) -> TestNet<TestBlockChainClient> {
		Self::new_with_config(n, SyncConfig::default())
	}

	pub fn new_with_fork(n: usize, fork: Option<(BlockNumber, H256)>) -> TestNet<TestBlockChainClient> {
		let mut config = SyncConfig::default();
		config.fork_block = fork;
		Self::new_with_config(n, config)
	}

	pub fn new_with_config(n: usize, config: SyncConfig) -> TestNet<TestBlockChainClient> {
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
			disconnect_events: Vec::new(),
		};
		for _ in 0..n {
			let chain = TestBlockChainClient::new();
			let ss = Arc::new(TestSnapshotService::new());
			let sync = ChainSync::new(config.clone(), &chain);
			net.peers.push(Arc::new(TestPeer {
				sync: RwLock::new(sync),
				snapshot_service: ss,
				chain: chain,
				queue: RwLock::new(VecDeque::new()),
			}));
		}
		net
	}
}

impl TestNet<EthcoreClient> {
	pub fn new_with_spec<F>(n: usize, config: SyncConfig, spec_factory: F) -> GuardedTempResult<TestNet<EthcoreClient>>
		where F: Fn() -> Spec
	{
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
			disconnect_events: Vec::new(),
		};
		let dir = devtools::RandomTempPath::new();
		for _ in 0..n {
			let mut client_dir = dir.as_path().clone();
			client_dir.push(devtools::random_filename());

			let db_config = DatabaseConfig::with_columns(NUM_COLUMNS);

			let spec = spec_factory();
			let client = Arc::try_unwrap(EthcoreClient::new(
				ClientConfig::default(),
				&spec,
				client_dir.as_path(),
				Arc::new(Miner::with_spec(&spec)),
				IoChannel::disconnected(),
				&db_config
			).unwrap()).ok().unwrap();

			let ss = Arc::new(TestSnapshotService::new());
			let sync = ChainSync::new(config.clone(), &client);
			let peer = Arc::new(TestPeer {
				sync: RwLock::new(sync),
				snapshot_service: ss,
				chain: client,
				queue: RwLock::new(VecDeque::new()),
			});
			peer.chain.add_notify(peer.clone());
			net.peers.push(peer);
		}
		GuardedTempResult::<TestNet<EthcoreClient>> {
			_temp: dir,
			result: Some(net)
		}
	}
}

impl<C> TestNet<C> where C: FlushingBlockChainClient {
	pub fn peer(&self, i: usize) -> &TestPeer<C> {
		&self.peers[i]
	}

	pub fn peer_mut(&mut self, i: usize) -> &mut TestPeer<C> {
		Arc::get_mut(&mut self.peers[i]).unwrap()
	}

	pub fn start(&mut self) {
		for peer in 0..self.peers.len() {
			for client in 0..self.peers.len() {
				if peer != client {
					let p = &self.peers[peer];
					p.sync.write().update_targets(&p.chain);
					p.sync.write().on_peer_connected(&mut TestIo::new(&p.chain, &p.snapshot_service, &mut p.queue.write(), Some(client as PeerId)), client as PeerId);
				}
			}
		}
	}

	pub fn sync_step(&mut self) {
		for peer in 0..self.peers.len() {
			let packet = self.peers[peer].queue.write().pop_front();
			if let Some(packet) = packet {
				let disconnecting = {
					let p = &self.peers[packet.recipient];
					let mut queue = p.queue.write();
					trace!("--- {} -> {} ---", peer, packet.recipient);
					let to_disconnect = {
						let mut io = TestIo::new(&p.chain, &p.snapshot_service, &mut queue, Some(peer as PeerId));
						ChainSync::dispatch_packet(&p.sync, &mut io, peer as PeerId, packet.packet_id, &packet.data);
						io.to_disconnect
					};
					for d in &to_disconnect {
						// notify this that disconnecting peers are disconnecting
						let mut io = TestIo::new(&p.chain, &p.snapshot_service, &mut queue, Some(*d));
						p.sync.write().on_peer_aborting(&mut io, *d);
						self.disconnect_events.push((peer, *d));
					}
					to_disconnect
				};
				for d in &disconnecting {
					// notify other peers that this peer is disconnecting
					let p = &self.peers[*d];
					let mut queue = p.queue.write();
					let mut io = TestIo::new(&p.chain, &p.snapshot_service, &mut queue, Some(peer as PeerId));
					p.sync.write().on_peer_aborting(&mut io, peer as PeerId);
				}
			}

			self.sync_step_peer(peer);
		}
	}

	pub fn sync_step_peer(&mut self, peer_num: usize) {
		let peer = self.peer(peer_num);
		peer.chain.flush();
		let mut queue = peer.queue.write();
		peer.sync.write().maintain_peers(&mut TestIo::new(&peer.chain, &peer.snapshot_service, &mut queue, None));
		peer.sync.write().maintain_sync(&mut TestIo::new(&peer.chain, &peer.snapshot_service, &mut queue, None));
		peer.sync.write().propagate_new_transactions(&mut TestIo::new(&peer.chain, &peer.snapshot_service, &mut queue, None));
	}

	pub fn restart_peer(&mut self, i: usize) {
		let peer = self.peer(i);
		peer.sync.write().restart(&mut TestIo::new(&peer.chain, &peer.snapshot_service, &mut peer.queue.write(), None));
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
		self.peers.iter().all(|p| p.queue.read().is_empty())
	}

	pub fn trigger_chain_new_blocks(&mut self, peer_id: usize) {
		let peer = self.peer(peer_id);
		let mut queue = peer.queue.write();
		peer.sync.write().chain_new_blocks(&mut TestIo::new(&peer.chain, &peer.snapshot_service, &mut queue, None), &[], &[], &[], &[], &[]);
	}
}

impl ChainNotify for TestPeer<EthcoreClient> {
	fn new_blocks(&self,
		imported: Vec<H256>,
		invalid: Vec<H256>,
		enacted: Vec<H256>,
		retracted: Vec<H256>,
		sealed: Vec<H256>,
		_duration: u64)
	{
		let mut queue = self.queue.write();
		let mut io = TestIo::new(&self.chain, &self.snapshot_service, &mut queue, None);
		self.sync.write().chain_new_blocks(
			&mut io,
			&imported,
			&invalid,
			&enacted,
			&retracted,
			&sealed);
	}

	fn start(&self) {}

	fn stop(&self) {}
}

