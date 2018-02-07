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
use rlp::*;
use hash::keccak;
use ethkey::Signature;

/// Message with private transaction encrypted
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PrivateTransaction {
	/// Encrypted data
	pub encrypted: Bytes,
	/// Address of the contract
	pub contract: Address,
}

impl Decodable for PrivateTransaction {
	fn decode(d: &UntrustedRlp) -> Result<Self, DecoderError> {
		if d.item_count()? != 2 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
		Ok(PrivateTransaction {
			encrypted: d.val_at(0)?,
			contract: d.val_at(1)?,
		})
	}
}

impl Encodable for PrivateTransaction {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.encrypted);
		s.append(&self.contract);
	}
}

impl PrivateTransaction {
	/// Compute hash on private transaction
	pub fn hash(&self) -> H256 {
		keccak(&*self.rlp_bytes())
	}
}

/// Message about private transaction's signing
#[derive(Default, Debug, Clone, PartialEq, Eq)]
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

impl Decodable for SignedPrivateTransaction {
	fn decode(d: &UntrustedRlp) -> Result<Self, DecoderError> {
		if d.item_count()? != 4 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
		Ok(SignedPrivateTransaction {
			private_transaction_hash: d.val_at(0)?,
			v: d.val_at(1)?,
			r: d.val_at(2)?,
			s: d.val_at(3)?,
		})
	}
}

impl Encodable for SignedPrivateTransaction {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.private_transaction_hash);
		s.append(&self.v);
		s.append(&self.r);
		s.append(&self.s);
	}
}

impl SignedPrivateTransaction {
	/// Construct a signed private transaction message
	pub fn new(private_transaction_hash: H256, sig: Signature, chain_id: Option<u64>) -> Self {
		SignedPrivateTransaction {
			private_transaction_hash: private_transaction_hash,
			r: sig.r().into(),
			s: sig.s().into(),
			v: sig.v() as u64 + if let Some(n) = chain_id { 35 + n * 2 } else { 27 },
		}
	}

	/// 0 if `v` would have been 27 under "Electrum" notation, 1 if 28 or 4 if invalid.
	pub fn standard_v(&self) -> u8 {
		match self.v {
			v if v == 27 => 0 as u8,
			v if v == 28 => 1 as u8,
			v if v > 36 => ((v - 1) % 2) as u8,
			 _ => 4
		}
	}

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature {
		Signature::from_rsv(&self.r.into(), &self.s.into(), self.standard_v())
	}

	/// Get the hash of of the original transaction.
	pub fn private_transaction_hash(&self) -> H256 {
		self.private_transaction_hash
	}
}
