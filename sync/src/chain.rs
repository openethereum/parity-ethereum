// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
/// 	If peer's total difficulty is higher and there are less than 5 peers downloading, request N/M headers with interval M+1 starting from l
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
/// All other messages are ignored.
///
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
/// 	Validate received headers:
/// 		For each header find a parent in H or R or the blockchain. Restart if there is a block with unknown parent.
/// 		Find at least one header from the received list in S. Restart if there is none.
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

use std::collections::{HashSet, HashMap};
use std::cmp;
use heapsize::HeapSizeOf;
use util::*;
use rlp::*;
use network::*;
use ethcore::header::{BlockNumber, Header as BlockHeader};
use ethcore::client::{BlockChainClient, BlockStatus, BlockId, BlockChainInfo, BlockImportError, BlockQueueInfo};
use ethcore::error::*;
use ethcore::snapshot::{ManifestData, RestorationStatus};
use ethcore::transaction::PendingTransaction;
use sync_io::SyncIo;
use time;
use super::SyncConfig;
use block_sync::{BlockDownloader, BlockRequest, BlockDownloaderImportError as DownloaderImportError, DownloadAction};
use rand::Rng;
use snapshot::{Snapshot, ChunkType};
use api::{EthProtocolInfo as PeerInfoDigest, WARP_SYNC_PROTOCOL_ID};
use transactions_stats::{TransactionsStats, Stats as TransactionStats};

known_heap_size!(0, PeerInfo);

type PacketDecodeError = DecoderError;

const PROTOCOL_VERSION_63: u8 = 63;
const PROTOCOL_VERSION_62: u8 = 62;
const PROTOCOL_VERSION_1: u8 = 1;
const PROTOCOL_VERSION_2: u8 = 2;
const MAX_BODIES_TO_SEND: usize = 256;
const MAX_HEADERS_TO_SEND: usize = 512;
const MAX_NODE_DATA_TO_SEND: usize = 1024;
const MAX_RECEIPTS_TO_SEND: usize = 1024;
const MAX_RECEIPTS_HEADERS_TO_SEND: usize = 256;
const MIN_PEERS_PROPAGATION: usize = 4;
const MAX_PEERS_PROPAGATION: usize = 128;
const MAX_PEER_LAG_PROPAGATION: BlockNumber = 20;
const MAX_NEW_HASHES: usize = 64;
const MAX_TX_TO_IMPORT: usize = 512;
const MAX_NEW_BLOCK_AGE: BlockNumber = 20;
const MAX_TRANSACTION_SIZE: usize = 300*1024;
// maximal packet size with transactions (cannot be greater than 16MB - protocol limitation).
const MAX_TRANSACTION_PACKET_SIZE: usize = 8 * 1024 * 1024;
// Maximal number of transactions in sent in single packet.
const MAX_TRANSACTIONS_TO_PROPAGATE: usize = 64;
// Min number of blocks to be behind for a snapshot sync
const SNAPSHOT_RESTORE_THRESHOLD: BlockNumber = 10000;
const SNAPSHOT_MIN_PEERS: usize = 3;

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

pub const ETH_PACKET_COUNT: u8 = 0x11;

const GET_SNAPSHOT_MANIFEST_PACKET: u8 = 0x11;
const SNAPSHOT_MANIFEST_PACKET: u8 = 0x12;
const GET_SNAPSHOT_DATA_PACKET: u8 = 0x13;
const SNAPSHOT_DATA_PACKET: u8 = 0x14;
const CONSENSUS_DATA_PACKET: u8 = 0x15;

pub const SNAPSHOT_SYNC_PACKET_COUNT: u8 = 0x16;

const MAX_SNAPSHOT_CHUNKS_DOWNLOAD_AHEAD: usize = 3;

const WAIT_PEERS_TIMEOUT_SEC: u64 = 5;
const STATUS_TIMEOUT_SEC: u64 = 5;
const HEADERS_TIMEOUT_SEC: u64 = 15;
const BODIES_TIMEOUT_SEC: u64 = 10;
const RECEIPTS_TIMEOUT_SEC: u64 = 10;
const FORK_HEADER_TIMEOUT_SEC: u64 = 3;
const SNAPSHOT_MANIFEST_TIMEOUT_SEC: u64 = 5;
const SNAPSHOT_DATA_TIMEOUT_SEC: u64 = 120;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
/// Sync state
pub enum SyncState {
	/// Collecting enough peers to start syncing.
	WaitingPeers,
	/// Waiting for snapshot manifest download
	SnapshotManifest,
	/// Downloading snapshot data
	SnapshotData,
	/// Waiting for snapshot restoration progress.
	SnapshotWaiting,
	/// Downloading new blocks
	Blocks,
	/// Initial chain sync complete. Waiting for new packets
	Idle,
	/// Block downloading paused. Waiting for block queue to process blocks and free some space
	Waiting,
	/// Downloading blocks learned from `NewHashes` packet
	NewBlocks,
}

/// Syncing status and statistics
#[derive(Clone, Copy)]
pub struct SyncStatus {
	/// State
	pub state: SyncState,
	/// Syncing protocol version. That's the maximum protocol version we connect to.
	pub protocol_version: u8,
	/// The underlying p2p network version.
	pub network_id: u64,
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
	/// Total number of active peers.
	pub num_active_peers: usize,
	/// Heap memory used in bytes.
	pub mem_used: usize,
	/// Snapshot chunks
	pub num_snapshot_chunks: usize,
	/// Snapshot chunks downloaded
	pub snapshot_chunks_done: usize,
	/// Last fully downloaded and imported ancient block number (if any).
	pub last_imported_old_block_number: Option<BlockNumber>,
}

impl SyncStatus {
	/// Indicates if snapshot download is in progress
	pub fn is_snapshot_syncing(&self) -> bool {
		self.state == SyncState::SnapshotManifest
			|| self.state == SyncState::SnapshotData
			|| self.state == SyncState::SnapshotWaiting
	}

	/// Returns max no of peers to display in informants
	pub fn current_max_peers(&self, min_peers: u32, max_peers: u32) -> u32 {
		if self.num_peers as u32 > min_peers {
			max_peers
		} else {
			min_peers
		}
	}

	/// Is it doing a major sync?
	pub fn is_syncing(&self, queue_info: BlockQueueInfo) -> bool {
		let is_syncing_state = match self.state { SyncState::Idle | SyncState::NewBlocks => false, _ => true };
		let is_verifying = queue_info.unverified_queue_size + queue_info.verified_queue_size > 3;
		is_verifying || is_syncing_state
	}
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Peer data type requested
enum PeerAsking {
	Nothing,
	ForkHeader,
	BlockHeaders,
	BlockBodies,
	BlockReceipts,
	SnapshotManifest,
	SnapshotData,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
/// Block downloader channel.
enum BlockSet {
	/// New blocks better than out best blocks
	NewBlocks,
	/// Missing old blocks
	OldBlocks,
}
#[derive(Clone, Eq, PartialEq)]
enum ForkConfirmation {
	/// Fork block confirmation pending.
	Unconfirmed,
	/// Peers chain is too short to confirm the fork.
	TooShort,
	/// Fork is confirmed.
	Confirmed,
}

#[derive(Clone)]
/// Syncing peer information
struct PeerInfo {
	/// eth protocol version
	protocol_version: u8,
	/// Peer chain genesis hash
	genesis: H256,
	/// Peer network id
	network_id: u64,
	/// Peer best block hash
	latest_hash: H256,
	/// Peer total difficulty if known
	difficulty: Option<U256>,
	/// Type of data currenty being requested from peer.
	asking: PeerAsking,
	/// A set of block numbers being requested
	asking_blocks: Vec<H256>,
	/// Holds requested header hash if currently requesting block header by hash
	asking_hash: Option<H256>,
	/// Holds requested snapshot chunk hash if any.
	asking_snapshot_data: Option<H256>,
	/// Request timestamp
	ask_time: u64,
	/// Holds a set of transactions recently sent to this peer to avoid spamming.
	last_sent_transactions: HashSet<H256>,
	/// Pending request is expired and result should be ignored
	expired: bool,
	/// Peer fork confirmation status
	confirmation: ForkConfirmation,
	/// Best snapshot hash
	snapshot_hash: Option<H256>,
	/// Best snapshot block number
	snapshot_number: Option<BlockNumber>,
	/// Block set requested
	block_set: Option<BlockSet>,
}

impl PeerInfo {
	fn can_sync(&self) -> bool {
		self.confirmation == ForkConfirmation::Confirmed && !self.expired
	}

	fn is_allowed(&self) -> bool {
		self.confirmation != ForkConfirmation::Unconfirmed && !self.expired
	}

	fn reset_asking(&mut self) {
		self.asking_blocks.clear();
		self.asking_hash = None;
		// mark any pending requests as expired
		if self.asking != PeerAsking::Nothing && self.is_allowed() {
			self.expired = true;
		}
	}
}

#[cfg(not(test))]
mod random {
	use rand;
	pub fn new() -> rand::ThreadRng { rand::thread_rng() }
}
#[cfg(test)]
mod random {
	use rand::{self, SeedableRng};
	pub fn new() -> rand::XorShiftRng { rand::XorShiftRng::from_seed([0, 1, 2, 3]) }
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
	/// Block download process for new blocks
	new_blocks: BlockDownloader,
	/// Block download process for ancient blocks
	old_blocks: Option<BlockDownloader>,
	/// Last propagated block number
	last_sent_block_number: BlockNumber,
	/// Network ID
	network_id: u64,
	/// Optional fork block to check
	fork_block: Option<(BlockNumber, H256)>,
	/// Snapshot downloader.
	snapshot: Snapshot,
	/// Connected peers pending Status message.
	/// Value is request timestamp.
	handshaking_peers: HashMap<PeerId, u64>,
	/// Sync start timestamp. Measured when first peer is connected
	sync_start_time: Option<u64>,
	/// Transactions propagation statistics
	transactions_stats: TransactionsStats,
	/// Enable ancient block downloading
	download_old_blocks: bool,
	/// Enable warp sync.
	enable_warp_sync: bool,
}

type RlpResponseResult = Result<Option<(PacketId, RlpStream)>, PacketDecodeError>;

impl ChainSync {
	/// Create a new instance of syncing strategy.
	pub fn new(config: SyncConfig, chain: &BlockChainClient) -> ChainSync {
		let chain_info = chain.chain_info();
		let mut sync = ChainSync {
			state: if config.warp_sync { SyncState::WaitingPeers } else { SyncState::Idle },
			starting_block: chain.chain_info().best_block_number,
			highest_block: None,
			peers: HashMap::new(),
			handshaking_peers: HashMap::new(),
			active_peers: HashSet::new(),
			new_blocks: BlockDownloader::new(false, &chain_info.best_block_hash, chain_info.best_block_number),
			old_blocks: None,
			last_sent_block_number: 0,
			network_id: config.network_id,
			fork_block: config.fork_block,
			download_old_blocks: config.download_old_blocks,
			snapshot: Snapshot::new(),
			sync_start_time: None,
			transactions_stats: TransactionsStats::default(),
			enable_warp_sync: config.warp_sync,
		};
		sync.update_targets(chain);
		sync
	}

	/// Returns synchonization status
	pub fn status(&self) -> SyncStatus {
		let last_imported_number = self.new_blocks.last_imported_block_number();
		SyncStatus {
			state: self.state.clone(),
			protocol_version: PROTOCOL_VERSION_63,
			network_id: self.network_id,
			start_block_number: self.starting_block,
			last_imported_block_number: Some(last_imported_number),
			last_imported_old_block_number: self.old_blocks.as_ref().map(|d| d.last_imported_block_number()),
			highest_block_number: self.highest_block.map(|n| cmp::max(n, last_imported_number)),
			blocks_received: if last_imported_number > self.starting_block { last_imported_number - self.starting_block } else { 0 },
			blocks_total: match self.highest_block { Some(x) if x > self.starting_block => x - self.starting_block, _ => 0 },
			num_peers: self.peers.values().filter(|p| p.is_allowed()).count(),
			num_active_peers: self.peers.values().filter(|p| p.is_allowed() && p.asking != PeerAsking::Nothing).count(),
			num_snapshot_chunks: self.snapshot.total_chunks(),
			snapshot_chunks_done: self.snapshot.done_chunks(),
			mem_used:
				self.new_blocks.heap_size()
				+ self.old_blocks.as_ref().map_or(0, |d| d.heap_size())
				+ self.peers.heap_size_of_children(),
		}
	}

	/// Returns information on peers connections
	pub fn peer_info(&self, peer_id: &PeerId) -> Option<PeerInfoDigest> {
		self.peers.get(peer_id).map(|peer_data| {
			PeerInfoDigest {
				version: peer_data.protocol_version as u32,
				difficulty: peer_data.difficulty,
				head: peer_data.latest_hash,
			}
		})
	}

	/// Returns transactions propagation statistics
	pub fn transactions_stats(&self) -> &H256FastMap<TransactionStats> {
		self.transactions_stats.stats()
	}

	/// Updates transactions were received by a peer
	pub fn transactions_received(&mut self, hashes: Vec<H256>, peer_id: PeerId) {
		if let Some(mut peer_info) = self.peers.get_mut(&peer_id) {
			peer_info.last_sent_transactions.extend(&hashes);
		}
	}

	/// Abort all sync activity
	pub fn abort(&mut self, io: &mut SyncIo) {
		self.reset_and_continue(io);
		self.peers.clear();
	}

