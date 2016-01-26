//! Eth rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Eth rpc interface.
pub trait Eth: Sized + Send + Sync + 'static {
	/// Returns protocol version.
	fn protocol_version(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns block author.
	fn author(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns current gas_price.
	fn gas_price(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns highest block number.
	fn block_number(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns block with given index / hash.
	fn block(&self, _: Params) -> Result<Value, Error> { rpcerr!() }
	
	/// Returns true if client is actively mining new blocks.
	fn is_mining(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns the number of hashes per second that the node is mining with.
	fn hashrate(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns the number of transactions in a block.
	fn block_transaction_count(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("eth_protocolVersion", Eth::protocol_version);
		delegate.add_method("eth_coinbase", Eth::author);
		delegate.add_method("eth_gasPrice", Eth::gas_price);
		delegate.add_method("eth_blockNumber", Eth::block_number);
		delegate.add_method("eth_getBlockByNumber", Eth::block);
		delegate.add_method("eth_mining", Eth::is_mining);
		delegate.add_method("eth_hashrate", Eth::hashrate);
		delegate.add_method("eth_getBlockTransactionCountByNumber", Eth::block_transaction_count);
		delegate
	}
}

// TODO: do filters api properly if we commit outselves to polling again...
pub trait EthFilter: Sized + Send + Sync + 'static {
	/// Returns id of new block filter
	fn new_block_filter(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns id of new block filter
	fn new_pending_transaction_filter(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns filter changes since last poll
	fn filter_changes(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("eth_newBlockFilter", EthFilter::new_block_filter);
		delegate.add_method("eth_newPendingTransactionFilter", EthFilter::new_pending_transaction_filter);
		delegate.add_method("eth_getFilterChanges", EthFilter::filter_changes);
		delegate
	}
}
