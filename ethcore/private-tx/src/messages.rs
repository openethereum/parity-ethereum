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

use ethereum_types::{H256, U256, Address, BigEndianHash};
use bytes::Bytes;
use hash::keccak;
use rlp::Encodable;
use ethkey::Signature;
use types::transaction::signature::{add_chain_replay_protection, check_replay_protection};

/// Message with private transaction encrypted
#[derive(Default, Debug, Clone, PartialEq, RlpEncodable, RlpDecodable, Eq)]
pub struct PrivateTransaction {
	/// Encrypted data
	encrypted: Bytes,
	/// Address of the contract
	contract: Address,
	/// Hash
	hash: H256,
}

impl PrivateTransaction {
	/// Constructor
	pub fn new(encrypted: Bytes, contract: Address) -> Self {
		PrivateTransaction {
			encrypted,
			contract,
			hash: H256::zero(),
		}.compute_hash()
	}

	fn compute_hash(mut self) -> PrivateTransaction {
		self.hash = keccak(&*self.rlp_bytes());
		self
	}

	/// Hash of the private transaction
	pub fn hash(&self) -> H256 {
		self.hash
	}

	/// Address of the contract
	pub fn contract(&self) -> Address {
		self.contract
	}

	/// Encrypted data
	pub fn encrypted(&self) -> Bytes {
		self.encrypted.clone()
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
	/// Hash
	hash: H256,
}

impl SignedPrivateTransaction {
	/// Construct a signed private transaction message
	pub fn new(private_transaction_hash: H256, sig: Signature, chain_id: Option<u64>) -> Self {
		SignedPrivateTransaction {
			private_transaction_hash: private_transaction_hash,
			r: sig.r().into(),
			s: sig.s().into(),
			v: add_chain_replay_protection(sig.v() as u64, chain_id),
			hash: H256::zero(),
		}.compute_hash()
	}

	fn compute_hash(mut self) -> SignedPrivateTransaction {
		self.hash = keccak(&*self.rlp_bytes());
		self
	}

	pub fn standard_v(&self) -> u8 { check_replay_protection(self.v) }

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature {
		Signature::from_rsv(
			&BigEndianHash::from_uint(&self.r),
			&BigEndianHash::from_uint(&self.s),
			self.standard_v(),
		)
	}

	/// Get the hash of of the original transaction.
	pub fn private_transaction_hash(&self) -> H256 {
		self.private_transaction_hash
	}

	/// Own hash
	pub fn hash(&self) -> H256 {
		self.hash
	}
}
