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

///
/// Blockchain downloader
///

use std::collections::{HashSet, VecDeque};
use std::cmp;

use crate::{
	blocks::{BlockCollection, SyncBody, SyncHeader},
	chain::BlockSet,
	sync_io::SyncIo
};

use ethereum_types::H256;
use log::{debug, trace};
use network::{client_version::ClientCapabilities, PeerId};
use rlp::Rlp;
use parity_util_mem::MallocSizeOf;
use common_types::{
	BlockNumber,
	block_status::BlockStatus,
	ids::BlockId,
	errors::{EthcoreError, BlockError, ImportError},
};

const MAX_HEADERS_TO_REQUEST: usize = 128;
const MAX_BODIES_TO_REQUEST_LARGE: usize = 128;
const MAX_BODIES_TO_REQUEST_SMALL: usize = 32; // Size request for parity clients prior to 2.4.0
const MAX_RECEPITS_TO_REQUEST: usize = 256;
const SUBCHAIN_SIZE: u64 = 256;
const MAX_ROUND_PARENTS: usize = 16;
const MAX_PARALLEL_SUBCHAIN_DOWNLOAD: usize = 5;
const MAX_USELESS_HEADERS_PER_ROUND: usize = 3;

// logging macros prepend BlockSet context for log filtering
macro_rules! trace_sync {
	($self:ident, $fmt:expr, $($arg:tt)+) => {
		trace!(target: "sync", concat!("{:?}: ", $fmt), $self.block_set, $($arg)+);
	};
	($self:ident, $fmt:expr) => {
		trace!(target: "sync", concat!("{:?}: ", $fmt), $self.block_set);
	};
}

macro_rules! debug_sync {
	($self:ident, $fmt:expr, $($arg:tt)+) => {
		debug!(target: "sync", concat!("{:?}: ", $fmt), $self.block_set, $($arg)+);
	};
	($self:ident, $fmt:expr) => {
		debug!(target: "sync", concat!("{:?}: ", $fmt), $self.block_set);
	};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, MallocSizeOf)]
/// Downloader state
pub enum State {
	/// No active downloads.
	Idle,
	/// Downloading subchain heads
	ChainHead,
	/// Downloading blocks
	Blocks,
	/// Download is complete
	Complete,
}

/// Data that needs to be requested from a peer.
pub enum BlockRequest {
	Headers {
		start: H256,
		count: u64,
		skip: u64,
	},
	Bodies {
		hashes: Vec<H256>,
	},
	Receipts {
		hashes: Vec<H256>,
	},
}

/// Indicates sync action
#[derive(Eq, PartialEq, Debug)]
pub enum DownloadAction {
	/// Do nothing
	None,
	/// Reset downloads for all peers
	Reset
}

#[derive(Eq, PartialEq, Debug)]
pub enum BlockDownloaderImportError {
	/// Imported data is rejected as invalid. Peer should be dropped.
	Invalid,
	/// Imported data is valid but rejected cause the downloader does not need it.
	Useless,
}

impl From<rlp::DecoderError> for BlockDownloaderImportError {
	fn from(_: rlp::DecoderError) -> BlockDownloaderImportError {
		BlockDownloaderImportError::Invalid
	}
}

/// Block downloader strategy.
/// Manages state and block data for a block download process.
#[derive(MallocSizeOf)]
pub struct BlockDownloader {
	/// Which set of blocks to download
	block_set: BlockSet,
	/// Downloader state
	state: State,
	/// Highest block number seen
	highest_block: Option<BlockNumber>,
	/// Downloaded blocks, holds `H`, `B` and `S`
	blocks: BlockCollection,
	/// Last imported block number
	last_imported_block: BlockNumber,
	/// Last imported block hash
	last_imported_hash: H256,
	/// Number of blocks imported this round
	imported_this_round: Option<usize>,
	/// Block number the last round started with.
	last_round_start: BlockNumber,
	last_round_start_hash: H256,
	/// Block parents imported this round (hash, parent)
	round_parents: VecDeque<(H256, H256)>,
	/// Do we need to download block recetips.
	download_receipts: bool,
	/// Sync up to the block with this hash.
	target_hash: Option<H256>,
	/// Probing range for seeking common best block.
	retract_step: u64,
	/// consecutive useless headers this round
	useless_headers_count: usize,
}

impl BlockDownloader {
	/// Create a new instance of syncing strategy.
	/// For BlockSet::NewBlocks this won't reorganize to before the last kept state.
	pub fn new(block_set: BlockSet, start_hash: &H256, start_number: BlockNumber) -> Self {
		let sync_receipts = match block_set {
			BlockSet::NewBlocks => false,
			BlockSet::OldBlocks => true
		};
		BlockDownloader {
			block_set,
			state: State::Idle,
			highest_block: None,
			last_imported_block: start_number,
			last_imported_hash: start_hash.clone(),
			last_round_start: start_number,
			last_round_start_hash: start_hash.clone(),
			blocks: BlockCollection::new(sync_receipts),
			imported_this_round: None,
			round_parents: VecDeque::new(),
			download_receipts: sync_receipts,
			target_hash: None,
			retract_step: 1,
			useless_headers_count: 0,
		}
	}

