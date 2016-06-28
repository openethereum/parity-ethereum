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
use ::rlp::{Rlp, RlpStream, View, Stream};

use elastic_array::ElasticArray1024;

use std::collections::{HashSet, VecDeque};
use std::mem;
use std::ops::Index;

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
			NodeHandle::InMemory(storage.alloc(Stored::New(child)))
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

	// encode a node to RLP
	// TODO: parallelize
	fn to_rlp<F>(self, mut child_cb: F) -> ElasticArray1024<u8>
		where F: FnMut(NodeHandle, &mut RlpStream)
	{
		match self {
			Node::Empty => {
				let mut stream = RlpStream::new();
				stream.append_empty_data();
				stream.drain()
			}
			Node::Leaf(partial, value) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&partial);
				stream.append(&value);
				stream.drain()
			}
			Node::Extension(partial, child) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&partial);
				child_cb(child, &mut stream);
				stream.drain()
			}
			Node::Branch(mut children, value) => {
				let mut stream = RlpStream::new_list(17);
				for child in children.iter_mut().map(Option::take) {
					if let Some(handle) = child {
						child_cb(handle, &mut stream);
					} else {
						stream.append_empty_data();
					}
				}
				if let Some(value) = value {
					stream.append(&value);
				} else {
					stream.append_empty_data();
				}

				stream.drain()
			}
		}
	}
}

// post-inspect action.
enum Action {
	// Replace a node with a new one.
	Replace(Node),
	// Restore the original node. This trusts that the node is actually the original.
	Restore(Node),
	// if it is a new node, just clears the storage.
	Delete,
}

// post-insert action. Same as action without delete
enum InsertAction {
	// Replace a node with a new one.
	Replace(Node),
	// Restore the original node.
	Restore(Node),
}

impl InsertAction {
	fn as_action(self) -> Action {
		match self {
			InsertAction::Replace(n) => Action::Replace(n),
			InsertAction::Restore(n) => Action::Restore(n),
		}
	}

	// unwrap the node, disregarding replace or restore state.
	fn unwrap_node(self) -> Node {
		match self {
			InsertAction::Replace(n) | InsertAction::Restore(n) => n,
		}
	}
}

// What kind of node is stored here.
enum Stored {
	// A new node.
	New(Node),
	// A cached node, loaded from the DB.
	Cached(Node, H256),
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

