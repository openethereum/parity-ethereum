use std::collections::{HashSet, HashMap};
use std::cmp::{min, max};
use std::mem::{replace};
use util::network::{PeerId, PacketId};
use util::hash::{H256, FixedHash};
use util::bytes::{Bytes};
use util::uint::{U256};
use util::rlp::{Rlp, RlpStream, self}; //TODO: use UntrustedRlp
use util::rlp::rlptraits::{Stream, View};
use util::sha3::Hashable;
use client::{BlockNumber, BlockChainClient, BlockStatus, QueueStatus, ImportResult};
use views::{HeaderView};
use sync::range_collection::{RangeCollection, ToUsize, FromUsize};
use sync::{SyncIo};

impl ToUsize for BlockNumber {
	fn to_usize(&self) -> usize {
		*self as usize
	}
}

impl FromUsize for BlockNumber {
	fn from_usize(s: usize) -> BlockNumber {
		s as BlockNumber
	}
}

const PROTOCOL_VERSION: u8 = 63u8;
const MAX_BODIES_TO_SEND: usize = 256;
const MAX_HEADERS_TO_SEND: usize = 512;
const MAX_NODE_DATA_TO_SEND: usize = 1024;
const MAX_RECEIPTS_TO_SEND: usize = 1024;
const MAX_HEADERS_TO_REQUEST: usize = 512;
const MAX_BODIES_TO_REQUEST: usize = 256;

const STATUS_PACKET: u8 = 0x00;
const NEW_BLOCK_HASHES_PACKET: u8 = 0x01;
const TRANSACTIONS_PACKET: u8 = 0x02;
const GET_BLOCK_HEADERS_PACKET: u8 = 0x03;
const BLOCK_HEADERS_PACKET: u8 = 0x04;
const GET_BLOCK_BODIES_PACKET: u8 = 0x05;
const BLOCK_BODIES_PACKET: u8 = 0x06;
const NEW_BLOCK_PACKET: u8 = 0x07;

const GET_NODE_DATA_PACKET: u8 = 0x0d;
const NODE_DATA_PACKET: u8 = 0x0e;
const GET_RECEIPTS_PACKET: u8 = 0x0f;
const RECEIPTS_PACKET: u8 = 0x10;

struct Header {
	///Header data
	data: Bytes,
	/// Block hash
	hash: H256,
	/// Parent hash
	parent: H256,
}

/// Used to identify header by transactions and uncles hashes
#[derive(Eq, PartialEq, Hash)]
struct HeaderId {
	transactions_root: H256,
	uncles: H256
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SyncState {
	/// Initial chain sync has not started yet
	NotSynced,
	/// Initial chain sync complete. Waiting for new packets
	Idle,
	/// Block downloading paused. Waiting for block queue to process blocks and free space
	Waiting,
	/// Downloading blocks
	Blocks,
	/// Downloading blocks learned from NewHashes packet
	NewBlocks,
}

pub struct SyncStatus {
	pub state: SyncState,
	pub protocol_version: u8,
	pub start_block_number: BlockNumber,
	pub last_imported_block_number: BlockNumber,
	pub highest_block_number: BlockNumber,
	pub blocks_total: usize,
	pub blocks_received: usize,
}

#[derive(PartialEq, Eq, Debug)]
enum PeerAsking {
	Nothing,
	BlockHeaders,
	BlockBodies,
}

struct PeerInfo {
	protocol_version: u32,
	genesis: H256,
	network_id: U256,
	latest: H256,
	difficulty: U256,
	asking: PeerAsking,
	asking_blocks: Vec<BlockNumber>,
}

type Body = Bytes;

pub struct ChainSync {
	/// Sync state
	state: SyncState,
	/// Last block number for the start of sync
	starting_block: BlockNumber,
	/// Highest block number seen
	highest_block: BlockNumber,
	/// Set of block header numbers being downloaded
	downloading_headers: HashSet<BlockNumber>,
	/// Set of block body numbers being downloaded
	downloading_bodies: HashSet<BlockNumber>,
	/// Downloaded headers.
	headers: Vec<(BlockNumber, Vec<Header>)>, //TODO: use BTreeMap once range API is sable. For now it is a vector sorted in descending order
	/// Downloaded bodies
	bodies: Vec<(BlockNumber, Vec<Body>)>, //TODO: use BTreeMap once range API is sable. For now it is a vector sorted in descending order
	/// Peer info
	peers: HashMap<PeerId, PeerInfo>,
	/// Used to map body to header
	header_ids: HashMap<HeaderId, BlockNumber>,
	/// Last impoted block number
	last_imported_block: BlockNumber,
	/// Last impoted block hash
	last_imported_hash: H256,
	/// Syncing total  difficulty
	syncing_difficulty: U256,
	/// True if common block for our and remote chain has been found
	have_common_block: bool,
}


impl ChainSync {
	pub fn new() -> ChainSync {
		ChainSync {
			state: SyncState::NotSynced,
			starting_block: 0,
			highest_block: 0,
			downloading_headers: HashSet::new(),
			downloading_bodies: HashSet::new(),
			headers: Vec::new(),
			bodies: Vec::new(),
			peers: HashMap::new(),
			header_ids: HashMap::new(),
			last_imported_block: 0,
			last_imported_hash: H256::new(),
			syncing_difficulty: U256::from(0u64),
			have_common_block: false,
		}
	}

