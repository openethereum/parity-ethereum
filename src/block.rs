use util::hash::*;
use util::hashdb::*;
use util::overlaydb::*;
use blockheader::*;
use state::*;

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
