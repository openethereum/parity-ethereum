// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! View onto block header rlp

use bytes::Bytes;
use ethereum_types::{H256, Bloom, U256, Address};
use hash::keccak;
use rlp::{self};
use super::ViewRlp;
use BlockNumber;

/// View onto block header rlp.
pub struct HeaderView<'a> {
	rlp: ViewRlp<'a>
}

impl<'a> HeaderView<'a> {
	/// Creates a new Header view from valid ViewRlp
	/// Use the `view!` macro to create this view in order to capture debugging info.
	///
	/// # Example
	///
	/// ```
	/// #[macro_use]
	/// extern crate common_types as types;
	///
	/// use types::views::{HeaderView};
	///
	/// fn main() {
	/// let bytes : &[u8] = &[];
	/// let tx_view = view!(HeaderView, bytes);
	/// }
	/// ```
	pub fn new(rlp: ViewRlp<'a>) -> HeaderView<'a> {
		HeaderView {
			rlp
		}
	}

	/// Returns header hash.
	pub fn hash(&self) -> H256 {
		keccak(self.rlp.rlp.as_raw())
	}

	/// Returns raw rlp.
	pub fn rlp(&self) -> &ViewRlp<'a> { &self.rlp }

	/// Returns parent hash.
	pub fn parent_hash(&self) -> H256 { self.rlp.val_at(0) }

	/// Returns uncles hash.
	pub fn uncles_hash(&self) -> H256 { self.rlp.val_at(1) }

	/// Returns author.
	pub fn author(&self) -> Address { self.rlp.val_at(2) }

	/// Returns state root.
	pub fn state_root(&self) -> H256 { self.rlp.val_at(3) }

	/// Returns transactions root.
	pub fn transactions_root(&self) -> H256 { self.rlp.val_at(4) }

	/// Returns block receipts root.
	pub fn receipts_root(&self) -> H256 { self.rlp.val_at(5) }

	/// Returns block log bloom.
	pub fn log_bloom(&self) -> Bloom { self.rlp.val_at(6) }

	/// Returns block difficulty.
	pub fn difficulty(&self) -> U256 { self.rlp.val_at(7) }

	/// Returns block number.
	pub fn number(&self) -> BlockNumber { self.rlp.val_at(8) }

	/// Returns block gas limit.
	pub fn gas_limit(&self) -> U256 { self.rlp.val_at(9) }

	/// Returns block gas used.
	pub fn gas_used(&self) -> U256 { self.rlp.val_at(10) }

	/// Returns timestamp.
	pub fn timestamp(&self) -> u64 { self.rlp.val_at(11) }

	/// Returns block extra data.
	pub fn extra_data(&self) -> Bytes { self.rlp.val_at(12) }

	/// Returns a vector of post-RLP-encoded seal fields.
	pub fn seal(&self) -> Vec<Bytes> {
		let mut seal = vec![];
		for i in 13..self.rlp.item_count() {
			seal.push(self.rlp.at(i).as_raw().to_vec());
		}
		seal
	}

	/// Returns a vector of seal fields (RLP-decoded).
	pub fn decode_seal(&self) -> Result<Vec<Bytes>, rlp::DecoderError> {
		let seal = self.seal();
		seal.into_iter()
			.map(|s| rlp::Rlp::new(&s).data().map(|x| x.to_vec()))
			.collect()
	}

}

#[cfg(test)]
mod tests {
	use rustc_hex::FromHex;
	use ethereum_types::{Bloom, H256, Address};
	use super::HeaderView;
	use std::str::FromStr;

	#[test]
	fn test_header_view() {
		// that's rlp of block header created with ethash engine.
		let rlp = "f901f9a0d405da4e66f1445d455195229624e133f5baafe72b5cf7b3c36c12c8146e98b7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a05fb2b4bfdef7b314451cb138a534d225c922fc0e5fbe25e451142732c3e25c25a088d2ec6b9860aae1a2c3b299f72b6a5d70d7f7ba4722c78f2c49ba96273c2158a007c6fdfa8eea7e86b81f5b0fc0f78f90cc19f4aa60d323151e0cac660199e9a1b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302008003832fefba82524d84568e932a80a0a0349d8c3df71f1a48a9df7d03fd5f14aeee7d91332c009ecaff0a71ead405bd88ab4e252a7e8c2a23".from_hex().unwrap();
		let mix_hash = "a0a0349d8c3df71f1a48a9df7d03fd5f14aeee7d91332c009ecaff0a71ead405bd".from_hex().unwrap();
		let nonce = "88ab4e252a7e8c2a23".from_hex().unwrap();

		let view = view!(HeaderView, &rlp);
		assert_eq!(view.hash(), H256::from_str("2c9747e804293bd3f1a986484343f23bc88fd5be75dfe9d5c2860aff61e6f259").unwrap());
		assert_eq!(view.parent_hash(), H256::from_str("d405da4e66f1445d455195229624e133f5baafe72b5cf7b3c36c12c8146e98b7").unwrap());
		assert_eq!(view.uncles_hash(), H256::from_str("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").unwrap());
		assert_eq!(view.author(), Address::from_str("8888f1f195afa192cfee860698584c030f4c9db1").unwrap());
		assert_eq!(view.state_root(), H256::from_str("5fb2b4bfdef7b314451cb138a534d225c922fc0e5fbe25e451142732c3e25c25").unwrap());
		assert_eq!(view.transactions_root(), H256::from_str("88d2ec6b9860aae1a2c3b299f72b6a5d70d7f7ba4722c78f2c49ba96273c2158").unwrap());
		assert_eq!(view.receipts_root(), H256::from_str("07c6fdfa8eea7e86b81f5b0fc0f78f90cc19f4aa60d323151e0cac660199e9a1").unwrap());
		assert_eq!(view.log_bloom(), Bloom::default());
		assert_eq!(view.difficulty(), 0x020080.into());
		assert_eq!(view.number(), 3);
		assert_eq!(view.gas_limit(), 0x2fefba.into());
		assert_eq!(view.gas_used(), 0x524d.into());
		assert_eq!(view.timestamp(), 0x56_8e_93_2a);
		assert_eq!(view.extra_data(), vec![] as Vec<u8>);
		assert_eq!(view.seal(), vec![mix_hash, nonce]);
	}
}
