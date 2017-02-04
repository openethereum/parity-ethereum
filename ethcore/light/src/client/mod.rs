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

use ethcore::block_import_error::BlockImportError;
use ethcore::block_status::BlockStatus;
use ethcore::client::ClientReport;
use ethcore::ids::BlockId;
use ethcore::header::Header;
use ethcore::verification::queue::{self, HeaderQueue};
use ethcore::transaction::{PendingTransaction, Condition as TransactionCondition};
use ethcore::blockchain_info::BlockChainInfo;
use ethcore::spec::Spec;
use ethcore::service::ClientIoMessage;
use ethcore::encoded;
use io::IoChannel;

use util::hash::{H256, H256FastMap};
use util::{Bytes, Mutex, RwLock};

use provider::Provider;
use request;

use self::header_chain::HeaderChain;

pub use self::service::Service;

mod header_chain;
mod service;

/// Configuration for the light client.
#[derive(Debug, Default, Clone)]
pub struct Config {
	/// Verification queue config.
	pub queue: queue::Config,
}

/// Trait for interacting with the header chain abstractly.
pub trait LightChainClient: Send + Sync {
	/// Get chain info.
	fn chain_info(&self) -> BlockChainInfo;

	/// Queue header to be verified. Required that all headers queued have their
	/// parent queued prior.
	fn queue_header(&self, header: Header) -> Result<H256, BlockImportError>;

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

/// Light client implementation.
pub struct Client {
	queue: HeaderQueue,
	chain: HeaderChain,
	tx_pool: Mutex<H256FastMap<PendingTransaction>>,
	report: RwLock<ClientReport>,
	import_lock: Mutex<()>,
}

impl Client {
	/// Create a new `Client`.
	pub fn new(config: Config, spec: &Spec, io_channel: IoChannel<ClientIoMessage>) -> Self {
		Client {
			queue: HeaderQueue::new(config.queue, spec.engine.clone(), io_channel, true),
			chain: HeaderChain::new(&::rlp::encode(&spec.genesis_header())),
			tx_pool: Mutex::new(Default::default()),
			report: RwLock::new(ClientReport::default()),
			import_lock: Mutex::new(()),
		}
	}

	/// Import a header to the queue for additional verification.
	pub fn import_header(&self, header: Header) -> Result<H256, BlockImportError> {
		self.queue.import(header).map_err(Into::into)
	}

	/// Import a local transaction.
	pub fn import_own_transaction(&self, tx: PendingTransaction) {
		self.tx_pool.lock().insert(tx.transaction.hash(), tx);
	}

	/// Fetch a vector of all pending transactions.
	pub fn ready_transactions(&self) -> Vec<PendingTransaction> {
		let best = self.chain.best_header();
		self.tx_pool.lock()
			.values()
			.filter(|t| match t.condition {
				Some(TransactionCondition::Number(x)) => x <= best.number(),
				Some(TransactionCondition::Timestamp(x)) => x <= best.timestamp(),
				None => true,
			})
			.cloned()
			.collect()
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

	/// Get a block header by Id.
	pub fn get_header(&self, id: BlockId) -> Option<encoded::Header> {
		self.chain.get_header(id)
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

			match self.chain.insert(verified_header) {
				Ok(()) => {
					good.push(hash);
					self.report.write().blocks_imported += 1;
				}
				Err(e) => {
					debug!(target: "client", "Error importing header {:?}: {}", (num, hash), e);
					bad.push(hash);
				}
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
}

impl LightChainClient for Client {
	fn chain_info(&self) -> BlockChainInfo { Client::chain_info(self) }

	fn queue_header(&self, header: Header) -> Result<H256, BlockImportError> {
		self.import_header(header)
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

// dummy implementation -- may draw from canonical cache further on.
impl Provider for Client {
	fn chain_info(&self) -> BlockChainInfo {
		Client::chain_info(self)
	}

	fn reorg_depth(&self, _a: &H256, _b: &H256) -> Option<u64> {
		None
	}

	fn earliest_state(&self) -> Option<u64> {
		None
	}

	fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		self.chain.get_header(id)
	}

	fn block_body(&self, _id: BlockId) -> Option<encoded::Body> {
		None
	}

	fn block_receipts(&self, _hash: &H256) -> Option<Bytes> {
		None
	}

	fn state_proof(&self, _req: request::StateProof) -> Vec<Bytes> {
		Vec::new()
	}

	fn contract_code(&self, _req: request::ContractCode) -> Bytes {
		Vec::new()
	}

	fn header_proof(&self, _req: request::HeaderProof) -> Option<(encoded::Header, Vec<Bytes>)> {
		None
	}

	fn ready_transactions(&self) -> Vec<PendingTransaction> {
		Vec::new()
	}
}
