//! Web3 rpc interface.
use std::sync::Arc;
use rpc::jsonrpc_core::*;

/// Web3 rpc interface.
pub trait Web3: Sized + Send + Sync + 'static {
	/// Returns current client version.
	fn client_version(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("web3_clientVersion", Web3::client_version);
		delegate
	}
}