	/// Reset sync. Clear all local downloaded data.
	pub fn reset(&mut self) {
		self.blocks.clear();
		self.useless_headers_count = 0;
		self.state = State::Idle;
	}

	/// Mark a block as known in the chain
	pub fn mark_as_known(&mut self, hash: &H256, number: BlockNumber) {
		if number >= self.last_imported_block + 1 {
			self.last_imported_block = number;
			self.last_imported_hash = hash.clone();
			self.imported_this_round = Some(self.imported_this_round.unwrap_or(0) + 1);
			self.last_round_start = number;
			self.last_round_start_hash = hash.clone();
		}
	}

	/// Check if download is complete
	pub fn is_complete(&self) -> bool {
		self.state == State::Complete
	}

	/// Check if particular block hash is being downloaded
	pub fn is_downloading(&self, hash: &H256) -> bool {
		self.blocks.is_downloading(hash)
	}

	/// Set starting sync block
	pub fn set_target(&mut self, hash: &H256) {
		self.target_hash = Some(hash.clone());
	}

	/// Unmark header as being downloaded.
	pub fn clear_header_download(&mut self, hash: &H256) {
		self.blocks.clear_header_download(hash)
	}

	/// Unmark block body as being downloaded.
	pub fn clear_body_download(&mut self, hashes: &[H256]) {
		self.blocks.clear_body_download(hashes)
	}

	/// Unmark block receipt as being downloaded.
	pub fn clear_receipt_download(&mut self, hashes: &[H256]) {
		self.blocks.clear_receipt_download(hashes)
	}
	/// Reset collection for a new sync round with given subchain block hashes.
	pub fn reset_to(&mut self, hashes: Vec<H256>) {
		self.reset();
		self.blocks.reset_to(hashes);
		self.state = State::Blocks;
	}

	/// Returns best imported block number.
	pub fn last_imported_block_number(&self) -> BlockNumber {
		self.last_imported_block
	}

