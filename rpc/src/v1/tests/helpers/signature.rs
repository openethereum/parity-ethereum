use std::sync::Arc;
use ethcore::account_provider::AccountProvider;
use v1::helpers::verify_signature;
use v1::helpers::dispatch::eth_data_hash;
use hash::keccak;

pub fn add_chain_replay_protection(v: u64, chain_id: Option<u64>) -> u64 {
	v + if let Some(n) = chain_id { 35 + n * 2 } else { 0 }
}

fn setup(is_prefixed: bool, signining_chain_id: Option<u64>, rpc_chain_id: Option<u64>, is_valid_for_current_chain: bool) {
	let accounts = Arc::new(AccountProvider::transient_provider());

	let address = accounts.new_account(&"password123".into()).unwrap();
	let data = vec![5u8];
	let  hash = if is_prefixed { eth_data_hash(data.clone()) } else { keccak(data.clone()) };
	let sig = accounts.sign(address, Some("password123".into()), hash).unwrap();
	let (v, r, s) = (sig.v(), sig.r(), sig.s());
	let v = add_chain_replay_protection(v as u64, signining_chain_id);
	let account = verify_signature(is_prefixed, data.into(), v, r.into(), s.into(), rpc_chain_id).unwrap();
	assert_eq!(account.inner.address, address);
	assert_eq!(account.inner.is_valid_for_current_chain, is_valid_for_current_chain)
}

#[test]
fn test_verify_signature_prefixed_mainnet() {
	setup(true, Some(1), Some(1), true)
}

#[test]
fn test_verify_signature_not_prefixed_mainnet() {
	setup(false, Some(1), Some(1), true)
}

#[test]
fn test_verify_signature_incompatible_chain_id() {
	setup(false, Some(65), Some(1), false);
	setup(false, Some(65), Some(1), false);
}

#[test]
fn test_verify_signature_no_signing_chain_id() {
	setup(false, None, Some(1), false);
	setup(true, None, Some(1), false);
}

#[test]
fn test_verify_signature_no_rpc_chain_id() {
	setup(false, Some(1), None, false);
	setup(true, Some(1), None, false);
}

#[test]
fn test_verify_signature_no_chain_replay_protection() {
	setup(false, Some(1), None, false);
	setup(true, Some(1), None, false);
}
