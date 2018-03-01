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

//! View onto transaction rlp
use bytes::Bytes;
use ethereum_types::{H256, U256};
use hash::keccak;
use rlp::Rlp;

/// View onto transaction rlp.
pub struct TransactionView<'a> {
	rlp: Rlp<'a>
}

impl<'a> TransactionView<'a> {
	/// Creates new view onto block from raw bytes.
	pub fn new(bytes: &'a [u8]) -> TransactionView<'a> {
		TransactionView {
			rlp: Rlp::new(bytes)
		}
	}

	/// Creates new view onto block from rlp.
	pub fn new_from_rlp(rlp: Rlp<'a>) -> TransactionView<'a> {
		TransactionView {
			rlp: rlp
		}
	}

	/// Return reference to underlaying rlp.
	pub fn rlp(&self) -> &Rlp<'a> {
		&self.rlp
	}

	/// Returns transaction hash.
	pub fn hash(&self) -> H256 {
		keccak(self.rlp.as_raw())
	}

	/// Get the nonce field of the transaction.
	pub fn nonce(&self) -> U256 { self.rlp.val_at(0) }

	/// Get the gas_price field of the transaction.
	pub fn gas_price(&self) -> U256 { self.rlp.val_at(1) }

	/// Get the gas field of the transaction.
	pub fn gas(&self) -> U256 { self.rlp.val_at(2) }

	/// Get the value field of the transaction.
	pub fn value(&self) -> U256 { self.rlp.val_at(4) }

	/// Get the data field of the transaction.
	pub fn data(&self) -> Bytes { self.rlp.val_at(5) }

	/// Get the v field of the transaction.
	pub fn v(&self) -> u8 { let r: u16 = self.rlp.val_at(6); r as u8 }

	/// Get the r field of the transaction.
	pub fn r(&self) -> U256 { self.rlp.val_at(7) }

	/// Get the s field of the transaction.
	pub fn s(&self) -> U256 { self.rlp.val_at(8) }
}

#[cfg(test)]
mod tests {
	use rustc_hex::FromHex;
	use super::TransactionView;

	#[test]
	fn test_transaction_view() {
		let rlp = "f87c80018261a894095e7baea6a6c7c4c2dfeb977efac326af552d870a9d00000000000000000000000000000000000000000000000000000000001ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804".from_hex().unwrap();

		let view = TransactionView::new(&rlp);
		assert_eq!(view.nonce(), 0.into());
		assert_eq!(view.gas_price(), 1.into());
		assert_eq!(view.gas(), 0x61a8.into());
		assert_eq!(view.value(), 0xa.into());
		assert_eq!(view.data(), "0000000000000000000000000000000000000000000000000000000000".from_hex().unwrap());
		assert_eq!(view.r(), "48b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353".into());
		assert_eq!(view.s(), "efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804".into());
		assert_eq!(view.v(), 0x1b);
	}
}
