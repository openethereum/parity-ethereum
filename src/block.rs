use util::hash::*;
use util::hashdb::*;
use util::overlaydb::*;
use state::*;

/// Active model of a block within the blockchain
pub struct Block {
	state: State
}

impl Block {
	/// Basic state object from database
	pub fn new(db: OverlayDB) -> Block {
		Block {
			state: State::new(db)
		}
	}
}
