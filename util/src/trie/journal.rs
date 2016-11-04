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

use std::default::Default;
use sha3::*;
use hash::H256;
use bytes::*;
use rlp::*;
use hashdb::*;

/// Type of operation for the backing database - either a new node or a node deletion.
#[derive(Debug)]
enum Operation {
	New(H256, DBValue),
	Delete(H256),
}

/// How many insertions and removals were done in an `apply` operation.
pub struct Score {
	/// Number of insertions.
	pub inserts: usize,
	/// Number of removals.
	pub removes: usize,
}

/// A journal of operations on the backing database.
#[derive(Debug)]
pub struct Journal (Vec<Operation>);

impl Default for Journal {
	fn default() -> Self {
		Journal::new()
	}
}

impl Journal {
	/// Create a new, empty, object.
	pub fn new() -> Journal { Journal(vec![]) }

	/// Given the RLP that encodes a node, append a reference to that node `out` and leave `journal`
	/// such that the reference is valid, once applied.
	pub fn new_node(&mut self, rlp: DBValue, out: &mut RlpStream) {
		if rlp.len() >= 32 {
			let rlp_sha3 = rlp.sha3();

			trace!("new_node: reference node {:?} => {:?}", rlp_sha3, &*rlp);
			out.append(&rlp_sha3);
			self.0.push(Operation::New(rlp_sha3, rlp));
		}
		else {
			trace!("new_node: inline node {:?}", &*rlp);
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

	/// Apply this journal to the HashDB `db` and return the number of insertions and removals done.
	pub fn apply(self, db: &mut HashDB) -> Score {
		trace!("applying {:?} changes", self.0.len());
		let mut ret = Score{inserts: 0, removes: 0};
		for d in self.0 {
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
