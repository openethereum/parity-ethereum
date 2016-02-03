//! Web3 rpc implementation.
use target_info::Target;
use jsonrpc_core::*;
use v1::traits::Web3;

/// Web3 rpc implementation.
pub struct Web3Client;

impl Web3Client {
	/// Creates new Web3Client.
	pub fn new() -> Self { Web3Client }
}

impl Web3 for Web3Client {
	fn client_version(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::String(format!("Parity/{}/{}-{}-{}", env!("CARGO_PKG_VERSION"), Target::arch(), Target::env(), Target::os()))),
			_ => Err(Error::invalid_params())
		}
	}
}
