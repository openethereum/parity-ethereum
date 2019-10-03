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

use std::{
	collections::BTreeMap,
	sync::Arc,
};

use account_state::state::StateInfo;
use blockchain::BlockProvider;
use bytes::Bytes;
use call_contract::CallContract;
use registrar::RegistrarClient;
use common_types::{
	basic_account::BasicAccount,
	block_status::BlockStatus,
	blockchain_info::BlockChainInfo,
	BlockNumber,
	call_analytics::CallAnalytics,
	chain_notify::{NewBlocks, ChainMessageType},
	client_types::Mode,
	encoded,
	engines::{epoch::Transition as EpochTransition, machine::Executed},
	errors::{EthcoreError, EthcoreResult},
	filter::Filter,
	header::Header,
	ids::{BlockId, TransactionId, TraceId, UncleId},
	log_entry::LocalizedLogEntry,
	pruning_info::PruningInfo,
	receipt::LocalizedReceipt,
	trace_filter::Filter as TraceFilter,
	transaction::{self, LocalizedTransaction, CallError, SignedTransaction, UnverifiedTransaction},
	tree_route::TreeRoute,
	verification::{VerificationQueueInfo, Unverified},
};
use ethereum_types::{Address, H256, U256};
use ethcore_db::keys::BlockReceipts;
use ethcore_miner::pool::VerifiedTransaction;
use kvdb::DBValue;
use stats;
use trace::{
	FlatTrace,
	localized::LocalizedTrace,
	VMTrace,
};
use common_types::data_format::DataFormat;
use vm::{LastHashes, Schedule};

/// State information to be used during client query
pub enum StateOrBlock {
	/// State to be used, may be pending
	State(Box<dyn StateInfo>),

	/// Id of an existing block from a chain to get state from
	Block(BlockId)
}

/// Result to be used during get address code at given block's state
// todo[botika] move to `common-types`
pub enum StateResult<T> {
	/// State is missing
	Missing,

	/// State is some
	Some(T),
}

impl From<Box<dyn StateInfo>> for StateOrBlock {
	fn from(info: Box<dyn StateInfo>) -> StateOrBlock {
		StateOrBlock::State(info)
	}
}

impl From<BlockId> for StateOrBlock {
	fn from(id: BlockId) -> StateOrBlock {
		StateOrBlock::Block(id)
	}
}

/// Provides `nonce` and `latest_nonce` methods
pub trait Nonce {
	/// Attempt to get address nonce at given block.
	/// May not fail on BlockId::Latest.
	fn nonce(&self, address: &Address, id: BlockId) -> Option<U256>;

	/// Get address nonce at the latest block's state.
	fn latest_nonce(&self, address: &Address) -> U256 {
		self.nonce(address, BlockId::Latest)
			.expect("nonce will return Some when given BlockId::Latest. nonce was given BlockId::Latest. \
			Therefore nonce has returned Some; qed")
	}
}

/// Provides `balance` and `latest_balance` methods
pub trait Balance {
	/// Get address balance at the given block's state.
	///
	/// May not return None if given BlockId::Latest.
	/// Returns None if and only if the block's root hash has been pruned from the DB.
	fn balance(&self, address: &Address, state: StateOrBlock) -> Option<U256>;

	/// Get address balance at the latest block's state.
	fn latest_balance(&self, address: &Address) -> U256 {
		self.balance(address, BlockId::Latest.into())
			.expect("balance will return Some if given BlockId::Latest. balance was given BlockId::Latest \
			Therefore balance has returned Some; qed")
	}
}

/// Provides methods to access account info
pub trait AccountData: Nonce + Balance {}

/// Provides `chain_info` method
pub trait ChainInfo {
	/// Get blockchain information.
	fn chain_info(&self) -> BlockChainInfo;
}

/// Provides various information on a block by it's ID
pub trait BlockInfo: Send + Sync {
	/// Get raw block header data by block id.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

	/// Get the best block header.
	fn best_block_header(&self) -> Header;

	/// Get raw block data by block header hash.
	fn block(&self, id: BlockId) -> Option<encoded::Block>;

	/// Get address code hash at given block's state.
	fn code_hash(&self, address: &Address, id: BlockId) -> Option<H256>;
}

/// Provides various information on a transaction by it's ID
pub trait TransactionInfo {
	/// Get the hash of block that contains the transaction, if any.
	fn transaction_block(&self, id: TransactionId) -> Option<H256>;
}