	/// @returns Synchonization status
	pub fn status(&self) -> SyncStatus {
		SyncStatus {
			state: self.state.clone(),
			protocol_version: 63,
			start_block_number: self.starting_block,
			last_imported_block_number: self.last_imported_block,
			highest_block_number: self.highest_block,
			blocks_total: (self.last_imported_block - self.starting_block) as usize,
			blocks_received: (self.highest_block - self.starting_block) as usize,
		}
	}

	/// Abort all sync activity
	pub fn abort(&mut self, io: &mut SyncIo) {
		self.restart(io);
		self.peers.clear();
	}

	/// @returns true is Sync is in progress
	pub fn is_syncing(&self) -> bool {
		self.state != SyncState::Idle
	}

	fn reset(&mut self) {
		self.downloading_headers.clear();
		self.downloading_bodies.clear();
		self.headers.clear();
		self.bodies.clear();
		for (_, ref mut p) in self.peers.iter_mut() {
			p.asking_blocks.clear();
		}
		self.header_ids.clear();
		self.syncing_difficulty = From::from(0u64);
		self.state = SyncState::Idle;
	}

	/// Restart sync
	pub fn restart(&mut self, io: &mut SyncIo) {
		self.reset();
		self.last_imported_block = 0;
		self.last_imported_hash = H256::new();
		self.starting_block = 0;
		self.highest_block = 0;
		self.have_common_block = false;
		io.chain().clear_queue();
		self.starting_block = io.chain().chain_info().best_block_number;
		self.state = SyncState::NotSynced;
	}

	/// Called by peer to report status
	fn on_peer_status(&mut self, io: &mut SyncIo, peer_id: &PeerId, r: &Rlp) {
		let peer = PeerInfo {
			protocol_version: r.val_at(0),
			network_id: r.val_at(1),
			difficulty: r.val_at(2),
			latest: r.val_at(3),
			genesis: r.val_at(4),
			asking: PeerAsking::Nothing,
			asking_blocks: Vec::new(),
		};

		trace!(target: "sync", "New peer (protocol: {}, network: {:?}, difficulty: {:?}, latest:{}, genesis:{})", peer.protocol_version, peer.network_id, peer.difficulty, peer.latest, peer.genesis);

		let old = self.peers.insert(peer_id.clone(), peer);
		if old.is_some() {
			panic!("ChainSync: new peer already exists");
		}
		self.sync_peer(io, peer_id, false);
	}

