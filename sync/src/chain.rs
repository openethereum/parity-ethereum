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
/// Syncing strategy summary.
/// Split the chain into ranges of N blocks each. Download ranges sequentially. Split each range into subchains of M blocks. Download subchains in parallel.
/// State.
/// Sync state consists of the following data:
/// - s: State enum which can be one of the following values: `ChainHead`, `Blocks`, `Idle`
/// - H: A set of downloaded block headers
/// - B: A set of downloaded block bodies
/// - S: Set of block subchain start block hashes to download.
/// - l: Last imported / common block hash
/// - P: A set of connected peers. For each peer we maintain its last known total difficulty and starting block hash being requested if any.
/// General behaviour.
/// We start with all sets empty, l is set to the best block in the block chain, s is set to `ChainHead`.
/// If at any moment a bad block is reported by the block queue, we set s to `ChainHead`, reset l to the best block in the block chain and clear H, B and S.
/// If at any moment P becomes empty, we set s to `ChainHead`, and clear H, B and S.
///
/// Workflow for `ChainHead` state.
/// In this state we try to get subchain headers with a single `GetBlockHeaders` request.
/// On `NewPeer` / On `Restart`:
/// 	If peer's total difficulty is higher, request N/M headers with interval M+1 starting from l
/// On `BlockHeaders(R)`:
/// 	If R is empty:
/// If l is equal to genesis block hash or l is more than 1000 blocks behind our best hash:
/// Remove current peer from P. set l to the best block in the block chain. Select peer with maximum total difficulty from P and restart.
/// Else
/// 	Set l to l’s parent and restart.
/// Else if we already have all the headers in the block chain or the block queue:
/// 	Set s to `Idle`,
/// Else
/// 	Set S to R, set s to `Blocks`.
///
///
/// All other messages are ignored.
/// Workflow for `Blocks` state.
/// In this state we download block headers and bodies from multiple peers.
/// On `NewPeer` / On `Restart`:
/// 	For all idle peers:
/// Find a set of 256 or less block hashes in H which are not in B and not being downloaded by other peers. If the set is not empty:
///  	Request block bodies for the hashes in the set.
/// Else
/// 	Find an element in S which is  not being downloaded by other peers. If found: Request M headers starting from the element.
///
/// On `BlockHeaders(R)`:
/// If R is empty remove current peer from P and restart.
/// 	Validate received headers. For each header find a parent in H or R or the blockchain. Restart if there is a block with unknown parent.
/// Go to `CollectBlocks`.
///
/// On `BlockBodies(R)`:
/// If R is empty remove current peer from P and restart.
/// 	Add bodies with a matching header in H to B.
/// 	Go to `CollectBlocks`.
///
/// `CollectBlocks`:
/// Find a chain of blocks C in H starting from h where h’s parent equals to l. The chain ends with the first block which does not have a body in B.
/// Add all blocks from the chain to the block queue. Remove them from H and B. Set l to the hash of the last block from C.
/// Update and merge subchain heads in S. For each h in S find a chain of blocks in B starting from h. Remove h from S. if the chain does not include an element from S add the end of the chain to S.
/// If H is empty and S contains a single element set s to `ChainHead`.
/// Restart.
///
/// All other messages are ignored.
/// Workflow for Idle state.
/// On `NewBlock`:
/// 	Import the block. If the block is unknown set s to `ChainHead` and restart.
/// On `NewHashes`:
/// 	Set s to `ChainHead` and restart.
///
/// All other messages are ignored.
///

use util::*;
use std::mem::{replace};
use ethcore::views::{HeaderView, BlockView};
use ethcore::header::{BlockNumber, Header as BlockHeader};
use ethcore::client::{BlockChainClient, BlockStatus, BlockID, BlockChainInfo};
use ethcore::error::*;
use ethcore::transaction::SignedTransaction;
use ethcore::block::Block;
use ethminer::{Miner, MinerService, AccountDetails};
use io::SyncIo;
use time;
use super::SyncConfig;
use blocks::BlockCollection;

known_heap_size!(0, PeerInfo);

type PacketDecodeError = DecoderError;

const PROTOCOL_VERSION: u8 = 63u8;
const MAX_BODIES_TO_SEND: usize = 256;
const MAX_HEADERS_TO_SEND: usize = 512;
const MAX_NODE_DATA_TO_SEND: usize = 1024;
const MAX_RECEIPTS_TO_SEND: usize = 1024;
const MAX_RECEIPTS_HEADERS_TO_SEND: usize = 256;
const MAX_HEADERS_TO_REQUEST: usize = 256;
const MAX_BODIES_TO_REQUEST: usize = 64;
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