/// Provides various blockchain information, like block header, chain state etc.
pub trait BlockChain: ChainInfo + BlockInfo + TransactionInfo {}

/// Client facilities used by internally sealing Engines.
pub trait EngineClient: Sync + Send + ChainInfo {
	/// Make a new block and seal it.
	fn update_sealing(&self);

	/// Submit a seal for a block in the mining queue.
	fn submit_seal(&self, block_hash: H256, seal: Vec<Bytes>);

	/// Broadcast a consensus message to the network.
	fn broadcast_consensus_message(&self, message: Bytes);

	/// Get the transition to the epoch the given parent hash is part of
	/// or transitions to.
	/// This will give the epoch that any children of this parent belong to.
	///
	/// The block corresponding the the parent hash must be stored already.
	fn epoch_transition_for(&self, parent_hash: H256) -> Option<EpochTransition>;

	/// Attempt to cast the engine client to a full client.
	fn as_full_client(&self) -> Option<&dyn BlockChainClient>;

	/// Get a block number by ID.
	fn block_number(&self, id: BlockId) -> Option<BlockNumber>;

	/// Get raw block header data by block id.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;
}

/// Provides methods to import block into blockchain
pub trait ImportBlock {
	/// Import a block into the blockchain.
	fn import_block(&self, block: Unverified) -> EthcoreResult<H256>;

	/// Triggered by a message from a block queue when the block is ready for insertion.
	/// Returns the number of blocks imported.
	fn import_verified_blocks(&self) -> usize;
}

/// IO operations that should off-load heavy work to another thread.
pub trait IoClient: Sync + Send {
	/// Queue transactions for importing.
	fn queue_transactions(&self, transactions: Vec<Bytes>, peer_id: usize);

	/// Queue block import with transaction receipts. Does no sealing and transaction validation.
	fn queue_ancient_block(&self, block_bytes: Unverified, receipts_bytes: Bytes) -> EthcoreResult<H256>;

	/// Queue consensus engine message.
	fn queue_consensus_message(&self, message: Bytes);
}

/// Implement this for clients that need logic to decide when/how to advance.
pub trait Tick {
	/// Tick the client
	fn tick(&self, _prevent_sleep: bool) {}
}

impl Tick for () {}

/// Provides recently seen bad blocks.
pub trait BadBlocks {
	/// Returns a list of blocks that were recently not imported because they were invalid.
	fn bad_blocks(&self) -> Vec<(Unverified, String)>;
}


