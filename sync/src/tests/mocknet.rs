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
use ethcore::client::{Client, BlockChainClient, ChainNotify};
use ethcore::spec::Spec;
use ethcore::snapshot::SnapshotService;
use ethcore::transaction::{Transaction, SignedTransaction, Action};
use ethcore::account_provider::AccountProvider;
use ethkey::{Random, Generator};
use sync_io::SyncIo;
use chain::ChainSync;
use ::SyncConfig;
use devtools::RandomTempPath;
use ethcore::miner::Miner;
use ethcore::service::ClientService;
use ethcore::header::BlockNumber;
use std::time::Duration;
use std::thread::sleep;
use rand::{thread_rng, Rng};

pub struct TestIo<'p> {
	pub client: Arc<Client>,
	pub snapshot_service: &'p TestSnapshotService,
	pub queue: &'p mut VecDeque<TestPacket>,
	pub sender: Option<PeerId>,
}

impl<'p> TestIo<'p> {
	pub fn new(client: Arc<Client>, ss: &'p TestSnapshotService, queue: &'p mut VecDeque<TestPacket>, sender: Option<PeerId>) -> TestIo<'p> {
		TestIo {
			client: client,
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
		self.client.as_ref()
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

fn transaction() -> Transaction {
	Transaction {
		action: Action::Create,
		value: U256::zero(),
		data: "3331600055".from_hex().unwrap(),
		gas: U256::from(100_000),
		gas_price: U256::zero(),
		nonce: U256::zero(),
	}
}

pub fn random_transaction() -> SignedTransaction {
	let keypair = Random.generate().unwrap();
	transaction().sign(&keypair.secret())
}

pub struct MockPeer {
	pub client: Arc<Client>,
	pub snapshot_service: Arc<TestSnapshotService>,
	pub sync: RwLock<ChainSync>,
	pub queue: RwLock<VecDeque<TestPacket>>,
	_service: ClientService,
	_paths: Vec<RandomTempPath>,
}

impl ChainNotify for MockPeer {
	fn new_blocks(&self,
		imported: Vec<H256>,
		invalid: Vec<H256>,
		enacted: Vec<H256>,
		retracted: Vec<H256>,
		sealed: Vec<H256>,
		_duration: u64) {
		println!("New sync blocks");
		let ref mut q = *self.queue.write();
		let mut sync_io = TestIo::new(
			self.client.clone(),
			&self.snapshot_service,
			q,
			None);
		self.sync.write().chain_new_blocks(
			&mut sync_io,
			&imported,
			&invalid,
			&enacted,
			&retracted,
			&sealed);
	}
}

impl MockPeer {
	pub fn new_with_spec<S>(get_spec: &S, author_secret: Option<Secret>) -> Arc<MockPeer> where S: Fn()->Spec {
		let (accounts, address) = if let Some(secret) = author_secret {
			let tap = AccountProvider::transient_provider();
			let addr = tap.insert_account(secret, "").unwrap();
			tap.unlock_account_permanently(addr, "".into()).unwrap();
			(Some(Arc::new(tap)), Some(addr))
		} else {
			(None, None)
		};

		let client_path = RandomTempPath::new();
		let snapshot_path = RandomTempPath::new();
		let ipc_path = RandomTempPath::new();
		let spec = get_spec();

		let service = ClientService::start(
			Default::default(),
			&spec,
			client_path.as_path(),
			snapshot_path.as_path(),
			ipc_path.as_path(),
			Arc::new(Miner::with_spec_and_accounts(&spec, accounts.clone())),
		).unwrap();

		let client = service.client();
		if let Some(addr) = address { client.set_author(addr) }
		let sync = ChainSync::new(SyncConfig::default(), &*client);

		let peer = Arc::new(MockPeer {
			sync: RwLock::new(sync),
			snapshot_service: Arc::new(TestSnapshotService::new()),
			client: client,
			queue: RwLock::new(VecDeque::new()),
			_service: service,
			_paths: vec![client_path, snapshot_path, ipc_path]
		});
		peer.client.add_notify(peer.clone());
		peer
	}

	pub fn issue_tx(&self, transaction: SignedTransaction) {
		self.client.import_own_transaction(transaction).unwrap();
	}

	pub fn issue_rand_tx(&self) {
		self.issue_tx(random_transaction())
	}

	pub fn issue_rand_txs(&self, n: usize) {
		for _ in 0..n {
			self.issue_rand_tx();
		}
	}
}

pub struct MockNet {
	pub peers: Vec<Arc<MockPeer>>,
	pub started: bool,
}

impl MockNet {
	pub fn new_with_spec<S>(nodes: usize, author_secrets: Vec<H256>, get_spec: &S) -> MockNet where S: Fn()->Spec {
		let mut net = MockNet {
			peers: Vec::new(),
			started: false,
		};
		for secret in author_secrets {
			net.peers.push(MockPeer::new_with_spec(get_spec, Some(secret)));
		}
		for _ in net.peers.len()..nodes {
			net.peers.push(MockPeer::new_with_spec(get_spec, None));
		}
		net
	}

	pub fn peer(&self, i: usize) -> Arc<MockPeer> {
		self.peers.get(i).unwrap().clone()
	}

	pub fn peer_mut(&mut self, i: usize) -> &mut MockPeer {
		Arc::get_mut(self.peers.get_mut(i).unwrap()).unwrap()
	}

	pub fn start(&mut self) {
		for peer in 0..self.peers.len() {
			for client in 0..self.peers.len() {
				if peer != client {
					let p = self.peers.get_mut(peer).unwrap();
					let mut q = p.queue.write();
					p.sync.write().on_peer_connected(&mut TestIo::new(p.client.clone(),
													 &p.snapshot_service,
													 &mut *q,
													 Some(client as PeerId)),
													 client as PeerId);
				}
			}
		}
	}

	pub fn sync_step(&mut self) {
		for (i, peer0) in self.peers.iter().enumerate() {
			let mut q0 = peer0.queue.write();
			if let Some(packet) = q0.pop_front() {
				let p = self.peers.get(packet.recipient).unwrap();
				let mut q1 = p.queue.write();
				trace!(target: "mocknet", "--- {} -> {} ---", i, packet.recipient);
				ChainSync::dispatch_packet(&p.sync,
										   &mut TestIo::new(p.client.clone(),
										   &p.snapshot_service,
										   &mut *q1,
										   Some(i as PeerId)),
										   i as PeerId,
										   packet.packet_id,
										   &packet.data);
				trace!(target: "mocknet", "----------------");
			}
			let p = self.peers.get(i).unwrap();
			peer0.client.flush_queue();
			let mut io = TestIo::new(peer0.client.clone(), &peer0.snapshot_service, &mut *q0, None);
			p.sync.write().maintain_sync(&mut io);
			p.sync.write().propagate_new_transactions(&mut io);
			sleep(Duration::from_millis(10));
		}
	}

	pub fn sync_step_peer(&mut self, peer_num: usize) {
		let mut peer = self.peer_mut(peer_num);
		let ref mut q = *peer.queue.write();
		peer.sync.write().maintain_sync(&mut TestIo::new(peer.client.clone(), &peer.snapshot_service, q, None));
	}

	pub fn restart_peer(&mut self, i: usize) {
		let peer = self.peer_mut(i);
		let ref mut q = *peer.queue.write();
		peer.sync.write().restart(&mut TestIo::new(peer.client.clone(), &peer.snapshot_service, q, None));
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
		self.peers.iter().all(|p| p.queue.try_read().unwrap().is_empty())
	}

	pub fn rand_peer(&self) -> Arc<MockPeer> {
		thread_rng().choose(&self.peers).unwrap().clone()
	}

	pub fn rand_simulation(&mut self, steps: usize) {
		for _ in 0..steps {
			self.rand_peer().issue_rand_tx();
			sleep(Duration::from_millis(500));
			self.sync();
		}
	}

	pub fn is_synced(&self, block: BlockNumber) {
		let hash = self.peer(0).client.chain_info().best_block_hash;
		for p in &self.peers {
			let ci = p.client.chain_info();
			assert_eq!(ci.best_block_number, block);
			assert_eq!(ci.best_block_hash, hash);
		}
	}
}
