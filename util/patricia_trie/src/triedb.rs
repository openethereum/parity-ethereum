// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
use node_codec::NodeCodec;
use super::lookup::Lookup;
use super::{Result, Trie, TrieItem, TrieError, TrieIterator, Query};
use bytes::Bytes;
use std::marker::PhantomData;

/// A `Trie` implementation using a generic `HashDB` backing database, a `Hasher`
/// implementation to generate keys and a `NodeCodec` implementation to encode/decode
/// the nodes.
///
/// Use it as a `Trie` trait object. You can use `db()` to get the backing database object.
/// Use `get` and `contains` to query values associated with keys in the trie.
///
/// # Example
/// ```
/// extern crate patricia_trie as trie;
/// extern crate patricia_trie_ethereum as ethtrie;
/// extern crate hashdb;
/// extern crate keccak_hasher;
/// extern crate memorydb;
/// extern crate ethereum_types;
///
/// use trie::*;
/// use hashdb::*;
/// use keccak_hasher::KeccakHasher;
/// use memorydb::*;
/// use ethereum_types::H256;
/// use ethtrie::{TrieDB, TrieDBMut};
///
///
/// fn main() {
///   let mut memdb = MemoryDB::<KeccakHasher>::new();
///   let mut root = H256::new();
///   TrieDBMut::new(&mut memdb, &mut root).insert(b"foo", b"bar").unwrap();
///   let t = TrieDB::new(&memdb, &root).unwrap();
///   assert!(t.contains(b"foo").unwrap());
///   assert_eq!(t.get(b"foo").unwrap().unwrap(), DBValue::from_slice(b"bar"));
/// }
/// ```
pub struct TrieDB<'db, H, C>
where 
	H: Hasher + 'db, 
	C: NodeCodec<H>
{
	db: &'db HashDB<H>,
	root: &'db H::Out,
	/// The number of hashes performed so far in operations on this trie.
	hash_count: usize,
	codec_marker: PhantomData<C>,
}

impl<'db, H, C> TrieDB<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	/// Create a new trie with the backing database `db` and `root`
	/// Returns an error if `root` does not exist
	pub fn new(db: &'db HashDB<H>, root: &'db H::Out) -> Result<Self, H::Out, C::Error> {
		if !db.contains(root) {
			Err(Box::new(TrieError::InvalidStateRoot(*root)))
		} else {
			Ok(TrieDB {db, root, hash_count: 0, codec_marker: PhantomData})
		}
	}

	/// Get the backing database.
	pub fn db(&'db self) -> &'db HashDB<H> { self.db }

	/// Get the data of the root node.
	fn root_data(&self) -> Result<DBValue, H::Out, C::Error> {
		self.db
			.get(self.root)
			.ok_or_else(|| Box::new(TrieError::InvalidStateRoot(*self.root)))
	}

	/// Given some node-describing data `node`, return the actual node RLP.
	/// This could be a simple identity operation in the case that the node is sufficiently small, but
	/// may require a database lookup.
	fn get_raw_or_lookup(&'db self, node: &'db [u8]) -> Result<DBValue, H::Out, C::Error> {
		match C::try_decode_hash(node) {
			Some(key) => {
				self.db.get(&key).ok_or_else(|| Box::new(TrieError::IncompleteDatabase(key)))
			}
			None => Ok(DBValue::from_slice(node))
		}
	}
}

impl<'db, H, C> Trie<H, C> for TrieDB<'db, H, C>
where
	H: Hasher,
	C: NodeCodec<H>
{
	fn root(&self) -> &H::Out { self.root }

	fn get_with<'a, 'key, Q: Query<H>>(&'a self, key: &'key [u8], query: Q) -> Result<Option<Q::Item>, H::Out, C::Error>
		where 'a: 'key
	{
		Lookup {
			db: self.db,
			query: query,
			hash: self.root.clone(),
			marker: PhantomData::<C>,
		}.look_up(NibbleSlice::new(key))
	}

	fn iter<'a>(&'a self) -> Result<Box<TrieIterator<H, C, Item=TrieItem<H::Out, C::Error>> + 'a>, H::Out, C::Error> {
		TrieDBIterator::new(self).map(|iter| Box::new(iter) as Box<_>)
	}
}

