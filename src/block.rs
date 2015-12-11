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
		Block {
			state: State::new(db)
		}
	}

	/// Creates block with state root
	pub fn new_existing(db: OverlayDB, state_root: H256) -> Block {
		Block {
			state: State::new_existing(db, state_root)
		}
	}

	/// Returns mutable reference to backing state
	pub fn mutable_state<'a>(&'a mut self) -> &'a mut State {
		&mut self.state
	}
}