	/// Called by peer once it has new block headers during sync
	fn on_peer_block_headers(&mut self, io: &mut SyncIo, peer_id: &PeerId, r: &Rlp) {
		self.reset_peer_asking(peer_id, PeerAsking::BlockHeaders);
		let item_count = r.item_count();
		trace!(target: "sync", "{} -> BlockHeaders ({} entries)", peer_id, item_count);
		self.clear_peer_download(peer_id);
		if self.state != SyncState::Blocks && self.state != SyncState::NewBlocks && self.state != SyncState::Waiting {
			trace!(target: "sync", "Ignored unexpected block headers");
			return;
		}
		if self.state == SyncState::Waiting {
			trace!(target: "sync", "Ignored block headers while waiting");
			return;
		}

		for i in 0..item_count {
			let info = HeaderView::new_from_rlp(r.at(i));
			let number = BlockNumber::from(info.number());
			if number <= self.last_imported_block || self.headers.have_item(&number) {
				trace!(target: "sync", "Skipping existing block header");
				continue;
			}
			if number > self.highest_block {
				self.highest_block = number;
			}
			let hash = info.sha3();
			match io.chain().block_status(&hash) {
				BlockStatus::InChain => {
					self.have_common_block = true;
					self.last_imported_block = number;
					self.last_imported_hash = hash.clone();
					trace!(target: "sync", "Found common header {} ({})", number, hash);
				},
				_ => {
					if self.have_common_block {
						//validate chain
						if self.have_common_block && number == self.last_imported_block + 1 && info.parent_hash() != self.last_imported_hash {
							// TODO: lower peer rating
							debug!(target: "sync", "Mismatched block header {} {}", number, hash);
							continue;
						}
						if self.headers.find_item(&(number - 1)).map_or(false, |p| p.hash != info.parent_hash()) {
							// mismatching parent id, delete the previous block and don't add this one
							// TODO: lower peer rating
							debug!(target: "sync", "Mismatched block header {} {}", number, hash);
							self.remove_downloaded_blocks(number - 1);
							continue;
						}
						if self.headers.find_item(&(number + 1)).map_or(false, |p| p.parent != hash) {
							// mismatching parent id for the next block, clear following headers
							debug!(target: "sync", "Mismatched block header {}", number + 1);
							self.remove_downloaded_blocks(number + 1);
						}
					}
					let hdr = Header {
						data: r.at(i).as_raw().to_vec(),
						hash: hash.clone(),
						parent: info.parent_hash(),
					};
					self.headers.insert_item(number, hdr);
					let header_id = HeaderId {
						transactions_root: info.transactions_root(),
						uncles: info.uncles_hash()
					};
					trace!(target: "sync", "Got header {} ({})", number, hash);
					if header_id.transactions_root == rlp::SHA3_NULL_RLP && header_id.uncles == rlp::SHA3_EMPTY_LIST_RLP {
						//empty body, just mark as downloaded
						let mut body_stream = RlpStream::new_list(2);
						body_stream.append_raw(&rlp::NULL_RLP, 1);
						body_stream.append_raw(&rlp::EMPTY_LIST_RLP, 1);
						self.bodies.insert_item(number, body_stream.out());
					}
					else {
						self.header_ids.insert(header_id, number);
					}
				}
			}
		}
		self.collect_blocks(io);
		self.continue_sync(io);
	}

	/// Called by peer once it has new block bodies
	fn on_peer_block_bodies(&mut self, io: &mut SyncIo, peer_id: &PeerId, r: &Rlp) {
		self.reset_peer_asking(peer_id, PeerAsking::BlockBodies);
		let item_count = r.item_count();
		trace!(target: "sync", "{} -> BlockBodies ({} entries)", peer_id, item_count);
		self.clear_peer_download(peer_id);
		if self.state != SyncState::Blocks && self.state != SyncState::NewBlocks && self.state != SyncState::Waiting {
			trace!(target: "sync", "Ignored unexpected block bodies");
			return;
		}
		if self.state  == SyncState::Waiting {
			trace!(target: "sync", "Ignored block bodies while waiting");
			return;
		}
		for i in 0..item_count {
			let body: Rlp = r.at(i);
			let tx = body.at(0);
			let tx_root = ::util::triehash::ordered_trie_root(tx.iter().map(|r| r.as_raw().to_vec()).collect()); //TODO: get rid of vectors here
			let uncles = body.at(1).as_raw().sha3();
			let header_id = HeaderId {
				transactions_root: tx_root,
				uncles: uncles
			};
			match self.header_ids.get(&header_id).map(|n| *n) {
				Some(n) => {
					self.header_ids.remove(&header_id);
					self.bodies.insert_item(n, body.as_raw().to_vec());
					trace!(target: "sync", "Got body {}", n);
				}
				None =>  {
					debug!(target: "sync", "Ignored unknown block body");
				}
			}
		}
		self.collect_blocks(io);
		self.continue_sync(io);
	}

