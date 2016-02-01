//! Net rpc implementation.
use std::sync::Arc;
use jsonrpc_core::*;
use ethsync::EthSync;
use v1::traits::Net;

/// Net rpc implementation.
pub struct NetClient {
	sync: Arc<EthSync>
}

impl NetClient {
	/// Creates new NetClient.
	pub fn new(sync: Arc<EthSync>) -> Self { 
		NetClient {
			sync: sync
		}
	}
}

impl Net for NetClient {
	fn version(&self, _: Params) -> Result<Value, Error> {
		Ok(Value::U64(self.sync.status().protocol_version as u64))
	}

	fn peer_count(&self, _params: Params) -> Result<Value, Error> {
		Ok(Value::U64(self.sync.status().num_peers as u64))
	}
}
