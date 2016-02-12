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

use std::collections::HashMap;
use std::collections::hash_map::Values;
use network::node::*;
use network::discovery::TableUpdates;

pub struct NodeTable {
	nodes: HashMap<NodeId, Node>
}

impl NodeTable {
	pub fn new(_path: Option<String>) -> NodeTable {
		NodeTable {
			nodes: HashMap::new()
		}
	}

	pub fn add_node(&mut self, node: Node) {
		self.nodes.insert(node.id.clone(), node);
	}

	pub fn nodes(&self) -> Values<NodeId, Node> {
		self.nodes.values()
	}

	pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
		self.nodes.get_mut(id)
	}

	pub fn update(&mut self, mut update: TableUpdates) {
		self.nodes.extend(update.added.drain());
		for r in update.removed {
			self.nodes.remove(&r);
		}
	}

}
