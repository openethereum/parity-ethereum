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

//! In-memory trie representation.

use super::{Trie, TrieError, TrieMut};
use super::node::Node as RlpNode;

use ::{Bytes, FixedHash, HashDB, H256, SHA3_NULL_RLP};
use ::nibbleslice::NibbleSlice;
use ::rlp::{Rlp, View};

use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

// For lookups into the Node storage buffer.
// This is deliberately non-copyable.
struct StorageHandle(usize);

// Handles to nodes in the trie.
enum NodeHandle {
	/// Loaded into memory.
	InMemory(StorageHandle),
	/// Either a hash or an inline node
	Hash(H256),
}

impl From<StorageHandle> for NodeHandle {
	fn from(handle: StorageHandle) -> Self {
		NodeHandle::InMemory(handle)
	}
}

impl From<H256> for NodeHandle {
	fn from(hash: H256) -> Self {
		NodeHandle::Hash(hash)
	}
}

fn empty_children() -> [Option<NodeHandle>; 16] {
	[
		None, None, None, None, None, None, None, None,
		None, None, None, None, None, None, None, None,
	]
}

/// Node types in the Trie.
enum Node {
	/// Empty node.
	Empty,
	/// A leaf node contains the end of a key and a value.
	/// This key is encoded from a `NibbleSlice`, meaning it contains
	/// a flag indicating it is a leaf.
	Leaf(Bytes, Bytes),
	/// An extension contains a shared portion of a key and a child node.
	/// The shared portion is encoded from a `NibbleSlice` meaning it contains
	/// a flag indicating it is an extension.
	/// The child node is always a branch.
	Extension(Bytes, NodeHandle),
	/// A branch has up to 16 children and an optional value.
	Branch([Option<NodeHandle>; 16], Option<Bytes>)
}

impl Node {
	// load an inline node into memory or get the hash to do the lookup later.
	fn inline_or_hash(node: &[u8], db: &HashDB, storage: &mut NodeStorage) -> NodeHandle {
		let r = Rlp::new(node);
		if r.is_data() && r.size() < 32 {
			let child = Node::from_rlp(r.data(), db, storage);
			NodeHandle::InMemory(storage.alloc(child))
		} else {
			NodeHandle::Hash(r.as_val::<H256>())
		}
	}

	// decode a node from rlp without getting its children.
	fn from_rlp(rlp: &[u8], db: &HashDB, storage: &mut NodeStorage) -> Self {
		match RlpNode::decoded(rlp) {
			RlpNode::Empty => Node::Empty,
			RlpNode::Leaf(k, v) => Node::Leaf(k.encoded(true), v.to_owned()),
			RlpNode::Extension(k, v) => {
				let key = k.encoded(false);

				Node::Extension(key, Self::inline_or_hash(v, db, storage))
			}
			RlpNode::Branch(children_rlp, v) => {
				let val = v.map(|x| x.to_owned());
				let mut children = empty_children();

				for i in 0..16 {
					let raw = children_rlp[i];
					let child_rlp = Rlp::new(raw);
					if !child_rlp.is_empty()  {
						children[i] = Some(Self::inline_or_hash(raw, db, storage));
					}
				}

				Node::Branch(children, val)
			}
		}
	}
}

/// Compact and cache-friendly storage for Trie nodes.
struct NodeStorage {
	nodes: Vec<Node>,
	free_indices: VecDeque<usize>,
}

impl NodeStorage {
	/// Create a new storage with empty root.
	fn empty() -> Self {
		NodeStorage {
			nodes: vec![Node::Empty],
			free_indices: VecDeque::new(),
		}
	}

	/// Create storage from root rlp.
	fn from_root_rlp(rlp: &[u8], db: &HashDB) -> Self {
		// reserve a slot for the root.
		let mut storage = NodeStorage::empty();

		// decode and overwrite.
		let root_node = Node::from_rlp(rlp, db, &mut storage);
		*storage.root_mut() = root_node;

		storage
	}

	/// Get a reference to the root node.
	fn root(&self) -> &Node {
		&self.nodes[0]
	}

	/// Get a mutable reference to the root node.
	fn root_mut(&mut self) -> &mut Node {
		&mut self.nodes[0]
	}

	/// Allocate  a node in the storage, yielding a handle.
	fn alloc(&mut self, node: Node) -> StorageHandle {
		if let Some(idx) = self.free_indices.pop_front() {
			self.nodes[idx] = node;
			StorageHandle(idx)
		} else {
			self.nodes.push(node);
			StorageHandle(self.nodes.len() - 1)
		}
	}

	/// Remove a node from the storage, consuming the handle.
	fn destroy(&mut self, handle: StorageHandle) -> Node {
		let idx = handle.0;

		self.free_indices.push_back(idx);
		::std::mem::replace(&mut self.nodes[idx], Node::Empty)
	}
}

impl<'a> Index<&'a StorageHandle> for NodeStorage {
	type Output = Node;

	fn index(&self, x: &'a StorageHandle) -> &Node {
		&self.nodes[x.0]
	}
}

