use memorydb::*;
use hashdb::*;
use hash::*;
//use rlp::*;
//use bytes::*;

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
/*	fn at(key: &[u8]) -> Option<Bytes>;
	fn insert(key: &[u8], value: &[u8]);
	fn remove(key: &[u8]);
	fn contains(key: &[u8]) -> bool;*/
}

pub struct TrieDB {
	db: Box<HashDB>,
	root: H256,
}

impl TrieDB {
	pub fn new<T>(db: T) -> Self where T: HashDB + 'static { TrieDB{ db: Box::new(db), root: H256::new() } }

	pub fn new_boxed(db_box: Box<HashDB>) -> Self { TrieDB{ db: db_box, root: H256::new() } }

	pub fn new_memory() -> Self { TrieDB{ db: Box::new(MemoryDB::new()), root: H256::new() } }

	pub fn init(&mut self) { self.insert_root(&NULL_RLP); }

	pub fn db(&self) -> &HashDB { self.db.as_ref() }

//	pub fn db_mut(&mut self) -> &mut HashDB { self.db.as }

	fn insert_root(&mut self, root_data: &[u8]) { self.root = self.db.insert(root_data); }

}

impl Trie for TrieDB {
	fn root(&self) -> &H256 { &self.root }
}

#[test]
fn it_works() {
	use overlaydb::*;

	(&[1, 2, 3]).starts_with(&[1, 2]);

	let mut t = TrieDB::new(OverlayDB::new_temp());
	t.init();
	assert_eq!(*t.root(), SHA3_NULL_RLP);
	assert!(t.is_empty());
	// TODO: make work:
	//assert_eq!(t.root(), SHA3_NULL_RLP);
}