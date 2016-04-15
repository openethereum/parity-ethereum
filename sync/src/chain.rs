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

///
/// `BlockChain` synchronization strategy.
/// Syncs to peers and keeps up to date.
/// This implementation uses ethereum protocol v63
///
/// Syncing strategy.
///
/// 1. A peer arrives with a total difficulty better than ours
/// 2. Find a common best block between our an peer chain.
/// Start with out best block and request headers from peer backwards until a common block is found
/// 3. Download headers and block bodies from peers in parallel.
/// As soon as a set of the blocks is fully downloaded at the head of the queue it is fed to the blockchain
/// 4. Maintain sync by handling NewBlocks/NewHashes messages
///

use util::*;
use std::mem::{replace};
use ethcore::views::{HeaderView};
use ethcore::header::{BlockNumber, Header as BlockHeader};
use ethcore::client::{BlockChainClient, BlockStatus, BlockId, BlockChainInfo};
use range_collection::{RangeCollection, ToUsize, FromUsize};
use ethcore::error::*;
use ethcore::transaction::SignedTransaction;
use ethcore::block::Block;
use ethminer::{Miner, MinerService, AccountDetails};
use io::SyncIo;
use time;
use super::SyncConfig;

known_heap_size!(0, PeerInfo, Header, HeaderId);

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

type PacketDecodeError = DecoderError;

const PROTOCOL_VERSION: u8 = 63u8;
const MAX_BODIES_TO_SEND: usize = 256;
const MAX_HEADERS_TO_SEND: usize = 512;
const MAX_NODE_DATA_TO_SEND: usize = 1024;
const MAX_RECEIPTS_TO_SEND: usize = 1024;
const MAX_RECEIPTS_HEADERS_TO_SEND: usize = 256;
const MAX_HEADERS_TO_REQUEST: usize = 512;
const MAX_BODIES_TO_REQUEST: usize = 256;
const MIN_PEERS_PROPAGATION: usize = 4;
const MAX_PEERS_PROPAGATION: usize = 128;
const MAX_PEER_LAG_PROPAGATION: BlockNumber = 20;

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

const CONNECTION_TIMEOUT_SEC: f64 = 5f64;

struct Header {
	/// Header data
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
/// Sync state
pub enum SyncState {
	/// Initial chain sync has not started yet
	NotSynced,
	/// Initial chain sync complete. Waiting for new packets
	Idle,
	/// Block downloading paused. Waiting for block queue to process blocks and free some space
	Waiting,
	/// Downloading blocks
	Blocks,
	/// Downloading blocks learned from NewHashes packet
	NewBlocks,
}

/// Syncing status and statistics
#[derive(Clone)]
pub struct SyncStatus {
	/// State
	pub state: SyncState,
	/// Syncing protocol version. That's the maximum protocol version we connect to.
	pub protocol_version: u8,
	/// The underlying p2p network version.
	pub network_id: U256,
	/// `BlockChain` height for the moment the sync started.
	pub start_block_number: BlockNumber,
	/// Last fully downloaded and imported block number (if any).
	pub last_imported_block_number: Option<BlockNumber>,
	/// Highest block number in the download queue (if any).
	pub highest_block_number: Option<BlockNumber>,
	/// Total number of blocks for the sync process.
	pub blocks_total: BlockNumber,
	/// Number of blocks downloaded so far.
	pub blocks_received: BlockNumber,
	/// Total number of connected peers
	pub num_peers: usize,
	/// Total number of active peers
	pub num_active_peers: usize,
	/// Heap memory used in bytes
	pub mem_used: usize,
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Peer data type requested
enum PeerAsking {
	Nothing,
	BlockHeaders,
	BlockBodies,
}

#[derive(Clone)]
/// Syncing peer information
struct PeerInfo {
	/// eth protocol version
	protocol_version: u32,
	/// Peer chain genesis hash
	genesis: H256,
	/// Peer network id
	network_id: U256,
	/// Peer best block hash
	latest_hash: H256,
	/// Peer best block number if known
	latest_number: Option<BlockNumber>,
	/// Peer total difficulty
	difficulty: U256,
	/// Type of data currenty being requested from peer.
	asking: PeerAsking,
	/// A set of block numbers being requested
	asking_blocks: Vec<BlockNumber>,
	/// Holds requested header hash if currently requesting block header by hash
	asking_hash: Option<H256>,
	/// Request timestamp
	ask_time: f64,
}

/// Blockchain sync handler.
/// See module documentation for more details.
pub struct ChainSync {
	/// Sync state
	state: SyncState,
	/// Last block number for the start of sync
	starting_block: BlockNumber,
	/// Highest block number seen
	highest_block: Option<BlockNumber>,
	/// Set of block header numbers being downloaded
	downloading_headers: HashSet<BlockNumber>,
	/// Set of block body numbers being downloaded
	downloading_bodies: HashSet<BlockNumber>,
	/// Set of block headers being downloaded by hash
	downloading_hashes: HashSet<H256>,
	/// Downloaded headers.
	headers: Vec<(BlockNumber, Vec<Header>)>, //TODO: use BTreeMap once range API is sable. For now it is a vector sorted in descending order
	/// Downloaded bodies
	bodies: Vec<(BlockNumber, Vec<Bytes>)>, //TODO: use BTreeMap once range API is sable. For now it is a vector sorted in descending order
	/// Peer info
	peers: HashMap<PeerId, PeerInfo>,
	/// Used to map body to header
	header_ids: HashMap<HeaderId, BlockNumber>,
	/// Last impoted block number
	last_imported_block: Option<BlockNumber>,
	/// Last impoted block hash
	last_imported_hash: Option<H256>,
	/// Syncing total  difficulty
	syncing_difficulty: U256,
	/// True if common block for our and remote chain has been found
	have_common_block: bool,
	/// Last propagated block number
	last_sent_block_number: BlockNumber,
	/// Max blocks to download ahead
	max_download_ahead_blocks: usize,
	/// Network ID
	network_id: U256,
	/// Miner
	miner: Arc<Miner>,
}

type RlpResponseResult = Result<Option<(PacketId, RlpStream)>, PacketDecodeError>;

impl ChainSync {
	/// Create a new instance of syncing strategy.
	pub fn new(config: SyncConfig, miner: Arc<Miner>) -> ChainSync {
		ChainSync {
			state: SyncState::NotSynced,
			starting_block: 0,
			highest_block: None,
			downloading_headers: HashSet::new(),
			downloading_bodies: HashSet::new(),
			downloading_hashes: HashSet::new(),
			headers: Vec::new(),
			bodies: Vec::new(),
			peers: HashMap::new(),
			header_ids: HashMap::new(),
			last_imported_block: None,
			last_imported_hash: None,
			syncing_difficulty: U256::from(0u64),
			have_common_block: false,
			last_sent_block_number: 0,
			max_download_ahead_blocks: max(MAX_HEADERS_TO_REQUEST, config.max_download_ahead_blocks),
			network_id: config.network_id,
			miner: miner,
		}
	}

	/// @returns Synchonization status
	pub fn status(&self) -> SyncStatus {
		SyncStatus {
			state: self.state.clone(),
			protocol_version: 63,
			network_id: self.network_id,
			start_block_number: self.starting_block,
			last_imported_block_number: self.last_imported_block,
			highest_block_number: self.highest_block,
			blocks_received: match self.last_imported_block { Some(x) if x > self.starting_block => x - self.starting_block, _ => 0 },
			blocks_total: match self.highest_block { Some(x) if x > self.starting_block => x - self.starting_block, _ => 0 },
			num_peers: self.peers.len(),
			num_active_peers: self.peers.values().filter(|p| p.asking != PeerAsking::Nothing).count(),
			mem_used:
				//  TODO: https://github.com/servo/heapsize/pull/50
				//  self.downloading_hashes.heap_size_of_children()
				//+ self.downloading_bodies.heap_size_of_children()
				//+ self.downloading_hashes.heap_size_of_children()
				self.headers.heap_size_of_children()
				+ self.bodies.heap_size_of_children()
				+ self.peers.heap_size_of_children()
				+ self.header_ids.heap_size_of_children(),
		}
	}

