use memorydb::*;
use sha3::*;
use hashdb::*;
use hash::*;
use nibbleslice::*;
use bytes::*;
use rlp::*;
//use log::*;

pub const NULL_RLP: [u8; 1] = [0x80; 1];
pub const SHA3_NULL_RLP: H256 = H256( [0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e, 0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21] );

pub trait Trie {
	fn root(&self) -> &H256;
	fn is_empty(&self) -> bool { *self.root() == SHA3_NULL_RLP }

	// TODO: consider returning &[u8]...
	fn contains(&self, key: &[u8]) -> bool;
	fn at(&self, key: &[u8]) -> Option<&[u8]>;
	fn insert(&mut self, key: &[u8], value: &[u8]);
	fn remove(&mut self, key: &[u8]);
}

enum Operation {
	New(H256, Bytes),
	Delete(H256),
}

struct Diff (Vec<Operation>);

impl Diff {
	fn new() -> Diff { Diff(vec![]) }

	/// Given the RLP that encodes a node, append a reference to that node `out` and leave `diff`
	/// such that the reference is valid, once applied.
	fn new_node(&mut self, rlp: Bytes, out: &mut RlpStream) {
		if rlp.len() >= 32 {
			let rlp_sha3 = rlp.sha3();
			out.append(&rlp_sha3);
			self.0.push(Operation::New(rlp_sha3, rlp));
		}
		else {
			out.append_raw(&rlp, 1);
		}
	}

	/// Given the RLP that encodes a now-unused node, leave `diff` in such a state that it is noted.
	fn delete_node_sha3(&mut self, old_sha3: H256) {
		self.0.push(Operation::Delete(old_sha3));
	}

	fn delete_node(&mut self, old: &Rlp) {
		if old.is_data() && old.size() == 32 {
			self.0.push(Operation::Delete(H256::decode(old)));
		}
	}

	fn replace_node(&mut self, old: &Rlp, rlp: Bytes, out: &mut RlpStream) {
		self.delete_node(old);
		self.new_node(rlp, out);
	}
}

pub struct TrieDB {
	db: Box<HashDB>,
	root: H256,
}

impl TrieDB {
	pub fn new<T>(db: T) -> Self where T: HashDB + 'static { TrieDB{ db: Box::new(db), root: H256::new() } }

	pub fn new_boxed(db_box: Box<HashDB>) -> Self { TrieDB{ db: db_box, root: H256::new() } }

	pub fn new_memory() -> Self { TrieDB{ db: Box::new(MemoryDB::new()), root: H256::new() } }

	pub fn init(&mut self) { self.set_root_rlp(&NULL_RLP); }

	pub fn db(&self) -> &HashDB { self.db.as_ref() }

	fn set_root_rlp(&mut self, root_data: &[u8]) {
		self.db.kill(&self.root);
		self.root = self.db.insert(root_data);
		trace!("set_root_rlp {:?} {:?}", root_data, self.root);
	}

	fn apply(&mut self, diff: Diff) {
		for d in diff.0.into_iter() {
			match d {
				Operation::Delete(h) => {
					trace!("TrieDB::apply --- {:?}", &h);
					self.db.kill(&h);
				},
				Operation::New(h, d) => {
					trace!("TrieDB::apply +++ {:?} -> {:?}", &h, &d);
					self.db.emplace(h, d);
				}
			}
		}
	}

	fn add(&mut self, key: &NibbleSlice, value: &[u8]) {
		// determine what the new root is, insert new nodes and remove old as necessary.
		let mut todo: Diff = Diff::new();
		let root_rlp = self.inject(self.db.lookup(&self.root).expect("Trie root not found!"), key, value, &mut todo);
		self.apply(todo);
		self.set_root_rlp(&root_rlp);
	}

	fn compose_leaf(partial: &NibbleSlice, value: &[u8]) -> Bytes {
		trace!("compose_leaf {:?} {:?} ({:?})", partial, value, partial.encoded(true));
		let mut s = RlpStream::new_list(2);
		s.append(&partial.encoded(true));
		s.append(&value);
		let r = s.out();
		trace!("output: -> {:?}", &r);
		r
	}

