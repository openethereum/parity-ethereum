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

//! View onto block rlp.

use hash::keccak;
use bigint::hash::H256;
use bytes::Bytes;
use header::*;
use transaction::*;
use super::{TransactionView, HeaderView};
use rlp::Rlp;

/// View onto block rlp.
pub struct BlockView<'a> {
	rlp: Rlp<'a>
}

impl<'a> BlockView<'a> {
	/// Creates new view onto block from raw bytes.
	pub fn new(bytes: &'a [u8]) -> BlockView<'a> {
		BlockView {
			rlp: Rlp::new(bytes)
		}
	}

	/// Creates new view onto block from rlp.
	pub fn new_from_rlp(rlp: Rlp<'a>) -> BlockView<'a> {
		BlockView {
			rlp: rlp
		}
	}

	/// Block header hash.
	pub fn hash(&self) -> H256 {
		self.header_view().hash()
	}

	/// Return reference to underlaying rlp.
	pub fn rlp(&self) -> &Rlp<'a> {
		&self.rlp
	}

	/// Create new Header object from header rlp.
	pub fn header(&self) -> Header {
		self.rlp.val_at(0)
	}

	/// Return header rlp.
	pub fn header_rlp(&self) -> Rlp {
		self.rlp.at(0)
	}

	/// Create new header view obto block head rlp.
	pub fn header_view(&self) -> HeaderView<'a> {
		HeaderView::new_from_rlp(self.rlp.at(0))
	}

	/// Return List of transactions in given block.
	pub fn transactions(&self) -> Vec<UnverifiedTransaction> {
		self.rlp.list_at(1)
	}

	/// Return List of transactions with additional localization info.
	pub fn localized_transactions(&self) -> Vec<LocalizedTransaction> {
		let header = self.header_view();
		let block_hash = header.hash();
		let block_number = header.number();
		self.transactions()
			.into_iter()
			.enumerate()
			.map(|(i, t)| LocalizedTransaction {
				signed: t,
				block_hash: block_hash.clone(),
				block_number: block_number,
				transaction_index: i,
				cached_sender: None,
			}).collect()
	}

	/// Return number of transactions in given block, without deserializing them.
	pub fn transactions_count(&self) -> usize {
		self.rlp.at(1).iter().count()
	}

	/// Return List of transactions in given block.
	pub fn transaction_views(&self) -> Vec<TransactionView<'a>> {
		self.rlp.at(1).iter().map(TransactionView::new_from_rlp).collect()
	}

	/// Return transaction hashes.
	pub fn transaction_hashes(&self) -> Vec<H256> {
		self.rlp.at(1).iter().map(|rlp| keccak(rlp.as_raw())).collect()
	}

	/// Returns transaction at given index without deserializing unnecessary data.
	pub fn transaction_at(&self, index: usize) -> Option<UnverifiedTransaction> {
		self.rlp.at(1).iter().nth(index).map(|rlp| rlp.as_val())
	}

	/// Returns localized transaction at given index.
	pub fn localized_transaction_at(&self, index: usize) -> Option<LocalizedTransaction> {
		let header = self.header_view();
		let block_hash = header.hash();
		let block_number = header.number();
		self.transaction_at(index).map(|t| LocalizedTransaction {
			signed: t,
			block_hash: block_hash,
			block_number: block_number,
			transaction_index: index,
			cached_sender: None,
		})
	}

	/// Return list of uncles of given block.
	pub fn uncles(&self) -> Vec<Header> {
		self.rlp.list_at(2)
	}

	/// Return number of uncles in given block, without deserializing them.
	pub fn uncles_count(&self) -> usize {
		self.rlp.at(2).iter().count()
	}

	/// Return List of transactions in given block.
	pub fn uncle_views(&self) -> Vec<HeaderView<'a>> {
		self.rlp.at(2).iter().map(HeaderView::new_from_rlp).collect()
	}

	/// Return list of uncle hashes of given block.
	pub fn uncle_hashes(&self) -> Vec<H256> {
		self.rlp.at(2).iter().map(|rlp| keccak(rlp.as_raw())).collect()
	}

	/// Return nth uncle.
	pub fn uncle_at(&self, index: usize) -> Option<Header> {
		self.rlp.at(2).iter().nth(index).map(|rlp| rlp.as_val())
	}

	/// Return nth uncle rlp.
	pub fn uncle_rlp_at(&self, index: usize) -> Option<Bytes> {
		self.rlp.at(2).iter().nth(index).map(|rlp| rlp.as_raw().to_vec())
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use rustc_hex::FromHex;
	use bigint::hash::H256;
	use super::BlockView;

	#[test]
	fn test_block_view() {
		// that's rlp of block created with ethash engine.
		let rlp = "f90261f901f9a0d405da4e66f1445d455195229624e133f5baafe72b5cf7b3c36c12c8146e98b7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a05fb2b4bfdef7b314451cb138a534d225c922fc0e5fbe25e451142732c3e25c25a088d2ec6b9860aae1a2c3b299f72b6a5d70d7f7ba4722c78f2c49ba96273c2158a007c6fdfa8eea7e86b81f5b0fc0f78f90cc19f4aa60d323151e0cac660199e9a1b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302008003832fefba82524d84568e932a80a0a0349d8c3df71f1a48a9df7d03fd5f14aeee7d91332c009ecaff0a71ead405bd88ab4e252a7e8c2a23f862f86002018304cb2f94ec0e71ad0a90ffe1909d27dac207f7680abba42d01801ba03a347e72953c860f32b1eb2c78a680d8734b2ea08085d949d729479796f218d5a047ea6239d9e31ccac8af3366f5ca37184d26e7646e3191a3aeb81c4cf74de500c0".from_hex().unwrap();

		let view = BlockView::new(&rlp);
		assert_eq!(view.hash(), H256::from_str("2c9747e804293bd3f1a986484343f23bc88fd5be75dfe9d5c2860aff61e6f259").unwrap());
		assert_eq!(view.transactions_count(), 1);
		assert_eq!(view.uncles_count(), 0);
	}
}
