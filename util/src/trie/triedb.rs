// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use common::*;
use hashdb::*;
use nibbleslice::*;
use rlp::*;
use super::node::Node;
use super::recorder::{Recorder, NoOp};
use super::{Trie, TrieItem, TrieError};

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
///
/// use util::trie::*;
/// use util::hashdb::*;
/// use util::memorydb::*;
/// use util::hash::*;
///
/// fn main() {
///   let mut memdb = MemoryDB::new();
///   let mut root = H256::new();
///   TrieDBMut::new(&mut memdb, &mut root).insert(b"foo", b"bar").unwrap();
///   let t = TrieDB::new(&memdb, &root).unwrap();
///   assert!(t.contains(b"foo").unwrap());
///   assert_eq!(t.get(b"foo").unwrap().unwrap(), DBValue::from_slice(b"bar"));
///   assert!(t.db_items_remaining().unwrap().is_empty());
/// }
/// ```
pub struct TrieDB<'db> {
	db: &'db HashDB,
	root: &'db H256,
	/// The number of hashes performed so far in operations on this trie.
	pub hash_count: usize,
}

#[cfg_attr(feature="dev", allow(wrong_self_convention))]
impl<'db> TrieDB<'db> {
	/// Create a new trie with the backing database `db` and `root`
	/// Returns an error if `root` does not exist
	pub fn new(db: &'db HashDB, root: &'db H256) -> super::Result<Self> {
		if !db.contains(root) {
			Err(Box::new(TrieError::InvalidStateRoot(*root)))
		} else {
			Ok(TrieDB {
				db: db,
				root: root,
				hash_count: 0
			})
		}
	}

