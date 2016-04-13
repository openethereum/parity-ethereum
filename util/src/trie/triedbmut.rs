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

use common::*;
use hashdb::*;
use nibbleslice::*;
use rlp::*;
use super::node::*;
use super::journal::*;
use super::trietraits::*;

/// A `Trie` implementation using a generic `HashDB` backing database.
///
/// Use it as a `Trie` trait object. You can use `db()` to get the backing database object, `keys`
/// to get the keys belonging to the trie in the backing database, and `db_items_remaining()` to get
/// which items in the backing database do not belong to this trie. If this is the only trie in the
/// backing database, then `db_items_remaining()` should be empty.
///
/// # Example
/// ```
/// extern crate ethcore_util as util;
/// use util::trie::*;
/// use util::hashdb::*;
/// use util::memorydb::*;
/// use util::hash::*;
/// use util::rlp::*;
///
/// fn main() {
///   let mut memdb = MemoryDB::new();
///   let mut root = H256::new();
///   let mut t = TrieDBMut::new(&mut memdb, &mut root);
///   assert!(t.is_empty());
///   assert_eq!(*t.root(), SHA3_NULL_RLP);
///   t.insert(b"foo", b"bar");
///   assert!(t.contains(b"foo"));
///   assert_eq!(t.get(b"foo").unwrap(), b"bar");
///   assert!(t.db_items_remaining().is_empty());
///   t.remove(b"foo");
///   assert!(!t.contains(b"foo"));
///   assert!(t.db_items_remaining().is_empty());
/// }
/// ```
pub struct TrieDBMut<'db> {
	db: &'db mut HashDB,
	root: &'db mut H256,
	/// The number of hashes performed so far in operations on this trie.
	pub hash_count: usize,
}

/// Option-like type allowing either a Node object passthrough or Bytes in the case of data alteration.
enum MaybeChanged<'a> {
	Same(Node<'a>),
	Changed(Bytes),
}

#[cfg_attr(feature="dev", allow(wrong_self_convention))]
impl<'db> TrieDBMut<'db> {
	/// Create a new trie with the backing database `db` and empty `root`
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	pub fn new(db: &'db mut HashDB, root: &'db mut H256) -> Self {
		let mut r = TrieDBMut {
			db: db,
			root: root,
			hash_count: 0,
		};

		// set root rlp
		*r.root = SHA3_NULL_RLP.clone();
		r
	}

	/// Create a new trie with the backing database `db` and `root`.
	/// Panics, if `root` does not exist.
	// TODO: return Result<Self, TrieError>
	pub fn from_existing(db: &'db mut HashDB, root: &'db mut H256) -> Self {
		if !db.exists(root) {
			flushln!("Trie root not found {}", root);
			panic!("Trie root not found!");
		}
		TrieDBMut {
			db: db,
			root: root,
			hash_count: 0,
		}
	}

