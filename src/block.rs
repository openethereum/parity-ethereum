use util::hash::*;
use util::overlaydb::*;
use util::rlp::*;
use util::sha3::*;
use blockheader::*;
use state::*;
use transaction::*;
use extras::*;

/// view onto block rlp
pub struct BlockView<'a> {
	rlp: Rlp<'a>
}

impl<'a> BlockView<'a> {
	/// Creates new view onto block from raw bytes
	pub fn new(bytes: &'a [u8]) -> BlockView<'a> {
		BlockView {
			rlp: Rlp::new(bytes)
		}
	}

	/// Creates new view onto block from rlp
	pub fn new_from_rlp(rlp: Rlp<'a>) -> BlockView<'a> {
		BlockView {
			rlp: rlp
		}
	}

	/// Return reference to underlaying rlp
	pub fn rlp(&self) -> &Rlp<'a> { 
		&self.rlp 
	}

	/// Create new Header object from header rlp
	pub fn header(&self) -> Header { 
		self.rlp.val_at(0)
	}

	/// Create new header view obto block head rlp
	pub fn header_view(&self) -> HeaderView<'a> { 
		HeaderView::new_from_rlp(self.rlp.at(0)) 
	}

	/// Return List of transactions in given block
	pub fn transactions(&self) -> Vec<Transaction> {
		self.rlp.val_at(1)
	}

	/// Return transaction hashes
	pub fn transaction_hashes(&self) -> Vec<H256> { 
		self.rlp.at(1).iter().map(|rlp| rlp.raw().sha3()).collect()
	}

	/// Return list of uncles of given block
	pub fn uncles(&self) -> Vec<Header> {
		self.rlp.val_at(2)
	}

	/// Return list of uncle hashes of given block
	pub fn uncle_hashes(&self) -> Vec<H256> { 
		self.rlp.at(2).iter().map(|rlp| rlp.raw().sha3()).collect()
	}

	/// Return BlockDetaile object of given block
	/// note* children is always an empty vector,
	/// cause we can't deducate them from rlp.
	pub fn block_details(&self) -> BlockDetails {
		let header = self.header_view();
		BlockDetails {
			number: header.number(),
			total_difficulty: header.difficulty(),
			parent: header.parent_hash(),
			children: vec![]
		}
	}
}

impl<'a> Hashable for BlockView<'a> {
	fn sha3(&self) -> H256 {
		self.header_view().sha3()
	}
}

/// Active model of a block within the blockchain
pub struct Block {
	state: State
}

impl Block {
	/// Creates block with empty state root
	pub fn new(_db: OverlayDB) -> Block {
		unimplemented!()
	}

	/// Creates block with state root
	pub fn new_existing(_db: OverlayDB, _state_root: H256) -> Block {
		unimplemented!()
	}

	/// Returns mutable reference to backing state
	pub fn mutable_state<'a>(&'a mut self) -> &'a mut State {
		&mut self.state
	}
}