	/// Remove a node from the storage, consuming the handle and returning the node.
	fn destroy(&mut self, handle: StorageHandle) -> Stored {
		let idx = handle.0;

		self.free_indices.push_back(idx);
		mem::replace(&mut self.nodes[idx], Stored::New(Node::Empty))
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
	death_row: HashSet<H256>,
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
			storage: storage,
			db: db,
			root: root,
			root_handle: root_handle,
			dirty: false,
			death_row: HashSet::new(),
			hash_count: 0,
		}
	}

	/// Create a new trie with the backing database `db` and `root.
	/// Returns an error if `root` does not exist.
	pub fn from_existing(db: &'a mut HashDB, root: &'a mut H256) -> Result<Self, TrieError> {
		let mut storage = NodeStorage::empty();
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
			death_row: HashSet::new(),
			hash_count: 0,
		})
	}

	// cache a node by hash
	fn cache(&mut self, hash: H256) -> StorageHandle {
		let node_rlp = self.db.get(&hash).expect("Not found!");
		let node = Node::from_rlp(node_rlp, &*self.db, &mut self.storage);
		self.storage.alloc(Stored::Cached(node, hash))
	}

	// inspect a node, choosing either to replace, restore, or delete it.
	// if restored or replaced, returns the new node along with a flag of whether it was changed.
	fn inspect<F>(&mut self, stored: Stored, inspector: F) -> Option<(Stored, bool)>
	where F: FnOnce(&mut Self, Node) -> Action {
		match stored {
			Stored::New(node) => match inspector(self, node) {
				Action::Restore(node) => Some((Stored::New(node), false)),
				Action::Replace(node) => Some((Stored::New(node), true)),
				Action::Delete => None,
			},
			Stored::Cached(node, hash) => match inspector(self, node) {
				Action::Restore(node) => Some((Stored::Cached(node, hash), false)),
				Action::Replace(node) => {
					self.death_row.insert(hash);
					Some((Stored::New(node), true))
				}
				Action::Delete => {
					self.death_row.insert(hash);
					None
				}
			},
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
	fn insert_at(&mut self, handle: NodeHandle, partial: NibbleSlice, value: Bytes) -> (StorageHandle, bool) {
		let h = match handle {
			NodeHandle::InMemory(h) => h,
			NodeHandle::Hash(h) => self.cache(h)
		};
		let stored = self.storage.destroy(h);
		let (new_stored, changed) = self.inspect(stored, move |trie, stored| {
			trie.insert_inspector(stored, partial, value).as_action()
		}).expect("Insertion never deletes.");

		(self.storage.alloc(new_stored), changed)
	}

	/// the insertion inspector.
	fn insert_inspector(&mut self, node: Node, partial: NibbleSlice, value: Bytes) -> InsertAction {
		trace!(target: "trie", "augmented (partial: {:?}, value: {:?})", partial, value.pretty());

		match node {
			Node::Empty => {
				trace!(target: "trie", "empty: COMPOSE");
				InsertAction::Replace(Node::Leaf(partial.encoded(true), value))
			}
			Node::Branch(mut children, stored_value) => {
				trace!(target: "trie", "branch: ROUTE,AUGMENT");

				if partial.is_empty() {
					let unchanged = stored_value.as_ref() == Some(&value);
					if unchanged {
						// no change required.
						InsertAction::Restore(Node::Branch(children, stored_value))
					} else {
						InsertAction::Replace(Node::Branch(children, Some(value)))
					}
				} else {
					let idx = partial.at(0) as usize;
					let partial = partial.mid(1);
					if let Some(child) = children[idx].take() {
						// original had something there. recurse down into it.
						let (new_child, changed) = self.insert_at(child, partial, value);
						children[idx] = Some(new_child.into());
						if !changed {
							// the new node we composed didn't change. that means our branch is untouched too.
							return InsertAction::Restore(Node::Branch(children, stored_value));
						}
					} else {
						// original had nothing there. compose a leaf.
						let leaf = self.storage.alloc(Stored::New(Node::Leaf(partial.encoded(true), value)));
						children[idx] = Some(leaf.into());
					}

					InsertAction::Replace(Node::Branch(children, stored_value))
				}
			}
			Node::Leaf(encoded, stored_value) => {
				let existing_key = NibbleSlice::from_encoded(&encoded).0;
				let cp = partial.common_prefix(&existing_key);
				if cp == existing_key.len() && cp == partial.len() {
					trace!(target: "trie", "equivalent-leaf: REPLACE");
					// equivalent leaf.
					if stored_value == value {
						// unchanged. restore
						InsertAction::Restore(Node::Leaf(encoded.clone(), value))
					} else {
						InsertAction::Replace(Node::Leaf(encoded.clone(), value))
					}
				} else if cp == 0 {
					trace!(target: "trie", "no-common-prefix, not-both-empty (exist={:?}; new={:?}): TRANSMUTE,AUGMENT", existing_key.len(), partial.len());

					// one of us isn't empty: transmute to branch here
					let mut children = empty_children();
					let branch = if existing_key.is_empty() {
						// always replace since branch isn't leaf.
						Node::Branch(children, Some(stored_value))
					} else {
						let idx = existing_key.at(0) as usize;
						let new_leaf = Node::Leaf(existing_key.mid(1).encoded(true), stored_value);
						children[idx] = Some(self.storage.alloc(Stored::New(new_leaf)).into());

						Node::Branch(children, None)
					};

					// always replace because whatever we get out here is not the branch we started with.
					InsertAction::Replace(self.insert_inspector(branch, partial, value).unwrap_node())
				} else if cp == existing_key.len() {
				    trace!(target: "trie", "complete-prefix (cp={:?}): AUGMENT-AT-END", cp);

					// fully-shared prefix for an extension.
					// make a stub branch and an extension.
					let branch = Node::Branch(empty_children(), Some(stored_value));
					// augment the new branch.
					let branch = self.insert_inspector(branch, partial.mid(cp), value).unwrap_node();

					// always replace since we took a leaf and made an extension.
					let branch_handle = self.storage.alloc(Stored::New(branch)).into();
					InsertAction::Replace(Node::Extension(existing_key.encoded(false), branch_handle))
				} else {
					trace!(target: "trie", "partially-shared-prefix (exist={:?}; new={:?}; cp={:?}): AUGMENT-AT-END", existing_key.len(), partial.len(), cp);

					// partially-shared prefix for an extension.
					// start by making a leaf.
					let low = Node::Leaf(existing_key.mid(cp).encoded(true), stored_value);
					// augment it. this will result in the Leaf -> cp == 0 routine,
					// which creates a branch.
					let augmented_low = self.insert_inspector(low, partial.mid(cp), value).unwrap_node();

					// make an extension using it. this is a replacement.
					InsertAction::Replace(Node::Extension(
						existing_key.encoded_leftmost(cp, false),
						self.storage.alloc(Stored::New(augmented_low)).into()
					))
				}
			}
			Node::Extension(encoded, child_branch) => {
				let existing_key = NibbleSlice::from_encoded(&encoded).0;
				let cp = partial.common_prefix(&existing_key);
				if cp == 0 {
					trace!(target: "trie", "no-common-prefix, not-both-empty (exist={:?}; new={:?}): TRANSMUTE,AUGMENT", existing_key.len(), partial.len());

					// partial isn't empty: make a branch here
					// extensions may not have empty partial keys.
					assert!(!existing_key.is_empty());
					let idx = existing_key.at(0) as usize;

					let mut children = empty_children();
					children[idx] = if existing_key.len() == 1 {
						// direct extension, just replace.
						Some(child_branch)
					} else {
						// more work required after branching.
						let ext = Node::Extension(existing_key.mid(1).encoded(false), child_branch);
						Some(self.storage.alloc(Stored::New(ext)).into())
					};

					// continue inserting.
					InsertAction::Replace(self.insert_inspector(Node::Branch(children, None), partial, value).unwrap_node())
				} else if cp == existing_key.len() {
					trace!(target: "trie", "complete-prefix (cp={:?}): AUGMENT-AT-END", cp);

					// fully-shared prefix.

					// insert into the child node.
					let (new_child, changed) = self.insert_at(child_branch, partial.mid(cp), value);
					let new_ext = Node::Extension(existing_key.encoded(false), new_child.into());
					if changed {
						InsertAction::Replace(new_ext)
					} else {
						// the child branch wasn't changed, meaning this extension remains the same.
						InsertAction::Restore(new_ext)
					}
				} else {
					trace!(target: "trie", "partially-shared-prefix (exist={:?}; new={:?}; cp={:?}): AUGMENT-AT-END", existing_key.len(), partial.len(), cp);

					// partially-shared.
					let low = Node::Extension(existing_key.mid(cp).encoded(false), child_branch);
					// augment the extension. this will take the cp == 0 path, creating a branch.
					let augmented_low = self.insert_inspector(low, partial.mid(cp), value).unwrap_node();

					// always replace, since this extension is not the one we started with.
					// this is known because the partial key is only the common prefix.
					InsertAction::Replace(Node::Extension(
						existing_key.encoded_leftmost(cp, false),
						self.storage.alloc(Stored::New(augmented_low)).into()
					))
				}
			}
		}
	}

	/// Remove a node from the trie based on key.
	fn remove_at(&mut self, handle: NodeHandle, partial: NibbleSlice) -> Option<(StorageHandle, bool)> {
		let stored = match handle {
			NodeHandle::InMemory(h) => self.storage.destroy(h),
			NodeHandle::Hash(h) => {
				let handle = self.cache(h);
				self.storage.destroy(handle)
			}
		};

		self.inspect(stored, move |trie, node| trie.remove_inspector(node, partial))
			.map(|(new, changed)| (self.storage.alloc(new), changed))
	}

	/// the removal inspector
	fn remove_inspector(&mut self, node: Node, partial: NibbleSlice) -> Action {
		match (node, partial.is_empty()) {
			(Node::Empty, _) => {
				trace!(target: "trie", "Deleting empty, partial={:?}", partial);
				Action::Delete
			}
			(Node::Branch(c, None), true) => {
				trace!(target: "trie", "Restoring branch, partial={:?}", partial);
				Action::Restore(Node::Branch(c, None))
			}
			(Node::Branch(children, _), true) => {
				trace!(target: "trie", "removing value out of branch, partial={:?}", partial);
				// always replace since we took the value out.
				Action::Replace(self.fix(Node::Branch(children, None)))
			}
			(Node::Branch(mut children, value), false) => {
				let idx = partial.at(0) as usize;
				if let Some(child) = children[idx].take() {
					trace!(target: "trie", "removing value out of branch child, partial={:?}", partial);
					match self.remove_at(child, partial.mid(1)) {
						Some((new, changed)) => {
							children[idx] = Some(new.into());
							let branch = Node::Branch(children, value);
							if changed {
								// child was changed, so we were too.
								trace!(target: "trie", "replacing branch child, partial={:?}", partial);
								Action::Replace(branch)
							} else {
								// unchanged, so we are too.
								trace!(target: "trie", "restoring branch child, partial={:?}", partial);
								Action::Restore(branch)
							}
						}
						None => {
							// the child we took was deleted.
							// the node may need fixing.
							trace!(target: "trie", "branch child deleted, partial={:?}", partial);
							Action::Replace(self.fix(Node::Branch(children, value)))
						}
					}
				} else {
					// no change needed.
					trace!(target: "trie", "Restoring branch, partial={:?}", partial);
					Action::Restore(Node::Branch(children, value))
				}
			}
			(Node::Leaf(encoded, value), _) => {
				if NibbleSlice::from_encoded(&encoded).0 == partial {
					// this is the node we were looking for. Let's delete it.
					trace!(target: "trie", "deleting leaf, partial={:?}", partial);
					Action::Delete
				} else {
					// leaf the node alone.
					trace!(target: "trie", "restoring leaf wrong partial, partial={:?}, existing={:?}", partial, NibbleSlice::from_encoded(&encoded).0);
					Action::Restore(Node::Leaf(encoded, value))
				}
			}
			(Node::Extension(encoded, child_branch), _) => {
				let (cp, existing_len) = {
					let existing_key = NibbleSlice::from_encoded(&encoded).0;
					(existing_key.common_prefix(&partial), existing_key.len())
				};
				if cp == existing_len {
					// try to remove from the child branch.
					trace!(target: "trie", "removing from extension child, partial={:?}", partial);
					match self.remove_at(child_branch, partial.mid(cp)) {
						Some((new_child, changed)) => {
							let new_child = new_child.into();
							if !changed {
								// the branch we put in was unchanged.
								// this means that the extension doesn't need changing either.
								trace!(target: "trie", "restoring extension, partial={:?}", partial);
								Action::Restore(Node::Extension(encoded, new_child))
							} else {
								trace!(target: "trie", "replacing extension child, partial={:?}", partial);
								// the new node may not be a branch.
								// always replace since the child was changed somehow.
								Action::Replace(self.fix(Node::Extension(encoded, new_child)))
							}
						}
						None => {
							// the whole branch got deleted.
							// that means that this extension is useless.
							trace!(target: "trie", "propagating branch delete to extension, partial={:?}", partial);
							Action::Delete
						}
					}
				} else {
					// partway through an extension -- nothing to do here.
					trace!(target: "trie", "restoring extension wrong partial, partial={:?}, existing={:?}", partial, NibbleSlice::from_encoded(&encoded).0);
					Action::Restore(Node::Extension(encoded, child_branch))
				}
			}
		}
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
				// if only a single value, transmute to leaf/extension and feed through fixed.
				#[derive(Debug)]
				enum UsedIndex {
					None,
					One(u8),
					Many,
				};
				let mut used_index = UsedIndex::None;
				for i in 0..16 {
					match (children[i].is_none(), &used_index) {
						(false, &UsedIndex::None) => used_index = UsedIndex::One(i as u8),
						(false, &UsedIndex::One(_)) => {
							used_index = UsedIndex::Many;
							break;
						}
						_ => continue,
					}
				}

				match (used_index, value) {
					(UsedIndex::None, None) => panic!("Branch with no subvalues. Something went wrong."),
					(UsedIndex::One(a), None) => {
						// only one onward node. make an extension.
						let new_partial = NibbleSlice::new_offset(&[a], 1).encoded(false);
						let new_node = Node::Extension(new_partial, children[a as usize].take().unwrap());
						self.fix(new_node)
					}
					(UsedIndex::None, Some(value)) => {
						// make a leaf.
						trace!(target: "trie", "fixing: branch -> leaf");
						Node::Leaf(NibbleSlice::new(&[]).encoded(true), value)
					}
					(_, value) => {
						// all is well.
						trace!(target: "trie", "fixing: restoring branch");
						Node::Branch(children, value)
					}
				}
			}
			Node::Extension(partial, child) => {
				let stored = match child {
					NodeHandle::InMemory(h) => self.storage.destroy(h),
					NodeHandle::Hash(h) => {
						let handle = self.cache(h);
						self.storage.destroy(handle)
					}
				};

				let (child_node, maybe_hash) = match stored {
					Stored::New(node) => (node, None),
					Stored::Cached(node, hash) => (node, Some(hash))
				};

				match child_node {
					Node::Extension(sub_partial, sub_child) => {
						// combine with node below.
						if let Some(hash) = maybe_hash {
							// delete the cached child since we are going to replace it.
							self.death_row.insert(hash);
						}
						let partial = NibbleSlice::from_encoded(&partial).0;
						let sub_partial = NibbleSlice::from_encoded(&sub_partial).0;

						let new_partial = NibbleSlice::new_composed(&partial, &sub_partial);
						trace!(target: "trie", "fixing: extension combination. new_partial={:?}", new_partial);
						self.fix(Node::Extension(new_partial.encoded(false), sub_child))
					}
					Node::Leaf(sub_partial, value) => {
						// combine with node below.
						if let Some(hash) = maybe_hash {
							// delete the cached child since we are going to replace it.
							self.death_row.insert(hash);
						}
						let partial = NibbleSlice::from_encoded(&partial).0;
						let sub_partial = NibbleSlice::from_encoded(&sub_partial).0;

						let new_partial = NibbleSlice::new_composed(&partial, &sub_partial);
						trace!(target: "trie", "fixing: extension -> leaf. new_partial={:?}", new_partial);
						Node::Leaf(new_partial.encoded(true), value)
					}
					child_node => {
						trace!(target: "trie", "fixing: restoring extension");

						// reallocate the child node.
						let stored = if let Some(hash) = maybe_hash {
							Stored::Cached(child_node, hash)
						} else {
							Stored::New(child_node)
						};

						Node::Extension(partial, self.storage.alloc(stored).into())
					}
				}
			}
			other => other, // only ext and branch need fixing.
		}
	}

	/// Commit the in-memory changes to disk, freeing their storage and
	/// updating the state root.
	pub fn commit(&mut self) {
		if !self.dirty { return }
		trace!(target: "trie", "Committing trie changes to db.");

		// kill all the nodes on death row.
		trace!(target: "trie", "{:?} nodes to remove from db", self.death_row.len());
		for hash in self.death_row.drain() {
			self.db.remove(&hash);
		}

		let root_handle = self.root_handle();
		match self.storage.destroy(root_handle) {
			Stored::New(node) => {
				let root_rlp = node.to_rlp(|child, stream| self.commit_node(child, stream));
				*self.root = self.db.insert(&root_rlp[..]);
				self.hash_count += 1;

				trace!(target: "trie", "root node rlp: {:?}", (&root_rlp[..]).pretty());

				// restore the root node.
				let root_node = Node::from_rlp(&root_rlp, &*self.db, &mut self.storage);
				self.root_handle = self.storage.alloc(Stored::Cached(root_node, *self.root));
			}
			Stored::Cached(node, hash) => {
				// probably won't happen, but update the root and move on.
				*self.root = hash;
				self.root_handle = self.storage.alloc(Stored::Cached(node, hash))
			}
		}

		self.dirty = false;
	}

	/// commit a node, hashing it, committing it to the db,
	/// and writing it to the rlp stream as necessary.
	fn commit_node(&mut self, handle: NodeHandle, stream: &mut RlpStream) {
		match handle {
			NodeHandle::Hash(h) => stream.append(&h),
			NodeHandle::InMemory(h) => match self.storage.destroy(h) {
				Stored::Cached(_, h) => stream.append(&h),
				Stored::New(node) => {
					let node_rlp = node.to_rlp(|child, stream| self.commit_node(child, stream));
					if node_rlp.len() >= 32 {
						let hash = self.db.insert(&node_rlp[..]);
						self.hash_count += 1;
						stream.append(&hash)
					} else {
						stream.append_raw(&node_rlp, 1)
					}
				}
			}
		};
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
		let (new_handle, changed) = self.insert_at(root_handle.into(), NibbleSlice::new(key), value.to_owned());
		self.root_handle = new_handle;
		self.dirty = self.dirty || changed;
	}

	fn remove(&mut self, key: &[u8]) {
		let root_handle = self.root_handle();
		let key = NibbleSlice::new(key);
		trace!(target: "trie", "Beginning remove, key={:?}", key);
		match self.remove_at(root_handle.into(), key) {
			Some((handle, changed)) => {
				self.root_handle = handle;
				self.dirty = self.dirty || changed;
			}
			None => {
				self.root_handle = self.storage.alloc(Stored::New(Node::Empty));
				self.dirty = true;
			}
		};
		trace!(target: "trie", "/");
	}
}

impl<'a> Drop for MemoryTrieDB<'a> {
	fn drop(&mut self) {
		trace!(target: "trie", "dropping trie");
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
		for test_i in 0..10 {
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