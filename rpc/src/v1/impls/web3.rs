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
			Params::None => Ok(Value::String(format!("parity/0.9.0/{}/rust1.8-nightly", Target::os()).to_owned())),
			_ => Err(Error::invalid_params())
		}
	}
}
