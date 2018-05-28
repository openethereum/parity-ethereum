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
use nibbleslice::NibbleSlice;
use super::node::{Node, OwnedNode};
use super::lookup::Lookup;
use super::{Trie, TrieItem, TrieError, TrieIterator, Query};
use ethereum_types::H256;
use bytes::{ToPretty, Bytes};

/// A `Trie` implementation using a generic `HashDB` backing database.
///
/// Use it as a `Trie` trait object. You can use `db()` to get the backing database object.
/// Use `get` and `contains` to query values associated with keys in the trie.
///
/// # Example
/// ```
/// extern crate patricia_trie as trie;
/// extern crate hashdb;
/// extern crate memorydb;
/// extern crate ethereum_types;
///
/// use trie::*;
/// use hashdb::*;
/// use memorydb::*;
/// use ethereum_types::H256;
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
	hash_count: usize,
}

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
		self.db
			.get(self.root)
			.ok_or_else(|| Box::new(TrieError::InvalidStateRoot(*self.root)))
	}

	/// Given some node-describing data `node`, return the actual node RLP.
	/// This could be a simple identity operation in the case that the node is sufficiently small, but
	/// may require a database lookup.
	fn get_raw_or_lookup(&'db self, node: &'db [u8]) -> super::Result<DBValue> {
		match Node::try_decode_hash(node) {
			Some(key) => {
				self.db.get(&key).ok_or_else(|| Box::new(TrieError::IncompleteDatabase(key)))
			}
			None => Ok(DBValue::from_slice(node))
		}
	}

	/// Create a node from raw rlp bytes, assumes valid rlp because encoded locally
	fn decode_node(node: &'db [u8]) -> Node {
		Node::decoded(node).expect("rlp read from db; qed")
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

// This is for pretty debug output only
struct TrieAwareDebugNode<'db, 'a> {
	trie: &'db TrieDB<'db>,
	key: &'a[u8]
}

impl<'db, 'a> fmt::Debug for TrieAwareDebugNode<'db, 'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Ok(node) = self.trie.get_raw_or_lookup(self.key) {
			match Node::decoded(&node) {
				Ok(Node::Leaf(slice, value)) => f.debug_struct("Node::Leaf")
						.field("slice", &slice)
						.field("value", &value)
					.finish(),
				Ok(Node::Extension(ref slice, ref item)) => f.debug_struct("Node::Extension")
						.field("slice", &slice)
						.field("item", &TrieAwareDebugNode{trie: self.trie, key: item})
					.finish(),
				Ok(Node::Branch(ref nodes, ref value)) => {
					let nodes: Vec<TrieAwareDebugNode> = nodes.into_iter().map(|n| TrieAwareDebugNode{trie: self.trie, key: n} ).collect();
					f.debug_struct("Node::Branch")
						.field("nodes", &nodes)
						.field("value", &value)
					.finish()
				},
				Ok(Node::Empty) => f.debug_struct("Node::Empty").finish(),

				Err(e) => f.debug_struct("BROKEN_NODE")
						.field("key", &self.key)
						.field("error",  &format!("ERROR decoding node branch Rlp: {}", e))
					.finish()
			}
		} else {
		   f.debug_struct("BROKEN_NODE")
			.field("key", &self.key)
			.field("error", &"Not found")
			.finish()
		}
	}
}


impl<'db> fmt::Debug for TrieDB<'db> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let root_rlp = self.db.get(self.root).expect("Trie root not found!");
		f.debug_struct("TrieDB")
		.field("hash_count", &self.hash_count)
		.field("root", &TrieAwareDebugNode {
			trie: self,
			key: &root_rlp
		})
		.finish()
	}
}

#[derive(Clone, Eq, PartialEq)]
enum Status {
	Entering,
	At,
	AtChild(usize),
	Exiting,
}