	/// Add new block headers.
	pub fn import_headers(&mut self, io: &mut dyn SyncIo, r: &Rlp, expected_hash: H256) -> Result<DownloadAction, BlockDownloaderImportError> {
		let item_count = r.item_count().unwrap_or(0);
		if self.state == State::Idle {
			trace_sync!(self, "Ignored unexpected block headers");
			return Ok(DownloadAction::None)
		}
		if item_count == 0 && (self.state == State::Blocks) {
			return Err(BlockDownloaderImportError::Invalid);
		}

		// The request is generated in ::request_blocks.
		let (max_count, skip) = if self.state == State::ChainHead {
			(SUBCHAIN_SIZE as usize, (MAX_HEADERS_TO_REQUEST - 2) as u64)
		} else {
			(MAX_HEADERS_TO_REQUEST, 0)
		};

		if item_count > max_count {
			debug!(target: "sync", "Headers response is larger than expected");
			return Err(BlockDownloaderImportError::Invalid);
		}

		let mut headers = Vec::new();
		let mut hashes = Vec::new();
		let mut last_header = None;
		for i in 0..item_count {
			let info = SyncHeader::from_rlp(r.at(i)?.as_raw().to_vec())?;
			let number = BlockNumber::from(info.header.number());
			let hash = info.header.hash();

			let valid_response = match last_header {
				// First header must match expected hash.
				None => expected_hash == hash,
				Some((last_number, last_hash)) => {
					// Subsequent headers must be spaced by skip interval.
					let skip_valid = number == last_number + skip + 1;
					// Consecutive headers must be linked by parent hash.
					let parent_valid = (number != last_number + 1) || *info.header.parent_hash() == last_hash;
					skip_valid && parent_valid
				}
			};

			// Disable the peer for this syncing round if it gives invalid chain
			if !valid_response {
				debug!(target: "sync", "Invalid headers response");
				return Err(BlockDownloaderImportError::Invalid);
			}

			last_header = Some((number, hash));
			if self.blocks.contains(&hash) {
				trace_sync!(self, "Skipping existing block header {} ({:?})", number, hash);
				continue;
			}

			match io.chain().block_status(BlockId::Hash(hash.clone())) {
				BlockStatus::InChain | BlockStatus::Queued => {
					match self.state {
						State::Blocks => trace_sync!(self, "Header already in chain {} ({})", number, hash),
						_ => trace_sync!(self, "Header already in chain {} ({}), state = {:?}", number, hash, self.state),
					}
					headers.push(info);
					hashes.push(hash);
				},
				BlockStatus::Bad => {
					return Err(BlockDownloaderImportError::Invalid);
				},
				BlockStatus::Unknown => {
					headers.push(info);
					hashes.push(hash);
				}
			}
		}

		if let Some((number, _)) = last_header {
			if self.highest_block.as_ref().map_or(true, |n| number > *n) {
				self.highest_block = Some(number);
			}
		}

		match self.state {
			State::ChainHead => {
				if !headers.is_empty() {
					trace_sync!(self, "Received {} subchain heads, proceeding to download", headers.len());
					self.blocks.reset_to(hashes);
					self.state = State::Blocks;
					return Ok(DownloadAction::Reset);
				} else {
					trace_sync!(self, "No useful subchain heads received, expected hash {:?}", expected_hash);
					let best = io.chain().chain_info().best_block_number;
					let oldest_reorg = io.chain().pruning_info().earliest_state;
					let last = self.last_imported_block;
					match self.block_set {
						BlockSet::NewBlocks if best > last && (last == 0 || last < oldest_reorg) => {
							trace_sync!(self, "No common block, disabling peer");
							return Err(BlockDownloaderImportError::Invalid)
						},
						BlockSet::OldBlocks => {
							trace_sync!(self, "Expected some useful headers for downloading OldBlocks. Try a different peer");
							return Err(BlockDownloaderImportError::Useless)
						},
						_ => (),
					}
				}
			},
			State::Blocks => {
				let count = headers.len();
				// At least one of the headers must advance the subchain. Otherwise they are all useless.
				if count == 0 {
					self.useless_headers_count += 1;
					trace_sync!(self, "No useful headers ({:?} this round), expected hash {:?}", self.useless_headers_count, expected_hash);
					// only reset download if we have multiple subchain heads, to avoid unnecessary resets
					// when we are at the head of the chain when we may legitimately receive no useful headers
					if self.blocks.heads_len() > 1 && self.useless_headers_count >= MAX_USELESS_HEADERS_PER_ROUND {
						trace_sync!(self, "Received {:?} useless responses this round. Resetting sync", MAX_USELESS_HEADERS_PER_ROUND);
						self.reset();
					}
					return Err(BlockDownloaderImportError::Useless);
				}
				self.blocks.insert_headers(headers);
				trace_sync!(self, "Inserted {} headers", count);
			},
			_ => trace_sync!(self, "Unexpected headers({})", headers.len()),
		}

		Ok(DownloadAction::None)
	}

	/// Called by peer once it has new block bodies
	pub fn import_bodies(&mut self, r: &Rlp, expected_hashes: &[H256]) -> Result<(), BlockDownloaderImportError> {
		let item_count = r.item_count().unwrap_or(0);
		if item_count == 0 {
			return Err(BlockDownloaderImportError::Useless);
		} else if self.state != State::Blocks {
			trace_sync!(self, "Ignored unexpected block bodies");
		} else {
			let mut bodies = Vec::with_capacity(item_count);
			for i in 0..item_count {
				let body = SyncBody::from_rlp(r.at(i)?.as_raw())?;
				bodies.push(body);
			}

			let hashes = self.blocks.insert_bodies(bodies);
			if hashes.len() != item_count {
				trace_sync!(self, "Deactivating peer for giving invalid block bodies");
				return Err(BlockDownloaderImportError::Invalid);
			}
			if !all_expected(hashes.as_slice(), expected_hashes, |&a, &b| a == b) {
				trace_sync!(self, "Deactivating peer for giving unexpected block bodies");
				return Err(BlockDownloaderImportError::Invalid);
			}
		}
		Ok(())
	}

	/// Called by peer once it has new block bodies
	pub fn import_receipts(&mut self, r: &Rlp, expected_hashes: &[H256]) -> Result<(), BlockDownloaderImportError> {
		let item_count = r.item_count().unwrap_or(0);
		if item_count == 0 {
			return Err(BlockDownloaderImportError::Useless);
		}
		else if self.state != State::Blocks {
			trace_sync!(self, "Ignored unexpected block receipts");
		}
		else {
			let mut receipts = Vec::with_capacity(item_count);
			for i in 0..item_count {
				let receipt = r.at(i).map_err(|e| {
					trace_sync!(self, "Error decoding block receipts RLP: {:?}", e);
					BlockDownloaderImportError::Invalid
				})?;
				receipts.push(receipt.as_raw().to_vec());
			}
			let hashes = self.blocks.insert_receipts(receipts);
			if hashes.len() != item_count {
				trace_sync!(self, "Deactivating peer for giving invalid block receipts");
				return Err(BlockDownloaderImportError::Invalid);
			}
			if !all_expected(hashes.as_slice(), expected_hashes, |a, b| a.contains(b)) {
				trace_sync!(self, "Deactivating peer for giving unexpected block receipts");
				return Err(BlockDownloaderImportError::Invalid);
			}
		}
		Ok(())
	}