impl<'a> IndexMut<&'a StorageHandle> for NodeStorage {
	fn index_mut(&mut self, x: &'a StorageHandle) -> &mut Node {
		&mut self.nodes[x.0]
	}
}

/// In memory trie representation.
///
/// This has the same API and behavior as `TrieDBMut`, but performs operations
/// on the trie fully in memory without RLP representation. It will encode to
/// rlp, commit to the database, and update the state root reference when `commit`
/// is explicitly called.
pub struct MemoryTrieDB<'a> {
	storage: NodeStorage,
	db: &'a mut HashDB,
	root: &'a mut H256,
	root_handle: StorageHandle,
}

impl<'a> MemoryTrieDB<'a> {
	/// Create a new trie with backing database `db` and empty `root`.
	pub fn new(db: &'a mut HashDB, root: &'a mut H256) -> Self {
		*root = SHA3_NULL_RLP;

		MemoryTrieDB {
			storage: NodeStorage::empty(),
			db: db,
			root: root,
			root_handle: StorageHandle(0),
		}
	}

	/// Create a new trie with the backing database `db` and `root.
	/// Returns an error if `root` does not exist.
	pub fn from_existing(db: &'a mut HashDB, root: &'a mut H256) -> Result<Self, TrieError> {
		let storage = {
			let root_rlp = match db.get(root) {
				Some(root_rlp) => root_rlp,
				None => return Err(TrieError::InvalidStateRoot),
			};

			NodeStorage::from_root_rlp(root_rlp, db)
		};

		Ok(MemoryTrieDB {
			storage: storage,
			db: db,
			root: root,
			root_handle: StorageHandle(0),
		})
	}

	// walk the trie, attempting to find the key's node.
	fn lookup<'x, 'key>(&'x self, partial: NibbleSlice<'key>, handle: &NodeHandle) -> Option<&'x [u8]>
	where 'x: 'key {
		match *handle {
			NodeHandle::Hash(ref hash) => self.do_db_lookup(hash, partial),
			NodeHandle::InMemory(ref handle) => match self.storage[handle] {
				Node::Empty => None,
				Node::Leaf(ref key, ref value) => {
					if NibbleSlice::from_encoded(key).0 == partial {
						Some(value)
					} else {
						None
					}
				}
				Node::Extension(ref slice, ref child) => {
					let slice = NibbleSlice::from_encoded(slice).0;
					if partial.starts_with(&slice) {
						self.lookup(partial.mid(slice.len()), child)
					} else {
						None
					}
				}
				Node::Branch(ref children, ref value) => {
					if partial.is_empty() {
						value.as_ref().map(|v| &v[..])
					} else {
						let idx = partial.at(0);
						(&children[idx as usize]).as_ref().and_then(|child| self.lookup(partial.mid(1), child))
					}
				}
			}
		}
	}

