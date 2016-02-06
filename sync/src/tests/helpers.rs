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
use ethcore::client::{BlockChainClient, BlockStatus, TreeRoute, BlockChainInfo};
use ethcore::block_queue::BlockQueueInfo;
use ethcore::header::{Header as BlockHeader, BlockNumber};
use ethcore::error::*;
use io::SyncIo;
use chain::{ChainSync};
use ethcore::receipt::Receipt;

pub struct TestBlockChainClient {
	pub blocks: RwLock<HashMap<H256, Bytes>>,
 	pub numbers: RwLock<HashMap<usize, H256>>,
	pub genesis_hash: H256,
	pub last_hash: RwLock<H256>,
	pub difficulty: RwLock<U256>,
}

impl TestBlockChainClient {
	pub fn new() -> TestBlockChainClient {

		let mut client = TestBlockChainClient {
			blocks: RwLock::new(HashMap::new()),
			numbers: RwLock::new(HashMap::new()),
			genesis_hash: H256::new(),
			last_hash: RwLock::new(H256::new()),
			difficulty: RwLock::new(From::from(0)),
		};
		client.add_blocks(1, true); // add genesis block
		client.genesis_hash = client.last_hash.read().unwrap().clone();
		client
	}

	pub fn add_blocks(&mut self, count: usize, empty: bool) {
		let len = self.numbers.read().unwrap().len();
		for n in len..(len + count) {
			let mut header = BlockHeader::new();
			header.difficulty = From::from(n);
			header.parent_hash = self.last_hash.read().unwrap().clone();
			header.number = n as BlockNumber;
			let mut uncles = RlpStream::new_list(if empty {0} else {1});
			if !empty {
				let mut uncle_header = BlockHeader::new();
				uncle_header.difficulty = From::from(n);
				uncle_header.parent_hash = self.last_hash.read().unwrap().clone();
				uncle_header.number = n as BlockNumber;
				uncles.append(&uncle_header);
				header.uncles_hash = uncles.as_raw().sha3();
			}
			let mut rlp = RlpStream::new_list(3);
			rlp.append(&header);
			rlp.append_raw(&rlp::NULL_RLP, 1);
			rlp.append_raw(uncles.as_raw(), 1);
			self.import_block(rlp.as_raw().to_vec()).unwrap();
		}
	}

	pub fn block_hash_delta_minus(&mut self, delta: usize) -> H256 {
		let blocks_read = self.numbers.read().unwrap();
		let index = blocks_read.len() - delta;
		blocks_read[&index].clone()
	}
}

impl BlockChainClient for TestBlockChainClient {
	fn block_total_difficulty(&self, _h: &H256) -> Option<U256> {
		unimplemented!();
	}

	fn block_header(&self, h: &H256) -> Option<Bytes> {
		self.blocks.read().unwrap().get(h).map(|r| Rlp::new(r).at(0).as_raw().to_vec())
	}

	fn block_body(&self, h: &H256) -> Option<Bytes> {
		self.blocks.read().unwrap().get(h).map(|r| {
			let mut stream = RlpStream::new_list(2);
			stream.append_raw(Rlp::new(&r).at(1).as_raw(), 1);
			stream.append_raw(Rlp::new(&r).at(2).as_raw(), 1);
			stream.out()
		})
	}

	fn block(&self, h: &H256) -> Option<Bytes> {
		self.blocks.read().unwrap().get(h).cloned()
	}

	fn block_status(&self, h: &H256) -> BlockStatus {
		match self.blocks.read().unwrap().get(h) {
			Some(_) => BlockStatus::InChain,
			None => BlockStatus::Unknown
		}
	}

	fn block_total_difficulty_at(&self, _number: BlockNumber) -> Option<U256> {
		unimplemented!();
	}

