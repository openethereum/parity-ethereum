use bytes::Bytes;
use ethereum_types::Address;
use types::ids::BlockId;

/// Provides `call_contract` method
pub trait CallContract {
	/// Like `call`, but with various defaults. Designed to be used for calling contracts.
	fn call_contract(&self, id: BlockId, address: Address, data: Bytes) -> Result<Bytes, String>;
}
