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

use ethcore_bytes::Bytes;
use std::net::SocketAddr;
use std::collections::{HashSet, HashMap, VecDeque};
use std::mem;
use std::default::Default;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use mio::*;
use mio::deprecated::{Handler, EventLoop};
use mio::udp::*;
use hash::keccak;
use ethereum_types::{H256, H520};
use rlp::{Rlp, RlpStream, encode_list};
use node_table::*;
use network::{Error, ErrorKind};
use io::{StreamToken, IoContext};
use ethkey::{Secret, KeyPair, sign, recover};
use network::IpFilter;

use PROTOCOL_VERSION;

const ADDRESS_BYTES_SIZE: usize = 32;						// Size of address type in bytes.
const ADDRESS_BITS: usize = 8 * ADDRESS_BYTES_SIZE;			// Denoted by n in [Kademlia].
const DISCOVERY_MAX_STEPS: u16 = 8;							// Max iterations of discovery. (discover)
const BUCKET_SIZE: usize = 16;		// Denoted by k in [Kademlia]. Number of nodes stored in each bucket.
const ALPHA: usize = 3;				// Denoted by \alpha in [Kademlia]. Number of concurrent FindNode requests.
const MAX_DATAGRAM_SIZE: usize = 1280;

const PACKET_PING: u8 = 1;
const PACKET_PONG: u8 = 2;
const PACKET_FIND_NODE: u8 = 3;
const PACKET_NEIGHBOURS: u8 = 4;

const PING_TIMEOUT: Duration = Duration::from_millis(300);
const MAX_NODES_PING: usize = 32; // Max nodes to add/ping at once

#[derive(Clone, Debug)]
pub struct NodeEntry {
	pub id: NodeId,
	pub endpoint: NodeEndpoint,
}

pub struct BucketEntry {
	pub address: NodeEntry,
	pub id_hash: H256,
	pub timeout: Option<Instant>,
}

pub struct NodeBucket {
	nodes: VecDeque<BucketEntry>, //sorted by last active
}

impl Default for NodeBucket {
	fn default() -> Self {
		NodeBucket::new()
	}
}

impl NodeBucket {
	fn new() -> Self {
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
	id_hash: H256,
	secret: Secret,
	public_endpoint: NodeEndpoint,
	udp_socket: UdpSocket,
	token: StreamToken,
	discovery_round: u16,
	discovery_id: NodeId,
	discovery_nodes: HashSet<NodeId>,
	node_buckets: Vec<NodeBucket>,
	send_queue: VecDeque<Datagramm>,
	check_timestamps: bool,
	adding_nodes: Vec<NodeEntry>,
	ip_filter: IpFilter,
}

pub struct TableUpdates {
	pub added: HashMap<NodeId, NodeEntry>,
	pub removed: HashSet<NodeId>,
}

impl Discovery {
	pub fn new(key: &KeyPair, listen: SocketAddr, public: NodeEndpoint, token: StreamToken, ip_filter: IpFilter) -> Discovery {
		let socket = UdpSocket::bind(&listen).expect("Error binding UDP socket");
		Discovery {
			id: key.public().clone(),
			id_hash: keccak(key.public()),
			secret: key.secret().clone(),
			public_endpoint: public,
			token: token,
			discovery_round: 0,
			discovery_id: NodeId::new(),
			discovery_nodes: HashSet::new(),
			node_buckets: (0..ADDRESS_BITS).map(|_| NodeBucket::new()).collect(),
			udp_socket: socket,
			send_queue: VecDeque::new(),
			check_timestamps: true,
			adding_nodes: Vec::new(),
			ip_filter: ip_filter,
		}
	}

	/// Add a new node to discovery table. Pings the node.
	pub fn add_node(&mut self, e: NodeEntry) {
		if self.is_allowed(&e) {
			let endpoint = e.endpoint.clone();
			self.update_node(e);
			self.ping(&endpoint);
		}
	}

	/// Add a list of nodes. Pings a few nodes each round
	pub fn add_node_list(&mut self, nodes: Vec<NodeEntry>) {
		self.adding_nodes = nodes;
		self.update_new_nodes();
	}

	/// Add a list of known nodes to the table.
	pub fn init_node_list(&mut self, mut nodes: Vec<NodeEntry>) {
		for n in nodes.drain(..) {
			if self.is_allowed(&n) {
				self.update_node(n);
			}
		}
	}