/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient:
	Sync + Send + AccountData + BlockChain + CallContract + RegistrarClient
	+ ImportBlock + IoClient + BadBlocks
{
	/// Look up the block number for the given block ID.
	fn block_number(&self, id: BlockId) -> Option<BlockNumber>;

	/// Get raw block body data by block id.
	/// Block body is an RLP list of two items: uncles and transactions.
	fn block_body(&self, id: BlockId) -> Option<encoded::Body>;

	/// Get block status by block header hash.
	fn block_status(&self, id: BlockId) -> BlockStatus;

	/// Get block total difficulty.
	fn block_total_difficulty(&self, id: BlockId) -> Option<U256>;

	/// Attempt to get address storage root at given block.
	/// May not fail on BlockId::Latest.
	fn storage_root(&self, address: &Address, id: BlockId) -> Option<H256>;

	/// Get block hash.
	fn block_hash(&self, id: BlockId) -> Option<H256>;

	/// Get address code at given block's state.
	fn code(&self, address: &Address, state: StateOrBlock) -> StateResult<Option<Bytes>>;

	/// Get address code at the latest block's state.
	fn latest_code(&self, address: &Address) -> Option<Bytes> {
		match self.code(address, BlockId::Latest.into()) {
			StateResult::Missing => panic!("code will return Some if given BlockId::Latest; qed"),
			StateResult::Some(t) => t,
		}
	}

	/// Get a reference to the `BlockProvider`.
	fn chain(&self) -> Arc<dyn BlockProvider>;

	/// Get block queue information.
	fn queue_info(&self) -> VerificationQueueInfo;

	/// Get address code hash at given block's state.

	/// Get value of the storage at given position at the given block's state.
	///
	/// May not return None if given BlockId::Latest.
	/// Returns None if and only if the block's root hash has been pruned from the DB.
	fn storage_at(&self, address: &Address, position: &H256, state: StateOrBlock) -> Option<H256>;

	/// Get value of the storage at given position at the latest block's state.
	fn latest_storage_at(&self, address: &Address, position: &H256) -> H256 {
		self.storage_at(address, position, BlockId::Latest.into())
			.expect("storage_at will return Some if given BlockId::Latest. storage_at was given BlockId::Latest. \
			Therefore storage_at has returned Some; qed")
	}

	/// Get a list of all accounts in the block `id`, if fat DB is in operation, otherwise `None`.
	/// If `after` is set the list starts with the following item.
	fn list_accounts(&self, id: BlockId, after: Option<&Address>, count: u64) -> Option<Vec<Address>>;

	/// Get a list of all storage keys in the block `id`, if fat DB is in operation, otherwise `None`.
	/// If `after` is set the list starts with the following item.
	fn list_storage(&self, id: BlockId, account: &Address, after: Option<&H256>, count: u64) -> Option<Vec<H256>>;

	/// Get transaction with given hash.
	fn transaction(&self, id: TransactionId) -> Option<LocalizedTransaction>;

	/// Get uncle with given id.
	fn uncle(&self, id: UncleId) -> Option<encoded::Header>;

	/// Get transaction receipt with given hash.
	fn transaction_receipt(&self, id: TransactionId) -> Option<LocalizedReceipt>;

	/// Get localized receipts for all transaction in given block.
	fn localized_block_receipts(&self, id: BlockId) -> Option<Vec<LocalizedReceipt>>;

	/// Get a tree route between `from` and `to`.
	/// See `BlockChain::tree_route`.
	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute>;

	/// Get all possible uncle hashes for a block.
	fn find_uncles(&self, hash: &H256) -> Option<Vec<H256>>;

	/// Get latest state node
	fn state_data(&self, hash: &H256) -> Option<Bytes>;

	/// Get block receipts data by block header hash.
	fn block_receipts(&self, hash: &H256) -> Option<BlockReceipts>;

	/// Returns true if block queue is empty.
	fn is_queue_empty(&self) -> bool {
		self.queue_info().is_empty()
	}

	/// Clear block queue and abort all import activity.
	fn clear_queue(&self);

	/// Returns logs matching given filter. If one of the filtering block cannot be found, returns the block id that caused the error.
	fn logs(&self, filter: Filter) -> Result<Vec<LocalizedLogEntry>, BlockId>;

	/// Replays a given transaction for inspection.
	fn replay(&self, t: TransactionId, analytics: CallAnalytics) -> Result<Executed<FlatTrace, VMTrace>, CallError>;

	/// Replays all the transactions in a given block for inspection.
	fn replay_block_transactions(&self, block: BlockId, analytics: CallAnalytics) -> Result<Box<dyn Iterator<Item = (H256, Executed<FlatTrace, VMTrace>)>>, CallError>;

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

	/// List all ready transactions that should be propagated to other peers.
	fn transactions_to_propagate(&self) -> Vec<Arc<VerifiedTransaction>>;

	/// Sorted list of transaction gas prices from at least last sample_size blocks.
	fn gas_price_corpus(&self, sample_size: usize) -> stats::Corpus<U256> {
		let mut h = self.chain_info().best_block_hash;
		let mut corpus = Vec::new();
		while corpus.is_empty() {
			for _ in 0..sample_size {
				let block = match self.block(BlockId::Hash(h)) {
					Some(block) => block,
					None => return corpus.into(),
				};

				if block.number() == 0 {
					return corpus.into();
				}
				for t in block.transaction_views().iter() {
					corpus.push( t.gas_price() )
				}
				h = block.parent_hash().clone();
			}
		}
		corpus.into()
	}

	/// Get the preferred chain ID to sign on
	fn signing_chain_id(&self) -> Option<u64>;

	/// Get the mode.
	fn mode(&self) -> Mode;

	/// Set the mode.
	fn set_mode(&self, mode: Mode);

	/// Get the chain spec name.
	fn spec_name(&self) -> String;

	/// Set the chain via a spec name.
	fn set_spec_name(&self, spec_name: String) -> Result<(), ()>;

	/// Disable the client from importing blocks. This cannot be undone in this session and indicates
	/// that a subsystem has reason to believe this executable incapable of syncing the chain.
	fn disable(&self);

	/// Returns engine-related extra info for `BlockId`.
	fn block_extra_info(&self, id: BlockId) -> Option<BTreeMap<String, String>>;

	/// Returns engine-related extra info for `UncleId`.
	fn uncle_extra_info(&self, id: UncleId) -> Option<BTreeMap<String, String>>;

	/// Returns information about pruning/data availability.
	fn pruning_info(&self) -> PruningInfo;

	/// Schedule state-altering transaction to be executed on the next pending block.
	fn transact_contract(&self, address: Address, data: Bytes) -> Result<(), transaction::Error>;
}

