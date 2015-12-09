use util::hash::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::trie::*;

pub struct State {
	trie: TrieDB
}

impl State {
	pub fn new(db: OverlayDB) -> State {
		State {
			trie: TrieDB::new(db)
		}
	}

	pub fn new_temp() -> State {
		Self::new(OverlayDB::new_temp())
	}
}
