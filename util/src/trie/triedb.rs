// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::fmt;
use hashdb::*;
use nibbleslice::*;
use rlp::*;
use super::node::{Node, OwnedNode};
use super::lookup::Lookup;
use super::{Trie, TrieItem, TrieError, TrieIterator, Query};
use {ToPretty, Bytes, H256};

/// A `Trie` implementation using a generic `HashDB` backing database.
///
/// Use it as a `Trie` trait object. You can use `db()` to get the backing database object.
/// Use `get` and `contains` to query values associated with keys in the trie.
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

	/// Get the data of the root node.
	fn root_data(&self) -> super::Result<DBValue> {
		self.db.get(self.root).ok_or_else(|| Box::new(TrieError::InvalidStateRoot(*self.root)))
	}

	/// Indentation helper for `format_all`.
	fn fmt_indent(&self, f: &mut fmt::Formatter, size: usize) -> fmt::Result {
		for _ in 0..size {
			write!(f, "  ")?;
		}
		Ok(())
	}

	/// Recursion helper for implementation of formatting trait.
	fn fmt_all(&self, node: Node, f: &mut fmt::Formatter, deepness: usize) -> fmt::Result {
		match node {
			Node::Leaf(slice, value) => writeln!(f, "'{:?}: {:?}.", slice, value.pretty())?,
			Node::Extension(ref slice, ref item) => {
				write!(f, "'{:?} ", slice)?;
				if let Ok(node) = self.get_raw_or_lookup(&*item) {
					self.fmt_all(Node::decoded(&node), f, deepness)?;
				}
			},
			Node::Branch(ref nodes, ref value) => {
				writeln!(f, "")?;
				if let Some(ref v) = *value {
					self.fmt_indent(f, deepness + 1)?;
					writeln!(f, "=: {:?}", v.pretty())?
				}
				for i in 0..16 {
					let node = self.get_raw_or_lookup(&*nodes[i]);
					match node.as_ref().map(|n| Node::decoded(&*n)) {
						Ok(Node::Empty) => {},
						Ok(n) => {
							self.fmt_indent(f, deepness + 1)?;
							write!(f, "'{:x} ", i)?;
							self.fmt_all(n, f, deepness + 1)?;
						}
						Err(e) => {
							write!(f, "ERROR: {}", e)?;
						}
					}
				}
			},
			// empty
			Node::Empty => {
				writeln!(f, "<empty>")?;
			}
		};
		Ok(())
	}

	/// Given some node-describing data `node`, return the actual node RLP.
	/// This could be a simple identity operation in the case that the node is sufficiently small, but
	/// may require a database lookup.
	fn get_raw_or_lookup(&'db self, node: &'db [u8]) -> super::Result<DBValue> {
		// check if its sha3 + len
		let r = Rlp::new(node);
		match r.is_data() && r.size() == 32 {
			true => {
				let key = r.as_val::<H256>();
				self.db.get(&key).ok_or_else(|| Box::new(TrieError::IncompleteDatabase(key)))
			}
			false => Ok(DBValue::from_slice(node))
		}
	}
}

impl<'db> Trie for TrieDB<'db> {
	fn iter<'a>(&'a self) -> super::Result<Box<TrieIterator<Item = TrieItem> + 'a>> {
		TrieDBIterator::new(self).map(|iter| Box::new(iter) as Box<_>)
	}

	fn root(&self) -> &H256 { self.root }

	fn get_with<'a, 'key, Q: Query>(&'a self, key: &'key [u8], query: Q) -> super::Result<Option<Q::Item>>
		where 'a: 'key
	{
		Lookup {
			db: self.db,
			query: query,
			hash: self.root.clone(),
		}.look_up(NibbleSlice::new(key))
	}
}

impl<'db> fmt::Debug for TrieDB<'db> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "c={:?} [", self.hash_count)?;
		let root_rlp = self.db.get(self.root).expect("Trie root not found!");
		self.fmt_all(Node::decoded(&root_rlp), f, 0)?;
		writeln!(f, "]")
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
	node: OwnedNode,
	status: Status,
}

