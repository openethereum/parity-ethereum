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
use ::bytes::ToPretty;
use ::nibbleslice::NibbleSlice;
use ::rlp::{Rlp, View};

use elastic_array::ElasticArray1024;

use std::collections::VecDeque;
use std::mem;
use std::ops::{Index, IndexMut};

// For lookups into the Node storage buffer.
// This is deliberately non-copyable.
#[derive(Debug)]
struct StorageHandle(usize);

// Handles to nodes in the trie.
#[derive(Debug)]
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
#[derive(Debug)]
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
		if r.is_data() && r.size() == 32 {
			NodeHandle::Hash(r.as_val::<H256>())
		} else {
			let child = Node::from_rlp(node, db, storage);
			NodeHandle::InMemory(storage.alloc(child))
		}
	}

	// decode a node from rlp without getting its children.
	fn from_rlp(rlp: &[u8], db: &HashDB, storage: &mut NodeStorage) -> Self {
		match RlpNode::decoded(rlp) {
			RlpNode::Empty => Node::Empty,
			RlpNode::Leaf(k, v) => Node::Leaf(k.encoded(true), v.to_owned()),
			RlpNode::Extension(partial, cb) => {
				let key = partial.encoded(false);

				Node::Extension(key, Self::inline_or_hash(cb, db, storage))
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
	// encode them, and write them to the DB. returns the rlp and the number of
	// hash operations performed.
	//
	// TODO: parallelize
	fn to_rlp(self, db: &mut HashDB, storage: &mut NodeStorage) -> (ElasticArray1024<u8>, usize) {
		use ::rlp::{RlpStream, Stream};

		match self {
			Node::Empty => {
				let mut stream = RlpStream::new();
				stream.append_empty_data();
				(stream.drain(), 0)
			}
			Node::Leaf(k, v) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&k);
				stream.append(&v);
				(stream.drain(), 0)
			}
			Node::Extension(partial, child_handle) => {
				let mut stream = RlpStream::new_list(2);
				let mut hashes = 0;

				stream.append(&partial);

				match child_handle {
					NodeHandle::InMemory(s_handle) => {
						let node = storage.destroy(s_handle);
						let (rlp, h) = node.to_rlp(db, storage);
						hashes += h;
						if rlp.len() >= 32 {
							let hash = db.insert(&rlp);
							hashes += 1;
							stream.append(&hash);
						} else {
							stream.append_raw(&rlp, 1);
						}
					}
					NodeHandle::Hash(hash) => {
						stream.append(&hash);
					}
				}

				(stream.drain(), hashes)
			}
			Node::Branch(mut children, value) => {
				let mut stream = RlpStream::new_list(17);
				let mut hashes = 0;
				// no moving iterators for arrays
				for child in children.iter_mut().map(Option::take) {
					match child {
						Some(handle) => match handle {
							NodeHandle::InMemory(s_handle) => {
								let node = storage.destroy(s_handle);
								let (rlp, h) = node.to_rlp(db, storage);
								hashes += h;
								if rlp.len() >= 32 {
									// too big,
									let hash = db.insert(&rlp);
									hashes += 1;
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

				(stream.drain(), hashes)
			}
		}
	}
}

// post-inspect action.
enum Action {
	// Replace a node with a new one.
	Replace(Node),
	// Restore the old node. trusts that the node is actually new.
	Restore(Node),
	// Delete the node. If it is a cached node, will slate it for removal.
	// if it is a new node, just clears the storage.
	Delete,
}

// What kind of node is stored here.
enum Stored {
	// A new node.
	New(Node),
	// A cached node, loaded from the DB.
	Cached(Node, Hash),
}

// maybe changed handle, maybe not.
enum MaybeChanged {
	Same(StorageHandle),
	Changed(StorageHandle),
	Deleted,
}

/// Compact and cache-friendly storage for Trie nodes.
struct NodeStorage {
	nodes: Vec<Stored>,
	free_indices: VecDeque<usize>,
}

impl NodeStorage {
	/// Create a new storage.
	fn empty() -> Self {
		NodeStorage {
			nodes: Vec::new(),
			free_indices: VecDeque::new(),
		}
	}

	/// Allocate a new node in the storage.
	fn alloc(&mut self, stored: Stored) -> StorageHandle {
		if let Some(idx) = self.free_indices.pop_front() {
			self.nodes[idx] = stored;
			StorageHandle(idx)
		} else {
			self.nodes.push(stored);
			StorageHandle(self.nodes.len() - 1)
		}
	}

	/// Remove a node from the storage, consuming the handle.
	fn destroy(&mut self, handle: StorageHandle) {
		let idx = handle.0;

		self.free_indices.push_back(idx);
		self.nodes[idx] = Stored::New(Node::Empty);
	}

	/// Borrow a "stored" instance mutably.
	fn borrow_stored(&mut self, handle: &StorageHandle) -> &mut Stored {
		&mut self.nodes[handle.0]
	}
}

impl<'a> Index<&'a StorageHandle> for NodeStorage {
	type Output = Node;

	fn index(&self, handle: &'a StorageHandle) -> &Node {
		match self.nodes[handle.0] {
			Stored::New(ref node) => node,
			Stored::Cached(ref node, _) => node,
		}
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
	dirty: bool,
	death_row: Vec<H256>,
	/// The number of hash operations this trie has performed.
	/// Note that none are performed until changes are committed.
	pub hash_count: usize,
}

impl<'a> MemoryTrieDB<'a> {
	/// Create a new trie with backing database `db` and empty `root`.
	pub fn new(db: &'a mut HashDB, root: &'a mut H256) -> Self {
		*root = SHA3_NULL_RLP;
		let mut storage = NodeStorage::empty();
		let root_handle = storage.alloc(Stored::New(Node::Empty));

		MemoryTrieDB {
			storage: NodeStorage::empty(),
			db: db,
			root: root,
			root_handle: root_handle,
			dirty: false,
			death_row: Vec::new(),
			hash_count: 0,
		}
	}

	/// Create a new trie with the backing database `db` and `root.
	/// Returns an error if `root` does not exist.
	pub fn from_existing(db: &'a mut HashDB, root: &'a mut H256) -> Result<Self, TrieError> {
		let mut storage = NodeStorage::new();
		let root_handle = {
			let root_rlp = match db.get(root) {
				Some(root_rlp) => root_rlp,
				None => return Err(TrieError::InvalidStateRoot),
			};

			let root_node = Node::from_rlp(root_rlp, db, &mut storage);
			storage.alloc(Stored::Cached(root_node, *root))
		};

		Ok(MemoryTrieDB {
			storage: storage,
			db: db,
			root: root,
			root_handle: root_handle,
			dirty: false,
			death_row: Vec::new(),
			hash_count: 0,
		})
	}

	// cache a node by hash
	fn cache(&mut self, hash: H256) -> StorageHandle {
		let node_rlp = self.db.get(&hash).expect("Not found!");
		let node = Node::from_rlp(node_rlp, &*self.db, &mut self.storage);
		self.storage.alloc(Stored::Cached(node, hash))
	}

	// inspect a node, performing an action.
	fn inspect_node<F>(&mut self, handle: NodeHandle, inspector: F) -> MaybeChanged
		where F: FnOnce(&mut Self, Node) -> Action {
		let s_handle = match handle {
			NodeHandle::InMemory(h) => h,
			NodeHandle::Hash(h) => self.cache(h),
		};

		let mut stored = mem::replace(self.storage.borrow_stored(&s_handle), Stored::New(Node::Empty));

		match stored {
			Stored::New(node) => match inspector(self, node) {
				Action::Replace(node) => {
					*self.storage.borrow_stored(&s_handle) = Stored::New(node);
					MaybeChanged::Changed(s_handle)
				}
				Action::Restore(node) => {
					*self.storage.borrow_stored(&s_handle) = Stored::New(node);
					MaybeChanged::Same(s_handle)
				}
				Action::Delete => {
					self.storage.destroy(s_handle);
					MaybeChanged::Deleted
				}
			}
			Stored::Cached(node, hash) => match inspector(self, node) {
				Action::Replace(node) => {
					self.death_row.push(hash);
					*self.storage.borrow_stored(&s_handle) = Stored::New(node);
					MaybeChanged::Changed(s_handle)
				}
				Action::Restore(node) => {
					*self.storage.borrow_stored(&s_handle) = Stored::Cached(node, hash);
					MaybeChanged::Same(s_handle)
				}
				Action::Delete => {
					self.death_row.push(hash);
					self.storage.destroy(s_handle);
					MaybeChanged::Deleted
				}
			}
		}
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
			true => self.db.get(&r.as_val::<H256>()).expect("Not found!"),
			false => node
		}
	}

	/// insert a key, value pair into the trie, creating new nodes if necessary.
	fn insert_at(&mut self, handle: NodeHandle, partial: NibbleSlice, value: Bytes) -> MaybeChanged {
		trace!(target: "trie", "insert_at (old: {:?}, partial: {:?}, value: {:?})", handle, partial, value.pretty());
		self.inspect_node(handle, move |trie, node| match node {
			Node::Empty => {
				Action::Replace(Node::Leaf(partial.encoded(true), value))
			}
			Node::Branch(mut children, mut stored_value) => {
				trace!(target: "trie", "branch: ROUTE,INSERT");
				if partial.is_empty() {
					if stored_value == value {
						Action::Restore(Node::Branch(children, stored_value))
					} else {
						Action::Replace(Node::Branch(children, value))
					}
				} else {
					let idx = partial.at(0) as usize;
					if let Some(child_handle) = children[idx].take() {
						// original had something there. continue insertion.
						match trie.insert_at(child_handle, partial.mid(1), value) {
							MaybeChanged::Same(handle) => {
								children[idx] = Some(handle.into());
								Action::Restore(Node::Branch(children, stored_value))
							}
							MaybeChanged::Changed(handle) => {
								children[idx] = Some(handle.into());
								Action::Replace(Node::Branch(children, stored_value))
							}
							MaybeChanged::Deleted => panic!("something went wrong: insertion deleted a node?"),
						}
					} else {
						// original had an empty slot, insert a leaf.
						let leaf = Node::Leaf(partial.mid(1).encode(true), value);
						children[idx] = Some(trie.storage.alloc(Stored::New(leaf)).into());
						Action::Replace(Node::Branch(children, stored_value))
					}
				}
			}
			Node::Leaf(encoded, existing_val) => {
				let existing_key = NibbleSlice::from_encoded(&encoded).0;
				let cp = partial.common_prefix(&existing_key);
				if cp == partial.len() && cp == existing_key.len() {
					trace!(target: "trie", "equivalent-leaf: REPLACE");
					// equivalent leaf.
					if existing_val == value {
						Action::Restore(Node::Leaf(encoded.clone(), existing_val))
					} else {
						Action::Replace(Node::Leaf(encoded.clone(), value))
					}
				} else if cp == 0 {
					// no common prefix. make a branch here.
					let mut children = empty_children();
					let branch = if existing_key.is_empty() {
						Node::Branch(children, Some(stored_value))
					} else {
						let index = existing_key.at(0) as usize;
						let leaf = Node::Leaf(existing_key.mid(1).encoded(true), stored_value);
						children[index] = Some(trie.storage.alloc(Stored::New(leaf)).into());
						Node::Branch(children, None)
					};

					// replace this node with the new branch, and then walk it further.
					let handle = trie.storage.alloc(Stored::New(branch)).into();
					match trie.insert_at(handle, partial, value) {
						// we know it isn't the same node as the leaf we just tried to put in.
						MaybeChanged::Same(h) | MaybeChanged::Changed(h) => Action::Replace(trie.storage.destroy(h)),
						MaybeChanged::Deleted => panic!("something went wrong: insertion deleted a node?"),
					}
				} else if cp == existing_key.len() {
					// fully-shared prefix.
					let downstream = {
						let branch = Node::Branch(empty_children(), existing_val);
						let handle = trie.storage.alloc(Stored::new(branch));
						match trie.insert_at(handle, partial.mid(cp), value) {
							MaybeChanged::Same(h) | MaybeChange::Changed(h) => trie.storage.destroy(h),
							MaybeChanged::Deleted => panic!("something went wrong: insertion deleted a node?"),
						}
					};

					let ext = Node::Extension(existing_key.encoded(false), trie.storage.alloc(Stored::new(downstream)));
					Action::Replace(ext)
				} else {
					// partially shared prefix. make a leaf
					let low = Node::Leaf(existing_key.mid(cp).encoded(true), existing_val);
					let low = trie.storage.alloc(Stored::new(low)).into();
					let augmented = match trie.insert_at(low, partial.mid(cp), value) {
						MaybeChanged::Same(h) | MaybeChanged::Changed(h) => h,
						MaybeChanged::Deleted => panic!("something went wrong: insertion deleted a node?"),
					};

					let ext = Node::Extension(existing_key.encoded_leftmost(cp, false), augmented.into());
					Action::Replace(ext)
				}
			}
			Node::Extension(encoded, child_branch) => {
				let existing_key = NibbleSlice::from_encoded(&encoded).0;
				let cp = partial.common_prefix(&existing_key);
				if cp == 0 {
					// make a branch.
					trace!(target: "trie", "no-common-prefix, not-both-empty (exist={:?}; new={:?}): TRANSMUTE,INSERT", existing_key.len(), partial.len());
					assert!(!existing_key.is_empty()); // extension nodes may not have empty partial keys.
					let mut children = empty_children();
					let index = existing_key.at(0) as usize;
					if existing_key.len() == 1 {
						// direct extension
						children[index] = Some(child_branch);
					} else {
						let extension = Node::Extension(existing_key.mid(1).encoded(false), child_branch);
						children[index] = Some(trie.storage.alloc(Stored::new(extension)).into());
					}

					// replace this node with a branch and walk it further.
					let handle = trie.storage.alloc(Stored::New(Node::Branch(children, None)));
					match trie.insert_at(handle.into(), partial, value) {
						MaybeChanged::Same(h) | MaybeChanged::Changed(h) => Action::Replace(trie.storage.destroy(h)),
						MaybeChanged::Deleted => panic!("something went wrong: insertion deleted a node?"),
					}
				} else if cp == existing_key.len() {
					// fully-shared prefix.
					trace!(target: "trie", "complete-prefix (cp={:?}): INSERT-AT-END", cp);
					let downstream = match trie.insert_at(child_branch, partial.mid(cp), value) {
						MaybeChanged::Same(h) | MaybeChanged::Changed(h) => h,
						MaybeChanged::Deleted => panic!("something went wrong: insertion deleted a node?"),
					};
					Node::Extension(existing_key.encoded(false), downstream.into())
				} else {
					// partially-shared prefix
					let low = trie.storage.alloc(Node::Extension(existing_key.mid(cp).encoded(false), child_branch));
					let augmented_low = match trie.insert_at(low.into(), partial.mid(cp), value) {
						MaybeChanged::Same(h) | MaybeChanged::Changed(h) => Action::Replace(trie.storage.destroy(h)),
						MaybeChanged::Deleted => panic!("something went wrong: insertion deleted a node?"),
					};
					trace!(target: "trie", "create_extension partial: {:?}, downstream_node: {:?}", existing_key, &self.storage[&augmented_low]);

					Node::Extension(existing_key.encoded_leftmost(cp, false), augmented_low.into())
				}
			}
		})
	}

	/// Remove a node from the trie based on key.
	fn remove_at(&mut self, handle: NodeHandle, partial: NibbleSlice) -> Option<StorageHandle> {
		unimplemented!()
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
						self.fix(Node::Extension(NibbleSlice::new_offset(&[i as u8; 1], 1).encoded(false), onward))
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
						let node_rlp = self.db.get(&hash).expect("Not found!");
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
		if !self.dirty { return }

		unimplemented!()
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
		match self.storage[&self.root_handle] {
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
		self.root_handle = self.insert_at(root_handle.into(), NibbleSlice::new(key), value.to_owned());
		self.dirty = true;
	}

	fn remove(&mut self, key: &[u8]) {
		let root_handle = self.root_handle();
		if self.remove_at(root_handle.into(), NibbleSlice::new(key)).is_none() {
			self.root_handle = self.storage.alloc(Node::Empty);
		}
		self.dirty = true;
	}
}

impl<'a> Drop for MemoryTrieDB<'a> {
	fn drop(&mut self) {
		self.commit();
	}
}

#[cfg(test)]
mod tests {
	extern crate json_tests;
	use self::json_tests::{trie, execute_tests_from_directory};
	use triehash::trie_root;
	use hash::*;
	use hashdb::*;
	use memorydb::*;
	use super::*;
	use rlp::*;
	use bytes::ToPretty;
	use super::super::trietraits::*;
	use super::super::standardmap::*;

	fn populate_trie<'db>(db: &'db mut HashDB, root: &'db mut H256, v: &[(Vec<u8>, Vec<u8>)]) -> MemoryTrieDB<'db> {
		let mut t = MemoryTrieDB::new(db, root);
		for i in 0..v.len() {
			let key: &[u8]= &v[i].0;
			let val: &[u8] = &v[i].1;
			t.insert(&key, &val);
		}
		t
	}

	fn unpopulate_trie<'db>(t: &mut MemoryTrieDB<'db>, v: &[(Vec<u8>, Vec<u8>)]) {
		for i in v {
			let key: &[u8]= &i.0;
			t.remove(&key);
		}
	}

	#[test]
	fn playpen() {
		::log::init_log();

		let mut seed = H256::new();
		for test_i in 0..1 {
			if test_i % 50 == 0 {
				debug!("{:?} of 10000 stress tests done", test_i);
			}
			let x = StandardMap {
				alphabet: Alphabet::Custom(b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_".to_vec()),
				min_key: 5,
				journal_key: 0,
				value_mode: ValueMode::Index,
				count: 100,
			}.make_with(&mut seed);

			let real = trie_root(x.clone());
			let mut memdb = MemoryDB::new();
			let mut root = H256::new();
			let mut memtrie = populate_trie(&mut memdb, &mut root, &x);

			memtrie.commit();
			if *memtrie.root() != real {
				println!("TRIE MISMATCH");
				println!("");
				println!("{:?} vs {:?}", memtrie.root(), real);
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
			}
			assert_eq!(*memtrie.root(), real);
			unpopulate_trie(&mut memtrie, &x);
			memtrie.commit();
			if *memtrie.root() != SHA3_NULL_RLP {
				println!("- TRIE MISMATCH");
				println!("");
				println!("{:?} vs {:?}", memtrie.root(), real);
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
			}
			assert_eq!(*memtrie.root(), SHA3_NULL_RLP);
		}
	}

	#[test]
	fn init() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let t = MemoryTrieDB::new(&mut memdb, &mut root);
		assert_eq!(*t.root(), SHA3_NULL_RLP);
	}

	#[test]
	fn insert_on_empty() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![ (vec![0x01u8, 0x23], vec![0x01u8, 0x23]) ]));
	}

	#[test]
	fn remove_to_empty() {
		let big_value = b"00000000000000000000000000000000";

		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t1 = MemoryTrieDB::new(&mut memdb, &mut root);
		t1.insert(&[0x01, 0x23], &big_value.to_vec());
		t1.insert(&[0x01, 0x34], &big_value.to_vec());
		t1.commit();
		let mut memdb2 = MemoryDB::new();
		let mut root2 = H256::new();
		let mut t2 = MemoryTrieDB::new(&mut memdb2, &mut root2);
		t2.insert(&[0x01], &big_value.to_vec());
		t2.insert(&[0x01, 0x23], &big_value.to_vec());
		t2.insert(&[0x01, 0x34], &big_value.to_vec());
		t2.remove(&[0x01]);
		t2.commit();
	}

	#[test]
	fn insert_replace_root() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0x01u8, 0x23], &[0x23u8, 0x45]);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![ (vec![0x01u8, 0x23], vec![0x23u8, 0x45]) ]));
	}

	#[test]
	fn insert_make_branch_root() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0x11u8, 0x23], &[0x11u8, 0x23]);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x11u8, 0x23], vec![0x11u8, 0x23])
		]));
	}

	#[test]
	fn insert_into_branch_root() {
		::log::init_log();
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0xf1u8, 0x23], &[0xf1u8, 0x23]);
		t.insert(&[0x81u8, 0x23], &[0x81u8, 0x23]);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
			(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
		]));
	}

	#[test]
	fn insert_value_into_branch_root() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[], &[0x0]);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![
			(vec![], vec![0x0]),
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
		]));
	}

	#[test]
	fn insert_split_leaf() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0x01u8, 0x34], &[0x01u8, 0x34]);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x01u8, 0x34], vec![0x01u8, 0x34]),
		]));
	}

	#[test]
	fn insert_split_extenstion() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01, 0x23, 0x45], &[0x01]);
		t.insert(&[0x01, 0xf3, 0x45], &[0x02]);
		t.insert(&[0x01, 0xf3, 0xf5], &[0x03]);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![
			(vec![0x01, 0x23, 0x45], vec![0x01]),
			(vec![0x01, 0xf3, 0x45], vec![0x02]),
			(vec![0x01, 0xf3, 0xf5], vec![0x03]),
		]));
	}

	#[test]
	fn insert_big_value() {
		let big_value0 = b"00000000000000000000000000000000";
		let big_value1 = b"11111111111111111111111111111111";

		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], big_value0);
		t.insert(&[0x11u8, 0x23], big_value1);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![
			(vec![0x01u8, 0x23], big_value0.to_vec()),
			(vec![0x11u8, 0x23], big_value1.to_vec())
		]));
	}

	#[test]
	fn insert_duplicate_value() {
		let big_value = b"00000000000000000000000000000000";

		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], big_value);
		t.insert(&[0x11u8, 0x23], big_value);
		t.commit();
		assert_eq!(*t.root(), trie_root(vec![
			(vec![0x01u8, 0x23], big_value.to_vec()),
			(vec![0x11u8, 0x23], big_value.to_vec())
		]));
	}

	#[test]
	fn test_at_empty() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let t = MemoryTrieDB::new(&mut memdb, &mut root);
		assert_eq!(t.get(&[0x5]), None);
	}

	#[test]
	fn test_at_one() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		assert_eq!(t.get(&[0x1, 0x23]).unwrap(), &[0x1u8, 0x23]);
		t.commit();
		assert_eq!(t.get(&[0x1, 0x23]).unwrap(), &[0x1u8, 0x23]);
	}

	#[test]
	fn test_at_three() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0xf1u8, 0x23], &[0xf1u8, 0x23]);
		t.insert(&[0x81u8, 0x23], &[0x81u8, 0x23]);
		assert_eq!(t.get(&[0x01, 0x23]).unwrap(), &[0x01u8, 0x23]);
		assert_eq!(t.get(&[0xf1, 0x23]).unwrap(), &[0xf1u8, 0x23]);
		assert_eq!(t.get(&[0x81, 0x23]).unwrap(), &[0x81u8, 0x23]);
		assert_eq!(t.get(&[0x82, 0x23]), None);
		t.commit();
		assert_eq!(t.get(&[0x01, 0x23]).unwrap(), &[0x01u8, 0x23]);
		assert_eq!(t.get(&[0xf1, 0x23]).unwrap(), &[0xf1u8, 0x23]);
		assert_eq!(t.get(&[0x81, 0x23]).unwrap(), &[0x81u8, 0x23]);
		assert_eq!(t.get(&[0x82, 0x23]), None);
	}

	#[test]
	fn stress() {
		let mut seed = H256::new();
		for _ in 0..50 {
			let x = StandardMap {
				alphabet: Alphabet::Custom(b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_".to_vec()),
				min_key: 5,
				journal_key: 0,
				value_mode: ValueMode::Index,
				count: 4,
			}.make_with(&mut seed);

			let real = trie_root(x.clone());
			let mut memdb = MemoryDB::new();
			let mut root = H256::new();
			let mut memtrie = populate_trie(&mut memdb, &mut root, &x);
			let mut y = x.clone();
			y.sort_by(|ref a, ref b| a.0.cmp(&b.0));
			let mut memdb2 = MemoryDB::new();
			let mut root2 = H256::new();
			let mut memtrie_sorted = populate_trie(&mut memdb2, &mut root2, &y);
			memtrie.commit();
			memtrie_sorted.commit();
			if *memtrie.root() != real || *memtrie_sorted.root() != real {
				println!("TRIE MISMATCH");
				println!("");
				println!("ORIGINAL... {:?}", memtrie.root());
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
				println!("SORTED... {:?}", memtrie_sorted.root());
				for i in &y {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
			}
			assert_eq!(*memtrie.root(), real);
			assert_eq!(*memtrie_sorted.root(), real);
		}
	}

	#[test]
	fn test_trie_json() {
		println!("Json trie test: ");
		execute_tests_from_directory::<trie::TrieTest, _>("json-tests/json/trie/*.json", &mut | file, input, output | {
			println!("file: {}", file);

			let mut memdb = MemoryDB::new();
			let mut root = H256::new();
			let mut t = MemoryTrieDB::new(&mut memdb, &mut root);
			for operation in input.into_iter() {
				match operation {
					trie::Operation::Insert(key, value) => t.insert(&key, &value),
					trie::Operation::Remove(key) => t.remove(&key)
				}
			}
			t.commit();
			assert_eq!(*t.root(), H256::from_slice(&output));
		});
	}

	#[test]
	fn test_trie_existing() {
		let mut root = H256::new();
		let mut db = MemoryDB::new();
		{
			let mut t = MemoryTrieDB::new(&mut db, &mut root);
			t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
			t.commit();
		}

		{
		 	let _ = MemoryTrieDB::from_existing(&mut db, &mut root);
		}
	}
}