	fn update_node(&mut self, e: NodeEntry) {
		trace!(target: "discovery", "Inserting {:?}", &e);
		let id_hash = keccak(e.id);
		let dist = match Discovery::distance(&self.id_hash, &id_hash) {
			Some(dist) => dist,
			None => {
				warn!(target: "discovery", "Attempted to update own entry: {:?}", e);
				return;
			}
		};

		let ping = {
			let bucket = &mut self.node_buckets[dist];
			let updated = if let Some(node) = bucket.nodes.iter_mut().find(|n| n.address.id == e.id) {
				node.address = e.clone();
				node.timeout = None;
				true
			} else { false };

			if !updated {
				bucket.nodes.push_front(BucketEntry { address: e, timeout: None, id_hash: id_hash, });
			}

			if bucket.nodes.len() > BUCKET_SIZE {
				//ping least active node
				let last = bucket.nodes.back_mut().expect("Last item is always present when len() > 0");
				last.timeout = Some(Instant::now());
				Some(last.address.endpoint.clone())
			} else { None }
		};
		if let Some(endpoint) = ping {
			self.ping(&endpoint);
		}
	}

	/// Removes the timeout of a given NodeId if it can be found in one of the discovery buckets
	fn clear_ping(&mut self, id: &NodeId) {
		let dist = match Discovery::distance(&self.id_hash, &keccak(id)) {
			Some(dist) => dist,
			None => {
				warn!(target: "discovery", "Received ping from self");
				return
			}
		};

		let bucket = &mut self.node_buckets[dist];
		if let Some(node) = bucket.nodes.iter_mut().find(|n| &n.address.id == id) {
			node.timeout = None;
		}
	}

	/// Starts the discovery process at round 0
	fn start(&mut self) {
		trace!(target: "discovery", "Starting discovery");
		self.discovery_round = 0;
		self.discovery_id.randomize(); //TODO: use cryptographic nonce
		self.discovery_nodes.clear();
	}

	fn update_new_nodes(&mut self) {
		let mut count = 0usize;
		while !self.adding_nodes.is_empty() && count < MAX_NODES_PING {
			let node = self.adding_nodes.pop().expect("pop is always Some if not empty; qed");
			self.add_node(node);
			count += 1;
		}
	}