	fn block_header_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.numbers.read().unwrap().get(&(n as usize)).and_then(|h| self.block_header(h))
	}

	fn block_body_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.numbers.read().unwrap().get(&(n as usize)).and_then(|h| self.block_body(h))
	}

	fn block_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.numbers.read().unwrap().get(&(n as usize)).map(|h| self.blocks.read().unwrap().get(h).unwrap().clone())
	}

	fn block_status_at(&self, n: BlockNumber) -> BlockStatus {
		if (n as usize) < self.blocks.read().unwrap().len() {
			BlockStatus::InChain
		} else {
			BlockStatus::Unknown
		}
	}

	// works only if blocks are one after another 1 -> 2 -> 3
	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute> {
		Some(TreeRoute {
			ancestor: H256::new(),
			index: 0,
			blocks: {
				let numbers_read = self.numbers.read().unwrap();
				let mut adding = false;

				let mut blocks = Vec::new();
				for (_, hash) in numbers_read.iter().sort_by(|tuple1, tuple2| tuple1.0.cmp(tuple2.0)) {
					if hash == to {
						if adding {
							blocks.push(hash.clone());
						}
						adding = false;
						break;
					}
					if hash == from {
						adding = true;
					}
					if adding {
						blocks.push(hash.clone());
					}
				}
				if adding { Vec::new() } else { blocks }
			}
		})
	}

	// TODO: returns just hashes instead of node state rlp(?)
	fn state_data(&self, hash: &H256) -> Option<Bytes> {
		// starts with 'f' ?
		if *hash > H256::from("f000000000000000000000000000000000000000000000000000000000000000") {
			let mut rlp = RlpStream::new();
			rlp.append(&hash.clone());
			return Some(rlp.out());
		}
		None
	}

	fn block_receipts(&self, hash: &H256) -> Option<Bytes> {
		// starts with 'f' ?
		if *hash > H256::from("f000000000000000000000000000000000000000000000000000000000000000") {
			let receipt = Receipt::new(
				H256::zero(),
				U256::zero(),
				vec![]);
			let mut rlp = RlpStream::new();
			rlp.append(&receipt);
			return Some(rlp.out());
		}
		None
	}

	fn import_block(&self, b: Bytes) -> ImportResult {
		let header = Rlp::new(&b).val_at::<BlockHeader>(0);
		let h = header.hash();
		let number: usize = header.number as usize;
		if number > self.blocks.read().unwrap().len() {
			panic!("Unexpected block number. Expected {}, got {}", self.blocks.read().unwrap().len(), number);
		}
		if number > 0 {
			match self.blocks.read().unwrap().get(&header.parent_hash) {
				Some(parent) => {
					let parent = Rlp::new(parent).val_at::<BlockHeader>(0);
					if parent.number != (header.number - 1) {
						panic!("Unexpected block parent");
					}
				},
				None => {
					panic!("Unknown block parent {:?} for block {}", header.parent_hash, number);
				}
			}
		}
		let len = self.numbers.read().unwrap().len();
		if number == len {
			*self.difficulty.write().unwrap().deref_mut() += header.difficulty;
			mem::replace(self.last_hash.write().unwrap().deref_mut(), h.clone());
			self.blocks.write().unwrap().insert(h.clone(), b);
			self.numbers.write().unwrap().insert(number, h.clone());
			let mut parent_hash = header.parent_hash;
			if number > 0 {
				let mut n = number - 1;
				while n > 0 && self.numbers.read().unwrap()[&n] != parent_hash {
					*self.numbers.write().unwrap().get_mut(&n).unwrap() = parent_hash.clone();
					n -= 1;
					parent_hash = Rlp::new(&self.blocks.read().unwrap()[&parent_hash]).val_at::<BlockHeader>(0).parent_hash;
				}
			}
		}
		else {
			self.blocks.write().unwrap().insert(h.clone(), b.to_vec());
		}
		Ok(h)
	}

	fn queue_info(&self) -> BlockQueueInfo {
		BlockQueueInfo {
			full: false,
			verified_queue_size: 0,
			unverified_queue_size: 0,
			verifying_queue_size: 0,
			empty: false,
		}
	}

	fn clear_queue(&self) {
	}

	fn chain_info(&self) -> BlockChainInfo {
		BlockChainInfo {
			total_difficulty: *self.difficulty.read().unwrap(),
			pending_total_difficulty: *self.difficulty.read().unwrap(),
			genesis_hash: self.genesis_hash.clone(),
			best_block_hash: self.last_hash.read().unwrap().clone(),
			best_block_number: self.blocks.read().unwrap().len() as BlockNumber - 1,
		}
	}
}

pub struct TestIo<'p> {
	pub chain: &'p mut TestBlockChainClient,
	pub queue: &'p mut VecDeque<TestPacket>,
	pub sender: Option<PeerId>,
}

impl<'p> TestIo<'p> {
	pub fn new(chain: &'p mut TestBlockChainClient, queue: &'p mut VecDeque<TestPacket>, sender: Option<PeerId>) -> TestIo<'p> {
		TestIo {
			chain: chain,
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

	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		self.queue.push_back(TestPacket {
			data: data,
			packet_id: packet_id,
			recipient: self.sender.unwrap()
		});
		Ok(())
	}

	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
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
}

pub struct TestPacket {
	pub data: Bytes,
	pub packet_id: PacketId,
	pub recipient: PeerId,
}

pub struct TestPeer {
	pub chain: TestBlockChainClient,
	pub sync: ChainSync,
	pub queue: VecDeque<TestPacket>,
}

pub struct TestNet {
	pub peers: Vec<TestPeer>,
	pub started: bool,
}

impl TestNet {
	pub fn new(n: usize) -> TestNet {
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
		};
		for _ in 0..n {
			net.peers.push(TestPeer {
				chain: TestBlockChainClient::new(),
				sync: ChainSync::new(),
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
					p.sync.on_peer_connected(&mut TestIo::new(&mut p.chain, &mut p.queue, Some(client as PeerId)), client as PeerId);
				}
			}
		}
	}

	pub fn sync_step(&mut self) {
		for peer in 0..self.peers.len() {
			if let Some(packet) = self.peers[peer].queue.pop_front() {
				let mut p = self.peers.get_mut(packet.recipient).unwrap();
				trace!("--- {} -> {} ---", peer, packet.recipient);
				p.sync.on_packet(&mut TestIo::new(&mut p.chain, &mut p.queue, Some(peer as PeerId)), peer as PeerId, packet.packet_id, &packet.data);
				trace!("----------------");
			}
			let mut p = self.peers.get_mut(peer).unwrap();
			p.sync.maintain_sync(&mut TestIo::new(&mut p.chain, &mut p.queue, None));
		}
	}

	pub fn sync_step_peer(&mut self, peer_num: usize) {
		let mut peer = self.peer_mut(peer_num);
		peer.sync.maintain_sync(&mut TestIo::new(&mut peer.chain, &mut peer.queue, None));
	}

	pub fn restart_peer(&mut self, i: usize) {
		let peer = self.peer_mut(i);
		peer.sync.restart(&mut TestIo::new(&mut peer.chain, &mut peer.queue, None));
	}

	pub fn sync(&mut self) -> u32 {
		self.start();
		let mut total_steps = 0;
		while !self.done() {
			self.sync_step();
			total_steps = total_steps + 1;
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
}
