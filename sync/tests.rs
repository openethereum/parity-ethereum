use util::*;
use client::{BlockChainClient, BlockStatus, TreeRoute, BlockChainInfo};
use block_queue::BlockQueueInfo;
use header::{Header as BlockHeader, BlockNumber};
use error::*;
use sync::io::SyncIo;
use sync::chain::{ChainSync, SyncState};

struct TestBlockChainClient {
	blocks: RwLock<HashMap<H256, Bytes>>,
 	numbers: RwLock<HashMap<usize, H256>>,
	genesis_hash: H256,
	last_hash: RwLock<H256>,
	difficulty: RwLock<U256>,
}

impl TestBlockChainClient {
	fn new() -> TestBlockChainClient {

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
				uncles.append(&H256::from(&U256::from(n)));
				header.uncles_hash = uncles.as_raw().sha3();
			}
			let mut rlp = RlpStream::new_list(3);
			rlp.append(&header);
			rlp.append_raw(&rlp::NULL_RLP, 1);
			rlp.append_raw(uncles.as_raw(), 1);
			self.import_block(rlp.as_raw().to_vec()).unwrap();
		}
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

	fn tree_route(&self, _from: &H256, _to: &H256) -> Option<TreeRoute> {
		Some(TreeRoute {
			blocks: Vec::new(),
			ancestor: H256::new(),
			index: 0
		})
	}

	fn state_data(&self, _h: &H256) -> Option<Bytes> {
		None
	}

	fn block_receipts(&self, _h: &H256) -> Option<Bytes> {
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
	fn disable_peer(&mut self, _peer_id: PeerId) {
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
	peers: Vec<TestPeer>,
	started: bool
}

impl TestNet {
	pub fn new(n: usize) -> TestNet {
		let mut net = TestNet {
			peers: Vec::new(),
			started: false
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
			p.sync._maintain_sync(&mut TestIo::new(&mut p.chain, &mut p.queue, None));
		}
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

#[test]
fn chain_two_peers() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, false);
	net.peer_mut(2).chain.add_blocks(1000, false);
	net.sync();
	assert!(net.peer(0).chain.block_at(1000).is_some());
	assert_eq!(net.peer(0).chain.blocks.read().unwrap().deref(), net.peer(1).chain.blocks.read().unwrap().deref());
}

#[test]
fn chain_status_after_sync() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, false);
	net.peer_mut(2).chain.add_blocks(1000, false);
	net.sync();
	let status = net.peer(0).sync.status();
	assert_eq!(status.state, SyncState::Idle);
}

#[test]
fn chain_takes_few_steps() {
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(100, false);
	net.peer_mut(2).chain.add_blocks(100, false);
	let total_steps = net.sync();
	assert!(total_steps < 7);
}

#[test]
fn chain_empty_blocks() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	for n in 0..200 {
		net.peer_mut(1).chain.add_blocks(5, n % 2 == 0);
		net.peer_mut(2).chain.add_blocks(5, n % 2 == 0);
	}
	net.sync();
	assert!(net.peer(0).chain.block_at(1000).is_some());
	assert_eq!(net.peer(0).chain.blocks.read().unwrap().deref(), net.peer(1).chain.blocks.read().unwrap().deref());
}

#[test]
fn chain_forged() {
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
	let peer1_chain = net.peer(1).chain.numbers.read().unwrap().clone();
	net.sync();
	assert_eq!(net.peer(0).chain.numbers.read().unwrap().deref(), &peer1_chain);
	assert_eq!(net.peer(1).chain.numbers.read().unwrap().deref(), &peer1_chain);
	assert_eq!(net.peer(2).chain.numbers.read().unwrap().deref(), &peer1_chain);
}

#[test]
fn chain_restart() {
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, false);
	net.peer_mut(2).chain.add_blocks(1000, false);

	net.sync_steps(8);

	// make sure that sync has actually happened
	assert!(net.peer(0).chain.chain_info().best_block_number > 100);
	net.restart_peer(0);

	let status = net.peer(0).sync.status();
	assert_eq!(status.state, SyncState::NotSynced);
}

#[test]
fn chain_status_empty() {
	let net = TestNet::new(2);
	assert_eq!(net.peer(0).sync.status().state, SyncState::NotSynced);
}