	fn discover(&mut self) {
		self.update_new_nodes();
		if self.discovery_round == DISCOVERY_MAX_STEPS {
			return;
		}
		trace!(target: "discovery", "Starting round {:?}", self.discovery_round);
		let mut tried_count = 0;
		{
			let nearest = self.nearest_node_entries(&self.discovery_id).into_iter();
			let nearest = nearest.filter(|x| !self.discovery_nodes.contains(&x.id)).take(ALPHA).collect::<Vec<_>>();
			for r in nearest {
				let rlp = encode_list(&(&[self.discovery_id.clone()][..]));
				self.send_packet(PACKET_FIND_NODE, &r.endpoint.udp_address(), &rlp)
					.unwrap_or_else(|e| warn!("Error sending node discovery packet for {:?}: {:?}", &r.endpoint, e));
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

	/// The base 2 log of the distance between a and b using the XOR metric.
	fn distance(a: &H256, b: &H256) -> Option<usize> {
		for i in (0..ADDRESS_BYTES_SIZE).rev() {
			let byte_index = ADDRESS_BYTES_SIZE - i - 1;
			let d: u8 = a[byte_index] ^ b[byte_index];
			if d != 0 {
				let high_bit_index = 7 - d.leading_zeros() as usize;
				return Some(i * 8 + high_bit_index);
			}
		}
		None // a and b are equal, so log distance is -inf
	}

	fn ping(&mut self, node: &NodeEndpoint) {
		let mut rlp = RlpStream::new_list(3);
		rlp.append(&PROTOCOL_VERSION);
		self.public_endpoint.to_rlp_list(&mut rlp);
		node.to_rlp_list(&mut rlp);
		trace!(target: "discovery", "Sent Ping to {:?}", &node);
		self.send_packet(PACKET_PING, &node.udp_address(), &rlp.drain())
			.unwrap_or_else(|e| warn!("Error sending Ping packet: {:?}", e))
	}

	fn send_packet(&mut self, packet_id: u8, address: &SocketAddr, payload: &[u8]) -> Result<(), Error> {
		let mut rlp = RlpStream::new();
		rlp.append_raw(&[packet_id], 1);
		let source = Rlp::new(payload);
		rlp.begin_list(source.item_count()? + 1);
		for i in 0 .. source.item_count()? {
			rlp.append_raw(source.at(i)?.as_raw(), 1);
		}
		let timestamp = 60 + SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as u32;
		rlp.append(&timestamp);

		let bytes = rlp.drain();
		let hash = keccak(bytes.as_ref());
		let signature = match sign(&self.secret, &hash) {
			Ok(s) => s,
			Err(e) => {
				warn!("Error signing UDP packet");
				return Err(Error::from(e));
			}
		};
		let mut packet = Bytes::with_capacity(bytes.len() + 32 + 65);
		packet.extend(hash.iter());
		packet.extend(signature.iter());
		packet.extend(bytes.iter());
		let signed_hash = keccak(&packet[32..]);
		packet[0..32].clone_from_slice(&signed_hash);
		self.send_to(packet, address.clone());
		Ok(())
	}

	fn nearest_node_entries(&self, target: &NodeId) -> Vec<NodeEntry> {
		let target_hash = keccak(target);
		let target_distance = self.id_hash ^ target_hash;

		let mut ret = Vec::<NodeEntry>::with_capacity(BUCKET_SIZE);

		// Sort bucket entries by distance to target and append to end of result vector.
		let append_bucket = |results: &mut Vec<NodeEntry>, bucket: &NodeBucket| -> bool {
			let mut sorted_entries: Vec<&BucketEntry> = bucket.nodes.iter().collect();
			sorted_entries.sort_unstable_by_key(|entry| entry.id_hash ^ target_hash);

			let remaining_capacity = results.capacity() - results.len();
			let to_append = if remaining_capacity < sorted_entries.len() {
				&sorted_entries[0..remaining_capacity]
			} else {
				&sorted_entries
			};
			for entry in to_append.iter() {
				results.push(entry.address.clone());
			}
			results.len() == results.capacity()
		};

		// This algorithm leverages the structure of the routing table to efficiently find the
		// nearest entries to a target hash. First, we compute the XOR distance from this node to
		// the target. On a first pass, we iterate from the MSB of the distance, stopping at any
		// buckets where the distance bit is set, and skipping the buckets where it is unset. These
		// must be in order the nearest to the target. On a second pass, we traverse from LSB to
		// MSB, appending the buckets skipped on the first pass. The reason this works is that all
		// entries in bucket i have a common prefix of length exactly 32 - i - 1 with the ID of this
		// node.

		for i in 0..ADDRESS_BITS {
			if ((target_distance[i / 8] << (i % 8)) & 0x80) != 0 {
				let bucket = &self.node_buckets[ADDRESS_BITS - i - 1];
				if !bucket.nodes.is_empty() && append_bucket(&mut ret, bucket) {
					return ret;
				}
			}
		}
		for i in (0..ADDRESS_BITS).rev() {
			if ((target_distance[i / 8] << (i % 8)) & 0x80) == 0 {
				let bucket = &self.node_buckets[ADDRESS_BITS - i - 1];
				if !bucket.nodes.is_empty() && append_bucket(&mut ret, bucket) {
					return ret;
				}
			}
		}
		ret
	}

	pub fn writable<Message>(&mut self, io: &IoContext<Message>) where Message: Send + Sync + Clone {
		while let Some(data) = self.send_queue.pop_front() {
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
					debug!("UDP send error: {:?}, address: {:?}", e, &data.address);
					return;
				}
			}
		}
		io.update_registration(self.token).unwrap_or_else(|e| debug!("Error updating discovery registration: {:?}", e));
	}

	fn send_to(&mut self, payload: Bytes, address: SocketAddr) {
		self.send_queue.push_back(Datagramm { payload: payload, address: address });
	}

	pub fn readable<Message>(&mut self, io: &IoContext<Message>) -> Option<TableUpdates> where Message: Send + Sync + Clone {
		let mut buf: [u8; MAX_DATAGRAM_SIZE] = unsafe { mem::uninitialized() };
		let writable = !self.send_queue.is_empty();
		let res = match self.udp_socket.recv_from(&mut buf) {
			Ok(Some((len, address))) => self.on_packet(&buf[0..len], address).unwrap_or_else(|e| {
				debug!("Error processing UDP packet: {:?}", e);
				None
			}),
			Ok(_) => None,
			Err(e) => {
				debug!("Error reading UPD socket: {:?}", e);
				None
			}
		};
		let new_writable = !self.send_queue.is_empty();
		if writable != new_writable {
			io.update_registration(self.token).unwrap_or_else(|e| debug!("Error updating discovery registration: {:?}", e));
		}
		res
	}

	fn on_packet(&mut self, packet: &[u8], from: SocketAddr) -> Result<Option<TableUpdates>, Error> {
		// validate packet
		if packet.len() < 32 + 65 + 4 + 1 {
			return Err(ErrorKind::BadProtocol.into());
		}

		let hash_signed = keccak(&packet[32..]);
		if hash_signed[..] != packet[0..32] {
			return Err(ErrorKind::BadProtocol.into());
		}

		let signed = &packet[(32 + 65)..];
		let signature = H520::from_slice(&packet[32..(32 + 65)]);
		let node_id = recover(&signature.into(), &keccak(signed))?;

		let packet_id = signed[0];
		let rlp = Rlp::new(&signed[1..]);
		match packet_id {
			PACKET_PING => self.on_ping(&rlp, &node_id, &from, &hash_signed),
			PACKET_PONG => self.on_pong(&rlp, &node_id, &from),
			PACKET_FIND_NODE => self.on_find_node(&rlp, &node_id, &from),
			PACKET_NEIGHBOURS => self.on_neighbours(&rlp, &node_id, &from),
			_ => {
				debug!("Unknown UDP packet: {}", packet_id);
				Ok(None)
			}
		}
	}

	/// Validate that given timestamp is in within one second of now or in the future
	fn check_timestamp(&self, timestamp: u64) -> Result<(), Error> {
		let secs_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
		if self.check_timestamps && timestamp < secs_since_epoch {
			debug!(target: "discovery", "Expired packet");
			return Err(ErrorKind::Expired.into());
		}
		Ok(())
	}

	fn is_allowed(&self, entry: &NodeEntry) -> bool {
		entry.endpoint.is_allowed(&self.ip_filter) && entry.id != self.id
	}

	fn on_ping(&mut self, rlp: &Rlp, node: &NodeId, from: &SocketAddr, echo_hash: &[u8]) -> Result<Option<TableUpdates>, Error> {
		trace!(target: "discovery", "Got Ping from {:?}", &from);
		let source = NodeEndpoint::from_rlp(&rlp.at(1)?)?;
		let dest = NodeEndpoint::from_rlp(&rlp.at(2)?)?;
		let timestamp: u64 = rlp.val_at(3)?;
		self.check_timestamp(timestamp)?;
		let mut added_map = HashMap::new();
		let entry = NodeEntry { id: node.clone(), endpoint: source.clone() };
		if !entry.endpoint.is_valid() {
			debug!(target: "discovery", "Got bad address: {:?}", entry);
		} else if !self.is_allowed(&entry) {
			debug!(target: "discovery", "Address not allowed: {:?}", entry);
		} else {
			self.update_node(entry.clone());
			added_map.insert(node.clone(), entry);
		}
		let mut response = RlpStream::new_list(2);
		dest.to_rlp_list(&mut response);
		response.append(&echo_hash);
		self.send_packet(PACKET_PONG, from, &response.drain())?;

		Ok(Some(TableUpdates { added: added_map, removed: HashSet::new() }))
	}

	fn on_pong(&mut self, rlp: &Rlp, node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, Error> {
		trace!(target: "discovery", "Got Pong from {:?}", &from);
		// TODO: validate pong packet in rlp.val_at(1)
		let dest = NodeEndpoint::from_rlp(&rlp.at(0)?)?;
		let timestamp: u64 = rlp.val_at(2)?;
		self.check_timestamp(timestamp)?;
		let mut entry = NodeEntry { id: node.clone(), endpoint: dest };
		if !entry.endpoint.is_valid() {
			debug!(target: "discovery", "Bad address: {:?}", entry);
			entry.endpoint.address = from.clone();
		}
		self.clear_ping(node);
		Ok(None)
	}

	fn on_find_node(&mut self, rlp: &Rlp, _node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, Error> {
		trace!(target: "discovery", "Got FindNode from {:?}", &from);
		let target: NodeId = rlp.val_at(0)?;
		let timestamp: u64 = rlp.val_at(1)?;
		self.check_timestamp(timestamp)?;
		let nearest = self.nearest_node_entries(&target);
		if nearest.is_empty() {
			return Ok(None);
		}
		let mut packets = Discovery::prepare_neighbours_packets(&nearest);
		for p in packets.drain(..) {
			self.send_packet(PACKET_NEIGHBOURS, from, &p)?;
		}
		trace!(target: "discovery", "Sent {} Neighbours to {:?}", nearest.len(), &from);
		Ok(None)
	}

	fn prepare_neighbours_packets(nearest: &[NodeEntry]) -> Vec<Bytes> {
		let limit = (MAX_DATAGRAM_SIZE - 109) / 90;
		let chunks = nearest.chunks(limit);
		let packets = chunks.map(|c| {
			let mut rlp = RlpStream::new_list(1);
			rlp.begin_list(c.len());
			for n in 0 .. c.len() {
				rlp.begin_list(4);
				c[n].endpoint.to_rlp(&mut rlp);
				rlp.append(&c[n].id);
			}
			rlp.out()
		});
		packets.collect()
	}

	fn on_neighbours(&mut self, rlp: &Rlp, _node: &NodeId, from: &SocketAddr) -> Result<Option<TableUpdates>, Error> {
		// TODO: validate packet
		let mut added = HashMap::new();
		trace!(target: "discovery", "Got {} Neighbours from {:?}", rlp.at(0)?.item_count()?, &from);
		for r in rlp.at(0)?.iter() {
			let endpoint = NodeEndpoint::from_rlp(&r)?;
			if !endpoint.is_valid() {
				debug!(target: "discovery", "Bad address: {:?}", endpoint);
				continue;
			}
			let node_id: NodeId = r.val_at(3)?;
			if node_id == self.id {
				continue;
			}
			let entry = NodeEntry { id: node_id.clone(), endpoint: endpoint };
			if !self.is_allowed(&entry) {
				debug!(target: "discovery", "Address not allowed: {:?}", entry);
				continue;
			}
			added.insert(node_id, entry.clone());
			self.ping(&entry.endpoint);
			self.update_node(entry);
		}
		Ok(Some(TableUpdates { added: added, removed: HashSet::new() }))
	}

	fn check_expired(&mut self, force: bool) -> HashSet<NodeId> {
		let now = Instant::now();
		let mut removed: HashSet<NodeId> = HashSet::new();
		for bucket in &mut self.node_buckets {
			bucket.nodes.retain(|node| {
				if let Some(timeout) = node.timeout {
					if !force && now.duration_since(timeout) < PING_TIMEOUT {
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
		let removed = self.check_expired(false);
		self.discover();
		if !removed.is_empty() {
			Some(TableUpdates { added: HashMap::new(), removed: removed })
		} else { None }
	}

	pub fn refresh(&mut self) {
		self.start();
	}

	pub fn register_socket<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), Error> {
		event_loop.register(&self.udp_socket, Token(self.token), Ready::all(), PollOpt::edge()).expect("Error registering UDP socket");
		Ok(())
	}

	pub fn update_registration<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), Error> {
		let registration = if !self.send_queue.is_empty() {
			Ready::readable() | Ready::writable()
		} else {
			Ready::readable()
		};
		event_loop.reregister(&self.udp_socket, Token(self.token), registration, PollOpt::edge()).expect("Error reregistering UDP socket");
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::net::{SocketAddr};
	use node_table::{Node, NodeId, NodeEndpoint};

	use std::str::FromStr;
	use rustc_hex::FromHex;
	use ethkey::{Random, Generator};

	#[test]
	fn find_node() {
		let mut nearest = Vec::new();
		let node = Node::from_str("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@127.0.0.1:7770").unwrap();
		for _ in 0..1000 {
			nearest.push( NodeEntry { id: node.id.clone(), endpoint: node.endpoint.clone() });
		}

		let packets = Discovery::prepare_neighbours_packets(&nearest);
		assert_eq!(packets.len(), 77);
		for p in &packets[0..76] {
			assert!(p.len() > 1280/2);
			assert!(p.len() <= 1280);
		}
		assert!(packets.last().unwrap().len() > 0);
	}

	#[test]
	fn discovery() {
		let key1 = Random.generate().unwrap();
		let key2 = Random.generate().unwrap();
		let ep1 = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40444").unwrap(), udp_port: 40444 };
		let ep2 = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40445").unwrap(), udp_port: 40445 };
		let mut discovery1 = Discovery::new(&key1, ep1.address.clone(), ep1.clone(), 0, IpFilter::default());
		let mut discovery2 = Discovery::new(&key2, ep2.address.clone(), ep2.clone(), 0, IpFilter::default());

		let node1 = Node::from_str("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@127.0.0.1:7770").unwrap();
		let node2 = Node::from_str("enode://b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@127.0.0.1:7771").unwrap();
		discovery1.add_node(NodeEntry { id: node1.id.clone(), endpoint: node1.endpoint.clone() });
		discovery1.add_node(NodeEntry { id: node2.id.clone(), endpoint: node2.endpoint.clone() });

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
		assert_eq!(discovery2.nearest_node_entries(&NodeId::new()).len(), 3)
	}

	#[test]
	fn removes_expired() {
		let key = Random.generate().unwrap();
		let ep = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40446").unwrap(), udp_port: 40447 };
		let mut discovery = Discovery::new(&key, ep.address.clone(), ep.clone(), 0, IpFilter::default());
		for _ in 0..1200 {
			discovery.add_node(NodeEntry { id: NodeId::random(), endpoint: ep.clone() });
		}
		assert!(discovery.nearest_node_entries(&NodeId::new()).len() <= 16);
		let removed = discovery.check_expired(true).len();
		assert!(removed > 0);
	}

	#[test]
	fn find_nearest_saturated() {
		use super::*;

		let key = Random.generate().unwrap();
		let ep = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40447").unwrap(), udp_port: 40447 };
		let mut discovery = Discovery::new(&key, ep.address.clone(), ep.clone(), 0, IpFilter::default());

		for _ in 0..(16 + 10) {
			discovery.node_buckets[0].nodes.push_back(BucketEntry {
				address: NodeEntry { id: NodeId::new(), endpoint: ep.clone() },
				timeout: None,
				id_hash: keccak(NodeId::new()),
			});
		}
		let nearest = discovery.nearest_node_entries(&NodeId::new());
		assert_eq!(nearest.len(), 16)
	}

	#[test]
	fn routing_table_insertions_lookups() {
		use super::*;
		let ep = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40448").unwrap(), udp_port: 40447 };
		let node_ids_hex: [&str; 32] = [
			"22536fa57acc12c4993295cbc26fef4550513496712b301ad2283d356c8108521244a362e64e6d907a0d0b4e65526699c5ae3cfebfc680505fe3b33d50672835",
			"22c482f42401546f8dd7ed6b1c0cad976da6630730f1116614579ccb084791a528ff2676bfe94434de80e5d7e479f1ea1d7737077da3bd5e69a0f3e5bf596091",
			"234c73e3a8f6835a7f9a9d2a896bff4908d66d21d5433a2c37d94f1fa9a6ca17d02388f31013ff87e3ad86506e76bd1006b9cac3815974a2b47c8d4f2124697e",
			"2a5aaf4e2046c521e890dc82313c6151a55078f045a7e3d259f168238d029271cdd9a0943468d45c1e36a34a8a6d4de4b0262e48d3c8cfdd4c2aab5df42926b9",
			"341d8c94d9670461186cfc1f66d4246cb12384940e9f621ec8d6c216b5d037cde5f7a41b70474ca36ced4a4f2fe91c9dc5a24a128414672661f78e8611d54bfd",
			"3d9fd01851f3ae1bfd06b48e89738f29f9a2b4dce3ab7864df4fccca55d1ac88044956ba47d0c4cb44a19924626a3a3aa5a4de8958365cb7385111ce7b929200",
			"406d5507a7fbc194a495800ae8cf408093336febc24d03d6c63756f522274ab02146ceb1b0213291a9a1544680503837519f88f1e8677d921de62c82935b4e6c",
			"4c537f00805f320616ee49c7bc36e1d7e52a04a782b0cc00fd3d6b77200b027cef5f875ed38f1167fef4b02d7bd49a661812301d9d680bb62297131204c035f9",
			"4fc8e3fdbdd7acad82b283ac52c121b805f3b15ffcaa6b2ca67b9e375aa88e978951ffa3d03ee13be99f0ee987db0bbfc6a7ca02b175e9123d79826025b4089d",
			"55b5042a6910bc908a0520966e8cbcc92ac299bdb7efbfbcf703df1506fa0f9b09c5eeb930080de848d2864cca71f885942852c51233db0ee46fe0447306d61f",
			"5d24f28b350c4c37fc4dad7f418e029992c9e4ac356bb3d9a1356ba1076339863c05044d7ceba233c65779401f8a3b38fe67b6a592c1be4834dc869f7bb932eb",
			"5f6edaf2f2ae3003f4b4ff90b8e71a717c832c71a634d96e77fe046f9a88adc8de5718ff3c47659aea4cead5376df5b731e1b6530e6b0999f56ad75d4dabd3f6",
			"6214c04211efe91abd23d65e2dc8e711b06d4fb13dcfd65b691dc51f58455b2145f9b38f523b72a45a12705a28d389308a34455720d774c9b805326df42b5a63",
			"69df92573ddbbce88b72a930843dbb70728b2a020e0cc4e8ba805dcf7f19297bfc5def4ca447e9e6ec66971be1815b8f49042720431f698b6a87a185d94fa6c8",
			"72ffc23de007cf8b6f4a117f7427b532d05861c314344ffa265175f57ee45dae041a710a4dc74124dba1dabdc0f52dfd21e3154d1d4285aab529810c6161d623",
			"80b567f279a9512f3a66ebd8f87a93acd4d50bf66f5eff6d04039c1f5838e37021e981539659b33e0644b243fc9671209a80cbef40d1bcf7c7117d353cb45532",
			"9009dc9e3bf50595f84271f46d4c7a5ad6971f7d2ffce1905bfc40a407d34fc5e2dcebd92746eadcd2c5fa4d5aaccb0e01b542d506b361851df3f19e6bc629a3",
			"95264f56e091efeba911003fd01eeb2c81f6fc4bb7b10c92e4c7bfaf460b7246d232e61ad8a223d74870981a84e15b2d5134c25d931cb860c6912b20a2d3ac01",
			"96013a472a9f7ff9c5c76b5ca958f14ee510d826703aa41d4c88eac51d30d14229b9f19f6e0469c37aaa6d2136a978a4aaa38ca766f48e53e569f84e44252962",
			"a513c988cf8480ad2992caa64e3fa059ce07efda260dfeefed78e1d41ea3f97844603b8a9737eb633086fd9ac2f201200cb656cda8a91bf6cc500d6039db6f53",
			"ab3311f38e3641c8b3b1fd36dd7f94b148166e267258e840d29d1859537c74f202bd3342359b3623f96c23fa662d1b65182a898bf20343744b37cb265182e500",
			"ac8f41dbd637891a08c9cf715c23577bdd431ba40231682a5a9ba7fd6cb6d66c04f63d6d65c7d9f8737e641e05fdbeede57138a174f0d55e7835575dd6cddd98",
			"accdad251888d53e4e18efee1e0d749d050216b14896efb657e9c7b1b78dab82a5b6fb3234017aa19a2f50475d73960f352d308b2e0e841cbebaf418362a4f21",
			"b138622208f74d2b8e8fc10bcd4cf3302685cd77d339280a939474b92be8b93e441c50709e25c82cc88a2a4207e9f2938912d60600226efe322b43c6ef5e7aef",
			"b4f64e1fa6a5cd6198b2515bde63fbdabaf7e7a31dbaf5369babbda4b8cd0bf5025ac4b7d2d6e6e3bc76c890df585d28d4815e464c8792ef677df9206864a12b",
			"c1136e08a27c93812ae2dd47201d9e81c82d1995001b88dba9eec700e1d3385dfaf7ae834226c3c90a138f1808cd10b5502f49ee774a2bc707f34bd7d160b7bd",
			"c203ae9b5d1953b0ac462e66338800ec26982e2af54bd444fc8978973191633d4f483e31b28233c07bb99f34d57c680fa5f8e093e64f13b235005b7ab6e2d594",
			"c2e1067c58a9948e773e0a3637d946e26d95762f89ec9d35e2ad84f770309d94168d4e112c78d62b60efc6216bc5d31475f24307b1b8e0fa8dcbb18a10cb85f5",
			"d60ecb1a89e0d5aeff14c9a95da9f5492eb15871c53563b86b7c5ddf0da74b4c29e682fdd22aae2290e0b16ef4b6d707ef55396ca98f755c95b689cf65ce5f80",
			"df5ad4ea6242929df86f2162d1cc62b0e0a6f0a03428a39dea98f6a689335b5ceaf1f0696c17b717b141aeb45a29108d95c3a7d2d1d0bb3441219504ae672917",
			"e1268f5dd9552a11989df9d4953bb388e7466711b2bd9882a3ed4d0767a21f046c53c20f9a18d66bae1d6a5544492857ddecb0b5b4818bd4557be252ddd66c71",
			"e626019dc0b50b9e254461f19d29e69a4669c5256134a6352c6c30d3bc55d201a5b43fc2e006556cfaf29765b683e807e03093798942826244e4ee9e47c75d3f",
		];
		let node_entries = node_ids_hex.iter()
			.map(|node_id_hex| NodeId::from_str(node_id_hex).unwrap())
			.map(|node_id| NodeEntry { id: node_id, endpoint: ep.clone() })
			.collect::<Vec<_>>();

		let secret_hex = "6c71d1b8930d29e6371be1081f2c909c64b46440a1716314c3c9df995cb3aed1";
		let key = Secret::from_str(secret_hex)
			.and_then(|secret| KeyPair::from_secret(secret))
			.unwrap();
		let mut discovery = Discovery::new(&key, ep.address.clone(), ep.clone(), 0, IpFilter::default());

		node_entries.iter().for_each(|entry| discovery.update_node(entry.clone()));

		let expected_bucket_sizes = vec![
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 7, 8, 12
		];
		let actual_bucket_sizes = discovery.node_buckets.iter()
			.map(|ref bucket| bucket.nodes.len())
			.collect::<Vec<_>>();
		assert_eq!(actual_bucket_sizes, expected_bucket_sizes);

		for entry in &node_entries {
			let nearest = discovery.nearest_node_entries(&entry.id);
			assert_eq!(nearest.len(), 16);
			assert_eq!(nearest[0].id, entry.id);

			let mut expected_ids: Vec<NodeId> = node_entries.iter().map(|entry| entry.id).collect();
			expected_ids.sort_unstable_by_key(|id| keccak(id) ^ keccak(entry.id));
			expected_ids.resize(BUCKET_SIZE, NodeId::default());

			let actual_ids: Vec<NodeId> = nearest.iter().map(|entry| entry.id).collect();
			assert_eq!(actual_ids, expected_ids);
		}
	}

	#[test]
	fn packets() {
		let key = Random.generate().unwrap();
		let ep = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40449").unwrap(), udp_port: 40449 };
		let mut discovery = Discovery::new(&key, ep.address.clone(), ep.clone(), 0, IpFilter::default());
		discovery.check_timestamps = false;
		let from = SocketAddr::from_str("99.99.99.99:40445").unwrap();

		let packet = "\
		e9614ccfd9fc3e74360018522d30e1419a143407ffcce748de3e22116b7e8dc92ff74788c0b6663a\
		aa3d67d641936511c8f8d6ad8698b820a7cf9e1be7155e9a241f556658c55428ec0563514365799a\
		4be2be5a685a80971ddcfa80cb422cdd0101ec04cb847f000001820cfa8215a8d790000000000000\
		000000000000000000018208ae820d058443b9a3550102\
		".from_hex().unwrap();
		assert!(discovery.on_packet(&packet, from.clone()).is_ok());

		let packet = "\
		577be4349c4dd26768081f58de4c6f375a7a22f3f7adda654d1428637412c3d7fe917cadc56d4e5e\
		7ffae1dbe3efffb9849feb71b262de37977e7c7a44e677295680e9e38ab26bee2fcbae207fba3ff3\
		d74069a50b902a82c9903ed37cc993c50001f83e82022bd79020010db83c4d001500000000abcdef\
		12820cfa8215a8d79020010db885a308d313198a2e037073488208ae82823a8443b9a355c5010203\
		040531b9019afde696e582a78fa8d95ea13ce3297d4afb8ba6433e4154caa5ac6431af1b80ba7602\
		3fa4090c408f6b4bc3701562c031041d4702971d102c9ab7fa5eed4cd6bab8f7af956f7d565ee191\
		7084a95398b6a21eac920fe3dd1345ec0a7ef39367ee69ddf092cbfe5b93e5e568ebc491983c09c7\
		6d922dc3\
		".from_hex().unwrap();
		assert!(discovery.on_packet(&packet, from.clone()).is_ok());

		let packet = "\
		09b2428d83348d27cdf7064ad9024f526cebc19e4958f0fdad87c15eb598dd61d08423e0bf66b206\
		9869e1724125f820d851c136684082774f870e614d95a2855d000f05d1648b2d5945470bc187c2d2\
		216fbe870f43ed0909009882e176a46b0102f846d79020010db885a308d313198a2e037073488208\
		ae82823aa0fbc914b16819237dcd8801d7e53f69e9719adecb3cc0e790c57e91ca4461c9548443b9\
		a355c6010203c2040506a0c969a58f6f9095004c0177a6b47f451530cab38966a25cca5cb58f0555
		42124e\
		".from_hex().unwrap();
		assert!(discovery.on_packet(&packet, from.clone()).is_ok());

		let packet = "\
		c7c44041b9f7c7e41934417ebac9a8e1a4c6298f74553f2fcfdcae6ed6fe53163eb3d2b52e39fe91\
		831b8a927bf4fc222c3902202027e5e9eb812195f95d20061ef5cd31d502e47ecb61183f74a504fe\
		04c51e73df81f25c4d506b26db4517490103f84eb840ca634cae0d49acb401d8a4c6b6fe8c55b70d\
		115bf400769cc1400f3258cd31387574077f301b421bc84df7266c44e9e6d569fc56be0081290476\
		7bf5ccd1fc7f8443b9a35582999983999999280dc62cc8255c73471e0a61da0c89acdc0e035e260a\
		dd7fc0c04ad9ebf3919644c91cb247affc82b69bd2ca235c71eab8e49737c937a2c396\
		".from_hex().unwrap();
		assert!(discovery.on_packet(&packet, from.clone()).is_ok());

		let packet = "\
		c679fc8fe0b8b12f06577f2e802d34f6fa257e6137a995f6f4cbfc9ee50ed3710faf6e66f932c4c8\
		d81d64343f429651328758b47d3dbc02c4042f0fff6946a50f4a49037a72bb550f3a7872363a83e1\
		b9ee6469856c24eb4ef80b7535bcf99c0004f9015bf90150f84d846321163782115c82115db84031\
		55e1427f85f10a5c9a7755877748041af1bcd8d474ec065eb33df57a97babf54bfd2103575fa8291\
		15d224c523596b401065a97f74010610fce76382c0bf32f84984010203040101b840312c55512422\
		cf9b8a4097e9a6ad79402e87a15ae909a4bfefa22398f03d20951933beea1e4dfa6f968212385e82\
		9f04c2d314fc2d4e255e0d3bc08792b069dbf8599020010db83c4d001500000000abcdef12820d05\
		820d05b84038643200b172dcfef857492156971f0e6aa2c538d8b74010f8e140811d53b98c765dd2\
		d96126051913f44582e8c199ad7c6d6819e9a56483f637feaac9448aacf8599020010db885a308d3\
		13198a2e037073488203e78203e8b8408dcab8618c3253b558d459da53bd8fa68935a719aff8b811\
		197101a4b2b47dd2d47295286fc00cc081bb542d760717d1bdd6bec2c37cd72eca367d6dd3b9df73\
		8443b9a355010203b525a138aa34383fec3d2719a0\
		".from_hex().unwrap();
		assert!(discovery.on_packet(&packet, from.clone()).is_ok());
	}

	#[test]
	fn test_ping() {
		let key1 = Random.generate().unwrap();
		let key2 = Random.generate().unwrap();
		let ep1 = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40344").unwrap(), udp_port: 40344 };
		let ep2 = NodeEndpoint { address: SocketAddr::from_str("127.0.0.1:40345").unwrap(), udp_port: 40345 };
		let mut discovery1 = Discovery::new(&key1, ep1.address.clone(), ep1.clone(), 0, IpFilter::default());
		let mut discovery2 = Discovery::new(&key2, ep2.address.clone(), ep2.clone(), 0, IpFilter::default());

		discovery1.ping(&ep2);
		let ping_data = discovery1.send_queue.pop_front().unwrap();
		discovery2.on_packet(&ping_data.payload, ep1.address.clone()).ok();
		let pong_data = discovery2.send_queue.pop_front().unwrap();
		let data = &pong_data.payload[(32 + 65)..];
		let rlp = Rlp::new(&data[1..]);
		assert_eq!(ping_data.payload[0..32], rlp.val_at::<Vec<u8>>(1).unwrap()[..])
	}
}
