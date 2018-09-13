use ethkey::{recover, public_to_address, Signature};
use jsonrpc_core::Result;
use v1::types::{Bytes, RichBasicAccount, BasicAccount, H256, to_rich_struct};
use v1::helpers::errors;
use v1::helpers::dispatch::eth_data_hash;
use hash::keccak;

pub fn verify_signature(is_prefixed: bool, message: Bytes, v: u64, r: H256, s: H256, chain_id: Option<u64>) -> Result<RichBasicAccount> {
	let hash = if is_prefixed {
		eth_data_hash(message.0)
	} else {
		keccak(message.0)
	};

	let is_valid_for_current_chain = match (chain_id, v) {
		(None, v) if v == 0 || v == 1 => true,
		(Some(chain_id), v) if v > 36 => (v - 35) / 2 == chain_id,
		_ => false,
	};

	let v = if v > 36 {
		(v - 1) % 2
	} else { v };

	let signature = Signature::from_rsv(&r.into(), &s.into(), v as u8);
	let public = recover(&signature, &hash).map_err(errors::encryption)?;
	let address = public_to_address(&public);
	let account = BasicAccount { address, public_key: public, is_valid_for_current_chain };
	Ok(to_rich_struct(account))
}