const CONNECTION_TIMEOUT_SEC: f64 = 10f64;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
/// Sync state
pub enum SyncState {
	/// Downloading subchain heads
	ChainHead,
	/// Initial chain sync complete. Waiting for new packets
	Idle,
	/// Block downloading paused. Waiting for block queue to process blocks and free some space
	Waiting,
	/// Downloading blocks
	Blocks,
	/// Downloading blocks learned from `NewHashes` packet
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
	Heads,
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
	/// Peer total difficulty if known
	difficulty: Option<U256>,
	/// Type of data currenty being requested from peer.
	asking: PeerAsking,
	/// A set of block numbers being requested
	asking_blocks: Vec<H256>,
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
	/// All connected peers
	peers: HashMap<PeerId, PeerInfo>,
	/// Peers active for current sync round
	active_peers: HashSet<PeerId>,
	/// Downloaded blocks, holds `H`, `B` and `S`
	blocks: BlockCollection,
	/// Last impoted block number
	last_imported_block: BlockNumber,
	/// Last impoted block hash
	last_imported_hash: H256,
	/// Syncing total  difficulty
	syncing_difficulty: U256,
	/// Last propagated block number
	last_sent_block_number: BlockNumber,
	/// Max blocks to download ahead
	_max_download_ahead_blocks: usize,
	/// Number of blocks imported this round
	imported_this_round: Option<usize>,
	/// Network ID
	network_id: U256,
	/// Miner
	miner: Arc<Miner>,
}

type RlpResponseResult = Result<Option<(PacketId, RlpStream)>, PacketDecodeError>;

impl ChainSync {
	/// Create a new instance of syncing strategy.
	pub fn new(config: SyncConfig, miner: Arc<Miner>, chain: &BlockChainClient) -> ChainSync {
		let chain = chain.chain_info();
		let mut sync = ChainSync {
			state: SyncState::ChainHead,
			starting_block: chain.best_block_number,
			highest_block: None,
			last_imported_block: chain.best_block_number,
			last_imported_hash: chain.best_block_hash,
			peers: HashMap::new(),
			active_peers: HashSet::new(),
			blocks: BlockCollection::new(),
			syncing_difficulty: U256::from(0u64),
			last_sent_block_number: 0,
			imported_this_round: None,
			_max_download_ahead_blocks: max(MAX_HEADERS_TO_REQUEST, config.max_download_ahead_blocks),
			network_id: config.network_id,
			miner: miner,
		};
		sync.reset();
		sync
	}

	/// @returns Synchonization status
	pub fn status(&self) -> SyncStatus {
		SyncStatus {
			state: self.state.clone(),
			protocol_version: 63,
			network_id: self.network_id,
			start_block_number: self.starting_block,
			last_imported_block_number: Some(self.last_imported_block),
			highest_block_number: self.highest_block.map(|n| max(n, self.last_imported_block)),
			blocks_received: if self.last_imported_block > self.starting_block { self.last_imported_block - self.starting_block } else { 0 },
			blocks_total: match self.highest_block { Some(x) if x > self.starting_block => x - self.starting_block, _ => 0 },
			num_peers: self.peers.len(),
			num_active_peers: self.peers.values().filter(|p| p.asking != PeerAsking::Nothing).count(),
			mem_used:
				//  TODO: https://github.com/servo/heapsize/pull/50
				//+ self.downloading_bodies.heap_size_of_children()
				//+ self.downloading_headers.heap_size_of_children()
				self.blocks.heap_size()
				+ self.peers.heap_size_of_children(),
		}
	}

	/// Abort all sync activity
	pub fn abort(&mut self, io: &mut SyncIo) {
		self.restart(io);
		self.peers.clear();
	}

	#[cfg_attr(feature="dev", allow(for_kv_map))] // Because it's not possible to get `values_mut()`
	/// Reset sync. Clear all downloaded data but keep the queue
	fn reset(&mut self) {
		self.blocks.clear();
		for (_, ref mut p) in &mut self.peers {
			p.asking_blocks.clear();
			p.asking_hash = None;
		}
		self.syncing_difficulty = From::from(0u64);
		self.state = SyncState::Idle;
		self.blocks.clear();
		self.active_peers = self.peers.keys().cloned().collect();
	}