// This is for pretty debug output only
struct TrieAwareDebugNode<'db, 'a, H, C>
where 
	H: Hasher + 'db, 
	C: NodeCodec<H> + 'db
{
	trie: &'db TrieDB<'db, H, C>,
	key: &'a[u8]
}

impl<'db, 'a, H, C> fmt::Debug for TrieAwareDebugNode<'db, 'a, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Ok(node) = self.trie.get_raw_or_lookup(self.key) {
			match C::decode(&node) {
				Ok(Node::Leaf(slice, value)) => f.debug_struct("Node::Leaf")
						.field("slice", &slice)
						.field("value", &value)
					.finish(),
				Ok(Node::Extension(ref slice, ref item)) => f.debug_struct("Node::Extension")
						.field("slice", &slice)
						.field("item", &TrieAwareDebugNode{trie: self.trie, key: item})
					.finish(),
				Ok(Node::Branch(ref nodes, ref value)) => {
					let nodes: Vec<TrieAwareDebugNode<H, C>> = nodes.into_iter().map(|n| TrieAwareDebugNode{trie: self.trie, key: n} ).collect();
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

impl<'db, H, C> fmt::Debug for TrieDB<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
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
pub struct TrieDBIterator<'a, H: Hasher + 'a, C: NodeCodec<H> + 'a> {
	db: &'a TrieDB<'a, H, C>,
	trail: Vec<Crumb>,
	key_nibbles: Bytes,
}

impl<'a, H: Hasher, C: NodeCodec<H>> TrieDBIterator<'a, H, C> {
	/// Create a new iterator.
	pub fn new(db: &'a TrieDB<H, C>) -> Result<TrieDBIterator<'a, H, C>, H::Out, C::Error> {
		let mut r = TrieDBIterator { db, trail: Vec::with_capacity(8), key_nibbles: Vec::with_capacity(64) };
		db.root_data().and_then(|root| r.descend(&root))?;
		Ok(r)
	}

	fn seek<'key>(&mut self, mut node_data: DBValue, mut key: NibbleSlice<'key>) -> Result<(), H::Out, C::Error> {
		loop {
			let (data, mid) = {
				let node = C::decode(&node_data).expect("encoded data read from db; qed");
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
	fn descend(&mut self, d: &[u8]) -> Result<(), H::Out, C::Error> {
		let node_data = &self.db.get_raw_or_lookup(d)?;
		let node = C::decode(&node_data).expect("encoded node read from db; qed");
		Ok(self.descend_into_node(node.into()))
	}

	/// Descend into a payload.
	fn descend_into_node(&mut self, node: OwnedNode) {
		self.trail.push(Crumb { status: Status::Entering, node });
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

impl<'a, H: Hasher, C: NodeCodec<H>> TrieIterator<H, C> for TrieDBIterator<'a, H, C> {
	/// Position the iterator on the first element with key >= `key`
	fn seek(&mut self, key: &[u8]) -> Result<(), H::Out, C::Error> {
		self.trail.clear();
		self.key_nibbles.clear();
		let root_rlp = self.db.root_data()?;
		self.seek(root_rlp, NibbleSlice::new(key.as_ref()))
	}
}

impl<'a, H: Hasher, C: NodeCodec<H>> Iterator for TrieDBIterator<'a, H, C> {
	type Item = TrieItem<'a, H::Out, C::Error>;

	fn next(&mut self) -> Option<Self::Item> {
		enum IterStep<O, E> {
			Continue,
			PopTrail,
			Descend(Result<DBValue, O, E>),
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
					(Status::At, &OwnedNode::Extension(_, ref d)) => {
						IterStep::Descend::<H::Out, C::Error>(self.db.get_raw_or_lookup(&*d))
					},
					(Status::At, &OwnedNode::Branch(_, _)) => IterStep::Continue,
					(Status::AtChild(i), &OwnedNode::Branch(ref children, _)) if children[i].len() > 0 => {
						match i {
							0 => self.key_nibbles.push(0),
							i => *self.key_nibbles.last_mut()
								.expect("pushed as 0; moves sequentially; removed afterwards; qed") = i as u8,
						}
						IterStep::Descend::<H::Out, C::Error>(self.db.get_raw_or_lookup(&*children[i]))
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
				IterStep::Descend::<H::Out, C::Error>(Ok(d)) => {
					let node = C::decode(&d).expect("encoded data read from db; qed");
					self.descend_into_node(node.into())
				},
				IterStep::Descend::<H::Out, C::Error>(Err(e)) => {
					return Some(Err(e))
				}
				IterStep::Continue => {},
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use hashdb::DBValue;
	use keccak_hasher::KeccakHasher;
	use memorydb::MemoryDB;
	use ethtrie::{TrieDB, TrieDBMut, RlpCodec, trie::{Trie, TrieMut, Lookup}};
	use ethereum_types::H256;

	#[test]
	fn iterator() {
		let d = vec![DBValue::from_slice(b"A"), DBValue::from_slice(b"AA"), DBValue::from_slice(b"AB"), DBValue::from_slice(b"B")];

		let mut memdb = MemoryDB::<KeccakHasher>::new();
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
		let d = vec![ DBValue::from_slice(b"A"), DBValue::from_slice(b"AA"), DBValue::from_slice(b"AB"), DBValue::from_slice(b"B") ];
	
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		{
			let mut t = TrieDBMut::new(&mut memdb, &mut root);
			for x in &d {
				t.insert(x, x).unwrap();
			}
		}
	
		let t = TrieDB::new(&memdb, &root).unwrap();
		let mut iter = t.iter().unwrap();
		assert_eq!(iter.next().unwrap().unwrap(), (b"A".to_vec(), DBValue::from_slice(b"A")));
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
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		{
			let mut t = TrieDBMut::new(&mut memdb, &mut root);
			t.insert(b"A", b"ABC").unwrap();
			t.insert(b"B", b"ABCBA").unwrap();
		}

		let t = TrieDB::new(&memdb, &root).unwrap();
		assert_eq!(t.get_with(b"A", |x: &[u8]| x.len()).unwrap(), Some(3));
		assert_eq!(t.get_with(b"B", |x: &[u8]| x.len()).unwrap(), Some(5));
		assert_eq!(t.get_with(b"C", |x: &[u8]| x.len()).unwrap(), None);
	}

	#[test]
	fn debug_output_supports_pretty_print() {
		let d = vec![ DBValue::from_slice(b"A"), DBValue::from_slice(b"AA"), DBValue::from_slice(b"AB"), DBValue::from_slice(b"B") ];

		let mut memdb = MemoryDB::<KeccakHasher>::new();
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

	#[test]
	fn test_lookup_with_corrupt_data_returns_decoder_error() {
		use rlp;
		use ethereum_types::H512;
		use std::marker::PhantomData;
		use ethtrie::trie::NibbleSlice;

		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		{
			let mut t = TrieDBMut::new(&mut memdb, &mut root);
			t.insert(b"A", b"ABC").unwrap();
			t.insert(b"B", b"ABCBA").unwrap();
		}

		let t = TrieDB::new(&memdb, &root).unwrap();

		// query for an invalid data type to trigger an error
		let q = rlp::decode::<H512>;
		let lookup = Lookup::<_, RlpCodec, _>{ db: t.db(), query: q, hash: root, marker: PhantomData };
		let query_result = lookup.look_up(NibbleSlice::new(b"A"));
		assert_eq!(query_result.unwrap().unwrap().unwrap_err(), rlp::DecoderError::RlpIsTooShort);
	}
}