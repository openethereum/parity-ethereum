use std::collections::{HashMap, VecDeque};
use util::bytes::Bytes;
use util::hash::{H256, FixedHash};
use util::uint::{U256};
use util::sha3::Hashable;
use util::rlp::{self, Rlp, RlpStream, View, Stream};
use util::network::{PeerId, PacketId, Error as NetworkError};
use eth::{BlockChainClient, BlockStatus, BlockNumber, TreeRoute, BlockQueueStatus, BlockChainInfo, ImportResult, BlockHeader, QueueStatus};
use sync::{SyncIo};
use sync::chain::{ChainSync};

struct TestBlockChainClient {
	blocks: HashMap<H256, Bytes>,
 	numbers: HashMap<usize, H256>,
	genesis_hash: H256,
	last_hash: H256,
	difficulty: U256
}

impl TestBlockChainClient {
	fn new() -> TestBlockChainClient {

		let mut client = TestBlockChainClient {
			blocks: HashMap::new(),
			numbers: HashMap::new(),
			genesis_hash: H256::new(),
			last_hash: H256::new(),
			difficulty: From::from(0),
		};
		client.add_blocks(1, true); // add genesis block
		client.genesis_hash = client.last_hash;
		client
	}

	pub fn add_blocks(&mut self, count: usize, empty: bool) {
		for n in self.numbers.len()..(self.numbers.len() + count) {
			let mut header = BlockHeader::new();
			header.difficulty = From::from(n);
			header.parent_hash = self.last_hash;
			header.number = From::from(n);
			let mut uncles = RlpStream::new_list(if empty {0} else {1});
			if !empty {
				uncles.append(&H256::from(&U256::from(n)));
				header.uncles_hash = uncles.raw().sha3();
			}
			let mut rlp = RlpStream::new_list(3);
			rlp.append(&header);
			rlp.append_raw(&rlp::NULL_RLP, 1);
			rlp.append_raw(uncles.raw(), 1);
			self.import_block(rlp.raw());
		}
	}
}

impl BlockChainClient for TestBlockChainClient {
	fn block_header(&self, h: &H256) -> Option<Bytes> {
		self.blocks.get(h).map(|r| Rlp::new(r).at(0).raw().to_vec())

	}

	fn block_body(&self, h: &H256) -> Option<Bytes> {
		self.blocks.get(h).map(|r| {
			let mut stream = RlpStream::new_list(2);
			stream.append_raw(Rlp::new(&r).at(1).raw(), 1);
			stream.append_raw(Rlp::new(&r).at(2).raw(), 1);
			stream.out()
		})
	}

	fn block(&self, h: &H256) -> Option<Bytes> {
		self.blocks.get(h).map(|b| b.clone())
	}

	fn block_status(&self, h: &H256) -> BlockStatus {
		match self.blocks.get(h) {
			Some(_) => BlockStatus::InChain,
			None => BlockStatus::Unknown
		}
	}

	fn block_header_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.numbers.get(&(n as usize)).and_then(|h| self.block_header(h))
	}

	fn block_body_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.numbers.get(&(n as usize)).and_then(|h| self.block_body(h))
	}

	fn block_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.numbers.get(&(n as usize)).map(|h| self.blocks.get(h).unwrap().clone())
	}

	fn block_status_at(&self, n: BlockNumber) -> BlockStatus {
		if (n as usize) < self.blocks.len() {
			BlockStatus::InChain
		} else {
			BlockStatus::Unknown
		}
	}

	fn tree_route(&self, _from: &H256, _to: &H256) -> TreeRoute {
		TreeRoute {
			blocks: Vec::new(),
			ancestor: H256::new(),
			index: 0
		}
	}

	fn state_data(&self, _h: &H256) -> Option<Bytes> {
		None
	}

	fn block_receipts(&self, _h: &H256) -> Option<Bytes> {
		None
	}

	fn import_block(&mut self, b: &[u8]) -> ImportResult {
		let header = Rlp::new(&b).val_at::<BlockHeader>(0);
		let number: usize = header.number.low_u64() as usize;
		if number > self.blocks.len() {
			panic!("Unexpected block number. Expected {}, got {}", self.blocks.len(), number);
		}
		if number > 0 {
			match self.blocks.get(&header.parent_hash) {
				Some(parent) => {
					let parent = Rlp::new(parent).val_at::<BlockHeader>(0);
					if parent.number != (header.number - From::from(1)) {
						panic!("Unexpected block parent");
					}
				},
				None => {
					panic!("Unknown block parent {:?} for block {}", header.parent_hash, number);
				}
			}
		}
		if number == self.numbers.len() {
			self.difficulty = self.difficulty + header.difficulty;
			self.last_hash = header.hash();
			self.blocks.insert(header.hash(), b.to_vec());
			self.numbers.insert(number, header.hash());
			let mut parent_hash = header.parent_hash;
			if number > 0 {
				let mut n = number - 1;
				while n > 0 && self.numbers[&n] != parent_hash {
					*self.numbers.get_mut(&n).unwrap() = parent_hash;
					n -= 1;
					parent_hash = Rlp::new(&self.blocks[&parent_hash]).val_at::<BlockHeader>(0).parent_hash;
				}
			}
		}
		else {
			self.blocks.insert(header.hash(), b.to_vec());
		}
		ImportResult::Queued(QueueStatus::Known)
	}

	fn queue_status(&self) -> BlockQueueStatus {
		BlockQueueStatus {
			full: false,
		}
	}

	fn clear_queue(&mut self) {
	}

	fn info(&self) -> BlockChainInfo {
		BlockChainInfo {
			total_difficulty: self.difficulty,
			pending_total_difficulty: self.difficulty,
			genesis_hash: self.genesis_hash,
			last_block_hash: self.last_hash,
			last_block_number: self.blocks.len() as BlockNumber - 1,
		}
	}
}

