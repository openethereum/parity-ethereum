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

/// For lookups into the Node storage buffer.
/// This is deliberately non-copyable.
struct StorageHandle(usize);

fn empty_children() -> [Option<StorageHandle>; 16] {
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
	Extension(Bytes, StorageHandle),
	/// A branch has up to 16 children and an optional value.
	Branch([Option<StorageHandle>; 16], Option<Bytes>)
}

impl Node {
	fn get_raw_or_lookup<'a>(node: &'a [u8], db: &'a HashDB) -> &'a [u8] {
		// check if its sha3 + len
		let r = Rlp::new(node);
		match r.is_data() && r.size() == 32 {
			true => db.get(&r.as_val::<H256>()).expect("Not found!"),
			false => node
		}
	}
	// decode a node from rlp. Also loads any inline child nodes into
	// storage.
	fn from_rlp(rlp: &[u8], db: &HashDB, storage: &mut NodeStorage) -> Self {
		match RlpNode::decoded(rlp) {
			RlpNode::Empty => Node::Empty,
			RlpNode::Leaf(k, v) => Node::Leaf(k.encoded(true), v.to_owned()),
			RlpNode::Extension(k, v) => {
				let key = k.encoded(false);
				let child_rlp = Node::get_raw_or_lookup(v, db);
				let child_node = Node::from_rlp(child_rlp, db, storage);
				Node::Extension(key, storage.alloc(child_node))
			}
			RlpNode::Branch(children_rlp, v) => {
				let val = v.map(|x| x.to_owned());
				let mut children = empty_children();

				for i in 0..16 {
					let raw = children_rlp[i];
					let child_rlp = Rlp::new(raw);
					if !child_rlp.is_empty()  {
						let child_rlp = Node::get_raw_or_lookup(raw, db);
						let child_node = Node::from_rlp(child_rlp, db, storage);
						children[i] = Some(storage.alloc(child_node));
					}
				}

				Node::Branch(children, val)
			}
		}
	}

	// walk the trie, attempting to find the key's node.
	fn lookup<'a, 'key>(&'a self, partial: NibbleSlice<'key>, storage: &'a NodeStorage) -> Option<&'a [u8]> {
		match *self {
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
					storage[child].lookup(partial.mid(slice.len()), storage)
				} else {
					None
				}
			}
			Node::Branch(ref children, ref value) => {
				if partial.is_empty() {
					value.as_ref().map(|v| &v[..])
				} else {
					let idx = partial.at(0);
					(&children[idx as usize]).as_ref().and_then(|child| {
						storage[child].lookup(partial.mid(1), storage)
					})
				}
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

	/// insert a key, value pair into the trie, creating new nodes if necessary.
	fn insert(&mut self, handle: StorageHandle, partial: NibbleSlice, value: Bytes) -> StorageHandle {
		match self.destroy(handle) {
			Node::Branch(mut children, mut stored_value) => {
				if partial.is_empty() {
					stored_value = Some(value);
				} else {
					let index = partial.at(0) as usize;
					if let Some(child_handle) = children[index].take() {
						// original had something there. continue to insert.
						let partial = partial.mid(1);
						children[index] = Some(self.insert(child_handle, partial, value));
					} else {
						// original had empty slot. place a leaf there.
						let leaf = Node::Leaf(partial.encoded(true), value);
						children[index] = Some(self.alloc(leaf));
					}
				}

				self.alloc(Node::Branch(children, stored_value))
			}
			Node::Leaf(encoded_key, stored_value) => {
				let (existing_key, _) = NibbleSlice::from_encoded(&encoded_key);
				let cp = partial.common_prefix(&existing_key);
				if cp == partial.len() && cp == existing_key.len() {
					// equivalent leaf: replace
					self.alloc(Node::Leaf(encoded_key.clone(), value))
				} else if cp == 0 {
					// make a branch.
					let mut children = empty_children();
					let branch = if existing_key.is_empty() {
						Node::Branch(children, Some(stored_value))
					} else {
						let index = existing_key.at(0) as usize;
						let leaf = Node::Leaf(existing_key.mid(1).encoded(true), stored_value);
						children[index] = Some(self.alloc(leaf));
						Node::Branch(children, None)
					};

					let temp_handle = self.alloc(branch);
					self.insert(temp_handle, partial, value)
				} else if cp == existing_key.len() {
					// fully shared prefix.
					// transform to an extension + augmented version of onward node.
					let stub_branch = self.alloc(Node::Branch(empty_children(), Some(stored_value)));
					let downstream = self.insert(stub_branch, partial.mid(cp), value);

					self.alloc(Node::Extension(existing_key.encoded(false), downstream))
				} else {
					// partially-shared prefix
					let low = self.alloc(Node::Leaf(existing_key.mid(cp).encoded(true), stored_value));
					let augmented_low = self.insert(low, partial.mid(cp), value);

					self.alloc(Node::Extension(existing_key.encoded_leftmost(cp, false), augmented_low))
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
						children[index] = Some(self.alloc(extension));
					}

					let temp_branch = self.alloc(Node::Branch(children, None));
					self.insert(temp_branch, partial, value)
				} else if cp == existing_key.len() {
					// fully-shared prefix.
					let downstream = self.insert(child_branch, partial.mid(cp), value);
					self.alloc(Node::Extension(existing_key.encoded(false), downstream))
				} else {
					// partially-shared prefix
					let low = self.alloc(Node::Extension(existing_key.mid(cp).encoded(false), child_branch));
					let augmented_low = self.insert(low, partial.mid(cp), value);

					self.alloc(Node::Extension(existing_key.encoded_leftmost(cp, false), augmented_low))
				}
			}
			Node::Empty => {
				self.alloc(Node::Leaf(partial.encoded(true), value))
			}
		}
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
		self.storage.root().lookup(NibbleSlice::new(key), &self.storage)
	}

	fn contains(&self, key: &[u8]) -> bool {
		self.get(key).is_some()
	}
}

impl<'a> TrieMut for MemoryTrieDB<'a> {
	fn insert(&mut self, key: &[u8], value: &[u8]) {
		// TODO [rob]: find a way to do this that doesn't subvert the ownership
		// semantics of StorageHandle
		let root_handle = StorageHandle(self.root_handle.0);
		self.root_handle = self.storage.insert(root_handle, NibbleSlice::new(key), value.to_owned());
	}

	fn remove(&mut self, key: &[u8]) {
		unimplemented!()
	}
}