use util::hash::*;
use util::hashdb::*;
use util::memorydb::*;
use util::overlaydb::*;
use util::bytes::*;
use util::trie::*;
use util::rlp::*;
use util::sha3::*;
use account::*;

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

	/// Return reference to root
	pub fn root(&self) -> &H256 {
		&self.root
	}

	/// Commit everything to the disk
	pub fn commit_db(&mut self) {
		self.db.commit().expect("Number of kills exceeded number of inserts!");
	}

	/// Commit accounts to TrieDB. This is simplified version of
	/// cpp-ethereum's dev::eth::commit.
	pub fn insert_accounts(&mut self, map: &AccountMap) {
		let mut trie = TrieDB::new_existing(&mut self.db, &mut self.root);

		for (address, account) in map.accounts().iter() {
			let mut stream = RlpStream::new_list(4);
			stream.append(account.nonce());
			stream.append(account.balance());
			let mut root = H256::new();
			{
				let mut db = MemoryDB::new();
				let mut t = TrieDB::new(&mut db, &mut root);
				for (k, v) in account.storage().iter() {
					// cast key and value to trait type,
					// so we can call overloaded `to_bytes` method
					let kas: &ToBytes = k;
					let vas: &ToBytes = v;
					t.insert(&kas.to_bytes(), &vas.to_bytes());
				}
			}
			stream.append(&root);
			
			let code_hash = account.code().sha3();
			stream.append(&code_hash);

			if account.code().len() > 0 {
				trie.insert(&code_hash, account.code());
			}
			trie.insert(address, &stream.out());
		}
	}
}
