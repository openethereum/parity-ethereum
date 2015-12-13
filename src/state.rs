use std::collections::HashMap;
use util::hash::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::trie::*;
use util::rlp::*;
use util::uint::*;
use account::Account;

pub struct State {
	db: OverlayDB,
	root: H256,
	_cache: HashMap<Address, Option<Account>>,

	_account_start_nonce: U256,
}

impl State {
	/// Creates new state with empty state root
	pub fn new(mut db: OverlayDB, account_start_nonce: U256) -> State {
		let mut root = H256::new();
		{
			// init trie and reset root too null
			let _ = TrieDB::new(&mut db, &mut root);
		}

		State {
			db: db,
			root: root,
			_cache: HashMap::new(),
			_account_start_nonce: account_start_nonce,
		}
	}

	/// Creates new state with existing state root
	pub fn new_existing(mut db: OverlayDB, mut root: H256, account_start_nonce: U256) -> State {
		{
			// trie should panic! if root does not exist
			let _ = TrieDB::new_existing(&mut db, &mut root);
		}

		State {
			db: db,
			root: root,
			_cache: HashMap::new(),
			_account_start_nonce: account_start_nonce,
		}
	}

	/// Create temporary state object
	pub fn new_temp() -> State {
		Self::new(OverlayDB::new_temp(), U256::from(0u8))
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
	pub fn commit(db: &mut HashDB, mut root: H256, accounts: &mut HashMap<Address, Account>) -> H256 {
		// first, commit the sub trees.
		for (_, ref mut account) in accounts.iter_mut() {
			account.commit_storage(db);
			account.commit_code(db);
		}

		{
			let mut trie = TrieDB::new_existing(db, &mut root);
			for (address, account) in accounts.iter() {
				let mut stream = RlpStream::new_list(4);
				stream.append(account.nonce());
				stream.append(account.balance());
				stream.append(account.storage_root().unwrap());
				stream.append(account.code_hash().unwrap());
				trie.insert(address, &stream.out());
			}
		}
		root
	}

	pub fn insert_accounts(&mut self, accounts: &mut HashMap<Address, Account>) {
		let r = self.root.clone();
		self.root = Self::commit(&mut self.db, r, accounts);
	}
}
