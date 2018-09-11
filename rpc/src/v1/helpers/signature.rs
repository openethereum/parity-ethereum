use ethkey::{recover, public_to_address, Signature};
use jsonrpc_core::Result;
use v1::types::{Bytes, H520, RichBasicAccount, BasicAccount};
use v1::helpers::errors;
use v1::helpers::dispatch::eth_data_hash;
use hash::keccak;

pub fn verify_signature(is_prefixed: bool, message: Bytes, signature: H520, chain_id: Option<u64>) -> Result<RichBasicAccount> {
	let hash = if is_prefixed {
		eth_data_hash(message.0)
	} else {
		keccak(message.0.clone())
	};

	let signature = Signature::from(signature.0);
	let is_valid_for_current_chain = chain_id.map(|chain_id| {
		let v = signature.v() as u64;
		if v <= 1  {
			return false
		}

		let decoded_v = (v - 35) - (chain_id * 2);
		if decoded_v == 0 || decoded_v == 1 {
			return true
		}
		false
	});

	let public = recover(&signature, &hash).map_err(errors::encryption)?;
	let address = public_to_address(&public);
	let account = BasicAccount { address, public_key: public, is_valid_for_current_chain };
	Ok(RichBasicAccount { inner: account, extra_info: Default::default() })
}