	/// Restart sync
	pub fn restart(&mut self, io: &mut SyncIo) {
		trace!(target: "sync", "Restarting");
		self.reset();
		self.start_sync_round(io);
		self.continue_sync(io);
	}

	/// Remove peer from active peer set
	fn deactivate_peer(&mut self, io: &mut SyncIo, peer_id: PeerId) {
		self.active_peers.remove(&peer_id);
		if self.active_peers.is_empty() {
			trace!(target: "sync", "No more active peers");
			if self.state == SyncState::ChainHead {
				self.complete_sync();
			} else {
				self.restart(io);
			}
		}
	}

	/// Restart sync after bad block has been detected. May end up re-downloading up to QUEUE_SIZE blocks
	fn restart_on_bad_block(&mut self, io: &mut SyncIo) {
		// Do not assume that the block queue/chain still has our last_imported_block
		let chain = io.chain().chain_info();
		self.last_imported_block = chain.best_block_number;
		self.last_imported_hash = chain.best_block_hash;
		self.restart(io);
	}

	/// Called by peer to report status
	fn on_peer_status(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		let peer = PeerInfo {
			protocol_version: try!(r.val_at(0)),
			network_id: try!(r.val_at(1)),
			difficulty: Some(try!(r.val_at(2))),
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
			trace!(target: "sync", "Peer {} genesis hash mismatch (ours: {}, theirs: {})", peer_id, chain_info.genesis_hash, peer.genesis);
			return Ok(());
		}
		if peer.network_id != self.network_id {
			io.disable_peer(peer_id);
			trace!(target: "sync", "Peer {} network id mismatch (ours: {}, theirs: {})", peer_id, self.network_id, peer.network_id);
			return Ok(());
		}

		self.peers.insert(peer_id.clone(), peer);
		self.active_peers.insert(peer_id.clone());
		debug!(target: "sync", "Connected {}:{}", peer_id, io.peer_info(peer_id));
		self.sync_peer(io, peer_id, false);
		Ok(())
	}

	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	/// Called by peer once it has new block headers during sync
	fn on_peer_block_headers(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		self.clear_peer_download(peer_id);
		let expected_asking = if self.state == SyncState::ChainHead { PeerAsking::Heads } else { PeerAsking::BlockHeaders };
		if !self.reset_peer_asking(peer_id, expected_asking) {
			trace!(target: "sync", "Ignored unexpected headers");
			self.continue_sync(io);
			return Ok(());
		}
		let item_count = r.item_count();
		trace!(target: "sync", "{} -> BlockHeaders ({} entries)", peer_id, item_count);
		if self.state == SyncState::Idle {
			trace!(target: "sync", "Ignored unexpected block headers");
			self.continue_sync(io);
			return Ok(());
		}
		if self.state == SyncState::Waiting {
			trace!(target: "sync", "Ignored block headers while waiting");
			self.continue_sync(io);
			return Ok(());
		}
		if item_count == 0 && (self.state == SyncState::Blocks || self.state == SyncState::NewBlocks) {
			self.deactivate_peer(io, peer_id); //TODO: is this too harsh?
			return Ok(());
		}

		let mut headers = Vec::new();
		let mut hashes = Vec::new();
		for i in 0..item_count {
			let info: BlockHeader = try!(r.val_at(i));
			let number = BlockNumber::from(info.number);
			if self.blocks.contains(&info.hash()) {
				trace!(target: "sync", "Skipping existing block header {} ({:?})", number, info.hash());
				continue;
			}

			if self.highest_block == None || number > self.highest_block.unwrap() {
				self.highest_block = Some(number);
			}
			let hash = info.hash();
			match io.chain().block_status(BlockID::Hash(hash.clone())) {
				BlockStatus::InChain | BlockStatus::Queued => {
					match self.state {
						SyncState::Blocks | SyncState::NewBlocks => trace!(target: "sync", "Header already in chain {} ({})", number, hash),
						_ => trace!(target: "sync", "Unexpected header already in chain {} ({}), state = {:?}", number, hash, self.state),
					}
					headers.push(try!(r.at(i)).as_raw().to_vec());
					hashes.push(hash);
				},
				BlockStatus::Bad => {
					warn!(target: "sync", "Bad header {} ({}) from {}: {}, state = {:?}", number, hash, peer_id, io.peer_info(peer_id), self.state);
					io.disable_peer(peer_id);
					return Ok(());
				},
				BlockStatus::Unknown => {
					headers.push(try!(r.at(i)).as_raw().to_vec());
					hashes.push(hash);
				}
			}
		}

		match self.state {
			SyncState::ChainHead => {
				if headers.is_empty() {
					// peer is not on our chain
					// track back and try again
					self.imported_this_round = Some(0);
					self.start_sync_round(io);
				} else {
					// TODO: validate heads better. E.g. check that there is enough distance between blocks.
					trace!(target: "sync", "Received {} subchain heads, proceeding to download", headers.len());
					self.blocks.reset_to(hashes);
					self.state = SyncState::Blocks;
				}
			},
			SyncState::Blocks | SyncState::NewBlocks | SyncState::Waiting => {
				trace!(target: "sync", "Inserted {} headers", headers.len());
				self.blocks.insert_headers(headers);
			},
			_ => trace!(target: "sync", "Unexpected headers({}) from  {} ({}), state = {:?}", headers.len(), peer_id, io.peer_info(peer_id), self.state)
		}

		self.collect_blocks(io);
		self.continue_sync(io);
		Ok(())
	}

