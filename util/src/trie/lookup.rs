// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use hashdb::{DBValue, HashDB};
use nibbleslice::NibbleSlice;
use rlp::{Rlp, View};
use ::{H256};

use super::TrieError;
use super::node::Node;
use super::recorder::Recorder;

/// Trie lookup helper object.
pub struct Lookup<'a, R: 'a + Recorder> {
	/// database to query from.
	pub db: &'a HashDB,
	/// Recorder to write into.
	pub rec: &'a mut R,
	/// Hash to start at
	pub hash: H256,
}

impl<'a, R: 'a + Recorder> Lookup<'a, R> {
	/// Look up the given key.
	pub fn look_up(self, mut key: NibbleSlice) -> super::Result<Option<DBValue>> {
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

			self.rec.record(&hash, &node_data, depth);

			// this loop iterates through all inline children (usually max 1)
			// without incrementing the depth.
			let mut node_data = &node_data[..];
			loop {
				match Node::decoded(node_data) {
					Node::Leaf(slice, value) => {
						return Ok(match slice == key {
							true => Some(DBValue::from_slice(value)),
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
						true => return Ok(value.map(DBValue::from_slice)),
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
