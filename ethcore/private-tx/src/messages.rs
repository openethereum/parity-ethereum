// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use ethereum_types::{H256, U256, Address};
use bytes::Bytes;
use hash::keccak;
use rlp::Encodable;
use ethkey::Signature;
use transaction::signature::{add_chain_replay_protection, check_replay_protection};

/// Message with private transaction encrypted
#[derive(Default, Debug, Clone, PartialEq, RlpEncodable, RlpDecodable, Eq)]
pub struct PrivateTransaction {
	/// Encrypted data
	pub encrypted: Bytes,
	/// Address of the contract
	pub contract: Address,
}

impl PrivateTransaction {
	/// Compute hash on private transaction
	pub fn hash(&self) -> H256 {
		keccak(&*self.rlp_bytes())
	}
}

/// Message about private transaction's signing
#[derive(Default, Debug, Clone, PartialEq, RlpEncodable, RlpDecodable, Eq)]
pub struct SignedPrivateTransaction {
	/// Hash of the corresponding private transaction
	private_transaction_hash: H256,
	/// Signature of the validator
	/// The V field of the signature
	v: u64,
	/// The R field of the signature
	r: U256,
	/// The S field of the signature
	s: U256,
}

impl SignedPrivateTransaction {
	/// Construct a signed private transaction message
	pub fn new(private_transaction_hash: H256, sig: Signature, chain_id: Option<u64>) -> Self {
		SignedPrivateTransaction {
			private_transaction_hash: private_transaction_hash,
			r: sig.r().into(),
			s: sig.s().into(),
			v: add_chain_replay_protection(sig.v() as u64, chain_id),
		}
	}

	pub fn standard_v(&self) -> u8 { check_replay_protection(self.v) }

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature {
		Signature::from_rsv(&self.r.into(), &self.s.into(), self.standard_v())
	}

	/// Get the hash of of the original transaction.
	pub fn private_transaction_hash(&self) -> H256 {
		self.private_transaction_hash
	}
}
