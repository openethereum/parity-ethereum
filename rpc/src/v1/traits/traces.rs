

//! Traces specific rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Traces specific rpc interface.
pub trait Traces: Sized + Send + Sync + 'static {
	/// Returns traces matching given filter.
	fn filter(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns transaction trace at given index.
	fn trace(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns all traces of given transaction.
	fn transaction_traces(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns all traces produced at given block.
	fn block_traces(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("trace_filter", Traces::filter);
		delegate.add_method("trace_get", Traces::trace);
		delegate.add_method("trace_transaction", Traces::transaction_traces);
		delegate.add_method("trace_block", Traces::block_traces);
		delegate
	}
}
