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

//! Light client implementation. Stores data from light sync

use std::sync::Arc;

use ethcore::block_import_error::BlockImportError;
use ethcore::block_status::BlockStatus;
use ethcore::client::{ClientReport, EnvInfo};
use ethcore::engines::Engine;
use ethcore::ids::BlockId;
use ethcore::header::Header;
use ethcore::verification::queue::{self, HeaderQueue};
use ethcore::blockchain_info::BlockChainInfo;
use ethcore::spec::Spec;
use ethcore::service::ClientIoMessage;
use ethcore::encoded;
use io::IoChannel;

use util::{H256, U256, Mutex, RwLock};
use util::kvdb::{KeyValueDB, CompactionProfile};

use self::header_chain::{AncestryIter, HeaderChain};

use cache::Cache;

pub use self::service::Service;

mod header_chain;
mod service;

/// Configuration for the light client.
#[derive(Debug, Default, Clone)]
pub struct Config {
	/// Verification queue config.
	pub queue: queue::Config,
	/// Chain column in database.
	pub chain_column: Option<u32>,
	/// Database cache size. `None` => rocksdb default.
	pub db_cache_size: Option<usize>,
	/// State db compaction profile
	pub db_compaction: CompactionProfile,
	/// Should db have WAL enabled?
	pub db_wal: bool,
}

/// Trait for interacting with the header chain abstractly.
pub trait LightChainClient: Send + Sync {
	/// Get chain info.
	fn chain_info(&self) -> BlockChainInfo;

	/// Queue header to be verified. Required that all headers queued have their
	/// parent queued prior.
	fn queue_header(&self, header: Header) -> Result<H256, BlockImportError>;

	/// Attempt to get a block hash by block id.
	fn block_hash(&self, id: BlockId) -> Option<H256>;

	/// Attempt to get block header by block id.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

	/// Get the best block header.
	fn best_block_header(&self) -> encoded::Header;

	/// Get a block's chain score by ID.
	fn score(&self, id: BlockId) -> Option<U256>;

	/// Get an iterator over a block and its ancestry.
	fn ancestry_iter<'a>(&'a self, start: BlockId) -> Box<Iterator<Item=encoded::Header> + 'a>;

	/// Get the signing network ID.
	fn signing_network_id(&self) -> Option<u64>;

	/// Query whether a block is known.
	fn is_known(&self, hash: &H256) -> bool;

	/// Clear the queue.
	fn clear_queue(&self);

	/// Flush the queue.
	fn flush_queue(&self);

	/// Get queue info.
	fn queue_info(&self) -> queue::QueueInfo;

	/// Get the `i`th CHT root.
	fn cht_root(&self, i: usize) -> Option<H256>;
}

/// Something which can be treated as a `LightChainClient`.
pub trait AsLightClient {
	/// The kind of light client this can be treated as.
	type Client: LightChainClient;

	/// Access the underlying light client.
	fn as_light_client(&self) -> &Self::Client;
}

impl<T: LightChainClient> AsLightClient for T {
	type Client = Self;

	fn as_light_client(&self) -> &Self { self }
}

/// Light client implementation.
pub struct Client {
	queue: HeaderQueue,
	engine: Arc<Engine>,
	chain: HeaderChain,
	report: RwLock<ClientReport>,
	import_lock: Mutex<()>,
	db: Arc<KeyValueDB>,
}

impl Client {
	/// Create a new `Client`.
	pub fn new(config: Config, db: Arc<KeyValueDB>, chain_col: Option<u32>, spec: &Spec, io_channel: IoChannel<ClientIoMessage>, cache: Arc<Mutex<Cache>>) -> Result<Self, String> {
		let gh = ::rlp::encode(&spec.genesis_header());

		Ok(Client {
			queue: HeaderQueue::new(config.queue, spec.engine.clone(), io_channel, true),
			engine: spec.engine.clone(),
			chain: HeaderChain::new(db.clone(), chain_col, &gh, cache)?,
			report: RwLock::new(ClientReport::default()),
			import_lock: Mutex::new(()),
			db: db,
		})
	}

	/// Create a new `Client` backed purely in-memory.
	/// This will ignore all database options in the configuration.
	pub fn in_memory(config: Config, spec: &Spec, io_channel: IoChannel<ClientIoMessage>, cache: Arc<Mutex<Cache>>) -> Self {
		let db = ::util::kvdb::in_memory(0);

		Client::new(config, Arc::new(db), None, spec, io_channel, cache).expect("New DB creation infallible; qed")
	}

	/// Import a header to the queue for additional verification.
	pub fn import_header(&self, header: Header) -> Result<H256, BlockImportError> {
		self.queue.import(header).map_err(Into::into)
	}

	/// Inquire about the status of a given header.
	pub fn status(&self, hash: &H256) -> BlockStatus {
		match self.queue.status(hash) {
			queue::Status::Unknown => self.chain.status(hash),
			other => other.into(),
		}
	}

	/// Get the chain info.
	pub fn chain_info(&self) -> BlockChainInfo {
		let best_hdr = self.chain.best_header();
		let best_td = self.chain.best_block().total_difficulty;

		let first_block = self.chain.first_block();
		let genesis_hash = self.chain.genesis_hash();

		BlockChainInfo {
			total_difficulty: best_td,
			pending_total_difficulty: best_td + self.queue.total_difficulty(),
			genesis_hash: genesis_hash,
			best_block_hash: best_hdr.hash(),
			best_block_number: best_hdr.number(),
			best_block_timestamp: best_hdr.timestamp(),
			ancient_block_hash: if first_block.is_some() { Some(genesis_hash) } else { None },
			ancient_block_number: if first_block.is_some() { Some(0) } else { None },
			first_block_hash: first_block.as_ref().map(|first| first.hash),
			first_block_number: first_block.as_ref().map(|first| first.number),
		}
	}

