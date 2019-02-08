// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! `BlockChain` synchronization strategy.
//! Syncs to peers and keeps up to date.
//! This implementation uses ethereum protocol v63
//!
//! Syncing strategy summary.
//! Split the chain into ranges of N blocks each. Download ranges sequentially. Split each range into subchains of M blocks. Download subchains in parallel.
//! State.
//! Sync state consists of the following data:
//! - s: State enum which can be one of the following values: `ChainHead`, `Blocks`, `Idle`
//! - H: A set of downloaded block headers
//! - B: A set of downloaded block bodies
//! - S: Set of block subchain start block hashes to download.
//! - l: Last imported / common block hash
//! - P: A set of connected peers. For each peer we maintain its last known total difficulty and starting block hash being requested if any.
//! General behaviour.
//! We start with all sets empty, l is set to the best block in the block chain, s is set to `ChainHead`.
//! If at any moment a bad block is reported by the block queue, we set s to `ChainHead`, reset l to the best block in the block chain and clear H, B and S.
//! If at any moment P becomes empty, we set s to `ChainHead`, and clear H, B and S.
//!
//! Workflow for `ChainHead` state.
//! In this state we try to get subchain headers with a single `GetBlockHeaders` request.
//! On `NewPeer` / On `Restart`:
//! 	If peer's total difficulty is higher and there are less than 5 peers downloading, request N/M headers with interval M+1 starting from l
//! On `BlockHeaders(R)`:
//! 	If R is empty:
//! If l is equal to genesis block hash or l is more than 1000 blocks behind our best hash:
//! Remove current peer from P. set l to the best block in the block chain. Select peer with maximum total difficulty from P and restart.
//! Else
//! 	Set l to l’s parent and restart.
//! Else if we already have all the headers in the block chain or the block queue:
//! 	Set s to `Idle`,
//! Else
//! 	Set S to R, set s to `Blocks`.
//!
//! All other messages are ignored.
//!
//! Workflow for `Blocks` state.
//! In this state we download block headers and bodies from multiple peers.
//! On `NewPeer` / On `Restart`:
//! 	For all idle peers:
//! Find a set of 256 or less block hashes in H which are not in B and not being downloaded by other peers. If the set is not empty:
//!  	Request block bodies for the hashes in the set.
//! Else
//! 	Find an element in S which is  not being downloaded by other peers. If found: Request M headers starting from the element.
//!
//! On `BlockHeaders(R)`:
//! If R is empty remove current peer from P and restart.
//! 	Validate received headers:
//! 		For each header find a parent in H or R or the blockchain. Restart if there is a block with unknown parent.
//! 		Find at least one header from the received list in S. Restart if there is none.
//! Go to `CollectBlocks`.
//!
//! On `BlockBodies(R)`:
//! If R is empty remove current peer from P and restart.
//! 	Add bodies with a matching header in H to B.
//! 	Go to `CollectBlocks`.
//!
//! `CollectBlocks`:
//! Find a chain of blocks C in H starting from h where h’s parent equals to l. The chain ends with the first block which does not have a body in B.
//! Add all blocks from the chain to the block queue. Remove them from H and B. Set l to the hash of the last block from C.
//! Update and merge subchain heads in S. For each h in S find a chain of blocks in B starting from h. Remove h from S. if the chain does not include an element from S add the end of the chain to S.
//! If H is empty and S contains a single element set s to `ChainHead`.
//! Restart.
//!
//! All other messages are ignored.
//! Workflow for Idle state.
//! On `NewBlock`:
//! 	Import the block. If the block is unknown set s to `ChainHead` and restart.
//! On `NewHashes`:
//! 	Set s to `ChainHead` and restart.
//!
//! All other messages are ignored.

mod handler;
pub mod sync_packet;
mod propagator;
mod requester;
mod supplier;

use std::sync::{Arc, mpsc};
use std::collections::{HashSet, HashMap, BTreeMap};
use std::cmp;
use std::time::{Duration, Instant};
use hash::keccak;
use heapsize::HeapSizeOf;
use futures::sync::mpsc as futures_mpsc;
use api::Notification;
use ethereum_types::{H256, U256};
use fastmap::{H256FastMap, H256FastSet};
use parking_lot::{Mutex, RwLock, RwLockWriteGuard};
use bytes::Bytes;
use rlp::{RlpStream, DecoderError};
use network::{self, PeerId, PacketId};
use network::client_version::ClientVersion;
use ethcore::client::{BlockChainClient, BlockStatus, BlockId, BlockChainInfo, BlockQueueInfo};
use ethcore::snapshot::{RestorationStatus};
use sync_io::SyncIo;
use super::{WarpSync, SyncConfig};
use block_sync::{BlockDownloader, DownloadAction};
use rand::Rng;
use snapshot::{Snapshot};
use api::{EthProtocolInfo as PeerInfoDigest, WARP_SYNC_PROTOCOL_ID, PriorityTask};
use private_tx::PrivateTxHandler;
use transactions_stats::{TransactionsStats, Stats as TransactionStats};
use types::transaction::UnverifiedTransaction;
use types::BlockNumber;

use self::handler::SyncHandler;
use self::sync_packet::{PacketInfo, SyncPacket};
use self::sync_packet::SyncPacket::{
	NewBlockPacket,
	StatusPacket,
};

