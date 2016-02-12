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
use std::cell::{RefCell};
use std::ops::{DerefMut};
use std::mem;
use mio::*;
use mio::udp::*;
use hash::*;
use sha3::Hashable;
use crypto::*;
use rlp::*;
use network::node::*;
use network::error::NetworkError;
use io::StreamToken;

const ADDRESS_BYTES_SIZE: u32 = 32;							// Size of address type in bytes.
const ADDRESS_BITS: u32 = 8 * ADDRESS_BYTES_SIZE;			// Denoted by n in [Kademlia].
const NODE_BINS: u32 = ADDRESS_BITS - 1;					// Size of m_state (excludes root, which is us).
const DISCOVERY_MAX_STEPS: u16 = 8;							// Max iterations of discovery. (discover)
const BUCKET_SIZE: u32 = 16;		// Denoted by k in [Kademlia]. Number of nodes stored in each bucket.
const ALPHA: usize = 3;				// Denoted by \alpha in [Kademlia]. Number of concurrent FindNode requests.
const MAX_DATAGRAM_SIZE: usize = 1280;

const PACKET_PING: u8 = 1;
const PACKET_PONG: u8 = 2;
const PACKET_FIND_NODE: u8 = 3;
const PACKET_NEIGHBOURS: u8 = 4;

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

struct Datagramm {
	payload: Bytes,
	address: SocketAddr,
}

pub struct Discovery {
	id: NodeId,
	udp_socket: UdpSocket,
	token: StreamToken,
	discovery_round: u16,
	discovery_id: NodeId,
	discovery_nodes: HashSet<NodeId>,
	node_buckets: Vec<NodeBucket>,
	send_queue: VecDeque<Datagramm>
}

pub struct TableUpdates {
	pub added: HashMap<NodeId, Node>,
	pub removed: HashSet<NodeId>,
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
	pub fn new(id: &NodeId, address: &SocketAddr, token: StreamToken) -> Discovery {
		let socket = UdpSocket::bound(address).expect("Error binding UDP socket");
		Discovery {
			id: id.clone(),
			token: token,
			discovery_round: 0,
			discovery_id: NodeId::new(),
			discovery_nodes: HashSet::new(),
			node_buckets: (0..NODE_BINS).map(NodeBucket::new).collect(),
			udp_socket: socket,
			send_queue: VecDeque::new(),
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
			while head != tail && head < NODE_BINS && count < BUCKET_SIZE {
				for n in &buckets[head as usize].nodes {
					if count < BUCKET_SIZE {
						count += 1;
						found.entry(Discovery::distance(target, &n)).or_insert_with(Vec::new).push(n);
					}
					else { break }
				}
				if count < BUCKET_SIZE && tail != 0 {
					for n in &buckets[tail as usize].nodes {
						if count < BUCKET_SIZE {
							count += 1;
							found.entry(Discovery::distance(target, &n)).or_insert_with(Vec::new).push(n);
						}
						else { break }
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
					else { break }
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
					else { break }
				}
				tail -= 1;
			}
		}

		let mut ret:Vec<&NodeId> = Vec::new();
		for (_, nodes) in found {
			for n in nodes {
				if ret.len() < BUCKET_SIZE as usize /* && n->endpoint && n->endpoint.isAllowed() */ {
					ret.push(n);
				}
			}
		}
		ret
	}

	pub fn writable(&mut self) {
		if self.send_queue.is_empty() {
			return;
		}
		let data = self.send_queue.pop_front().unwrap();
		match self.udp_socket.send_to(&data.payload, &data.address) {
			Ok(Some(size)) if size == data.payload.len() => {
			},
			Ok(Some(size)) => {
				warn!("UDP sent incomplete datagramm");
			},
			Ok(None) => {
				self.send_queue.push_front(data);
			}
			Err(e) => {
				warn!("UDP sent error: {:?}", e);
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
		Ok(None)
	}

	fn on_pong(&mut self, rlp: &UntrustedRlp, node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, NetworkError> {
		Ok(None)
	}

	fn on_find_node(&mut self, rlp: &UntrustedRlp, node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, NetworkError> {
		Ok(None)
	}

	fn on_neighbours(&mut self, rlp: &UntrustedRlp, node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, NetworkError> {
		Ok(None)
	}

	pub fn round(&mut self) {
	}

	pub fn refresh(&mut self) {
	}

	pub fn register_socket<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), NetworkError> {
		event_loop.register(&self.udp_socket, Token(self.token), EventSet::all(), PollOpt::edge()).expect("Error registering UDP socket");
		Ok(())
	}

	pub fn update_registration<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), NetworkError> {
		let mut registration = EventSet::readable();
		if !self.send_queue.is_empty() {
			registration &= EventSet::writable();
		}
		event_loop.reregister(&self.udp_socket, Token(self.token), registration, PollOpt::edge()).expect("Error reregistering UDP socket");
		Ok(())
	}
}
