// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use ethkey::{recover, public_to_address, Signature};
use ethereum_types::{H256, U64};
use jsonrpc_core::Result;
use v1::types::{Bytes, RecoveredAccount};
use v1::helpers::errors;
use v1::helpers::dispatch::eth_data_hash;
use hash::keccak;

/// helper method for parity_verifySignature
pub fn verify_signature(
	is_prefixed: bool,
	message: Bytes,
	r: H256,
	s: H256,
	v: U64,
	chain_id: Option<u64>
) -> Result<RecoveredAccount> {
	let hash = if is_prefixed {
		eth_data_hash(message.0)
	} else {
		keccak(message.0)
	};
	let v = v.as_u64();
	let is_valid_for_current_chain = match (chain_id, v) {
		(None, v) if v == 0 || v == 1 => true,
		(Some(chain_id), v) if v >= 35 => (v - 35) / 2 == chain_id,
		_ => false,
	};

	let v = if v >= 35 { (v - 1) % 2 } else { v };

	let signature = Signature::from_rsv(&r, &s, v as u8);
	let public_key = recover(&signature, &hash).map_err(errors::encryption)?;
	let address = public_to_address(&public_key);
	Ok(RecoveredAccount { address, public_key, is_valid_for_current_chain })
}

#[cfg(test)]
mod tests {
	use super::*;
	use ethkey::Generator;
	use ethereum_types::{H160, U64};

	pub fn add_chain_replay_protection(v: u64, chain_id: Option<u64>) -> u64 {
		v + if let Some(n) = chain_id { 35 + n * 2 } else { 0 }
	}

	struct TestCase {
		should_prefix: bool,
		signing_chain_id: Option<u64>,
		rpc_chain_id: Option<u64>,
		is_valid_for_current_chain: bool,
	}

	/// mocked signer
	fn sign(should_prefix: bool, data: Vec<u8>, signing_chain_id: Option<u64>) -> (H160, [u8; 32], [u8; 32], U64) {
		let hash = if should_prefix { eth_data_hash(data) } else { keccak(data) };
		let account = ethkey::Random.generate().unwrap();
		let address = account.address();
		let sig = ethkey::sign(account.secret(), &hash).unwrap();
		let (r, s, v) = (sig.r(), sig.s(), sig.v());
		let v = add_chain_replay_protection(v as u64, signing_chain_id);
		let (r_buf, s_buf) = {
			let (mut r_buf, mut s_buf) = ([0u8; 32], [0u8; 32]);
			r_buf.copy_from_slice(r);
			s_buf.copy_from_slice(s);
			(r_buf, s_buf)
		};
		(address.into(), r_buf, s_buf, v.into())
	}

	fn run_test(test_case: TestCase) {
		let TestCase { should_prefix, signing_chain_id, rpc_chain_id, is_valid_for_current_chain } = test_case;
		let data = vec![5u8];

		let (address, r, s, v) = sign(should_prefix, data.clone(), signing_chain_id);
		let account = verify_signature(should_prefix, data.into(), r.into(), s.into(), v, rpc_chain_id).unwrap();

		assert_eq!(account.address, address.into());
		assert_eq!(account.is_valid_for_current_chain, is_valid_for_current_chain)
	}

	#[test]
	fn test_verify_signature_prefixed_mainnet() {
		run_test(TestCase {
			should_prefix: true,
			signing_chain_id: Some(1),
			rpc_chain_id: Some(1),
			is_valid_for_current_chain: true,
		})
	}

	#[test]
	fn test_verify_signature_not_prefixed_mainnet() {
		run_test(TestCase {
			should_prefix: false,
			signing_chain_id: Some(1),
			rpc_chain_id: Some(1),
			is_valid_for_current_chain: true,
		})
	}

	#[test]
	fn test_verify_signature_incompatible_chain_id() {
		run_test(TestCase {
			should_prefix: false,
			signing_chain_id: Some(65),
			rpc_chain_id: Some(1),
			is_valid_for_current_chain: false,
		});
		run_test(TestCase {
			should_prefix: true,
			signing_chain_id: Some(65),
			rpc_chain_id: Some(1),
			is_valid_for_current_chain: false,
		});
	}

	#[test]
	fn test_verify_signature_no_signing_chain_id() {
		run_test(TestCase {
			should_prefix: false,
			signing_chain_id: None,
			rpc_chain_id: Some(1),
			is_valid_for_current_chain: false,
		});
		run_test(TestCase {
			should_prefix: true,
			signing_chain_id: None,
			rpc_chain_id: Some(1),
			is_valid_for_current_chain: false,
		});
	}

	#[test]
	fn test_verify_signature_no_rpc_chain_id() {
		run_test(TestCase {
			should_prefix: false,
			signing_chain_id: Some(1),
			rpc_chain_id: None,
			is_valid_for_current_chain: false,
		});
		run_test(TestCase {
			should_prefix: true,
			signing_chain_id: Some(1),
			rpc_chain_id: None,
			is_valid_for_current_chain: false,
		});
	}

	#[test]
	fn test_verify_signature_no_chain_replay_protection() {
		run_test(TestCase {
			should_prefix: false,
			signing_chain_id: None,
			rpc_chain_id: None,
			is_valid_for_current_chain: true,
		});
		run_test(TestCase {
			should_prefix: true,
			signing_chain_id: None,
			rpc_chain_id: None,
			is_valid_for_current_chain: true,
		});
	}
}
