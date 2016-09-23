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

//! Eth rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

use v1::types::{Block, BlockNumber, Bytes, CallRequest, Filter, Index};
use v1::types::{Log, Receipt, SyncStatus, Transaction, Work};
use v1::types::{H64, H160, H256, U256};

use v1::helpers::auto_args::{Trailing, Wrap};

build_rpc_trait! {
	/// Eth rpc interface.
	pub trait Eth {
		/// Returns protocol version.
		#[name("eth_protocolVersion")]
		fn protocol_version(&self) -> Result<u8, Error>;

		/// Returns an object with data about the sync status or false. (wtf?)
		#[name("eth_syncing")]
		fn syncing(&self) -> Result<SyncStatus, Error>;

		/// Returns the number of hashes per second that the node is mining with.
		#[name("eth_hashrate")]
		fn hashrate(&self) -> Result<U256, Error>;

		/// Returns block author.
		#[name("eth_getAuthor")]
		fn author(&self) -> Result<H160, Error>;

		/// Returns true if client is actively mining new blocks.
		#[name("eth_mining")]
		fn is_mining(&self) -> Result<bool, Error>;

		/// Returns current gas_price.
		#[name("eth_gasPrice")]
		fn gas_price(&self) -> Result<U256, Error>;

		/// Returns accounts list.
		#[name("eth_accounts")]
		fn accounts(&self) -> Result<Vec<H160>, Error>;

		/// Returns highest block number.
		#[name("eth_blockNumber")]
		fn block_number(&self) -> Result<U256, Error>;

		/// Returns balance of the given account.
		#[name("eth_getBalance")]
		fn balance(&self, H160, Trailing<BlockNumber>) -> Result<U256, Error>;

		/// Returns content of the storage at given address.
		#[name("eth_getStorageAt")]
		fn storage_at(&self, H160, U256, Trailing<BlockNumber>) -> Result<H256, Error>;

		/// Returns block with given hash.
		#[name("eth_getBlockByHash")]
		fn block_by_hash(&self, H256, bool) -> Result<Option<Block>, Error>;

		/// Returns block with given number.
		#[name("eth_getBlockByNumber")]
		fn block_by_number(&self, BlockNumber, bool) -> Result<Option<Block>, Error>;

		/// Returns the number of transactions sent from given address at given time (block number).
		#[name("eth_getTransactionCount")]
		fn transaction_count(&self, H160, Trailing<BlockNumber>) -> Result<U256, Error>;

		/// Returns the number of transactions in a block with given hash.
		#[name("eth_getBlockTransactionCountByHash")]
		fn block_transaction_count_by_hash(&self, H256) -> Result<Option<U256>, Error>;

		/// Returns the number of transactions in a block with given block number.
		#[name("eth_getBlockTransactionCountByNumber")]
		fn block_transaction_count_by_number(&self, BlockNumber) -> Result<Option<U256>, Error>;

		/// Returns the number of uncles in a block with given hash.
		#[name("eth_getUncleCountByBlockHash")]
		fn block_uncles_count_by_hash(&self, H256) -> Result<Option<U256>, Error>;

		/// Returns the number of uncles in a block with given block number.
		#[name("eth_getUncleCountByBlockNumber")]
		fn block_uncles_count_by_number(&self, BlockNumber) -> Result<Option<U256>, Error>;

		/// Returns the code at given address at given time (block number).
		#[name("eth_getCode")]
		fn code_at(&self, H160, Trailing<BlockNumber>) -> Result<Bytes, Error>;

		/// Sends signed transaction, returning its hash.
		#[name("eth_sendRawTransaction")]
		fn send_raw_transaction(&self, Bytes) -> Result<H256, Error>;

		/// Call contract, returning the output data.
		#[name("eth_call")]
		fn call(&self, CallRequest, Trailing<BlockNumber>) -> Result<Bytes, Error>;

		/// Estimate gas needed for execution of given contract.
		#[name("eth_estimateGas")]
		fn estimate_gas(&self, CallRequest, Trailing<BlockNumber>) -> Result<U256, Error>;

		/// Get transaction by its hash.
		#[name("eth_getTransactionByHash")]
		fn transaction_by_hash(&self, H256) -> Result<Option<Transaction>, Error>;

		/// Returns transaction at given block hash and index.
		#[name("eth_getTransactionByBlockHashAndIndex")]
		fn transaction_by_block_hash_and_index(&self, H256, Index) -> Result<Option<Transaction>, Error>;

		/// Returns transaction by given block number and index.
		#[name("eth_getTransactionByBlockNumberAndIndex")]
		fn transaction_by_block_number_and_index(&self, BlockNumber, Index) -> Result<Option<Transaction>, Error>;

		/// Returns transaction receipt.
		#[name("eth_getTransactionReceipt")]
		fn transaction_receipt(&self, H256) -> Result<Option<Receipt>, Error>;

		/// Returns an uncles at given block and index.
		#[name("eth_getUncleByBlockHashAndIndex")]
		fn uncle_by_block_hash_and_index(&self, H256, Index) -> Result<Option<Block>, Error>;

		/// Returns an uncles at given block and index.
		#[name("eth_getUncleByBlockNumberAndIndex")]
		fn uncle_by_block_number_and_index(&self, BlockNumber, Index) -> Result<Option<Block>, Error>;

		/// Returns available compilers.
		#[name("eth_getCompilers")]
		fn compilers(&self) -> Result<Vec<String>, Error>;

		/// Compiles lll code.
		#[name("eth_compileLLL")]
		fn compile_lll(&self, String) -> Result<Bytes, Error>;

		/// Compiles solidity.
		#[name("eth_compileSolidity")]
		fn compile_solidity(&self, String) -> Result<Bytes, Error>;

		/// Compiles serpent.
		#[name("eth_compileSerpent")]
		fn compile_serpent(&self, String) -> Result<Bytes, Error>;

		/// Returns logs matching given filter object.
		#[name("eth_getLogs")]
		fn logs(&self, Filter) -> Result<Vec<Log>, Error>;

		/// Returns the hash of the current block, the seedHash, and the boundary condition to be met.
		#[name("eth_getWork")]
		fn work(&self, Trailing<u64>) -> Result<Work, Error>;

		/// Used for submitting a proof-of-work solution.
		#[name("eth_submitWork")]
		fn submit_work(&self, H64, H256, H256) -> Result<bool, Error>;

		/// Used for submitting mining hashrate.
		#[name("eth_submitHashrate")]
		fn submit_hashrate(&self, U256, H256) -> Result<bool, Error>;
	}
}