	/// Get the backing database.
	pub fn db(&'db self) -> &'db HashDB {
		self.db
	}

	/// Get the backing database.
	pub fn db_mut(&'db mut self) -> &'db mut HashDB {
		self.db
	}

	/// Determine all the keys in the backing database that belong to the trie.
	pub fn keys(&self) -> Vec<H256> {
		let mut ret: Vec<H256> = Vec::new();
		ret.push(self.root.clone());
		self.accumulate_keys(self.root_node(), &mut ret);
		ret
	}

	/// Convert a vector of hashes to a hashmap of hash to occurances.
	pub fn to_map(hashes: Vec<H256>) -> HashMap<H256, u32> {
		let mut r: HashMap<H256, u32> = HashMap::new();
		for h in hashes.into_iter() {
			let c = *r.get(&h).unwrap_or(&0);
			r.insert(h, c + 1);
		}
		r
	}

	/// Determine occurances of items in the backing database which are not related to this
	/// trie.
	pub fn db_items_remaining(&self) -> HashMap<H256, i32> {
		let mut ret = self.db.keys();
		for (k, v) in Self::to_map(self.keys()).into_iter() {
			let keycount = *ret.get(&k).unwrap_or(&0);
			match keycount <= v as i32 {
				true => ret.remove(&k),
				_ => ret.insert(k, keycount - v as i32),
			};
		}
		ret
	}

	/// Set the trie to a new root node's RLP, inserting the new RLP into the backing database
	/// and removing the old.
	fn set_root_rlp(&mut self, root_data: &[u8]) {
		self.db.kill(&self.root);
		*self.root = self.db.insert(root_data);
		self.hash_count += 1;
		trace!("set_root_rlp {:?} {:?}", root_data.pretty(), self.root);
	}

	/// Apply the items in `journal` into the backing database.
	fn apply(&mut self, journal: Journal) {
		self.hash_count += journal.apply(self.db).inserts;
	}

	/// Recursion helper for `keys`.
	fn accumulate_keys(&self, node: Node, acc: &mut Vec<H256>) {
		let mut handle_payload = |payload| {
			let p = Rlp::new(payload);
			if p.is_data() && p.size() == 32 {
				acc.push(p.as_val());
			}

			self.accumulate_keys(self.get_node(payload), acc);
		};

		match node {
			Node::Extension(_, payload) => handle_payload(payload),
			Node::Branch(payloads, _) => for payload in &payloads {
				handle_payload(payload)
			},
			_ => {}
		}
	}

	/// Get the root node's RLP.
	fn root_node(&self) -> Node {
		Node::decoded(self.db.lookup(&self.root).expect("Trie root not found!"))
	}

	/// Get the root node as a `Node`.
	fn get_node<'a>(&'a self, node: &'a [u8]) -> Node {
		Node::decoded(self.get_raw_or_lookup(node))
	}

	/// Indentation helper for `formal_all`.
	fn fmt_indent(&self, f: &mut fmt::Formatter, size: usize) -> fmt::Result {
		for _ in 0..size {
			try!(write!(f, "  "));
		}
		Ok(())
	}

	/// Recursion helper for implementation of formatting trait.
	fn fmt_all(&self, node: Node, f: &mut fmt::Formatter, deepness: usize) -> fmt::Result {
		match node {
			Node::Leaf(slice, value) => try!(writeln!(f, "'{:?}: {:?}.", slice, value.pretty())),
			Node::Extension(ref slice, ref item) => {
				try!(write!(f, "'{:?} ", slice));
				try!(self.fmt_all(self.get_node(item), f, deepness));
			}
			Node::Branch(ref nodes, ref value) => {
				try!(writeln!(f, ""));
				if let Some(v) = *value {
					try!(self.fmt_indent(f, deepness + 1));
					try!(writeln!(f, "=: {:?}", v.pretty()))
				}
				for i in 0..16 {
					match self.get_node(nodes[i]) {
						Node::Empty => {}
						n => {
							try!(self.fmt_indent(f, deepness + 1));
							try!(write!(f, "'{:x} ", i));
							try!(self.fmt_all(n, f, deepness + 1));
						}
					}
				}
			}
			// empty
			Node::Empty => {
				try!(writeln!(f, "<empty>"));
			}
		};
		Ok(())
	}

	/// Return optional data for a key given as a `NibbleSlice`. Returns `None` if no data exists.
	fn do_lookup<'a, 'key>(&'a self, key: &NibbleSlice<'key>) -> Option<&'a [u8]>
		where 'a: 'key,
	{
		let root_rlp = self.db.lookup(&self.root).expect("Trie root not found!");
		self.get_from_node(&root_rlp, key)
	}

	/// Recursible function to retrieve the value given a `node` and a partial `key`. `None` if no
	/// value exists for the key.
	///
	/// Note: Not a public API; use Trie trait functions.
	fn get_from_node<'a, 'key>(&'a self, node: &'a [u8], key: &NibbleSlice<'key>) -> Option<&'a [u8]>
		where 'a: 'key,
	{
		match Node::decoded(node) {
			Node::Leaf(ref slice, ref value) if key == slice => Some(value),
			Node::Extension(ref slice, ref item) if key.starts_with(slice) => {
				self.get_from_node(self.get_raw_or_lookup(item), &key.mid(slice.len()))
			}
			Node::Branch(ref nodes, value) => match key.is_empty() {
				true => value,
				false => self.get_from_node(self.get_raw_or_lookup(nodes[key.at(0) as usize]), &key.mid(1)),
			},
			_ => None,
		}
	}

	/// Given some node-describing data `node`, return the actual node RLP.
	/// This could be a simple identity operation in the case that the node is sufficiently small, but
	/// may require a database lookup.
	fn get_raw_or_lookup<'a>(&'a self, node: &'a [u8]) -> &'a [u8] {
		// check if its sha3 + len
		let r = Rlp::new(node);
		match r.is_data() && r.size() == 32 {
			true => self.db.lookup(&r.as_val::<H256>()).expect("Not found!"),
			false => node,
		}
	}

	/// Insert a `key` and `value` pair into the trie.
	///
	/// Note: Not a public API; use Trie trait functions.
	fn insert_ns(&mut self, key: &NibbleSlice, value: &[u8]) {
		trace!("ADD: {:?} {:?}", key, value.pretty());
		// determine what the new root is, insert new nodes and remove old as necessary.
		let mut todo: Journal = Journal::new();
		let root_rlp = self.augmented(self.db.lookup(&self.root).expect("Trie root not found!"), key, value, &mut todo);
		self.apply(todo);
		self.set_root_rlp(&root_rlp);
		trace!("/");
	}

	/// Remove a `key` and `value` pair from the trie.
	///
	/// Note: Not a public API; use Trie trait functions.
	fn remove_ns(&mut self, key: &NibbleSlice) {
		trace!("DELETE: {:?}", key);
		// determine what the new root is, insert new nodes and remove old as necessary.
		let mut todo: Journal = Journal::new();
		match self.cleared_from_slice(self.db.lookup(&self.root).expect("Trie root not found!"), key, &mut todo) {
			Some(root_rlp) => {
				self.apply(todo);
				self.set_root_rlp(&root_rlp);
			}
			None => {
				trace!("no change needed");
			}
		}
		trace!("/");
	}

	/// Compose a leaf node in RLP given the `partial` key and `value`.
	fn compose_leaf(partial: &NibbleSlice, value: &[u8]) -> Bytes {
		trace!("compose_leaf {:?} {:?} ({:?})", partial, value.pretty(), partial.encoded(true).pretty());
		let mut s = RlpStream::new_list(2);
		s.append(&partial.encoded(true));
		s.append(&value);
		let r = s.out();
		trace!("compose_leaf: -> {:?}", r.pretty());
		r
	}

	/// Compose a raw extension/leaf node in RLP given the `partial` key, `raw_payload` and whether it `is_leaf`.
	fn compose_raw(partial: &NibbleSlice, raw_payload: &[u8], is_leaf: bool) -> Bytes {
		trace!("compose_raw {:?} {:?} {:?} ({:?})", partial, raw_payload.pretty(), is_leaf, partial.encoded(is_leaf));
		let mut s = RlpStream::new_list(2);
		s.append(&partial.encoded(is_leaf));
		s.append_raw(raw_payload, 1);
		let r = s.out();
		trace!("compose_raw: -> {:?}", r.pretty());
		r
	}

	/// Compose a branch node in RLP with a particular `value` sitting in the value position (17th place).
	fn compose_stub_branch(value: &[u8]) -> Bytes {
		let mut s = RlpStream::new_list(17);
		for _ in 0..16 {
			s.append_empty_data();
		}
		s.append(&value);
		s.out()
	}

	/// Compose an extension node's RLP with the `partial` key and `raw_payload`.
	fn compose_extension(partial: &NibbleSlice, raw_payload: &[u8]) -> Bytes {
		Self::compose_raw(partial, raw_payload, false)
	}

	/// Return the bytes encoding the node represented by `rlp`. `journal` will record necessary
	/// removal instructions from the backing database.
	fn take_node<'a, 'rlp_view>(&'a self, rlp: &'rlp_view Rlp<'a>, journal: &mut Journal) -> &'a [u8]
		where 'a: 'rlp_view,
	{
		if rlp.is_list() {
			trace!("take_node {:?} (inline)", rlp.as_raw().pretty());
			rlp.as_raw()
		} else if rlp.is_data() && rlp.size() == 32 {
			let h = rlp.as_val();
			let r = self.db.lookup(&h).unwrap_or_else(|| {
				println!("Node not found! rlp={:?}, node_hash={:?}", rlp.as_raw().pretty(), h);
				println!("Journal: {:?}", journal);
				panic!();
			});
			trace!("take_node {:?} (indirect for {:?})", rlp.as_raw().pretty(), r);
			journal.delete_node_sha3(h);
			r
		} else {
			trace!("take_node {:?} (???)", rlp.as_raw().pretty());
			panic!("Empty or invalid node given?");
		}
	}

	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	/// Determine the RLP of the node, assuming we're inserting `partial` into the
	/// node currently of data `old`. This will *not* delete any hash of `old` from the database;
	/// it will just return the new RLP that includes the new node.
	///
	/// `journal` will record the database updates so as to make the returned RLP valid through inserting
	/// and deleting nodes as necessary.
	///
	/// **This operation will not insert the new node nor destroy the original.**
	fn augmented(&self, old: &[u8], partial: &NibbleSlice, value: &[u8], journal: &mut Journal) -> Bytes {
		trace!("augmented (old: {:?}, partial: {:?}, value: {:?})", old.pretty(), partial, value.pretty());
		// already have an extension. either fast_forward, cleve or transmute_to_branch.
		let old_rlp = Rlp::new(old);
		match old_rlp.prototype() {
			Prototype::List(17) => {
				trace!("branch: ROUTE,AUGMENT");
				// already have a branch. route and augment.
				let mut s = RlpStream::new_list(17);
				let index = if partial.is_empty() { 16 } else { partial.at(0) as usize };
				for i in 0..17 {
					match index == i {
						// not us - leave alone.
						false => {
							s.append_raw(old_rlp.at(i).as_raw(), 1);
						}
						// branch-leaf entry - just replace.
						true if i == 16 => {
							s.append(&value);
						}
						// original had empty slot - place a leaf there.
						true if old_rlp.at(i).is_empty() => journal.new_node(Self::compose_leaf(&partial.mid(1), value), &mut s),
						// original has something there already; augment.
						true => {
							let new = self.augmented(self.take_node(&old_rlp.at(i), journal), &partial.mid(1), value, journal);
							journal.new_node(new, &mut s);
						}
					}
				}
				s.out()
			}
			Prototype::List(2) => {
				let existing_key_rlp = old_rlp.at(0);
				let (existing_key, is_leaf) = NibbleSlice::from_encoded(existing_key_rlp.data());
				match (is_leaf, partial.common_prefix(&existing_key)) {
					(true, cp) if cp == existing_key.len() && partial.len() == existing_key.len() => {
						// equivalent-leaf: replace
						trace!("equivalent-leaf: REPLACE");
						Self::compose_leaf(partial, value)
					}
					(_, 0) => {
						// one of us isn't empty: transmute to branch here
						trace!("no-common-prefix, not-both-empty (exist={:?}; new={:?}): TRANSMUTE,AUGMENT", existing_key.len(), partial.len());
						assert!(is_leaf || !existing_key.is_empty());	// extension nodes are not allowed to have empty partial keys.
						let mut s = RlpStream::new_list(17);
						let index = if existing_key.is_empty() { 16 } else { existing_key.at(0) };
						for i in 0..17 {
							match is_leaf {
								// not us - empty.
								_ if index != i => {
									s.append_empty_data();
								}
								// branch-value: just replace.
								true if i == 16 => {
									s.append_raw(old_rlp.at(1).as_raw(), 1);
								}
								// direct extension: just replace.
								false if existing_key.len() == 1 => {
									s.append_raw(old_rlp.at(1).as_raw(), 1);
								}
								// original has empty slot.
								true => journal.new_node(Self::compose_leaf(&existing_key.mid(1), old_rlp.at(1).data()), &mut s),
								// additional work required after branching.
								false => journal.new_node(Self::compose_extension(&existing_key.mid(1), old_rlp.at(1).as_raw()), &mut s),
							}
						}
						self.augmented(&s.out(), partial, value, journal)
					}
					(_, cp) if cp == existing_key.len() => {
						trace!("complete-prefix (cp={:?}): AUGMENT-AT-END", cp);
						// fully-shared prefix for this extension:
						// transform to an extension + augmented version of onward node.
						let downstream_node: Bytes = match is_leaf {
							// no onward node because we're a leaf - create fake stub and use that.
							true => self.augmented(&Self::compose_stub_branch(old_rlp.at(1).data()), &partial.mid(cp), value, journal),
							false => self.augmented(self.take_node(&old_rlp.at(1), journal), &partial.mid(cp), value, journal),
						};

						trace!("create_extension partial: {:?}, downstream_node: {:?}", existing_key, downstream_node.pretty());
						let mut s = RlpStream::new_list(2);
						s.append(&existing_key.encoded(false));
						journal.new_node(downstream_node, &mut s);
						s.out()
					}
					(_, cp) => {
						// partially-shared prefix for this extension:
						// split into two extensions, high and low, pass the
						// low through augment with the value before inserting the result
						// into high to create the new.

						// TODO: optimise by doing this without creating augmented_low.

						trace!("partially-shared-prefix (exist={:?}; new={:?}; cp={:?}): AUGMENT-AT-END", existing_key.len(), partial.len(), cp);

						// low (farther from root)
						let low = Self::compose_raw(&existing_key.mid(cp), old_rlp.at(1).as_raw(), is_leaf);
						let augmented_low = self.augmented(&low, &partial.mid(cp), value, journal);

						// high (closer to root)
						let mut s = RlpStream::new_list(2);
						s.append(&existing_key.encoded_leftmost(cp, false));
						journal.new_node(augmented_low, &mut s);
						s.out()
					}
				}
			}
			Prototype::Data(0) => {
				trace!("empty: COMPOSE");
				Self::compose_leaf(partial, value)
			}
			_ => panic!("Invalid RLP for node: {:?}", old.pretty()),
		}
	}

	/// Given a `MaybeChanged` result `n`, return the node's RLP regardless of whether it changed.
	fn encoded(n: MaybeChanged) -> Bytes {
		match n {
			MaybeChanged::Same(n) => n.encoded(),
			MaybeChanged::Changed(b) => b,
		}
	}

	/// Fix the node payload's sizes in `n`, replacing any over-size payloads with the hashed reference
	/// and placing the payload DB insertions in the `journal`.
	fn fixed_indirection<'a>(n: Node<'a>, journal: &mut Journal) -> MaybeChanged<'a> {
		match n {
			Node::Extension(partial, payload) if payload.len() >= 32 && Rlp::new(payload).is_list() => {
				// make indirect
				MaybeChanged::Changed(Node::Extension(partial, &Node::decoded(payload).encoded_and_added(journal)).encoded())
			}
			Node::Branch(payloads, value) => {
				// check each child isn't too big
				// TODO OPTIMISE - should really check at the point of (re-)constructing the branch.
				for i in 0..16 {
					if payloads[i].len() >= 32 && Rlp::new(payloads[i]).is_list() {
						let n = Node::decoded(payloads[i]).encoded_and_added(journal);
						let mut new_nodes = payloads;
						new_nodes[i] = &n;
						return MaybeChanged::Changed(Node::Branch(new_nodes, value).encoded());
					}
				}
				MaybeChanged::Same(n)
			}
			_ => MaybeChanged::Same(n),
		}
	}

	/// Given a node `n` which may be in an _invalid state_, fix it such that it is then in a valid
	/// state.
	///
	/// _invalid state_ means:
	/// - Branch node where there is only a single entry;
	/// - Extension node followed by anything other than a Branch node.
	/// - Extension node with a child which has too many bytes to be inline.
	///
	/// `journal` will record the database updates so as to make the returned RLP valid through inserting
	/// and deleting nodes as necessary.
	///
	/// **This operation will not insert the new node nor destroy the original.**
	fn fixed<'a, 'b>(&'a self, n: Node<'b>, journal: &mut Journal) -> MaybeChanged<'b>
		where 'a: 'b,
	{
		trace!("fixed node={:?}", n);
		match n {
			Node::Branch(nodes, node_value) => {
				// if only a single value, transmute to leaf/extension and feed through fixed.
				#[derive(Debug)]
				enum UsedIndex {
					None,
					One(u8),
					Many,
				};
				let mut used_index = UsedIndex::None;
				for i in 0..16 {
					match (nodes[i] == NULL_RLP, &used_index) {
						(false, &UsedIndex::None) => used_index = UsedIndex::One(i as u8),
						(false, &UsedIndex::One(_)) => used_index = UsedIndex::Many,
						(_, _) => {}
					}
				}
				trace!("branch: used_index={:?}, node_value={:?}", used_index, node_value);
				match (used_index, node_value) {
					(UsedIndex::None, None) => panic!("Branch with no subvalues. Something went wrong."),
					(UsedIndex::One(a), None) => {
						// one onward node
						// transmute to extension.
						// TODO: OPTIMISE: - don't call fixed again but put the right node in straight away here.
						// call fixed again since the transmute may cause invalidity.
						let new_partial: [u8; 1] = [a; 1];
						MaybeChanged::Changed(Self::encoded(self.fixed(Node::Extension(NibbleSlice::new_offset(&new_partial[..], 1), nodes[a as usize]), journal)))
					}
					(UsedIndex::None, Some(value)) => {
						// one leaf value
						// transmute to leaf.
						// call fixed again since the transmute may cause invalidity.
						MaybeChanged::Changed(Self::encoded(self.fixed(Node::Leaf(NibbleSlice::new(&b""[..]), value), journal)))
					}
					_ => {
						// onwards node(s) and/or leaf
						// no transmute needed, but should still fix the indirection.
						trace!("no-transmute: FIXINDIRECTION");
						Self::fixed_indirection(Node::Branch(nodes, node_value), journal)
					}
				}
			}
			Node::Extension(partial, payload) => {
				match Node::decoded(self.get_raw_or_lookup(payload)) {
					Node::Extension(sub_partial, sub_payload) => {
						// combine with node below
						journal.delete_node(payload);
						MaybeChanged::Changed(Self::encoded(Self::fixed_indirection(Node::Extension(NibbleSlice::new_composed(&partial, &sub_partial), sub_payload), journal)))
					}
					Node::Leaf(sub_partial, sub_value) => {
						// combine with node below
						journal.delete_node(payload);
						MaybeChanged::Changed(Self::encoded(Self::fixed_indirection(Node::Leaf(NibbleSlice::new_composed(&partial, &sub_partial), sub_value), journal)))
					}
					// no change, might still have an oversize node inline - fix indirection
					_ => Self::fixed_indirection(n, journal),
				}
			}
			// leaf or empty. no change.
			n => {
				MaybeChanged::Same(n)
			}
		}
	}

	/// Determine the RLP of the node, assuming we're removing `partial` from the
	/// node currently of data `old`. This will *not* delete any hash of `old` from the database;
	/// it will just return the new RLP that represents the new node.
	/// `None` may be returned should no change be needed.
	///
	/// `journal` will record the database updates so as to make the returned RLP valid through inserting
	/// and deleting nodes as necessary.
	///
	/// **This operation will not insert the new node nor destroy the original.**
	fn cleared_from_slice(&self, old: &[u8], partial: &NibbleSlice, journal: &mut Journal) -> Option<Bytes> {
		self.cleared(Node::decoded(old), partial, journal)
	}

	/// Compose the RLP of the node equivalent to `n` except with the `partial` key removed from its (sub-)trie.
	///
	/// `journal` will record the database updates so as to make the returned RLP valid through inserting
	/// and deleting nodes as necessary.
	///
	/// **This operation will not insert the new node nor destroy the original.**
	fn cleared(&self, n: Node, partial: &NibbleSlice, journal: &mut Journal) -> Option<Bytes> {
		trace!("cleared old={:?}, partial={:?})", n, partial);

		match (n, partial.is_empty()) {
			(Node::Empty, _) => None,
			(Node::Branch(_, None), true) => {
				None
			}
			(Node::Branch(payloads, _), true) => Some(Self::encoded(self.fixed(Node::Branch(payloads, None), journal))),	// matched as leaf-branch - give back fixed branch with it.
			(Node::Branch(payloads, value), false) => {
				// Branch with partial left - route, clear, fix.
				let i: usize = partial.at(0) as usize;
				trace!("branch-with-partial node[{:?}]={:?}", i, payloads[i].pretty());
				self.cleared(self.get_node(payloads[i]), &partial.mid(1), journal).map(|new_payload| {
					trace!("branch-new-payload={:?}; delete-old={:?}", new_payload.pretty(), payloads[i].pretty());

					// downsteam node needed to be changed.
					journal.delete_node(payloads[i]);
					// return fixed up new node.
					let mut new_payloads = payloads;
					new_payloads[i] = &new_payload;
					Self::encoded(self.fixed(Node::Branch(new_payloads, value), journal))
				})
			}
			(Node::Leaf(node_partial, _), _) => {
				trace!("leaf partial={:?}", node_partial);
				match node_partial.common_prefix(partial) {
					cp if cp == partial.len() => {
						// leaf to be deleted - delete it :)
						trace!("matched-prefix (cp={:?}): REPLACE-EMPTY", cp);
						Some(Node::Empty.encoded())
					}
					_ => None,												// anything else and the key doesn't exit - no change.
				}
			}
			(Node::Extension(node_partial, node_payload), _) => {
				trace!("extension partial={:?}, payload={:?}", node_partial, node_payload.pretty());
				match node_partial.common_prefix(partial) {
					cp if cp == node_partial.len() => {
						trace!("matching-prefix (cp={:?}): SKIP,CLEAR,FIXUP", cp);
						// key at end of extension - skip, clear, fix
						self.cleared(self.get_node(node_payload), &partial.mid(node_partial.len()), journal).map(|new_payload| {
							trace!("extension-new-payload={:?}; delete-old={:?}", new_payload.pretty(), node_payload.pretty());
							// downsteam node needed to be changed.
							journal.delete_node(node_payload);
							// return fixed up new node.
							Self::encoded(self.fixed(Node::Extension(node_partial, &new_payload), journal))
						})
					}
					_ => None,	// key in the middle of an extension - doesn't exist.
				}
			}
		}
	}
}

impl<'db> Trie for TrieDBMut<'db> {
	fn root(&self) -> &H256 {
		&self.root
	}

