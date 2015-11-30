use memorydb::*;
use hashdb::*;
use hash::*;
use nibbleslice::*;
use bytes::*;
use rlp::*;

pub const NULL_RLP: [u8; 1] = [0x80; 1];
pub const SHA3_NULL_RLP: H256 = H256( [0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e, 0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21] );

/*lazy_static! {
	pub static ref NULL_RLP: Bytes = { let mut r = RlpStream::new(); r.append(&""); r.out().unwrap() };
	pub static ref SHA3_NULL_RLP: H256 = { use sha3::Hashable; NULL_RLP.sha3() };
}*/

pub trait Trie {
	fn root(&self) -> &H256;
	fn is_empty(&self) -> bool { *self.root() == SHA3_NULL_RLP }

	// TODO: consider returning &[u8]...
	fn contains(&self, key: &[u8]) -> bool;
	fn at(&self, key: &[u8]) -> Option<&[u8]>;
	fn insert(&mut self, key: &[u8], value: &[u8]);
	fn remove(&mut self, key: &[u8]);
}

pub struct TrieDB {
	db: Box<HashDB>,
	root: H256,
}

struct Diff {
	new: Vec<(H256, Bytes)>,
	old: Vec<H256>,
}

impl TrieDB {
	pub fn new<T>(db: T) -> Self where T: HashDB + 'static { TrieDB{ db: Box::new(db), root: H256::new() } }

	pub fn new_boxed(db_box: Box<HashDB>) -> Self { TrieDB{ db: db_box, root: H256::new() } }

	pub fn new_memory() -> Self { TrieDB{ db: Box::new(MemoryDB::new()), root: H256::new() } }

	pub fn init(&mut self) { self.set_root_rlp(&NULL_RLP); }

	pub fn db(&self) -> &HashDB { self.db.as_ref() }

	fn set_root_rlp(&mut self, root_data: &[u8]) {
		self.root = self.db.insert(root_data);
	}

	fn add(&mut self, key: &NibbleSlice, value: &[u8]) {
		// determine what the new root is, insert new nodes and remove old as necessary.
		let mut todo: (Bytes, Diff);
		{
			let root_rlp = self.db.lookup(&self.root).unwrap();
			todo = self.merge(root_rlp, key, value);
		}
		self.apply(todo.1);
		self.set_root_rlp(&todo.0);
	}

	fn apply(&mut self, diff: Diff) {
		for d in diff.old.iter() {
			self.db.kill(&d);
		}
		for d in diff.new.into_iter() {
			self.db.emplace(d.0, d.1);
		}
	}

	/// Determine the RLP of the node, assuming we're inserting `partial_key` into the
	/// node at `old`. This will *not* delete the old mode; it will just return the new RLP
	/// that includes the new node.
	///
	/// The database will be updated so as to make the returned RLP valid through inserting
	/// and deleting nodes as necessary.
	fn merge(&self, old: &[u8], partial_key: &NibbleSlice, value: &[u8]) -> (Bytes, Diff) {
		unimplemented!();
/*		let o = Rlp::new(old);
		match (o.type()) {
			List(17) => {
				// already have a branch. route and merge.
			},
			List(2) => {
				// already have an extension. either fast_forward, cleve or transmute_to_branch.
			},
			Data(0) => compose_extension(partial_key),
			_ => panic!("Invalid RLP for node."),
		}*/
	}

	fn compose_extension(partial_key: &NibbleSlice, value: &[u8], is_leaf: bool) -> Bytes {
		let mut s = RlpStream::new_list(2);
		s.append(&partial_key.encoded(is_leaf));
		s.append(&value.to_vec());	// WTF?!?!
		//s.append(value);	// <-- should be.
		s.out()
	}
}

impl Trie for TrieDB {
	fn root(&self) -> &H256 { &self.root }

	fn contains(&self, _key: &[u8]) -> bool {
		unimplemented!();
	}

	fn at(&self, _key: &[u8]) -> Option<&[u8]> {
		unimplemented!();
	}

	fn insert(&mut self, key: &[u8], value: &[u8]) {
		(self as &mut TrieDB).add(&NibbleSlice::new(key), value);
	}

	fn remove(&mut self, _key: &[u8]) {
		unimplemented!();
	}
}

#[test]
fn playpen() {
	use overlaydb::*;

	(&[1, 2, 3]).starts_with(&[1, 2]);

	let mut t = TrieDB::new(OverlayDB::new_temp());
	t.init();
	assert_eq!(*t.root(), SHA3_NULL_RLP);
	assert!(t.is_empty());
	t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
}