#[derive(Eq, PartialEq)]
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

	fn seek<'key>(&mut self, mut node_data: DBValue, mut key: NibbleSlice<'key>) -> super::Result<()> {
		loop {
			let (data, mid) = {
				let node = TrieDB::decode_node(&node_data);
				match node {
					Node::Leaf(slice, _) => {
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
						return Ok(())
					},
					Node::Extension(ref slice, ref item) => {
						if key.starts_with(slice) {
							self.trail.push(Crumb {
								status: Status::At,
								node: node.clone().into(),
							});
							self.key_nibbles.extend(slice.iter());
							let data = self.db.get_raw_or_lookup(&*item)?;
							(data, slice.len())
						} else {
							self.descend(&node_data)?;
							return Ok(())
						}
					},
					Node::Branch(ref nodes, _) => match key.is_empty() {
						true => {
							self.trail.push(Crumb {
								status: Status::At,
								node: node.clone().into(),
							});
							return Ok(())
						},
						false => {
							let i = key.at(0);
							self.trail.push(Crumb {
								status: Status::AtChild(i as usize),
								node: node.clone().into(),
							});
							self.key_nibbles.push(i);
							let child = self.db.get_raw_or_lookup(&*nodes[i as usize])?;
							(child, 1)
						}
					},
					_ => return Ok(()),
				}
			};

			node_data = data;
			key = key.mid(mid);
		}
	}

	/// Descend into a payload.
	fn descend(&mut self, d: &[u8]) -> super::Result<()> {
		let node = TrieDB::decode_node(&self.db.get_raw_or_lookup(d)?).into();
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
		self.seek(root_rlp, NibbleSlice::new(key))
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
				self.trail.last_mut()?.increment();
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
					self.descend_into_node(TrieDB::decode_node(&d).into())
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


#[test]
fn debug_output_supports_pretty_print() {
	use memorydb::*;
	use super::TrieMut;
	use super::triedbmut::*;

	let d = vec![ DBValue::from_slice(b"A"), DBValue::from_slice(b"AA"), DBValue::from_slice(b"AB"), DBValue::from_slice(b"B") ];

	let mut memdb = MemoryDB::new();
	let mut root = H256::new();
	let root = {
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for x in &d {
			t.insert(x, x).unwrap();
		}
		t.root().clone()
	};
	let t = TrieDB::new(&memdb, &root).unwrap();
	 
	assert_eq!(format!("{:?}", t), "TrieDB { hash_count: 0, root: Node::Extension { slice: 4, item: Node::Branch { nodes: [Node::Empty, Node::Branch { nodes: [Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Branch { nodes: [Node::Empty, Node::Leaf { slice: , value: [65, 65] }, Node::Leaf { slice: , value: [65, 66] }, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty], value: None }, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty], value: Some([65]) }, Node::Leaf { slice: , value: [66] }, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty, Node::Empty], value: None } } }");
	assert_eq!(format!("{:#?}", t),
"TrieDB {
    hash_count: 0,
    root: Node::Extension {
        slice: 4,
        item: Node::Branch {
            nodes: [
                Node::Empty,
                Node::Branch {
                    nodes: [
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Branch {
                            nodes: [
                                Node::Empty,
                                Node::Leaf {
                                    slice: ,
                                    value: [
                                        65,
                                        65
                                    ]
                                },
                                Node::Leaf {
                                    slice: ,
                                    value: [
                                        65,
                                        66
                                    ]
                                },
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty,
                                Node::Empty
                            ],
                            value: None
                        },
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty,
                        Node::Empty
                    ],
                    value: Some(
                        [
                            65
                        ]
                    )
                },
                Node::Leaf {
                    slice: ,
                    value: [
                        66
                    ]
                },
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty,
                Node::Empty
            ],
            value: None
        }
    }
}");
}

// Test will work once https://github.com/paritytech/parity/pull/8527 is merged and rlp::decode returns Result instead of panicking
//#[test]
//fn test_lookup_with_corrupt_data_returns_decoder_error() {
//	use memorydb::*;
//	use super::TrieMut;
//	use super::triedbmut::*;
//	use rlp;
//	use ethereum_types::H512;
//
//	let mut memdb = MemoryDB::new();
//	let mut root = H256::new();
//	{
//		let mut t = TrieDBMut::new(&mut memdb, &mut root);
//		t.insert(b"A", b"ABC").unwrap();
//		t.insert(b"B", b"ABCBA").unwrap();
//	}
//
//	let t = TrieDB::new(&memdb, &root).unwrap();
//
//	// query for an invalid data type to trigger an error
//	let q = rlp::decode::<H512>;
//	let lookup = Lookup{ db: t.db, query: q, hash: root };
//	let query_result = lookup.look_up(NibbleSlice::new(b"A"));
//	let expected = Box::new(TrieError::DecoderError(::rlp::DecoderError::RlpIsTooShort));
//	assert_eq!(query_result.unwrap_err(), expected);
//}