	fn start_sync_round(&mut self, io: &mut dyn SyncIo) {
		self.state = State::ChainHead;
		trace_sync!(self, "Starting round (last imported count = {:?}, last started = {}, block = {:?}", self.imported_this_round, self.last_round_start, self.last_imported_block);
		// Check if need to retract to find the common block. The problem is that the peers still return headers by hash even
		// from the non-canonical part of the tree. So we also retract if nothing has been imported last round.
		let start = self.last_round_start;
		let start_hash = self.last_round_start_hash;
		match self.imported_this_round {
			Some(n) if n == 0 && start > 0 => {
				// nothing was imported last round, step back to a previous block
				// search parent in last round known parents first
				if let Some(&(_, p)) = self.round_parents.iter().find(|&&(h, _)| h == start_hash) {
					self.last_imported_block = start - 1;
					self.last_imported_hash = p.clone();
					trace_sync!(self, "Searching common header from the last round {} ({})", self.last_imported_block, self.last_imported_hash);
				} else {
					let best = io.chain().chain_info().best_block_number;
					let oldest_reorg = io.chain().pruning_info().earliest_state;
					if self.block_set == BlockSet::NewBlocks && best > start && start < oldest_reorg {
						debug_sync!(self, "Could not revert to previous ancient block, last: {} ({})", start, start_hash);
						self.reset();
					} else {
						let n = start - cmp::min(self.retract_step, start);
						self.retract_step *= 2;
						match io.chain().block_hash(BlockId::Number(n)) {
							Some(h) => {
								self.last_imported_block = n;
								self.last_imported_hash = h;
								trace_sync!(self, "Searching common header in the blockchain {} ({})", start, self.last_imported_hash);
							}
							None => {
								debug_sync!(self, "Could not revert to previous block, last: {} ({})", start, self.last_imported_hash);
								self.reset();
							}
						}
					}
				}
			},
			_ => {
				self.retract_step = 1;
			},
		}
		self.last_round_start = self.last_imported_block;
		self.last_round_start_hash = self.last_imported_hash;
		self.imported_this_round = None;
	}

	/// Find some headers or blocks to download for a peer.
	pub fn request_blocks(&mut self, peer_id: PeerId, io: &mut dyn SyncIo, num_active_peers: usize) -> Option<BlockRequest> {
		match self.state {
			State::Idle => {
				self.start_sync_round(io);
				if self.state == State::ChainHead {
					return self.request_blocks(peer_id, io, num_active_peers);
				}
			},
			State::ChainHead => {
				if num_active_peers < MAX_PARALLEL_SUBCHAIN_DOWNLOAD {
					// Request subchain headers
					trace_sync!(self, "Starting sync with better chain");
					// Request MAX_HEADERS_TO_REQUEST - 2 headers apart so that
					// MAX_HEADERS_TO_REQUEST would include headers for neighbouring subchains
					return Some(BlockRequest::Headers {
						start: self.last_imported_hash.clone(),
						count: SUBCHAIN_SIZE,
						skip: (MAX_HEADERS_TO_REQUEST - 2) as u64,
					});
				}
			},
			State::Blocks => {
				// check to see if we need to download any block bodies first
				let client_version = io.peer_version(peer_id);

				let number_of_bodies_to_request = if client_version.can_handle_large_requests() {
					MAX_BODIES_TO_REQUEST_LARGE
				} else {
					MAX_BODIES_TO_REQUEST_SMALL
				};

				let needed_bodies = self.blocks.needed_bodies(number_of_bodies_to_request, false);
				if !needed_bodies.is_empty() {
					return Some(BlockRequest::Bodies {
						hashes: needed_bodies,
					});
				}

				if self.download_receipts {
					let needed_receipts = self.blocks.needed_receipts(MAX_RECEPITS_TO_REQUEST, false);
					if !needed_receipts.is_empty() {
						return Some(BlockRequest::Receipts {
							hashes: needed_receipts,
						});
					}
				}

				// find subchain to download
				if let Some((h, count)) = self.blocks.needed_headers(MAX_HEADERS_TO_REQUEST, false) {
					return Some(BlockRequest::Headers {
						start: h,
						count: count as u64,
						skip: 0,
					});
				}
			},
			State::Complete => (),
		}
		None
	}

