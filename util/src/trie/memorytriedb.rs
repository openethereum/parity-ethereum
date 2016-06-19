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

use std::ops::{Index, IndexMut};

/// For lookups into the Node storage buffer.
struct StorageHandle(usize);

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

fn empty_children() -> [Option<StorageHandle>; 16] {
	[
		None, None, None, None, None, None, None, None,
		None, None, None, None, None, None, None, None,
	]
}

// attempt to add a child to this child array.
fn add_child(children: &mut [Option<StorageHandle>; 16], storage: &mut NodeStorage, key: &NibbleSlice, value: Bytes) -> Option<Bytes> {
	if key.is_empty() {
		Some(value)
	} else {
		let idx = key.at(0);
		let leaf = Node::Leaf(key.mid(1).encoded(true), value);
		children[idx as usize] = Some(storage.insert(leaf));
		None
	}
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
				Node::Extension(key, storage.insert(child_node))
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
						children[i] = Some(storage.insert(child_node));
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

	// insert a key, value pair into the trie, creating new nodes if necessary.
	fn insert(&mut self, key: NibbleSlice, value: Bytes, storage: &mut NodeStorage) {
		let maybe_new = match *self {
			Node::Empty => {
				Some(Node::Leaf(key.encoded(true), value))
			}
			Node::Leaf(ref existing, ref mut data) => {
				let existing_key = NibbleSlice::from_encoded(existing).0;
				let cp = key.common_prefix(&existing_key);
				if cp == existing_key.len() {
					if cp == key.len() {
						// equivalent leaf. replace
						*data = value;
					} else {
						// shared prefix, make an extension.
						// factor out the common prefix, and then build a branch.
						// the data will be from the shorter of the two keys, and
						// one child with the longer.
						let remainder = key.mid(cp);
						let idx = key.at(0);

						let mut children = empty_children();
						let leaf_node = Node::Leaf(key.mid(1).encoded(true), value);
						children[idx as usize] = Some(storage.insert(leaf_node));
						let branch = Node::Branch(children, data.clone());

						Some(Node::Extension(existing_key.encoded(false), storage.insert(branch)))
					}
				} else {
					// partially shared prefix. make an extension followed by a branch.
					let mid_slice = key.encoded_leftmost(cp, false);
					let key = key.mid(cp);
					let existing_key_partial = existing_key.mid(cp);

					let mut children = empty_children();
					// it is impossible for existing_key_partial to be empty here. then cp
					// would have been equal to existing_key.len(), and we wouldn't be in this branch.
					add_child(&mut children, storage, &existing_key_partial, data.clone());
					let branch = if let Some(value) = add_child(&mut children, storage, &key, value) {
						// key is empty, value needs to be put in branch value.
						Node::Branch(children, Some(value))
					} else {
						Node::Branch(children, None)
					};

					Some(Node::Extension(mid_slice, storage.insert(branch)))
				}
		}
		Node::Extension(ref common, ref mut child_branch) => {
			let common_key = NibbleSlice::from_encoded(common).0;
			let cp = key.common_prefix(&common_key);
			assert!(!common_key.is_empty(), "Extension nodes cannot have empty common prefixes");

			if cp == 0 {
				// make a branch, punting the extension's child branch into it.
			} else if cp == common_key.len() {
				// fully shared key.
			} else {

			}

		}
	};

	if let Some(new_node) = maybe_new {
		*self = new_node;
	}
}

/// Compact and cache-friendly storage for Trie nodes.
struct NodeStorage {
	nodes: Vec<Node>,
}

impl NodeStorage {
	/// Create a new storage with empty root.
	fn empty() -> Self {
		NodeStorage {
			nodes: vec![Node::Empty],
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

	/// Insert a node into the storage, yielding a handle.
	fn insert(&mut self, node: Node) -> StorageHandle {
		self.nodes.push(node);
		StorageHandle(self.nodes.len() - 1)
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
}

impl<'a> MemoryTrieDB<'a> {
	/// Create a new trie with backing database `db` and empty `root`.
	pub fn new(db: &'a mut HashDB, root: &'a mut H256) -> Self {
		*root = SHA3_NULL_RLP;

		MemoryTrieDB {
			storage: NodeStorage::empty(),
			db: db,
			root: root,
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
		self.storage.root_mut().insert(NibbleSlice::new(key), value.to_owned(), &mut self.storage);
	}
}