use self::propagator::SyncPropagator;
use self::requester::SyncRequester;
pub(crate) use self::supplier::SyncSupplier;

known_heap_size!(0, PeerInfo);

pub type PacketDecodeError = DecoderError;

/// 63 version of Ethereum protocol.
pub const ETH_PROTOCOL_VERSION_63: (u8, u8) = (63, 0x11);
/// 62 version of Ethereum protocol.
pub const ETH_PROTOCOL_VERSION_62: (u8, u8) = (62, 0x11);
/// 1 version of Parity protocol and the packet count.
pub const PAR_PROTOCOL_VERSION_1: (u8, u8) = (1, 0x15);
/// 2 version of Parity protocol (consensus messages added).
pub const PAR_PROTOCOL_VERSION_2: (u8, u8) = (2, 0x16);
/// 3 version of Parity protocol (private transactions messages added).
pub const PAR_PROTOCOL_VERSION_3: (u8, u8) = (3, 0x18);

pub const MAX_BODIES_TO_SEND: usize = 256;
pub const MAX_HEADERS_TO_SEND: usize = 512;
pub const MAX_NODE_DATA_TO_SEND: usize = 1024;
pub const MAX_RECEIPTS_HEADERS_TO_SEND: usize = 256;
const MIN_PEERS_PROPAGATION: usize = 4;
const MAX_PEERS_PROPAGATION: usize = 128;
const MAX_PEER_LAG_PROPAGATION: BlockNumber = 20;
const MAX_NEW_HASHES: usize = 64;
const MAX_NEW_BLOCK_AGE: BlockNumber = 20;
// maximal packet size with transactions (cannot be greater than 16MB - protocol limitation).
// keep it under 8MB as well, cause it seems that it may result oversized after compression.
const MAX_TRANSACTION_PACKET_SIZE: usize = 5 * 1024 * 1024;
// Min number of blocks to be behind for a snapshot sync
const SNAPSHOT_RESTORE_THRESHOLD: BlockNumber = 30000;
const SNAPSHOT_MIN_PEERS: usize = 3;

const MAX_SNAPSHOT_CHUNKS_DOWNLOAD_AHEAD: usize = 3;

const WAIT_PEERS_TIMEOUT: Duration = Duration::from_secs(5);
const STATUS_TIMEOUT: Duration = Duration::from_secs(5);
const HEADERS_TIMEOUT: Duration = Duration::from_secs(15);
const BODIES_TIMEOUT: Duration = Duration::from_secs(20);
const RECEIPTS_TIMEOUT: Duration = Duration::from_secs(10);
const FORK_HEADER_TIMEOUT: Duration = Duration::from_secs(3);
const SNAPSHOT_MANIFEST_TIMEOUT: Duration = Duration::from_secs(5);
const SNAPSHOT_DATA_TIMEOUT: Duration = Duration::from_secs(120);

/// Defines how much time we have to complete priority transaction or block propagation.
/// after the deadline is reached the task is considered finished
/// (so we might sent only to some part of the peers we originally intended to send to)
const PRIORITY_TASK_DEADLINE: Duration = Duration::from_millis(100);

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
		match self.state {
			SyncState::SnapshotManifest |
				SyncState::SnapshotData |
				SyncState::SnapshotWaiting => true,
			_ => false,
		}
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
pub enum PeerAsking {
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
pub enum BlockSet {
	/// New blocks better than out best blocks
	NewBlocks,
	/// Missing old blocks
	OldBlocks,
}
#[derive(Clone, Eq, PartialEq)]
pub enum ForkConfirmation {
	/// Fork block confirmation pending.
	Unconfirmed,
	/// Peer's chain is too short to confirm the fork.
	TooShort,
	/// Fork is confirmed.
	Confirmed,
}

#[derive(Clone)]
/// Syncing peer information
pub struct PeerInfo {
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
	ask_time: Instant,
	/// Holds a set of transactions recently sent to this peer to avoid spamming.
	last_sent_transactions: H256FastSet,
	/// Holds a set of private transactions and their signatures recently sent to this peer to avoid spamming.
	last_sent_private_transactions: H256FastSet,
	/// Pending request is expired and result should be ignored
	expired: bool,
	/// Private transactions enabled
	private_tx_enabled: bool,
	/// Peer fork confirmation status
	confirmation: ForkConfirmation,
	/// Best snapshot hash
	snapshot_hash: Option<H256>,
	/// Best snapshot block number
	snapshot_number: Option<BlockNumber>,
	/// Block set requested
	block_set: Option<BlockSet>,
	/// Version of the software the peer is running
	client_version: ClientVersion,
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

	fn reset_private_stats(&mut self) {
		self.last_sent_private_transactions.clear();
	}
}

#[cfg(not(test))]
pub mod random {
	use rand;
	pub fn new() -> rand::ThreadRng { rand::thread_rng() }
}
#[cfg(test)]
pub mod random {
	use rand::{self, SeedableRng};
	pub fn new() -> rand::XorShiftRng { rand::XorShiftRng::from_seed([0, 1, 2, 3]) }
}

pub type RlpResponseResult = Result<Option<(PacketId, RlpStream)>, PacketDecodeError>;
pub type Peers = HashMap<PeerId, PeerInfo>;

/// Thread-safe wrapper for `ChainSync`.
///
/// NOTE always lock in order of fields declaration
pub struct ChainSyncApi {
	/// Priority tasks queue
	priority_tasks: Mutex<mpsc::Receiver<PriorityTask>>,
	/// The rest of sync data
	sync: RwLock<ChainSync>,
}

impl ChainSyncApi {
	/// Creates new `ChainSyncApi`
	pub fn new(
		config: SyncConfig,
		chain: &BlockChainClient,
		private_tx_handler: Option<Arc<PrivateTxHandler>>,
		priority_tasks: mpsc::Receiver<PriorityTask>,
	) -> Self {
		ChainSyncApi {
			sync: RwLock::new(ChainSync::new(config, chain, private_tx_handler)),
			priority_tasks: Mutex::new(priority_tasks),
		}
	}

