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

use sha3::Hashable;
use {Bytes, H256};

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

/// Trie node recorder.
///
/// These are used to record which nodes are visited during a trie query.
/// Inline nodes are not to be recorded, as they are contained within their parent.
pub trait Recorder {

	/// Record that the given node has been visited.
	///
	/// The depth parameter is the depth of the visited node, with the root node having depth 0.
	fn record(&mut self, hash: &H256, data: &[u8], depth: u32);

	/// Record that the given node (with unknown hash) has been visited.
	/// The depth parameter is the depth of the visited node.
	fn record_unknown(&mut self, data: &[u8], depth: u32);

	/// Drain all accepted records from the recorder in no particular order.
	fn drain(self) -> Vec<Record> where Self: Sized;
}

/// A no-op trie recorder. This ignores everything which is thrown at it.
pub struct NoOp;

impl Recorder for NoOp {
	#[inline]
	fn record(&mut self, _hash: &H256, _data: &[u8], _depth: u32) {}

	#[inline]
	fn record_unknown(&mut self, _data: &[u8], _depth: u32) {}

	#[inline]
	fn drain(self) -> Vec<Record> { Vec::new() }
}

/// A simple recorder. Does nothing fancy but fulfills the `Recorder` interface
/// properly.
pub struct BasicRecorder {
	nodes: Vec<Record>,
	min_depth: u32,
}

impl BasicRecorder {
	/// Create a new `BasicRecorder` which records all given nodes.
	#[inline]
	pub fn new() -> Self {
		BasicRecorder::with_depth(0)
	}

	/// Create a `BasicRecorder` which only records nodes beyond a given depth.
	pub fn with_depth(depth: u32) -> Self {
		BasicRecorder {
			nodes: Vec::new(),
			min_depth: depth,
		}
	}
}

impl Recorder for BasicRecorder {
	fn record(&mut self, hash: &H256, data: &[u8], depth: u32) {
		debug_assert_eq!(data.sha3(), *hash);

		if depth >= self.min_depth {
			self.nodes.push(Record {
				depth: depth,
				data: data.into(),
				hash: *hash,
			})
		}
	}

	fn record_unknown(&mut self, data: &[u8], depth: u32) {
		if depth >= self.min_depth {
			self.nodes.push(Record {
				depth: depth,
				data: data.into(),
				hash: data.sha3(),
			})
		}
	}

	fn drain(self) -> Vec<Record> {
		self.nodes
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sha3::Hashable;

	#[test]
	fn no_op_does_nothing() {
		let mut no_op = NoOp;
		no_op.record(&Default::default(), &[], 1);
		no_op.record_unknown(&[], 2);

		assert_eq!(no_op.drain(), Vec::new());
	}

	#[test]
	fn basic_recorder() {
		let mut basic = BasicRecorder::new();

		let node1 = vec![1, 2, 3, 4];
		let node2 = vec![4, 5, 6, 7, 8, 9, 10];

		let (hash1, hash2) = (node1.sha3(), node2.sha3());
		basic.record(&hash1, &node1, 0);
		basic.record_unknown(&node2, 456);

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
		let mut basic = BasicRecorder::with_depth(400);

		let node1 = vec![1, 2, 3, 4];
		let node2 = vec![4, 5, 6, 7, 8, 9, 10];

		let hash2 = node2.sha3();
		basic.record_unknown(&node1, 0);
		basic.record(&hash2, &node2, 456);

		let records = basic.drain();

		assert_eq!(records.len(), 1);

		assert_eq!(records[0].clone(), Record {
			data: node2,
			hash: hash2,
			depth: 456,
		});
	}
}