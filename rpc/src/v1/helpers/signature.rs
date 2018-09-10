use ethkey::{recover, public_to_address, Signature};
use jsonrpc_core::Result;
use v1::types::{Bytes, H520, RichBasicAccount, BasicAccount};
use v1::helpers::errors;
use ethereum_types::H256;
use tiny_keccak::Keccak;

pub fn verify_signature(is_prefixed: bool, message: Bytes, signature: H520) -> Result<RichBasicAccount> {
	let mut buf = [0; 32];
	buf.copy_from_slice(&message.0[..]);
	let mut message = H256(buf);

	if is_prefixed {
		let mut buf = [0; 32];
		let mut keccak = Keccak::new_keccak256();
		keccak.update(b"\x19Ethereum Signed Message:\n32");
		keccak.update(&message.0[..]);
		keccak.finalize(&mut buf);
		message = H256(buf);
	}

	let signature = Signature::from(signature.0);
	let public = recover(&signature, &message).map_err(errors::verification_error)?;
	let address = public_to_address(&public);
	let account = BasicAccount { address, public_key: public };
	Ok(RichBasicAccount { inner: account, extra_info: Default::default() })
}