/// resets the blockchain
pub trait BlockChainReset {
	/// reset to best_block - n
	fn reset(&self, num: u32) -> Result<(), String>;

	/// Number of eras kept in a journal before they are pruned
	fn pruning_history(&self) -> u64;
}


/// Provides `latest_schedule` method
pub trait ScheduleInfo {
	/// Returns latest schedule.
	fn latest_schedule(&self) -> Schedule;
}

/// Provides methods to access chain state
pub trait StateClient {
	/// Type representing chain state
	type State: StateInfo;

	/// Get a copy of the best block's state.
	fn latest_state(&self) -> Self::State;

	/// Attempt to get a copy of a specific block's final state.
	///
	/// This will not fail if given BlockId::Latest.
	/// Otherwise, this can fail (but may not) if the DB prunes state or the block
	/// is unknown.
	fn state_at(&self, id: BlockId) -> Option<Self::State>;
}

/// Extended client interface for providing proofs of the state.
pub trait ProvingBlockChainClient: BlockChainClient {
	/// Prove account storage at a specific block id.
	///
	/// Both provided keys assume a secure trie.
	/// Returns a vector of raw trie nodes (in order from the root) proving the storage query.
	fn prove_storage(&self, key1: H256, key2: H256, id: BlockId) -> Option<(Vec<Bytes>, H256)>;

	/// Prove account existence at a specific block id.
	/// The key is the keccak hash of the account's address.
	/// Returns a vector of raw trie nodes (in order from the root) proving the query.
	fn prove_account(&self, key1: H256, id: BlockId) -> Option<(Vec<Bytes>, BasicAccount)>;

	/// Prove execution of a transaction at the given block.
	/// Returns the output of the call and a vector of database items necessary
	/// to reproduce it.
	fn prove_transaction(&self, transaction: SignedTransaction, id: BlockId) -> Option<(Bytes, Vec<DBValue>)>;

	/// Get an epoch change signal by block hash.
	fn epoch_signal(&self, hash: H256) -> Option<Vec<u8>>;
}

/// External database restoration handler
pub trait DatabaseRestore: Send + Sync {
	/// Restart with a new backend. Takes ownership of passed database and moves it to a new location.
	fn restore_db(&self, new_db: &str) -> Result<(), EthcoreError>;
}

/// Represents what has to be handled by actor listening to chain events
pub trait ChainNotify: Send + Sync {
	/// fires when chain has new blocks.
	fn new_blocks(&self, _new_blocks: NewBlocks) {
		// does nothing by default
	}

	/// fires when chain achieves active mode
	fn start(&self) {
		// does nothing by default
	}

	/// fires when chain achieves passive mode
	fn stop(&self) {
		// does nothing by default
	}

	/// fires when chain broadcasts a message
	fn broadcast(&self, _message_type: ChainMessageType) {
		// does nothing by default
	}

	/// fires when new block is about to be imported
	/// implementations should be light
	fn block_pre_import(&self, _bytes: &Bytes, _hash: &H256, _difficulty: &U256) {
		// does nothing by default
	}

	/// fires when new transactions are received from a peer
	fn transactions_received(&self, _txs: &[UnverifiedTransaction], _peer_id: usize) {
		// does nothing by default
	}
}

/// Provides a method for importing/exporting blocks
pub trait ImportExportBlocks {
    /// Export blocks to destination, with the given from, to and format argument.
    /// destination could be a file or stdout.
    /// If the format is hex, each block is written on a new line.
    /// For binary exports, all block data is written to the same line.
	fn export_blocks<'a>(
        &self,
        destination: Box<dyn std::io::Write + 'a>,
        from: BlockId,
        to: BlockId,
        format: Option<DataFormat>
    ) -> Result<(), String>;

	/// Import blocks from destination, with the given format argument
	/// Source could be a file or stdout.
	/// For hex format imports, it attempts to read the blocks on a line by line basis.
	/// For binary format imports, reads the 8 byte RLP header in order to decode the block
	/// length to be read.
	fn import_blocks<'a>(
		&self,
		source: Box<dyn std::io::Read + 'a>,
		format: Option<DataFormat>
	) -> Result<(), String>;
}