	/// Called by peer once it has new block bodies
	fn on_peer_block_bodies(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		self.clear_peer_download(peer_id);
		self.reset_peer_asking(peer_id, PeerAsking::BlockBodies);
		let item_count = r.item_count();
		trace!(target: "sync", "{} -> BlockBodies ({} entries)", peer_id, item_count);
		if item_count == 0 {
			self.deactivate_peer(io, peer_id);
		}
		else if self.state != SyncState::Blocks && self.state != SyncState::NewBlocks && self.state != SyncState::Waiting {
			trace!(target: "sync", "Ignored unexpected block bodies");
		}
		else if self.state == SyncState::Waiting {
			trace!(target: "sync", "Ignored block bodies while waiting");
		}
		else
		{
			let mut bodies = Vec::with_capacity(item_count);
			for i in 0..item_count {
				bodies.push(try!(r.at(i)).as_raw().to_vec());
			}
			self.blocks.insert_bodies(bodies);
			self.collect_blocks(io);
		}
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
		if self.state != SyncState::Idle {
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
		if header.number <= self.last_imported_block + 1 {
			match io.chain().import_block(block_rlp.as_raw().to_vec()) {
				Err(Error::Import(ImportError::AlreadyInChain)) => {
					trace!(target: "sync", "New block already in chain {:?}", h);
				},
				Err(Error::Import(ImportError::AlreadyQueued)) => {
					trace!(target: "sync", "New block already queued {:?}", h);
				},
				Ok(_) => {
					if header.number == self.last_imported_block + 1 {
						self.last_imported_block = header.number;
						self.last_imported_hash = header.hash();
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
			trace!(target: "sync", "New unknown block {:?}", h);
			//TODO: handle too many unknown blocks
			let difficulty: U256 = try!(r.val_at(1));
			if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
				if peer.difficulty.map_or(true, |pd| difficulty > pd) {
					//self.state = SyncState::ChainHead;
					peer.difficulty = Some(difficulty);
					trace!(target: "sync", "Received block {:?}  with no known parent. Peer needs syncing...", h);
				}
			}
			self.sync_peer(io, peer_id, true);
		}
		Ok(())
	}

	/// Handles `NewHashes` packet. Initiates headers download for any unknown hashes.
	fn on_peer_new_hashes(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		if self.state != SyncState::Idle {
			trace!(target: "sync", "Ignoring new hashes since we're already downloading.");
			return Ok(());
		}
		trace!(target: "sync", "{} -> NewHashes ({} entries)", peer_id, r.item_count());
		let hashes = r.iter().map(|item| (item.val_at::<H256>(0), item.val_at::<BlockNumber>(1)));
		let mut max_height: BlockNumber = 0;
		let mut new_hashes = Vec::new();
		for (rh, rd) in hashes {
			let h = try!(rh);
			let d = try!(rd);
			if self.blocks.is_downloading(&h) {
				continue;
			}
			match io.chain().block_status(BlockID::Hash(h.clone())) {
				BlockStatus::InChain  => {
					trace!(target: "sync", "New block hash already in chain {:?}", h);
				},
				BlockStatus::Queued => {
					trace!(target: "sync", "New hash block already queued {:?}", h);
				},
				BlockStatus::Unknown => {
					new_hashes.push(h.clone());
					if d > max_height {
						trace!(target: "sync", "New unknown block hash {:?}", h);
						let peer = self.peers.get_mut(&peer_id).unwrap();
						peer.latest_hash = h.clone();
						peer.latest_number = Some(d);
						max_height = d;
					}
				},
				BlockStatus::Bad => {
					debug!(target: "sync", "Bad new block hash {:?}", h);
					io.disable_peer(peer_id);
					return Ok(());
				}
			}
		};
		if max_height != 0 {
			trace!(target: "sync", "Downloading blocks for new hashes");
			self.blocks.reset_to(new_hashes);
			self.state = SyncState::NewBlocks;
			self.sync_peer(io, peer_id, true);
		}
		Ok(())
	}

	/// Called by peer when it is disconnecting
	pub fn on_peer_aborting(&mut self, io: &mut SyncIo, peer: PeerId) {
		trace!(target: "sync", "== Disconnecting {}: {}", peer, io.peer_info(peer));
		if self.peers.contains_key(&peer) {
			debug!(target: "sync", "Disconnected {}", peer);
			self.clear_peer_download(peer);
			self.peers.remove(&peer);
			self.active_peers.remove(&peer);
			self.continue_sync(io);
		}
	}

	/// Called when a new peer is connected
	pub fn on_peer_connected(&mut self, io: &mut SyncIo, peer: PeerId) {
		trace!(target: "sync", "== Connected {}: {}", peer, io.peer_info(peer));
		if let Err(e) = self.send_status(io) {
			debug!(target:"sync", "Error sending status request: {:?}", e);
			io.disable_peer(peer);
		}
	}

	/// Resume downloading
	fn continue_sync(&mut self, io: &mut SyncIo) {
		let mut peers: Vec<(PeerId, U256)> = self.peers.iter().map(|(k, p)| (*k, p.difficulty.unwrap_or_else(U256::zero))).collect();
		peers.sort_by(|&(_, d1), &(_, d2)| d1.cmp(&d2).reverse()); //TODO: sort by rating
		for (p, _) in peers {
			if self.active_peers.contains(&p) {
				self.sync_peer(io, p, false);
			}
		}
		if !self.peers.values().any(|p| p.asking != PeerAsking::Nothing) {
			self.complete_sync();
		}
	}

	/// Called after all blocks have been downloaded
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
		let chain_info = io.chain().chain_info();
		let td = chain_info.pending_total_difficulty;
		let syncing_difficulty = max(self.syncing_difficulty, td);

		if force || self.state == SyncState::NewBlocks || peer_difficulty.map_or(true, |pd| pd > syncing_difficulty) {
			match self.state {
				SyncState::Idle => {
					if self.last_imported_block < chain_info.best_block_number {
						self.last_imported_block = chain_info.best_block_number;
						self.last_imported_hash = chain_info.best_block_hash;
					}
					trace!(target: "sync", "Starting sync with {}", peer_id);
					self.start_sync_round(io);
					self.sync_peer(io, peer_id, force);
				},
				SyncState::ChainHead => {
					// Request subchain headers
					trace!(target: "sync", "Starting sync with better chain");
					let last = self.last_imported_hash.clone();
					self.request_headers_by_hash(io, peer_id, &last, 128, 255, false, PeerAsking::Heads);
				},
				SyncState::Blocks | SyncState::NewBlocks => {
					if io.chain().block_status(BlockID::Hash(peer_latest)) == BlockStatus::Unknown {
						self.request_blocks(io, peer_id, false);
					}
				}
				SyncState::Waiting => ()
			}
		}
	}

	fn start_sync_round(&mut self, io: &mut SyncIo) {
		self.state = SyncState::ChainHead;
		trace!(target: "sync", "Starting round (last imported count = {:?}, block = {:?}", self.imported_this_round, self.last_imported_block);
		if self.imported_this_round.is_some() && self.imported_this_round.unwrap() == 0 && self.last_imported_block > 0 {
			match io.chain().block_hash(BlockID::Number(self.last_imported_block - 1)) {
				Some(h) => {
					self.last_imported_block -= 1;
					self.last_imported_hash = h;
					trace!(target: "sync", "Searching common header {} ({})", self.last_imported_block, self.last_imported_hash);
				}
				None => {
					// TODO: get hash by number from the block queue
					trace!(target: "sync", "Could not revert to previous block, last: {} ({})", self.last_imported_block, self.last_imported_hash);
				}
			}
		}
		self.imported_this_round = None;
	}

	/// Find some headers or blocks to download for a peer.
	fn request_blocks(&mut self, io: &mut SyncIo, peer_id: PeerId, ignore_others: bool) {
		self.clear_peer_download(peer_id);
		if io.chain().queue_info().is_full() {
			self.pause_sync();
			return;
		}

		// check to see if we need to download any block bodies first
		let needed_bodies = self.blocks.needed_bodies(MAX_BODIES_TO_REQUEST, ignore_others);
		if !needed_bodies.is_empty() {
			replace(&mut self.peers.get_mut(&peer_id).unwrap().asking_blocks, needed_bodies.clone());
			self.request_bodies(io, peer_id, needed_bodies);
			return;
		}

		// find subchain to download
		if let Some((h, count)) = self.blocks.needed_headers(MAX_HEADERS_TO_REQUEST, ignore_others) {
			replace(&mut self.peers.get_mut(&peer_id).unwrap().asking_blocks, vec![h.clone()]);
			self.request_headers_by_hash(io, peer_id, &h, count, 0, false, PeerAsking::BlockHeaders);
		}
	}

	/// Clear all blocks/headers marked as being downloaded by a peer.
	fn clear_peer_download(&mut self, peer_id: PeerId) {
		let peer = self.peers.get_mut(&peer_id).unwrap();
		match peer.asking {
			PeerAsking::BlockHeaders | PeerAsking::Heads => {
				for b in &peer.asking_blocks {
					self.blocks.clear_header_download(b);
				}
			},
			PeerAsking::BlockBodies => {
				for b in &peer.asking_blocks {
					self.blocks.clear_body_download(b);
				}
			},
			_ => (),
		}
		peer.asking_blocks.clear();
	}

	/// Checks if there are blocks fully downloaded that can be imported into the blockchain and does the import.
	fn collect_blocks(&mut self, io: &mut SyncIo) {
		let mut restart = false;
		let mut imported = HashSet::new();
		let blocks = self.blocks.drain();
		let count = blocks.len();
		for block in blocks {
			let number = BlockView::new(&block).header_view().number();
			let h = BlockView::new(&block).header_view().sha3();

			// Perform basic block verification
			if !Block::is_good(&block) {
				debug!(target: "sync", "Bad block rlp {:?} : {:?}", h, block);
				restart = true;
				break;
			}

			match io.chain().import_block(block) {
				Err(Error::Import(ImportError::AlreadyInChain)) => {
					trace!(target: "sync", "Block already in chain {:?}", h);
				},
				Err(Error::Import(ImportError::AlreadyQueued)) => {
					trace!(target: "sync", "Block already queued {:?}", h);
				},
				Ok(_) => {
					trace!(target: "sync", "Block queued {:?}", h);
					self.last_imported_block = number;
					self.last_imported_hash = h.clone();
					imported.insert(h.clone());
				},
				Err(Error::Block(BlockError::UnknownParent(_))) if self.state == SyncState::NewBlocks => {
					trace!(target: "sync", "Unknown new block parent, restarting sync");
					break;
				},
				Err(e) => {
					debug!(target: "sync", "Bad block {:?} : {:?}", h, e);
					restart = true;
					break;
				}
			}
		}
		trace!(target: "sync", "Imported {} of {}", imported.len(), count);
		self.imported_this_round = Some(self.imported_this_round.unwrap_or(0) + imported.len());

		if restart {
			self.restart_on_bad_block(io);
			return;
		}

		if self.blocks.is_empty() {
			// complete sync round
			trace!(target: "sync", "Sync round complete");
			self.restart(io);
		}
	}

	/// Request headers from a peer by block hash
	#[cfg_attr(feature="dev", allow(too_many_arguments))]
	fn request_headers_by_hash(&mut self, sync: &mut SyncIo, peer_id: PeerId, h: &H256, count: usize, skip: usize, reverse: bool, asking: PeerAsking) {
		trace!(target: "sync", "{} <- GetBlockHeaders: {} entries starting from {}", peer_id, count, h);
		let mut rlp = RlpStream::new_list(4);
		rlp.append(h);
		rlp.append(&count);
		rlp.append(&skip);
		rlp.append(&if reverse {1u32} else {0u32});
		self.send_request(sync, peer_id, asking, GET_BLOCK_HEADERS_PACKET, rlp.out());
	}

	/// Request block bodies from a peer
	fn request_bodies(&mut self, sync: &mut SyncIo, peer_id: PeerId, hashes: Vec<H256>) {
		let mut rlp = RlpStream::new_list(hashes.len());
		trace!(target: "sync", "{} <- GetBlockBodies: {} entries starting from {:?}", peer_id, hashes.len(), hashes.first());
		for h in hashes {
			rlp.append(&h);
		}
		self.send_request(sync, peer_id, PeerAsking::BlockBodies, GET_BLOCK_BODIES_PACKET, rlp.out());
	}

	/// Reset peer status after request is complete.
	fn reset_peer_asking(&mut self, peer_id: PeerId, asking: PeerAsking) -> bool {
		let peer = self.peers.get_mut(&peer_id).unwrap();
		if peer.asking != asking {
			trace!(target:"sync", "Asking {:?} while expected {:?}", peer.asking, asking);
			peer.asking = PeerAsking::Nothing;
			false
		}
		else {
			peer.asking = PeerAsking::Nothing;
			true
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
			match io.chain().block_header(BlockID::Hash(hash)) {
				Some(hdr) => From::from(HeaderView::new(&hdr).number()),
				None => return Ok(Some((BLOCK_HEADERS_PACKET, RlpStream::new_list(0)))) //no such header, return nothing
			}
		} else {
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
			if let Some(mut hdr) = io.chain().block_header(BlockID::Number(number)) {
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
			if let Some(mut hdr) = io.chain().block_body(BlockID::Hash(try!(r.val_at::<H256>(i)))) {
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
				let uncles = chain.find_uncles(from).unwrap_or_else(Vec::new);
				match route.blocks.len() {
					0 => None,
					_ => {
						let mut blocks = route.blocks;
						blocks.extend(uncles);
						let mut rlp_stream = RlpStream::new_list(blocks.len());
						for block_hash in  blocks {
							let mut hash_rlp = RlpStream::new_list(2);
							let difficulty = chain.block_total_difficulty(BlockID::Hash(block_hash.clone())).expect("Malformed block without a difficulty on the chain!");
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
		rlp_stream.append_raw(&chain.block(BlockID::Hash(chain.chain_info().best_block_hash)).unwrap(), 1);
		rlp_stream.append(&chain.chain_info().total_difficulty);
		rlp_stream.out()
	}

	/// returns peer ids that have less blocks than our chain
	fn get_lagging_peers(&mut self, chain_info: &BlockChainInfo, io: &SyncIo) -> Vec<(PeerId, BlockNumber)> {
		let latest_hash = chain_info.best_block_hash;
		let latest_number = chain_info.best_block_number;
		self.peers.iter_mut().filter_map(|(&id, ref mut peer_info)|
			match io.chain().block_status(BlockID::Hash(peer_info.latest_hash.clone())) {
				BlockStatus::InChain => {
					if peer_info.latest_number.is_none() {
						peer_info.latest_number = Some(HeaderView::new(&io.chain().block_header(BlockID::Hash(peer_info.latest_hash.clone())).unwrap()).number());
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
			sent += 1;
		}
		sent
	}

	/// propagates new known hashes to all peers
	fn propagate_new_hashes(&mut self, chain_info: &BlockChainInfo, io: &mut SyncIo) -> usize {
		let updated_peers = self.get_lagging_peers(chain_info, io);
		let mut sent = 0;
		let last_parent = HeaderView::new(&io.chain().block_header(BlockID::Hash(chain_info.best_block_hash.clone())).unwrap()).parent_hash();
		for (peer_id, peer_number) in updated_peers {
			let mut peer_best = self.peers.get(&peer_id).unwrap().latest_hash.clone();
			if chain_info.best_block_number - peer_number > MAX_PEERS_PROPAGATION as BlockNumber {
				// If we think peer is too far behind just send one latest hash
				peer_best = last_parent.clone();
			}
			sent += match ChainSync::create_new_hashes_rlp(io.chain(), &peer_best, &chain_info.best_block_hash) {
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

		let mut transactions = self.miner.all_transactions();
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
	use ethcore::spec::Spec;
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
		let mut sync = dummy_sync_with_peer(H256::new(), &client);
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

		io.sender = Some(2usize);
		sync.on_packet(&mut io, 0usize, super::GET_RECEIPTS_PACKET, &receipts_request);
		assert_eq!(1, io.queue.len());
	}

	#[test]
	fn return_block_headers() {
		use ethcore::views::HeaderView;
		fn make_hash_req(h: &H256, count: usize, skip: usize, reverse: bool) -> Bytes {
			let mut rlp = RlpStream::new_list(4);
			rlp.append(h);
			rlp.append(&count);
			rlp.append(&skip);
			rlp.append(&if reverse {1u32} else {0u32});
			rlp.out()
		}

		fn make_num_req(n: usize, count: usize, skip: usize, reverse: bool) -> Bytes {
			let mut rlp = RlpStream::new_list(4);
			rlp.append(&n);
			rlp.append(&count);
			rlp.append(&skip);
			rlp.append(&if reverse {1u32} else {0u32});
			rlp.out()
		}
		fn to_header_vec(rlp: ::chain::RlpResponseResult) -> Vec<Bytes> {
			Rlp::new(&rlp.unwrap().unwrap().1.out()).iter().map(|r| r.as_raw().to_vec()).collect()
		}

		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Nothing);
		let blocks: Vec<_> = (0 .. 100).map(|i| (&client as &BlockChainClient).block(BlockID::Number(i as BlockNumber)).unwrap()).collect();
		let headers: Vec<_> = blocks.iter().map(|b| Rlp::new(b).at(0).as_raw().to_vec()).collect();
		let hashes: Vec<_> = headers.iter().map(|h| HeaderView::new(h).sha3()).collect();

		let mut queue = VecDeque::new();
		let io = TestIo::new(&mut client, &mut queue, None);

		let unknown: H256 = H256::new();
		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&unknown, 1, 0, false)));
		assert!(to_header_vec(result).is_empty());
		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&unknown, 1, 0, true)));
		assert!(to_header_vec(result).is_empty());

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&hashes[2], 1, 0, true)));
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&hashes[2], 1, 0, false)));
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&hashes[50], 3, 5, false)));
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[56].clone(), headers[62].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&hashes[50], 3, 5, true)));
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[44].clone(), headers[38].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_num_req(2, 1, 0, true)));
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_num_req(2, 1, 0, false)));
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_num_req(50, 3, 5, false)));
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[56].clone(), headers[62].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_num_req(50, 3, 5, true)));
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[44].clone(), headers[38].clone()]);
	}

	#[test]
	fn return_nodes() {
		let mut client = TestBlockChainClient::new();
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(H256::new(), &client);
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

		io.sender = Some(2usize);
		sync.on_packet(&mut io, 0usize, super::GET_NODE_DATA_PACKET, &node_request);
		assert_eq!(1, io.queue.len());
	}

	fn dummy_sync_with_peer(peer_latest_hash: H256, client: &BlockChainClient) -> ChainSync {
		let mut sync = ChainSync::new(SyncConfig::default(), Miner::new(false, Spec::new_test()), client);
		sync.peers.insert(0,
			PeerInfo {
				protocol_version: 0,
				genesis: H256::zero(),
				network_id: U256::zero(),
				latest_hash: peer_latest_hash,
				latest_number: None,
				difficulty: None,
				asking: PeerAsking::Nothing,
				asking_blocks: Vec::new(),
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(10), &client);
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
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
	fn handles_peer_new_block_malformed() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);

		let block_data = get_dummy_block(11, client.chain_info().best_block_hash);

		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		//sync.have_common_block = true;
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let chain_info = client.chain_info();
		let mut io = TestIo::new(&mut client, &mut queue, None);

		sync.propagate_new_hashes(&chain_info, &mut io);

		let data = &io.queue[0].data.clone();
		let result = sync.on_peer_new_hashes(&mut io, 0, &UntrustedRlp::new(data));
		assert!(result.is_ok());
	}

	// idea is that what we produce when propagading latest block should be accepted in
	// on_peer_new_block  in our code as well
	#[test]
	fn block_rlp_mutually_acceptable() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut queue = VecDeque::new();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let chain_info = client.chain_info();
		let mut io = TestIo::new(&mut client, &mut queue, None);

		sync.propagate_blocks(&chain_info, &mut io);

		let data = &io.queue[0].data.clone();
		let result = sync.on_peer_new_block(&mut io, 0, &UntrustedRlp::new(data));
		assert!(result.is_ok());
	}

	#[test]
	fn should_add_transactions_to_queue() {
		// given
		let mut client = TestBlockChainClient::new();
		client.add_blocks(98, EachBlockWith::Uncle);
		client.add_blocks(1, EachBlockWith::UncleAndTransaction);
		client.add_blocks(1, EachBlockWith::Transaction);
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);

		let good_blocks = vec![client.block_hash_delta_minus(2)];
		let retracted_blocks = vec![client.block_hash_delta_minus(1)];

		// Add some balance to clients and reset nonces
		for h in &[good_blocks[0], retracted_blocks[0]] {
			let block = client.block(BlockID::Hash(*h)).unwrap();
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
			let block = client.block(BlockID::Hash(*h)).unwrap();
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
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);

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
}
