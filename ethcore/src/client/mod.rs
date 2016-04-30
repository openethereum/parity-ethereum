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

//! Blockchain database client.

mod client;
mod config;
mod ids;
mod test_client;
mod trace;

pub use self::client::*;
pub use self::config::{ClientConfig, BlockQueueConfig, BlockChainConfig, Switch};
pub use self::ids::{BlockId, TransactionId, UncleId, TraceId};
pub use self::test_client::{TestBlockChainClient, EachBlockWith};
pub use self::trace::Filter as TraceFilter;
pub use executive::{Executed, Executive, TransactOptions};
pub use env_info::{LastHashes, EnvInfo};

use std::collections::HashSet;
use util::bytes::Bytes;
use util::hash::{Address, H256, H2048};
use util::numbers::U256;
use blockchain::TreeRoute;
use block_queue::BlockQueueInfo;
use block::{ClosedBlock, LockedBlock, SealedBlock};
use header::{BlockNumber, Header};
use transaction::{LocalizedTransaction, SignedTransaction};
use log_entry::LocalizedLogEntry;
use filter::Filter;
use error::{ImportResult, Error};
use receipt::LocalizedReceipt;
use engine::{Engine};
use trace::LocalizedTrace;

/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient : Sync + Send {
	/// Get raw block header data by block id.
	fn block_header(&self, id: BlockId) -> Option<Bytes>;

	/// Get raw block body data by block id.
	/// Block body is an RLP list of two items: uncles and transactions.
	fn block_body(&self, id: BlockId) -> Option<Bytes>;

	/// Get raw block data by block header hash.
	fn block(&self, id: BlockId) -> Option<Bytes>;

	/// Get block status by block header hash.
	fn block_status(&self, id: BlockId) -> BlockStatus;

	/// Get block total difficulty.
	fn block_total_difficulty(&self, id: BlockId) -> Option<U256>;

	/// Get address nonce.
	fn nonce(&self, address: &Address) -> U256;

	/// Get block hash.
	fn block_hash(&self, id: BlockId) -> Option<H256>;

	/// Get address code.
	fn code(&self, address: &Address) -> Option<Bytes>;

	/// Get address balance.
	fn balance(&self, address: &Address) -> U256;

	/// Get value of the storage at given position.
	fn storage_at(&self, address: &Address, position: &H256) -> H256;

	/// Get transaction with given hash.
	fn transaction(&self, id: TransactionId) -> Option<LocalizedTransaction>;

	/// Get uncle with given id.
	fn uncle(&self, id: UncleId) -> Option<Header>;

	/// Get transaction receipt with given hash.
	fn transaction_receipt(&self, id: TransactionId) -> Option<LocalizedReceipt>;

	/// Get a tree route between `from` and `to`.
	/// See `BlockChain::tree_route`.
	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute>;

	/// Get latest state node
	fn state_data(&self, hash: &H256) -> Option<Bytes>;

	/// Get raw block receipts data by block header hash.
	fn block_receipts(&self, hash: &H256) -> Option<Bytes>;

	/// Import a block into the blockchain.
	fn import_block(&self, bytes: Bytes) -> ImportResult;

	/// Get block queue information.
	fn queue_info(&self) -> BlockQueueInfo;

	/// Clear block queue and abort all import activity.
	fn clear_queue(&self);

	/// Get blockchain information.
	fn chain_info(&self) -> BlockChainInfo;

	/// Get the best block header.
	fn best_block_header(&self) -> Bytes {
		// TODO: lock blockchain only once
		self.block_header(BlockId::Hash(self.chain_info().best_block_hash)).unwrap()
	}

	/// Returns numbers of blocks containing given bloom.
	fn blocks_with_bloom(&self, bloom: &H2048, from_block: BlockId, to_block: BlockId) -> Option<Vec<BlockNumber>>;

	/// Returns logs matching given filter.
	fn logs(&self, filter: Filter) -> Vec<LocalizedLogEntry>;

	// TODO [todr] Should be moved to miner crate eventually.
	/// Returns ClosedBlock prepared for sealing.
	fn prepare_sealing(&self, author: Address, gas_floor_target: U256, extra_data: Bytes, transactions: Vec<SignedTransaction>)
		-> (Option<ClosedBlock>, HashSet<H256>);

	// TODO [todr] Should be moved to miner crate eventually.
	/// Attempts to seal given block. Returns `SealedBlock` on success and the same block in case of error.
	fn try_seal(&self, block: LockedBlock, seal: Vec<Bytes>) -> Result<SealedBlock, LockedBlock>;

	/// Makes a non-persistent transaction call.
	fn call(&self, t: &SignedTransaction) -> Result<Executed, Error>;

	/// Executes a function providing it with a reference to an engine.
	fn engine(&self) -> &Engine;

	/// Returns traces matching given filter.
	fn filter_traces(&self, filter: TraceFilter) -> Option<Vec<LocalizedTrace>>;

	/// Returns trace with given id.
	fn trace(&self, trace: TraceId) -> Option<LocalizedTrace>;

	/// Returns traces created by transaction.
	fn transaction_traces(&self, trace: TransactionId) -> Option<Vec<LocalizedTrace>>;

	/// Returns traces created by transaction from block.
	fn block_traces(&self, trace: BlockId) -> Option<Vec<LocalizedTrace>>;

	/// Get last hashes starting from best block.
	fn last_hashes(&self) -> LastHashes;
}