	/// Abort all sync activity
	pub fn abort(&mut self, io: &mut SyncIo) {
		self.restart(io);
		self.peers.clear();
	}

	#[cfg_attr(feature="dev", allow(for_kv_map))] // Because it's not possible to get `values_mut()`
	/// Rest sync. Clear all downloaded data but keep the queue
	fn reset(&mut self) {
		self.downloading_headers.clear();
		self.downloading_bodies.clear();
		self.headers.clear();
		self.bodies.clear();
		for (_, ref mut p) in &mut self.peers {
			p.asking_blocks.clear();
			p.asking_hash = None;
		}
		self.header_ids.clear();
		self.syncing_difficulty = From::from(0u64);
		self.state = SyncState::Idle;
	}

	/// Restart sync
	pub fn restart(&mut self, io: &mut SyncIo) {
		self.reset();
		self.starting_block = 0;
		self.highest_block = None;
		self.have_common_block = false;
		self.starting_block = io.chain().chain_info().best_block_number;
		self.state = SyncState::NotSynced;
	}

	/// Restart sync after bad block has been detected. May end up re-downloading up to QUEUE_SIZE blocks
	pub fn restart_on_bad_block(&mut self, io: &mut SyncIo) {
		self.restart(io);
		// Do not assume that the block queue/chain still has our last_imported_block
		self.last_imported_block = None;
		self.last_imported_hash = None;
	}
	/// Called by peer to report status
	fn on_peer_status(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		let peer = PeerInfo {
			protocol_version: try!(r.val_at(0)),
			network_id: try!(r.val_at(1)),
			difficulty: try!(r.val_at(2)),
			latest_hash: try!(r.val_at(3)),
			latest_number: None,
			genesis: try!(r.val_at(4)),
			asking: PeerAsking::Nothing,
			asking_blocks: Vec::new(),
			asking_hash: None,
			ask_time: 0f64,
		};

		trace!(target: "sync", "New peer {} (protocol: {}, network: {:?}, difficulty: {:?}, latest:{}, genesis:{})", peer_id, peer.protocol_version, peer.network_id, peer.difficulty, peer.latest_hash, peer.genesis);

		if self.peers.contains_key(&peer_id) {
			warn!("Unexpected status packet from {}:{}", peer_id, io.peer_info(peer_id));
			return Ok(());
		}
		let chain_info = io.chain().chain_info();
		if peer.genesis != chain_info.genesis_hash {
			io.disable_peer(peer_id);
			trace!(target: "sync", "Peer {} genesis hash not matched", peer_id);
			return Ok(());
		}
		if peer.network_id != self.network_id {
			io.disable_peer(peer_id);
			trace!(target: "sync", "Peer {} network id not matched", peer_id);
			return Ok(());
		}

		self.peers.insert(peer_id.clone(), peer);
		debug!(target: "sync", "Connected {}:{}", peer_id, io.peer_info(peer_id));
		self.sync_peer(io, peer_id, false);
		Ok(())
	}

	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	/// Called by peer once it has new block headers during sync
	fn on_peer_block_headers(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		self.reset_peer_asking(peer_id, PeerAsking::BlockHeaders);
		let item_count = r.item_count();
		trace!(target: "sync", "{} -> BlockHeaders ({} entries)", peer_id, item_count);
		self.clear_peer_download(peer_id);
		if self.state != SyncState::Blocks && self.state != SyncState::NewBlocks && self.state != SyncState::Waiting {
			trace!(target: "sync", "Ignored unexpected block headers");
			return Ok(());
		}
		if self.state == SyncState::Waiting {
			trace!(target: "sync", "Ignored block headers while waiting");
			return Ok(());
		}

		for i in 0..item_count {
			let info: BlockHeader = try!(r.val_at(i));
			let number = BlockNumber::from(info.number);
			if (number <= self.current_base_block() && self.have_common_block) || self.headers.have_item(&number) {
				trace!(target: "sync", "Skipping existing block header");
				continue;
			}

			if self.highest_block == None || number > self.highest_block.unwrap() {
				self.highest_block = Some(number);
			}
			let hash = info.hash();
			match io.chain().block_status(BlockId::Hash(hash.clone())) {
				BlockStatus::InChain | BlockStatus::Queued => {
					if !self.have_common_block || self.current_base_block() < number {
						self.last_imported_block = Some(number);
						self.last_imported_hash = Some(hash.clone());
					}
					if !self.have_common_block {
						self.have_common_block = true;
						trace!(target: "sync", "Found common header {} ({})", number, hash);
					} else {
						trace!(target: "sync", "Header already in chain {} ({})", number, hash);
					}
				},
				_ => {
					if self.have_common_block {
						//validate chain
						let base_hash = self.last_imported_hash.clone().unwrap();
						if self.have_common_block && number == self.current_base_block() + 1 && info.parent_hash != base_hash {
							// Part of the forked chain. Restart to find common block again
							debug!(target: "sync", "Mismatched block header {} {}, restarting sync", number, hash);
							self.restart(io);
							return Ok(());
						}
						if self.headers.find_item(&(number - 1)).map_or(false, |p| p.hash != info.parent_hash) {
							// mismatching parent id, delete the previous block and don't add this one
							debug!(target: "sync", "Mismatched block header {} {}", number, hash);
							self.remove_downloaded_blocks(number - 1);
							continue;
						}
						if self.headers.find_item(&(number + 1)).map_or(false, |p| p.parent != hash) {
							// mismatching parent id for the next block, clear following headers
							debug!(target: "sync", "Mismatched block header {}", number + 1);
							self.remove_downloaded_blocks(number + 1);
						}
						if self.have_common_block && number < self.current_base_block() + 1 {
							// unkown header
							debug!(target: "sync", "Old block header {:?} ({}) is unknown, restarting sync", hash, number);
							self.restart(io);
							return Ok(());
						}
					}
					let hdr = Header {
						data: try!(r.at(i)).as_raw().to_vec(),
						hash: hash.clone(),
						parent: info.parent_hash,
					};
					self.headers.insert_item(number, hdr);
					let header_id = HeaderId {
						transactions_root: info.transactions_root,
						uncles: info.uncles_hash
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
		Ok(())
	}

	/// Called by peer once it has new block bodies
	fn on_peer_block_bodies(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		use util::triehash::ordered_trie_root;
		self.reset_peer_asking(peer_id, PeerAsking::BlockBodies);
		let item_count = r.item_count();
		trace!(target: "sync", "{} -> BlockBodies ({} entries)", peer_id, item_count);
		self.clear_peer_download(peer_id);
		if self.state != SyncState::Blocks && self.state != SyncState::NewBlocks && self.state != SyncState::Waiting {
			trace!(target: "sync", "Ignored unexpected block bodies");
			return Ok(());
		}
		if self.state == SyncState::Waiting {
			trace!(target: "sync", "Ignored block bodies while waiting");
			return Ok(());
		}
		for i in 0..item_count {
			let body = try!(r.at(i));
			let tx = try!(body.at(0));
			let tx_root = ordered_trie_root(tx.iter().map(|r| r.as_raw().to_vec()).collect()); //TODO: get rid of vectors here
			let uncles = try!(body.at(1)).as_raw().sha3();
			let header_id = HeaderId {
				transactions_root: tx_root,
				uncles: uncles
			};
			match self.header_ids.get(&header_id).cloned() {
				Some(n) => {
					self.header_ids.remove(&header_id);
					self.bodies.insert_item(n, body.as_raw().to_vec());
					trace!(target: "sync", "Got body {}", n);
				}
				None =>  {
					trace!(target: "sync", "Ignored unknown/stale block body");
				}
			}
		}
		self.collect_blocks(io);
		self.continue_sync(io);
		Ok(())
	}

	/// Called by peer once it has new block bodies
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn on_peer_new_block(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		let block_rlp = try!(r.at(0));
		let header_rlp = try!(block_rlp.at(0));
		let h = header_rlp.as_raw().sha3();
		trace!(target: "sync", "{} -> NewBlock ({})", peer_id, h);
		if !self.have_common_block {
			trace!(target: "sync", "NewBlock ignored while seeking");
			return Ok(());
		}
		let header: BlockHeader = try!(header_rlp.as_val());
 		let mut unknown = false;
		{
			let peer = self.peers.get_mut(&peer_id).unwrap();
			peer.latest_hash = header.hash();
			peer.latest_number = Some(header.number());
		}
		// TODO: Decompose block and add to self.headers and self.bodies instead
		if header.number <= From::from(self.current_base_block() + 1) {
			match io.chain().import_block(block_rlp.as_raw().to_vec()) {
				Err(Error::Import(ImportError::AlreadyInChain)) => {
					trace!(target: "sync", "New block already in chain {:?}", h);
				},
				Err(Error::Import(ImportError::AlreadyQueued)) => {
					trace!(target: "sync", "New block already queued {:?}", h);
				},
				Ok(_) => {
					if self.current_base_block() < header.number {
						self.last_imported_block = Some(header.number);
						self.last_imported_hash = Some(header.hash());
						self.remove_downloaded_blocks(header.number);
					}
					trace!(target: "sync", "New block queued {:?} ({})", h, header.number);
				},
				Err(Error::Block(BlockError::UnknownParent(p))) => {
					unknown = true;
					trace!(target: "sync", "New block with unknown parent ({:?}) {:?}", p, h);
				},
				Err(e) => {
					debug!(target: "sync", "Bad new block {:?} : {:?}", h, e);
					io.disable_peer(peer_id);
				}
			};
		}
  		else {
			unknown = true;
		}
		if unknown {
			trace!(target: "sync", "New block unknown {:?}", h);
			//TODO: handle too many unknown blocks
			let difficulty: U256 = try!(r.val_at(1));
			let peer_difficulty = self.peers.get_mut(&peer_id).unwrap().difficulty;
			if difficulty > peer_difficulty {
				trace!(target: "sync", "Received block {:?}  with no known parent. Peer needs syncing...", h);
				self.sync_peer(io, peer_id, true);
			}
		}
		Ok(())
	}

	/// Handles NewHashes packet. Initiates headers download for any unknown hashes.
	fn on_peer_new_hashes(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		if self.peers.get_mut(&peer_id).unwrap().asking != PeerAsking::Nothing {
			trace!(target: "sync", "Ignoring new hashes since we're already downloading.");
			return Ok(());
		}
		trace!(target: "sync", "{} -> NewHashes ({} entries)", peer_id, r.item_count());
		let hashes = r.iter().map(|item| (item.val_at::<H256>(0), item.val_at::<BlockNumber>(1)));
		let mut max_height: BlockNumber = 0;
		for (rh, rd) in hashes {
			let h = try!(rh);
			let d = try!(rd);
			if self.downloading_hashes.contains(&h) {
				continue;
			}
			match io.chain().block_status(BlockId::Hash(h.clone())) {
				BlockStatus::InChain  => {
					trace!(target: "sync", "New block hash already in chain {:?}", h);
				},
				BlockStatus::Queued => {
					trace!(target: "sync", "New hash block already queued {:?}", h);
				},
				BlockStatus::Unknown => {
					if d > max_height {
						trace!(target: "sync", "New unknown block hash {:?}", h);
						let peer = self.peers.get_mut(&peer_id).unwrap();
						peer.latest_hash = h.clone();
						peer.latest_number = Some(d);
						max_height = d;
					}
				},
				BlockStatus::Bad =>{
					debug!(target: "sync", "Bad new block hash {:?}", h);
					io.disable_peer(peer_id);
					return Ok(());
				}
			}
		};
		if max_height != 0 {
			self.sync_peer(io, peer_id, true);
		}
		Ok(())
	}

	/// Called by peer when it is disconnecting
	pub fn on_peer_aborting(&mut self, io: &mut SyncIo, peer: PeerId) {
		trace!(target: "sync", "== Disconnecting {}", peer);
		if self.peers.contains_key(&peer) {
			debug!(target: "sync", "Disconnected {}", peer);
			self.clear_peer_download(peer);
			self.peers.remove(&peer);
			self.continue_sync(io);
		}
	}

	/// Called when a new peer is connected
	pub fn on_peer_connected(&mut self, io: &mut SyncIo, peer: PeerId) {
		trace!(target: "sync", "== Connected {}", peer);
		if let Err(e) = self.send_status(io) {
			debug!(target:"sync", "Error sending status request: {:?}", e);
			io.disable_peer(peer);
		}
	}

	/// Resume downloading
	fn continue_sync(&mut self, io: &mut SyncIo) {
		let mut peers: Vec<(PeerId, U256)> = self.peers.iter().map(|(k, p)| (*k, p.difficulty)).collect();
		peers.sort_by(|&(_, d1), &(_, d2)| d1.cmp(&d2).reverse()); //TODO: sort by rating
		for (p, _) in peers {
			self.sync_peer(io, p, false);
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

	/// Find something to do for a peer. Called for a new peer or when a peer is done with it's task.
	fn sync_peer(&mut self, io: &mut SyncIo,  peer_id: PeerId, force: bool) {
		let (peer_latest, peer_difficulty) = {
			let peer = self.peers.get_mut(&peer_id).unwrap();
			if peer.asking != PeerAsking::Nothing {
				return;
			}
			if self.state == SyncState::Waiting {
				trace!(target: "sync", "Waiting for block queue");
				return;
			}
			(peer.latest_hash.clone(), peer.difficulty.clone())
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
			self.peers.get_mut(&peer_id).unwrap().asking_hash = Some(peer_latest.clone());
			self.downloading_hashes.insert(peer_latest.clone());
			self.request_headers_by_hash(io, peer_id, &peer_latest, 1, 0, false);
		}
		else if self.state == SyncState::Blocks && io.chain().block_status(BlockId::Hash(peer_latest)) == BlockStatus::Unknown {
			self.request_blocks(io, peer_id, false);
		}
	}

	fn current_base_block(&self) -> BlockNumber {
		match self.last_imported_block { None => 0, Some(x) => x }
	}

	fn find_block_bodies_hashes_to_request(&self, ignore_others: bool) -> (Vec<H256>, Vec<BlockNumber>) {
		let mut needed_bodies: Vec<H256> = Vec::new();
		let mut needed_numbers: Vec<BlockNumber> = Vec::new();

		if self.have_common_block && !self.headers.is_empty() && self.headers.range_iter().next().unwrap().0 == self.current_base_block() + 1 {
			if let Some((start, ref items)) = self.headers.range_iter().next() {
				let mut index: BlockNumber = 0;
				while index != items.len() as BlockNumber && needed_bodies.len() < MAX_BODIES_TO_REQUEST {
					let block = start + index;
					if  ignore_others || (!self.downloading_bodies.contains(&block) && !self.bodies.have_item(&block)) {
						needed_bodies.push(items[index as usize].hash.clone());
						needed_numbers.push(block);
					}
					index += 1;
				}
			}
		}
		(needed_bodies, needed_numbers)
	}

	/// Find some headers or blocks to download for a peer.
	fn request_blocks(&mut self, io: &mut SyncIo, peer_id: PeerId, ignore_others: bool) {
		self.clear_peer_download(peer_id);

		if io.chain().queue_info().is_full() {
			self.pause_sync();
			return;
		}

		// check to see if we need to download any block bodies first
		let (needed_bodies, needed_numbers) = self.find_block_bodies_hashes_to_request(ignore_others);
		if !needed_bodies.is_empty() {
			let (head, _) = self.headers.range_iter().next().unwrap();
			if needed_numbers.first().unwrap() - head > self.max_download_ahead_blocks as BlockNumber {
				trace!(target: "sync", "{}: Stalled download ({} vs {}), helping with downloading block bodies", peer_id, needed_numbers.first().unwrap(), head);
				self.request_blocks(io, peer_id, true);
			} else {
				self.downloading_bodies.extend(needed_numbers.iter());
				replace(&mut self.peers.get_mut(&peer_id).unwrap().asking_blocks, needed_numbers);
				self.request_bodies(io, peer_id, needed_bodies);
			}
			return;
		}

		// check if need to download headers
		let mut start = 0;
		if !self.have_common_block {
			// download backwards until common block is found 1 header at a time
			let chain_info = io.chain().chain_info();
			start = match self.last_imported_block {
				Some(n) => n,
				None => chain_info.best_block_number,
			};
			if !self.headers.is_empty() {
				start = min(start, self.headers.range_iter().next().unwrap().0 - 1);
			}
			if start == 0 {
				self.have_common_block = true; //reached genesis
				self.last_imported_hash = Some(chain_info.genesis_hash);
				self.last_imported_block = Some(0);
			}
		}
		if self.have_common_block {
			let mut headers: Vec<BlockNumber> = Vec::new();
			let mut prev = self.current_base_block() + 1;
			let head = self.headers.range_iter().next().map(|(h, _)| h);
			for (next, ref items) in self.headers.range_iter() {
				if !headers.is_empty() {
					break;
				}
				if next <= prev {
					prev = next + items.len() as BlockNumber;
					continue;
				}
				let mut block = prev;
				while block < next && headers.len() < MAX_HEADERS_TO_REQUEST {
					if ignore_others || !self.downloading_headers.contains(&(block as BlockNumber)) {
						headers.push(block as BlockNumber);
					}
					block += 1;
				}
				prev = next + items.len() as BlockNumber;
			}

			if !headers.is_empty() {
				start = headers[0];
				if head.is_some() && start > head.unwrap() && start - head.unwrap() > self.max_download_ahead_blocks as BlockNumber {
					trace!(target: "sync", "{}: Stalled download ({} vs {}), helping with downloading headers", peer_id, start, head.unwrap());
					self.request_blocks(io, peer_id, true);
					return;
				}
				let count = headers.len();
				self.downloading_headers.extend(headers.iter());
				replace(&mut self.peers.get_mut(&peer_id).unwrap().asking_blocks, headers);
				assert!(!self.headers.have_item(&start));
				self.request_headers_by_number(io, peer_id, start, count, 0, false);
			}
		}
		else {
			// continue search for common block
			self.downloading_headers.insert(start);
			self.request_headers_by_number(io, peer_id, start, 1, 0, false);
		}
	}

	/// Clear all blocks/headers marked as being downloaded by a peer.
	fn clear_peer_download(&mut self, peer_id: PeerId) {
		let peer = self.peers.get_mut(&peer_id).unwrap();
		if let Some(hash) = peer.asking_hash.take() {
			self.downloading_hashes.remove(&hash);
		}
		for b in &peer.asking_blocks {
			self.downloading_headers.remove(&b);
			self.downloading_bodies.remove(&b);
		}
		peer.asking_blocks.clear();
	}

	/// Checks if there are blocks fully downloaded that can be imported into the blockchain and does the import.
	fn collect_blocks(&mut self, io: &mut SyncIo) {
		if !self.have_common_block || self.headers.is_empty() || self.bodies.is_empty() {
			return;
		}

		let mut restart = false;
		// merge headers and bodies
		{
			let headers = self.headers.range_iter().next().unwrap();
			let bodies = self.bodies.range_iter().next().unwrap();
			if headers.0 != bodies.0 || headers.0 > self.current_base_block() + 1 {
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

				// Perform basic block verification
				if !Block::is_good(block_rlp.as_raw()) {
					debug!(target: "sync", "Bad block rlp {:?} : {:?}", h, block_rlp.as_raw());
					restart = true;
					break;
				}

				match io.chain().import_block(block_rlp.out()) {
					Err(Error::Import(ImportError::AlreadyInChain)) => {
						trace!(target: "sync", "Block already in chain {:?}", h);
						self.last_imported_block = Some(headers.0 + i as BlockNumber);
						self.last_imported_hash = Some(h.clone());
					},
					Err(Error::Import(ImportError::AlreadyQueued)) => {
						trace!(target: "sync", "Block already queued {:?}", h);
						self.last_imported_block = Some(headers.0 + i as BlockNumber);
						self.last_imported_hash = Some(h.clone());
					},
					Ok(_) => {
						trace!(target: "sync", "Block queued {:?}", h);
						self.last_imported_block = Some(headers.0 + i as BlockNumber);
						self.last_imported_hash = Some(h.clone());
						imported += 1;
					},
					Err(e) => {
						debug!(target: "sync", "Bad block {:?} : {:?}", h, e);
						restart = true;
					}
				}
			}
			trace!(target: "sync", "Imported {} of {}", imported, count);
		}

		if restart {
			self.restart_on_bad_block(io);
			return;
		}

		self.headers.remove_head(&(self.last_imported_block.unwrap() + 1));
		self.bodies.remove_head(&(self.last_imported_block.unwrap() + 1));

		if self.headers.is_empty() {
			assert!(self.bodies.is_empty());
			self.complete_sync();
		}
	}

	/// Remove downloaded bocks/headers starting from specified number.
	/// Used to recover from an error and re-download parts of the chain detected as bad.
	fn remove_downloaded_blocks(&mut self, start: BlockNumber) {
		let ids = self.header_ids.drain().filter(|&(_, v)| v < start).collect();
		self.header_ids = ids;
		let hdrs = self.downloading_headers.drain().filter(|v| *v < start).collect();
		self.downloading_headers = hdrs;
		let bodies = self.downloading_bodies.drain().filter(|v| *v < start).collect();
		self.downloading_bodies = bodies;
		self.headers.remove_from(&start);
		self.bodies.remove_from(&start);
	}

	/// Request headers from a peer by block hash
	fn request_headers_by_hash(&mut self, sync: &mut SyncIo, peer_id: PeerId, h: &H256, count: usize, skip: usize, reverse: bool) {
		trace!(target: "sync", "{} <- GetBlockHeaders: {} entries starting from {}", peer_id, count, h);
		let mut rlp = RlpStream::new_list(4);
		rlp.append(h);
		rlp.append(&count);
		rlp.append(&skip);
		rlp.append(&if reverse {1u32} else {0u32});
		self.send_request(sync, peer_id, PeerAsking::BlockHeaders, GET_BLOCK_HEADERS_PACKET, rlp.out());
	}

	/// Request headers from a peer by block number
	fn request_headers_by_number(&mut self, sync: &mut SyncIo, peer_id: PeerId, n: BlockNumber, count: usize, skip: usize, reverse: bool) {
		let mut rlp = RlpStream::new_list(4);
		trace!(target: "sync", "{} <- GetBlockHeaders: {} entries starting from {}", peer_id, count, n);
		rlp.append(&n);
		rlp.append(&count);
		rlp.append(&skip);
		rlp.append(&if reverse {1u32} else {0u32});
		self.send_request(sync, peer_id, PeerAsking::BlockHeaders, GET_BLOCK_HEADERS_PACKET, rlp.out());
	}

	/// Request block bodies from a peer
	fn request_bodies(&mut self, sync: &mut SyncIo, peer_id: PeerId, hashes: Vec<H256>) {
		let mut rlp = RlpStream::new_list(hashes.len());
		trace!(target: "sync", "{} <- GetBlockBodies: {} entries", peer_id, hashes.len());
		for h in hashes {
			rlp.append(&h);
		}
		self.send_request(sync, peer_id, PeerAsking::BlockBodies, GET_BLOCK_BODIES_PACKET, rlp.out());
	}

	/// Reset peer status after request is complete.
	fn reset_peer_asking(&mut self, peer_id: PeerId, asking: PeerAsking) {
		let peer = self.peers.get_mut(&peer_id).unwrap();
		if peer.asking != asking {
			warn!(target:"sync", "Asking {:?} while expected {:?}", peer.asking, asking);
		}
		else {
			peer.asking = PeerAsking::Nothing;
		}
	}

	/// Generic request sender
	fn send_request(&mut self, sync: &mut SyncIo, peer_id: PeerId, asking: PeerAsking,  packet_id: PacketId, packet: Bytes) {
		{
			let peer = self.peers.get_mut(&peer_id).unwrap();
			if peer.asking != PeerAsking::Nothing {
				warn!(target:"sync", "Asking {:?} while requesting {:?}", peer.asking, asking);
			}
		}
		match sync.send(peer_id, packet_id, packet) {
			Err(e) => {
				debug!(target:"sync", "Error sending request: {:?}", e);
				sync.disable_peer(peer_id);
			}
			Ok(_) => {
				let mut peer = self.peers.get_mut(&peer_id).unwrap();
				peer.asking = asking;
				peer.ask_time = time::precise_time_s();
			}
		}
	}

	/// Generic packet sender
	fn send_packet(&mut self, sync: &mut SyncIo, peer_id: PeerId, packet_id: PacketId, packet: Bytes) {
		if let Err(e) = sync.send(peer_id, packet_id, packet) {
			debug!(target:"sync", "Error sending packet: {:?}", e);
			sync.disable_peer(peer_id);
		}
	}

	/// Called when peer sends us new transactions
	fn on_peer_transactions(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		// accepting transactions once only fully synced
		if !io.is_chain_queue_empty() {
			return Ok(());
		}

		let item_count = r.item_count();
		trace!(target: "sync", "{} -> Transactions ({} entries)", peer_id, item_count);

		let mut transactions = Vec::with_capacity(item_count);
		for i in 0..item_count {
			let tx: SignedTransaction = try!(r.val_at(i));
			transactions.push(tx);
		}
		let chain = io.chain();
		let fetch_account = |a: &Address| AccountDetails {
			nonce: chain.nonce(a),
			balance: chain.balance(a),
		};
		let _ = self.miner.import_transactions(transactions, fetch_account);
 		Ok(())
	}

	/// Send Status message
	fn send_status(&mut self, io: &mut SyncIo) -> Result<(), UtilError> {
		let mut packet = RlpStream::new_list(5);
		let chain = io.chain().chain_info();
		packet.append(&(PROTOCOL_VERSION as u32));
		packet.append(&self.network_id);
		packet.append(&chain.total_difficulty);
		packet.append(&chain.best_block_hash);
		packet.append(&chain.genesis_hash);
		io.respond(STATUS_PACKET, packet.out())
	}

	/// Respond to GetBlockHeaders request
	fn return_block_headers(io: &SyncIo, r: &UntrustedRlp) -> RlpResponseResult {
		// Packet layout:
		// [ block: { P , B_32 }, maxHeaders: P, skip: P, reverse: P in { 0 , 1 } ]
		let max_headers: usize = try!(r.val_at(1));
		let skip: usize = try!(r.val_at(2));
		let reverse: bool = try!(r.val_at(3));
		let last = io.chain().chain_info().best_block_number;
		let mut number = if try!(r.at(0)).size() == 32 {
			// id is a hash
			let hash: H256 = try!(r.val_at(0));
			trace!(target: "sync", "-> GetBlockHeaders (hash: {}, max: {}, skip: {}, reverse:{})", hash, max_headers, skip, reverse);
			match io.chain().block_header(BlockId::Hash(hash)) {
				Some(hdr) => From::from(HeaderView::new(&hdr).number()),
				None => last
			}
		}
		else {
			trace!(target: "sync", "-> GetBlockHeaders (number: {}, max: {}, skip: {}, reverse:{})", try!(r.val_at::<BlockNumber>(0)), max_headers, skip, reverse);
			try!(r.val_at(0))
		};

		if reverse {
			number = min(last, number);
		} else {
			number = max(0, number);
		}
		let max_count = min(MAX_HEADERS_TO_SEND, max_headers);
		let mut count = 0;
		let mut data = Bytes::new();
		let inc = (skip + 1) as BlockNumber;
		while number <= last && count < max_count {
			if let Some(mut hdr) = io.chain().block_header(BlockId::Number(number)) {
				data.append(&mut hdr);
				count += 1;
			}
			if reverse {
				if number <= inc || number == 0 {
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
		trace!(target: "sync", "-> GetBlockHeaders: returned {} entries", count);
		Ok(Some((BLOCK_HEADERS_PACKET, rlp)))
	}

	/// Respond to GetBlockBodies request
	fn return_block_bodies(io: &SyncIo, r: &UntrustedRlp) -> RlpResponseResult {
		let mut count = r.item_count();
		if count == 0 {
			debug!(target: "sync", "Empty GetBlockBodies request, ignoring.");
			return Ok(None);
		}
		trace!(target: "sync", "-> GetBlockBodies: {} entries", count);
		count = min(count, MAX_BODIES_TO_SEND);
		let mut added = 0usize;
		let mut data = Bytes::new();
		for i in 0..count {
			if let Some(mut hdr) = io.chain().block_body(BlockId::Hash(try!(r.val_at::<H256>(i)))) {
				data.append(&mut hdr);
				added += 1;
			}
		}
		let mut rlp = RlpStream::new_list(added);
		rlp.append_raw(&data, added);
		trace!(target: "sync", "-> GetBlockBodies: returned {} entries", added);
		Ok(Some((BLOCK_BODIES_PACKET, rlp)))
	}

	/// Respond to GetNodeData request
	fn return_node_data(io: &SyncIo, r: &UntrustedRlp) -> RlpResponseResult {
		let mut count = r.item_count();
		if count == 0 {
			debug!(target: "sync", "Empty GetNodeData request, ignoring.");
			return Ok(None);
		}
		count = min(count, MAX_NODE_DATA_TO_SEND);
		let mut added = 0usize;
		let mut data = Bytes::new();
		for i in 0..count {
			if let Some(mut hdr) = io.chain().state_data(&try!(r.val_at::<H256>(i))) {
				data.append(&mut hdr);
				added += 1;
			}
		}
		let mut rlp = RlpStream::new_list(added);
		rlp.append_raw(&data, added);
		Ok(Some((NODE_DATA_PACKET, rlp)))
	}

	fn return_receipts(io: &SyncIo, rlp: &UntrustedRlp) -> RlpResponseResult {
		let mut count = rlp.item_count();
		if count == 0 {
			debug!(target: "sync", "Empty GetReceipts request, ignoring.");
			return Ok(None);
		}
		count = min(count, MAX_RECEIPTS_HEADERS_TO_SEND);
		let mut added_headers = 0usize;
		let mut added_receipts = 0usize;
		let mut data = Bytes::new();
		for i in 0..count {
			if let Some(mut receipts_bytes) = io.chain().block_receipts(&try!(rlp.val_at::<H256>(i))) {
				data.append(&mut receipts_bytes);
				added_receipts += receipts_bytes.len();
				added_headers += 1;
				if added_receipts > MAX_RECEIPTS_TO_SEND { break; }
			}
		}
		let mut rlp_result = RlpStream::new_list(added_headers);
		rlp_result.append_raw(&data, added_headers);
		Ok(Some((RECEIPTS_PACKET, rlp_result)))
	}

	fn return_rlp<FRlp, FError>(&self, io: &mut SyncIo, rlp: &UntrustedRlp, rlp_func: FRlp, error_func: FError) -> Result<(), PacketDecodeError>
		where FRlp : Fn(&SyncIo, &UntrustedRlp) -> RlpResponseResult,
			FError : FnOnce(UtilError) -> String
	{
		let response = rlp_func(io, rlp);
		match response {
			Err(e) => Err(e),
			Ok(Some((packet_id, rlp_stream))) => {
				io.respond(packet_id, rlp_stream.out()).unwrap_or_else(
					|e| debug!(target: "sync", "{:?}", error_func(e)));
				Ok(())
			}
			_ => Ok(())
		}
	}

	/// Dispatch incoming requests and responses
	pub fn on_packet(&mut self, io: &mut SyncIo, peer: PeerId, packet_id: u8, data: &[u8]) {
		let rlp = UntrustedRlp::new(data);

		if packet_id != STATUS_PACKET && !self.peers.contains_key(&peer) {
			debug!(target:"sync", "Unexpected packet from unregistered peer: {}:{}", peer, io.peer_info(peer));
			return;
		}
		let result = match packet_id {
			STATUS_PACKET => self.on_peer_status(io, peer, &rlp),
			TRANSACTIONS_PACKET => self.on_peer_transactions(io, peer, &rlp),
			BLOCK_HEADERS_PACKET => self.on_peer_block_headers(io, peer, &rlp),
			BLOCK_BODIES_PACKET => self.on_peer_block_bodies(io, peer, &rlp),
			NEW_BLOCK_PACKET => self.on_peer_new_block(io, peer, &rlp),
			NEW_BLOCK_HASHES_PACKET => self.on_peer_new_hashes(io, peer, &rlp),

			GET_BLOCK_BODIES_PACKET => self.return_rlp(io, &rlp,
				ChainSync::return_block_bodies,
				|e| format!("Error sending block bodies: {:?}", e)),

			GET_BLOCK_HEADERS_PACKET => self.return_rlp(io, &rlp,
				ChainSync::return_block_headers,
				|e| format!("Error sending block headers: {:?}", e)),

			GET_RECEIPTS_PACKET => self.return_rlp(io, &rlp,
				ChainSync::return_receipts,
				|e| format!("Error sending receipts: {:?}", e)),

			GET_NODE_DATA_PACKET => self.return_rlp(io, &rlp,
				ChainSync::return_node_data,
				|e| format!("Error sending nodes: {:?}", e)),

			_ => {
				debug!(target: "sync", "Unknown packet {}", packet_id);
				Ok(())
			}
		};
		result.unwrap_or_else(|e| {
			debug!(target:"sync", "{} -> Malformed packet {} : {}", peer, packet_id, e);
		})
	}

	pub fn maintain_peers(&self, io: &mut SyncIo) {
		let tick = time::precise_time_s();
		for (peer_id, peer) in &self.peers {
			if peer.asking != PeerAsking::Nothing && (tick - peer.ask_time) > CONNECTION_TIMEOUT_SEC {
				io.disconnect_peer(*peer_id);
			}
		}
	}

	fn check_resume(&mut self, io: &mut SyncIo) {
		if !io.chain().queue_info().is_full() && self.state == SyncState::Waiting {
			self.state = SyncState::Blocks;
			self.continue_sync(io);
		}
	}

	/// creates rlp to send for the tree defined by 'from' and 'to' hashes
	fn create_new_hashes_rlp(chain: &BlockChainClient, from: &H256, to: &H256) -> Option<Bytes> {
		match chain.tree_route(from, to) {
			Some(route) => {
				match route.blocks.len() {
					0 => None,
					_ => {
						let mut rlp_stream = RlpStream::new_list(route.blocks.len());
						for block_hash in route.blocks {
							let mut hash_rlp = RlpStream::new_list(2);
							let difficulty = chain.block_total_difficulty(BlockId::Hash(block_hash.clone())).expect("Mallformed block without a difficulty on the chain!");
							hash_rlp.append(&block_hash);
							hash_rlp.append(&difficulty);
							rlp_stream.append_raw(&hash_rlp.out(), 1);
						}
						Some(rlp_stream.out())
					}
				}
			},
			None => None
		}
	}

	/// creates latest block rlp for the given client
	fn create_latest_block_rlp(chain: &BlockChainClient) -> Bytes {
		let mut rlp_stream = RlpStream::new_list(2);
		rlp_stream.append_raw(&chain.block(BlockId::Hash(chain.chain_info().best_block_hash)).unwrap(), 1);
		rlp_stream.append(&chain.chain_info().total_difficulty);
		rlp_stream.out()
	}

	/// returns peer ids that have less blocks than our chain
	fn get_lagging_peers(&mut self, chain_info: &BlockChainInfo, io: &SyncIo) -> Vec<(PeerId, BlockNumber)> {
		let latest_hash = chain_info.best_block_hash;
		let latest_number = chain_info.best_block_number;
		self.peers.iter_mut().filter_map(|(&id, ref mut peer_info)|
			match io.chain().block_status(BlockId::Hash(peer_info.latest_hash.clone())) {
				BlockStatus::InChain => {
					if peer_info.latest_number.is_none() {
						peer_info.latest_number = Some(HeaderView::new(&io.chain().block_header(BlockId::Hash(peer_info.latest_hash.clone())).unwrap()).number());
					}
					if peer_info.latest_hash != latest_hash && latest_number > peer_info.latest_number.unwrap() {
						Some((id, peer_info.latest_number.unwrap()))
					} else { None }
				},
				_ => None
			})
			.collect::<Vec<_>>()
	}

	/// propagates latest block to lagging peers
	fn propagate_blocks(&mut self, chain_info: &BlockChainInfo, io: &mut SyncIo) -> usize {
		let updated_peers = {
			let lagging_peers = self.get_lagging_peers(chain_info, io);

			// sqrt(x)/x scaled to max u32
			let fraction = (self.peers.len() as f64).powf(-0.5).mul(u32::max_value() as f64).round() as u32;
			let lucky_peers = match lagging_peers.len() {
				0 ... MIN_PEERS_PROPAGATION => lagging_peers,
				_ => lagging_peers.into_iter().filter(|_| ::rand::random::<u32>() < fraction).collect::<Vec<_>>()
			};

			// taking at max of MAX_PEERS_PROPAGATION
			lucky_peers.iter().map(|&(id, _)| id.clone()).take(min(lucky_peers.len(), MAX_PEERS_PROPAGATION)).collect::<Vec<PeerId>>()
		};

		let mut sent = 0;
		for peer_id in updated_peers {
			let rlp = ChainSync::create_latest_block_rlp(io.chain());
			self.send_packet(io, peer_id, NEW_BLOCK_PACKET, rlp);
			self.peers.get_mut(&peer_id).unwrap().latest_hash = chain_info.best_block_hash.clone();
			self.peers.get_mut(&peer_id).unwrap().latest_number = Some(chain_info.best_block_number);
			sent = sent + 1;
		}
		sent
	}

	/// propagates new known hashes to all peers
	fn propagate_new_hashes(&mut self, chain_info: &BlockChainInfo, io: &mut SyncIo) -> usize {
		let updated_peers = self.get_lagging_peers(chain_info, io);
		let mut sent = 0;
		let last_parent = HeaderView::new(&io.chain().block_header(BlockId::Hash(chain_info.best_block_hash.clone())).unwrap()).parent_hash();
		for (peer_id, peer_number) in updated_peers {
			let mut peer_best = self.peers.get(&peer_id).unwrap().latest_hash.clone();
			if chain_info.best_block_number - peer_number > MAX_PEERS_PROPAGATION as BlockNumber {
				// If we think peer is too far behind just send one latest hash
				peer_best = last_parent.clone();
			}
			sent = sent + match ChainSync::create_new_hashes_rlp(io.chain(), &peer_best, &chain_info.best_block_hash) {
				Some(rlp) => {
					{
						let peer = self.peers.get_mut(&peer_id).unwrap();
						peer.latest_hash = chain_info.best_block_hash.clone();
						peer.latest_number = Some(chain_info.best_block_number);
					}
					self.send_packet(io, peer_id, NEW_BLOCK_HASHES_PACKET, rlp);
					1
				},
				None => 0
			}
		}
		sent
	}

	/// propagates new transactions to all peers
	fn propagate_new_transactions(&mut self, io: &mut SyncIo) -> usize {

		// Early out of nobody to send to.
		if self.peers.is_empty() {
			return 0;
		}

		let mut transactions = self.miner.pending_transactions();
		if transactions.is_empty() {
			return 0;
		}

		let mut packet = RlpStream::new_list(transactions.len());
		let tx_count = transactions.len();
		for tx in transactions.drain(..) {
			packet.append(&tx);
		}
		let rlp = packet.out();

		let lucky_peers = {
			// sqrt(x)/x scaled to max u32
			let fraction = (self.peers.len() as f64).powf(-0.5).mul(u32::max_value() as f64).round() as u32;
			let small = self.peers.len() < MIN_PEERS_PROPAGATION;
			let lucky_peers = self.peers.iter()
				.filter_map(|(&p, _)| if small || ::rand::random::<u32>() < fraction { Some(p.clone()) } else { None })
				.collect::<Vec<_>>();

			// taking at max of MAX_PEERS_PROPAGATION
			lucky_peers.iter().cloned().take(min(lucky_peers.len(), MAX_PEERS_PROPAGATION)).collect::<Vec<PeerId>>()
		};

		let sent = lucky_peers.len();
		for peer_id in lucky_peers {
			self.send_packet(io, peer_id, TRANSACTIONS_PACKET, rlp.clone());
		}
		trace!(target: "sync", "Sent {} transactions to {} peers.", tx_count, sent);
		sent
	}

	fn propagate_latest_blocks(&mut self, io: &mut SyncIo) {
		self.propagate_new_transactions(io);
		let chain_info = io.chain().chain_info();
		if (((chain_info.best_block_number as i64) - (self.last_sent_block_number as i64)).abs() as BlockNumber) < MAX_PEER_LAG_PROPAGATION {
			let blocks = self.propagate_blocks(&chain_info, io);
			let hashes = self.propagate_new_hashes(&chain_info, io);
			if blocks != 0 || hashes != 0 {
				trace!(target: "sync", "Sent latest {} blocks and {} hashes to peers.", blocks, hashes);
			}
		}
		self.last_sent_block_number = chain_info.best_block_number;
	}

	/// Maintain other peers. Send out any new blocks and transactions
	pub fn maintain_sync(&mut self, io: &mut SyncIo) {
		self.check_resume(io);
	}

	/// called when block is imported to chain, updates transactions queue and propagates the blocks
	pub fn chain_new_blocks(&mut self, io: &mut SyncIo, imported: &[H256], invalid: &[H256], enacted: &[H256], retracted: &[H256]) {
		if io.is_chain_queue_empty() {
			// Notify miner
			self.miner.chain_new_blocks(io.chain(), imported, invalid, enacted, retracted);
			// Propagate latests blocks
			self.propagate_latest_blocks(io);
		}
		if !invalid.is_empty() {
			trace!(target: "sync", "Bad blocks in the queue, restarting");
			self.restart_on_bad_block(io);
		}
		// TODO [todr] propagate transactions?
	}

	pub fn chain_new_head(&mut self, io: &mut SyncIo) {
		self.miner.update_sealing(io.chain());
	}
}

#[cfg(test)]
mod tests {
	use tests::helpers::*;
	use super::*;
	use ::SyncConfig;
	use util::*;
	use super::{PeerInfo, PeerAsking};
	use ethcore::views::BlockView;
	use ethcore::header::*;
	use ethcore::client::*;
	use ethminer::{Miner, MinerService};

	fn get_dummy_block(order: u32, parent_hash: H256) -> Bytes {
		let mut header = Header::new();
		header.gas_limit = x!(0);
		header.difficulty = x!(order * 100);
		header.timestamp = (order * 10) as u64;
		header.number = order as u64;
		header.parent_hash = parent_hash;
		header.state_root = H256::zero();

		let mut rlp = RlpStream::new_list(3);
		rlp.append(&header);
		rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
		rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
		rlp.out()
	}

	fn get_dummy_blocks(order: u32, parent_hash: H256) -> Bytes {
		let mut rlp = RlpStream::new_list(1);
		rlp.append_raw(&get_dummy_block(order, parent_hash), 1);
		let difficulty: U256 = x!(100 * order);
		rlp.append(&difficulty);
		rlp.out()
	}

	fn get_dummy_hashes() -> Bytes {
		let mut rlp = RlpStream::new_list(5);
		for _ in 0..5 {
			let mut hash_d_rlp = RlpStream::new_list(2);
			let hash: H256 = H256::from(0u64);
			let diff: U256 = U256::from(1u64);
			hash_d_rlp.append(&hash);
			hash_d_rlp.append(&diff);

			rlp.append_raw(&hash_d_rlp.out(), 1);
		}

		rlp.out()
	}

	#[test]
	fn return_receipts_empty() {
		let mut client = TestBlockChainClient::new();
		let mut queue = VecDeque::new();
		let io = TestIo::new(&mut client, &mut queue, None);

		let result = ChainSync::return_receipts(&io, &UntrustedRlp::new(&[0xc0]));

		assert!(result.is_ok());
	}

	#[test]
	fn return_receipts() {
		let mut client = TestBlockChainClient::new();
		let mut queue = VecDeque::new();
		let mut io = TestIo::new(&mut client, &mut queue, None);

		let mut receipt_list = RlpStream::new_list(4);
		receipt_list.append(&H256::from("0000000000000000000000000000000000000000000000005555555555555555"));
		receipt_list.append(&H256::from("ff00000000000000000000000000000000000000000000000000000000000000"));
		receipt_list.append(&H256::from("fff0000000000000000000000000000000000000000000000000000000000000"));
		receipt_list.append(&H256::from("aff0000000000000000000000000000000000000000000000000000000000000"));

		let receipts_request = receipt_list.out();
		// it returns rlp ONLY for hashes started with "f"
		let result = ChainSync::return_receipts(&io, &UntrustedRlp::new(&receipts_request.clone()));

		assert!(result.is_ok());
		let rlp_result = result.unwrap();
		assert!(rlp_result.is_some());

		// the length of two rlp-encoded receipts
		assert_eq!(603, rlp_result.unwrap().1.out().len());

		let mut sync = dummy_sync_with_peer(H256::new());
		io.sender = Some(2usize);
		sync.on_packet(&mut io, 0usize, super::GET_RECEIPTS_PACKET, &receipts_request);
		assert_eq!(1, io.queue.len());
	}

	#[test]
	fn return_nodes() {
		let mut client = TestBlockChainClient::new();
		let mut queue = VecDeque::new();
		let mut io = TestIo::new(&mut client, &mut queue, None);

		let mut node_list = RlpStream::new_list(3);
		node_list.append(&H256::from("0000000000000000000000000000000000000000000000005555555555555555"));
		node_list.append(&H256::from("ffffffffffffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaa"));
		node_list.append(&H256::from("aff0000000000000000000000000000000000000000000000000000000000000"));

		let node_request = node_list.out();
		// it returns rlp ONLY for hashes started with "f"
		let result = ChainSync::return_node_data(&io, &UntrustedRlp::new(&node_request.clone()));

		assert!(result.is_ok());
		let rlp_result = result.unwrap();
		assert!(rlp_result.is_some());

		// the length of one rlp-encoded hashe
		assert_eq!(34, rlp_result.unwrap().1.out().len());

		let mut sync = dummy_sync_with_peer(H256::new());
		io.sender = Some(2usize);
		sync.on_packet(&mut io, 0usize, super::GET_NODE_DATA_PACKET, &node_request);
		assert_eq!(1, io.queue.len());
	}

	fn dummy_sync_with_peer(peer_latest_hash: H256) -> ChainSync {
		let mut sync = ChainSync::new(SyncConfig::default(), Miner::new(false));
		sync.peers.insert(0,
		  	PeerInfo {
				protocol_version: 0,
				genesis: H256::zero(),
				network_id: U256::zero(),
				latest_hash: peer_latest_hash,
				latest_number: None,
				difficulty: U256::zero(),
				asking: PeerAsking::Nothing,
				asking_blocks: Vec::<BlockNumber>::new(),
				asking_hash: None,
				ask_time: 0f64,
	  		});
		sync
	}

	#[test]
	fn finds_lagging_peers() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(10));
		let chain_info = client.chain_info();
		let io = TestIo::new(&mut client, &mut queue, None);

		let lagging_peers = sync.get_lagging_peers(&chain_info, &io);

		assert_eq!(1, lagging_peers.len())
	}

	#[test]
	fn calculates_tree_for_lagging_peer() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(15, EachBlockWith::Uncle);

		let start = client.block_hash_delta_minus(4);
		let end = client.block_hash_delta_minus(2);

		// wrong way end -> start, should be None
		let rlp = ChainSync::create_new_hashes_rlp(&client, &end, &start);
		assert!(rlp.is_none());

		let rlp = ChainSync::create_new_hashes_rlp(&client, &start, &end).unwrap();
		// size of three rlp encoded hash-difficulty
		assert_eq!(107, rlp.len());
	}

	#[test]
	fn sends_new_hashes_to_lagging_peer() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		let chain_info = client.chain_info();
		let mut io = TestIo::new(&mut client, &mut queue, None);

		let peer_count = sync.propagate_new_hashes(&chain_info, &mut io);

		// 1 message should be send
		assert_eq!(1, io.queue.len());
		// 1 peer should be updated
		assert_eq!(1, peer_count);
		// NEW_BLOCK_HASHES_PACKET
		assert_eq!(0x01, io.queue[0].packet_id);
	}

	#[test]
	fn sends_latest_block_to_lagging_peer() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		let chain_info = client.chain_info();
		let mut io = TestIo::new(&mut client, &mut queue, None);
		let peer_count = sync.propagate_blocks(&chain_info, &mut io);

		// 1 message should be send
		assert_eq!(1, io.queue.len());
		// 1 peer should be updated
		assert_eq!(1, peer_count);
		// NEW_BLOCK_PACKET
		assert_eq!(0x07, io.queue[0].packet_id);
	}

	#[test]
	fn handles_peer_new_block_mallformed() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);

		let block_data = get_dummy_block(11, client.chain_info().best_block_hash);

		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		sync.have_common_block = true;
		let mut io = TestIo::new(&mut client, &mut queue, None);

		let block = UntrustedRlp::new(&block_data);

		let result = sync.on_peer_new_block(&mut io, 0, &block);

		assert!(result.is_err());
	}

	#[test]
	fn handles_peer_new_block() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);

		let block_data = get_dummy_blocks(11, client.chain_info().best_block_hash);

		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		let mut io = TestIo::new(&mut client, &mut queue, None);

		let block = UntrustedRlp::new(&block_data);

		let result = sync.on_peer_new_block(&mut io, 0, &block);

		assert!(result.is_ok());
	}

