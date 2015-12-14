use util::hash::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::rlp::*;
use util::sha3::*;
use blockheader::*;
use state::*;
use transaction::*;

/// view onto block rlp
pub struct BlockView<'a> {
	rlp: Rlp<'a>
}

impl<'a> BlockView<'a> {
	pub fn new(bytes: &'a [u8]) -> BlockView<'a> {
		BlockView {
			rlp: Rlp::new(bytes)
		}
	}

	pub fn new_from_rlp(rlp: Rlp<'a>) -> BlockView<'a> {
		BlockView {
			rlp: rlp
		}
	}

	pub fn rlp(&self) -> &Rlp<'a> { &self.rlp }
	pub fn header(&self) -> Header { self.rlp.val_at(0) }

	pub fn header_view(&self) -> HeaderView<'a> { 
		HeaderView::new_from_rlp(self.rlp.at(0)) 
	}

	pub fn transactions(&self) -> Vec<Transaction> {
		self.rlp.val_at(1)
	}

	pub fn transaction_hashes(&self) -> Vec<H256> { 
		self.rlp.at(1).iter().map(|rlp| rlp.raw().sha3()).collect()
	}

	pub fn uncles(&self) -> Vec<Header> {
		self.rlp.val_at(2)
	}

	pub fn uncle_hashes(&self) -> Vec<H256> { 
		self.rlp.at(2).iter().map(|rlp| rlp.raw().sha3()).collect()
	}
}

/// Active model of a block within the blockchain
pub struct Block {
	state: State
}

impl Block {
	/// Creates block with empty state root
	pub fn new(db: OverlayDB) -> Block {
		unimplemented!()
	}

	/// Creates block with state root
	pub fn new_existing(db: OverlayDB, state_root: H256) -> Block {
		unimplemented!()
	}

	/// Returns mutable reference to backing state
	pub fn mutable_state<'a>(&'a mut self) -> &'a mut State {
		&mut self.state
	}
}
