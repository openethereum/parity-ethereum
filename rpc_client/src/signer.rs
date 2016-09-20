
use client::{Rpc, RpcError};
use rpc::v1::types::{ConfirmationRequest,
					 ConfirmationPayload,
					 TransactionModification,
					 U256};
use serde_json::{Value as JsonValue, to_value};
use std::path::PathBuf;
use futures::{BoxFuture, Canceled};

pub struct SignerRpc {
	rpc: Rpc,
}

impl SignerRpc {
	pub fn new(url: &str, authfile: &PathBuf) -> Result<Self, RpcError> {
		match Rpc::new(&url, authfile) {
			Ok(rpc) => Ok(SignerRpc { rpc: rpc }),
			Err(e) => Err(e),
		}
	}
	pub fn requests_to_confirm(&mut self) ->
		BoxFuture<Result<Vec<ConfirmationRequest>, RpcError>, Canceled>
	{
		self.rpc.request::<Vec<ConfirmationRequest>>
			("personal_requestsToConfirm", vec![])
	}
	pub fn confirm_request(&mut self,
						   id: U256,
						   new_gas_price: Option<U256>,
						   pwd: &str) ->
		BoxFuture<Result<U256, RpcError>, Canceled>
	{
		self.rpc.request::<U256>("personal_confirmRequest", vec![
			to_value(&format!("{:#x}", id)),
			to_value(&TransactionModification { gas_price: new_gas_price }),
			to_value(&pwd),
		])
	}
	pub fn reject_request(&mut self, id: U256) ->
		BoxFuture<Result<bool, RpcError>, Canceled>
	{
		self.rpc.request::<bool>("personal_rejectRequest", vec![
			JsonValue::String(format!("{:#x}", id))
		])
	}
}
