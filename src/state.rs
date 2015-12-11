use util::hash::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::trie::*;

pub struct State {
	db: OverlayDB,
	root: H256
}

impl State {
	/// Creates new state with empty state root
	pub fn new(mut db: OverlayDB) -> State {
		let mut root = H256::new();
		{
			// init trie and reset root too null
			let _ = TrieDB::new(&mut db, &mut root);
		}

		State {
			db: db,
			root: root
		}
	}

	/// Creates new state with existing state root
	pub fn new_existing(mut db: OverlayDB, mut root: H256) -> State {
		{
			// trie should panic! if root does not exist
			let _ = TrieDB::new_existing(&mut db, &mut root);
		}

		State {
			db: db,
			root: root
		}
	}

	/// Create temporary state object
	pub fn new_temp() -> State {
		Self::new(OverlayDB::new_temp())
	}

	/// Commit everything to the disk
	pub fn commit_db(&mut self) {
		self.db.commit().expect("Number of kills exceeded number of inserts!");
	}
}