impl Crumb {
	/// Move on to next status in the node's sequence.
	fn increment(&mut self) {
		self.status = match (&self.status, &self.node) {
			(_, &OwnedNode::Empty) => Status::Exiting,
			(&Status::Entering, _) => Status::At,
			(&Status::At, &OwnedNode::Branch(_, _)) => Status::AtChild(0),
			(&Status::AtChild(x), &OwnedNode::Branch(_, _)) if x < 15 => Status::AtChild(x + 1),
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

		db.root_data().and_then(|root| r.descend(&root))?;
		Ok(r)
	}

	fn seek_descend<'key>(&mut self, node_data: DBValue, key: &NibbleSlice<'key>) -> super::Result<()> {
		let node = Node::decoded(&node_data);
		match node {
			Node::Leaf(ref slice, _) => {
				if slice == key {
					self.trail.push(Crumb {
						status: Status::At,
						node: node.clone().into(),
					});
				} else {
					self.trail.push(Crumb {
						status: Status::Exiting,
						node: node.clone().into(),
					});
				}

				self.key_nibbles.extend(slice.iter());
				Ok(())
			},
			Node::Extension(ref slice, ref item) => {
				if key.starts_with(slice) {
					self.trail.push(Crumb {
						status: Status::At,
						node: node.clone().into(),
					});
					self.key_nibbles.extend(slice.iter());
					let data = self.db.get_raw_or_lookup(&*item)?;
					self.seek_descend(data, &key.mid(slice.len()))
				} else {
					self.descend(&node_data)?;
					Ok(())
				}
			},
			Node::Branch(ref nodes, _) => match key.is_empty() {
				true => {
					self.trail.push(Crumb {
						status: Status::At,
						node: node.clone().into(),
					});
					Ok(())
				},
				false => {
					let i = key.at(0);
					self.trail.push(Crumb {
						status: Status::AtChild(i as usize),
						node: node.clone().into(),
					});
					self.key_nibbles.push(i);
					let child = self.db.get_raw_or_lookup(&*nodes[i as usize])?;
					self.seek_descend(child, &key.mid(1))
				}
			},
			_ => Ok(())
		}
	}

	/// Descend into a payload.
	fn descend(&mut self, d: &[u8]) -> super::Result<()> {
		let node = Node::decoded(&self.db.get_raw_or_lookup(d)?).into();
		Ok(self.descend_into_node(node))
	}

	/// Descend into a payload.
	fn descend_into_node(&mut self, node: OwnedNode) {
		self.trail.push(Crumb {
			status: Status::Entering,
			node: node,
		});
		match &self.trail.last().expect("just pushed item; qed").node {
			&OwnedNode::Leaf(ref n, _) | &OwnedNode::Extension(ref n, _) => {
				self.key_nibbles.extend((0..n.len()).map(|i| n.at(i)));
			},
			_ => {}
		}
	}

	/// The present key.
	fn key(&self) -> Bytes {
		// collapse the key_nibbles down to bytes.
		let nibbles = &self.key_nibbles;
		let mut i = 1;
		let mut result = Bytes::with_capacity(nibbles.len() / 2);
		let len = nibbles.len();
		while i < len {
			result.push(nibbles[i - 1] * 16 + nibbles[i]);
			i += 2;
		}
		result
	}
}

impl<'a> TrieIterator for TrieDBIterator<'a> {
	/// Position the iterator on the first element with key >= `key`
	fn seek(&mut self, key: &[u8]) -> super::Result<()> {
		self.trail.clear();
		self.key_nibbles.clear();
		let root_rlp = self.db.root_data()?;
		self.seek_descend(root_rlp, &NibbleSlice::new(key))
	}
}

impl<'a> Iterator for TrieDBIterator<'a> {
	type Item = TrieItem<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		enum IterStep {
			Continue,
			PopTrail,
			Descend(super::Result<DBValue>),
		}

