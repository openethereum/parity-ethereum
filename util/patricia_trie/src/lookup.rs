// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Trie lookup via HashDB.

use hashdb::HashDB;
use nibbleslice::NibbleSlice;
use rlp::Rlp;
use bigint::hash::H256;

use super::{TrieError, Query};
use super::node::Node;

/// Trie lookup helper object.
pub struct Lookup<'a, Q: Query> {
	/// database to query from.
	pub db: &'a HashDB,
	/// Query object to record nodes and transform data.
	pub query: Q,
	/// Hash to start at
	pub hash: H256,
}

impl<'a, Q: Query> Lookup<'a, Q> {
	/// Look up the given key. If the value is found, it will be passed to the given
	/// function to decode or copy.
	pub fn look_up(mut self, mut key: NibbleSlice) -> super::Result<Option<Q::Item>> {
		let mut hash = self.hash;

		// this loop iterates through non-inline nodes.
		for depth in 0.. {
			let node_data = match self.db.get(&hash) {
				Some(value) => value,
				None => return Err(Box::new(match depth {
					0 => TrieError::InvalidStateRoot(hash),
					_ => TrieError::IncompleteDatabase(hash),
				})),
			};

			self.query.record(&hash, &node_data, depth);

			// this loop iterates through all inline children (usually max 1)
			// without incrementing the depth.
			let mut node_data = &node_data[..];
			loop {
				match Node::decoded(node_data) {
					Node::Leaf(slice, value) => {
						return Ok(match slice == key {
							true => Some(self.query.decode(value)),
							false => None,
						})
					}
					Node::Extension(slice, item) => {
						if key.starts_with(&slice) {
							node_data = item;
							key = key.mid(slice.len());
						} else {
							return Ok(None)
						}
					}
					Node::Branch(children, value) => match key.is_empty() {
						true => return Ok(value.map(move |val| self.query.decode(val))),
						false => {
							node_data = children[key.at(0) as usize];
							key = key.mid(1);
						}
					},
					_ => return Ok(None),
				}

				// check if new node data is inline or hash.
				let r = Rlp::new(node_data);
				if r.is_data() && r.size() == 32 {
					hash = r.as_val();
					break
				}
			}
		}
		Ok(None)
	}
}