	fn contains(&self, key: &[u8]) -> bool {
		self.get(key).is_some()
	}

	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> Option<&'a [u8]>
		where 'a: 'key,
	{
		self.do_lookup(&NibbleSlice::new(key))
	}
}

impl<'db> TrieMut for TrieDBMut<'db> {
	fn insert(&mut self, key: &[u8], value: &[u8]) {
		match value.is_empty() {
			false => self.insert_ns(&NibbleSlice::new(key), value),
			true => self.remove_ns(&NibbleSlice::new(key)),
		}
	}

	fn remove(&mut self, key: &[u8]) {
		self.remove_ns(&NibbleSlice::new(key));
	}
}

impl<'db> fmt::Debug for TrieDBMut<'db> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		try!(writeln!(f, "c={:?} [", self.hash_count));
		let root_rlp = self.db.lookup(&self.root).expect("Trie root not found!");
		try!(self.fmt_all(Node::decoded(root_rlp), f, 0));
		writeln!(f, "]")
	}
}

#[cfg(test)]
mod tests {
	extern crate json_tests;
	use self::json_tests::{execute_tests_from_directory, trie};
	use triehash::*;
	use hash::*;
	use hashdb::*;
	use memorydb::*;
	use super::*;
	use nibbleslice::*;
	use rlp::*;
	use bytes::ToPretty;
	use super::super::node::*;
	use super::super::trietraits::*;
	use super::super::standardmap::*;

