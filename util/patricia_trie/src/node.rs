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

use elastic_array::ElasticArray36;
use nibbleslice::NibbleSlice;
use nibblevec::NibbleVec;
use bytes::*;
use rlp::*;
use hashdb::DBValue;

/// Partial node key type.
pub type NodeKey = ElasticArray36<u8>;

/// Type of node in the trie and essential information thereof.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Node<'a> {
	/// Null trie node; could be an empty root or an empty branch entry.
	Empty,
	/// Leaf node; has key slice and value. Value may not be empty.
	Leaf(NibbleSlice<'a>, &'a [u8]),
	/// Extension node; has key slice and node data. Data may not be null.
	Extension(NibbleSlice<'a>, &'a [u8]),
	/// Branch node; has array of 16 child nodes (each possibly null) and an optional immediate node data.
	Branch([&'a [u8]; 16], Option<&'a [u8]>)
}

impl<'a> Node<'a> {
	/// Decode the `node_rlp` and return the Node.
	pub fn decoded(node_rlp: &'a [u8]) -> Self {
		let r = Rlp::new(node_rlp);
		match r.prototype() {
			// either leaf or extension - decode first item with NibbleSlice::???
			// and use is_leaf return to figure out which.
			// if leaf, second item is a value (is_data())
			// if extension, second item is a node (either SHA3 to be looked up and
			// fed back into this function or inline RLP which can be fed back into this function).
			Prototype::List(2) => match NibbleSlice::from_encoded(r.at(0).data()) {
				(slice, true) => Node::Leaf(slice, r.at(1).data()),
				(slice, false) => Node::Extension(slice, r.at(1).as_raw()),
			},
			// branch - first 16 are nodes, 17th is a value (or empty).
			Prototype::List(17) => {
				let mut nodes = [&[] as &[u8]; 16];
				for i in 0..16 {
					nodes[i] = r.at(i).as_raw();
				}
				Node::Branch(nodes, if r.at(16).is_empty() { None } else { Some(r.at(16).data()) })
			},
			// an empty branch index.
			Prototype::Data(0) => Node::Empty,
			// something went wrong.
			_ => panic!("Rlp is not valid.")
		}
	}

	/// Encode the node into RLP.
	///
	/// Will always return the direct node RLP even if it's 32 or more bytes. To get the
	/// RLP which would be valid for using in another node, use `encoded_and_added()`.
	pub fn encoded(&self) -> Bytes {
		match *self {
			Node::Leaf(ref slice, ref value) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&&*slice.encoded(true));
				stream.append(value);
				stream.out()
			},
			Node::Extension(ref slice, ref raw_rlp) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&&*slice.encoded(false));
				stream.append_raw(raw_rlp, 1);
				stream.out()
			},
			Node::Branch(ref nodes, ref value) => {
				let mut stream = RlpStream::new_list(17);
				for i in 0..16 {
					stream.append_raw(nodes[i], 1);
				}
				match *value {
					Some(ref n) => { stream.append(n); },
					None => { stream.append_empty_data(); },
				}
				stream.out()
			},
			Node::Empty => {
				let mut stream = RlpStream::new();
				stream.append_empty_data();
				stream.out()
			}
		}
	}
}

/// An owning node type. Useful for trie iterators.
#[derive(Debug, PartialEq, Eq)]
pub enum OwnedNode {
	/// Empty trie node.
	Empty,
	/// Leaf node: partial key and value.
	Leaf(NibbleVec, DBValue),
	/// Extension node: partial key and child node.
	Extension(NibbleVec, DBValue),
	/// Branch node: 16 children and an optional value.
	Branch([NodeKey; 16], Option<DBValue>),
}

impl Clone for OwnedNode {
	fn clone(&self) -> Self {
		match *self {
			OwnedNode::Empty => OwnedNode::Empty,
			OwnedNode::Leaf(ref k, ref v) => OwnedNode::Leaf(k.clone(), v.clone()),
			OwnedNode::Extension(ref k, ref c) => OwnedNode::Extension(k.clone(), c.clone()),
			OwnedNode::Branch(ref c, ref v) => {
				let mut children = [
					NodeKey::new(), NodeKey::new(), NodeKey::new(), NodeKey::new(),
					NodeKey::new(), NodeKey::new(), NodeKey::new(), NodeKey::new(),
					NodeKey::new(), NodeKey::new(), NodeKey::new(), NodeKey::new(),
					NodeKey::new(), NodeKey::new(), NodeKey::new(), NodeKey::new(),
				];

				for (owned, borrowed) in children.iter_mut().zip(c.iter()) {
					*owned = borrowed.clone()
				}

				OwnedNode::Branch(children, v.as_ref().cloned())
			}
		}
	}
}

impl<'a> From<Node<'a>> for OwnedNode {
	fn from(node: Node<'a>) -> Self {
		match node {
			Node::Empty => OwnedNode::Empty,
			Node::Leaf(k, v) => OwnedNode::Leaf(k.into(), DBValue::from_slice(v)),
			Node::Extension(k, child) => OwnedNode::Extension(k.into(), DBValue::from_slice(child)),
			Node::Branch(c, val) => {
				let mut children = [
					NodeKey::new(), NodeKey::new(), NodeKey::new(), NodeKey::new(),
					NodeKey::new(), NodeKey::new(), NodeKey::new(), NodeKey::new(),
					NodeKey::new(), NodeKey::new(), NodeKey::new(), NodeKey::new(),
					NodeKey::new(), NodeKey::new(), NodeKey::new(), NodeKey::new(),
				];

				for (owned, borrowed) in children.iter_mut().zip(c.iter()) {
					*owned = NodeKey::from_slice(borrowed)
				}

				OwnedNode::Branch(children, val.map(DBValue::from_slice))
			}
		}
	}
}