	/// Called by peer once it has new block bodies
	fn on_peer_new_block(&mut self, io: &mut SyncIo, peer_id: &PeerId, r: &Rlp) {
		let block_rlp = r.at(0);
		let header_rlp = block_rlp.at(0);
		let h = header_rlp.as_raw().sha3();

		trace!(target: "sync", "{} -> NewBlock ({})", peer_id, h);
		match io.chain().import_block(block_rlp.as_raw()) {
			ImportResult::AlreadyInChain  => {
				trace!(target: "sync", "New block already in chain {:?}", h);
			},
			ImportResult::AlreadyQueued(_)  => {
				trace!(target: "sync", "New block already queued {:?}", h);
			},
			ImportResult::Queued(QueueStatus::Known) => {
				trace!(target: "sync", "New block queued {:?}", h);
			},
			ImportResult::Queued(QueueStatus::Unknown) => {
				trace!(target: "sync", "New block unknown {:?}", h);
				//TODO: handle too many unknown blocks
				let difficulty: U256 = r.val_at(1);
				let peer_difficulty = self.peers.get_mut(&peer_id).expect("ChainSync: unknown peer").difficulty;
				if difficulty > peer_difficulty {
					trace!(target: "sync", "Received block {:?}  with no known parent. Peer needs syncing...", h);
					self.sync_peer(io, peer_id, true);
				}
			},
			ImportResult::Bad =>{
				debug!(target: "sync", "Bad new block {:?}", h);
				io.disable_peer(peer_id);
			}
		}
	}

	fn on_peer_new_hashes(&mut self, io: &mut SyncIo, peer_id: &PeerId, r: &Rlp) {
		if self.peers.get_mut(&peer_id).expect("ChainSync: unknown peer").asking != PeerAsking::Nothing {
			trace!(target: "sync", "Ignoring new hashes since we're already downloading.");
			return;
		}
		trace!(target: "sync", "{} -> NewHashes ({} entries)", peer_id, r.item_count());
		let hashes = r.iter().map(|item| (item.val_at::<H256>(0), item.val_at::<U256>(1)));
		let mut max_height: U256 = From::from(0);
		for (h, d) in hashes {
			match io.chain().block_status(&h) {
				BlockStatus::InChain  => {
					trace!(target: "sync", "New block hash already in chain {:?}", h);
				},
				BlockStatus::Queued(_) => {
					trace!(target: "sync", "New hash block already queued {:?}", h);
				},
				BlockStatus::Unknown => {
					trace!(target: "sync", "New unknown block hash {:?}", h);
					if d > max_height {
						let peer = self.peers.get_mut(&peer_id).expect("ChainSync: unknown peer");
						peer.latest = h.clone();
						max_height = d;
					}
				},
				BlockStatus::Bad =>{
					debug!(target: "sync", "Bad new block hash {:?}", h);
					io.disable_peer(peer_id);
					return;
				}
			}
		}
	}

	/// Called by peer when it is disconnecting
	pub fn on_peer_aborting(&mut self, io: &mut SyncIo, peer: &PeerId) {
		trace!(target: "sync", "== Disconnected {}", peer);
		self.clear_peer_download(peer);
		self.continue_sync(io);
	}

	pub fn on_peer_connected(&mut self, io: &mut SyncIo, peer: &PeerId) {
		trace!(target: "sync", "== Connected {}", peer);
		self.send_status(io, peer);
	}

	/// Resume downloading
	fn continue_sync(&mut self, io: &mut SyncIo) {
		let mut peers: Vec<(PeerId, U256)> = self.peers.iter().map(|(k, p)| (*k, p.difficulty)).collect();
		peers.sort_by(|&(_, d1), &(_, d2)| d1.cmp(&d2).reverse()); //TODO: sort by rating
		for (p, _) in peers {
			self.sync_peer(io, &p, false);
		}
	}

	/// Called after all blocks have been donloaded
	fn complete_sync(&mut self) {
		trace!(target: "sync", "Sync complete");
		self.reset();
		self.state = SyncState::Idle;
	}

	/// Enter waiting state
	fn pause_sync(&mut self) {
		trace!(target: "sync", "Block queue full, pausing sync");
		self.state = SyncState::Waiting;
	}