	fn compose_raw(partial: &NibbleSlice, raw_payload: &[u8], is_leaf: bool) -> Bytes {
		println!("compose_raw {:?} {:?} {:?} ({:?})", partial, raw_payload, is_leaf, partial.encoded(is_leaf));
		let mut s = RlpStream::new_list(2);
		s.append(&partial.encoded(is_leaf));
		s.append_raw(raw_payload, 1);
		let r = s.out();
		println!("output: -> {:?}", &r);
		r
	}

	fn compose_extension(partial: &NibbleSlice, raw_payload: &[u8]) -> Bytes {
		Self::compose_raw(partial, raw_payload, false)
	}

	/// Return the bytes encoding the node represented by `rlp`. It will be unlinked from
	/// the trie.
	fn take_node(&self, rlp: &Rlp, diff: &mut Diff) -> Bytes {
		if rlp.is_list() {
			rlp.raw().to_vec()
		}
		else if rlp.is_data() && rlp.size() == 32 {
			let h = H256::decode(rlp);
			let r = self.db.lookup(&h).expect("Trie root not found!").to_vec();
			diff.delete_node_sha3(h);
			r
		}
		else {
			panic!("Empty or invalid node given?");
		}
	}

	/// Transform an existing extension or leaf node to an invalid single-entry branch.
	///
	/// **This operation will not insert the new node nor destroy the original.**
	fn transmute_extension_to_branch(orig_partial: &NibbleSlice, orig_raw_payload: &[u8], diff: &mut Diff) -> Bytes {
		let mut s = RlpStream::new_list(17);
		assert!(!orig_partial.is_empty());	// extension nodes are not allowed to have empty partial keys.
		let index = orig_partial.at(0);
		// orig is extension - orig_raw_payload is a node itself.
		for i in 0..17 {
			if index == i {
				if orig_partial.len() > 1 {
					// still need an extension
					diff.new_node(Self::compose_extension(&orig_partial.mid(1), orig_raw_payload), &mut s);
				} else {
					// was an extension of length 1 - just redirect the payload into here.
					s.append_raw(orig_raw_payload, 1);
				}
			} else {
				s.append_empty_data();
			}
		}
		s.out()
	}

	fn transmute_leaf_to_branch(orig_partial: &NibbleSlice, orig_raw_payload: &[u8], diff: &mut Diff) -> Bytes {
		let mut s = RlpStream::new_list(17);
		let index = if orig_partial.is_empty() {16} else {orig_partial.at(0)};
		// orig is leaf - orig_raw_payload is data representing the actual value.
		for i in 0..17 {
			if index == i {
				// this is our node.
				diff.new_node(Self::compose_raw(&orig_partial.mid(if i == 16 {0} else {1}), orig_raw_payload, true), &mut s);
			} else {
				s.append_empty_data();
			}
		}
		s.out()
	}

	/// Transform an existing extension or leaf node plus a new partial/value to a two-entry branch.
	///
	/// **This operation will not insert the new node nor destroy the original.**
	fn transmute_to_branch_and_inject(&self, orig_is_leaf: bool, orig_partial: &NibbleSlice, orig_raw_payload: &[u8], partial: &NibbleSlice, value: &[u8], diff: &mut Diff) -> Bytes {
		let intermediate = match orig_is_leaf {
			true => Self::transmute_leaf_to_branch(orig_partial, orig_raw_payload, diff),
			false => Self::transmute_extension_to_branch(orig_partial, orig_raw_payload, diff),
		};
		self.inject(&intermediate, partial, value, diff)
		// TODO: implement without having to make an intermediate representation.
	}

	/// Given a branch node's RLP `orig` together with a `partial` key and `value`, return the
	/// RLP-encoded node that accomodates the trie with the new entry. Mutate `diff` so that
	/// once applied the returned node is valid.
	fn injected_into_branch(&self, orig: &Rlp, partial: &NibbleSlice, value: &[u8], diff: &mut Diff) -> Bytes {
		let mut s = RlpStream::new_list(17);
		let index = if partial.is_empty() {16} else {partial.at(0) as usize};
		for i in 0usize..17 {
			if index == i {
				// this is node to inject into...
				if orig.at(i).is_empty() {
					// easy - original had empty slot.
					diff.new_node(Self::compose_leaf(&partial.mid(if i == 16 {0} else {1}), value), &mut s);
				} else if i == 16 {
					// leaf entry - just replace.
					let new = Self::compose_leaf(&partial.mid(if i == 16 {0} else {1}), value);
					diff.replace_node(&orig.at(i), new, &mut s); 
				} else {
					// harder - original has something there already
					let new = self.inject(orig.at(i).raw(), &partial.mid(1), value, diff);
					diff.replace_node(&orig.at(i), new, &mut s);
				}
			} else {
				s.append_raw(orig.at(i).raw(), 1);
			}
		}
		s.out()
	}



