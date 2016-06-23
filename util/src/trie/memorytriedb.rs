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
use std::mem;
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

	// encode a node to RLP. if it has child nodes, remove them from the storage,
	// encode them, and write them to the DB.
	//
	// TODO: parallelize
	fn to_rlp(self, db: &mut HashDB, storage: &mut NodeStorage) -> Bytes {
		use ::rlp::{RlpStream, Stream};

		match self {
			Node::Empty => {
				let mut stream = RlpStream::new();
				stream.append_empty_data();
				stream.out()
			}
			Node::Leaf(k, v) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&k);
				stream.append(&v);
				stream.out()
			}
			Node::Extension(partial, child_handle) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&partial);

				match child_handle {
					NodeHandle::InMemory(s_handle) => {
						let node = storage.destroy(s_handle);
						let rlp = node.to_rlp(db, storage);
						if rlp.len() >= 32 {
							let hash = db.insert(&rlp);
							stream.append(&hash);
						} else {
							stream.append_raw(&rlp, 1);
						}
					}
					NodeHandle::Hash(hash) => {
						stream.append(&hash);
					}
				}

				stream.out()
			}
			Node::Branch(mut children, value) => {
				let mut stream = RlpStream::new_list(17);
				// no moving iterators for arrays
				for child in children.iter_mut().map(Option::take) {
					match child {
						Some(handle) => match handle {
							NodeHandle::InMemory(s_handle) => {
								let node = storage.destroy(s_handle);
								let rlp = node.to_rlp(db, storage);
								if rlp.len() >= 32 {
									// too big,
									let hash = db.insert(&rlp);
									stream.append(&hash)
								} else {
									// inline node.
									stream.append_raw(&rlp, 1)
								}
							}
							NodeHandle::Hash(hash) => stream.append(&hash),
						},
						None => stream.append_empty_data(),
					};
				}

				match value {
					Some(value) => stream.append(&value),
					None => stream.append_empty_data(),
				};

				stream.out()
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
		mem::replace(&mut self.nodes[idx], Node::Empty)
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
		self.db.get(hash).and_then(|node_rlp| self.get_from_db_node(&node_rlp, key))
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

		let new_node = match mem::replace(&mut self.storage[&handle], Node::Empty) {
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

				Node::Branch(children, stored_value)
			}
			Node::Leaf(encoded_key, stored_value) => {
				let (existing_key, _) = NibbleSlice::from_encoded(&encoded_key);
				let cp = partial.common_prefix(&existing_key);
				if cp == partial.len() && cp == existing_key.len() {
					// equivalent leaf: replace
					Node::Leaf(encoded_key.clone(), stored_value)
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

					// replace this node with the new branch, and then walk it further.
					self.storage[&handle] = branch;
					return self.insert(handle.into(), partial, value);
				} else if cp == existing_key.len() {
					// fully shared prefix.
					// transform to an extension + augmented version of onward node.
					let stub_branch = self.storage.alloc(Node::Branch(empty_children(), Some(stored_value)));
					let downstream = self.insert(stub_branch.into(), partial.mid(cp), value);

					Node::Extension(existing_key.encoded(false), downstream.into())
				} else {
					// partially-shared prefix
					let low = self.storage.alloc(Node::Leaf(existing_key.mid(cp).encoded(true), stored_value));
					let augmented_low = self.insert(low.into(), partial.mid(cp), value);

					Node::Extension(existing_key.encoded_leftmost(cp, false), augmented_low.into())
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

					// replace this node with a branch and walk it further.
					self.storage[&handle] = Node::Branch(children, None);
					return self.insert(handle.into(), partial, value);
				} else if cp == existing_key.len() {
					// fully-shared prefix.
					let downstream = self.insert(child_branch, partial.mid(cp), value);
					Node::Extension(existing_key.encoded(false), downstream.into())
				} else {
					// partially-shared prefix
					let low = self.storage.alloc(Node::Extension(existing_key.mid(cp).encoded(false), child_branch));
					let augmented_low = self.insert(low.into(), partial.mid(cp), value);

					Node::Extension(existing_key.encoded_leftmost(cp, false), augmented_low.into())
				}
			}
			Node::Empty => {
				Node::Leaf(partial.encoded(true), value)
			}
		};

		self.storage[&handle] = new_node;
		handle
	}

	/// Remove a node from the trie based on key.
	fn remove(&mut self, handle: NodeHandle, partial: NibbleSlice) -> Option<StorageHandle> {
		// load the node we're inspecting into memory if it isn't already.
		let handle = match handle {
			NodeHandle::Hash(hash) => {
				let node_rlp = self.db.lookup(&hash).expect("Not found!");
				let node = Node::from_rlp(node_rlp, &*self.db, &mut self.storage);
				self.storage.alloc(node)
			}
			NodeHandle::InMemory(h) => h,
		};

		let new_node = match (mem::replace(&mut self.storage[&handle], Node::Empty), partial.is_empty()) {
			(Node::Empty, _) => Node::Empty,
			(Node::Branch(children, None), true) => Node::Branch(children, None), // already gone.
			(Node::Branch(children, _), true) => self.fix(Node::Branch(children, None)),
			(Node::Branch(mut children, value), false) => {
				let index = partial.at(0) as usize;
				let temp = children[index]
					.take().and_then(|child| self.remove(child, partial.mid(1)));

				children[index] = temp.map(|x| x.into());

				self.fix(Node::Branch(children, value))
			}
			(Node::Leaf(encoded_key, value), _) => {
				if NibbleSlice::from_encoded(&encoded_key).0 == partial {
					// this is the node we were trying to delete. delete it.
					self.storage.destroy(handle);
					return None;
				} else {
					// leaf the node alone
					Node::Leaf(encoded_key, value)
				}
			}
			(Node::Extension(encoded_key, child_branch), _) => {
				let (existing_len, cp) = {
					let existing_key = NibbleSlice::from_encoded(&encoded_key).0;
					(existing_key.len(), existing_key.common_prefix(&partial))
				};
				if cp == existing_len {
					match self.remove(child_branch, partial.mid(cp)) {
						Some(new_child) => self.fix(Node::Extension(encoded_key, new_child.into())),
						None => panic!("extension child is a branch; remove on a branch node always returns Some; qed")
					}
				} else {
					// key in the middle of an extension. ignore and return old node
					Node::Extension(encoded_key, child_branch)
				}
			}
		};

		self.storage[&handle] = new_node;
		Some(handle)
	}

	/// Given a node which may be in an _invalid state_, fix it such that it is then in a valid
	/// state.
	///
	/// _invalid state_ means:
	/// - Branch node where there is only a single entry;
	/// - Extension node followed by anything other than a Branch node.
	fn fix(&mut self, node: Node) -> Node {
		match node {
			Node::Branch(mut children, value) => {
				#[derive(Debug)]
				enum UsedIndex {
					None,
					One(usize),
					Many,
				}
				let mut used_index = UsedIndex::None;
				for i in 0..16 {
					match (children[i].is_some(), &used_index) {
						(false, _) => continue,
						(true, &UsedIndex::None) => used_index = UsedIndex::One(i),
						(true, &UsedIndex::One(_)) => {
							used_index = UsedIndex::Many;
							break;
						}
						_ => {}
					}
				}

				match (used_index, value) {
					(UsedIndex::None, None) => panic!("Branch with no subvalues. Something went wrong."),
					(UsedIndex::One(i), None) => {
						// turn into an extension and fix it (the onward node isn't necessarily a branch)
						let onward = children[i].take().unwrap();
						self.fix(Node::Extension(NibbleSlice::new(&[i as u8; 1]).encoded(false), onward))
					}
					(UsedIndex::None, Some(value)) => {
						// turn into a leaf.
						Node::Leaf(NibbleSlice::new(&[]).encoded(true), value)
					}
					(_, value) => Node::Branch(children, value) // all good here
				}
			}
			Node::Extension(partial, child) => {
				// load the child into memory if it isn't already.
				let child = match child {
					NodeHandle::Hash(hash) => {
						let node_rlp = self.db.lookup(&hash).expect("Not found!");
						Node::from_rlp(node_rlp, &*self.db, &mut self.storage)
					}
					NodeHandle::InMemory(h) => self.storage.destroy(h),
				};

				// inspect the child
				match child {
					Node::Extension(sub_partial, sub_child) => {
						// combine with node below.
						let partial = NibbleSlice::from_encoded(&partial).0;
						let sub_partial = NibbleSlice::from_encoded(&sub_partial).0;

						let composed = Node::Extension(NibbleSlice::new_composed(&partial, &sub_partial).encoded(false), sub_child);
						self.fix(composed)
					}
					Node::Leaf(sub_partial, sub_value) => {
						// combine with node below.
						let partial = NibbleSlice::from_encoded(&partial).0;
						let sub_partial = NibbleSlice::from_encoded(&sub_partial).0;
						// combine with node below.
						Node::Leaf(NibbleSlice::new_composed(&partial, &sub_partial).encoded(true), sub_value)
					}
					// nothing wrong here. reallocate the child and move on.
					_ => Node::Extension(partial, self.storage.alloc(child).into()),
				}
			}
			node => node,
		}
	}

	/// Commit the in-memory changes to disk, freeing their storage and
	/// updating the state root.
	pub fn commit(&mut self) {
		let root_handle = self.root_handle();
		let root_node = mem::replace(&mut self.storage[&root_handle], Node::Empty);
		let root_rlp = root_node.to_rlp(self.db, &mut self.storage);

		*self.root = self.db.insert(&root_rlp);

		// reload the root node from the rlp just in case someone keeps using this trie after
		// commit.
		let root_node = Node::from_rlp(&root_rlp, &*self.db, &mut self.storage);
		self.storage[&root_handle] = root_node;
	}

	// a hack to get the root node's handle
	fn root_handle(&self) -> StorageHandle {
		StorageHandle(self.root_handle.0)
	}
}

impl<'a> Trie for MemoryTrieDB<'a> {
	// TODO [rob] do something about the root not being consistent with trie state until commit.
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
		let root_handle = self.root_handle();
		self.lookup(NibbleSlice::new(key), &NodeHandle::InMemory(root_handle))
	}

	fn contains(&self, key: &[u8]) -> bool {
		self.get(key).is_some()
	}
}

impl<'a> TrieMut for MemoryTrieDB<'a> {
	fn insert(&mut self, key: &[u8], value: &[u8]) {
		let root_handle = self.root_handle();
		self.insert(root_handle.into(), NibbleSlice::new(key), value.to_owned());
	}

	fn remove(&mut self, key: &[u8]) {
		let root_handle = self.root_handle();
		if self.remove(root_handle.into(), NibbleSlice::new(key)).is_none() {
			self.root_handle = self.storage.alloc(Node::Empty);
		}
	}
}

impl<'a> Drop for MemoryTrieDB<'a> {
	fn drop(&mut self) {
		self.commit();
	}
}