		loop {
			let iter_step = {
				match self.trail.last_mut() {
					Some(b) => { b.increment(); },
					None => return None,
				}

				let b = self.trail.last().expect("trail.last_mut().is_some(); qed");

				match (b.status.clone(), &b.node) {
					(Status::Exiting, n) => {
						match *n {
							OwnedNode::Leaf(ref n, _) | OwnedNode::Extension(ref n, _) => {
								let l = self.key_nibbles.len();
								self.key_nibbles.truncate(l - n.len());
							},
							OwnedNode::Branch(_, _) => { self.key_nibbles.pop(); },
							_ => {}
						}
						IterStep::PopTrail
					},
					(Status::At, &OwnedNode::Leaf(_, ref v)) | (Status::At, &OwnedNode::Branch(_, Some(ref v))) => {
						return Some(Ok((self.key(), v.clone())));
					},
					(Status::At, &OwnedNode::Extension(_, ref d)) => IterStep::Descend(self.db.get_raw_or_lookup(&*d)),
					(Status::At, &OwnedNode::Branch(_, _)) => IterStep::Continue,
					(Status::AtChild(i), &OwnedNode::Branch(ref children, _)) if children[i].len() > 0 => {
						match i {
							0 => self.key_nibbles.push(0),
							i => *self.key_nibbles.last_mut()
								.expect("pushed as 0; moves sequentially; removed afterwards; qed") = i as u8,
						}
						IterStep::Descend(self.db.get_raw_or_lookup(&*children[i]))
					},
					(Status::AtChild(i), &OwnedNode::Branch(_, _)) => {
						if i == 0 {
							self.key_nibbles.push(0);
						}
						IterStep::Continue
					},
					_ => panic!() // Should never see Entering or AtChild without a Branch here.
				}
			};

			match iter_step {
				IterStep::PopTrail => {
					self.trail.pop();
				},
				IterStep::Descend(Ok(d)) => {
					self.descend_into_node(Node::decoded(&d).into())
				},
				IterStep::Descend(Err(e)) => {
					return Some(Err(e))
				}
				IterStep::Continue => {},
			}
		}
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
	assert_eq!(d.iter().map(|i| i.clone().into_vec()).collect::<Vec<_>>(), t.iter().unwrap().map(|x| x.unwrap().0).collect::<Vec<_>>());
	assert_eq!(d, t.iter().unwrap().map(|x| x.unwrap().1).collect::<Vec<_>>());
}

#[test]
fn iterator_seek() {
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
	let mut iter = t.iter().unwrap();
	assert_eq!(iter.next(), Some(Ok((b"A".to_vec(), DBValue::from_slice(b"A")))));
	iter.seek(b"!").unwrap();
	assert_eq!(d, iter.map(|x| x.unwrap().1).collect::<Vec<_>>());
	let mut iter = t.iter().unwrap();
	iter.seek(b"A").unwrap();
	assert_eq!(&d[1..], &iter.map(|x| x.unwrap().1).collect::<Vec<_>>()[..]);
	let mut iter = t.iter().unwrap();
	iter.seek(b"AA").unwrap();
	assert_eq!(&d[2..], &iter.map(|x| x.unwrap().1).collect::<Vec<_>>()[..]);
	let mut iter = t.iter().unwrap();
	iter.seek(b"A!").unwrap();
	assert_eq!(&d[1..], &iter.map(|x| x.unwrap().1).collect::<Vec<_>>()[..]);
	let mut iter = t.iter().unwrap();
	iter.seek(b"AB").unwrap();
	assert_eq!(&d[3..], &iter.map(|x| x.unwrap().1).collect::<Vec<_>>()[..]);
	let mut iter = t.iter().unwrap();
	iter.seek(b"AB!").unwrap();
	assert_eq!(&d[3..], &iter.map(|x| x.unwrap().1).collect::<Vec<_>>()[..]);
	let mut iter = t.iter().unwrap();
	iter.seek(b"B").unwrap();
	assert_eq!(&d[4..], &iter.map(|x| x.unwrap().1).collect::<Vec<_>>()[..]);
	let mut iter = t.iter().unwrap();
	iter.seek(b"C").unwrap();
	assert_eq!(&d[4..], &iter.map(|x| x.unwrap().1).collect::<Vec<_>>()[..]);
}

#[test]
fn get_len() {
	use memorydb::*;
	use super::TrieMut;
	use super::triedbmut::*;

	let mut memdb = MemoryDB::new();
	let mut root = H256::new();
	{
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(b"A", b"ABC").unwrap();
		t.insert(b"B", b"ABCBA").unwrap();
	}

	let t = TrieDB::new(&memdb, &root).unwrap();
	assert_eq!(t.get_with(b"A", |x: &[u8]| x.len()), Ok(Some(3)));
	assert_eq!(t.get_with(b"B", |x: &[u8]| x.len()), Ok(Some(5)));
	assert_eq!(t.get_with(b"C", |x: &[u8]| x.len()), Ok(None));
}