	/// Return optional data for a key given as a `NibbleSlice`. Returns `None` if no data exists.
	fn do_db_lookup<'x, 'key>(&'x self, hash: &H256, key: NibbleSlice<'key>) -> Option<&'x [u8]> where 'x: 'key {
		let node_rlp = self.db.get(hash).expect("Not found!");
		self.get_from_db_node(&node_rlp, key)
	}

	/// Recursible function to retrieve the value given a `node` and a partial `key`. `None` if no
	/// value exists for the key.
	///
	/// Note: Not a public API; use Trie trait functions.
	fn get_from_db_node<'x, 'key>(&'x self, node: &'x [u8], key: NibbleSlice<'key>) -> Option<&'x [u8]> where 'x: 'key {
		match RlpNode::decoded(node) {
			RlpNode::Leaf(ref slice, ref value) if &key == slice => Some(value),
			RlpNode::Extension(ref slice, ref item) if key.starts_with(slice) => {
				self.get_from_db_node(self.get_raw_or_lookup(item), key.mid(slice.len()))
			},
			RlpNode::Branch(ref nodes, value) => match key.is_empty() {
				true => value,
				false => self.get_from_db_node(self.get_raw_or_lookup(nodes[key.at(0) as usize]), key.mid(1))
			},
			_ => None
		}
	}

	/// Given some node-describing data `node`, return the actual node RLP.
	/// This could be a simple identity operation in the case that the node is sufficiently small, but
	/// may require a database lookup.
	fn get_raw_or_lookup<'x>(&'x self, node: &'x [u8]) -> &'x [u8] {
		// check if its sha3 + len
		let r = Rlp::new(node);
		match r.is_data() && r.size() == 32 {
			true => self.db.lookup(&r.as_val::<H256>()).expect("Not found!"),
			false => node
		}
	}

	/// insert a key, value pair into the trie, creating new nodes if necessary.
	fn insert(&mut self, handle: NodeHandle, partial: NibbleSlice, value: Bytes) -> StorageHandle {
		let handle = match handle {
			// load the node if we haven't already,
			NodeHandle::Hash(hash) => {
				let node_rlp = self.db.lookup(&hash).expect("Not found!");
				let node = Node::from_rlp(node_rlp, &*self.db, &mut self.storage);
				self.storage.alloc(node)
			}
			NodeHandle::InMemory(h) => h,
		};

		match self.storage.destroy(handle) {
			Node::Branch(mut children, mut stored_value) => {
				if partial.is_empty() {
					stored_value = Some(value);
				} else {
					let index = partial.at(0) as usize;
					if let Some(child_handle) = children[index].take() {
						// original had something there. continue to insert.
						let partial = partial.mid(1);
						children[index] = Some(self.insert(child_handle, partial, value).into());
					} else {
						// original had empty slot. place a leaf there.
						let leaf = Node::Leaf(partial.encoded(true), value);
						children[index] = Some(self.storage.alloc(leaf).into());
					}
				}

				self.storage.alloc(Node::Branch(children, stored_value))
			}
			Node::Leaf(encoded_key, stored_value) => {
				let (existing_key, _) = NibbleSlice::from_encoded(&encoded_key);
				let cp = partial.common_prefix(&existing_key);
				if cp == partial.len() && cp == existing_key.len() {
					// equivalent leaf: replace
					self.storage.alloc(Node::Leaf(encoded_key.clone(), value))
				} else if cp == 0 {
					// make a branch.
					let mut children = empty_children();
					let branch = if existing_key.is_empty() {
						Node::Branch(children, Some(stored_value))
					} else {
						let index = existing_key.at(0) as usize;
						let leaf = Node::Leaf(existing_key.mid(1).encoded(true), stored_value);
						children[index] = Some(self.storage.alloc(leaf).into());
						Node::Branch(children, None)
					};

					let temp_handle = self.storage.alloc(branch);
					self.insert(temp_handle.into(), partial, value)
				} else if cp == existing_key.len() {
					// fully shared prefix.
					// transform to an extension + augmented version of onward node.
					let stub_branch = self.storage.alloc(Node::Branch(empty_children(), Some(stored_value)));
					let downstream = self.insert(stub_branch.into(), partial.mid(cp), value);

					self.storage.alloc(Node::Extension(existing_key.encoded(false), downstream.into()))
				} else {
					// partially-shared prefix
					let low = self.storage.alloc(Node::Leaf(existing_key.mid(cp).encoded(true), stored_value));
					let augmented_low = self.insert(low.into(), partial.mid(cp), value);

					self.storage.alloc(Node::Extension(existing_key.encoded_leftmost(cp, false), augmented_low.into()))
				}
			}
			Node::Extension(encoded_key, child_branch) => {
				let (existing_key, _) = NibbleSlice::from_encoded(&encoded_key);
				let cp = partial.common_prefix(&existing_key);
				if cp == 0 {
					// make a branch.
					assert!(!existing_key.is_empty()); // extension nodes may not have empty partial keys.
					let mut children = empty_children();
					let index = existing_key.at(0) as usize;
					if existing_key.len() == 1 {
						// direct extension
						children[index] = Some(child_branch);
					} else {
						let extension = Node::Extension(existing_key.mid(1).encoded(false), child_branch);
						children[index] = Some(self.storage.alloc(extension).into());
					}

					let temp_branch = self.storage.alloc(Node::Branch(children, None));
					self.insert(temp_branch.into(), partial, value)
				} else if cp == existing_key.len() {
					// fully-shared prefix.
					let downstream = self.insert(child_branch, partial.mid(cp), value);
					self.storage.alloc(Node::Extension(existing_key.encoded(false), downstream.into()))
				} else {
					// partially-shared prefix
					let low = self.storage.alloc(Node::Extension(existing_key.mid(cp).encoded(false), child_branch));
					let augmented_low = self.insert(low.into(), partial.mid(cp), value);

					self.storage.alloc(Node::Extension(existing_key.encoded_leftmost(cp, false), augmented_low.into()))
				}
			}
			Node::Empty => {
				self.storage.alloc(Node::Leaf(partial.encoded(true), value))
			}
		}
	}
}

impl<'a> Trie for MemoryTrieDB<'a> {
	// TODO [rob] do something about the root not being consistent with trie state.
	fn root(&self) -> &H256 {
		&self.root
	}

	fn is_empty(&self) -> bool {
		match *self.storage.root() {
			Node::Empty => true,
			_ => false,
		}
	}

	fn get<'b, 'key>(&'b self, key: &'key [u8]) -> Option<&'b [u8]> where 'b: 'key {
		let root_handle = StorageHandle(self.root_handle.0);
		self.lookup(NibbleSlice::new(key), &NodeHandle::InMemory(root_handle))
	}

	fn contains(&self, key: &[u8]) -> bool {
		self.get(key).is_some()
	}
}

impl<'a> TrieMut for MemoryTrieDB<'a> {
	fn insert(&mut self, key: &[u8], value: &[u8]) {
		let root_handle = StorageHandle(self.root_handle.0);
		// insert the leaf and update the in-memory root.
		self.root_handle = self.insert(root_handle.into(), NibbleSlice::new(key), value.to_owned());
	}

	fn remove(&mut self, key: &[u8]) {
		unimplemented!()
	}
}