	/// Determine the RLP of the node, assuming we're inserting `partial` into the
	/// node currently of data `old`. This will *not* delete any hash of `old` from the database;
	/// it will just return the new RLP that includes the new node.
	///
	/// The database will be updated so as to make the returned RLP valid through inserting
	/// and deleting nodes as necessary.
	///
	/// **This operation will not insert the new node now destroy the original.**
	fn inject(&self, old: &[u8], partial: &NibbleSlice, value: &[u8], diff: &mut Diff) -> Bytes {
		// already have an extension. either fast_forward, cleve or transmute_to_branch.
		let old_rlp = Rlp::new(old);
		match old_rlp.prototype() {
			Prototype::List(17) => {
				// already have a branch. route and inject.
				self.injected_into_branch(&old_rlp, partial, value, diff)
			},
			Prototype::List(2) => {
				let their_key_rlp = old_rlp.at(0);
				let (them, is_leaf) = NibbleSlice::from_encoded(their_key_rlp.data());
				match partial.common_prefix(&them) {
					0 if partial.is_empty() && them.is_empty() => {
						// both empty: just replace.
						Self::compose_leaf(partial, value)
					},
					0 => {
						// one of us isn't empty: transmute to branch here
						self.transmute_to_branch_and_inject(is_leaf, &them, old_rlp.at(1).raw(), partial, value, diff)
					},
					cp if cp == them.len() => {
						// fully-shared prefix for this extension:
						// skip to the end of this extension and continue the inject there.
						let n = self.take_node(&old_rlp.at(1), diff);
						let downstream_node = self.inject(&n, &partial.mid(cp), value, diff);
						let mut s = RlpStream::new_list(2);
						s.append_raw(old_rlp.at(0).raw(), 1);
						diff.new_node(downstream_node, &mut s);
						s.out()
					},
					cp => {
						// partially-shared prefix for this extension:
						// split into two extensions, high and low, pass the
						// low through inject with the value before inserting the result
						// into high to create the new.

						// TODO: optimise by doing this without creating injected_low.

						// low (farther from root)
						let low = Self::compose_raw(&them.mid(cp), old_rlp.at(1).raw(), is_leaf);
						let injected_low = self.inject(&low, &partial.mid(cp), value, diff);

						// high (closer to root)
						let mut s = RlpStream::new_list(2);
						s.append(&them.encoded_leftmost(cp, false));
						diff.new_node(injected_low, &mut s);
						s.out()
					},
				}
			},
			Prototype::Data(0) => {
				Self::compose_leaf(partial, value)
			},
			_ => panic!("Invalid RLP for node."),
		}
	}
}

impl Trie for TrieDB {
	fn root(&self) -> &H256 { &self.root }

	fn contains(&self, _key: &[u8]) -> bool {
		unimplemented!();
	}

	fn at(&self, _key: &[u8]) -> Option<&[u8]> {
		unimplemented!();
	}

	fn insert(&mut self, key: &[u8], value: &[u8]) {
		self.add(&NibbleSlice::new(key), value);
	}

	fn remove(&mut self, _key: &[u8]) {
		unimplemented!();
	}
}

#[test]
fn playpen() {
	use overlaydb::*;
	use triehash::*;
	use env_logger;

	env_logger::init().unwrap();

	(&[1, 2, 3]).starts_with(&[1, 2]);

	let mut t = TrieDB::new(OverlayDB::new_temp());
	t.init();
	assert_eq!(*t.root(), SHA3_NULL_RLP);
	assert!(t.is_empty());

	t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]);
	assert_eq!(*t.root(), trie_root(vec![ (vec![1u8, 0x23], vec![1u8, 0x23]) ]));
}