	#[test]
	fn handles_peer_new_block_empty() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		let mut io = TestIo::new(&mut client, &mut queue, None);

		let empty_data = vec![];
		let block = UntrustedRlp::new(&empty_data);

		let result = sync.on_peer_new_block(&mut io, 0, &block);

		assert!(result.is_err());
	}

	#[test]
	fn handles_peer_new_hashes() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		let mut io = TestIo::new(&mut client, &mut queue, None);

		let hashes_data = get_dummy_hashes();
		let hashes_rlp = UntrustedRlp::new(&hashes_data);

		let result = sync.on_peer_new_hashes(&mut io, 0, &hashes_rlp);

		assert!(result.is_ok());
	}

	#[test]
	fn handles_peer_new_hashes_empty() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		let mut io = TestIo::new(&mut client, &mut queue, None);

		let empty_hashes_data = vec![];
		let hashes_rlp = UntrustedRlp::new(&empty_hashes_data);

		let result = sync.on_peer_new_hashes(&mut io, 0, &hashes_rlp);

		assert!(result.is_ok());
	}

	// idea is that what we produce when propagading latest hashes should be accepted in
	// on_peer_new_hashes in our code as well
	#[test]
	fn hashes_rlp_mutually_acceptable() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		let chain_info = client.chain_info();
		let mut io = TestIo::new(&mut client, &mut queue, None);

		sync.propagate_new_hashes(&chain_info, &mut io);

		let data = &io.queue[0].data.clone();
		let result = sync.on_peer_new_hashes(&mut io, 0, &UntrustedRlp::new(&data));
		assert!(result.is_ok());
	}

	// idea is that what we produce when propagading latest block should be accepted in
	// on_peer_new_block  in our code as well
	#[test]
	fn block_rlp_mutually_acceptable() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));
		let chain_info = client.chain_info();
		let mut io = TestIo::new(&mut client, &mut queue, None);

		sync.propagate_blocks(&chain_info, &mut io);

		let data = &io.queue[0].data.clone();
		let result = sync.on_peer_new_block(&mut io, 0, &UntrustedRlp::new(&data));
		assert!(result.is_ok());
	}

	#[test]
	fn should_add_transactions_to_queue() {
		// given
		let mut client = TestBlockChainClient::new();
		client.add_blocks(98, EachBlockWith::Uncle);
		client.add_blocks(1, EachBlockWith::UncleAndTransaction);
		client.add_blocks(1, EachBlockWith::Transaction);
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));

		let good_blocks = vec![client.block_hash_delta_minus(2)];
		let retracted_blocks = vec![client.block_hash_delta_minus(1)];

		// Add some balance to clients and reset nonces
		for h in &[good_blocks[0], retracted_blocks[0]] {
			let block = client.block(BlockId::Hash(*h)).unwrap();
			let view = BlockView::new(&block);
			client.set_balance(view.transactions()[0].sender().unwrap(), U256::from(1_000_000_000));
			client.set_nonce(view.transactions()[0].sender().unwrap(), U256::from(0));
		}


		// when
		{
			let mut queue = VecDeque::new();
			let mut io = TestIo::new(&mut client, &mut queue, None);
			sync.chain_new_blocks(&mut io, &[], &[], &[], &good_blocks);
			assert_eq!(sync.miner.status().transactions_in_future_queue, 0);
			assert_eq!(sync.miner.status().transactions_in_pending_queue, 1);
		}
		// We need to update nonce status (because we say that the block has been imported)
		for h in &[good_blocks[0]] {
			let block = client.block(BlockId::Hash(*h)).unwrap();
			let view = BlockView::new(&block);
			client.set_nonce(view.transactions()[0].sender().unwrap(), U256::from(1));
		}
		{
			let mut queue = VecDeque::new();
			let mut io = TestIo::new(&mut client, &mut queue, None);
			sync.chain_new_blocks(&mut io, &[], &[], &good_blocks, &retracted_blocks);
		}

		// then
		let status = sync.miner.status();
		assert_eq!(status.transactions_in_pending_queue, 1);
		assert_eq!(status.transactions_in_future_queue, 0);
	}

	#[test]
	fn should_not_add_transactions_to_queue_if_not_synced() {
		// given
		let mut client = TestBlockChainClient::new();
		client.add_blocks(98, EachBlockWith::Uncle);
		client.add_blocks(1, EachBlockWith::UncleAndTransaction);
		client.add_blocks(1, EachBlockWith::Transaction);
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5));

		let good_blocks = vec![client.block_hash_delta_minus(2)];
		let retracted_blocks = vec![client.block_hash_delta_minus(1)];

		let mut queue = VecDeque::new();
		let mut io = TestIo::new(&mut client, &mut queue, None);

		// when
		sync.chain_new_blocks(&mut io, &[], &[], &[], &good_blocks);
		assert_eq!(sync.miner.status().transactions_in_future_queue, 0);
		assert_eq!(sync.miner.status().transactions_in_pending_queue, 0);
		sync.chain_new_blocks(&mut io, &[], &[], &good_blocks, &retracted_blocks);

		// then
		let status = sync.miner.status();
		assert_eq!(status.transactions_in_pending_queue, 0);
		assert_eq!(status.transactions_in_future_queue, 0);
	}

	#[test]
	fn returns_requested_block_headers() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let io = TestIo::new(&mut client, &mut queue, None);

		let mut rlp = RlpStream::new_list(4);
		rlp.append(&0u64);
		rlp.append(&10u64);
		rlp.append(&0u64);
		rlp.append(&0u64);
		let data = rlp.out();

		let response = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&data));

		assert!(response.is_ok());
		let (_, rlp_stream) = response.unwrap().unwrap();
		let response_data = rlp_stream.out();
		let rlp = UntrustedRlp::new(&response_data);
		assert!(rlp.at(0).is_ok());
		assert!(rlp.at(9).is_ok());
	}

	#[test]
	fn returns_requested_block_headers_reverse() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let io = TestIo::new(&mut client, &mut queue, None);

		let mut rlp = RlpStream::new_list(4);
		rlp.append(&15u64);
		rlp.append(&15u64);
		rlp.append(&0u64);
		rlp.append(&1u64);
		let data = rlp.out();

		let response = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&data));

		assert!(response.is_ok());
		let (_, rlp_stream) = response.unwrap().unwrap();
		let response_data = rlp_stream.out();
		let rlp = UntrustedRlp::new(&response_data);
		assert!(rlp.at(0).is_ok());
		assert!(rlp.at(14).is_ok());
		assert!(!rlp.at(15).is_ok());
	}
}
