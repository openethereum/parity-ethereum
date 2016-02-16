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

use bytes::Bytes;
use std::net::SocketAddr;
use std::collections::{HashSet, HashMap, BTreeMap, VecDeque};
use std::mem;
use std::cmp;
use mio::*;
use mio::udp::*;
use sha3::*;
use time;
use hash::*;
use crypto::*;
use rlp::*;
use network::node_table::*;
use network::error::NetworkError;
use io::StreamToken;

use network::PROTOCOL_VERSION;

const ADDRESS_BYTES_SIZE: u32 = 32;							// Size of address type in bytes.
const ADDRESS_BITS: u32 = 8 * ADDRESS_BYTES_SIZE;			// Denoted by n in [Kademlia].
const NODE_BINS: u32 = ADDRESS_BITS - 1;					// Size of m_state (excludes root, which is us).
const DISCOVERY_MAX_STEPS: u16 = 8;							// Max iterations of discovery. (discover)
const BUCKET_SIZE: usize = 16;		// Denoted by k in [Kademlia]. Number of nodes stored in each bucket.
const ALPHA: usize = 3;				// Denoted by \alpha in [Kademlia]. Number of concurrent FindNode requests.
const MAX_DATAGRAM_SIZE: usize = 1280;

const PACKET_PING: u8 = 1;
const PACKET_PONG: u8 = 2;
const PACKET_FIND_NODE: u8 = 3;
const PACKET_NEIGHBOURS: u8 = 4;

const PING_TIMEOUT_MS: u64 = 300;

#[derive(Clone, Debug)]
pub struct NodeEntry {
	pub id: NodeId,
	pub endpoint: NodeEndpoint,
}

pub struct BucketEntry {
	pub address: NodeEntry,
	pub timeout: Option<u64>,
}

struct NodeBucket {
	nodes: VecDeque<BucketEntry>, //sorted by last active
}

impl NodeBucket {
	fn new() -> NodeBucket {
		NodeBucket {
			nodes: VecDeque::new()
		}
	}
}

struct Datagramm {
	payload: Bytes,
	address: SocketAddr,
}

pub struct Discovery {
	id: NodeId,
	secret: Secret,
	address: NodeEndpoint,
	udp_socket: UdpSocket,
	token: StreamToken,
	discovery_round: u16,
	discovery_id: NodeId,
	discovery_nodes: HashSet<NodeId>,
	node_buckets: Vec<NodeBucket>,
	send_queue: VecDeque<Datagramm>
}

pub struct TableUpdates {
	pub added: HashMap<NodeId, NodeEntry>,
	pub removed: HashSet<NodeId>,
}

impl Discovery {
	pub fn new(key: &KeyPair, address: NodeEndpoint, token: StreamToken) -> Discovery {
		let socket = UdpSocket::bound(&address.udp_address()).expect("Error binding UDP socket");
		Discovery {
			id: key.public().clone(),
			secret: key.secret().clone(),
			address: address,
			token: token,
			discovery_round: 0,
			discovery_id: NodeId::new(),
			discovery_nodes: HashSet::new(),
			node_buckets: (0..NODE_BINS).map(|_| NodeBucket::new()).collect(),
			udp_socket: socket,
			send_queue: VecDeque::new(),
		}
	}

	/// Add a new node to discovery table. Pings the node.
	pub fn add_node(&mut self, e: NodeEntry) { 
		let endpoint = e.endpoint.clone();
		self.update_node(e);
		self.ping(&endpoint);
	}

	/// Add a list of known nodes to the table.
	pub fn init_node_list(&mut self, mut nodes: Vec<NodeEntry>) { 
		for n in nodes.drain(..) {
			self.update_node(n);
		}
	}

	fn update_node(&mut self, e: NodeEntry) {
		trace!(target: "discovery", "Inserting {:?}", &e);
		let ping = {
			let mut bucket = self.node_buckets.get_mut(Discovery::distance(&self.id, &e.id) as usize).unwrap();
			let updated = if let Some(node) = bucket.nodes.iter_mut().find(|n| n.address.id == e.id) {
				node.address = e.clone();
				node.timeout = None;
				true
			} else { false };

			if !updated {
				bucket.nodes.push_front(BucketEntry { address: e, timeout: None });
			}

			if bucket.nodes.len() > BUCKET_SIZE {
				//ping least active node
				bucket.nodes.back_mut().unwrap().timeout = Some(time::precise_time_ns());
				Some(bucket.nodes.back().unwrap().address.endpoint.clone())
			} else { None }
		};
		if let Some(endpoint) = ping {
			self.ping(&endpoint);
		}
	}

	fn start(&mut self) {
		trace!(target: "discovery", "Starting discovery");
		self.discovery_round = 0;
		self.discovery_id.randomize(); //TODO: use cryptographic nonce
		self.discovery_nodes.clear();
	}