	fn sync_peer(&mut self, io: &mut SyncIo,  peer_id: &PeerId, force: bool) {
		let (peer_latest, peer_difficulty) = {
			let peer = self.peers.get_mut(&peer_id).expect("ChainSync: unknown peer");
			if peer.asking != PeerAsking::Nothing {
				return;
			}
			if self.state == SyncState::Waiting {
				trace!(target: "sync", "Waiting for block queue");
				return;
			}
			(peer.latest.clone(), peer.difficulty.clone())
		};

		let td = io.chain().chain_info().pending_total_difficulty;
		let syncing_difficulty = max(self.syncing_difficulty, td);
		if force || peer_difficulty > syncing_difficulty {
			// start sync
			self.syncing_difficulty = peer_difficulty;
			if self.state == SyncState::Idle || self.state == SyncState::NotSynced {
				self.state = SyncState::Blocks;
			}
			trace!(target: "sync", "Starting sync with better chain");
			self.request_headers_by_hash(io, peer_id, &peer_latest, 1, 0, false);
		}
		else if self.state == SyncState::Blocks {
			self.request_blocks(io, peer_id);
		}
	}

	fn request_blocks(&mut self, io: &mut SyncIo, peer_id: &PeerId) {
		self.clear_peer_download(peer_id);

		if io.chain().queue_status().full {
			self.pause_sync();
			return;
		}

		// check to see if we need to download any block bodies first
		let mut needed_bodies: Vec<H256> = Vec::new();
		let mut needed_numbers: Vec<BlockNumber> = Vec::new();

		if self.have_common_block && !self.headers.is_empty() && self.headers.range_iter().next().unwrap().0 == self.last_imported_block + 1 {
			for (start, ref items) in self.headers.range_iter() {
				if needed_bodies.len() > MAX_BODIES_TO_REQUEST {
					break;
				}
				let mut index: BlockNumber = 0;
				while index != items.len() as BlockNumber && needed_bodies.len() < MAX_BODIES_TO_REQUEST {
					let block = start + index;
					if !self.downloading_bodies.contains(&block) && !self.bodies.have_item(&block) {
						needed_bodies.push(items[index as usize].hash.clone());
						needed_numbers.push(block);
						self.downloading_bodies.insert(block);
					}
					index += 1;
				}
			}
		}
		if !needed_bodies.is_empty() {
			replace(&mut self.peers.get_mut(peer_id).unwrap().asking_blocks, needed_numbers);
			self.request_bodies(io, peer_id, needed_bodies);
		}
		else {
			// check if need to download headers
			let mut start = 0usize;
			if !self.have_common_block {
				// download backwards until common block is found 1 header at a time
				let chain_info = io.chain().chain_info();
				start = chain_info.best_block_number as usize;
				if !self.headers.is_empty() {
					start = min(start, self.headers.range_iter().next().unwrap().0 as usize - 1);
				}
				if start == 0 {
					self.have_common_block = true; //reached genesis
					self.last_imported_hash = chain_info.genesis_hash;
				}
			}
			if self.have_common_block {
				let mut headers: Vec<BlockNumber> = Vec::new();
				let mut prev = self.last_imported_block + 1;
				for (next, ref items) in self.headers.range_iter() {
					if !headers.is_empty() {
						break;
					}
					if next <= prev {
						prev = next + items.len() as BlockNumber;
						continue;
					}
					let mut block = prev;
					while block < next && headers.len() <= MAX_HEADERS_TO_REQUEST {
						if !self.downloading_headers.contains(&(block as BlockNumber)) {
							headers.push(block as BlockNumber);
							self.downloading_headers.insert(block as BlockNumber);
						}
						block += 1;
					}
					prev = next + items.len() as BlockNumber;
				}

				if !headers.is_empty() {
					start = headers[0] as usize;
					let count = headers.len();
					replace(&mut self.peers.get_mut(peer_id).unwrap().asking_blocks, headers);
					assert!(!self.headers.have_item(&(start as BlockNumber)));
					self.request_headers_by_number(io, peer_id, start as BlockNumber, count, 0, false);
				}
			}
			else {
				self.request_headers_by_number(io, peer_id, start as BlockNumber, 1, 0, false);
			}
		}
	}

	fn clear_peer_download(&mut self, peer_id: &PeerId) {
		let peer = self.peers.get_mut(&peer_id).expect("ChainSync: unknown peer");
		for b in &peer.asking_blocks {
			self.downloading_headers.remove(&b);
			self.downloading_bodies.remove(&b);
		}
		peer.asking_blocks.clear();
	}