	#[cfg_attr(feature="dev", allow(for_kv_map))] // Because it's not possible to get `values_mut()`
	/// Reset sync. Clear all downloaded data but keep the queue
	fn reset(&mut self, io: &mut SyncIo) {
		self.new_blocks.reset();
		let chain_info = io.chain().chain_info();
		for (_, ref mut p) in &mut self.peers {
			if p.block_set != Some(BlockSet::OldBlocks) {
				p.reset_asking();
				if p.difficulty.is_none() {
					// assume peer has up to date difficulty
					p.difficulty = Some(chain_info.pending_total_difficulty);
				}
			}
		}
		self.state = SyncState::Idle;
		// Reactivate peers only if some progress has been made
		// since the last sync round of if starting fresh.
		self.active_peers = self.peers.keys().cloned().collect();
	}

	/// Restart sync
	pub fn reset_and_continue(&mut self, io: &mut SyncIo) {
		trace!(target: "sync", "Restarting");
		if self.state == SyncState::SnapshotData {
			debug!(target:"sync", "Aborting snapshot restore");
			io.snapshot_service().abort_restore();
		}
		self.snapshot.clear();
		self.reset(io);
		self.continue_sync(io);
	}

	/// Remove peer from active peer set. Peer will be reactivated on the next sync
	/// round.
	fn deactivate_peer(&mut self, _io: &mut SyncIo, peer_id: PeerId) {
		trace!(target: "sync", "Deactivating peer {}", peer_id);
		self.active_peers.remove(&peer_id);
	}

	fn maybe_start_snapshot_sync(&mut self, io: &mut SyncIo) {
		if !self.enable_warp_sync || io.snapshot_service().supported_versions().is_none() {
			return;
		}
		if self.state != SyncState::WaitingPeers && self.state != SyncState::Blocks && self.state != SyncState::Waiting {
			return;
		}
		// Make sure the snapshot block is not too far away from best block and network best block and
		// that it is higher than fork detection block
		let our_best_block = io.chain().chain_info().best_block_number;
		let fork_block = self.fork_block.as_ref().map(|&(n, _)| n).unwrap_or(0);

		let (best_hash, max_peers, snapshot_peers) = {
			//collect snapshot infos from peers
			let snapshots = self.peers.iter()
				.filter(|&(_, p)| p.is_allowed() && p.snapshot_number.map_or(false, |sn|
					our_best_block < sn && (sn - our_best_block) > SNAPSHOT_RESTORE_THRESHOLD &&
					sn > fork_block &&
					self.highest_block.map_or(true, |highest| highest >= sn && (highest - sn) <= SNAPSHOT_RESTORE_THRESHOLD)
				))
				.filter_map(|(p, peer)| peer.snapshot_hash.map(|hash| (p, hash.clone())))
				.filter(|&(_, ref hash)| !self.snapshot.is_known_bad(hash));

			let mut snapshot_peers = HashMap::new();
			let mut max_peers: usize = 0;
			let mut best_hash = None;
			for (p, hash) in snapshots {
				let peers = snapshot_peers.entry(hash).or_insert_with(Vec::new);
				peers.push(*p);
				if peers.len() > max_peers {
					max_peers = peers.len();
					best_hash = Some(hash);
				}
			}
			(best_hash, max_peers, snapshot_peers)
		};

		let timeout = (self.state == SyncState::WaitingPeers) && self.sync_start_time.map_or(false, |t| ((time::precise_time_ns() - t) / 1_000_000_000) > WAIT_PEERS_TIMEOUT_SEC);

		if let (Some(hash), Some(peers)) = (best_hash, best_hash.map_or(None, |h| snapshot_peers.get(&h))) {
			if max_peers >= SNAPSHOT_MIN_PEERS {
				trace!(target: "sync", "Starting confirmed snapshot sync {:?} with {:?}", hash, peers);
				self.start_snapshot_sync(io, peers);
			} else if timeout {
				trace!(target: "sync", "Starting unconfirmed snapshot sync {:?} with {:?}", hash, peers);
				self.start_snapshot_sync(io, peers);
			}
		} else if timeout {
			trace!(target: "sync", "No snapshots found, starting full sync");
			self.state = SyncState::Idle;
			self.continue_sync(io);
		}
	}

	fn start_snapshot_sync(&mut self, io: &mut SyncIo, peers: &[PeerId]) {
		if !self.snapshot.have_manifest() {
			for p in peers {
				if self.peers.get(p).map_or(false, |p| p.asking == PeerAsking::Nothing) {
					self.request_snapshot_manifest(io, *p);
				}
			}
			self.state = SyncState::SnapshotManifest;
			trace!(target: "sync", "New snapshot sync with {:?}", peers);
		} else {
			self.state = SyncState::SnapshotData;
			trace!(target: "sync", "Resumed snapshot sync with {:?}", peers);
		}
	}

	/// Restart sync disregarding the block queue status. May end up re-downloading up to QUEUE_SIZE blocks
	pub fn restart(&mut self, io: &mut SyncIo) {
		self.update_targets(io.chain());
		self.reset_and_continue(io);
	}

	/// Update sync after the blockchain has been changed externally.
	pub fn update_targets(&mut self, chain: &BlockChainClient) {
		// Do not assume that the block queue/chain still has our last_imported_block
		let chain = chain.chain_info();
		self.new_blocks = BlockDownloader::new(false, &chain.best_block_hash, chain.best_block_number);
		self.old_blocks = None;
		if self.download_old_blocks {
			if let (Some(ancient_block_hash), Some(ancient_block_number)) = (chain.ancient_block_hash, chain.ancient_block_number) {

				trace!(target: "sync", "Downloading old blocks from {:?} (#{}) till {:?} (#{:?})", ancient_block_hash, ancient_block_number, chain.first_block_hash, chain.first_block_number);
				let mut downloader = BlockDownloader::with_unlimited_reorg(true, &ancient_block_hash, ancient_block_number);
				if let Some(hash) = chain.first_block_hash {
					trace!(target: "sync", "Downloader target set to {:?}", hash);
					downloader.set_target(&hash);
				}
				self.old_blocks = Some(downloader);
			}
		}
	}

	/// Called by peer to report status
	fn on_peer_status(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		self.handshaking_peers.remove(&peer_id);
		let protocol_version: u8 = r.val_at(0)?;
		let warp_protocol = io.protocol_version(&WARP_SYNC_PROTOCOL_ID, peer_id) != 0;
		let peer = PeerInfo {
			protocol_version: protocol_version,
			network_id: r.val_at(1)?,
			difficulty: Some(r.val_at(2)?),
			latest_hash: r.val_at(3)?,
			genesis: r.val_at(4)?,
			asking: PeerAsking::Nothing,
			asking_blocks: Vec::new(),
			asking_hash: None,
			ask_time: 0,
			last_sent_transactions: HashSet::new(),
			expired: false,
			confirmation: if self.fork_block.is_none() { ForkConfirmation::Confirmed } else { ForkConfirmation::Unconfirmed },
			asking_snapshot_data: None,
			snapshot_hash: if warp_protocol { Some(r.val_at(5)?) } else { None },
			snapshot_number: if warp_protocol { Some(r.val_at(6)?) } else { None },
			block_set: None,
		};

		if self.sync_start_time.is_none() {
			self.sync_start_time = Some(time::precise_time_ns());
		}

		trace!(target: "sync", "New peer {} (protocol: {}, network: {:?}, difficulty: {:?}, latest:{}, genesis:{}, snapshot:{:?})",
			peer_id, peer.protocol_version, peer.network_id, peer.difficulty, peer.latest_hash, peer.genesis, peer.snapshot_number);
		if io.is_expired() {
			trace!(target: "sync", "Status packet from expired session {}:{}", peer_id, io.peer_info(peer_id));
			return Ok(());
		}

		if self.peers.contains_key(&peer_id) {
			debug!(target: "sync", "Unexpected status packet from {}:{}", peer_id, io.peer_info(peer_id));
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
		if (warp_protocol && peer.protocol_version != PROTOCOL_VERSION_1 && peer.protocol_version != PROTOCOL_VERSION_2) || (!warp_protocol && peer.protocol_version != PROTOCOL_VERSION_63 && peer.protocol_version != PROTOCOL_VERSION_62) {
			io.disable_peer(peer_id);
			trace!(target: "sync", "Peer {} unsupported eth protocol ({})", peer_id, peer.protocol_version);
			return Ok(());
		}

		self.peers.insert(peer_id.clone(), peer);
		// Don't activate peer immediatelly when searching for common block.
		// Let the current sync round complete first.
		self.active_peers.insert(peer_id.clone());
		debug!(target: "sync", "Connected {}:{}", peer_id, io.peer_info(peer_id));
		if let Some((fork_block, _)) = self.fork_block {
			self.request_fork_header_by_number(io, peer_id, fork_block);
		} else {
			self.sync_peer(io, peer_id, false);
		}
		Ok(())
	}

	#[cfg_attr(feature="dev", allow(cyclomatic_complexity, needless_borrow))]
	/// Called by peer once it has new block headers during sync
	fn on_peer_block_headers(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		let confirmed = match self.peers.get_mut(&peer_id) {
			Some(ref mut peer) if peer.asking == PeerAsking::ForkHeader => {
				peer.asking = PeerAsking::Nothing;
				let item_count = r.item_count()?;
				let (fork_number, fork_hash) = self.fork_block.expect("ForkHeader request is sent only fork block is Some; qed").clone();
				if item_count == 0 || item_count != 1 {
					trace!(target: "sync", "{}: Chain is too short to confirm the block", peer_id);
					peer.confirmation = ForkConfirmation::TooShort;
				} else {
					let header = r.at(0)?.as_raw();
					if header.sha3() == fork_hash {
						trace!(target: "sync", "{}: Confirmed peer", peer_id);
						peer.confirmation = ForkConfirmation::Confirmed;
						if !io.chain_overlay().read().contains_key(&fork_number) {
							io.chain_overlay().write().insert(fork_number, header.to_vec());
						}
					} else {
						trace!(target: "sync", "{}: Fork mismatch", peer_id);
						io.disconnect_peer(peer_id);
						return Ok(());
					}
				}
				true
			},
			_ => false,
		};
		if confirmed {
			self.sync_peer(io, peer_id, false);
			return Ok(());
		}

		self.clear_peer_download(peer_id);
		let expected_hash = self.peers.get(&peer_id).and_then(|p| p.asking_hash);
		let allowed = self.peers.get(&peer_id).map(|p| p.is_allowed()).unwrap_or(false);
		let block_set = self.peers.get(&peer_id).and_then(|p| p.block_set).unwrap_or(BlockSet::NewBlocks);
		if !self.reset_peer_asking(peer_id, PeerAsking::BlockHeaders) || expected_hash.is_none() || !allowed {
			trace!(target: "sync", "{}: Ignored unexpected headers, expected_hash = {:?}", peer_id, expected_hash);
			self.continue_sync(io);
			return Ok(());
		}
		let item_count = r.item_count()?;
		trace!(target: "sync", "{} -> BlockHeaders ({} entries), state = {:?}, set = {:?}", peer_id, item_count, self.state, block_set);
		if (self.state == SyncState::Idle || self.state == SyncState::WaitingPeers) && self.old_blocks.is_none() {
			trace!(target: "sync", "Ignored unexpected block headers");
			self.continue_sync(io);
			return Ok(());
		}
		if self.state == SyncState::Waiting {
			trace!(target: "sync", "Ignored block headers while waiting");
			self.continue_sync(io);
			return Ok(());
		}

		let result =  {
			let mut downloader = match block_set {
				BlockSet::NewBlocks => &mut self.new_blocks,
				BlockSet::OldBlocks => {
					match self.old_blocks {
						None => {
							trace!(target: "sync", "Ignored block headers while block download is inactive");
							self.continue_sync(io);
							return Ok(());
						},
						Some(ref mut blocks) => blocks,
					}
				}
			};
			downloader.import_headers(io, r, expected_hash)
		};

		match result {
			Err(DownloaderImportError::Useless) => {
				self.deactivate_peer(io, peer_id);
			},
			Err(DownloaderImportError::Invalid) => {
				io.disable_peer(peer_id);
				self.deactivate_peer(io, peer_id);
				self.continue_sync(io);
				return Ok(());
			},
			Ok(DownloadAction::Reset) => {
				// mark all outstanding requests as expired
				trace!("Resetting downloads for {:?}", block_set);
				for (_, ref mut p) in self.peers.iter_mut().filter(|&(_, ref p)| p.block_set == Some(block_set)) {
					p.reset_asking();
				}

			}
			Ok(DownloadAction::None) => {},
		}

		self.collect_blocks(io, block_set);
		// give a task to the same peer first if received valuable headers.
		self.sync_peer(io, peer_id, false);
		// give tasks to other peers
		self.continue_sync(io);
		Ok(())
	}

	/// Called by peer once it has new block bodies
	fn on_peer_block_bodies(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		self.clear_peer_download(peer_id);
		let block_set = self.peers.get(&peer_id).and_then(|p| p.block_set).unwrap_or(BlockSet::NewBlocks);
		if !self.reset_peer_asking(peer_id, PeerAsking::BlockBodies) {
			trace!(target: "sync", "{}: Ignored unexpected bodies", peer_id);
			self.continue_sync(io);
			return Ok(());
		}
		let item_count = r.item_count()?;
		trace!(target: "sync", "{} -> BlockBodies ({} entries), set = {:?}", peer_id, item_count, block_set);
		if item_count == 0 {
			self.deactivate_peer(io, peer_id);
		}
		else if self.state == SyncState::Waiting {
			trace!(target: "sync", "Ignored block bodies while waiting");
		}
		else
		{
			let result = {
				let mut downloader = match block_set {
					BlockSet::NewBlocks => &mut self.new_blocks,
					BlockSet::OldBlocks => match self.old_blocks {
						None => {
							trace!(target: "sync", "Ignored block headers while block download is inactive");
							self.continue_sync(io);
							return Ok(());
						},
						Some(ref mut blocks) => blocks,
					}
				};
				downloader.import_bodies(io, r)
			};

			match result {
				Err(DownloaderImportError::Invalid) => {
					io.disable_peer(peer_id);
					self.deactivate_peer(io, peer_id);
					self.continue_sync(io);
					return Ok(());
				},
				Err(DownloaderImportError::Useless) => {
					self.deactivate_peer(io, peer_id);
				},
				Ok(()) => (),
			}

			self.collect_blocks(io, block_set);
			self.sync_peer(io, peer_id, false);
		}
		self.continue_sync(io);
		Ok(())
	}

	/// Called by peer once it has new block receipts
	fn on_peer_block_receipts(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		self.clear_peer_download(peer_id);
		let block_set = self.peers.get(&peer_id).and_then(|p| p.block_set).unwrap_or(BlockSet::NewBlocks);
		if !self.reset_peer_asking(peer_id, PeerAsking::BlockReceipts) {
			trace!(target: "sync", "{}: Ignored unexpected receipts", peer_id);
			self.continue_sync(io);
			return Ok(());
		}
		let item_count = r.item_count()?;
		trace!(target: "sync", "{} -> BlockReceipts ({} entries)", peer_id, item_count);
		if item_count == 0 {
			self.deactivate_peer(io, peer_id);
		}
		else if self.state == SyncState::Waiting {
			trace!(target: "sync", "Ignored block receipts while waiting");
		}
		else
		{
			let result = {
				let mut downloader = match block_set {
					BlockSet::NewBlocks => &mut self.new_blocks,
					BlockSet::OldBlocks => match self.old_blocks {
						None => {
							trace!(target: "sync", "Ignored block headers while block download is inactive");
							self.continue_sync(io);
							return Ok(());
						},
						Some(ref mut blocks) => blocks,
					}
				};
				downloader.import_receipts(io, r)
			};

			match result {
				Err(DownloaderImportError::Invalid) => {
					io.disable_peer(peer_id);
					self.deactivate_peer(io, peer_id);
					self.continue_sync(io);
					return Ok(());
				},
				Err(DownloaderImportError::Useless) => {
					self.deactivate_peer(io, peer_id);
				},
				Ok(()) => (),
			}

			self.collect_blocks(io, block_set);
			self.sync_peer(io, peer_id, false);
		}
		self.continue_sync(io);
		Ok(())
	}

	/// Called by peer once it has new block bodies
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn on_peer_new_block(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		if !self.peers.get(&peer_id).map_or(false, |p| p.can_sync()) {
			trace!(target: "sync", "Ignoring new block from unconfirmed peer {}", peer_id);
			return Ok(());
		}
		let difficulty: U256 = r.val_at(1)?;
		if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
			if peer.difficulty.map_or(true, |pd| difficulty > pd) {
				peer.difficulty = Some(difficulty);
			}
		}
		let block_rlp = r.at(0)?;
		let header_rlp = block_rlp.at(0)?;
		let h = header_rlp.as_raw().sha3();
		trace!(target: "sync", "{} -> NewBlock ({})", peer_id, h);
		let header: BlockHeader = header_rlp.as_val()?;
		if header.number() > self.highest_block.unwrap_or(0) {
			self.highest_block = Some(header.number());
		}
		let mut unknown = false;
		{
			if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
				peer.latest_hash = header.hash();
			}
		}
		let last_imported_number = self.new_blocks.last_imported_block_number();
		if last_imported_number > header.number() && last_imported_number - header.number() > MAX_NEW_BLOCK_AGE {
			trace!(target: "sync", "Ignored ancient new block {:?}", h);
			io.disable_peer(peer_id);
			return Ok(());
		}
		match io.chain().import_block(block_rlp.as_raw().to_vec()) {
			Err(BlockImportError::Import(ImportError::AlreadyInChain)) => {
				trace!(target: "sync", "New block already in chain {:?}", h);
			},
			Err(BlockImportError::Import(ImportError::AlreadyQueued)) => {
				trace!(target: "sync", "New block already queued {:?}", h);
			},
			Ok(_) => {
				// abort current download of the same block
				self.complete_sync(io);
				self.new_blocks.mark_as_known(&header.hash(), header.number());
				trace!(target: "sync", "New block queued {:?} ({})", h, header.number());
			},
			Err(BlockImportError::Block(BlockError::UnknownParent(p))) => {
				unknown = true;
				trace!(target: "sync", "New block with unknown parent ({:?}) {:?}", p, h);
			},
			Err(e) => {
				debug!(target: "sync", "Bad new block {:?} : {:?}", h, e);
				io.disable_peer(peer_id);
			}
		};
		if unknown {
			if self.state != SyncState::Idle {
				trace!(target: "sync", "NewBlock ignored while seeking");
			} else {
				trace!(target: "sync", "New unknown block {:?}", h);
				//TODO: handle too many unknown blocks
				self.sync_peer(io, peer_id, true);
			}
		}
		self.continue_sync(io);
		Ok(())
	}