	fn discover(&mut self) {
		if self.discovery_round == DISCOVERY_MAX_STEPS {
			return;
		}
		trace!(target: "discovery", "Starting round {:?}", self.discovery_round);
		let mut tried_count = 0;
		{
			let nearest = Discovery::nearest_node_entries(&self.discovery_id, &self.node_buckets).into_iter();
			let nearest = nearest.filter(|x| !self.discovery_nodes.contains(&x.id)).take(ALPHA).collect::<Vec<_>>();
			for r in nearest {
				let rlp = encode(&(&[self.discovery_id.clone()][..]));
				self.send_packet(PACKET_FIND_NODE, &r.endpoint.udp_address(), &rlp);
				self.discovery_nodes.insert(r.id.clone());
				tried_count += 1;
				trace!(target: "discovery", "Sent FindNode to {:?}", &r.endpoint);
			}
		}

		if tried_count == 0 {
			trace!(target: "discovery", "Completing discovery");
			self.discovery_round = DISCOVERY_MAX_STEPS;
			self.discovery_nodes.clear();
			return;
		}
		self.discovery_round += 1;
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

	fn ping(&mut self, node: &NodeEndpoint) {
		let mut rlp = RlpStream::new_list(3);
		rlp.append(&PROTOCOL_VERSION);
		self.address.to_rlp_list(&mut rlp);
		node.to_rlp_list(&mut rlp);
		trace!(target: "discovery", "Sent Ping to {:?}", &node);
		self.send_packet(PACKET_PING, &node.udp_address(), &rlp.drain());
	}

	fn send_packet(&mut self, packet_id: u8, address: &SocketAddr, payload: &[u8]) {
		let mut rlp = RlpStream::new();
		rlp.append_raw(&[packet_id], 1);
		let source = Rlp::new(payload);
		rlp.begin_list(source.item_count() + 1);
		for i in 0 .. source.item_count() {
			rlp.append_raw(source.at(i).as_raw(), 1);
		}
		let timestamp = time::get_time().sec as u32 + 60;
		rlp.append(&timestamp);

		let bytes = rlp.drain();
		let hash = bytes.as_ref().sha3();
		let signature = match ec::sign(&self.secret, &hash) {
			Ok(s) => s,
			Err(_) => {
				warn!("Error signing UDP packet");
				return;
			}
		};
		let mut packet = Bytes::with_capacity(bytes.len() + 32 + 65);
		packet.extend(hash.iter());
		packet.extend(signature.iter());
		packet.extend(bytes.iter());
		let signed_hash = (&packet[32..]).sha3();
		packet[0..32].clone_from_slice(&signed_hash);
		self.send_to(packet, address.clone());
	}

	#[allow(map_clone)]
	fn nearest_node_entries(target: &NodeId, buckets: &[NodeBucket]) -> Vec<NodeEntry> {
		let mut found: BTreeMap<u32, Vec<&NodeEntry>> = BTreeMap::new();
		let mut count = 0;

		// Sort nodes by distance to target
		for bucket in buckets {
			for node in &bucket.nodes {
				let distance = Discovery::distance(target, &node.address.id); 
				found.entry(distance).or_insert_with(Vec::new).push(&node.address);
				if count == BUCKET_SIZE {
					// delete the most distant element
					let remove = {
						let (_, last) = found.iter_mut().next_back().unwrap();
						last.pop();
						last.is_empty()
					};
					if remove {
						found.remove(&distance);
					}
				}
				else {
					count += 1;
				}
			}
		}

		let mut ret:Vec<NodeEntry> = Vec::new();
		for nodes in found.values() {
			ret.extend(nodes.iter().map(|&n| n.clone()));
		}
		ret
	}

	pub fn writable(&mut self) {
		if self.send_queue.is_empty() {
			return;
		}
		while !self.send_queue.is_empty() {
			let data = self.send_queue.pop_front().unwrap();
			match self.udp_socket.send_to(&data.payload, &data.address) {
				Ok(Some(size)) if size == data.payload.len() => {
				},
				Ok(Some(_)) => {
					warn!("UDP sent incomplete datagramm");
				},
				Ok(None) => {
					self.send_queue.push_front(data);
					return;
				}
				Err(e) => {
					warn!("UDP send error: {:?}, address: {:?}", e, &data.address);
					return;
				}
			}
		}
	}

	fn send_to(&mut self, payload: Bytes, address: SocketAddr) {
		self.send_queue.push_back(Datagramm { payload: payload, address: address });
	}

	pub fn readable(&mut self) -> Option<TableUpdates> {
		let mut buf: [u8; MAX_DATAGRAM_SIZE] = unsafe { mem::uninitialized() };
		match self.udp_socket.recv_from(&mut buf) {
			Ok(Some((len, address))) => self.on_packet(&buf[0..len], address).unwrap_or_else(|e| {
				debug!("Error processing UDP packet: {:?}", e);
				None
			}),
			Ok(_) => None,
			Err(e) => { 
				warn!("Error reading UPD socket: {:?}", e);
				None
			}
		}
	}

	fn on_packet(&mut self, packet: &[u8], from: SocketAddr) -> Result<Option<TableUpdates>, NetworkError> {
		// validate packet
		if packet.len() < 32 + 65 + 4 + 1 {
			return Err(NetworkError::BadProtocol);
		}

		let hash_signed = (&packet[32..]).sha3();
		if hash_signed[..] != packet[0..32] {
			return Err(NetworkError::BadProtocol);
		}

		let signed = &packet[(32 + 65)..];
		let signature = Signature::from_slice(&packet[32..(32 + 65)]);
		let node_id = try!(ec::recover(&signature, &signed.sha3()));

		let packet_id = signed[0];
		let rlp = UntrustedRlp::new(&signed[1..]);
		match packet_id {
			PACKET_PING => self.on_ping(&rlp, &node_id, &from),
			PACKET_PONG => self.on_pong(&rlp, &node_id, &from),
			PACKET_FIND_NODE => self.on_find_node(&rlp, &node_id, &from),
			PACKET_NEIGHBOURS => self.on_neighbours(&rlp, &node_id, &from),
			_ => { 
				debug!("Unknown UDP packet: {}", packet_id);
				Ok(None)
			}
		}
	}

	fn on_ping(&mut self, rlp: &UntrustedRlp, node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, NetworkError> {
		trace!(target: "discovery", "Got Ping from {:?}", &from);
		let version: u32 = try!(rlp.val_at(0));
		if version != PROTOCOL_VERSION {
			debug!(target: "discovery", "Unexpected protocol version: {}", version);
			return Err(NetworkError::BadProtocol);
		}
		let source = try!(NodeEndpoint::from_rlp(&try!(rlp.at(1))));
		let dest = try!(NodeEndpoint::from_rlp(&try!(rlp.at(2))));
		let timestamp: u64 = try!(rlp.val_at(3));
		if timestamp < time::get_time().sec as u64{
			debug!(target: "discovery", "Expired ping");
			return Err(NetworkError::Expired);
		}
		let mut entry = NodeEntry { id: node.clone(), endpoint: source.clone() };
		if !entry.endpoint.is_valid() {
			debug!(target: "discovery", "Bad address: {:?}", entry);
			entry.endpoint.address = from.clone();
		}
		self.update_node(entry.clone());
		let hash = rlp.as_raw().sha3();
		let mut response = RlpStream::new_list(2);
		dest.to_rlp_list(&mut response);
		response.append(&hash);
		self.send_packet(PACKET_PONG, &entry.endpoint.udp_address(), &response.drain());
		
		let mut added_map = HashMap::new();
		added_map.insert(node.clone(), entry); 
		Ok(Some(TableUpdates { added: added_map, removed: HashSet::new() }))
	}

	fn on_pong(&mut self, rlp: &UntrustedRlp, node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, NetworkError> {
		trace!(target: "discovery", "Got Pong from {:?}", &from);
		// TODO: validate pong packet
		let dest = try!(NodeEndpoint::from_rlp(&try!(rlp.at(0))));
		let timestamp: u64 = try!(rlp.val_at(2));
		if timestamp < time::get_time().sec as u64 {
			return Err(NetworkError::Expired);
		}
		let mut entry = NodeEntry { id: node.clone(), endpoint: dest };
		if !entry.endpoint.is_valid() {
			debug!(target: "discovery", "Bad address: {:?}", entry);
			entry.endpoint.address = from.clone();
		}
		self.update_node(entry.clone());
		let mut added_map = HashMap::new();
		added_map.insert(node.clone(), entry); 
		Ok(Some(TableUpdates { added: added_map, removed: HashSet::new() }))
	}

	fn on_find_node(&mut self, rlp: &UntrustedRlp, _node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, NetworkError> {
		trace!(target: "discovery", "Got FindNode from {:?}", &from);
		let target: NodeId = try!(rlp.val_at(0));
		let timestamp: u64 = try!(rlp.val_at(1));
		if timestamp < time::get_time().sec as u64 {
			return Err(NetworkError::Expired);
		}

		let limit = (MAX_DATAGRAM_SIZE - 109) / 90;
		let nearest = Discovery::nearest_node_entries(&target, &self.node_buckets);
		if nearest.is_empty() {
			return Ok(None);
		}
		let mut rlp = RlpStream::new_list(1);
		rlp.begin_list(cmp::min(limit, nearest.len()));
		for n in 0 .. nearest.len() {
			rlp.begin_list(4);
			nearest[n].endpoint.to_rlp(&mut rlp);
			rlp.append(&nearest[n].id);
			if (n + 1) % limit == 0 || n == nearest.len() - 1 {
				self.send_packet(PACKET_NEIGHBOURS, &from, &rlp.drain());
				trace!(target: "discovery", "Sent {} Neighbours to {:?}", n, &from);
				rlp = RlpStream::new_list(1);
				rlp.begin_list(cmp::min(limit, nearest.len() - n));
			}
		}
		Ok(None)
	}

	fn on_neighbours(&mut self, rlp: &UntrustedRlp, _node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, NetworkError> {
		// TODO: validate packet
		let mut added = HashMap::new();
		trace!(target: "discovery", "Got {} Neighbours from {:?}", try!(rlp.at(0)).item_count(), &from);
		for r in try!(rlp.at(0)).iter() {
			let endpoint = try!(NodeEndpoint::from_rlp(&r));
			if !endpoint.is_valid() {
				debug!(target: "discovery", "Bad address: {:?}", endpoint);
				continue;
			}
			let node_id: NodeId = try!(r.val_at(3));
			if node_id == self.id {
				continue;
			}
			let entry = NodeEntry { id: node_id.clone(), endpoint: endpoint };
			added.insert(node_id, entry.clone());
			self.ping(&entry.endpoint);
			self.update_node(entry);
		}
		Ok(Some(TableUpdates { added: added, removed: HashSet::new() }))
	}

	fn check_expired(&mut self) -> HashSet<NodeId> {
		let now = time::precise_time_ns();
		let mut removed: HashSet<NodeId> = HashSet::new();
		for bucket in &mut self.node_buckets {
			bucket.nodes.retain(|node| {
				if let Some(timeout) = node.timeout {
					if now - timeout < PING_TIMEOUT_MS * 1000_0000 {
						true
					}
					else {
						trace!(target: "discovery", "Removed expired node {:?}", &node.address);
						removed.insert(node.address.id.clone());
						false
					}
				} else { true }
			});
		}
		removed
	}

	pub fn round(&mut self) -> Option<TableUpdates> {
		let removed = self.check_expired();
		self.discover();
		if !removed.is_empty() { 
			Some(TableUpdates { added: HashMap::new(), removed: removed }) 
		} else { None }
	}

	pub fn refresh(&mut self) {
		self.start();
	}

	pub fn register_socket<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), NetworkError> {
		event_loop.register(&self.udp_socket, Token(self.token), EventSet::all(), PollOpt::edge()).expect("Error registering UDP socket");
		Ok(())
	}

	pub fn update_registration<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), NetworkError> {
		let mut registration = EventSet::readable();
		if !self.send_queue.is_empty() {
			registration = registration | EventSet::writable();
		}
		event_loop.reregister(&self.udp_socket, Token(self.token), registration, PollOpt::edge()).expect("Error reregistering UDP socket");
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hash::*;
	use std::net::*;
	use network::node_table::*;
	use crypto::KeyPair;
	use std::str::FromStr;

	#[test]
	fn discovery() {
		let key1 = KeyPair::create().unwrap();
		let key2 = KeyPair::create().unwrap();
		let ep1 = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40444").unwrap(), udp_port: 40444 };
		let ep2 = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40445").unwrap(), udp_port: 40445 };
		let mut discovery1 = Discovery::new(&key1, ep1.clone(), 0);
		let mut discovery2 = Discovery::new(&key2, ep2.clone(), 0);

		let node1 = Node::from_str("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@127.0.0.1:7770").unwrap();
		let node2 = Node::from_str("enode://b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@127.0.0.1:7771").unwrap();
		discovery1.add_node(NodeEntry { id: node1.id.clone(), endpoint: node1. endpoint.clone() });
		discovery1.add_node(NodeEntry { id: node2.id.clone(), endpoint: node2. endpoint.clone() });

		discovery2.add_node(NodeEntry { id: key1.public().clone(), endpoint: ep1.clone() });
		discovery2.refresh();

		for _ in 0 .. 10 {
			while !discovery1.send_queue.is_empty() {
				let datagramm = discovery1.send_queue.pop_front().unwrap();
				if datagramm.address == ep2.address {
					discovery2.on_packet(&datagramm.payload, ep1.address.clone()).ok();
				}
			}
			while !discovery2.send_queue.is_empty() {
				let datagramm = discovery2.send_queue.pop_front().unwrap();
				if datagramm.address == ep1.address {
					discovery1.on_packet(&datagramm.payload, ep2.address.clone()).ok();
				}
			}
			discovery2.round();
		}
		assert_eq!(Discovery::nearest_node_entries(&NodeId::new(), &discovery2.node_buckets).len(), 3)
	}
}