	/// Checks if there are blocks fully downloaded that can be imported into the blockchain and does the import.
	/// Returns DownloadAction::Reset if it is imported all the the blocks it can and all downloading peers should be reset
	pub fn collect_blocks(&mut self, io: &mut dyn SyncIo, allow_out_of_order: bool) -> DownloadAction {
		let mut download_action = DownloadAction::None;
		let mut imported = HashSet::new();
		let blocks = self.blocks.drain();
		let count = blocks.len();
		for block_and_receipts in blocks {
			let block = block_and_receipts.block;
			let receipts = block_and_receipts.receipts;

			let h = block.header.hash();
			let number = block.header.number();
			let parent = *block.header.parent_hash();

			if self.target_hash.as_ref().map_or(false, |t| t == &h) {
				self.state = State::Complete;
				trace_sync!(self, "Sync target reached");
				return download_action;
			}

			let result = if let Some(receipts) = receipts {
				io.chain().queue_ancient_block(block, receipts)
			} else {
				trace_sync!(self, "Importing block #{}/{}", number, h);
				io.chain().import_block(block)
			};

			match result {
				Err(EthcoreError::Import(ImportError::AlreadyInChain)) => {
					let is_canonical = if io.chain().block_hash(BlockId::Number(number)).is_some() {
						"canoncial"
					} else {
						"not canonical"
					};
					trace_sync!(self, "Block #{} is already in chain {:?} – {}", number, h, is_canonical);
					self.block_imported(&h, number, &parent);
				},
				Err(EthcoreError::Import(ImportError::AlreadyQueued)) => {
					trace_sync!(self, "Block already queued {:?}", h);
					self.block_imported(&h, number, &parent);
				},
				Ok(_) => {
					trace_sync!(self, "Block queued {:?}", h);
					imported.insert(h.clone());
					self.block_imported(&h, number, &parent);
				},
				Err(EthcoreError::Block(BlockError::UnknownParent(_))) if allow_out_of_order => {
					break;
				},
				Err(EthcoreError::Block(BlockError::UnknownParent(_))) => {
					trace_sync!(self, "Unknown new block parent, restarting sync");
					break;
				},
				Err(EthcoreError::Block(BlockError::TemporarilyInvalid(_))) => {
					debug_sync!(self, "Block temporarily invalid: {:?}, restarting sync", h);
					break;
				},
				Err(EthcoreError::FullQueue(limit)) => {
					debug_sync!(self, "Block import queue full ({}), restarting sync", limit);
					download_action = DownloadAction::Reset;
					break;
				},
				Err(e) => {
					debug_sync!(self, "Bad block {:?} : {:?}", h, e);
					download_action = DownloadAction::Reset;
					break;
				}
			}
		}
		trace_sync!(self, "Imported {} of {}", imported.len(), count);
		self.imported_this_round = Some(self.imported_this_round.unwrap_or(0) + imported.len());

		if self.blocks.is_empty() {
			// complete sync round
			trace_sync!(self, "Sync round complete");
			download_action = DownloadAction::Reset;
		}
		download_action
	}

	fn block_imported(&mut self, hash: &H256, number: BlockNumber, parent: &H256) {
		self.last_imported_block = number;
		self.last_imported_hash = hash.clone();
		self.round_parents.push_back((hash.clone(), parent.clone()));
		if self.round_parents.len() > MAX_ROUND_PARENTS {
			self.round_parents.pop_front();
		}
	}
}

// Determines if the first argument matches an ordered subset of the second, according to some predicate.
fn all_expected<A, B, F>(values: &[A], expected_values: &[B], is_expected: F) -> bool
	where F: Fn(&A, &B) -> bool
{
	let mut expected_iter = expected_values.iter();
	values.iter().all(|val1| {
		while let Some(val2) = expected_iter.next() {
			if is_expected(val1, val2) {
				return true;
			}
		}
		false
	})
}

#[cfg(test)]
mod tests {
	use super::{
		BlockSet, BlockDownloader, BlockDownloaderImportError, DownloadAction, SyncIo, H256,
		MAX_HEADERS_TO_REQUEST, MAX_USELESS_HEADERS_PER_ROUND, SUBCHAIN_SIZE, State, Rlp, VecDeque
	};

	use crate::tests::{helpers::TestIo, snapshot::TestSnapshotService};

	use ethcore::test_helpers::TestBlockChainClient;
	use parity_crypto::publickey::{Random, Generator};
	use keccak_hash::keccak;
	use parking_lot::RwLock;
	use rlp::{encode_list, RlpStream};
	use triehash_ethereum::ordered_trie_root;
	use common_types::{
		transaction::{Transaction, SignedTransaction},
		header::Header as BlockHeader,
	};

	fn dummy_header(number: u64, parent_hash: H256) -> BlockHeader {
		let mut header = BlockHeader::new();
		header.set_gas_limit(0.into());
		header.set_difficulty((number * 100).into());
		header.set_timestamp(number * 10);
		header.set_number(number);
		header.set_parent_hash(parent_hash);
		header.set_state_root(H256::zero());
		header
	}

