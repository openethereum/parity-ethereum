use common::*;
use hashdb::*;
use nibbleslice::*;
use rlp::*;
use super::trietraits::*;
use super::node::*;

/// A `Trie` implementation using a generic `HashDB` backing database.
/// 
/// Use it as a `Trie` trait object. You can use `db()` to get the backing database object, `keys`
/// to get the keys belonging to the trie in the backing database, and `db_items_remaining()` to get
/// which items in the backing database do not belong to this trie. If this is the only trie in the
/// backing database, then `db_items_remaining()` should be empty.
///
/// # Example
/// ```
/// extern crate ethcore_util as util;
/// use util::trie::*;
/// use util::hashdb::*;
/// use util::memorydb::*;
/// use util::hash::*;
/// use util::rlp::*;
///
/// fn main() {
///   let mut memdb = MemoryDB::new();
///   let mut root = H256::new();
///   TrieDBMut::new(&mut memdb, &mut root).insert(b"foo", b"bar");
///   let t = TrieDB::new(&memdb, &root);
///   assert!(t.contains(b"foo"));
///   assert_eq!(t.get(b"foo").unwrap(), b"bar");
///   assert!(t.db_items_remaining().is_empty());
/// }
/// ```
pub struct TrieDB<'db> {
	db: &'db HashDB,
	root: &'db H256,
	pub hash_count: usize,
}

impl<'db> TrieDB<'db> {
	/// Create a new trie with the backing database `db` and `root`
	/// Panics, if `root` does not exist
	pub fn new(db: &'db HashDB, root: &'db H256) -> Self {
		if !db.exists(root) {
			flush(format!("Trie root not found {}", root));
			panic!("Trie root not found!");
		}
		TrieDB { 
			db: db, 
			root: root,
			hash_count: 0 
		}
	}

	/// Get the backing database.
	pub fn db(&'db self) -> &'db HashDB { 
		self.db 
	}

	/// Determine all the keys in the backing database that belong to the trie.
	pub fn keys(&self) -> Vec<H256> {
		let mut ret: Vec<H256> = Vec::new();
		ret.push(self.root.clone());
		self.accumulate_keys(self.root_node(), &mut ret);
		ret
	}

	/// Convert a vector of hashes to a hashmap of hash to occurances.
	pub fn to_map(hashes: Vec<H256>) -> HashMap<H256, u32> {
		let mut r: HashMap<H256, u32> = HashMap::new();
		for h in hashes.into_iter() {
			let c = *r.get(&h).unwrap_or(&0);
			r.insert(h, c + 1);
		}
		r
	}

	/// Determine occurances of items in the backing database which are not related to this
	/// trie.
	pub fn db_items_remaining(&self) -> HashMap<H256, i32> {
		let mut ret = self.db.keys();
		for (k, v) in Self::to_map(self.keys()).into_iter() {
			let keycount = *ret.get(&k).unwrap_or(&0);
			match keycount <= v as i32 {
				true => ret.remove(&k),
				_ => ret.insert(k, keycount - v as i32),
			};
		}
		ret
	}

	/// Recursion helper for `keys`.
	fn accumulate_keys(&self, node: Node, acc: &mut Vec<H256>) {
		let mut handle_payload = |payload| {
			let p = Rlp::new(payload);
			if p.is_data() && p.size() == 32 {
				acc.push(p.as_val());
			}

			self.accumulate_keys(self.get_node(payload), acc);
		};

		match node {
			Node::Extension(_, payload) => handle_payload(payload),
			Node::Branch(payloads, _) => for payload in payloads.iter() { handle_payload(payload) },
			_ => {},
		}
	}

	/// Get the root node's RLP.
	fn root_node(&self) -> Node {
		Node::decoded(self.db.lookup(&self.root).expect("Trie root not found!"))
	}

	/// Get the root node as a `Node`.
	fn get_node<'a>(&'a self, node: &'a [u8]) -> Node {
		Node::decoded(self.get_raw_or_lookup(node))
	}

