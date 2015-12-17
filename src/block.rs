use util::hash::*;
use util::overlaydb::*;
use state::*;

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
