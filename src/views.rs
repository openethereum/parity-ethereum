//! Block oriented views onto rlp.
use util::*;
use header::*;
use transaction::*;

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
	pub fn transactions(&self) -> Vec<Transaction> {
		self.rlp.val_at(1)
	}

	/// Return transaction hashes.
	pub fn transaction_hashes(&self) -> Vec<H256> {
		self.rlp.at(1).iter().map(|rlp| rlp.as_raw().sha3()).collect()
	}

	/// Return list of uncles of given block.
	pub fn uncles(&self) -> Vec<Header> {
		self.rlp.val_at(2)
	}

	/// Return list of uncle hashes of given block.
	pub fn uncle_hashes(&self) -> Vec<H256> {
		self.rlp.at(2).iter().map(|rlp| rlp.as_raw().sha3()).collect()
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
	pub fn number(&self) -> U256 { self.rlp.val_at(8) }

	/// Returns block gas limit.
	pub fn gas_limit(&self) -> U256 { self.rlp.val_at(9) }

	/// Returns block gas used.
	pub fn gas_used(&self) -> U256 { self.rlp.val_at(10) }

	/// Returns timestamp.
	pub fn timestamp(&self) -> U256 { self.rlp.val_at(11) }

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