	fn collect_blocks(&mut self, io: &mut SyncIo) {
		if !self.have_common_block || self.headers.is_empty() || self.bodies.is_empty() {
			return;
		}

		let mut restart = false;
		// merge headers and bodies
		{
			let headers = self.headers.range_iter().next().unwrap();
			let bodies = self.bodies.range_iter().next().unwrap();
			if headers.0 != bodies.0 || headers.0 != self.last_imported_block + 1 {
				return;
			}

			let count = min(headers.1.len(), bodies.1.len());
			let mut imported = 0;
			for i in 0..count {
				let mut block_rlp = RlpStream::new_list(3);
				block_rlp.append_raw(&headers.1[i].data, 1);
				let body = Rlp::new(&bodies.1[i]);
				block_rlp.append_raw(body.at(0).as_raw(), 1);
				block_rlp.append_raw(body.at(1).as_raw(), 1);
				let h = &headers.1[i].hash;
				match io.chain().import_block(&block_rlp.out()) {
					ImportResult::AlreadyInChain  => {
						trace!(target: "sync", "Block already in chain {:?}", h);
						self.last_imported_block = headers.0 + i as BlockNumber;
						self.last_imported_hash = h.clone();
					},
					ImportResult::AlreadyQueued(_)  => {
						trace!(target: "sync", "Block already queued {:?}", h);
						self.last_imported_block = headers.0 + i as BlockNumber;
						self.last_imported_hash = h.clone();
					},
					ImportResult::Queued(QueueStatus::Known) => {
						trace!(target: "sync", "Block queued {:?}", h);
						self.last_imported_block = headers.0 + i as BlockNumber;
						self.last_imported_hash = h.clone();
						imported += 1;
					},
					ImportResult::Queued(QueueStatus::Unknown) => {
						panic!("Queued out of order block");
					},
					ImportResult::Bad =>{
						debug!(target: "sync", "Bad block {:?}", h);
						restart = true;
					}
				}
			}
			trace!(target: "sync", "Imported {} of {}", imported, count);
		}

		if restart {
			self.restart(io);
			return;
		}

		self.headers.remove_head(&(self.last_imported_block + 1));
		self.bodies.remove_head(&(self.last_imported_block + 1));

		if self.headers.is_empty() {
			assert!(self.bodies.is_empty());
			self.complete_sync();
		}
	}

	fn remove_downloaded_blocks(&mut self, start: BlockNumber) {
		for n in self.headers.get_tail(&start) {
			match self.headers.find_item(&n) {
				Some(ref header_data) => {
					let header_to_delete = HeaderView::new(&header_data.data);
					let header_id = HeaderId {
						transactions_root: header_to_delete.transactions_root(),
						uncles: header_to_delete.uncles_hash()
					};
					self.header_ids.remove(&header_id);
				},
				None => {}
			}
			self.downloading_bodies.remove(&n);
			self.downloading_headers.remove(&n);
		}
		self.headers.remove_tail(&start);
		self.bodies.remove_tail(&start);
	}

	fn request_headers_by_hash(&mut self, sync: &mut SyncIo, peer_id: &PeerId, h: &H256, count: usize, skip: usize, reverse: bool) {
		trace!(target: "sync", "{}<- GetBlockHeaders: {} entries starting from {}", peer_id, count, h);
		let mut rlp = RlpStream::new_list(4);
		rlp.append(h);
		rlp.append(&count);
		rlp.append(&skip);
		rlp.append(&if reverse {1u32} else {0u32});
		self.send_request(sync, peer_id, PeerAsking::BlockHeaders, GET_BLOCK_HEADERS_PACKET, rlp.out());
	}

	fn request_headers_by_number(&mut self, sync: &mut SyncIo, peer_id: &PeerId, n: BlockNumber, count: usize, skip: usize, reverse: bool) {
		let mut rlp = RlpStream::new_list(4);
		trace!(target: "sync", "{}<- GetBlockHeaders: {} entries starting from {}", peer_id, count, n);
		rlp.append(&n);
		rlp.append(&count);
		rlp.append(&skip);
		rlp.append(&if reverse {1u32} else {0u32});
		self.send_request(sync, peer_id, PeerAsking::BlockHeaders, GET_BLOCK_HEADERS_PACKET, rlp.out());
	}

