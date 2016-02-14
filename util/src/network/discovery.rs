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

// This module is a work in progress

#![allow(dead_code)] //TODO: remove this after everything is done

use std::collections::{HashSet, BTreeMap};
use std::cell::{RefCell};
use std::ops::{DerefMut};
use mio::*;
use mio::udp::*;
use hash::*;
use sha3::Hashable;
use crypto::*;
use network::node::*;

const ADDRESS_BYTES_SIZE: u32 = 32;							///< Size of address type in bytes.
const ADDRESS_BITS: u32 = 8 * ADDRESS_BYTES_SIZE;			///< Denoted by n in [Kademlia].
const NODE_BINS: u32 = ADDRESS_BITS - 1;					///< Size of m_state (excludes root, which is us).
const DISCOVERY_MAX_STEPS: u16 = 8;							///< Max iterations of discovery. (discover)
const BUCKET_SIZE: u32 = 16;		///< Denoted by k in [Kademlia]. Number of nodes stored in each bucket.
const ALPHA: usize = 3;				///< Denoted by \alpha in [Kademlia]. Number of concurrent FindNode requests.

struct NodeBucket {
	distance: u32,
	nodes: Vec<NodeId>
}

impl NodeBucket {
	fn new(distance: u32) -> NodeBucket {
		NodeBucket {
			distance: distance,
			nodes: Vec::new()
		}
	}
}

struct Discovery {
	id: NodeId,
	discovery_round: u16,
	discovery_id: NodeId,
	discovery_nodes: HashSet<NodeId>,
	node_buckets: Vec<NodeBucket>,
}

struct FindNodePacket;

impl FindNodePacket {
	fn new(_endpoint: &NodeEndpoint, _id: &NodeId) -> FindNodePacket {
		FindNodePacket
	}

	fn sign(&mut self, _secret: &Secret) {
	}

	fn send(& self, _socket: &mut UdpSocket) {
	}
}

impl Discovery {
	pub fn new(id: &NodeId) -> Discovery {
		Discovery {
			id: id.clone(),
			discovery_round: 0,
			discovery_id: NodeId::new(),
			discovery_nodes: HashSet::new(),
			node_buckets: (0..NODE_BINS).map(NodeBucket::new).collect(),
		}
	}

	pub fn add_node(&mut self, id: &NodeId) {
		self.node_buckets[Discovery::distance(&self.id, &id) as usize].nodes.push(id.clone());
	}

	fn start_node_discovery<Host:Handler>(&mut self, event_loop: &mut EventLoop<Host>) {
		self.discovery_round = 0;
		self.discovery_id.randomize();
		self.discovery_nodes.clear();
		self.discover(event_loop);
	}

	fn discover<Host:Handler>(&mut self, event_loop: &mut EventLoop<Host>) {
		if self.discovery_round == DISCOVERY_MAX_STEPS
		{
			debug!("Restarting discovery");
			self.start_node_discovery(event_loop);
			return;
		}
		let mut tried_count = 0;
		{
			let nearest = Discovery::nearest_node_entries(&self.id, &self.discovery_id, &self.node_buckets).into_iter();
			let nodes = RefCell::new(&mut self.discovery_nodes);
			let nearest = nearest.filter(|x| nodes.borrow().contains(&x)).take(ALPHA);
			for r in nearest {
				//let mut p = FindNodePacket::new(&r.endpoint, &self.discovery_id);
				//p.sign(&self.secret);
				//p.send(&mut self.udp_socket);
				let mut borrowed = nodes.borrow_mut();
				borrowed.deref_mut().insert(r.clone());
				tried_count += 1;
			}
		}

		if tried_count == 0
		{
			debug!("Restarting discovery");
			self.start_node_discovery(event_loop);
			return;
		}
		self.discovery_round += 1;
		//event_loop.timeout_ms(Token(NODETABLE_DISCOVERY), 1200).unwrap();
	}

	fn distance(a: &NodeId, b: &NodeId) -> u32 {
		let d = a.sha3() ^ b.sha3();
		let mut ret:u32 = 0;
		for i in 0..32 {
			let mut v: u8 = d[i];
			while v != 0 {
				v >>= 1;
				ret += 1;
			}
		}
		ret
	}

	#[allow(cyclomatic_complexity)]
	fn nearest_node_entries<'b>(source: &NodeId, target: &NodeId, buckets: &'b [NodeBucket]) -> Vec<&'b NodeId>
	{
		// send ALPHA FindNode packets to nodes we know, closest to target
		const LAST_BIN: u32 = NODE_BINS - 1;
		let mut head = Discovery::distance(source, target);
		let mut tail = if head == 0  { LAST_BIN } else { (head - 1) % NODE_BINS };

		let mut found: BTreeMap<u32, Vec<&'b NodeId>> = BTreeMap::new();
		let mut count = 0;

		// if d is 0, then we roll look forward, if last, we reverse, else, spread from d
		if head > 1 && tail != LAST_BIN {
			while head != tail && head < NODE_BINS && count < BUCKET_SIZE
			{
				for n in &buckets[head as usize].nodes
				{
						if count < BUCKET_SIZE {
							count += 1;
							found.entry(Discovery::distance(target, &n)).or_insert_with(Vec::new).push(n);
						}
						else {
							break;
						}
				}
				if count < BUCKET_SIZE && tail != 0 {
					for n in &buckets[tail as usize].nodes {
						if count < BUCKET_SIZE {
							count += 1;
							found.entry(Discovery::distance(target, &n)).or_insert_with(Vec::new).push(n);
						}
						else {
							break;
						}
					}
				}

				head += 1;
				if tail > 0 {
					tail -= 1;
				}
			}
		}
		else if head < 2 {
			while head < NODE_BINS && count < BUCKET_SIZE {
				for n in &buckets[head as usize].nodes {
						if count < BUCKET_SIZE {
							count += 1;
							found.entry(Discovery::distance(target, &n)).or_insert_with(Vec::new).push(n);
						}
						else {
							break;
						}
				}
				head += 1;
			}
		}
		else {
			while tail > 0 && count < BUCKET_SIZE {
				for n in &buckets[tail as usize].nodes {
						if count < BUCKET_SIZE {
							count += 1;
							found.entry(Discovery::distance(target, &n)).or_insert_with(Vec::new).push(n);
						}
						else {
							break;
						}
				}
				tail -= 1;
			}
		}

		let mut ret:Vec<&NodeId> = Vec::new();
		for nodes in found.values() {
			for n in nodes {
				if ret.len() < BUCKET_SIZE as usize /* && n->endpoint && n->endpoint.isAllowed() */ {
					ret.push(n);
				}
			}
		}
		ret
	}
}