	/// Gives `write` access to underlying `ChainSync`
	pub fn write(&self) -> RwLockWriteGuard<ChainSync> {
		self.sync.write()
	}

	/// Returns info about given list of peers
	pub fn peer_info(&self, ids: &[PeerId]) -> Vec<Option<PeerInfoDigest>> {
		let sync = self.sync.read();
		ids.iter().map(|id| sync.peer_info(id)).collect()
	}

	/// Returns synchonization status
	pub fn status(&self) -> SyncStatus {
		self.sync.read().status()
	}

	/// Returns transactions propagation statistics
	pub fn transactions_stats(&self) -> BTreeMap<H256, ::TransactionStats> {
		self.sync.read().transactions_stats()
			.iter()
			.map(|(hash, stats)| (*hash, stats.into()))
			.collect()
	}

	/// Dispatch incoming requests and responses
	pub fn dispatch_packet(&self, io: &mut SyncIo, peer: PeerId, packet_id: u8, data: &[u8]) {
		SyncSupplier::dispatch_packet(&self.sync, io, peer, packet_id, data)
	}

	/// Process a priority propagation queue.
	/// This task is run from a timer and should be time constrained.
	/// Hence we set up a deadline for the execution and cancel the task if the deadline is exceeded.
	///
	/// NOTE This method should only handle stuff that can be canceled and would reach other peers
	/// by other means.
	pub fn process_priority_queue(&self, io: &mut SyncIo) {
		fn check_deadline(deadline: Instant) -> Option<Duration> {
			let now = Instant::now();
			if now > deadline {
				None
			} else {
				Some(deadline - now)
			}
		}

		// deadline to get the task from the queue
		let deadline = Instant::now() + ::api::PRIORITY_TIMER_INTERVAL;
		let mut work = || {
			let task = {
				let tasks = self.priority_tasks.try_lock_until(deadline)?;
				let left = check_deadline(deadline)?;
				tasks.recv_timeout(left).ok()?
			};
			task.starting();
			// wait for the sync lock until deadline,
			// note we might drop the task here if we won't manage to acquire the lock.
			let mut sync = self.sync.try_write_until(deadline)?;
			// since we already have everything let's use a different deadline
			// to do the rest of the job now, so that previous work is not wasted.
			let deadline = Instant::now() + PRIORITY_TASK_DEADLINE;
			let as_ms = move |prev| {
				let dur: Duration = Instant::now() - prev;
				dur.as_secs() * 1_000 + dur.subsec_millis() as u64
			};
			match task {
				// NOTE We can't simply use existing methods,
				// cause the block is not in the DB yet.
				PriorityTask::PropagateBlock { started, block, hash, difficulty } => {
					// try to send to peers that are on the same block as us
					// (they will most likely accept the new block).
					let chain_info = io.chain().chain_info();
					let total_difficulty = chain_info.total_difficulty + difficulty;
					let rlp = ChainSync::create_block_rlp(&block, total_difficulty);
					for peers in sync.get_peers(&chain_info, PeerState::SameBlock).chunks(10) {
						check_deadline(deadline)?;
						for peer in peers {
							SyncPropagator::send_packet(io, *peer, NewBlockPacket, rlp.clone());
							if let Some(ref mut peer) = sync.peers.get_mut(peer) {
								peer.latest_hash = hash;
							}
						}
					}
					debug!(target: "sync", "Finished block propagation, took {}ms", as_ms(started));
				},
				PriorityTask::PropagateTransactions(time, _) => {
					SyncPropagator::propagate_new_transactions(&mut sync, io, || {
						check_deadline(deadline).is_some()
					});
					debug!(target: "sync", "Finished transaction propagation, took {}ms", as_ms(time));
				},
			}

			Some(())
		};

		// Process as many items as we can until the deadline is reached.
		loop {
			if work().is_none() {
				return;
			}
		}
	}
}

// Static methods
impl ChainSync {
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
		Self::create_block_rlp(
			&chain.block(BlockId::Hash(chain.chain_info().best_block_hash))
				.expect("Best block always exists").into_inner(),
			chain.chain_info().total_difficulty
		)
	}