	fn dummy_signed_tx() -> SignedTransaction {
		let keypair = Random.generate().unwrap();
		Transaction::default().sign(keypair.secret(), None)
	}

	fn import_headers(headers: &[BlockHeader], downloader: &mut BlockDownloader, io: &mut dyn SyncIo) -> Result<DownloadAction, BlockDownloaderImportError> {
		let mut stream = RlpStream::new();
		stream.append_list(headers);
		let bytes = stream.out();
		let rlp = Rlp::new(&bytes);
		let expected_hash = headers.first().unwrap().hash();
		downloader.import_headers(io, &rlp, expected_hash)
	}

	fn import_headers_ok(headers: &[BlockHeader], downloader: &mut BlockDownloader, io: &mut dyn SyncIo) {
		let res = import_headers(headers, downloader, io);
		assert!(res.is_ok());
	}

	#[test]
	fn import_headers_in_chain_head_state() {
		env_logger::try_init().ok();

		let spec = spec::new_test();
		let genesis_hash = spec.genesis_header().hash();

		let mut downloader = BlockDownloader::new(BlockSet::NewBlocks, &genesis_hash, 0);
		downloader.state = State::ChainHead;

		let mut chain = TestBlockChainClient::new();
		let snapshot_service = TestSnapshotService::new();
		let queue = RwLock::new(VecDeque::new());
		let mut io = TestIo::new(&mut chain, &snapshot_service, &queue, None, None);

		// Valid headers sequence.
		let valid_headers = [
			spec.genesis_header(),
			dummy_header(127, H256::random()),
			dummy_header(254, H256::random()),
		];
		let rlp_data = encode_list(&valid_headers);
		let valid_rlp = Rlp::new(&rlp_data);

		match downloader.import_headers(&mut io, &valid_rlp, genesis_hash) {
			Ok(DownloadAction::Reset) => assert_eq!(downloader.state, State::Blocks),
			_ => panic!("expected transition to Blocks state"),
		};

		// Headers are rejected because the expected hash does not match.
		let invalid_start_block_headers = [
			dummy_header(0, H256::random()),
			dummy_header(127, H256::random()),
			dummy_header(254, H256::random()),
		];
		let rlp_data = encode_list(&invalid_start_block_headers);
		let invalid_start_block_rlp = Rlp::new(&rlp_data);

		match downloader.import_headers(&mut io, &invalid_start_block_rlp, genesis_hash) {
			Err(BlockDownloaderImportError::Invalid) => (),
			_ => panic!("expected BlockDownloaderImportError"),
		};

		// Headers are rejected because they are not spaced as expected.
		let invalid_skip_headers = [
			spec.genesis_header(),
			dummy_header(128, H256::random()),
			dummy_header(256, H256::random()),
		];
		let rlp_data = encode_list(&invalid_skip_headers);
		let invalid_skip_rlp = Rlp::new(&rlp_data);

		match downloader.import_headers(&mut io, &invalid_skip_rlp, genesis_hash) {
			Err(BlockDownloaderImportError::Invalid) => (),
			_ => panic!("expected BlockDownloaderImportError"),
		};

		// Invalid because the packet size is too large.
		let mut too_many_headers = Vec::with_capacity((SUBCHAIN_SIZE + 1) as usize);
		too_many_headers.push(spec.genesis_header());
		for i in 1..(SUBCHAIN_SIZE + 1) {
			too_many_headers.push(dummy_header((MAX_HEADERS_TO_REQUEST as u64 - 1) * i, H256::random()));
		}
		let rlp_data = encode_list(&too_many_headers);

		let too_many_rlp = Rlp::new(&rlp_data);
		match downloader.import_headers(&mut io, &too_many_rlp, genesis_hash) {
			Err(BlockDownloaderImportError::Invalid) => (),
			_ => panic!("expected BlockDownloaderImportError"),
		};
	}

