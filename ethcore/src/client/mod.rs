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
mod error;
mod test_client;
mod trace;

pub use self::client::*;
pub use self::config::{ClientConfig, BlockQueueConfig, BlockChainConfig, Switch, VMType};
pub use self::error::Error;
pub use types::ids::*;
pub use self::test_client::{TestBlockChainClient, EachBlockWith};
pub use self::trace::Filter as TraceFilter;
pub use executive::{Executed, Executive, TransactOptions};
pub use env_info::{LastHashes, EnvInfo};

use std::path::Path;
use util::bytes::Bytes;
use util::hash::{Address, H256, H2048};
use util::numbers::U256;
use blockchain::TreeRoute;
use block_queue::BlockQueueInfo;
use block::{LockedBlock, SealedBlock, OpenBlock};
use header::{BlockNumber, Header};
use transaction::{LocalizedTransaction, SignedTransaction};
use log_entry::LocalizedLogEntry;
use filter::Filter;
use error::{ImportResult, ExecutionError};
use receipt::LocalizedReceipt;
use trace::LocalizedTrace;
use evm::Factory as EvmFactory;
use miner::{TransactionImportResult};
use error::Error as EthError;

/// Options concerning what analytics we run on the call.
#[derive(Eq, PartialEq, Default, Clone, Copy, Debug)]
pub struct CallAnalytics {
	/// Make a transaction trace.
	pub transaction_tracing: bool,
	/// Make a VM trace.
	pub vm_tracing: bool,
	/// Make a diff.
	pub state_diffing: bool,
}

/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient : Sync + Send {
	/// Get raw block header data by block id.
	fn block_header(&self, id: BlockID) -> Option<Bytes>;

	/// Get raw block body data by block id.
	/// Block body is an RLP list of two items: uncles and transactions.
	fn block_body(&self, id: BlockID) -> Option<Bytes>;

	/// Get raw block data by block header hash.
	fn block(&self, id: BlockID) -> Option<Bytes>;

	/// Get block status by block header hash.
	fn block_status(&self, id: BlockID) -> BlockStatus;

	/// Get block total difficulty.
	fn block_total_difficulty(&self, id: BlockID) -> Option<U256>;

	/// Attempt to get address nonce at given block.
	/// May not fail on BlockID::Latest.
	fn nonce(&self, address: &Address, id: BlockID) -> Option<U256>;

	/// Get address nonce at the latest block's state.
	fn latest_nonce(&self, address: &Address) -> U256 {
		self.nonce(address, BlockID::Latest)
			.expect("nonce will return Some when given BlockID::Latest. nonce was given BlockID::Latest. \
			Therefore nonce has returned Some; qed")
	}

	/// Get block hash.
	fn block_hash(&self, id: BlockID) -> Option<H256>;

	/// Get address code.
	fn code(&self, address: &Address) -> Option<Bytes>;

	/// Get address balance at the given block's state.
	///
	/// May not return None if given BlockID::Latest.
	/// Returns None if and only if the block's root hash has been pruned from the DB.
	fn balance(&self, address: &Address, id: BlockID) -> Option<U256>;

	/// Get address balance at the latest block's state.
	fn latest_balance(&self, address: &Address) -> U256 {
		self.balance(address, BlockID::Latest)
			.expect("balance will return Some if given BlockID::Latest. balance was given BlockID::Latest \
			Therefore balance has returned Some; qed")
	}

	/// Get value of the storage at given position at the given block's state.
	///
	/// May not return None if given BlockID::Latest.
	/// Returns None if and only if the block's root hash has been pruned from the DB.
	fn storage_at(&self, address: &Address, position: &H256, id: BlockID) -> Option<H256>;

	/// Get value of the storage at given position at the latest block's state.
	fn latest_storage_at(&self, address: &Address, position: &H256) -> H256 {
		self.storage_at(address, position, BlockID::Latest)
			.expect("storage_at will return Some if given BlockID::Latest. storage_at was given BlockID::Latest. \
			Therefore storage_at has returned Some; qed")
	}

	/// Get transaction with given hash.
	fn transaction(&self, id: TransactionID) -> Option<LocalizedTransaction>;

	/// Get uncle with given id.
	fn uncle(&self, id: UncleID) -> Option<Header>;

	/// Get transaction receipt with given hash.
	fn transaction_receipt(&self, id: TransactionID) -> Option<LocalizedReceipt>;

	/// Get a tree route between `from` and `to`.
	/// See `BlockChain::tree_route`.
	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute>;

	/// Get all possible uncle hashes for a block.
	fn find_uncles(&self, hash: &H256) -> Option<Vec<H256>>;

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
		self.block_header(BlockID::Hash(self.chain_info().best_block_hash)).unwrap()
	}

	/// Returns numbers of blocks containing given bloom.
	fn blocks_with_bloom(&self, bloom: &H2048, from_block: BlockID, to_block: BlockID) -> Option<Vec<BlockNumber>>;

	/// Returns logs matching given filter.
	fn logs(&self, filter: Filter) -> Vec<LocalizedLogEntry>;

	/// Makes a non-persistent transaction call.
	// TODO: should be able to accept blockchain location for call.
	fn call(&self, t: &SignedTransaction, analytics: CallAnalytics) -> Result<Executed, ExecutionError>;

	/// Returns EvmFactory.
	fn vm_factory(&self) -> &EvmFactory;

	/// Returns traces matching given filter.
	fn filter_traces(&self, filter: TraceFilter) -> Option<Vec<LocalizedTrace>>;

	/// Returns trace with given id.
	fn trace(&self, trace: TraceId) -> Option<LocalizedTrace>;

	/// Returns traces created by transaction.
	fn transaction_traces(&self, trace: TransactionID) -> Option<Vec<LocalizedTrace>>;

	/// Returns traces created by transaction from block.
	fn block_traces(&self, trace: BlockID) -> Option<Vec<LocalizedTrace>>;

	/// Get last hashes starting from best block.
	fn last_hashes(&self) -> LastHashes;

	/// import transactions from network/other 3rd party
	fn import_transactions(&self, transactions: Vec<SignedTransaction>) -> Vec<Result<TransactionImportResult, EthError>>;

	/// list all transactions
	fn all_transactions(&self) -> Vec<SignedTransaction>;

	/// Generate a PV64 snapshot for the current best block, saving it within the
	/// root directory, whose path is given.
	fn take_snapshot(&self, root_dir: &Path);
}

/// Extended client interface used for mining
pub trait MiningBlockChainClient : BlockChainClient {
	/// Attempts to seal given block. Returns `SealedBlock` on success and the same block in case of error.
	fn try_seal(&self, block: LockedBlock, seal: Vec<Bytes>) -> Result<SealedBlock, LockedBlock>;

	/// Returns OpenBlock prepared for closing.
	fn prepare_open_block(&self, author: Address, gas_floor_target: U256, extra_data: Bytes)
		-> OpenBlock;
}