	fn request_bodies(&mut self, sync: &mut SyncIo, peer_id: &PeerId, hashes: Vec<H256>) {
		let mut rlp = RlpStream::new_list(hashes.len());
		trace!(target: "sync", "{}<- GetBlockBodies: {} entries", peer_id, hashes.len());
		for h in hashes {
			rlp.append(&h);
		}
		self.send_request(sync, peer_id, PeerAsking::BlockBodies, GET_BLOCK_BODIES_PACKET, rlp.out());
	}

	fn reset_peer_asking(&mut self, peer_id: &PeerId, asking: PeerAsking) {
		let peer = self.peers.get_mut(&peer_id).expect("ChainSync: unknown peer");
		if peer.asking != asking {
			warn!(target:"sync", "Asking {:?} while expected {:?}", peer.asking, asking);
		}
		else {
			peer.asking = PeerAsking::Nothing;
		}
	}

	fn send_request(&mut self, sync: &mut SyncIo, peer_id: &PeerId, asking: PeerAsking,  packet_id: PacketId, packet: Bytes) {
		{
			let peer = self.peers.get_mut(&peer_id).expect("ChainSync: unknown peer");
			if peer.asking != PeerAsking::Nothing {
				warn!(target:"sync", "Asking {:?} while requesting {:?}", asking, peer.asking);
			}
		}
		match sync.send(*peer_id, packet_id, packet) {
			Err(e) => {
				warn!(target:"sync", "Error sending request: {:?}", e);
				sync.disable_peer(peer_id);
				self.on_peer_aborting(sync, peer_id);
			}
			Ok(_) => {
				let mut peer = self.peers.get_mut(&peer_id).unwrap();
				peer.asking = asking;
			}
		}
	}

	fn on_peer_transactions(&mut self, _io: &mut SyncIo, _peer_id: &PeerId, _r: &Rlp) {
	}

	fn send_status(&mut self, io: &mut SyncIo, peer_id: &PeerId) {
		let mut packet = RlpStream::new_list(5);
		let chain = io.chain().chain_info();
		packet.append(&(PROTOCOL_VERSION as u32));
		packet.append(&0u32); //TODO: network id
		packet.append(&chain.total_difficulty);
		packet.append(&chain.best_block_hash);
		packet.append(&chain.genesis_hash);
		//TODO: handle timeout for status request
		match io.send(*peer_id, STATUS_PACKET, packet.out()) {
			Err(e) => {
				warn!(target:"sync", "Error sending status request: {:?}", e);
				io.disable_peer(peer_id);
			}
			Ok(_) => ()
		}
	}

	fn return_block_headers(&self, io: &mut SyncIo, r: &Rlp) {
		// Packet layout:
		// [ block: { P , B_32 }, maxHeaders: P, skip: P, reverse: P in { 0 , 1 } ]
		let max_headers: usize = r.val_at(1);
		let skip: usize = r.val_at(2);
		let reverse: bool = r.val_at(3);
		let last = io.chain().chain_info().best_block_number;
		let mut number = if r.at(0).size() == 32 {
			// id is a hash
			let hash: H256 = r.val_at(0);
			trace!(target: "sync", "-> GetBlockHeaders (hash: {}, max: {}, skip: {}, reverse:{})", hash, max_headers, skip, reverse);
			match io.chain().block_header(&hash) {
				Some(hdr) => From::from(HeaderView::new(&hdr).number()),
				None => last
			}
		}
		else {
			trace!(target: "sync", "-> GetBlockHeaders (number: {}, max: {}, skip: {}, reverse:{})", r.val_at::<BlockNumber>(0), max_headers, skip, reverse);
			r.val_at(0)
		};

		if reverse {
			number = min(last, number);
		} else {
			number = max(1, number);
		}
		let max_count = min(MAX_HEADERS_TO_SEND, max_headers);
		let mut count = 0;
		let mut data = Bytes::new();
		let inc = (skip + 1) as BlockNumber;
		while number <= last && number > 0 && count < max_count {
			match io.chain().block_header_at(number) {
				Some(mut hdr) => {
					data.append(&mut hdr);
					count += 1;
				}
				None => {}
			}
			if reverse {
				if number <= inc {
					break;
				}
				number -= inc;
			}
			else {
				number += inc;
			}
		}
		let mut rlp = RlpStream::new_list(count as usize);
		rlp.append_raw(&data, count as usize);
		io.respond(BLOCK_HEADERS_PACKET, rlp.out()).unwrap_or_else(|e|
			debug!(target: "sync", "Error sending headers: {:?}", e));
		trace!(target: "sync", "-> GetBlockHeaders: returned {} entries", count);
	}