	/// creates given hash block rlp for the given client
	fn create_new_block_rlp(chain: &BlockChainClient, hash: &H256) -> Bytes {
		Self::create_block_rlp(
			&chain.block(BlockId::Hash(hash.clone())).expect("Block has just been sealed; qed").into_inner(),
			chain.block_total_difficulty(BlockId::Hash(hash.clone())).expect("Block has just been sealed; qed.")
		)
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

	fn get_init_state(warp_sync: WarpSync, chain: &BlockChainClient) -> SyncState {
		let best_block = chain.chain_info().best_block_number;
		match warp_sync {
			WarpSync::Enabled => SyncState::WaitingPeers,
			WarpSync::OnlyAndAfter(block) if block > best_block => SyncState::WaitingPeers,
			_ => SyncState::Idle,
		}
	}
}

/// A peer query method for getting a list of peers
enum PeerState {
	/// Peer is on different hash than us
	Lagging,
	/// Peer is on the same block as us
	SameBlock
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
	peers: Peers,
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
	handshaking_peers: HashMap<PeerId, Instant>,
	/// Sync start timestamp. Measured when first peer is connected
	sync_start_time: Option<Instant>,
	/// Transactions propagation statistics
	transactions_stats: TransactionsStats,
	/// Enable ancient block downloading
	download_old_blocks: bool,
	/// Shared private tx service.
	private_tx_handler: Option<Arc<PrivateTxHandler>>,
	/// Enable warp sync.
	warp_sync: WarpSync,

	status_sinks: Vec<futures_mpsc::UnboundedSender<SyncState>>
}

impl ChainSync {
	/// Create a new instance of syncing strategy.
	pub fn new(
		config: SyncConfig,
		chain: &BlockChainClient,
		private_tx_handler: Option<Arc<PrivateTxHandler>>,
	) -> Self {
		let chain_info = chain.chain_info();
		let best_block = chain.chain_info().best_block_number;
		let state = Self::get_init_state(config.warp_sync, chain);

		let mut sync = ChainSync {
			state,
			starting_block: best_block,
			highest_block: None,
			peers: HashMap::new(),
			handshaking_peers: HashMap::new(),
			active_peers: HashSet::new(),
			new_blocks: BlockDownloader::new(BlockSet::NewBlocks, &chain_info.best_block_hash, chain_info.best_block_number),
			old_blocks: None,
			last_sent_block_number: 0,
			network_id: config.network_id,
			fork_block: config.fork_block,
			download_old_blocks: config.download_old_blocks,
			snapshot: Snapshot::new(),
			sync_start_time: None,
			transactions_stats: TransactionsStats::default(),
			private_tx_handler,
			warp_sync: config.warp_sync,
			status_sinks: Vec::new()
		};
		sync.update_targets(chain);
		sync
	}

