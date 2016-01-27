//! Eth rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Eth rpc interface.
pub trait Eth: Sized + Send + Sync + 'static {
	/// Returns protocol version.
	fn protocol_version(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns the number of hashes per second that the node is mining with.
	fn hashrate(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns block author.
	fn author(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns true if client is actively mining new blocks.
	fn is_mining(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns current gas_price.
	fn gas_price(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns accounts list.
	fn accounts(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns highest block number.
	fn block_number(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns balance of the given account.
	fn balance(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns content of the storage at given address.
	fn storage_at(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns block with given index / hash.
	fn block(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }
	
	/// Returns the number of transactions sent from given address at given time (block number).
	fn transaction_count(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns the number of transactions in a block.
	fn block_transaction_count(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns the number of uncles in a given block.
	fn block_uncles_count(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns the code at given address at given time (block number).
	fn code_at(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Sends transaction.
	fn send_transaction(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Call contract.
	fn call(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Estimate gas needed for execution of given contract.
	fn estimate_gas(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns transaction at given block and index.
	fn transaction_at(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns transaction receipt.
	fn transaction_receipt(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns an uncles at given block and index.
	fn uncle_at(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns available compilers.
	fn compilers(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Compiles lll code.
	fn compile_lll(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Compiles solidity.
	fn compile_solidity(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Compiles serpent.
	fn compile_serpent(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns logs matching given filter object.
	fn logs(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns the hash of the current block, the seedHash, and the boundary condition to be met.
	fn work(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Used for submitting a proof-of-work solution.
	fn submit_work(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Used for submitting mining hashrate.
	fn submit_hashrate(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("eth_protocolVersion", Eth::protocol_version);
		delegate.add_method("eth_hashrate", Eth::hashrate);
		delegate.add_method("eth_coinbase", Eth::author);
		delegate.add_method("eth_mining", Eth::is_mining);
		delegate.add_method("eth_gasPrice", Eth::gas_price);
		delegate.add_method("eth_accounts", Eth::accounts);
		delegate.add_method("eth_blockNumber", Eth::block_number);
		delegate.add_method("eth_balance", Eth::balance);
		delegate.add_method("eth_getStorageAt", Eth::storage_at);
		delegate.add_method("eth_getTransactionCount", Eth::transaction_count);
		delegate.add_method("eth_getBlockTransactionCountByHash", Eth::block_transaction_count);
		delegate.add_method("eth_getBlockTransactionCountByNumber", Eth::block_transaction_count);
		delegate.add_method("eth_getUncleCountByBlockHash", Eth::block_uncles_count);
		delegate.add_method("eth_getUncleCountByBlockNumber", Eth::block_uncles_count);
		delegate.add_method("eth_code", Eth::code_at);
		delegate.add_method("eth_sendTransaction", Eth::send_transaction);
		delegate.add_method("eth_call", Eth::call);
		delegate.add_method("eth_estimateGas", Eth::estimate_gas);
		delegate.add_method("eth_getBlockByHash", Eth::block);
		delegate.add_method("eth_getBlockByNumber", Eth::block);
		delegate.add_method("eth_getTransactionByBlockHashAndIndex", Eth::transaction_at);
		delegate.add_method("eth_getTransactionByBlockNumberAndIndex", Eth::transaction_at);
		delegate.add_method("eth_getTransactionReceipt", Eth::transaction_receipt);
		delegate.add_method("eth_getUncleByBlockHashAndIndex", Eth::uncle_at);
		delegate.add_method("eth_getUncleByBlockNumberAndIndex", Eth::uncle_at);
		delegate.add_method("eth_getCompilers", Eth::compilers);
		delegate.add_method("eth_compileLLL", Eth::compile_lll);
		delegate.add_method("eth_compileSolidity", Eth::compile_solidity);
		delegate.add_method("eth_compileSerpent", Eth::compile_serpent);
		delegate.add_method("eth_getLogs", Eth::logs);
		delegate.add_method("eth_getWork", Eth::work);
		delegate.add_method("eth_submitWork", Eth::submit_work);
		delegate.add_method("eth_submitHashrate", Eth::submit_hashrate);
		delegate
	}
}

/// Eth filters rpc api (polling).
// TODO: do filters api properly
pub trait EthFilter: Sized + Send + Sync + 'static {
	/// Returns id of new block filter
	fn new_block_filter(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns id of new block filter
	fn new_pending_transaction_filter(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns filter changes since last poll
	fn filter_changes(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("eth_newBlockFilter", EthFilter::new_block_filter);
		delegate.add_method("eth_newPendingTransactionFilter", EthFilter::new_pending_transaction_filter);
		delegate.add_method("eth_getFilterChanges", EthFilter::filter_changes);
		delegate
	}
}