struct TestIo<'p> {
	chain: &'p mut TestBlockChainClient,
	queue: &'p mut VecDeque<TestPacket>,
	sender: Option<PeerId>,
}

impl<'p> TestIo<'p> {
	fn new(chain: &'p mut TestBlockChainClient, queue: &'p mut VecDeque<TestPacket>, sender: Option<PeerId>) -> TestIo<'p> {
		TestIo {
			chain: chain,
			queue: queue,
			sender: sender
		}
	}
}

impl<'p> SyncIo for TestIo<'p> {
	fn disable_peer(&mut self, _peer_id: &PeerId) {
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

	fn chain<'a>(&'a mut self) -> &'a mut BlockChainClient {
		self.chain
	}
}

struct TestPacket {
	data: Bytes,
	packet_id: PacketId,
	recipient: PeerId,
}

struct TestPeer {
	chain: TestBlockChainClient,
	sync: ChainSync,
	queue: VecDeque<TestPacket>,
}

struct TestNet {
	peers: Vec<TestPeer>
}

impl TestNet {
	pub fn new(n: usize) -> TestNet {
		let mut net = TestNet {
			peers: Vec::new(),
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
					p.sync.on_peer_connected(&mut TestIo::new(&mut p.chain, &mut p.queue, Some(client as PeerId)), &(client as PeerId));
				}
			}
		}
	}

	pub fn sync_step(&mut self) {
		for peer in 0..self.peers.len() {
			match self.peers[peer].queue.pop_front() {
				Some(packet) => {
					let mut p = self.peers.get_mut(packet.recipient).unwrap();
					trace!("--- {} -> {} ---", peer, packet.recipient);
					p.sync.on_packet(&mut TestIo::new(&mut p.chain, &mut p.queue, Some(peer as PeerId)), &(peer as PeerId), packet.packet_id, &packet.data);
					trace!("----------------");
				},
				None => {}
			}
			let mut p = self.peers.get_mut(peer).unwrap();
			p.sync.maintain_sync(&mut TestIo::new(&mut p.chain, &mut p.queue, None));
		}
	}

	pub fn sync(&mut self) {
		self.start();
		while !self.done() {
			self.sync_step()
		}
	}

	pub fn done(&self) -> bool {
		self.peers.iter().all(|p| p.queue.is_empty())
	}
}


#[test]
fn full_sync_two_peers() {
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, false);
	net.peer_mut(2).chain.add_blocks(1000, false);
	net.sync();
	assert!(net.peer(0).chain.block_at(1000).is_some());
	assert_eq!(net.peer(0).chain.blocks, net.peer(1).chain.blocks);
}

#[test]
fn full_sync_empty_blocks() {
	let mut net = TestNet::new(3);
	for n in 0..200 {
		net.peer_mut(1).chain.add_blocks(5, n % 2 == 0);
		net.peer_mut(2).chain.add_blocks(5, n % 2 == 0);
	}
	net.sync();
	assert!(net.peer(0).chain.block_at(1000).is_some());
	assert_eq!(net.peer(0).chain.blocks, net.peer(1).chain.blocks);
}

#[test]
fn forked_sync() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(0).chain.add_blocks(300, false);
	net.peer_mut(1).chain.add_blocks(300, false);
	net.peer_mut(2).chain.add_blocks(300, false);
	net.peer_mut(0).chain.add_blocks(100, true); //fork
	net.peer_mut(1).chain.add_blocks(200, false);
	net.peer_mut(2).chain.add_blocks(200, false);
	net.peer_mut(1).chain.add_blocks(100, false); //fork between 1 and 2
	net.peer_mut(2).chain.add_blocks(10, true);
	// peer 1 has the best chain of 601 blocks
	let peer1_chain = net.peer(1).chain.numbers.clone();
	net.sync();
	assert_eq!(net.peer(0).chain.numbers, peer1_chain);
	assert_eq!(net.peer(1).chain.numbers, peer1_chain);
	assert_eq!(net.peer(2).chain.numbers, peer1_chain);
}