	/// Returns synchonization status
	pub fn status(&self) -> SyncStatus {
		let last_imported_number = self.new_blocks.last_imported_block_number();
		SyncStatus {
			state: self.state.clone(),
			protocol_version: ETH_PROTOCOL_VERSION_63.0,
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
	pub fn transactions_received(&mut self, txs: &[UnverifiedTransaction], peer_id: PeerId) {
		if let Some(peer_info) = self.peers.get_mut(&peer_id) {
			peer_info.last_sent_transactions.extend(txs.iter().map(|tx| tx.hash()));
		}
	}

	/// Abort all sync activity
	pub fn abort(&mut self, io: &mut SyncIo) {
		self.reset_and_continue(io);
		self.peers.clear();
	}

	/// returns the receiving end of a future::mpsc channel that can
	/// be polled for changes to node's SyncState.
	pub fn sync_notifications(&mut self) -> Notification<SyncState> {
		let (sender, receiver) = futures_mpsc::unbounded();
		self.status_sinks.push(sender);
		receiver
	}

	/// notify all subscibers of a new SyncState
	fn notify_sync_state(&mut self, state: SyncState) {
		// remove any sender whose receiving end has been dropped
		self.status_sinks.retain(|sender| {
			sender.unbounded_send(state).is_ok()
		});
	}

	/// sets a new SyncState
	fn set_state(&mut self, state: SyncState) {
		self.notify_sync_state(state);

		self.state = state;
	}

	/// Reset sync. Clear all downloaded data but keep the queue.
	/// Set sync state to the given state or to the initial state if `None` is provided.
	fn reset(&mut self, io: &mut SyncIo, state: Option<SyncState>) {
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

		let warp_sync = self.warp_sync;

		self.set_state(state.unwrap_or_else(|| Self::get_init_state(warp_sync, io.chain())));
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
		self.reset(io, None);
		self.continue_sync(io);
	}

	/// Remove peer from active peer set. Peer will be reactivated on the next sync
	/// round.
	fn deactivate_peer(&mut self, _io: &mut SyncIo, peer_id: PeerId) {
		trace!(target: "sync", "Deactivating peer {}", peer_id);
		self.active_peers.remove(&peer_id);
	}

	fn maybe_start_snapshot_sync(&mut self, io: &mut SyncIo) {
		if !self.warp_sync.is_enabled() || io.snapshot_service().supported_versions().is_none() {
			trace!(target: "sync", "Skipping warp sync. Disabled or not supported.");
			return;
		}
		if self.state != SyncState::WaitingPeers && self.state != SyncState::Blocks && self.state != SyncState::Waiting {
			trace!(target: "sync", "Skipping warp sync. State: {:?}", self.state);
			return;
		}
		// Make sure the snapshot block is not too far away from best block and network best block and
		// that it is higher than fork detection block
		let our_best_block = io.chain().chain_info().best_block_number;
		let fork_block = self.fork_block.map_or(0, |(n, _)| n);

		let (best_hash, max_peers, snapshot_peers) = {
			let expected_warp_block = match self.warp_sync {
				WarpSync::OnlyAndAfter(block) => block,
				_ => 0,
			};
			//collect snapshot infos from peers
			let snapshots = self.peers.iter()
				.filter(|&(_, p)| p.is_allowed() && p.snapshot_number.map_or(false, |sn|
					// Snapshot must be old enough that it's usefull to sync with it
					our_best_block < sn && (sn - our_best_block) > SNAPSHOT_RESTORE_THRESHOLD &&
					// Snapshot must have been taken after the Fork
					sn > fork_block &&
					// Snapshot must be greater than the warp barrier if any
					sn > expected_warp_block &&
					// If we know a highest block, snapshot must be recent enough
					self.highest_block.map_or(true, |highest| {
						highest < sn || (highest - sn) <= SNAPSHOT_RESTORE_THRESHOLD
					})
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

		let timeout = (self.state == SyncState::WaitingPeers) && self.sync_start_time.map_or(false, |t| t.elapsed() > WAIT_PEERS_TIMEOUT);

		if let (Some(hash), Some(peers)) = (best_hash, best_hash.map_or(None, |h| snapshot_peers.get(&h))) {
			if max_peers >= SNAPSHOT_MIN_PEERS {
				trace!(target: "sync", "Starting confirmed snapshot sync {:?} with {:?}", hash, peers);
				self.start_snapshot_sync(io, peers);
			} else if timeout {
				trace!(target: "sync", "Starting unconfirmed snapshot sync {:?} with {:?}", hash, peers);
				self.start_snapshot_sync(io, peers);
			}
		} else if timeout && !self.warp_sync.is_warp_only() {
			trace!(target: "sync", "No snapshots found, starting full sync");
			self.set_state(SyncState::Idle);
			self.continue_sync(io);
		}
	}

	fn start_snapshot_sync(&mut self, io: &mut SyncIo, peers: &[PeerId]) {
		if !self.snapshot.have_manifest() {
			for p in peers {
				if self.peers.get(p).map_or(false, |p| p.asking == PeerAsking::Nothing) {
					SyncRequester::request_snapshot_manifest(self, io, *p);
				}
			}
			self.set_state(SyncState::SnapshotManifest);
			trace!(target: "sync", "New snapshot sync with {:?}", peers);
		} else {
			self.set_state(SyncState::SnapshotData);
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
		self.new_blocks = BlockDownloader::new(BlockSet::NewBlocks, &chain.best_block_hash, chain.best_block_number);
		self.old_blocks = None;
		if self.download_old_blocks {
			if let (Some(ancient_block_hash), Some(ancient_block_number)) = (chain.ancient_block_hash, chain.ancient_block_number) {

				trace!(target: "sync", "Downloading old blocks from {:?} (#{}) till {:?} (#{:?})", ancient_block_hash, ancient_block_number, chain.first_block_hash, chain.first_block_number);
				let mut downloader = BlockDownloader::new(BlockSet::OldBlocks, &ancient_block_hash, ancient_block_number);
				if let Some(hash) = chain.first_block_hash {
					trace!(target: "sync", "Downloader target set to {:?}", hash);
					downloader.set_target(&hash);
				}
				self.old_blocks = Some(downloader);
			}
		}
	}

	/// Resume downloading
	pub fn continue_sync(&mut self, io: &mut SyncIo) {
		if self.state == SyncState::Waiting {
			trace!(target: "sync", "Waiting for the block queue");
		} else if self.state == SyncState::SnapshotWaiting {
			trace!(target: "sync", "Waiting for the snapshot restoration");
		} else {
			// Collect active peers that can sync
			let mut peers: Vec<(PeerId, u8)> = self.peers.iter().filter_map(|(peer_id, peer)|
				if peer.can_sync() && peer.asking == PeerAsking::Nothing && self.active_peers.contains(&peer_id) {
					Some((*peer_id, peer.protocol_version))
				} else {
					None
				}
			).collect();

			if peers.len() > 0 {
				trace!(
					target: "sync",
					"Syncing with peers: {} active, {} available, {} total",
					self.active_peers.len(), peers.len(), self.peers.len()
				);

				random::new().shuffle(&mut peers); // TODO (#646): sort by rating
				// prefer peers with higher protocol version
				peers.sort_by(|&(_, ref v1), &(_, ref v2)| v1.cmp(v2));

				for (peer_id, _) in peers {
					self.sync_peer(io, peer_id, false);
				}
			}
		}

		if
			(self.state == SyncState::Blocks || self.state == SyncState::NewBlocks) &&
			!self.peers.values().any(|p| p.asking != PeerAsking::Nothing && p.block_set != Some(BlockSet::OldBlocks) && p.can_sync())
		{
			self.complete_sync(io);
		}
	}

	/// Called after all blocks have been downloaded
	fn complete_sync(&mut self, io: &mut SyncIo) {
		trace!(target: "sync", "Sync complete");
		self.reset(io, Some(SyncState::Idle));
	}

	/// Enter waiting state
	fn pause_sync(&mut self) {
		trace!(target: "sync", "Block queue full, pausing sync");
		self.set_state(SyncState::Waiting);
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
					trace!(
						target: "sync",
						"Checking snapshot sync: {} vs {} (peer: {})",
						peer_snapshot_number,
						chain_info.best_block_number,
						peer_id
					);
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
						if let Some(request) = self.new_blocks.request_blocks(peer_id, io, num_active_peers) {
							SyncRequester::request_blocks(self, io, peer_id, request, BlockSet::NewBlocks);
							if self.state == SyncState::Idle {
								self.set_state(SyncState::Blocks);
							}
							return;
						}
					}

					// Only ask for old blocks if the peer has an equal or higher difficulty
					let equal_or_higher_difficulty = peer_difficulty.map_or(false, |pd| pd >= syncing_difficulty);

					if force || equal_or_higher_difficulty {
						if let Some(request) = self.old_blocks.as_mut().and_then(|d| d.request_blocks(peer_id, io, num_active_peers)) {
							SyncRequester::request_blocks(self, io, peer_id, request, BlockSet::OldBlocks);
							return;
						}
					} else {
						trace!(
							target: "sync",
							"peer {:?} is not suitable for requesting old blocks, syncing_difficulty={:?}, peer_difficulty={:?}",
							peer_id,
							syncing_difficulty,
							peer_difficulty
						);
						self.deactivate_peer(io, peer_id);
					}
				},
				SyncState::SnapshotData => {
					match io.snapshot_service().status() {
						RestorationStatus::Ongoing { state_chunks_done, block_chunks_done, .. } => {
							// Initialize the snapshot if not already done
							self.snapshot.initialize(io.snapshot_service());
							if self.snapshot.done_chunks() - (state_chunks_done + block_chunks_done) as usize > MAX_SNAPSHOT_CHUNKS_DOWNLOAD_AHEAD {
								trace!(target: "sync", "Snapshot queue full, pausing sync");
								self.set_state(SyncState::SnapshotWaiting);
								return;
							}
						},
						RestorationStatus::Initializing { .. } => {
							trace!(target: "warp", "Snapshot is stil initializing.");
							return;
						},
						_ => {
							return;
						},
					}

					if peer_snapshot_hash.is_some() && peer_snapshot_hash == self.snapshot.snapshot_hash() {
						self.clear_peer_download(peer_id);
						SyncRequester::request_snapshot_data(self, io, peer_id);
					}
				},
				SyncState::SnapshotManifest | //already downloading from other peer
					SyncState::Waiting |
					SyncState::SnapshotWaiting => ()
			}
		} else {
			trace!(target: "sync", "Skipping peer {}, force={}, td={:?}, our td={}, state={:?}", peer_id, force, peer_difficulty, syncing_difficulty, self.state);
		}
	}

	/// Clear all blocks/headers marked as being downloaded by a peer.
	fn clear_peer_download(&mut self, peer_id: PeerId) {
		if let Some(ref peer) = self.peers.get(&peer_id) {
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
	fn collect_blocks(&mut self, io: &mut SyncIo, block_set: BlockSet) {
		match block_set {
			BlockSet::NewBlocks => {
				if self.new_blocks.collect_blocks(io, self.state == SyncState::NewBlocks) == DownloadAction::Reset {
					self.reset_downloads(block_set);
					self.new_blocks.reset();
				}
			},
			BlockSet::OldBlocks => {
				let mut is_complete = false;
				let mut download_action = DownloadAction::None;
				if let Some(downloader) = self.old_blocks.as_mut() {
					download_action = downloader.collect_blocks(io, false);
					is_complete = downloader.is_complete();
				}

				if download_action == DownloadAction::Reset {
					self.reset_downloads(block_set);
					if let Some(downloader) = self.old_blocks.as_mut() {
						downloader.reset();
					}
				}

				if is_complete {
					trace!(target: "sync", "Background block download is complete");
					self.old_blocks = None;
				}
			}
		};
	}

	/// Mark all outstanding requests as expired
	fn reset_downloads(&mut self, block_set: BlockSet) {
		trace!(target: "sync", "Resetting downloads for {:?}", block_set);
		for (_, ref mut p) in self.peers.iter_mut().filter(|&(_, ref p)| p.block_set == Some(block_set)) {
			p.reset_asking();
		}
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

	/// Send Status message
	fn send_status(&mut self, io: &mut SyncIo, peer: PeerId) -> Result<(), network::Error> {
		let warp_protocol_version = io.protocol_version(&WARP_SYNC_PROTOCOL_ID, peer);
		let warp_protocol = warp_protocol_version != 0;
		let private_tx_protocol = warp_protocol_version >= PAR_PROTOCOL_VERSION_3.0;
		let protocol = if warp_protocol { warp_protocol_version } else { ETH_PROTOCOL_VERSION_63.0 };
		trace!(target: "sync", "Sending status to {}, protocol version {}", peer, protocol);
		let mut packet = RlpStream::new();
		packet.begin_unbounded_list();
		let chain = io.chain().chain_info();
		packet.append(&(protocol as u32));
		packet.append(&self.network_id);
		packet.append(&chain.total_difficulty);
		packet.append(&chain.best_block_hash);
		packet.append(&chain.genesis_hash);
		if warp_protocol {
			let manifest = io.snapshot_service().manifest();
			let block_number = manifest.as_ref().map_or(0, |m| m.block_number);
			let manifest_hash = manifest.map_or(H256::new(), |m| keccak(m.into_rlp()));
			packet.append(&manifest_hash);
			packet.append(&block_number);
			if private_tx_protocol {
				packet.append(&self.private_tx_handler.is_some());
			}
		}
		packet.complete_unbounded_list();
		io.respond(StatusPacket.id(), packet.out())
	}

	pub fn maintain_peers(&mut self, io: &mut SyncIo) {
		let tick = Instant::now();
		let mut aborting = Vec::new();
		for (peer_id, peer) in &self.peers {
			let elapsed = tick - peer.ask_time;
			let timeout = match peer.asking {
				PeerAsking::BlockHeaders => elapsed > HEADERS_TIMEOUT,
				PeerAsking::BlockBodies => elapsed > BODIES_TIMEOUT,
				PeerAsking::BlockReceipts => elapsed > RECEIPTS_TIMEOUT,
				PeerAsking::Nothing => false,
				PeerAsking::ForkHeader => elapsed > FORK_HEADER_TIMEOUT,
				PeerAsking::SnapshotManifest => elapsed > SNAPSHOT_MANIFEST_TIMEOUT,
				PeerAsking::SnapshotData => elapsed > SNAPSHOT_DATA_TIMEOUT,
			};
			if timeout {
				debug!(target:"sync", "Timeout {}", peer_id);
				io.disconnect_peer(*peer_id);
				aborting.push(*peer_id);
			}
		}
		for p in aborting {
			SyncHandler::on_peer_aborting(self, io, p);
		}

		// Check for handshake timeouts
		for (peer, &ask_time) in &self.handshaking_peers {
			let elapsed = (tick - ask_time) / 1_000_000_000;
			if elapsed > STATUS_TIMEOUT {
				trace!(target:"sync", "Status timeout {}", peer);
				io.disconnect_peer(*peer);
			}
		}
	}

	fn check_resume(&mut self, io: &mut SyncIo) {
		match self.state {
			SyncState::Waiting if !io.chain().queue_info().is_full() => {
				self.set_state(SyncState::Blocks);
				self.continue_sync(io);
			},
			SyncState::SnapshotData => match io.snapshot_service().status() {
				RestorationStatus::Inactive | RestorationStatus::Failed => {
					self.set_state(SyncState::SnapshotWaiting);
				},
				RestorationStatus::Initializing { .. } | RestorationStatus::Ongoing { .. } => (),
			},
			SyncState::SnapshotWaiting => {
				match io.snapshot_service().status() {
					RestorationStatus::Inactive => {
						trace!(target:"sync", "Snapshot restoration is complete");
						self.restart(io);
					},
					RestorationStatus::Initializing { .. } => {
						trace!(target:"sync", "Snapshot restoration is initializing");
					},
					RestorationStatus::Ongoing { state_chunks_done, block_chunks_done, .. } => {
						if !self.snapshot.is_complete() && self.snapshot.done_chunks() - (state_chunks_done + block_chunks_done) as usize <= MAX_SNAPSHOT_CHUNKS_DOWNLOAD_AHEAD {
							trace!(target:"sync", "Resuming snapshot sync");
							self.set_state(SyncState::SnapshotData);
							self.continue_sync(io);
						}
					},
					RestorationStatus::Failed => {
						trace!(target: "sync", "Snapshot restoration aborted");
						self.set_state(SyncState::WaitingPeers);
						self.snapshot.clear();
						self.continue_sync(io);
					},
				}
			},
			_ => (),
		}
	}

	/// returns peer ids that have different block than our chain
	fn get_lagging_peers(&self, chain_info: &BlockChainInfo) -> Vec<PeerId> {
		self.get_peers(chain_info, PeerState::Lagging)
	}

	/// returns peer ids that have different or the same blocks than our chain
	fn get_peers(&self, chain_info: &BlockChainInfo, peers: PeerState) -> Vec<PeerId> {
		let latest_hash = chain_info.best_block_hash;
		self
			.peers
			.iter()
			.filter_map(|(&id, ref mut peer_info)| {
				trace!(target: "sync", "Checking peer our best {} their best {}", latest_hash, peer_info.latest_hash);
				let matches = match peers {
					PeerState::Lagging => peer_info.latest_hash != latest_hash,
					PeerState::SameBlock => peer_info.latest_hash == latest_hash,
				};
				if matches {
					Some(id)
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
	}

	fn get_consensus_peers(&self) -> Vec<PeerId> {
		self.peers.iter().filter_map(|(id, p)| if p.protocol_version >= PAR_PROTOCOL_VERSION_2.0 { Some(*id) } else { None }).collect()
	}

	fn get_private_transaction_peers(&self, transaction_hash: &H256) -> Vec<PeerId> {
		self.peers.iter().filter_map(
			|(id, p)| if p.protocol_version >= PAR_PROTOCOL_VERSION_3.0
				&& !p.last_sent_private_transactions.contains(transaction_hash)
				&& p.private_tx_enabled {
					Some(*id)
				} else {
					None
				}
		).collect()
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
			SyncPropagator::propagate_latest_blocks(self, io, sealed);
			SyncPropagator::propagate_proposed_blocks(self, io, proposed);
		}
		if !invalid.is_empty() {
			trace!(target: "sync", "Bad blocks in the queue, restarting");
			self.restart(io);
		}

		if !is_syncing && !enacted.is_empty() && !self.peers.is_empty() {
			// Select random peer to re-broadcast transactions to.
			let peer = random::new().gen_range(0, self.peers.len());
			trace!(target: "sync", "Re-broadcasting transactions to a random peer.");
			self.peers.values_mut().nth(peer).map(|peer_info| {
					peer_info.last_sent_transactions.clear();
					peer_info.reset_private_stats()
				}
			);
		}
	}

	pub fn on_packet(&mut self, io: &mut SyncIo, peer: PeerId, packet_id: u8, data: &[u8]) {
		SyncHandler::on_packet(self, io, peer, packet_id, data);
	}

	/// Called by peer when it is disconnecting
	pub fn on_peer_aborting(&mut self, io: &mut SyncIo, peer: PeerId) {
		SyncHandler::on_peer_aborting(self, io, peer);
	}

	/// Called when a new peer is connected
	pub fn on_peer_connected(&mut self, io: &mut SyncIo, peer: PeerId) {
		SyncHandler::on_peer_connected(self, io, peer);
	}

	/// propagates new transactions to all peers
	pub fn propagate_new_transactions(&mut self, io: &mut SyncIo) {
		let deadline = Instant::now() + Duration::from_millis(500);
		SyncPropagator::propagate_new_transactions(self, io, || {
			if deadline > Instant::now() {
				true
			} else {
				debug!(target: "sync", "Wasn't able to finish transaction propagation within a deadline.");
				false
			}
		});
	}

	/// Broadcast consensus message to peers.
	pub fn propagate_consensus_packet(&mut self, io: &mut SyncIo, packet: Bytes) {
		SyncPropagator::propagate_consensus_packet(self, io, packet);
	}

	/// Broadcast private transaction message to peers.
	pub fn propagate_private_transaction(&mut self, io: &mut SyncIo, transaction_hash: H256, packet_id: SyncPacket, packet: Bytes) {
		SyncPropagator::propagate_private_transaction(self, io, transaction_hash, packet_id, packet);
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::{VecDeque};
	use ethkey;
	use network::PeerId;
	use tests::helpers::{TestIo};
	use tests::snapshot::TestSnapshotService;
	use ethereum_types::{H256, U256, Address};
	use parking_lot::RwLock;
	use bytes::Bytes;
	use rlp::{Rlp, RlpStream};
	use super::*;
	use ::SyncConfig;
	use super::{PeerInfo, PeerAsking};
	use ethcore::client::{BlockChainClient, EachBlockWith, TestBlockChainClient, ChainInfo, BlockInfo};
	use ethcore::miner::{MinerService, PendingOrdering};
	use types::header::Header;

	pub fn get_dummy_block(order: u32, parent_hash: H256) -> Bytes {
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

	pub fn get_dummy_blocks(order: u32, parent_hash: H256) -> Bytes {
		let mut rlp = RlpStream::new_list(2);
		rlp.append_raw(&get_dummy_block(order, parent_hash), 1);
		let difficulty: U256 = (100 * order).into();
		rlp.append(&difficulty);
		rlp.out()
	}

	pub fn get_dummy_hashes() -> Bytes {
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

	pub fn dummy_sync_with_peer(peer_latest_hash: H256, client: &BlockChainClient) -> ChainSync {

		let mut sync = ChainSync::new(SyncConfig::default(), client, None,);
		insert_dummy_peer(&mut sync, 0, peer_latest_hash);
		sync
	}

	pub fn insert_dummy_peer(sync: &mut ChainSync, peer_id: PeerId, peer_latest_hash: H256) {
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
				ask_time: Instant::now(),
				last_sent_transactions: Default::default(),
				last_sent_private_transactions: Default::default(),
				expired: false,
				private_tx_enabled: false,
				confirmation: super::ForkConfirmation::Confirmed,
				snapshot_number: None,
				snapshot_hash: None,
				asking_snapshot_data: None,
				block_set: None,
				client_version: ClientVersion::from(""),
			});

	}

	#[test]
	fn finds_lagging_peers() {
		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Uncle);
		let sync = dummy_sync_with_peer(client.block_hash_delta_minus(10), &client);
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
		SyncPropagator::propagate_new_hashes(&mut sync, &chain_info, &mut io, &peers);

		let data = &io.packets[0].data.clone();
		let result = SyncHandler::on_peer_new_hashes(&mut sync, &mut io, 0, &Rlp::new(data));
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
		SyncPropagator::propagate_blocks(&mut sync, &chain_info, &mut io, &[], &peers);

		let data = &io.packets[0].data.clone();
		let result = SyncHandler::on_peer_new_block(&mut sync, &mut io, 0, &Rlp::new(data));
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
			io.chain.miner.chain_new_blocks(io.chain, &[], &[], &[], &good_blocks, false);
			sync.chain_new_blocks(&mut io, &[], &[], &[], &good_blocks, &[], &[]);
			assert_eq!(io.chain.miner.ready_transactions(io.chain, 10, PendingOrdering::Priority).len(), 1);
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
			io.chain.miner.chain_new_blocks(io.chain, &[], &[], &good_blocks, &retracted_blocks, false);
			sync.chain_new_blocks(&mut io, &[], &[], &good_blocks, &retracted_blocks, &[], &[]);
		}

		// then
		assert_eq!(client.miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 1);
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
		assert_eq!(io.chain.miner.queue_status().status.transaction_count, 0);
		sync.chain_new_blocks(&mut io, &[], &[], &good_blocks, &retracted_blocks, &[], &[]);

		// then
		let status = io.chain.miner.queue_status();
		assert_eq!(status.status.transaction_count, 0);
	}
}