	/// Get the backing database.
	pub fn db(&'db self) -> &'db HashDB {
		self.db
	}

	/// Determine all the keys in the backing database that belong to the trie.
	pub fn keys(&self) -> super::Result<Vec<H256>> {
		let mut ret: Vec<H256> = Vec::new();
		ret.push(self.root.clone());
		try!(self.accumulate_keys(try!(self.root_node(&mut NoOp)), &mut ret));
		Ok(ret)
	}

	/// Convert a vector of hashes to a hashmap of hash to occurrences.
	pub fn to_map(hashes: Vec<H256>) -> HashMap<H256, u32> {
		let mut r: HashMap<H256, u32> = HashMap::new();
		for h in hashes {
			*r.entry(h).or_insert(0) += 1;
		}
		r
	}

	/// Determine occurrences of items in the backing database which are not related to this
	/// trie.
	pub fn db_items_remaining(&self) -> super::Result<HashMap<H256, i32>> {
		let mut ret = self.db.keys();
		for (k, v) in Self::to_map(try!(self.keys())) {
			let keycount = *ret.get(&k).unwrap_or(&0);
			match keycount <= v as i32 {
				true => ret.remove(&k),
				_ => ret.insert(k, keycount - v as i32),
			};
		}
		Ok(ret)
	}

	/// Recursion helper for `keys`.
	fn accumulate_keys(&self, node: Node, acc: &mut Vec<H256>) -> super::Result<()> {
		let mut handle_payload = |payload| {
			let p = Rlp::new(payload);
			if p.is_data() && p.size() == 32 {
				acc.push(p.as_val());
			}

			self.accumulate_keys(try!(self.get_node(payload, &mut NoOp, 0)), acc)
		};

		match node {
			Node::Extension(_, ref payload) => try!(handle_payload(payload)),
			Node::Branch(ref payloads, _) => for payload in payloads { try!(handle_payload(payload)) },
			_ => {},
		}

		Ok(())
	}

	/// Get the root node's RLP.
	fn root_node<R: Recorder>(&self, r: &mut R) -> super::Result<Node> {
		self.root_data(r).map(|d| Node::decoded(&d))
	}

	/// Get the data of the root node.
	fn root_data<'a, R: 'a + Recorder>(&self, r: &'a mut R) -> super::Result<DBValue> {
		self.db.get(self.root).ok_or_else(|| Box::new(TrieError::InvalidStateRoot(*self.root)))
			.map(|node| { r.record(self.root, &*node, 0); node })
	}

	/// Get the root node as a `Node`.
	fn get_node<'a, R: 'a + Recorder>(&'db self, node: &'db [u8], r: &'a mut R, depth: u32) -> super::Result<Node> {
		self.get_raw_or_lookup(node, r, depth).map(|n| Node::decoded(&n))
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
				if let Ok(node) = self.get_node(&*item, &mut NoOp, 0) {
					try!(self.fmt_all(node, f, deepness));
				}
			},
			Node::Branch(ref nodes, ref value) => {
				try!(writeln!(f, ""));
				if let Some(ref v) = *value {
					try!(self.fmt_indent(f, deepness + 1));
					try!(writeln!(f, "=: {:?}", v.pretty()))
				}
				for i in 0..16 {
					match self.get_node(&*nodes[i], &mut NoOp, 0) {
						Ok(Node::Empty) => {},
						Ok(n) => {
							try!(self.fmt_indent(f, deepness + 1));
							try!(write!(f, "'{:x} ", i));
							try!(self.fmt_all(n, f, deepness + 1));
						}
						Err(e) => {
							try!(write!(f, "ERROR: {}", e));
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
	fn do_lookup<'key, R: 'key>(&'db self, key: &NibbleSlice<'key>, r: &'key mut R) -> super::Result<Option<DBValue>>
		where 'db: 'key, R: Recorder
	{
		let root_rlp = try!(self.root_data(r));
		self.get_from_node(&root_rlp, key, r, 1)
	}

	/// Recursible function to retrieve the value given a `node` and a partial `key`. `None` if no
	/// value exists for the key.
	///
	/// Note: Not a public API; use Trie trait functions.
	fn get_from_node<'key, R: 'key>(
		&'db self,
		node: &'db [u8],
		key: &NibbleSlice<'key>,
		r: &'key mut R,
		d: u32
	) -> super::Result<Option<DBValue>> where 'db: 'key, R: Recorder {
		match Node::decoded(node) {
			Node::Leaf(ref slice, ref value) if NibbleSlice::from_encoded(slice).0 == *key => Ok(Some(value.clone())),
			Node::Extension(ref slice, ref item) => {
				let slice = &NibbleSlice::from_encoded(slice).0;
				if key.starts_with(slice) {
					let data = try!(self.get_raw_or_lookup(&*item, r, d));
					self.get_from_node(&data, &key.mid(slice.len()), r, d + 1)
				} else {
					Ok(None)
				}
			},
			Node::Branch(ref nodes, ref value) => match key.is_empty() {
				true => Ok(value.clone()),
				false => {
					let node = try!(self.get_raw_or_lookup(&*nodes[key.at(0) as usize], r, d));
					self.get_from_node(&node, &key.mid(1), r, d + 1)
				}
			},
			_ => Ok(None)
		}
	}

	/// Given some node-describing data `node`, return the actual node RLP.
	/// This could be a simple identity operation in the case that the node is sufficiently small, but
	/// may require a database lookup.
	fn get_raw_or_lookup<R: Recorder>(&'db self, node: &'db [u8], rec: &mut R, d: u32) -> super::Result<DBValue> {
		// check if its sha3 + len
		let r = Rlp::new(node);
		match r.is_data() && r.size() == 32 {
			true => {
				let key = r.as_val::<H256>();
				self.db.get(&key).ok_or_else(|| Box::new(TrieError::IncompleteDatabase(key)))
					.map(|raw| { rec.record(&key, &raw, d); raw })
			}
			false => Ok(DBValue::from_slice(node))
		}
	}
}

#[derive(Clone, Eq, PartialEq)]
enum Status {
	Entering,
	At,
	AtChild(usize),
	Exiting,
}

#[derive(Clone, Eq, PartialEq)]
struct Crumb {
	node: Node,
	status: Status,
}

impl Crumb {
	/// Move on to next status in the node's sequence.
	fn increment(&mut self) {
		self.status = match (&self.status, &self.node) {
			(_, &Node::Empty) => Status::Exiting,
			(&Status::Entering, _) => Status::At,
			(&Status::At, &Node::Branch(_, _)) => Status::AtChild(0),
			(&Status::AtChild(x), &Node::Branch(_, _)) if x < 15 => Status::AtChild(x + 1),
			_ => Status::Exiting,
		}
	}
}

/// Iterator for going through all values in the trie.
#[derive(Clone)]
pub struct TrieDBIterator<'a> {
	db: &'a TrieDB<'a>,
	trail: Vec<Crumb>,
	key_nibbles: Bytes,
}

impl<'a> TrieDBIterator<'a> {
	/// Create a new iterator.
	pub fn new(db: &'a TrieDB) -> super::Result<TrieDBIterator<'a>> {
		let mut r = TrieDBIterator {
			db: db,
			trail: vec![],
			key_nibbles: Vec::new(),
		};

		try!(db.root_data(&mut NoOp).and_then(|root| r.descend(&root)));
		Ok(r)
	}

	/// Descend into a payload.
	fn descend(&mut self, d: &[u8]) -> super::Result<()> {
		self.trail.push(Crumb {
			status: Status::Entering,
			node: try!(self.db.get_node(d, &mut NoOp, 0)),
		});
		match self.trail.last().expect("just pushed item; qed").node {
			Node::Leaf(ref n, _) | Node::Extension(ref n, _) => { self.key_nibbles.extend(NibbleSlice::from_encoded(n).0.iter()); },
			_ => {}
		}

		Ok(())
	}

	/// The present key.
	fn key(&self) -> Bytes {
		// collapse the key_nibbles down to bytes.
		self.key_nibbles.iter().step(2).zip(self.key_nibbles.iter().skip(1).step(2)).map(|(h, l)| h * 16 + l).collect()
	}
}

impl<'a> Iterator for TrieDBIterator<'a> {
	type Item = TrieItem<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let b = match self.trail.last_mut() {
				Some(mut b) => { b.increment(); b.clone() },
				None => return None,
			};
			match (b.status, b.node) {
				(Status::Exiting, n) => {
					match n {
						Node::Leaf(n, _) | Node::Extension(n, _) => {
							let l = self.key_nibbles.len();
							self.key_nibbles.truncate(l - NibbleSlice::from_encoded(&*n).0.len());
						},
						Node::Branch(_, _) => { self.key_nibbles.pop(); },
						_ => {}
					}
					self.trail.pop();
					// continue
				},
				(Status::At, Node::Leaf(_, v)) | (Status::At, Node::Branch(_, Some(v))) => {
					return Some(Ok((self.key(), v)));
				},
				(Status::At, Node::Extension(_, d)) => {
					if let Err(e) = self.descend(&*d) {
						return Some(Err(e));
					}
					// continue
				},
				(Status::At, Node::Branch(_, _)) => {},
				(Status::AtChild(i), Node::Branch(ref children, _)) if children[i].len() > 0 => {
					match i {
						0 => self.key_nibbles.push(0),
						i => *self.key_nibbles.last_mut()
							.expect("pushed as 0; moves sequentially; removed afterwards; qed") = i as u8,
					}
					if let Err(e) = self.descend(&*children[i]) {
						return Some(Err(e));
					}
					// continue
				},
				(Status::AtChild(i), Node::Branch(_, _)) => {
					if i == 0 {
						self.key_nibbles.push(0);
					}
					// continue
				},
				_ => panic!() // Should never see Entering or AtChild without a Branch here.
			}
		}
	}
}

impl<'db> Trie for TrieDB<'db> {
	fn iter<'a>(&'a self) -> super::Result<Box<Iterator<Item = TrieItem> + 'a>> {
		TrieDBIterator::new(self).map(|iter| Box::new(iter) as Box<_>)
	}

	fn root(&self) -> &H256 { self.root }

	fn get_recorded<'a, 'b, R: 'b>(&'a self, key: &'b [u8], rec: &'b mut R) -> super::Result<Option<DBValue>>
		where 'a: 'b, R: Recorder
	{
		self.do_lookup(&NibbleSlice::new(key), rec)
	}
}

impl<'db> fmt::Debug for TrieDB<'db> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		try!(writeln!(f, "c={:?} [", self.hash_count));
		let root_rlp = self.db.get(self.root).expect("Trie root not found!");
		try!(self.fmt_all(Node::decoded(&root_rlp), f, 0));
		writeln!(f, "]")
	}
}

#[test]
fn iterator() {
	use memorydb::*;
	use super::TrieMut;
	use super::triedbmut::*;

	let d = vec![ DBValue::from_slice(b"A"), DBValue::from_slice(b"AA"), DBValue::from_slice(b"AB"), DBValue::from_slice(b"B") ];

	let mut memdb = MemoryDB::new();
	let mut root = H256::new();
	{
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for x in &d {
			t.insert(x, x).unwrap();
		}
	}

	let t = TrieDB::new(&memdb, &root).unwrap();
	assert_eq!(d.iter().map(|i| i.clone().to_vec()).collect::<Vec<_>>(), t.iter().unwrap().map(|x| x.unwrap().0).collect::<Vec<_>>());
	assert_eq!(d, t.iter().unwrap().map(|x| x.unwrap().1).collect::<Vec<_>>());
}
