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

//! Trie query recorder.

use keccak::keccak;
use ethereum_types::H256;
use bytes::Bytes;

/// A record of a visited node.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Record {
	/// The depth of this node.
	pub depth: u32,

	/// The raw data of the node.
	pub data: Bytes,

	/// The hash of the data.
	pub hash: H256,
}

/// Records trie nodes as they pass it.
#[derive(Debug)]
pub struct Recorder {
	nodes: Vec<Record>,
	min_depth: u32,
}

impl Default for Recorder {
	fn default() -> Self {
		Recorder::new()
	}
}

impl Recorder {
	/// Create a new `Recorder` which records all given nodes.
	#[inline]
	pub fn new() -> Self {
		Recorder::with_depth(0)
	}

	/// Create a `Recorder` which only records nodes beyond a given depth.
	pub fn with_depth(depth: u32) -> Self {
		Recorder {
			nodes: Vec::new(),
			min_depth: depth,
		}
	}

	/// Record a visited node, given its hash, data, and depth.
	pub fn record(&mut self, hash: &H256, data: &[u8], depth: u32) {
		debug_assert_eq!(keccak(data), *hash);

		if depth >= self.min_depth {
			self.nodes.push(Record {
				depth: depth,
				data: data.into(),
				hash: *hash,
			})
		}
	}

	/// Drain all visited records.
	pub fn drain(&mut self) -> Vec<Record> {
		::std::mem::replace(&mut self.nodes, Vec::new())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use ethereum_types::H256;

	#[test]
	fn basic_recorder() {
		let mut basic = Recorder::new();

		let node1 = vec![1, 2, 3, 4];
		let node2 = vec![4, 5, 6, 7, 8, 9, 10];

		let (hash1, hash2) = (keccak(&node1), keccak(&node2));
		basic.record(&hash1, &node1, 0);
		basic.record(&hash2, &node2, 456);

		let record1 = Record {
			data: node1,
			hash: hash1,
			depth: 0,
		};

		let record2 = Record {
			data: node2,
			hash: hash2,
			depth: 456
		};

		assert_eq!(basic.drain(), vec![record1, record2]);
	}

	#[test]
	fn basic_recorder_min_depth() {
		let mut basic = Recorder::with_depth(400);

		let node1 = vec![1, 2, 3, 4];
		let node2 = vec![4, 5, 6, 7, 8, 9, 10];

		let hash1 = keccak(&node1);
		let hash2 = keccak(&node2);
		basic.record(&hash1, &node1, 0);
		basic.record(&hash2, &node2, 456);

		let records = basic.drain();

		assert_eq!(records.len(), 1);

		assert_eq!(records[0].clone(), Record {
			data: node2,
			hash: hash2,
			depth: 456,
		});
	}

	#[test]
	fn trie_record() {
		use super::super::{TrieDB, TrieDBMut, Trie, TrieMut};
		use memorydb::MemoryDB;

		let mut db = MemoryDB::new();

		let mut root = H256::default();

		{
			let mut x = TrieDBMut::new(&mut db, &mut root);

			x.insert(b"dog", b"cat").unwrap();
			x.insert(b"lunch", b"time").unwrap();
			x.insert(b"notdog", b"notcat").unwrap();
			x.insert(b"hotdog", b"hotcat").unwrap();
			x.insert(b"letter", b"confusion").unwrap();
			x.insert(b"insert", b"remove").unwrap();
			x.insert(b"pirate", b"aargh!").unwrap();
			x.insert(b"yo ho ho", b"and a bottle of rum").unwrap();
		}

		let trie = TrieDB::new(&db, &root).unwrap();
		let mut recorder = Recorder::new();

		trie.get_with(b"pirate", &mut recorder).unwrap().unwrap();

		let nodes: Vec<_> = recorder.drain().into_iter().map(|r| r.data).collect();
		assert_eq!(nodes, vec![
			vec![
				248, 81, 128, 128, 128, 128, 128, 128, 160, 50, 19, 71, 57, 213, 63, 125, 149,
				92, 119, 88, 96, 80, 126, 59, 11, 160, 142, 98, 229, 237, 200, 231, 224, 79, 118,
				215, 93, 144, 246, 179, 176, 160, 118, 211, 171, 199, 172, 136, 136, 240, 221, 59,
				110, 82, 86, 54, 23, 95, 48, 108, 71, 125, 59, 51, 253, 210, 18, 116, 79, 0, 236,
				102, 142, 48, 128, 128, 128, 128, 128, 128, 128, 128, 128
			],
			vec![
				248, 60, 206, 134, 32, 105, 114, 97, 116, 101, 134, 97, 97, 114, 103, 104, 33,
				128, 128, 128, 128, 128, 128, 128, 128, 221, 136, 32, 111, 32, 104, 111, 32, 104,
				111, 147, 97, 110, 100, 32, 97, 32, 98, 111, 116, 116, 108, 101, 32, 111, 102,
				32, 114, 117, 109, 128, 128, 128, 128, 128, 128, 128
			]
		]);

		trie.get_with(b"letter", &mut recorder).unwrap().unwrap();

		let nodes: Vec<_> = recorder.drain().into_iter().map(|r| r.data).collect();
		assert_eq!(nodes, vec![
			vec![
				248, 81, 128, 128, 128, 128, 128, 128, 160, 50, 19, 71, 57, 213, 63, 125, 149,
				92, 119, 88, 96, 80, 126, 59, 11, 160, 142, 98, 229, 237, 200, 231, 224, 79, 118,
				215, 93, 144, 246, 179, 176, 160, 118, 211, 171, 199, 172, 136, 136, 240, 221,
				59, 110, 82, 86, 54, 23, 95, 48, 108, 71, 125, 59, 51, 253, 210, 18, 116, 79,
				0, 236, 102, 142, 48, 128, 128, 128, 128, 128, 128, 128, 128, 128
			],
			vec![
				248, 99, 128, 128, 128, 128, 200, 131, 32, 111, 103, 131, 99, 97, 116, 128, 128,
				128, 206, 134, 32, 111, 116, 100, 111, 103, 134, 104, 111, 116, 99, 97, 116, 206,
				134, 32, 110, 115, 101, 114, 116, 134, 114, 101, 109, 111, 118, 101, 128, 128,
				160, 202, 250, 252, 153, 229, 63, 255, 13, 100, 197, 80, 120, 190, 186, 92, 5,
				255, 135, 245, 205, 180, 213, 161, 8, 47, 107, 13, 105, 218, 1, 9, 5, 128,
				206, 134, 32, 111, 116, 100, 111, 103, 134, 110, 111, 116, 99, 97, 116, 128, 128
			],
			vec![
				235, 128, 128, 128, 128, 128, 128, 208, 133, 53, 116, 116, 101, 114, 137, 99,
				111, 110, 102, 117, 115, 105, 111, 110, 202, 132, 53, 110, 99, 104, 132, 116,
				105, 109, 101, 128, 128, 128, 128, 128, 128, 128, 128, 128
			]
		]);
	}
}