	#[test]
	fn import_headers_in_blocks_state() {
		env_logger::try_init().ok();

		let mut chain = TestBlockChainClient::new();
		let snapshot_service = TestSnapshotService::new();
		let queue = RwLock::new(VecDeque::new());
		let mut io = TestIo::new(&mut chain, &snapshot_service, &queue, None, None);

		let mut headers = Vec::with_capacity(3);
		let parent_hash = H256::random();
		headers.push(dummy_header(127, parent_hash));
		let parent_hash = headers[0].hash();
		headers.push(dummy_header(128, parent_hash));
		let parent_hash = headers[1].hash();
		headers.push(dummy_header(129, parent_hash));

		let mut downloader = BlockDownloader::new(BlockSet::NewBlocks, &H256::random(), 0);
		downloader.state = State::Blocks;
		downloader.blocks.reset_to(vec![headers[0].hash()]);

		let rlp_data = encode_list(&headers);
		let headers_rlp = Rlp::new(&rlp_data);

		match downloader.import_headers(&mut io, &headers_rlp, headers[0].hash()) {
			Ok(DownloadAction::None) => (),
			_ => panic!("expected successful import"),
		};

		// Invalidate parent_hash link.
		headers[2] = dummy_header(129, H256::random());
		let rlp_data = encode_list(&headers);
		let headers_rlp = Rlp::new(&rlp_data);

		match downloader.import_headers(&mut io, &headers_rlp, headers[0].hash()) {
			Err(BlockDownloaderImportError::Invalid) => (),
			_ => panic!("expected BlockDownloaderImportError"),
		};

		// Invalidate header sequence by skipping a header.
		headers[2] = dummy_header(130, headers[1].hash());
		let rlp_data = encode_list(&headers);
		let headers_rlp = Rlp::new(&rlp_data);

		match downloader.import_headers(&mut io, &headers_rlp, headers[0].hash()) {
			Err(BlockDownloaderImportError::Invalid) => (),
			_ => panic!("expected BlockDownloaderImportError"),
		};
	}

	#[test]
	fn import_bodies() {
		env_logger::try_init().ok();

		let mut chain = TestBlockChainClient::new();
		let snapshot_service = TestSnapshotService::new();
		let queue = RwLock::new(VecDeque::new());
		let mut io = TestIo::new(&mut chain, &snapshot_service, &queue, None, None);

		// Import block headers.
		let mut headers = Vec::with_capacity(4);
		let mut bodies = Vec::with_capacity(4);
		let mut parent_hash = H256::zero();
		for i in 0..4 {
			// Construct the block body
			let uncles = if i > 0 {
				encode_list(&[dummy_header(i - 1, H256::random())])
			} else {
				::rlp::EMPTY_LIST_RLP.to_vec()
			};

			let txs = encode_list(&[dummy_signed_tx()]);
			let tx_root = ordered_trie_root(Rlp::new(&txs).iter().map(|r| r.as_raw()));

			let mut rlp = RlpStream::new_list(2);
			rlp.append_raw(&txs, 1);
			rlp.append_raw(&uncles, 1);
			bodies.push(rlp.out());

			// Construct the block header
			let mut header = dummy_header(i, parent_hash);
			header.set_transactions_root(tx_root);
			header.set_uncles_hash(keccak(&uncles));
			parent_hash = header.hash();
			headers.push(header);
		}

		let mut downloader = BlockDownloader::new(BlockSet::NewBlocks, &headers[0].hash(), 0);
		downloader.state = State::Blocks;
		downloader.blocks.reset_to(vec![headers[0].hash()]);

		// Only import the first three block headers.
		let rlp_data = encode_list(&headers[0..3]);
		let headers_rlp = Rlp::new(&rlp_data);
		assert!(downloader.import_headers(&mut io, &headers_rlp, headers[0].hash()).is_ok());

		// Import first body successfully.
		let mut rlp_data = RlpStream::new_list(1);
		rlp_data.append_raw(&bodies[0], 1);
		let bodies_rlp = Rlp::new(rlp_data.as_raw());
		assert!(downloader.import_bodies(&bodies_rlp, &[headers[0].hash(), headers[1].hash()]).is_ok());

		// Import second body successfully.
		let mut rlp_data = RlpStream::new_list(1);
		rlp_data.append_raw(&bodies[1], 1);
		let bodies_rlp = Rlp::new(rlp_data.as_raw());
		assert!(downloader.import_bodies(&bodies_rlp, &[headers[0].hash(), headers[1].hash()]).is_ok());

		// Import unexpected third body.
		let mut rlp_data = RlpStream::new_list(1);
		rlp_data.append_raw(&bodies[2], 1);
		let bodies_rlp = Rlp::new(rlp_data.as_raw());
		match downloader.import_bodies(&bodies_rlp, &[headers[0].hash(), headers[1].hash()]) {
			Err(BlockDownloaderImportError::Invalid) => (),
			_ => panic!("expected BlockDownloaderImportError"),
		};
	}