	/// Handles `NewHashes` packet. Initiates headers download for any unknown hashes.
	fn on_peer_new_hashes(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		if !self.peers.get(&peer_id).map_or(false, |p| p.can_sync()) {
			trace!(target: "sync", "Ignoring new hashes from unconfirmed peer {}", peer_id);
			return Ok(());
		}
		let hashes: Vec<_> = r.iter().take(MAX_NEW_HASHES).map(|item| (item.val_at::<H256>(0), item.val_at::<BlockNumber>(1))).collect();
		if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
			// Peer has new blocks with unknown difficulty
			peer.difficulty = None;
			if let Some(&(Ok(ref h), _)) = hashes.last() {
				peer.latest_hash = h.clone();
			}
		}
		if self.state != SyncState::Idle {
			trace!(target: "sync", "Ignoring new hashes since we're already downloading.");
			let max = r.iter().take(MAX_NEW_HASHES).map(|item| item.val_at::<BlockNumber>(1).unwrap_or(0)).fold(0u64, cmp::max);
			if max > self.highest_block.unwrap_or(0) {
				self.highest_block = Some(max);
			}
			self.continue_sync(io);
			return Ok(());
		}
		trace!(target: "sync", "{} -> NewHashes ({} entries)", peer_id, r.item_count()?);
		let mut max_height: BlockNumber = 0;
		let mut new_hashes = Vec::new();
		let last_imported_number = self.new_blocks.last_imported_block_number();
		for (rh, rn) in hashes {
			let hash = rh?;
			let number = rn?;
			if number > self.highest_block.unwrap_or(0) {
				self.highest_block = Some(number);
			}
			if self.new_blocks.is_downloading(&hash) {
				continue;
			}
			if last_imported_number > number && last_imported_number - number > MAX_NEW_BLOCK_AGE {
				trace!(target: "sync", "Ignored ancient new block hash {:?}", hash);
				io.disable_peer(peer_id);
				continue;
			}
			match io.chain().block_status(BlockId::Hash(hash.clone())) {
				BlockStatus::InChain  => {
					trace!(target: "sync", "New block hash already in chain {:?}", hash);
				},
				BlockStatus::Queued => {
					trace!(target: "sync", "New hash block already queued {:?}", hash);
				},
				BlockStatus::Unknown | BlockStatus::Pending => {
					new_hashes.push(hash.clone());
					if number > max_height {
						trace!(target: "sync", "New unknown block hash {:?}", hash);
						if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
							peer.latest_hash = hash.clone();
						}
						max_height = number;
					}
				},
				BlockStatus::Bad => {
					debug!(target: "sync", "Bad new block hash {:?}", hash);
					io.disable_peer(peer_id);
					return Ok(());
				}
			}
		};
		if max_height != 0 {
			trace!(target: "sync", "Downloading blocks for new hashes");
			self.new_blocks.reset_to(new_hashes);
			self.state = SyncState::NewBlocks;
			self.sync_peer(io, peer_id, true);
		}
		self.continue_sync(io);
		Ok(())
	}

	/// Called when snapshot manifest is downloaded from a peer.
	fn on_snapshot_manifest(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		if !self.peers.get(&peer_id).map_or(false, |p| p.can_sync()) {
			trace!(target: "sync", "Ignoring snapshot manifest from unconfirmed peer {}", peer_id);
			return Ok(());
		}
		self.clear_peer_download(peer_id);
		if !self.reset_peer_asking(peer_id, PeerAsking::SnapshotManifest) || self.state != SyncState::SnapshotManifest {
			trace!(target: "sync", "{}: Ignored unexpected/expired manifest", peer_id);
			self.continue_sync(io);
			return Ok(());
		}

		let manifest_rlp = r.at(0)?;
		let manifest = match ManifestData::from_rlp(manifest_rlp.as_raw()) {
			Err(e) => {
				trace!(target: "sync", "{}: Ignored bad manifest: {:?}", peer_id, e);
				io.disable_peer(peer_id);
				self.continue_sync(io);
				return Ok(());
			}
			Ok(manifest) => manifest,
		};

		let is_supported_version = io.snapshot_service().supported_versions()
			.map_or(false, |(l, h)| manifest.version >= l && manifest.version <= h);

		if !is_supported_version {
			trace!(target: "sync", "{}: Snapshot manifest version not supported: {}", peer_id, manifest.version);
			io.disable_peer(peer_id);
			self.continue_sync(io);
			return Ok(());
		}
		self.snapshot.reset_to(&manifest, &manifest_rlp.as_raw().sha3());
		io.snapshot_service().begin_restore(manifest);
		self.state = SyncState::SnapshotData;

		// give a task to the same peer first.
		self.sync_peer(io, peer_id, false);
		// give tasks to other peers
		self.continue_sync(io);
		Ok(())
	}

	/// Called when snapshot data is downloaded from a peer.
	fn on_snapshot_data(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		if !self.peers.get(&peer_id).map_or(false, |p| p.can_sync()) {
			trace!(target: "sync", "Ignoring snapshot data from unconfirmed peer {}", peer_id);
			return Ok(());
		}
		self.clear_peer_download(peer_id);
		if !self.reset_peer_asking(peer_id, PeerAsking::SnapshotData) || (self.state != SyncState::SnapshotData && self.state != SyncState::SnapshotWaiting) {
			trace!(target: "sync", "{}: Ignored unexpected snapshot data", peer_id);
			self.continue_sync(io);
			return Ok(());
		}

		// check service status
		let status = io.snapshot_service().status();
		match status {
			RestorationStatus::Inactive | RestorationStatus::Failed => {
				trace!(target: "sync", "{}: Snapshot restoration aborted", peer_id);
				self.state = SyncState::WaitingPeers;

				// only note bad if restoration failed.
				if let (Some(hash), RestorationStatus::Failed) = (self.snapshot.snapshot_hash(), status) {
					trace!(target: "sync", "Noting snapshot hash {} as bad", hash);
					self.snapshot.note_bad(hash);
				}

				self.snapshot.clear();
				self.continue_sync(io);
				return Ok(());
			},
			RestorationStatus::Ongoing { .. } => {
				trace!(target: "sync", "{}: Snapshot restoration is ongoing", peer_id);
			},
		}

		let snapshot_data: Bytes = r.val_at(0)?;
		match self.snapshot.validate_chunk(&snapshot_data) {
			Ok(ChunkType::Block(hash)) => {
				trace!(target: "sync", "{}: Processing block chunk", peer_id);
				io.snapshot_service().restore_block_chunk(hash, snapshot_data);
			}
			Ok(ChunkType::State(hash)) => {
				trace!(target: "sync", "{}: Processing state chunk", peer_id);
				io.snapshot_service().restore_state_chunk(hash, snapshot_data);
			}
			Err(()) => {
				trace!(target: "sync", "{}: Got bad snapshot chunk", peer_id);
				io.disconnect_peer(peer_id);
				self.continue_sync(io);
				return Ok(());
			}
		}

		if self.snapshot.is_complete() {
			// wait for snapshot restoration process to complete
			self.state = SyncState::SnapshotWaiting;
		}
		// give a task to the same peer first.
		self.sync_peer(io, peer_id, false);
		// give tasks to other peers
		self.continue_sync(io);
		Ok(())
	}

	/// Called by peer when it is disconnecting
	pub fn on_peer_aborting(&mut self, io: &mut SyncIo, peer: PeerId) {
		trace!(target: "sync", "== Disconnecting {}: {}", peer, io.peer_info(peer));
		self.handshaking_peers.remove(&peer);
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
		if let Err(e) = self.send_status(io, peer) {
			debug!(target:"sync", "Error sending status request: {:?}", e);
			io.disable_peer(peer);
		} else {
			self.handshaking_peers.insert(peer, time::precise_time_ns());
		}
	}

	/// Resume downloading
	fn continue_sync(&mut self, io: &mut SyncIo) {
		let mut peers: Vec<(PeerId, U256, u8)> = self.peers.iter().filter_map(|(k, p)|
			if p.can_sync() { Some((*k, p.difficulty.unwrap_or_else(U256::zero), p.protocol_version)) } else { None }).collect();
		random::new().shuffle(&mut peers); //TODO: sort by rating
		// prefer peers with higher protocol version
		peers.sort_by(|&(_, _, ref v1), &(_, _, ref v2)| v1.cmp(v2));
		trace!(target: "sync", "Syncing with peers: {} active, {} confirmed, {} total", self.active_peers.len(), peers.len(), self.peers.len());
		for (p, _, _) in peers {
			if self.active_peers.contains(&p) {
				self.sync_peer(io, p, false);
			}
		}
		if (self.state != SyncState::WaitingPeers && self.state != SyncState::SnapshotWaiting && self.state != SyncState::Waiting && self.state != SyncState::Idle)
			&& !self.peers.values().any(|p| p.asking != PeerAsking::Nothing && p.block_set != Some(BlockSet::OldBlocks) && p.can_sync()) {

			self.complete_sync(io);
		}
	}

	/// Called after all blocks have been downloaded
	fn complete_sync(&mut self, io: &mut SyncIo) {
		trace!(target: "sync", "Sync complete");
		self.reset(io);
		self.state = SyncState::Idle;
	}

	/// Enter waiting state
	fn pause_sync(&mut self) {
		trace!(target: "sync", "Block queue full, pausing sync");
		self.state = SyncState::Waiting;
	}

	/// Find something to do for a peer. Called for a new peer or when a peer is done with its task.
	fn sync_peer(&mut self, io: &mut SyncIo, peer_id: PeerId, force: bool) {
		if !self.active_peers.contains(&peer_id) {
			trace!(target: "sync", "Skipping deactivated peer {}", peer_id);
			return;
		}
		let (peer_latest, peer_difficulty, peer_snapshot_number, peer_snapshot_hash) = {
			if let Some(peer) = self.peers.get_mut(&peer_id) {
				if peer.asking != PeerAsking::Nothing || !peer.can_sync() {
					trace!(target: "sync", "Skipping busy peer {}", peer_id);
					return;
				}
				if self.state == SyncState::Waiting {
					trace!(target: "sync", "Waiting for the block queue");
					return;
				}
				if self.state == SyncState::SnapshotWaiting {
					trace!(target: "sync", "Waiting for the snapshot restoration");
					return;
				}
				(peer.latest_hash.clone(), peer.difficulty.clone(), peer.snapshot_number.as_ref().cloned().unwrap_or(0), peer.snapshot_hash.as_ref().cloned())
			} else {
				return;
			}
		};
		let chain_info = io.chain().chain_info();
		let syncing_difficulty = chain_info.pending_total_difficulty;
		let num_active_peers = self.peers.values().filter(|p| p.asking != PeerAsking::Nothing).count();

		let higher_difficulty = peer_difficulty.map_or(true, |pd| pd > syncing_difficulty);
		if force || higher_difficulty || self.old_blocks.is_some() {
			match self.state {
				SyncState::WaitingPeers => {
					trace!(target: "sync", "Checking snapshot sync: {} vs {}", peer_snapshot_number, chain_info.best_block_number);
					self.maybe_start_snapshot_sync(io);
				},
				SyncState::Idle | SyncState::Blocks | SyncState::NewBlocks => {
					if io.chain().queue_info().is_full() {
						self.pause_sync();
						return;
					}

					let have_latest = io.chain().block_status(BlockId::Hash(peer_latest)) != BlockStatus::Unknown;
					trace!(target: "sync", "Considering peer {}, force={}, td={:?}, our td={}, latest={}, have_latest={}, state={:?}", peer_id, force, peer_difficulty, syncing_difficulty, peer_latest, have_latest, self.state);
					if !have_latest && (higher_difficulty || force || self.state == SyncState::NewBlocks) {
						// check if got new blocks to download
						trace!(target: "sync", "Syncing with peer {}, force={}, td={:?}, our td={}, state={:?}", peer_id, force, peer_difficulty, syncing_difficulty, self.state);
						if let Some(request) = self.new_blocks.request_blocks(io, num_active_peers) {
							self.request_blocks(io, peer_id, request, BlockSet::NewBlocks);
							if self.state == SyncState::Idle {
								self.state = SyncState::Blocks;
							}
							return;
						}
					}

					if let Some(request) = self.old_blocks.as_mut().and_then(|d| d.request_blocks(io, num_active_peers)) {
						self.request_blocks(io, peer_id, request, BlockSet::OldBlocks);
						return;
					}
				},
				SyncState::SnapshotData => {
					if let RestorationStatus::Ongoing { state_chunks_done, block_chunks_done, .. } = io.snapshot_service().status() {
						if self.snapshot.done_chunks() - (state_chunks_done + block_chunks_done) as usize > MAX_SNAPSHOT_CHUNKS_DOWNLOAD_AHEAD {
							trace!(target: "sync", "Snapshot queue full, pausing sync");
							self.state = SyncState::SnapshotWaiting;
							return;
						}
					}
					if peer_snapshot_hash.is_some() && peer_snapshot_hash == self.snapshot.snapshot_hash() {
						self.request_snapshot_data(io, peer_id);
					}
				},
				SyncState::SnapshotManifest | //already downloading from other peer
					SyncState::Waiting | SyncState::SnapshotWaiting => ()
			}
		} else {
			trace!(target: "sync", "Skipping peer {}, force={}, td={:?}, our td={}, state={:?}", peer_id, force, peer_difficulty, syncing_difficulty, self.state);
		}
	}

	/// Perofrm block download request`
	fn request_blocks(&mut self, io: &mut SyncIo, peer_id: PeerId, request: BlockRequest, block_set: BlockSet) {
		match request {
			BlockRequest::Headers { start, count, skip } => {
				self.request_headers_by_hash(io, peer_id, &start, count, skip, false, block_set);
			},
			BlockRequest::Bodies { hashes } => {
				self.request_bodies(io, peer_id, hashes, block_set);
			},
			BlockRequest::Receipts { hashes } => {
				self.request_receipts(io, peer_id, hashes, block_set);
			},
		}
	}

	/// Find some headers or blocks to download for a peer.
	fn request_snapshot_data(&mut self, io: &mut SyncIo, peer_id: PeerId) {
		self.clear_peer_download(peer_id);
		// find chunk data to download
		if let Some(hash) = self.snapshot.needed_chunk() {
			if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
				peer.asking_snapshot_data = Some(hash.clone());
			}
			self.request_snapshot_chunk(io, peer_id, &hash);
		}
	}

	/// Clear all blocks/headers marked as being downloaded by a peer.
	fn clear_peer_download(&mut self, peer_id: PeerId) {
		if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
			match peer.asking {
				PeerAsking::BlockHeaders => {
					if let Some(ref hash) = peer.asking_hash {
						self.new_blocks.clear_header_download(hash);
						if let Some(ref mut old) = self.old_blocks {
							old.clear_header_download(hash);
						}
					}
				},
				PeerAsking::BlockBodies => {
					self.new_blocks.clear_body_download(&peer.asking_blocks);
					if let Some(ref mut old) = self.old_blocks {
						old.clear_body_download(&peer.asking_blocks);
					}
				},
				PeerAsking::BlockReceipts => {
					self.new_blocks.clear_receipt_download(&peer.asking_blocks);
					if let Some(ref mut old) = self.old_blocks {
						old.clear_receipt_download(&peer.asking_blocks);
					}
				},
				PeerAsking::SnapshotData => {
					if let Some(hash) = peer.asking_snapshot_data {
						self.snapshot.clear_chunk_download(&hash);
					}
				},
				_ => (),
			}
		}
	}

	/// Checks if there are blocks fully downloaded that can be imported into the blockchain and does the import.
	#[cfg_attr(feature="dev", allow(block_in_if_condition_stmt))]
	fn collect_blocks(&mut self, io: &mut SyncIo, block_set: BlockSet) {
		match block_set {
			BlockSet::NewBlocks => {
				if self.new_blocks.collect_blocks(io, self.state == SyncState::NewBlocks) == Err(DownloaderImportError::Invalid) {
					self.restart(io);
				}
			},
			BlockSet::OldBlocks => {
				if self.old_blocks.as_mut().map_or(false, |downloader| { downloader.collect_blocks(io, false) == Err(DownloaderImportError::Invalid) }) {
					self.restart(io);
				} else if self.old_blocks.as_ref().map_or(false, |downloader| { downloader.is_complete() }) {
					trace!(target: "sync", "Background block download is complete");
					self.old_blocks = None;
				}
			}
		}
	}

	/// Request headers from a peer by block hash
	#[cfg_attr(feature="dev", allow(too_many_arguments))]
	fn request_headers_by_hash(&mut self, sync: &mut SyncIo, peer_id: PeerId, h: &H256, count: u64, skip: u64, reverse: bool, set: BlockSet) {
		trace!(target: "sync", "{} <- GetBlockHeaders: {} entries starting from {}, set = {:?}", peer_id, count, h, set);
		let mut rlp = RlpStream::new_list(4);
		rlp.append(h);
		rlp.append(&count);
		rlp.append(&skip);
		rlp.append(&if reverse {1u32} else {0u32});
		self.send_request(sync, peer_id, PeerAsking::BlockHeaders, GET_BLOCK_HEADERS_PACKET, rlp.out());
		let peer = self.peers.get_mut(&peer_id).expect("peer_id may originate either from on_packet, where it is already validated or from enumerating self.peers. qed");
		peer.asking_hash = Some(h.clone());
		peer.block_set = Some(set);
	}

	/// Request headers from a peer by block number
	#[cfg_attr(feature="dev", allow(too_many_arguments))]
	fn request_fork_header_by_number(&mut self, sync: &mut SyncIo, peer_id: PeerId, n: BlockNumber) {
		trace!(target: "sync", "{} <- GetForkHeader: at {}", peer_id, n);
		let mut rlp = RlpStream::new_list(4);
		rlp.append(&n);
		rlp.append(&1u32);
		rlp.append(&0u32);
		rlp.append(&0u32);
		self.send_request(sync, peer_id, PeerAsking::ForkHeader, GET_BLOCK_HEADERS_PACKET, rlp.out());
	}

	/// Request snapshot manifest from a peer.
	fn request_snapshot_manifest(&mut self, sync: &mut SyncIo, peer_id: PeerId) {
		trace!(target: "sync", "{} <- GetSnapshotManifest", peer_id);
		let rlp = RlpStream::new_list(0);
		self.send_request(sync, peer_id, PeerAsking::SnapshotManifest, GET_SNAPSHOT_MANIFEST_PACKET, rlp.out());
	}

	/// Request snapshot chunk from a peer.
	fn request_snapshot_chunk(&mut self, sync: &mut SyncIo, peer_id: PeerId, chunk: &H256) {
		trace!(target: "sync", "{} <- GetSnapshotData {:?}", peer_id, chunk);
		let mut rlp = RlpStream::new_list(1);
		rlp.append(chunk);
		self.send_request(sync, peer_id, PeerAsking::SnapshotData, GET_SNAPSHOT_DATA_PACKET, rlp.out());
	}

	/// Request block bodies from a peer
	fn request_bodies(&mut self, sync: &mut SyncIo, peer_id: PeerId, hashes: Vec<H256>, set: BlockSet) {
		let mut rlp = RlpStream::new_list(hashes.len());
		trace!(target: "sync", "{} <- GetBlockBodies: {} entries starting from {:?}, set = {:?}", peer_id, hashes.len(), hashes.first(), set);
		for h in &hashes {
			rlp.append(&h.clone());
		}
		self.send_request(sync, peer_id, PeerAsking::BlockBodies, GET_BLOCK_BODIES_PACKET, rlp.out());
		let peer = self.peers.get_mut(&peer_id).expect("peer_id may originate either from on_packet, where it is already validated or from enumerating self.peers. qed");
		peer.asking_blocks = hashes;
		peer.block_set = Some(set);
	}

	/// Request block receipts from a peer
	fn request_receipts(&mut self, sync: &mut SyncIo, peer_id: PeerId, hashes: Vec<H256>, set: BlockSet) {
		let mut rlp = RlpStream::new_list(hashes.len());
		trace!(target: "sync", "{} <- GetBlockReceipts: {} entries starting from {:?}, set = {:?}", peer_id, hashes.len(), hashes.first(), set);
		for h in &hashes {
			rlp.append(&h.clone());
		}
		self.send_request(sync, peer_id, PeerAsking::BlockReceipts, GET_RECEIPTS_PACKET, rlp.out());
		let peer = self.peers.get_mut(&peer_id).expect("peer_id may originate either from on_packet, where it is already validated or from enumerating self.peers. qed");
		peer.asking_blocks = hashes;
		peer.block_set = Some(set);
	}

	/// Reset peer status after request is complete.
	fn reset_peer_asking(&mut self, peer_id: PeerId, asking: PeerAsking) -> bool {
		if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
			peer.expired = false;
			peer.block_set = None;
			if peer.asking != asking {
				trace!(target:"sync", "Asking {:?} while expected {:?}", peer.asking, asking);
				peer.asking = PeerAsking::Nothing;
				return false;
			} else {
				peer.asking = PeerAsking::Nothing;
				return true;
			}
		}
		false
	}

	/// Generic request sender
	fn send_request(&mut self, sync: &mut SyncIo, peer_id: PeerId, asking: PeerAsking,  packet_id: PacketId, packet: Bytes) {
		if let Some(ref mut peer) = self.peers.get_mut(&peer_id) {
			if peer.asking != PeerAsking::Nothing {
				warn!(target:"sync", "Asking {:?} while requesting {:?}", peer.asking, asking);
			}
			peer.asking = asking;
			peer.ask_time = time::precise_time_ns();
			let result = if packet_id >= ETH_PACKET_COUNT {
				sync.send_protocol(WARP_SYNC_PROTOCOL_ID, peer_id, packet_id, packet)
			} else {
				sync.send(peer_id, packet_id, packet)
			};
			if let Err(e) = result {
				debug!(target:"sync", "Error sending request: {:?}", e);
				sync.disable_peer(peer_id);
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
		// Accept transactions only when fully synced
		if !io.is_chain_queue_empty() || (self.state != SyncState::Idle && self.state != SyncState::NewBlocks) {
			trace!(target: "sync", "{} Ignoring transactions while syncing", peer_id);
			return Ok(());
		}
		if !self.peers.get(&peer_id).map_or(false, |p| p.can_sync()) {
			trace!(target: "sync", "{} Ignoring transactions from unconfirmed/unknown peer", peer_id);
		}

		let mut item_count = r.item_count()?;
		trace!(target: "sync", "{:02} -> Transactions ({} entries)", peer_id, item_count);
		item_count = cmp::min(item_count, MAX_TX_TO_IMPORT);
		let mut transactions = Vec::with_capacity(item_count);
		for i in 0 .. item_count {
			let rlp = r.at(i)?;
			if rlp.as_raw().len() > MAX_TRANSACTION_SIZE {
				debug!("Skipped oversized transaction of {} bytes", rlp.as_raw().len());
				continue;
			}
			let tx = rlp.as_raw().to_vec();
			transactions.push(tx);
		}
		io.chain().queue_transactions(transactions, peer_id);
		Ok(())
	}

	/// Send Status message
	fn send_status(&mut self, io: &mut SyncIo, peer: PeerId) -> Result<(), NetworkError> {
		let warp_protocol_version = io.protocol_version(&WARP_SYNC_PROTOCOL_ID, peer);
		let warp_protocol = warp_protocol_version != 0;
		let protocol = if warp_protocol { warp_protocol_version } else { PROTOCOL_VERSION_63 };
		trace!(target: "sync", "Sending status to {}, protocol version {}", peer, protocol);
		let mut packet = RlpStream::new_list(if warp_protocol { 7 } else { 5 });
		let chain = io.chain().chain_info();
		packet.append(&(protocol as u32));
		packet.append(&self.network_id);
		packet.append(&chain.total_difficulty);
		packet.append(&chain.best_block_hash);
		packet.append(&chain.genesis_hash);
		if warp_protocol {
			let manifest = match self.old_blocks.is_some() {
				true => None,
				false => io.snapshot_service().manifest(),
			};
			let block_number = manifest.as_ref().map_or(0, |m| m.block_number);
			let manifest_hash = manifest.map_or(H256::new(), |m| m.into_rlp().sha3());
			packet.append(&manifest_hash);
			packet.append(&block_number);
		}
		io.respond(STATUS_PACKET, packet.out())
	}

	/// Respond to GetBlockHeaders request
	fn return_block_headers(io: &SyncIo, r: &UntrustedRlp, peer_id: PeerId) -> RlpResponseResult {
		// Packet layout:
		// [ block: { P , B_32 }, maxHeaders: P, skip: P, reverse: P in { 0 , 1 } ]
		let max_headers: usize = r.val_at(1)?;
		let skip: usize = r.val_at(2)?;
		let reverse: bool = r.val_at(3)?;
		let last = io.chain().chain_info().best_block_number;
		let number = if r.at(0)?.size() == 32 {
			// id is a hash
			let hash: H256 = r.val_at(0)?;
			trace!(target: "sync", "{} -> GetBlockHeaders (hash: {}, max: {}, skip: {}, reverse:{})", peer_id, hash, max_headers, skip, reverse);
			match io.chain().block_header(BlockId::Hash(hash)) {
				Some(hdr) => {
					let number = hdr.number().into();
					debug_assert_eq!(hdr.sha3(), hash);

					if max_headers == 1 || io.chain().block_hash(BlockId::Number(number)) != Some(hash) {
						// Non canonical header or single header requested
						// TODO: handle single-step reverse hashchains of non-canon hashes
						trace!(target:"sync", "Returning single header: {:?}", hash);
						let mut rlp = RlpStream::new_list(1);
						rlp.append_raw(&hdr.into_inner(), 1);
						return Ok(Some((BLOCK_HEADERS_PACKET, rlp)));
					}
					number
				}
				None => return Ok(Some((BLOCK_HEADERS_PACKET, RlpStream::new_list(0)))) //no such header, return nothing
			}
		} else {
			trace!(target: "sync", "{} -> GetBlockHeaders (number: {}, max: {}, skip: {}, reverse:{})", peer_id, r.val_at::<BlockNumber>(0)?, max_headers, skip, reverse);
			r.val_at(0)?
		};

		let mut number = if reverse {
			cmp::min(last, number)
		} else {
			cmp::max(0, number)
		};
		let max_count = cmp::min(MAX_HEADERS_TO_SEND, max_headers);
		let mut count = 0;
		let mut data = Bytes::new();
		let inc = (skip + 1) as BlockNumber;
		let overlay = io.chain_overlay().read();

		while number <= last && count < max_count {
			if let Some(hdr) = overlay.get(&number) {
				trace!(target: "sync", "{}: Returning cached fork header", peer_id);
				data.extend_from_slice(hdr);
				count += 1;
			} else if let Some(hdr) = io.chain().block_header(BlockId::Number(number)) {
				data.append(&mut hdr.into_inner());
				count += 1;
			} else {
				// No required block.
				break;
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
		trace!(target: "sync", "{} -> GetBlockHeaders: returned {} entries", peer_id, count);
		Ok(Some((BLOCK_HEADERS_PACKET, rlp)))
	}

	/// Respond to GetBlockBodies request
	fn return_block_bodies(io: &SyncIo, r: &UntrustedRlp, peer_id: PeerId) -> RlpResponseResult {
		let mut count = r.item_count().unwrap_or(0);
		if count == 0 {
			debug!(target: "sync", "Empty GetBlockBodies request, ignoring.");
			return Ok(None);
		}
		count = cmp::min(count, MAX_BODIES_TO_SEND);
		let mut added = 0usize;
		let mut data = Bytes::new();
		for i in 0..count {
			if let Some(body) = io.chain().block_body(BlockId::Hash(r.val_at::<H256>(i)?)) {
				data.append(&mut body.into_inner());
				added += 1;
			}
		}
		let mut rlp = RlpStream::new_list(added);
		rlp.append_raw(&data, added);
		trace!(target: "sync", "{} -> GetBlockBodies: returned {} entries", peer_id, added);
		Ok(Some((BLOCK_BODIES_PACKET, rlp)))
	}

	/// Respond to GetNodeData request
	fn return_node_data(io: &SyncIo, r: &UntrustedRlp, peer_id: PeerId) -> RlpResponseResult {
		let mut count = r.item_count().unwrap_or(0);
		trace!(target: "sync", "{} -> GetNodeData: {} entries", peer_id, count);
		if count == 0 {
			debug!(target: "sync", "Empty GetNodeData request, ignoring.");
			return Ok(None);
		}
		count = cmp::min(count, MAX_NODE_DATA_TO_SEND);
		let mut added = 0usize;
		let mut data = Vec::new();
		for i in 0..count {
			if let Some(node) = io.chain().state_data(&r.val_at::<H256>(i)?) {
				data.push(node);
				added += 1;
			}
		}
		trace!(target: "sync", "{} -> GetNodeData: return {} entries", peer_id, added);
		let mut rlp = RlpStream::new_list(added);
		for d in data {
			rlp.append(&d);
		}
		Ok(Some((NODE_DATA_PACKET, rlp)))
	}

	fn return_receipts(io: &SyncIo, rlp: &UntrustedRlp, peer_id: PeerId) -> RlpResponseResult {
		let mut count = rlp.item_count().unwrap_or(0);
		trace!(target: "sync", "{} -> GetReceipts: {} entries", peer_id, count);
		if count == 0 {
			debug!(target: "sync", "Empty GetReceipts request, ignoring.");
			return Ok(None);
		}
		count = cmp::min(count, MAX_RECEIPTS_HEADERS_TO_SEND);
		let mut added_headers = 0usize;
		let mut added_receipts = 0usize;
		let mut data = Bytes::new();
		for i in 0..count {
			if let Some(mut receipts_bytes) = io.chain().block_receipts(&rlp.val_at::<H256>(i)?) {
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

	/// Respond to GetSnapshotManifest request
	fn return_snapshot_manifest(io: &SyncIo, r: &UntrustedRlp, peer_id: PeerId) -> RlpResponseResult {
		let count = r.item_count().unwrap_or(0);
		trace!(target: "sync", "{} -> GetSnapshotManifest", peer_id);
		if count != 0 {
			debug!(target: "sync", "Invalid GetSnapshotManifest request, ignoring.");
			return Ok(None);
		}
		let rlp = match io.snapshot_service().manifest() {
			Some(manifest) => {
				trace!(target: "sync", "{} <- SnapshotManifest", peer_id);
				let mut rlp = RlpStream::new_list(1);
				rlp.append_raw(&manifest.into_rlp(), 1);
				rlp
			},
			None => {
				trace!(target: "sync", "{}: No manifest to return", peer_id);
				RlpStream::new_list(0)
			}
		};
		Ok(Some((SNAPSHOT_MANIFEST_PACKET, rlp)))
	}

	/// Respond to GetSnapshotData request
	fn return_snapshot_data(io: &SyncIo, r: &UntrustedRlp, peer_id: PeerId) -> RlpResponseResult {
		let hash: H256 = r.val_at(0)?;
		trace!(target: "sync", "{} -> GetSnapshotData {:?}", peer_id, hash);
		let rlp = match io.snapshot_service().chunk(hash) {
			Some(data) => {
				let mut rlp = RlpStream::new_list(1);
				trace!(target: "sync", "{} <- SnapshotData", peer_id);
				rlp.append(&data);
				rlp
			},
			None => {
				RlpStream::new_list(0)
			}
		};
		Ok(Some((SNAPSHOT_DATA_PACKET, rlp)))
	}

	fn return_rlp<FRlp, FError>(io: &mut SyncIo, rlp: &UntrustedRlp, peer: PeerId, rlp_func: FRlp, error_func: FError) -> Result<(), PacketDecodeError>
		where FRlp : Fn(&SyncIo, &UntrustedRlp, PeerId) -> RlpResponseResult,
			FError : FnOnce(NetworkError) -> String
	{
		let response = rlp_func(io, rlp, peer);
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
	pub fn dispatch_packet(sync: &RwLock<ChainSync>, io: &mut SyncIo, peer: PeerId, packet_id: u8, data: &[u8]) {
		let rlp = UntrustedRlp::new(data);
		let result = match packet_id {
			GET_BLOCK_BODIES_PACKET => ChainSync::return_rlp(io, &rlp, peer,
				ChainSync::return_block_bodies,
				|e| format!("Error sending block bodies: {:?}", e)),

			GET_BLOCK_HEADERS_PACKET => ChainSync::return_rlp(io, &rlp, peer,
				ChainSync::return_block_headers,
				|e| format!("Error sending block headers: {:?}", e)),

			GET_RECEIPTS_PACKET => ChainSync::return_rlp(io, &rlp, peer,
				ChainSync::return_receipts,
				|e| format!("Error sending receipts: {:?}", e)),

			GET_NODE_DATA_PACKET => ChainSync::return_rlp(io, &rlp, peer,
				ChainSync::return_node_data,
				|e| format!("Error sending nodes: {:?}", e)),

			GET_SNAPSHOT_MANIFEST_PACKET => ChainSync::return_rlp(io, &rlp, peer,
				ChainSync::return_snapshot_manifest,
				|e| format!("Error sending snapshot manifest: {:?}", e)),

			GET_SNAPSHOT_DATA_PACKET => ChainSync::return_rlp(io, &rlp, peer,
				ChainSync::return_snapshot_data,
				|e| format!("Error sending snapshot data: {:?}", e)),
			CONSENSUS_DATA_PACKET => ChainSync::on_consensus_packet(io, peer, &rlp),
			_ => {
				sync.write().on_packet(io, peer, packet_id, data);
				Ok(())
			}
		};
		result.unwrap_or_else(|e| {
			debug!(target:"sync", "{} -> Malformed packet {} : {}", peer, packet_id, e);
		})
	}

	pub fn on_packet(&mut self, io: &mut SyncIo, peer: PeerId, packet_id: u8, data: &[u8]) {
		if packet_id != STATUS_PACKET && !self.peers.contains_key(&peer) {
			debug!(target:"sync", "Unexpected packet {} from unregistered peer: {}:{}", packet_id, peer, io.peer_info(peer));
			return;
		}
		let rlp = UntrustedRlp::new(data);
		let result = match packet_id {
			STATUS_PACKET => self.on_peer_status(io, peer, &rlp),
			TRANSACTIONS_PACKET => self.on_peer_transactions(io, peer, &rlp),
			BLOCK_HEADERS_PACKET => self.on_peer_block_headers(io, peer, &rlp),
			BLOCK_BODIES_PACKET => self.on_peer_block_bodies(io, peer, &rlp),
			RECEIPTS_PACKET => self.on_peer_block_receipts(io, peer, &rlp),
			NEW_BLOCK_PACKET => self.on_peer_new_block(io, peer, &rlp),
			NEW_BLOCK_HASHES_PACKET => self.on_peer_new_hashes(io, peer, &rlp),
			SNAPSHOT_MANIFEST_PACKET => self.on_snapshot_manifest(io, peer, &rlp),
			SNAPSHOT_DATA_PACKET => self.on_snapshot_data(io, peer, &rlp),
			_ => {
				debug!(target: "sync", "{}: Unknown packet {}", peer, packet_id);
				Ok(())
			}
		};
		result.unwrap_or_else(|e| {
			debug!(target:"sync", "{} -> Malformed packet {} : {}", peer, packet_id, e);
		})
	}

	#[cfg_attr(feature="dev", allow(match_same_arms))]
	pub fn maintain_peers(&mut self, io: &mut SyncIo) {
		let tick = time::precise_time_ns();
		let mut aborting = Vec::new();
		for (peer_id, peer) in &self.peers {
			let elapsed = (tick - peer.ask_time) / 1_000_000_000;
			let timeout = match peer.asking {
				PeerAsking::BlockHeaders => elapsed > HEADERS_TIMEOUT_SEC,
				PeerAsking::BlockBodies => elapsed > BODIES_TIMEOUT_SEC,
				PeerAsking::BlockReceipts => elapsed > RECEIPTS_TIMEOUT_SEC,
				PeerAsking::Nothing => false,
				PeerAsking::ForkHeader => elapsed > FORK_HEADER_TIMEOUT_SEC,
				PeerAsking::SnapshotManifest => elapsed > SNAPSHOT_MANIFEST_TIMEOUT_SEC,
				PeerAsking::SnapshotData => elapsed > SNAPSHOT_DATA_TIMEOUT_SEC,
			};
			if timeout {
				trace!(target:"sync", "Timeout {}", peer_id);
				io.disconnect_peer(*peer_id);
				aborting.push(*peer_id);
			}
		}
		for p in aborting {
			self.on_peer_aborting(io, p);
		}

		// Check for handshake timeouts
		for (peer, ask_time) in &self.handshaking_peers {
			let elapsed = (tick - ask_time) / 1_000_000_000;
			if elapsed > STATUS_TIMEOUT_SEC {
				trace!(target:"sync", "Status timeout {}", peer);
				io.disconnect_peer(*peer);
			}
		}
	}

	fn check_resume(&mut self, io: &mut SyncIo) {
		if self.state == SyncState::Waiting && !io.chain().queue_info().is_full() && self.state == SyncState::Waiting {
			self.state = SyncState::Blocks;
			self.continue_sync(io);
		} else if self.state == SyncState::SnapshotWaiting {
			match io.snapshot_service().status() {
				RestorationStatus::Inactive => {
					trace!(target:"sync", "Snapshot restoration is complete");
					self.restart(io);
					self.continue_sync(io);
				},
				RestorationStatus::Ongoing { state_chunks_done, block_chunks_done, .. } => {
					if !self.snapshot.is_complete() && self.snapshot.done_chunks() - (state_chunks_done + block_chunks_done) as usize <= MAX_SNAPSHOT_CHUNKS_DOWNLOAD_AHEAD {
						trace!(target:"sync", "Resuming snapshot sync");
						self.state = SyncState::SnapshotData;
						self.continue_sync(io);
					}
				},
				RestorationStatus::Failed => {
					trace!(target: "sync", "Snapshot restoration aborted");
					self.state = SyncState::WaitingPeers;
					self.snapshot.clear();
					self.continue_sync(io);
				},
			}
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
							let number = chain.block_header(BlockId::Hash(block_hash.clone()))
								.expect("chain.tree_route and chain.find_uncles only return hahses of blocks that are in the blockchain. qed.").number();
							hash_rlp.append(&block_hash);
							hash_rlp.append(&number);
							rlp_stream.append_raw(hash_rlp.as_raw(), 1);
						}
						Some(rlp_stream.out())
					}
				}
			},
			None => None
		}
	}

	/// creates rlp from block bytes and total difficulty
	fn create_block_rlp(bytes: &Bytes, total_difficulty: U256) -> Bytes {
		let mut rlp_stream = RlpStream::new_list(2);
		rlp_stream.append_raw(bytes, 1);
		rlp_stream.append(&total_difficulty);
		rlp_stream.out()
	}

	/// creates latest block rlp for the given client
	fn create_latest_block_rlp(chain: &BlockChainClient) -> Bytes {
		ChainSync::create_block_rlp(
			&chain.block(BlockId::Hash(chain.chain_info().best_block_hash))
				.expect("Best block always exists").into_inner(),
			chain.chain_info().total_difficulty
		)
	}

	/// creates given hash block rlp for the given client
	fn create_new_block_rlp(chain: &BlockChainClient, hash: &H256) -> Bytes {
		ChainSync::create_block_rlp(
			&chain.block(BlockId::Hash(hash.clone())).expect("Block has just been sealed; qed").into_inner(),
			chain.block_total_difficulty(BlockId::Hash(hash.clone())).expect("Block has just been sealed; qed.")
		)
	}

	/// returns peer ids that have different blocks than our chain
	fn get_lagging_peers(&mut self, chain_info: &BlockChainInfo) -> Vec<PeerId> {
		let latest_hash = chain_info.best_block_hash;
		self
			.peers
			.iter_mut()
			.filter_map(|(&id, ref mut peer_info)| {
				trace!(target: "sync", "Checking peer our best {} their best {}", latest_hash, peer_info.latest_hash);
				if peer_info.latest_hash != latest_hash {
					Some(id)
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
	}

	fn select_random_peers(peers: &[PeerId]) -> Vec<PeerId> {
		// take sqrt(x) peers
		let mut peers = peers.to_vec();
		let mut count = (peers.len() as f64).powf(0.5).round() as usize;
		count = cmp::min(count, MAX_PEERS_PROPAGATION);
		count = cmp::max(count, MIN_PEERS_PROPAGATION);
		random::new().shuffle(&mut peers);
		peers.truncate(count);
		peers
	}

	fn get_consensus_peers(&self) -> Vec<PeerId> {
		self.peers.iter().filter_map(|(id, p)| if p.protocol_version == PROTOCOL_VERSION_2 { Some(*id) } else { None }).collect()
	}

	/// propagates latest block to a set of peers
	fn propagate_blocks(&mut self, chain_info: &BlockChainInfo, io: &mut SyncIo, blocks: &[H256], peers: &[PeerId]) -> usize {
		trace!(target: "sync", "Sending NewBlocks to {:?}", peers);
		let mut sent = 0;
		for peer_id in peers {
			if blocks.is_empty() {
				let rlp =  ChainSync::create_latest_block_rlp(io.chain());
				self.send_packet(io, *peer_id, NEW_BLOCK_PACKET, rlp);
			} else {
				for h in blocks {
					let rlp =  ChainSync::create_new_block_rlp(io.chain(), h);
					self.send_packet(io, *peer_id, NEW_BLOCK_PACKET, rlp);
				}
			}
			if let Some(ref mut peer) = self.peers.get_mut(peer_id) {
				peer.latest_hash = chain_info.best_block_hash.clone();
			}
			sent += 1;
		}
		sent
	}

	/// propagates new known hashes to all peers
	fn propagate_new_hashes(&mut self, chain_info: &BlockChainInfo, io: &mut SyncIo, peers: &[PeerId]) -> usize {
		trace!(target: "sync", "Sending NewHashes to {:?}", peers);
		let mut sent = 0;
		let last_parent = &io.chain().best_block_header().parent_hash();
		for peer_id in peers {
			sent += match ChainSync::create_new_hashes_rlp(io.chain(), &last_parent, &chain_info.best_block_hash) {
				Some(rlp) => {
					{
						if let Some(ref mut peer) = self.peers.get_mut(peer_id) {
							peer.latest_hash = chain_info.best_block_hash.clone();
						}
					}
					self.send_packet(io, *peer_id, NEW_BLOCK_HASHES_PACKET, rlp);
					1
				},
				None => 0
			}
		}
		sent
	}

	/// propagates new transactions to all peers
	pub fn propagate_new_transactions(&mut self, io: &mut SyncIo) -> usize {
		// Early out if nobody to send to.
		if self.peers.is_empty() {
			return 0;
		}

		let transactions = io.chain().ready_transactions();
		if transactions.is_empty() {
			return 0;
		}

		let (transactions, service_transactions): (Vec<_>, Vec<_>) = transactions.into_iter()
			.partition(|tx| !tx.transaction.gas_price.is_zero());

		// usual transactions could be propagated to all peers
		let mut affected_peers = HashSet::new();
		if !transactions.is_empty() {
			let peers = self.select_peers_for_transactions(|_| true);
			affected_peers = self.propagate_transactions_to_peers(io, peers, transactions);
		}

		// most of times service_transactions will be empty
		// => there's no need to merge packets
		if !service_transactions.is_empty() {
			let service_transactions_peers = self.select_peers_for_transactions(|peer_id| accepts_service_transaction(&io.peer_info(*peer_id)));
			let service_transactions_affected_peers = self.propagate_transactions_to_peers(io, service_transactions_peers, service_transactions);
			affected_peers.extend(&service_transactions_affected_peers);
		}

		affected_peers.len()
	}

	fn select_peers_for_transactions<F>(&self, filter: F) -> Vec<PeerId>
		where F: Fn(&PeerId) -> bool {
		// sqrt(x)/x scaled to max u32
		let fraction = ((self.peers.len() as f64).powf(-0.5) * (u32::max_value() as f64).round()) as u32;
		let small = self.peers.len() < MIN_PEERS_PROPAGATION;

		let mut random = random::new();
		self.peers.keys()
			.cloned()
			.filter(filter)
			.filter(|_| small || random.next_u32() < fraction)
			.take(MAX_PEERS_PROPAGATION)
			.collect()
	}

	fn propagate_transactions_to_peers(&mut self, io: &mut SyncIo, peers: Vec<PeerId>, transactions: Vec<PendingTransaction>) -> HashSet<PeerId> {
		let all_transactions_hashes = transactions.iter()
			.map(|tx| tx.transaction.hash())
			.collect::<HashSet<H256>>();
		let all_transactions_rlp = {
			let mut packet = RlpStream::new_list(transactions.len());
			for tx in &transactions { packet.append(&tx.transaction); }
			packet.out()
		};

		// Clear old transactions from stats
		self.transactions_stats.retain(&all_transactions_hashes);

		// sqrt(x)/x scaled to max u32
		let block_number = io.chain().chain_info().best_block_number;

		let lucky_peers = {
			peers.into_iter()
				.filter_map(|peer_id| {
					let stats = &mut self.transactions_stats;
					let peer_info = self.peers.get_mut(&peer_id)
						.expect("peer_id is form peers; peers is result of select_peers_for_transactions; select_peers_for_transactions selects peers from self.peers; qed");

					// Send all transactions
					if peer_info.last_sent_transactions.is_empty() {
						// update stats
						for hash in &all_transactions_hashes {
							let id = io.peer_session_info(peer_id).and_then(|info| info.id);
							stats.propagated(hash, id, block_number);
						}
						peer_info.last_sent_transactions = all_transactions_hashes.clone();
						return Some((peer_id, all_transactions_hashes.len(), all_transactions_rlp.clone()));
					}

					// Get hashes of all transactions to send to this peer
					let to_send = all_transactions_hashes.difference(&peer_info.last_sent_transactions)
						.take(MAX_TRANSACTIONS_TO_PROPAGATE)
						.cloned()
						.collect::<HashSet<_>>();
					if to_send.is_empty() {
						return None;
					}

					// Construct RLP
					let (packet, to_send) = {
						let mut to_send = to_send;
						let mut packet = RlpStream::new();
						packet.begin_unbounded_list();
						let mut pushed = 0;
						for tx in &transactions {
							let hash = tx.transaction.hash();
							if to_send.contains(&hash) {
								let mut transaction = RlpStream::new();
								tx.transaction.rlp_append(&mut transaction);
								let appended = packet.append_raw_checked(&transaction.drain(), 1, MAX_TRANSACTION_PACKET_SIZE);
								if !appended {
									// Maximal packet size reached just proceed with sending
									debug!("Transaction packet size limit reached. Sending incomplete set of {}/{} transactions.", pushed, to_send.len());
									to_send = to_send.into_iter().take(pushed).collect();
									break;
								}
								pushed += 1;
							}
						}
						packet.complete_unbounded_list();
						(packet, to_send)
					};

					// Update stats
					let id = io.peer_session_info(peer_id).and_then(|info| info.id);
					for hash in &to_send {
						// update stats
						stats.propagated(hash, id, block_number);
					}

					peer_info.last_sent_transactions = all_transactions_hashes
						.intersection(&peer_info.last_sent_transactions)
						.chain(&to_send)
						.cloned()
						.collect();
					Some((peer_id, to_send.len(), packet.out()))
				})
				.collect::<Vec<_>>()
		};

		// Send RLPs
		let mut peers = HashSet::new();
		if lucky_peers.len() > 0 {
			let mut max_sent = 0;
			let lucky_peers_len = lucky_peers.len();
			for (peer_id, sent, rlp) in lucky_peers {
				peers.insert(peer_id);
				self.send_packet(io, peer_id, TRANSACTIONS_PACKET, rlp);
				trace!(target: "sync", "{:02} <- Transactions ({} entries)", peer_id, sent);
				max_sent = cmp::max(max_sent, sent);
			}
			debug!(target: "sync", "Sent up to {} transactions to {} peers.", max_sent, lucky_peers_len);
		}

		peers
	}

	fn propagate_latest_blocks(&mut self, io: &mut SyncIo, sealed: &[H256]) {
		let chain_info = io.chain().chain_info();
		if (((chain_info.best_block_number as i64) - (self.last_sent_block_number as i64)).abs() as BlockNumber) < MAX_PEER_LAG_PROPAGATION {
			let mut peers = self.get_lagging_peers(&chain_info);
			if sealed.is_empty() {
				let hashes = self.propagate_new_hashes(&chain_info, io, &peers);
				peers = ChainSync::select_random_peers(&peers);
				let blocks = self.propagate_blocks(&chain_info, io, sealed, &peers);
				if blocks != 0 || hashes != 0 {
					trace!(target: "sync", "Sent latest {} blocks and {} hashes to peers.", blocks, hashes);
				}
			} else {
				self.propagate_blocks(&chain_info, io, sealed, &peers);
				self.propagate_new_hashes(&chain_info, io, &peers);
				trace!(target: "sync", "Sent sealed block to all peers");
			};
		}
		self.last_sent_block_number = chain_info.best_block_number;
	}

	/// Distribute valid proposed blocks to subset of current peers.
	fn propagate_proposed_blocks(&mut self, io: &mut SyncIo, proposed: &[Bytes]) {
		let peers = self.get_consensus_peers();
		trace!(target: "sync", "Sending proposed blocks to {:?}", peers);
		for block in proposed {
			let rlp = ChainSync::create_block_rlp(
				block,
				io.chain().chain_info().total_difficulty
			);
			for peer_id in &peers {
				self.send_packet(io, *peer_id, NEW_BLOCK_PACKET, rlp.clone());
			}
		}
	}

	/// Maintain other peers. Send out any new blocks and transactions
	pub fn maintain_sync(&mut self, io: &mut SyncIo) {
		self.maybe_start_snapshot_sync(io);
		self.check_resume(io);
	}

	/// called when block is imported to chain - propagates the blocks and updates transactions sent to peers
	pub fn chain_new_blocks(&mut self, io: &mut SyncIo, _imported: &[H256], invalid: &[H256], enacted: &[H256], _retracted: &[H256], sealed: &[H256], proposed: &[Bytes]) {
		let queue_info = io.chain().queue_info();
		let is_syncing = self.status().is_syncing(queue_info);

		if !is_syncing || !sealed.is_empty() || !proposed.is_empty() {
			trace!(target: "sync", "Propagating blocks, state={:?}", self.state);
			self.propagate_latest_blocks(io, sealed);
			self.propagate_proposed_blocks(io, proposed);
		}
		if !invalid.is_empty() {
			trace!(target: "sync", "Bad blocks in the queue, restarting");
			self.restart(io);
		}

		if !is_syncing && !enacted.is_empty() && !self.peers.is_empty() {
			// Select random peer to re-broadcast transactions to.
			let peer = random::new().gen_range(0, self.peers.len());
			trace!(target: "sync", "Re-broadcasting transactions to a random peer.");
			self.peers.values_mut().nth(peer).map(|mut peer_info|
				peer_info.last_sent_transactions.clear()
			);
		}
	}

	/// Called when peer sends us new consensus packet
	fn on_consensus_packet(io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		trace!(target: "sync", "Received consensus packet from {:?}", peer_id);
		io.chain().queue_consensus_message(r.as_raw().to_vec());
		Ok(())
	}

	/// Broadcast consensus message to peers.
	pub fn propagate_consensus_packet(&mut self, io: &mut SyncIo, packet: Bytes) {
		let lucky_peers = ChainSync::select_random_peers(&self.get_consensus_peers());
		trace!(target: "sync", "Sending consensus packet to {:?}", lucky_peers);
		for peer_id in lucky_peers {
			self.send_packet(io, peer_id, CONSENSUS_DATA_PACKET, packet.clone());
		}
	}
}

/// Checks if peer is able to process service transactions
fn accepts_service_transaction(client_id: &str) -> bool {
	// Parity versions starting from this will accept service-transactions
	const SERVICE_TRANSACTIONS_VERSION: (u32, u32) = (1u32, 6u32);
	// Parity client string prefix
	const PARITY_CLIENT_ID_PREFIX: &'static str = "Parity/v";

	if !client_id.starts_with(PARITY_CLIENT_ID_PREFIX) {
		return false;
	}
	let ver: Vec<u32> = client_id[PARITY_CLIENT_ID_PREFIX.len()..].split('.')
		.take(2)
		.filter_map(|s| s.parse().ok())
		.collect();
	ver.len() == 2 && (ver[0] > SERVICE_TRANSACTIONS_VERSION.0 || (ver[0] == SERVICE_TRANSACTIONS_VERSION.0 && ver[1] >= SERVICE_TRANSACTIONS_VERSION.1))
}

#[cfg(test)]
mod tests {
	use std::collections::{HashSet, VecDeque};
	use network::PeerId;
	use tests::helpers::*;
	use tests::snapshot::TestSnapshotService;
	use util::{U256, Address, RwLock};
	use util::sha3::Hashable;
	use util::hash::H256;
	use util::bytes::Bytes;
	use rlp::{Rlp, RlpStream, UntrustedRlp};
	use super::*;
	use ::SyncConfig;
	use super::{PeerInfo, PeerAsking};
	use ethkey;
	use ethcore::header::*;
	use ethcore::client::{BlockChainClient, EachBlockWith, TestBlockChainClient};
	use ethcore::transaction::UnverifiedTransaction;
	use ethcore::miner::MinerService;

	fn get_dummy_block(order: u32, parent_hash: H256) -> Bytes {
		let mut header = Header::new();
		header.set_gas_limit(0.into());
		header.set_difficulty((order * 100).into());
		header.set_timestamp((order * 10) as u64);
		header.set_number(order as u64);
		header.set_parent_hash(parent_hash);
		header.set_state_root(H256::zero());

		let mut rlp = RlpStream::new_list(3);
		rlp.append(&header);
		rlp.append_raw(&::rlp::EMPTY_LIST_RLP, 1);
		rlp.append_raw(&::rlp::EMPTY_LIST_RLP, 1);
		rlp.out()
	}

	fn get_dummy_blocks(order: u32, parent_hash: H256) -> Bytes {
		let mut rlp = RlpStream::new_list(1);
		rlp.append_raw(&get_dummy_block(order, parent_hash), 1);
		let difficulty: U256 = (100 * order).into();
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

	fn queue_info(unverified: usize, verified: usize) -> BlockQueueInfo {
		BlockQueueInfo {
			unverified_queue_size: unverified,
			verified_queue_size: verified,
			verifying_queue_size: 0,
			max_queue_size: 1000,
			max_mem_use: 1000,
			mem_used: 500
		}
	}

	fn sync_status(state: SyncState) -> SyncStatus {
		SyncStatus {
			state: state,
			protocol_version: 0,
			network_id: 0,
			start_block_number: 0,
			last_imported_block_number: None,
			highest_block_number: None,
			blocks_total: 0,
			blocks_received: 0,
			num_peers: 0,
			num_active_peers: 0,
			mem_used: 0,
			num_snapshot_chunks: 0,
			snapshot_chunks_done: 0,
			last_imported_old_block_number: None,
		}
	}

	#[test]
	fn is_still_verifying() {
		assert!(!sync_status(SyncState::Idle).is_syncing(queue_info(2, 1)));
		assert!(sync_status(SyncState::Idle).is_syncing(queue_info(2, 2)));
	}

	#[test]
	fn is_synced_state() {
		assert!(sync_status(SyncState::Blocks).is_syncing(queue_info(0, 0)));
		assert!(!sync_status(SyncState::Idle).is_syncing(queue_info(0, 0)));
	}

	#[test]
	fn return_receipts_empty() {
		let mut client = TestBlockChainClient::new();
		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let io = TestIo::new(&mut client, &ss, &queue, None);

		let result = ChainSync::return_receipts(&io, &UntrustedRlp::new(&[0xc0]), 0);

		assert!(result.is_ok());
	}

	#[test]
	fn return_receipts() {
		let mut client = TestBlockChainClient::new();
		let queue = RwLock::new(VecDeque::new());
		let sync = dummy_sync_with_peer(H256::new(), &client);
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let mut receipt_list = RlpStream::new_list(4);
		receipt_list.append(&H256::from("0000000000000000000000000000000000000000000000005555555555555555"));
		receipt_list.append(&H256::from("ff00000000000000000000000000000000000000000000000000000000000000"));
		receipt_list.append(&H256::from("fff0000000000000000000000000000000000000000000000000000000000000"));
		receipt_list.append(&H256::from("aff0000000000000000000000000000000000000000000000000000000000000"));

		let receipts_request = receipt_list.out();
		// it returns rlp ONLY for hashes started with "f"
		let result = ChainSync::return_receipts(&io, &UntrustedRlp::new(&receipts_request.clone()), 0);

		assert!(result.is_ok());
		let rlp_result = result.unwrap();
		assert!(rlp_result.is_some());

		// the length of two rlp-encoded receipts
		assert_eq!(603, rlp_result.unwrap().1.out().len());

		io.sender = Some(2usize);
		ChainSync::dispatch_packet(&RwLock::new(sync), &mut io, 0usize, super::GET_RECEIPTS_PACKET, &receipts_request);
		assert_eq!(1, io.packets.len());
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
		let blocks: Vec<_> = (0 .. 100)
			.map(|i| (&client as &BlockChainClient).block(BlockId::Number(i as BlockNumber)).map(|b| b.into_inner()).unwrap()).collect();
		let headers: Vec<_> = blocks.iter().map(|b| Rlp::new(b).at(0).as_raw().to_vec()).collect();
		let hashes: Vec<_> = headers.iter().map(|h| HeaderView::new(h).sha3()).collect();

		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let io = TestIo::new(&mut client, &ss, &queue, None);

		let unknown: H256 = H256::new();
		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&unknown, 1, 0, false)), 0);
		assert!(to_header_vec(result).is_empty());
		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&unknown, 1, 0, true)), 0);
		assert!(to_header_vec(result).is_empty());

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&hashes[2], 1, 0, true)), 0);
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&hashes[2], 1, 0, false)), 0);
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&hashes[50], 3, 5, false)), 0);
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[56].clone(), headers[62].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_hash_req(&hashes[50], 3, 5, true)), 0);
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[44].clone(), headers[38].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_num_req(2, 1, 0, true)), 0);
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_num_req(2, 1, 0, false)), 0);
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_num_req(50, 3, 5, false)), 0);
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[56].clone(), headers[62].clone()]);

		let result = ChainSync::return_block_headers(&io, &UntrustedRlp::new(&make_num_req(50, 3, 5, true)), 0);
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[44].clone(), headers[38].clone()]);
	}

	#[test]
	fn return_nodes() {
		let mut client = TestBlockChainClient::new();
		let queue = RwLock::new(VecDeque::new());
		let sync = dummy_sync_with_peer(H256::new(), &client);
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let mut node_list = RlpStream::new_list(3);
		node_list.append(&H256::from("0000000000000000000000000000000000000000000000005555555555555555"));
		node_list.append(&H256::from("ffffffffffffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaa"));
		node_list.append(&H256::from("aff0000000000000000000000000000000000000000000000000000000000000"));

		let node_request = node_list.out();
		// it returns rlp ONLY for hashes started with "f"
		let result = ChainSync::return_node_data(&io, &UntrustedRlp::new(&node_request.clone()), 0);

		assert!(result.is_ok());
		let rlp_result = result.unwrap();
		assert!(rlp_result.is_some());

		// the length of one rlp-encoded hashe
		let rlp = rlp_result.unwrap().1.out();
		let rlp = Rlp::new(&rlp);
		assert_eq!(1, rlp.item_count());

		io.sender = Some(2usize);

		ChainSync::dispatch_packet(&RwLock::new(sync), &mut io, 0usize, super::GET_NODE_DATA_PACKET, &node_request);
		assert_eq!(1, io.packets.len());
	}

	fn dummy_sync_with_peer(peer_latest_hash: H256, client: &BlockChainClient) -> ChainSync {
		let mut sync = ChainSync::new(SyncConfig::default(), client);
		insert_dummy_peer(&mut sync, 0, peer_latest_hash);
		sync
	}

	fn insert_dummy_peer(sync: &mut ChainSync, peer_id: PeerId, peer_latest_hash: H256) {
		sync.peers.insert(peer_id,
			PeerInfo {
				protocol_version: 0,
				genesis: H256::zero(),
				network_id: 0,
				latest_hash: peer_latest_hash,
				difficulty: None,
				asking: PeerAsking::Nothing,
				asking_blocks: Vec::new(),
				asking_hash: None,
				ask_time: 0,
				last_sent_transactions: HashSet::new(),
				expired: false,
				confirmation: super::ForkConfirmation::Confirmed,
				snapshot_number: None,
				snapshot_hash: None,
				asking_snapshot_data: None,
				block_set: None,
			});

	}

	#[test]
	fn finds_lagging_peers() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(10), &client);
		let chain_info = client.chain_info();

		let lagging_peers = sync.get_lagging_peers(&chain_info);

		assert_eq!(1, lagging_peers.len());
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
		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let chain_info = client.chain_info();
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let peers = sync.get_lagging_peers(&chain_info);
		let peer_count = sync.propagate_new_hashes(&chain_info, &mut io, &peers);

		// 1 message should be send
		assert_eq!(1, io.packets.len());
		// 1 peer should be updated
		assert_eq!(1, peer_count);
		// NEW_BLOCK_HASHES_PACKET
		assert_eq!(0x01, io.packets[0].packet_id);
	}

	#[test]
	fn sends_latest_block_to_lagging_peer() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let chain_info = client.chain_info();
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);
		let peers = sync.get_lagging_peers(&chain_info);
		let peer_count = sync.propagate_blocks(&chain_info, &mut io, &[], &peers);

		// 1 message should be send
		assert_eq!(1, io.packets.len());
		// 1 peer should be updated
		assert_eq!(1, peer_count);
		// NEW_BLOCK_PACKET
		assert_eq!(0x07, io.packets[0].packet_id);
	}

	#[test]
	fn sends_sealed_block() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let queue = RwLock::new(VecDeque::new());
		let hash = client.block_hash(BlockId::Number(99)).unwrap();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let chain_info = client.chain_info();
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);
		let peers = sync.get_lagging_peers(&chain_info);
		let peer_count = sync.propagate_blocks(&chain_info, &mut io, &[hash.clone()], &peers);

		// 1 message should be send
		assert_eq!(1, io.packets.len());
		// 1 peer should be updated
		assert_eq!(1, peer_count);
		// NEW_BLOCK_PACKET
		assert_eq!(0x07, io.packets[0].packet_id);
	}

	#[test]
	fn sends_proposed_block() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(2, EachBlockWith::Uncle);
		let queue = RwLock::new(VecDeque::new());
		let block = client.block(BlockId::Latest).unwrap().into_inner();
		let mut sync = ChainSync::new(SyncConfig::default(), &client);
		sync.peers.insert(0,
			PeerInfo {
				// Messaging protocol
				protocol_version: 2,
				genesis: H256::zero(),
				network_id: 0,
				latest_hash: client.block_hash_delta_minus(1),
				difficulty: None,
				asking: PeerAsking::Nothing,
				asking_blocks: Vec::new(),
				asking_hash: None,
				ask_time: 0,
				last_sent_transactions: HashSet::new(),
				expired: false,
				confirmation: super::ForkConfirmation::Confirmed,
				snapshot_number: None,
				snapshot_hash: None,
				asking_snapshot_data: None,
				block_set: None,
			});
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);
		sync.propagate_proposed_blocks(&mut io, &[block]);

		// 1 message should be sent
		assert_eq!(1, io.packets.len());
		// NEW_BLOCK_PACKET
		assert_eq!(0x07, io.packets[0].packet_id);
	}

	#[test]
	fn propagates_transactions() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		client.insert_transaction_to_queue();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(1), &client);
		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);
		let peer_count = sync.propagate_new_transactions(&mut io);
		// Try to propagate same transactions for the second time
		let peer_count2 = sync.propagate_new_transactions(&mut io);
		// Even after new block transactions should not be propagated twice
		sync.chain_new_blocks(&mut io, &[], &[], &[], &[], &[], &[]);
		// Try to propagate same transactions for the third time
		let peer_count3 = sync.propagate_new_transactions(&mut io);

		// 1 message should be send
		assert_eq!(1, io.packets.len());
		// 1 peer should be updated but only once
		assert_eq!(1, peer_count);
		assert_eq!(0, peer_count2);
		assert_eq!(0, peer_count3);
		// TRANSACTIONS_PACKET
		assert_eq!(0x02, io.packets[0].packet_id);
	}

	#[test]
	fn does_not_propagate_new_transactions_after_new_block() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		client.insert_transaction_to_queue();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(1), &client);
		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);
		let peer_count = sync.propagate_new_transactions(&mut io);
		io.chain.insert_transaction_to_queue();
		// New block import should not trigger propagation.
		// (we only propagate on timeout)
		sync.chain_new_blocks(&mut io, &[], &[], &[], &[], &[], &[]);

		// 2 message should be send
		assert_eq!(1, io.packets.len());
		// 1 peer should receive the message
		assert_eq!(1, peer_count);
		// TRANSACTIONS_PACKET
		assert_eq!(0x02, io.packets[0].packet_id);
	}

	#[test]
	fn does_not_fail_for_no_peers() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		client.insert_transaction_to_queue();
		// Sync with no peers
		let mut sync = ChainSync::new(SyncConfig::default(), &client);
		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);
		let peer_count = sync.propagate_new_transactions(&mut io);
		sync.chain_new_blocks(&mut io, &[], &[], &[], &[], &[], &[]);
		// Try to propagate same transactions for the second time
		let peer_count2 = sync.propagate_new_transactions(&mut io);

		assert_eq!(0, io.packets.len());
		assert_eq!(0, peer_count);
		assert_eq!(0, peer_count2);
	}

	#[test]
	fn propagates_transactions_without_alternating() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		client.insert_transaction_to_queue();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(1), &client);
		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		// should sent some
		{
			let mut io = TestIo::new(&mut client, &ss, &queue, None);
			let peer_count = sync.propagate_new_transactions(&mut io);
			assert_eq!(1, io.packets.len());
			assert_eq!(1, peer_count);
		}
		// Insert some more
		client.insert_transaction_to_queue();
		let (peer_count2, peer_count3) = {
			let mut io = TestIo::new(&mut client, &ss, &queue, None);
			// Propagate new transactions
			let peer_count2 = sync.propagate_new_transactions(&mut io);
			// And now the peer should have all transactions
			let peer_count3 = sync.propagate_new_transactions(&mut io);
			(peer_count2, peer_count3)
		};

		// 2 message should be send (in total)
		assert_eq!(2, queue.read().len());
		// 1 peer should be updated but only once after inserting new transaction
		assert_eq!(1, peer_count2);
		assert_eq!(0, peer_count3);
		// TRANSACTIONS_PACKET
		assert_eq!(0x02, queue.read()[0].packet_id);
		assert_eq!(0x02, queue.read()[1].packet_id);
	}

	#[test]
	fn should_maintain_transations_propagation_stats() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		client.insert_transaction_to_queue();
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(1), &client);
		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);
		sync.propagate_new_transactions(&mut io);

		let stats = sync.transactions_stats();
		assert_eq!(stats.len(), 1, "Should maintain stats for single transaction.")
	}

	#[test]
	fn should_propagate_service_transaction_to_selected_peers_only() {
		let mut client = TestBlockChainClient::new();
		client.insert_transaction_with_gas_price_to_queue(U256::zero());
		let block_hash = client.block_hash_delta_minus(1);
		let mut sync = ChainSync::new(SyncConfig::default(), &client);
		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		// when peer#1 is Geth
		insert_dummy_peer(&mut sync, 1, block_hash);
		io.peers_info.insert(1, "Geth".to_owned());
		// and peer#2 is Parity, accepting service transactions
		insert_dummy_peer(&mut sync, 2, block_hash);
		io.peers_info.insert(2, "Parity/v1.6".to_owned());
		// and peer#3 is Parity, discarding service transactions
		insert_dummy_peer(&mut sync, 3, block_hash);
		io.peers_info.insert(3, "Parity/v1.5".to_owned());
		// and peer#4 is Parity, accepting service transactions
		insert_dummy_peer(&mut sync, 4, block_hash);
		io.peers_info.insert(4, "Parity/v1.7.3-ABCDEFGH".to_owned());

		// and new service transaction is propagated to peers
		sync.propagate_new_transactions(&mut io);

		// peer#2 && peer#4 are receiving service transaction
		assert!(io.packets.iter().any(|p| p.packet_id == 0x02 && p.recipient == 2)); // TRANSACTIONS_PACKET
		assert!(io.packets.iter().any(|p| p.packet_id == 0x02 && p.recipient == 4)); // TRANSACTIONS_PACKET
		assert_eq!(io.packets.len(), 2);
	}

	#[test]
	fn should_propagate_service_transaction_is_sent_as_separate_message() {
		let mut client = TestBlockChainClient::new();
		let tx1_hash = client.insert_transaction_to_queue();
		let tx2_hash = client.insert_transaction_with_gas_price_to_queue(U256::zero());
		let block_hash = client.block_hash_delta_minus(1);
		let mut sync = ChainSync::new(SyncConfig::default(), &client);
		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		// when peer#1 is Parity, accepting service transactions
		insert_dummy_peer(&mut sync, 1, block_hash);
		io.peers_info.insert(1, "Parity/v1.6".to_owned());

		// and service + non-service transactions are propagated to peers
		sync.propagate_new_transactions(&mut io);

		// two separate packets for peer are queued:
		// 1) with non-service-transaction
		// 2) with service transaction
		let sent_transactions: Vec<UnverifiedTransaction> = io.packets.iter()
			.filter_map(|p| {
				if p.packet_id != 0x02 || p.recipient != 1 { // TRANSACTIONS_PACKET
					return None;
				}

				let rlp = UntrustedRlp::new(&*p.data);
				let item_count = rlp.item_count().unwrap_or(0);
				if item_count != 1 {
					return None;
				}

				rlp.at(0).ok().and_then(|r| r.as_val().ok())
			})
			.collect();
		assert_eq!(sent_transactions.len(), 2);
		assert!(sent_transactions.iter().any(|tx| tx.hash() == tx1_hash));
		assert!(sent_transactions.iter().any(|tx| tx.hash() == tx2_hash));
	}

	#[test]
	fn handles_peer_new_block_malformed() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);

		let block_data = get_dummy_block(11, client.chain_info().best_block_hash);

		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		//sync.have_common_block = true;
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let block = UntrustedRlp::new(&block_data);

		let result = sync.on_peer_new_block(&mut io, 0, &block);

		assert!(result.is_err());
	}

	#[test]
	fn handles_peer_new_block() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);

		let block_data = get_dummy_blocks(11, client.chain_info().best_block_hash);

		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let block = UntrustedRlp::new(&block_data);

		let result = sync.on_peer_new_block(&mut io, 0, &block);

		assert!(result.is_ok());
	}

	#[test]
	fn handles_peer_new_block_empty() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);
		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let empty_data = vec![];
		let block = UntrustedRlp::new(&empty_data);

		let result = sync.on_peer_new_block(&mut io, 0, &block);

		assert!(result.is_err());
	}

	#[test]
	fn handles_peer_new_hashes() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);
		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let hashes_data = get_dummy_hashes();
		let hashes_rlp = UntrustedRlp::new(&hashes_data);

		let result = sync.on_peer_new_hashes(&mut io, 0, &hashes_rlp);

		assert!(result.is_ok());
	}

	#[test]
	fn handles_peer_new_hashes_empty() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(10, EachBlockWith::Uncle);
		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

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
		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let chain_info = client.chain_info();
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let peers = sync.get_lagging_peers(&chain_info);
		sync.propagate_new_hashes(&chain_info, &mut io, &peers);

		let data = &io.packets[0].data.clone();
		let result = sync.on_peer_new_hashes(&mut io, 0, &UntrustedRlp::new(data));
		assert!(result.is_ok());
	}

	// idea is that what we produce when propagading latest block should be accepted in
	// on_peer_new_block  in our code as well
	#[test]
	fn block_rlp_mutually_acceptable() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let queue = RwLock::new(VecDeque::new());
		let mut sync = dummy_sync_with_peer(client.block_hash_delta_minus(5), &client);
		let chain_info = client.chain_info();
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		let peers = sync.get_lagging_peers(&chain_info);
		sync.propagate_blocks(&chain_info, &mut io, &[], &peers);

		let data = &io.packets[0].data.clone();
		let result = sync.on_peer_new_block(&mut io, 0, &UntrustedRlp::new(data));
		assert!(result.is_ok());
	}

	#[test]
	fn should_add_transactions_to_queue() {
		fn sender(tx: &UnverifiedTransaction) -> Address {
			ethkey::public_to_address(&tx.recover_public().unwrap())
		}

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
			let block = client.block(BlockId::Hash(*h)).unwrap();
			let sender = sender(&block.transactions()[0]);;
			client.set_balance(sender, U256::from(10_000_000_000_000_000_000u64));
			client.set_nonce(sender, U256::from(0));
		}


		// when
		{
			let queue = RwLock::new(VecDeque::new());
			let ss = TestSnapshotService::new();
			let mut io = TestIo::new(&mut client, &ss, &queue, None);
			io.chain.miner.chain_new_blocks(io.chain, &[], &[], &[], &good_blocks);
			sync.chain_new_blocks(&mut io, &[], &[], &[], &good_blocks, &[], &[]);
			assert_eq!(io.chain.miner.status().transactions_in_future_queue, 0);
			assert_eq!(io.chain.miner.status().transactions_in_pending_queue, 1);
		}
		// We need to update nonce status (because we say that the block has been imported)
		for h in &[good_blocks[0]] {
			let block = client.block(BlockId::Hash(*h)).unwrap();
			client.set_nonce(sender(&block.transactions()[0]), U256::from(1));
		}
		{
			let queue = RwLock::new(VecDeque::new());
			let ss = TestSnapshotService::new();
			let mut io = TestIo::new(&client, &ss, &queue, None);
			io.chain.miner.chain_new_blocks(io.chain, &[], &[], &good_blocks, &retracted_blocks);
			sync.chain_new_blocks(&mut io, &[], &[], &good_blocks, &retracted_blocks, &[], &[]);
		}

		// then
		let status = client.miner.status();
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

		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let mut io = TestIo::new(&mut client, &ss, &queue, None);

		// when
		sync.chain_new_blocks(&mut io, &[], &[], &[], &good_blocks, &[], &[]);
		assert_eq!(io.chain.miner.status().transactions_in_future_queue, 0);
		assert_eq!(io.chain.miner.status().transactions_in_pending_queue, 0);
		sync.chain_new_blocks(&mut io, &[], &[], &good_blocks, &retracted_blocks, &[], &[]);

		// then
		let status = io.chain.miner.status();
		assert_eq!(status.transactions_in_pending_queue, 0);
		assert_eq!(status.transactions_in_future_queue, 0);
	}
}