	/// Indentation helper for `formal_all`.
	fn fmt_indent(&self, f: &mut fmt::Formatter, size: usize) -> fmt::Result {
		for _ in 0..size { 
			try!(write!(f, "  "));
		}
		Ok(())
	}

	/// Recursion helper for implementation of formatting trait.
	fn fmt_all(&self, node: Node, f: &mut fmt::Formatter, deepness: usize) -> fmt::Result {
		match node {
			Node::Leaf(slice, value) => try!(writeln!(f, "'{:?}: {:?}.", slice, value.pretty())),
			Node::Extension(ref slice, ref item) => {
				try!(write!(f, "'{:?} ", slice));
				try!(self.fmt_all(self.get_node(item), f, deepness));
			},
			Node::Branch(ref nodes, ref value) => {
				try!(writeln!(f, ""));
				match value {
					&Some(v) => {
						try!(self.fmt_indent(f, deepness + 1));
						try!(writeln!(f, "=: {:?}", v.pretty()))
					},
					&None => {}
				}
				for i in 0..16 {
					match self.get_node(nodes[i]) {
						Node::Empty => {},
						n => {
							try!(self.fmt_indent(f, deepness + 1));
							try!(write!(f, "'{:x} ", i));
							try!(self.fmt_all(n, f, deepness + 1));
						}
					}
				}
			},
			// empty
			Node::Empty => {
				try!(writeln!(f, "<empty>"));
			}
		};
		Ok(())
	}

	/// Return optional data for a key given as a `NibbleSlice`. Returns `None` if no data exists.
	fn do_lookup<'a, 'key>(&'a self, key: &NibbleSlice<'key>) -> Option<&'a [u8]> where 'a: 'key {
		let root_rlp = self.db.lookup(&self.root).expect("Trie root not found!");
		self.get_from_node(&root_rlp, key)
	}

	/// Recursible function to retrieve the value given a `node` and a partial `key`. `None` if no
	/// value exists for the key.
	///
	/// Note: Not a public API; use Trie trait functions.
	fn get_from_node<'a, 'key>(&'a self, node: &'a [u8], key: &NibbleSlice<'key>) -> Option<&'a [u8]> where 'a: 'key {
		match Node::decoded(node) {
			Node::Leaf(ref slice, ref value) if key == slice => Some(value),
			Node::Extension(ref slice, ref item) if key.starts_with(slice) => {
				self.get_from_node(self.get_raw_or_lookup(item), &key.mid(slice.len()))
			},
			Node::Branch(ref nodes, value) => match key.is_empty() {
				true => value,
				false => self.get_from_node(self.get_raw_or_lookup(nodes[key.at(0) as usize]), &key.mid(1))
			},
			_ => None
		}
	}

	/// Given some node-describing data `node`, return the actual node RLP.
	/// This could be a simple identity operation in the case that the node is sufficiently small, but
	/// may require a database lookup.
	fn get_raw_or_lookup<'a>(&'a self, node: &'a [u8]) -> &'a [u8] {
		// check if its sha3 + len
		let r = Rlp::new(node);
		match r.is_data() && r.size() == 32 {
			true => self.db.lookup(&r.as_val::<H256>()).expect("Not found!"),
			false => node
		}
	}
}

impl<'db> Trie for TrieDB<'db> {
	fn root(&self) -> &H256 { &self.root }

	fn contains(&self, key: &[u8]) -> bool {
		self.get(key).is_some()
	}

	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> Option<&'a [u8]> where 'a: 'key {
		self.do_lookup(&NibbleSlice::new(key))
	}
}

impl<'db> fmt::Debug for TrieDB<'db> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		try!(writeln!(f, "c={:?} [", self.hash_count));
		let root_rlp = self.db.lookup(&self.root).expect("Trie root not found!");
		try!(self.fmt_all(Node::decoded(root_rlp), f, 0));
		writeln!(f, "]")
	}
}
