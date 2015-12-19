use sha3::*;
use hash::H256;
use bytes::*;
use rlp::*;
use hashdb::*;

/// Type of operation for the backing database - either a new node or a node deletion.
#[derive(Debug)]
enum Operation {
	New(H256, Bytes),
	Delete(H256),
}

pub struct Score {
	pub inserts: usize,
	pub removes: usize,
}

/// A journal of operations on the backing database.
#[derive(Debug)]
pub struct Journal (Vec<Operation>);

impl Journal {
	/// Create a new, empty, object.
	pub fn new() -> Journal { Journal(vec![]) }

	/// Given the RLP that encodes a node, append a reference to that node `out` and leave `journal`
	/// such that the reference is valid, once applied.
	pub fn new_node(&mut self, rlp: Bytes, out: &mut RlpStream) {
		if rlp.len() >= 32 {
			let rlp_sha3 = rlp.sha3();

			trace!("new_node: reference node {:?} => {:?}", rlp_sha3, rlp.pretty());
			out.append(&rlp_sha3);
			self.0.push(Operation::New(rlp_sha3, rlp));
		}
		else {
			trace!("new_node: inline node {:?}", rlp.pretty());
			out.append_raw(&rlp, 1);
		}
	}

	/// Given the RLP that encodes a now-unused node, leave `journal` in such a state that it is noted.
	pub fn delete_node_sha3(&mut self, old_sha3: H256) {
		trace!("delete_node:  {:?}", old_sha3);
		self.0.push(Operation::Delete(old_sha3));
	}

	/// Register an RLP-encoded node for deletion (given a slice), if it needs to be deleted.
	pub fn delete_node(&mut self, old: &[u8]) {
		let r = Rlp::new(old);
		if r.is_data() && r.size() == 32 {
			self.delete_node_sha3(r.as_val());
		}
	}

	pub fn apply(self, db: &mut HashDB) -> Score {
		trace!("applying {:?} changes", self.0.len());
		let mut ret = Score{inserts: 0, removes: 0};
		for d in self.0.into_iter() {
			match d {
				Operation::Delete(h) => {
					trace!("TrieDBMut::apply --- {:?}", &h);
					db.remove(&h);
					ret.removes += 1;
				},
				Operation::New(h, d) => {
					trace!("TrieDBMut::apply +++ {:?} -> {:?}", &h, d.pretty());
					db.emplace(h, d);
					ret.inserts += 1;
				}
			}
		}
		ret
	}
}
