//! Net rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Net rpc interface.
pub trait Net: Sized + Send + Sync + 'static {
	/// Returns protocol version.
	fn version(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns number of peers connected to node.
	fn peer_count(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("net_version", Net::version);
		delegate.add_method("net_peerCount", Net::peer_count);
		delegate
	}
}