	#[test]
	fn import_receipts() {
		env_logger::try_init().ok();

		let mut chain = TestBlockChainClient::new();
		let snapshot_service = TestSnapshotService::new();
		let queue = RwLock::new(VecDeque::new());
		let mut io = TestIo::new(&mut chain, &snapshot_service, &queue, None, None);

		// Import block headers.
		let mut headers = Vec::with_capacity(4);
		let mut receipts = Vec::with_capacity(4);
		let mut parent_hash = H256::zero();
		for i in 0..4 {
			// Construct the receipts. Receipt root for the first two blocks is the same.
			//
			// The RLP-encoded integers are clearly not receipts, but the BlockDownloader treats
			// all receipts as byte blobs, so it does not matter.
			let receipts_rlp = if i < 2 {
				encode_list(&[0u32])
			} else {
				encode_list(&[i as u32])
			};
			let receipts_root = ordered_trie_root(Rlp::new(&receipts_rlp).iter().map(|r| r.as_raw()));
			receipts.push(receipts_rlp);

			// Construct the block header.
			let mut header = dummy_header(i, parent_hash);
			header.set_receipts_root(receipts_root);
			parent_hash = header.hash();
			headers.push(header);
		}

		let mut downloader = BlockDownloader::new(BlockSet::OldBlocks, &headers[0].hash(), 0);
		downloader.state = State::Blocks;
		downloader.blocks.reset_to(vec![headers[0].hash()]);

		// Only import the first three block headers.
		let rlp_data = encode_list(&headers[0..3]);
		let headers_rlp = Rlp::new(&rlp_data);
		assert!(downloader.import_headers(&mut io, &headers_rlp, headers[0].hash()).is_ok());

		// Import second and third receipts successfully.
		let mut rlp_data = RlpStream::new_list(2);
		rlp_data.append_raw(&receipts[1], 1);
		rlp_data.append_raw(&receipts[2], 1);
		let receipts_rlp = Rlp::new(rlp_data.as_raw());
		assert!(downloader.import_receipts(&receipts_rlp, &[headers[1].hash(), headers[2].hash()]).is_ok());

		// Import unexpected fourth receipt.
		let mut rlp_data = RlpStream::new_list(1);
		rlp_data.append_raw(&receipts[3], 1);
		let bodies_rlp = Rlp::new(rlp_data.as_raw());
		match downloader.import_bodies(&bodies_rlp, &[headers[1].hash(), headers[2].hash()]) {
			Err(BlockDownloaderImportError::Invalid) => (),
			_ => panic!("expected BlockDownloaderImportError"),
		};
	}

	#[test]
	fn reset_after_multiple_sets_of_useless_headers() {
		env_logger::try_init().ok();

		let spec = spec::new_test();
		let genesis_hash = spec.genesis_header().hash();

		let mut downloader = BlockDownloader::new(BlockSet::NewBlocks, &genesis_hash, 0);
		downloader.state = State::ChainHead;

		let mut chain = TestBlockChainClient::new();
		let snapshot_service = TestSnapshotService::new();
		let queue = RwLock::new(VecDeque::new());
		let mut io = TestIo::new(&mut chain, &snapshot_service, &queue, None, None);

		let heads = [
			spec.genesis_header(),
			dummy_header(127, H256::random()),
			dummy_header(254, H256::random()),
		];

		let short_subchain = [dummy_header(1, genesis_hash)];

		import_headers_ok(&heads, &mut downloader, &mut io);
		import_headers_ok(&short_subchain, &mut downloader, &mut io);

		assert_eq!(downloader.state, State::Blocks);
		assert!(!downloader.blocks.is_empty());

		// simulate receiving useless headers
		let head = vec![short_subchain.last().unwrap().clone()];
		for _ in 0..MAX_USELESS_HEADERS_PER_ROUND {
			let res = import_headers(&head, &mut downloader, &mut io);
			assert!(res.is_err());
		}

		assert_eq!(downloader.state, State::Idle);
		assert!(downloader.blocks.is_empty());
	}

	#[test]
	fn dont_reset_after_multiple_sets_of_useless_headers_for_chain_head() {
		env_logger::try_init().ok();

		let spec = spec::new_test();
		let genesis_hash = spec.genesis_header().hash();

		let mut downloader = BlockDownloader::new(BlockSet::NewBlocks, &genesis_hash, 0);
		downloader.state = State::ChainHead;

		let mut chain = TestBlockChainClient::new();
		let snapshot_service = TestSnapshotService::new();
		let queue = RwLock::new(VecDeque::new());
		let mut io = TestIo::new(&mut chain, &snapshot_service, &queue, None, None);

		let heads = [
			spec.genesis_header()
		];

		let short_subchain = [dummy_header(1, genesis_hash)];

		import_headers_ok(&heads, &mut downloader, &mut io);
		import_headers_ok(&short_subchain, &mut downloader, &mut io);

		assert_eq!(downloader.state, State::Blocks);
		assert!(!downloader.blocks.is_empty());

		// simulate receiving useless headers
		let head = vec![short_subchain.last().unwrap().clone()];
		for _ in 0..MAX_USELESS_HEADERS_PER_ROUND {
			let res = import_headers(&head, &mut downloader, &mut io);
			assert!(res.is_err());
		}

		// download shouldn't be reset since this is the chain head for a single subchain.
		// this state usually occurs for NewBlocks when it has reached the chain head.
		assert_eq!(downloader.state, State::Blocks);
		assert!(!downloader.blocks.is_empty());
	}
}