	fn return_block_bodies(&self, io: &mut SyncIo, r: &Rlp) {
		let mut count = r.item_count();
		if count == 0 {
			debug!(target: "sync", "Empty GetBlockBodies request, ignoring.");
			return;
		}
		trace!(target: "sync", "-> GetBlockBodies: {} entries", count);
		count = min(count, MAX_BODIES_TO_SEND);
		let mut added = 0usize;
		let mut data = Bytes::new();
		for i in 0..count {
			match io.chain().block_body(&r.val_at::<H256>(i)) {
				Some(mut hdr) => {
					data.append(&mut hdr);
					added += 1;
				}
				None => {}
			}
		}
		let mut rlp = RlpStream::new_list(added);
		rlp.append_raw(&data, added);
		io.respond(BLOCK_BODIES_PACKET, rlp.out()).unwrap_or_else(|e|
			debug!(target: "sync", "Error sending headers: {:?}", e));
		trace!(target: "sync", "-> GetBlockBodies: returned {} entries", added);
	}

	fn return_node_data(&self, io: &mut SyncIo, r: &Rlp) {
		let mut count = r.item_count();
		if count == 0 {
			debug!(target: "sync", "Empty GetNodeData request, ignoring.");
			return;
		}
		count = min(count, MAX_NODE_DATA_TO_SEND);
		let mut added = 0usize;
		let mut data = Bytes::new();
		for i in 0..count {
			match io.chain().state_data(&r.val_at::<H256>(i)) {
				Some(mut hdr) => {
					data.append(&mut hdr);
					added += 1;
				}
				None => {}
			}
		}
		let mut rlp = RlpStream::new_list(added);
		rlp.append_raw(&data, added);
		io.respond(NODE_DATA_PACKET, rlp.out()).unwrap_or_else(|e|
			debug!(target: "sync", "Error sending headers: {:?}", e));
	}

	fn return_receipts(&self, io: &mut SyncIo, r: &Rlp) {
		let mut count = r.item_count();
		if count == 0 {
			debug!(target: "sync", "Empty GetReceipts request, ignoring.");
			return;
		}
		count = min(count, MAX_RECEIPTS_TO_SEND);
		let mut added = 0usize;
		let mut data = Bytes::new();
		for i in 0..count {
			match io.chain().block_receipts(&r.val_at::<H256>(i)) {
				Some(mut hdr) => {
					data.append(&mut hdr);
					added += 1;
				}
				None => {}
			}
		}
		let mut rlp = RlpStream::new_list(added);
		rlp.append_raw(&data, added);
		io.respond(RECEIPTS_PACKET, rlp.out()).unwrap_or_else(|e|
			debug!(target: "sync", "Error sending headers: {:?}", e));
	}

	pub fn on_packet(&mut self, io: &mut SyncIo, peer: &PeerId, packet_id: u8, data: &[u8]) {
		let rlp = Rlp::new(data);
		match packet_id {
			STATUS_PACKET => self.on_peer_status(io, peer, &rlp),
			TRANSACTIONS_PACKET => self.on_peer_transactions(io, peer, &rlp),
			GET_BLOCK_HEADERS_PACKET => self.return_block_headers(io, &rlp),
			BLOCK_HEADERS_PACKET => self.on_peer_block_headers(io, peer, &rlp),
			GET_BLOCK_BODIES_PACKET => self.return_block_bodies(io, &rlp),
			BLOCK_BODIES_PACKET => self.on_peer_block_bodies(io, peer, &rlp),
			NEW_BLOCK_PACKET => self.on_peer_new_block(io, peer, &rlp),
			NEW_BLOCK_HASHES_PACKET => self.on_peer_new_hashes(io, peer, &rlp),
			GET_NODE_DATA_PACKET => self.return_node_data(io, &rlp),
			GET_RECEIPTS_PACKET => self.return_receipts(io, &rlp),
			_ => debug!(target: "sync", "Unknown packet {}", packet_id)
		}
	}

	pub fn maintain_sync(&mut self, _io: &mut SyncIo) {
	}
}

