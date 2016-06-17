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
use ::rlp::{Rlp, View};

use std::ops::{Index, IndexMut};

/// For lookups into the Node storage buffer.
struct StorageHandle(usize);

/// Lazily loaded child nodes.
enum ChildNode {
	/// A hash of a node we can look up with.
	Hash(H256),
	/// A handle into the trie storage
	Node(StorageHandle),
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
	Extension(Bytes, ChildNode),
	/// A branch has up to 16 children and an optional value.
	Branch([Option<ChildNode>; 16], Option<Bytes>)
}

impl Node {
	// decode a node from rlp. Also loads any inline child nodes into
	// storage.
	fn from_rlp(rlp: &[u8], storage: &mut NodeStorage) -> Self {
		match RlpNode::decoded(rlp) {
			RlpNode::Empty => Node::Empty,
			RlpNode::Leaf(k, v) => Node::Leaf(k.encoded(true), v.to_owned()),
			RlpNode::Extension(k, v) => {
				let key = k.encoded(false);
				// inline node.
				if v.len() < 32 {
					let child = Node::from_rlp(v, storage);
					Node::Extension(key, ChildNode::Node(storage.insert(child)))
				} else {
					Node::Extension(key, ChildNode::Hash(Rlp::new(v).as_val()))
				}
			}
			RlpNode::Branch(children_rlp, v) => {
				let val = v.map(|x| x.to_owned());
				let mut children = [
					None, None, None, None, None, None, None, None,
					None, None, None, None, None, None, None, None,
				];

				for i in 0..16 {
					let raw = children_rlp[i];
					let child_rlp = Rlp::new(raw);
					if !child_rlp.is_empty()  {
						// inline node.
						if raw.len() < 32 {
							let child = Node::from_rlp(raw, storage);
							children[i] = Some(ChildNode::Node(storage.insert(child)));
						} else {
							children[i] = Some(ChildNode::Hash(child_rlp.as_val()));
						}
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
}

impl NodeStorage {
	/// Create a new storage with empty root.
	fn empty() -> Self {
		NodeStorage {
			nodes: vec![Node::Empty],
		}
	}

	/// Create storage from root rlp.
	fn from_root_rlp(rlp: &[u8]) -> Self {
		// reserve a slot for the root.
		let mut storage = NodeStorage::empty();

		// decode and overwrite.
		let root_node = Node::from_rlp(rlp, &mut storage);
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

		MemoryTrie {
			storage: NodeStorage::empty(),
			db: db,
			root: root,
		}
	}

	/// Create a new trie with the backing database `db` and `root.
	/// Returns an error if `root` does not exist.
	pub fn from_existing(db: &'a mut HashDB, root: &'a mut H256) -> Result<Self, TrieError> {
		let storage = match db.get(root) {
			Some(root_rlp) => NodeStorage::from_root_rlp(root_rlp),
			None => return Err(TrieError::InvalidStateRoot),
		};

		Ok(MemoryTrie {
			storage: storage,
			db: db,
			root: root,
		})
	}
}