/// Eth filters rpc api (polling).
// TODO: do filters api properly
pub trait EthFilter: Sized + Send + Sync + 'static {
	/// Returns id of new filter.
	fn new_filter(&self, _: Params) -> Result<Value, Error>;

	/// Returns id of new block filter.
	fn new_block_filter(&self, _: Params) -> Result<Value, Error>;

	/// Returns id of new block filter.
	fn new_pending_transaction_filter(&self, _: Params) -> Result<Value, Error>;

	/// Returns filter changes since last poll.
	fn filter_changes(&self, _: Params) -> Result<Value, Error>;

	/// Returns all logs matching given filter (in a range 'from' - 'to').
	fn filter_logs(&self, _: Params) -> Result<Value, Error>;

	/// Uninstalls filter.
	fn uninstall_filter(&self, _: Params) -> Result<Value, Error>;

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("eth_newFilter", EthFilter::new_filter);
		delegate.add_method("eth_newBlockFilter", EthFilter::new_block_filter);
		delegate.add_method("eth_newPendingTransactionFilter", EthFilter::new_pending_transaction_filter);
		delegate.add_method("eth_getFilterChanges", EthFilter::filter_changes);
		delegate.add_method("eth_getFilterLogs", EthFilter::filter_logs);
		delegate.add_method("eth_uninstallFilter", EthFilter::uninstall_filter);
		delegate
	}
}

/// Signing methods implementation relying on unlocked accounts.
pub trait EthSigning: Sized + Send + Sync + 'static {
	/// Signs the data with given address signature.
	fn sign(&self, _: Params, _: Ready);

	/// Posts sign request asynchronously.
	/// Will return a confirmation ID for later use with check_transaction.
	fn post_sign(&self, _: Params) -> Result<Value, Error>;

	/// Sends transaction; will block for 20s to try to return the
	/// transaction hash.
	/// If it cannot yet be signed, it will return a transaction ID for
	/// later use with check_transaction.
	fn send_transaction(&self, _: Params, _: Ready);

	/// Posts transaction asynchronously.
	/// Will return a transaction ID for later use with check_transaction.
	fn post_transaction(&self, _: Params) -> Result<Value, Error>;

	/// Checks the progress of a previously posted request (transaction/sign).
	/// Should be given a valid send_transaction ID.
	/// Returns the transaction hash, the zero hash (not yet available),
	/// or the signature,
	/// or an error.
	fn check_request(&self, _: Params) -> Result<Value, Error>;

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_async_method("eth_sign", EthSigning::sign);
		delegate.add_async_method("eth_sendTransaction", EthSigning::send_transaction);
		delegate.add_method("eth_postSign", EthSigning::post_sign);
		delegate.add_method("eth_postTransaction", EthSigning::post_transaction);
		delegate.add_method("eth_checkRequest", EthSigning::check_request);
		delegate
	}
}
