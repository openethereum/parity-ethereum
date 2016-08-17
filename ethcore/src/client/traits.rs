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

use std::mem;
use std::collections::{BTreeMap, VecDeque};
use util::{U256, Address, H256, H2048, Bytes, Itertools};
use blockchain::TreeRoute;
use block_queue::BlockQueueInfo;
use block::{OpenBlock, SealedBlock};
use header::{BlockNumber};
use transaction::{LocalizedTransaction, SignedTransaction};
use log_entry::LocalizedLogEntry;
use filter::Filter;
use views::{BlockView};
use error::{ImportResult, CallError};
use receipt::LocalizedReceipt;
use trace::LocalizedTrace;
use evm::Factory as EvmFactory;
use types::ids::*;
use types::trace_filter::Filter as TraceFilter;
use executive::Executed;
use env_info::LastHashes;
use types::call_analytics::CallAnalytics;
use block_import_error::BlockImportError;
use ipc::IpcConfig;
use types::blockchain_info::BlockChainInfo;
use types::block_status::BlockStatus;

#[derive(Ipc)]
#[ipc(client_ident="RemoteClient")]
/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient : Sync + Send {

	/// Should be called by any external-facing interface when actively using the client.
	/// To minimise chatter, there's no need to call more than once every 30s.
	fn keep_alive(&self) {}

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

	/// Get address code at given block's state.
	fn code(&self, address: &Address, id: BlockID) -> Option<Option<Bytes>>;

	/// Get address code at the latest block's state.
	fn latest_code(&self, address: &Address) -> Option<Bytes> {
		self.code(address, BlockID::Latest)
			.expect("code will return Some if given BlockID::Latest; qed")
	}

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
	fn uncle(&self, id: UncleID) -> Option<Bytes>;

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
	fn import_block(&self, bytes: Bytes) -> Result<H256, BlockImportError>;

	/// Get block queue information.
	fn queue_info(&self) -> BlockQueueInfo;

	/// Clear block queue and abort all import activity.
	fn clear_queue(&self);

	/// Get blockchain information.
	fn chain_info(&self) -> BlockChainInfo;

	/// Get the registrar address, if it exists.
	fn additional_params(&self) -> BTreeMap<String, String>;

	/// Get the best block header.
	fn best_block_header(&self) -> Bytes;

	/// Returns numbers of blocks containing given bloom.
	fn blocks_with_bloom(&self, bloom: &H2048, from_block: BlockID, to_block: BlockID) -> Option<Vec<BlockNumber>>;

	/// Returns logs matching given filter.
	fn logs(&self, filter: Filter) -> Vec<LocalizedLogEntry>;

	/// Makes a non-persistent transaction call.
	fn call(&self, t: &SignedTransaction, block: BlockID, analytics: CallAnalytics) -> Result<Executed, CallError>;

	/// Replays a given transaction for inspection.
	fn replay(&self, t: TransactionID, analytics: CallAnalytics) -> Result<Executed, CallError>;

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

	/// Queue transactions for importing.
	fn queue_transactions(&self, transactions: Vec<Bytes>);

	/// list all transactions
	fn pending_transactions(&self) -> Vec<SignedTransaction>;

	/// Get the gas price distribution.
	fn gas_price_statistics(&self, sample_size: usize, distribution_size: usize) -> Result<Vec<U256>, ()> {
		let mut h = self.chain_info().best_block_hash;
		let mut corpus = Vec::new();
		for _ in 0..sample_size {
			let block_bytes = self.block(BlockID::Hash(h)).expect("h is either the best_block_hash or an ancestor; qed");
			let block = BlockView::new(&block_bytes);
			let header = block.header_view();
			if header.number() == 0 {
				break;
			}
			block.transaction_views().iter().foreach(|t| corpus.push(t.gas_price()));
			h = header.parent_hash().clone();
		}
		corpus.sort();
		let n = corpus.len();
		if n > 0 {
			Ok((0..(distribution_size + 1))
				.map(|i| corpus[i * (n - 1) / distribution_size])
				.collect::<Vec<_>>()
			)
		} else {
			Err(())
		}
	}
}

/// Extended client interface used for mining
pub trait MiningBlockChainClient : BlockChainClient {
	/// Returns OpenBlock prepared for closing.
	fn prepare_open_block(&self, author: Address, gas_range_target: (U256, U256), extra_data: Bytes)
		-> OpenBlock;

	/// Returns EvmFactory.
	fn vm_factory(&self) -> &EvmFactory;

	/// Import sealed block. Skips all verifications.
	fn import_sealed_block(&self, block: SealedBlock) -> ImportResult;
}

impl IpcConfig for BlockChainClient { }