	/// Get the header queue info.
	pub fn queue_info(&self) -> queue::QueueInfo {
		self.queue.queue_info()
	}

	/// Attempt to get a block hash by block id.
	pub fn block_hash(&self, id: BlockId) -> Option<H256> {
		self.chain.block_hash(id)
	}

	/// Get a block header by Id.
	pub fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		self.chain.block_header(id)
	}

	/// Get the best block header.
	pub fn best_block_header(&self) -> encoded::Header {
		self.chain.best_header()
	}

	/// Get a block's chain score.
	pub fn score(&self, id: BlockId) -> Option<U256> {
		self.chain.score(id)
	}

	/// Get an iterator over a block and its ancestry.
	pub fn ancestry_iter(&self, start: BlockId) -> AncestryIter {
		self.chain.ancestry_iter(start)
	}

	/// Get the signing network id.
	pub fn signing_network_id(&self) -> Option<u64> {
		self.engine.signing_network_id(&self.latest_env_info())
	}

	/// Flush the header queue.
	pub fn flush_queue(&self) {
		self.queue.flush()
	}

	/// Get the `i`th CHT root.
	pub fn cht_root(&self, i: usize) -> Option<H256> {
		self.chain.cht_root(i)
	}

	/// Import a set of pre-verified headers from the queue.
	pub fn import_verified(&self) {
		const MAX: usize = 256;

		let _lock = self.import_lock.lock();

		let mut bad = Vec::new();
		let mut good = Vec::new();
		for verified_header in self.queue.drain(MAX) {
			let (num, hash) = (verified_header.number(), verified_header.hash());

			let mut tx = self.db.transaction();
			let pending = match self.chain.insert(&mut tx, verified_header) {
				Ok(pending) => {
					good.push(hash);
					self.report.write().blocks_imported += 1;
					pending
				}
				Err(e) => {
					debug!(target: "client", "Error importing header {:?}: {}", (num, hash), e);
					bad.push(hash);
					break;
				}
			};
			self.db.write_buffered(tx);
			self.chain.apply_pending(pending);
			if let Err(e) = self.db.flush() {
				panic!("Database flush failed: {}. Check disk health and space.", e);
			}
		}

		self.queue.mark_as_bad(&bad);
		self.queue.mark_as_good(&good);
	}

	/// Get a report about blocks imported.
	pub fn report(&self) -> ClientReport {
		::std::mem::replace(&mut *self.report.write(), ClientReport::default())
	}

	/// Get blockchain mem usage in bytes.
	pub fn chain_mem_used(&self) -> usize {
		use util::HeapSizeOf;

		self.chain.heap_size_of_children()
	}

	/// Get a handle to the verification engine.
	pub fn engine(&self) -> &Arc<Engine> {
		&self.engine
	}

	/// Get the latest environment info.
	pub fn latest_env_info(&self) -> EnvInfo {
		self.env_info(BlockId::Latest)
			.expect("Best block header and recent hashes always stored; qed")
	}

	/// Get environment info for a given block.
	pub fn env_info(&self, id: BlockId) -> Option<EnvInfo> {
		let header = match self.block_header(id) {
			Some(hdr) => hdr,
			None => return None,
		};

		Some(EnvInfo {
			number: header.number(),
			author: header.author(),
			timestamp: header.timestamp(),
			difficulty: header.difficulty(),
			last_hashes: self.build_last_hashes(header.parent_hash()),
			gas_used: Default::default(),
			gas_limit: header.gas_limit(),
		})
	}

	fn build_last_hashes(&self, mut parent_hash: H256) -> Arc<Vec<H256>> {
		let mut v = Vec::with_capacity(256);
		for _ in 0..255 {
			v.push(parent_hash);
			match self.block_header(BlockId::Hash(parent_hash)) {
				Some(header) => parent_hash = header.hash(),
				None => break,
			}
		}

		Arc::new(v)
	}
}

impl LightChainClient for Client {
	fn chain_info(&self) -> BlockChainInfo { Client::chain_info(self) }

	fn queue_header(&self, header: Header) -> Result<H256, BlockImportError> {
		self.import_header(header)
	}

	fn block_hash(&self, id: BlockId) -> Option<H256> {
		Client::block_hash(self, id)
	}

	fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		Client::block_header(self, id)
	}

	fn best_block_header(&self) -> encoded::Header {
		Client::best_block_header(self)
	}

	fn score(&self, id: BlockId) -> Option<U256> {
		Client::score(self, id)
	}

	fn ancestry_iter<'a>(&'a self, start: BlockId) -> Box<Iterator<Item=encoded::Header> + 'a> {
		Box::new(Client::ancestry_iter(self, start))
	}

	fn signing_network_id(&self) -> Option<u64> {
		Client::signing_network_id(self)
	}

	fn is_known(&self, hash: &H256) -> bool {
		self.status(hash) == BlockStatus::InChain
	}

	fn clear_queue(&self) {
		self.queue.clear()
	}

	fn flush_queue(&self) {
		Client::flush_queue(self);
	}

	fn queue_info(&self) -> queue::QueueInfo {
		self.queue.queue_info()
	}

	fn cht_root(&self, i: usize) -> Option<H256> {
		Client::cht_root(self, i)
	}
}
