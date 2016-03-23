// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Block oriented views onto rlp.
use util::*;
use header::*;
use transaction::*;

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

	// TODO: something like pub fn action(&self) -> Action { self.rlp.val_at(3) }
}

impl<'a> Hashable for TransactionView<'a> {
	fn sha3(&self) -> H256 {
		self.rlp.as_raw().sha3()
	}
}

/// View onto transaction rlp.
pub struct AccountView<'a> {
	rlp: Rlp<'a>
}

impl<'a> AccountView<'a> {
	/// Creates new view onto block from raw bytes.
	pub fn new(bytes: &'a [u8]) -> AccountView<'a> {
		AccountView {
			rlp: Rlp::new(bytes)
		}
	}

	/// Creates new view onto block from rlp.
	pub fn new_from_rlp(rlp: Rlp<'a>) -> AccountView<'a> {
		AccountView {
			rlp: rlp
		}
	}

	/// Return reference to underlaying rlp.
	pub fn rlp(&self) -> &Rlp<'a> {
		&self.rlp
	}

	/// Get the nonce field of the transaction.
	pub fn nonce(&self) -> U256 { self.rlp.val_at(0) }

	/// Get the gas_price field of the transaction.
	pub fn balance(&self) -> U256 { self.rlp.val_at(1) }

	/// Get the gas field of the transaction.
	pub fn storage_root(&self) -> H256 { self.rlp.val_at(2) }

	/// Get the value field of the transaction.
	pub fn code_hash(&self) -> H256 { self.rlp.val_at(3) }
}

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

	/// Return reference to underlaying rlp.
	pub fn rlp(&self) -> &Rlp<'a> {
		&self.rlp
	}

	/// Create new Header object from header rlp.
	pub fn header(&self) -> Header {
		self.rlp.val_at(0)
	}

	/// Create new header view obto block head rlp.
	pub fn header_view(&self) -> HeaderView<'a> {
		HeaderView::new_from_rlp(self.rlp.at(0))
	}

	/// Return List of transactions in given block.
	pub fn transactions(&self) -> Vec<SignedTransaction> {
		self.rlp.val_at(1)
	}

	/// Return List of transactions with additional localization info.
	pub fn localized_transactions(&self) -> Vec<LocalizedTransaction> {
		let header = self.header_view();
		let block_hash = header.sha3();
		let block_number = header.number();
		self.transactions()
			.into_iter()
			.enumerate()
			.map(|(i, t)| LocalizedTransaction {
				signed: t,
				block_hash: block_hash.clone(),
				block_number: block_number,
				transaction_index: i
			}).collect()
	}

	/// Return number of transactions in given block, without deserializing them.
	pub fn transactions_count(&self) -> usize {
		self.rlp.at(1).iter().count()
	}

	/// Return List of transactions in given block.
	pub fn transaction_views(&self) -> Vec<TransactionView> {
		self.rlp.at(1).iter().map(TransactionView::new_from_rlp).collect()
	}

	/// Return transaction hashes.
	pub fn transaction_hashes(&self) -> Vec<H256> {
		self.rlp.at(1).iter().map(|rlp| rlp.as_raw().sha3()).collect()
	}

	/// Returns transaction at given index without deserializing unnecessary data.
	pub fn transaction_at(&self, index: usize) -> Option<SignedTransaction> {
		self.rlp.at(1).iter().nth(index).map(|rlp| rlp.as_val())
	}

	/// Returns localized transaction at given index.
	pub fn localized_transaction_at(&self, index: usize) -> Option<LocalizedTransaction> {
		let header = self.header_view();
		let block_hash = header.sha3();
		let block_number = header.number();
		self.transaction_at(index).map(|t| LocalizedTransaction {
			signed: t,
			block_hash: block_hash,
			block_number: block_number,
			transaction_index: index
		})
	}

	/// Return list of uncles of given block.
	pub fn uncles(&self) -> Vec<Header> {
		self.rlp.val_at(2)
	}

	/// Return number of uncles in given block, without deserializing them.
	pub fn uncles_count(&self) -> usize {
		self.rlp.at(2).iter().count()
	}

	/// Return List of transactions in given block.
	pub fn uncle_views(&self) -> Vec<HeaderView> {
		self.rlp.at(2).iter().map(HeaderView::new_from_rlp).collect()
	}

	/// Return list of uncle hashes of given block.
	pub fn uncle_hashes(&self) -> Vec<H256> {
		self.rlp.at(2).iter().map(|rlp| rlp.as_raw().sha3()).collect()
	}

	/// Return nth uncle.
	pub fn uncle_at(&self, index: usize) -> Option<Header> {
		self.rlp.at(2).iter().nth(index).map(|rlp| rlp.as_val())
	}
}

impl<'a> Hashable for BlockView<'a> {
	fn sha3(&self) -> H256 {
		self.header_view().sha3()
	}
}

/// View onto block header rlp.
pub struct HeaderView<'a> {
	rlp: Rlp<'a>
}

impl<'a> HeaderView<'a> {
	/// Creates new view onto header from raw bytes.
	pub fn new(bytes: &'a [u8]) -> HeaderView<'a> {
		HeaderView {
			rlp: Rlp::new(bytes)
		}
	}

	/// Creates new view onto header from rlp.
	pub fn new_from_rlp(rlp: Rlp<'a>) -> HeaderView<'a> {
		HeaderView {
			rlp: rlp
		}
	}

	/// Returns header hash.
	pub fn hash(&self) -> H256 { self.sha3() }

	/// Returns raw rlp.
	pub fn rlp(&self) -> &Rlp<'a> { &self.rlp }

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
	pub fn log_bloom(&self) -> H2048 { self.rlp.val_at(6) }

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

	/// Returns block seal.
	pub fn seal(&self) -> Vec<Bytes> {
		let mut seal = vec![];
		for i in 13..self.rlp.item_count() {
			seal.push(self.rlp.val_at(i));
		}
		seal
	}
}

impl<'a> Hashable for HeaderView<'a> {
	fn sha3(&self) -> H256 {
		self.rlp.as_raw().sha3()
	}
}
