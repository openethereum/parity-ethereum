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
use ethcore::client::{Client, BlockChainClient};
use ethcore::tests::helpers::*;
use ethcore::header::BlockNumber;
use ethcore::spec::Spec;
use ethcore::snapshot::SnapshotService;
use ethcore::transaction::{Transaction, SignedTransaction, Action};
use ethkey::{Random, Generator};
use sync_io::SyncIo;
use chain::ChainSync;
use ::SyncConfig;

pub struct TestIo<'p> {
	pub chain: &'p mut Client,
	pub snapshot_service: &'p TestSnapshotService,
	pub queue: &'p mut VecDeque<TestPacket>,
	pub sender: Option<PeerId>,
}

impl<'p> TestIo<'p> {
	pub fn new(chain: &'p mut Arc<Client>, ss: &'p TestSnapshotService, queue: &'p mut VecDeque<TestPacket>, sender: Option<PeerId>) -> TestIo<'p> {
		TestIo {
			chain: Arc::get_mut(chain).unwrap(),
			snapshot_service: ss,
			queue: queue,
			sender: sender
		}
	}
}

impl<'p> SyncIo for TestIo<'p> {
	fn disable_peer(&mut self, _peer_id: PeerId) {
	}

	fn disconnect_peer(&mut self, _peer_id: PeerId) {
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

	fn eth_protocol_version(&self, _peer: PeerId) -> u8 {
		64
	}
}

pub struct TestPacket {
	pub data: Bytes,
	pub packet_id: PacketId,
	pub recipient: PeerId,
}


pub fn random_transaction() -> SignedTransaction {
	let keypair = Random.generate().unwrap();
	Transaction {
		action: Action::Create,
		value: U256::zero(),
		data: "3331600055".from_hex().unwrap(),
		gas: U256::from(100_000),
		gas_price: U256::zero(),
		nonce: U256::zero(),
	}.sign(keypair.secret())
}

pub struct TestPeer {
	pub chain: Arc<Client>,
	pub snapshot_service: Arc<TestSnapshotService>,
	pub sync: RwLock<ChainSync>,
	pub queue: VecDeque<TestPacket>,
}

impl TestPeer {
	pub fn issue_tx(&self, transaction: SignedTransaction) {
		self.chain.import_own_transaction(transaction);
	}

	pub fn issue_rand_tx(&self) {
		self.issue_tx(random_transaction())
	}
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
			let mut chain = generate_dummy_client(1);
			let mut config = SyncConfig::default();
			config.fork_block = fork;
			let ss = Arc::new(TestSnapshotService::new());
			let sync = ChainSync::new(config, chain.reference().as_ref());
			net.peers.push(TestPeer {
				sync: RwLock::new(sync),
				snapshot_service: ss,
				chain: chain.take(),
				queue: VecDeque::new(),
			});
		}
		net
	}

	pub fn new_with_spec<S>(n: usize, get_spec: &S) -> TestNet where S: Fn()->Spec {
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
		};
		for _ in 0..n {
			let mut chain = generate_dummy_client_with_spec(get_spec, 1);
			let sync = ChainSync::new(SyncConfig::default(), chain.reference().as_ref());
			net.peers.push(TestPeer {
				sync: RwLock::new(sync),
				snapshot_service: Arc::new(TestSnapshotService::new()),
				chain: chain.take(),
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
					p.sync.write().on_peer_connected(&mut TestIo::new(&mut p.chain,
													 &p.snapshot_service,
													 &mut p.queue,
													 Some(client as PeerId)),
													 client as PeerId);
				}
			}
		}
	}

	pub fn sync_step(&mut self) {
		for peer in 0..self.peers.len() {
			if let Some(packet) = self.peers[peer].queue.pop_front() {
				let mut p = self.peers.get_mut(packet.recipient).unwrap();
				trace!("--- {} -> {} ---", peer, packet.recipient);
				ChainSync::dispatch_packet(&p.sync,
										   &mut TestIo::new(&mut p.chain,
										   &p.snapshot_service,
										   &mut p.queue,
										   Some(peer as PeerId)),
										   peer as PeerId,
										   packet.packet_id,
										   &packet.data);
				trace!("----------------");
			}
			let mut p = self.peers.get_mut(peer).unwrap();
			p.sync.write().maintain_sync(&mut TestIo::new(&mut p.chain, &p.snapshot_service, &mut p.queue, None));
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
