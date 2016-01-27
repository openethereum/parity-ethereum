//! Web3 rpc implementation.
use jsonrpc_core::*;
use traits::Web3;

/// Web3 rpc implementation.
pub struct Web3Client;

impl Web3Client {
	/// Creates new Web3Client.
	pub fn new() -> Self { Web3Client }
}

impl Web3 for Web3Client {
	fn client_version(&self, params: Params) -> Result<Value, Error> {
		match params {
			//Params::None => Ok(Value::String("parity/0.1.0/-/rust1.7-nightly".to_owned())),
			Params::None => Ok(Value::String("surprise/0.1.0/surprise/surprise".to_owned())),
			_ => Err(Error::invalid_params())
		}
	}
}