	fn populate_trie<'db>(db: &'db mut HashDB, root: &'db mut H256, v: &[(Vec<u8>, Vec<u8>)]) -> TrieDBMut<'db> {
		let mut t = TrieDBMut::new(db, root);
		for i in 0..v.len() {
			let key: &[u8] = &v[i].0;
			let val: &[u8] = &v[i].1;
			t.insert(&key, &val);
		}
		t
	}

	fn unpopulate_trie<'db>(t: &mut TrieDBMut<'db>, v: &[(Vec<u8>, Vec<u8>)]) {
		for i in v {
			let key: &[u8] = &i.0;
			t.remove(&key);
		}
	}

	macro_rules! map({$($key:expr => $value:expr),+ } => {
		{
			let mut m = ::std::collections::HashMap::new();
			$(
				m.insert($key, $value);
			)+
			m
		}
	};);

	#[test]
	fn playpen() {

		// let maps = map!{
		// "six-low" => StandardMap{alphabet: Alphabet::Low, min_key: 6, journal_key: 0, count: 1000},
		// "six-mid" => StandardMap{alphabet: Alphabet::Mid, min_key: 6, journal_key: 0, count: 1000},
		// "six-all" => StandardMap{alphabet: Alphabet::All, min_key: 6, journal_key: 0, count: 1000},
		// "mix-mid" => StandardMap{alphabet: Alphabet::Mid, min_key: 1, journal_key: 5, count: 1000}
		// };
		// for sm in maps {
		// let m = sm.1.make();
		// let t = populate_trie(&m);
		// println!("{:?}: root={:?}, hash_count={:?}", sm.0, t.root(), t.hash_count);
		// };
		// panic!();

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
			}
			.make_with(&mut seed);

			let real = trie_root(x.clone());
			let mut memdb = MemoryDB::new();
			let mut root = H256::new();
			let mut memtrie = populate_trie(&mut memdb, &mut root, &x);
			if *memtrie.root() != real || !memtrie.db_items_remaining().is_empty() {
				println!("TRIE MISMATCH");
				println!("");
				println!("{:?} vs {:?}", memtrie.root(), real);
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
				println!("{:?}", memtrie);
			}
			assert_eq!(*memtrie.root(), real);
			assert!(memtrie.db_items_remaining().is_empty());
			unpopulate_trie(&mut memtrie, &x);
			if *memtrie.root() != SHA3_NULL_RLP || !memtrie.db_items_remaining().is_empty() {
				println!("- TRIE MISMATCH");
				println!("");
				println!("remaining: {:?}", memtrie.db_items_remaining());
				println!("{:?} vs {:?}", memtrie.root(), real);
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
				println!("{:?}", memtrie);
			}
			assert_eq!(*memtrie.root(), SHA3_NULL_RLP);
			assert!(memtrie.db_items_remaining().is_empty());
		}
	}

	#[test]
	fn init() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let t = TrieDBMut::new(&mut memdb, &mut root);
		assert_eq!(*t.root(), SHA3_NULL_RLP);
		assert!(t.is_empty());
	}

	#[test]
	fn insert_on_empty() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		assert_eq!(*t.root(), trie_root(vec![(vec![0x01u8, 0x23], vec![0x01u8, 0x23])]));
	}

	#[test]
	fn remove_to_empty() {
		let big_value = b"00000000000000000000000000000000";

		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t1 = TrieDBMut::new(&mut memdb, &mut root);
		t1.insert(&[0x01, 0x23], &big_value.to_vec());
		t1.insert(&[0x01, 0x34], &big_value.to_vec());
		println!("********************** keys remaining {:?}", t1.db_items_remaining());
		assert!(t1.db_items_remaining().is_empty());
		let mut memdb2 = MemoryDB::new();
		let mut root2 = H256::new();
		let mut t2 = TrieDBMut::new(&mut memdb2, &mut root2);
		t2.insert(&[0x01], &big_value.to_vec());
		t2.insert(&[0x01, 0x23], &big_value.to_vec());
		t2.insert(&[0x01, 0x34], &big_value.to_vec());
		t2.remove(&[0x01]);
		assert!(t2.db_items_remaining().is_empty());
		// if t1.root() != t2.root()
		{
			trace!("{:?}", t1);
			trace!("{:?}", t2);
		}
	}

	#[test]
	fn insert_replace_root() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0x01u8, 0x23], &[0x23u8, 0x45]);
		assert_eq!(*t.root(), trie_root(vec![(vec![0x01u8, 0x23], vec![0x23u8, 0x45])]));
	}

	#[test]
	fn insert_make_branch_root() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0x11u8, 0x23], &[0x11u8, 0x23]);
		assert_eq!(*t.root(), trie_root(vec![(vec![0x01u8, 0x23], vec![0x01u8, 0x23]), (vec![0x11u8, 0x23], vec![0x11u8, 0x23])]));
	}

	#[test]
	fn insert_into_branch_root() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0xf1u8, 0x23], &[0xf1u8, 0x23]);
		t.insert(&[0x81u8, 0x23], &[0x81u8, 0x23]);
		assert_eq!(*t.root(),
		           trie_root(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
			(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
		]));
	}

	#[test]
	fn insert_value_into_branch_root() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[], &[0x0]);
		assert_eq!(*t.root(),
		           trie_root(vec![
			(vec![], vec![0x0]),
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
		]));
	}

	#[test]
	fn insert_split_leaf() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0x01u8, 0x34], &[0x01u8, 0x34]);
		assert_eq!(*t.root(),
		           trie_root(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x01u8, 0x34], vec![0x01u8, 0x34]),
		]));
	}

	#[test]
	fn insert_split_extenstion() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01, 0x23, 0x45], &[0x01]);
		t.insert(&[0x01, 0xf3, 0x45], &[0x02]);
		t.insert(&[0x01, 0xf3, 0xf5], &[0x03]);
		assert_eq!(*t.root(),
		           trie_root(vec![
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
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], big_value0);
		t.insert(&[0x11u8, 0x23], big_value1);
		assert_eq!(*t.root(), trie_root(vec![(vec![0x01u8, 0x23], big_value0.to_vec()), (vec![0x11u8, 0x23], big_value1.to_vec())]));
	}

	#[test]
	fn insert_duplicate_value() {
		let big_value = b"00000000000000000000000000000000";

		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], big_value);
		t.insert(&[0x11u8, 0x23], big_value);
		assert_eq!(*t.root(), trie_root(vec![(vec![0x01u8, 0x23], big_value.to_vec()), (vec![0x11u8, 0x23], big_value.to_vec())]));
	}

	#[test]
	fn test_node_leaf() {
		let k = vec![0x20u8, 0x01, 0x23, 0x45];
		let v: Vec<u8> = From::from("cat");
		let (slice, is_leaf) = NibbleSlice::from_encoded(&k);
		assert_eq!(is_leaf, true);
		let leaf = Node::Leaf(slice, &v);
		let rlp = leaf.encoded();
		let leaf2 = Node::decoded(&rlp);
		assert_eq!(leaf, leaf2);
	}

	#[test]
	fn test_node_extension() {
		let k = vec![0x00u8, 0x01, 0x23, 0x45];
		// in extension, value must be valid rlp
		let v = encode(&"cat");
		let (slice, is_leaf) = NibbleSlice::from_encoded(&k);
		assert_eq!(is_leaf, false);
		let ex = Node::Extension(slice, &v);
		let rlp = ex.encoded();
		let ex2 = Node::decoded(&rlp);
		assert_eq!(ex, ex2);
	}

	#[test]
	fn test_node_empty_branch() {
		let null_rlp = NULL_RLP;
		let branch = Node::Branch([&null_rlp; 16], None);
		let rlp = branch.encoded();
		let branch2 = Node::decoded(&rlp);
		println!("{:?}", rlp);
		assert_eq!(branch, branch2);
	}

	#[test]
	fn test_node_branch() {
		let k = encode(&"cat");
		let mut nodes: [&[u8]; 16] = unsafe { ::std::mem::uninitialized() };
		for i in 0..16 {
			nodes[i] = &k;
		}
		let v: Vec<u8> = From::from("dog");
		let branch = Node::Branch(nodes, Some(&v));
		let rlp = branch.encoded();
		let branch2 = Node::decoded(&rlp);
		assert_eq!(branch, branch2);
	}

	#[test]
	fn test_at_empty() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let t = TrieDBMut::new(&mut memdb, &mut root);
		assert_eq!(t.get(&[0x5]), None);
	}

	#[test]
	fn test_at_one() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		assert_eq!(t.get(&[0x1, 0x23]).unwrap(), &[0x1u8, 0x23]);
	}

	#[test]
	fn test_at_three() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0xf1u8, 0x23], &[0xf1u8, 0x23]);
		t.insert(&[0x81u8, 0x23], &[0x81u8, 0x23]);
		assert_eq!(t.get(&[0x01, 0x23]).unwrap(), &[0x01u8, 0x23]);
		assert_eq!(t.get(&[0xf1, 0x23]).unwrap(), &[0xf1u8, 0x23]);
		assert_eq!(t.get(&[0x81, 0x23]).unwrap(), &[0x81u8, 0x23]);
		assert_eq!(t.get(&[0x82, 0x23]), None);
	}

	#[test]
	fn test_print_trie() {
		let mut memdb = MemoryDB::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0x02u8, 0x23], &[0x01u8, 0x23]);
		t.insert(&[0xf1u8, 0x23], &[0xf1u8, 0x23]);
		t.insert(&[0x81u8, 0x23], &[0x81u8, 0x23]);
		println!("trie:");
		println!("{:?}", t);
		// assert!(false);
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
			}
			.make_with(&mut seed);

			let real = trie_root(x.clone());
			let mut memdb = MemoryDB::new();
			let mut root = H256::new();
			let memtrie = populate_trie(&mut memdb, &mut root, &x);
			let mut y = x.clone();
			y.sort_by(|ref a, ref b| a.0.cmp(&b.0));
			let mut memdb2 = MemoryDB::new();
			let mut root2 = H256::new();
			let memtrie_sorted = populate_trie(&mut memdb2, &mut root2, &y);
			if *memtrie.root() != real || *memtrie_sorted.root() != real {
				println!("TRIE MISMATCH");
				println!("");
				println!("ORIGINAL... {:?}", memtrie.root());
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
				println!("{:?}", memtrie);
				println!("SORTED... {:?}", memtrie_sorted.root());
				for i in &y {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
				println!("{:?}", memtrie_sorted);
			}
			assert_eq!(*memtrie.root(), real);
			assert_eq!(*memtrie_sorted.root(), real);
		}
	}

	#[test]
	fn test_trie_json() {
		println!("Json trie test: ");
		execute_tests_from_directory::<trie::TrieTest, _>("json-tests/json/trie/*.json",
		                                                  &mut |file, input, output| {
			                                                  println!("file: {}", file);

			                                                  let mut memdb = MemoryDB::new();
			                                                  let mut root = H256::new();
			                                                  let mut t = TrieDBMut::new(&mut memdb, &mut root);
			                                                  for operation in input.into_iter() {
				                                                  match operation {
					                                                  trie::Operation::Insert(key, value) => t.insert(&key, &value),
					                                                  trie::Operation::Remove(key) => t.remove(&key),
				                                                  }
				                                                 }

			                                                  assert_eq!(*t.root(), H256::from_slice(&output));
			                                                 });
	}

	#[test]
	fn test_trie_existing() {
		let mut root = H256::new();
		let mut db = MemoryDB::new();
		{
			let mut t = TrieDBMut::new(&mut db, &mut root);
			t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
		}

		{
			let _ = TrieDBMut::from_existing(&mut db, &mut root);
		}
	}
}
