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

use std::collections::BTreeMap;
use itertools::Itertools;

use block::{OpenBlock, SealedBlock, ClosedBlock};
use blockchain::TreeRoute;
use encoded;
use vm::LastHashes;
use error::{ImportResult, CallError, Error as EthcoreError};
use error::{TransactionImportResult, BlockImportError};
use evm::{Factory as EvmFactory, Schedule};
use executive::Executed;
use filter::Filter;
use header::{BlockNumber};
use ipc::IpcConfig;
use log_entry::LocalizedLogEntry;
use receipt::LocalizedReceipt;
use trace::LocalizedTrace;
use transaction::{LocalizedTransaction, PendingTransaction, SignedTransaction};
use verification::queue::QueueInfo as BlockQueueInfo;

use util::{U256, Address, H256, H2048, Bytes};
use util::hashdb::DBValue;

use types::ids::*;
use types::basic_account::BasicAccount;
use types::trace_filter::Filter as TraceFilter;
use types::call_analytics::CallAnalytics;
use types::blockchain_info::BlockChainInfo;
use types::block_status::BlockStatus;
use types::mode::Mode;
use types::pruning_info::PruningInfo;

#[ipc(client_ident="RemoteClient")]
/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient : Sync + Send {

	/// Get raw block header data by block id.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

	/// Look up the block number for the given block ID.
	fn block_number(&self, id: BlockId) -> Option<BlockNumber>;

	/// Get raw block body data by block id.
	/// Block body is an RLP list of two items: uncles and transactions.
	fn block_body(&self, id: BlockId) -> Option<encoded::Body>;

	/// Get raw block data by block header hash.
	fn block(&self, id: BlockId) -> Option<encoded::Block>;

	/// Get block status by block header hash.
	fn block_status(&self, id: BlockId) -> BlockStatus;

	/// Get block total difficulty.
	fn block_total_difficulty(&self, id: BlockId) -> Option<U256>;

	/// Attempt to get address nonce at given block.
	/// May not fail on BlockId::Latest.
	fn nonce(&self, address: &Address, id: BlockId) -> Option<U256>;

	/// Attempt to get address storage root at given block.
	/// May not fail on BlockId::Latest.
	fn storage_root(&self, address: &Address, id: BlockId) -> Option<H256>;

	/// Get address nonce at the latest block's state.
	fn latest_nonce(&self, address: &Address) -> U256 {
		self.nonce(address, BlockId::Latest)
			.expect("nonce will return Some when given BlockId::Latest. nonce was given BlockId::Latest. \
			Therefore nonce has returned Some; qed")
	}

	/// Get block hash.
	fn block_hash(&self, id: BlockId) -> Option<H256>;

	/// Get address code at given block's state.
	fn code(&self, address: &Address, id: BlockId) -> Option<Option<Bytes>>;

	/// Get address code at the latest block's state.
	fn latest_code(&self, address: &Address) -> Option<Bytes> {
		self.code(address, BlockId::Latest)
			.expect("code will return Some if given BlockId::Latest; qed")
	}

	/// Get address balance at the given block's state.
	///
	/// May not return None if given BlockId::Latest.
	/// Returns None if and only if the block's root hash has been pruned from the DB.
	fn balance(&self, address: &Address, id: BlockId) -> Option<U256>;

	/// Get address balance at the latest block's state.
	fn latest_balance(&self, address: &Address) -> U256 {
		self.balance(address, BlockId::Latest)
			.expect("balance will return Some if given BlockId::Latest. balance was given BlockId::Latest \
			Therefore balance has returned Some; qed")
	}

	/// Get value of the storage at given position at the given block's state.
	///
	/// May not return None if given BlockId::Latest.
	/// Returns None if and only if the block's root hash has been pruned from the DB.
	fn storage_at(&self, address: &Address, position: &H256, id: BlockId) -> Option<H256>;

	/// Get value of the storage at given position at the latest block's state.
	fn latest_storage_at(&self, address: &Address, position: &H256) -> H256 {
		self.storage_at(address, position, BlockId::Latest)
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

	/// Get the hash of block that contains the transaction, if any.
	fn transaction_block(&self, id: TransactionId) -> Option<H256>;

	/// Get uncle with given id.
	fn uncle(&self, id: UncleId) -> Option<encoded::Header>;

	/// Get transaction receipt with given hash.
	fn transaction_receipt(&self, id: TransactionId) -> Option<LocalizedReceipt>;

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

	/// Import a block with transaction receipts. Does no sealing and transaction validation.
	fn import_block_with_receipts(&self, block_bytes: Bytes, receipts_bytes: Bytes) -> Result<H256, BlockImportError>;

	/// Get block queue information.
	fn queue_info(&self) -> BlockQueueInfo;

	/// Clear block queue and abort all import activity.
	fn clear_queue(&self);

	/// Get blockchain information.
	fn chain_info(&self) -> BlockChainInfo;

	/// Get the registrar address, if it exists.
	fn additional_params(&self) -> BTreeMap<String, String>;

	/// Get the best block header.
	fn best_block_header(&self) -> encoded::Header;

	/// Returns numbers of blocks containing given bloom.
	fn blocks_with_bloom(&self, bloom: &H2048, from_block: BlockId, to_block: BlockId) -> Option<Vec<BlockNumber>>;

	/// Returns logs matching given filter.
	fn logs(&self, filter: Filter) -> Vec<LocalizedLogEntry>;

	/// Makes a non-persistent transaction call.
	fn call(&self, tx: &SignedTransaction, analytics: CallAnalytics, block: BlockId) -> Result<Executed, CallError>;

	/// Makes multiple non-persistent but dependent transaction calls.
	/// Returns a vector of successes or a failure if any of the transaction fails.
	fn call_many(&self, txs: &[(SignedTransaction, CallAnalytics)], block: BlockId) -> Result<Vec<Executed>, CallError>;

	/// Estimates how much gas will be necessary for a call.
	fn estimate_gas(&self, t: &SignedTransaction, block: BlockId) -> Result<U256, CallError>;

	/// Replays a given transaction for inspection.
	fn replay(&self, t: TransactionId, analytics: CallAnalytics) -> Result<Executed, CallError>;

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

	/// Queue transactions for importing.
	fn queue_transactions(&self, transactions: Vec<Bytes>, peer_id: usize);

	/// Queue conensus engine message.
	fn queue_consensus_message(&self, message: Bytes);

	/// List all transactions that are allowed into the next block.
	fn ready_transactions(&self) -> Vec<PendingTransaction>;

	/// Sorted list of transaction gas prices from at least last sample_size blocks.
	fn gas_price_corpus(&self, sample_size: usize) -> ::stats::Corpus<U256> {
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
				block.transaction_views().iter().foreach(|t| corpus.push(t.gas_price()));
				h = block.parent_hash().clone();
			}
		}
		corpus.into()
	}

	/// Get the preferred network ID to sign on
	fn signing_network_id(&self) -> Option<u64>;

	/// Get the mode.
	fn mode(&self) -> Mode;

	/// Set the mode.
	fn set_mode(&self, mode: Mode);

	/// Get the chain spec name.
	fn spec_name(&self) -> String;

	/// Set the chain via a spec name.
	fn set_spec_name(&self, spec_name: String);

	/// Disable the client from importing blocks. This cannot be undone in this session and indicates
	/// that a subsystem has reason to believe this executable incapable of syncing the chain.
	fn disable(&self);

	/// Returns engine-related extra info for `BlockId`.
	fn block_extra_info(&self, id: BlockId) -> Option<BTreeMap<String, String>>;

	/// Returns engine-related extra info for `UncleId`.
	fn uncle_extra_info(&self, id: UncleId) -> Option<BTreeMap<String, String>>;

	/// Returns information about pruning/data availability.
	fn pruning_info(&self) -> PruningInfo;

	/// Like `call`, but with various defaults. Designed to be used for calling contracts.
	fn call_contract(&self, id: BlockId, address: Address, data: Bytes) -> Result<Bytes, String>;

	/// Import a transaction: used for misbehaviour reporting.
	fn transact_contract(&self, address: Address, data: Bytes) -> Result<TransactionImportResult, EthcoreError>;

	/// Get the address of the registry itself.
	fn registrar_address(&self) -> Option<Address>;

	/// Get the address of a particular blockchain service, if available.
	fn registry_address(&self, name: String) -> Option<Address>;

	/// Get the EIP-86 transition block number.
	fn eip86_transition(&self) -> u64;
}

impl IpcConfig for BlockChainClient { }

/// Extended client interface used for mining
pub trait MiningBlockChainClient: BlockChainClient {
	/// Returns OpenBlock prepared for closing.
	fn prepare_open_block(&self,
		author: Address,
		gas_range_target: (U256, U256),
		extra_data: Bytes
	) -> OpenBlock;

	/// Reopens an OpenBlock and updates uncles.
	fn reopen_block(&self, block: ClosedBlock) -> OpenBlock;

	/// Returns EvmFactory.
	fn vm_factory(&self) -> &EvmFactory;

	/// Broadcast a block proposal.
	fn broadcast_proposal_block(&self, block: SealedBlock);

	/// Import sealed block. Skips all verifications.
	fn import_sealed_block(&self, block: SealedBlock) -> ImportResult;

	/// Returns latest schedule.
	fn latest_schedule(&self) -> Schedule;
}

/// Client facilities used by internally sealing Engines.
pub trait EngineClient: MiningBlockChainClient {
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
	fn epoch_transition_for(&self, parent_hash: H256) -> Option<::engines::EpochTransition>;
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
}
