// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
	Branch([&'a [u8]; 16], Option<&'a [u8]>),
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

impl<'a> From<Node<'a>> for OwnedNode {
	fn from(node: Node<'a>) -> Self {
		match node {
			Node::Empty => OwnedNode::Empty,
			Node::Leaf(k, v) => OwnedNode::Leaf(k.into(), DBValue::from_slice(v)),
			Node::Extension(k, child) => OwnedNode::Extension(k.into(), DBValue::from_slice(child)),
			Node::Branch(c, val) => {
				let children = [
					NodeKey::from_slice(c[0]), NodeKey::from_slice(c[1]), NodeKey::from_slice(c[2]), NodeKey::from_slice(c[3]),
					NodeKey::from_slice(c[4]), NodeKey::from_slice(c[5]), NodeKey::from_slice(c[6]), NodeKey::from_slice(c[7]),
					NodeKey::from_slice(c[8]), NodeKey::from_slice(c[9]), NodeKey::from_slice(c[10]), NodeKey::from_slice(c[11]),
					NodeKey::from_slice(c[12]), NodeKey::from_slice(c[13]), NodeKey::from_slice(c[14]), NodeKey::from_slice(c[15]),
				];

				OwnedNode::Branch(children, val.map(DBValue::from_slice))
			}
		}
	}
}
