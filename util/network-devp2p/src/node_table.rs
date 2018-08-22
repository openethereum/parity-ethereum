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

use discovery::{TableUpdates, NodeEntry};
use ethereum_types::H512;
use ip_utils::*;
use network::{Error, ErrorKind, AllowIP, IpFilter};
use rlp::{Rlp, RlpStream, DecoderError};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::net::{SocketAddr, ToSocketAddrs, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, slice};
use std::time::{self, Duration, SystemTime};
use rand::{self, Rng};

/// Node public key
pub type NodeId = H512;

#[derive(Debug, Clone, PartialEq)]
/// Node address info
pub struct NodeEndpoint {
	/// IP(V4 or V6) address
	pub address: SocketAddr,
	/// Connection port.
	pub udp_port: u16
}

impl NodeEndpoint {
	pub fn udp_address(&self) -> SocketAddr {
		match self.address {
			SocketAddr::V4(a) => SocketAddr::V4(SocketAddrV4::new(*a.ip(), self.udp_port)),
			SocketAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(*a.ip(), self.udp_port, a.flowinfo(), a.scope_id())),
		}
	}

	pub fn is_allowed(&self, filter: &IpFilter) -> bool {
		(self.is_allowed_by_predefined(&filter.predefined) || filter.custom_allow.iter().any(|ipnet| {
			self.address.ip().is_within(ipnet)
		}))
		&& !filter.custom_block.iter().any(|ipnet| {
			self.address.ip().is_within(ipnet)
		})
	}

	pub fn is_allowed_by_predefined(&self, filter: &AllowIP) -> bool {
		match filter {
			AllowIP::All => true,
			AllowIP::Private => self.address.ip().is_usable_private(),
			AllowIP::Public => self.address.ip().is_usable_public(),
			AllowIP::None => false,
		}
	}

	pub fn from_rlp(rlp: &Rlp) -> Result<Self, DecoderError> {
		let tcp_port = rlp.val_at::<u16>(2)?;
		let udp_port = rlp.val_at::<u16>(1)?;
		let addr_bytes = rlp.at(0)?.data()?;
		let address = match addr_bytes.len() {
			4 => Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(addr_bytes[0], addr_bytes[1], addr_bytes[2], addr_bytes[3]), tcp_port))),
			16 => unsafe {
				let o: *const u16 = addr_bytes.as_ptr() as *const u16;
				let o = slice::from_raw_parts(o, 8);
				Ok(SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(o[0], o[1], o[2], o[3], o[4], o[5], o[6], o[7]), tcp_port, 0, 0)))
			},
			_ => Err(DecoderError::RlpInconsistentLengthAndData)
		}?;
		Ok(NodeEndpoint { address, udp_port })
	}

	pub fn to_rlp(&self, rlp: &mut RlpStream) {
		match self.address {
			SocketAddr::V4(a) => {
				rlp.append(&(&a.ip().octets()[..]));
			}
			SocketAddr::V6(a) => unsafe {
				let o: *const u8 = a.ip().segments().as_ptr() as *const u8;
				rlp.append(&slice::from_raw_parts(o, 16));
			}
		};
		rlp.append(&self.udp_port);
		rlp.append(&self.address.port());
	}

	pub fn to_rlp_list(&self, rlp: &mut RlpStream) {
		rlp.begin_list(3);
		self.to_rlp(rlp);
	}

	/// Validates that the port is not 0 and address IP is specified
	pub fn is_valid(&self) -> bool {
		self.udp_port != 0 && self.address.port() != 0 &&
		match self.address {
			SocketAddr::V4(a) => !a.ip().is_unspecified(),
			SocketAddr::V6(a) => !a.ip().is_unspecified()
		}
	}
}

impl FromStr for NodeEndpoint {
	type Err = Error;

	/// Create endpoint from string. Performs name resolution if given a host name.
	fn from_str(s: &str) -> Result<NodeEndpoint, Error> {
		let address = s.to_socket_addrs().map(|mut i| i.next());
		match address {
			Ok(Some(a)) => Ok(NodeEndpoint {
				address: a,
				udp_port: a.port()
			}),
			Ok(None) => bail!(ErrorKind::AddressResolve(None)),
			Err(_) => Err(ErrorKind::AddressParse.into()) // always an io::Error of InvalidInput kind
		}
	}
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PeerType {
	_Required,
	Optional
}

/// A type for representing an interaction (contact) with a node at a given time
/// that was either a success or a failure.
#[derive(Clone, Copy, Debug)]
pub enum NodeContact {
	Success(SystemTime),
	Failure(SystemTime),
}

impl NodeContact {
	fn success() -> NodeContact {
		NodeContact::Success(SystemTime::now())
	}

	fn failure() -> NodeContact {
		NodeContact::Failure(SystemTime::now())
	}

	fn time(&self) -> SystemTime {
		match *self {
			NodeContact::Success(t) | NodeContact::Failure(t) => t
		}
	}

	/// Filters and old contact, returning `None` if it happened longer than a
	/// week ago.
	fn recent(&self) -> Option<&NodeContact> {
		let t = self.time();
		if let Ok(d) = t.elapsed() {
			if d < Duration::from_secs(60 * 60 * 24 * 7) {
				return Some(self);
			}
		}

		None
	}
}

#[derive(Debug)]
pub struct Node {
	pub id: NodeId,
	pub endpoint: NodeEndpoint,
	pub peer_type: PeerType,
	pub last_contact: Option<NodeContact>,
}

impl Node {
	pub fn new(id: NodeId, endpoint: NodeEndpoint) -> Node {
		Node {
			id,
			endpoint,
			peer_type: PeerType::Optional,
			last_contact: None,
		}
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.endpoint.udp_port != self.endpoint.address.port() {
			write!(f, "enode://{:x}@{}+{}", self.id, self.endpoint.address, self.endpoint.udp_port)?;
		} else {
			write!(f, "enode://{:x}@{}", self.id, self.endpoint.address)?;
		}
		Ok(())
	}
}

impl FromStr for Node {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (id, endpoint) = if s.len() > 136 && &s[0..8] == "enode://" && &s[136..137] == "@" {
			(s[8..136].parse().map_err(|_| ErrorKind::InvalidNodeId)?, NodeEndpoint::from_str(&s[137..])?)
		}
		else {
			(NodeId::new(), NodeEndpoint::from_str(s)?)
		};

		Ok(Node {
			id,
			endpoint,
			peer_type: PeerType::Optional,
			last_contact: None,
		})
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}
impl Eq for Node {}

impl Hash for Node {
	fn hash<H>(&self, state: &mut H) where H: Hasher {
		self.id.hash(state)
	}
}

const MAX_NODES: usize = 1024;
const NODES_FILE: &str = "nodes.json";

/// Node table backed by disk file.
pub struct NodeTable {
	nodes: HashMap<NodeId, Node>,
	useless_nodes: HashSet<NodeId>,
	path: Option<String>,
}

impl NodeTable {
	pub fn new(path: Option<String>) -> NodeTable {
		NodeTable {
			path: path.clone(),
			nodes: NodeTable::load(path),
			useless_nodes: HashSet::new(),
		}
	}

	/// Add a node to table
	pub fn add_node(&mut self, mut node: Node) {
		// preserve node last_contact
		node.last_contact = self.nodes.get(&node.id).and_then(|n| n.last_contact);
		self.nodes.insert(node.id, node);
	}

	/// Returns a list of ordered nodes according to their most recent contact
	/// and filtering useless nodes. The algorithm for creating the sorted nodes
	/// is:
	/// - Contacts that aren't recent (older than 1 week) are discarded
	/// - (1) Nodes with a successful contact are ordered (most recent success first)
	/// - (2) Nodes with unknown contact (older than 1 week or new nodes) are randomly shuffled
	/// - (3) Nodes with a failed contact are ordered (oldest failure first)
	/// - The final result is the concatenation of (1), (2) and (3)
	fn ordered_entries(&self) -> Vec<&Node> {
		let mut success = Vec::new();
		let mut failures = Vec::new();
		let mut unknown = Vec::new();

		let nodes = self.nodes.values()
			.filter(|n| !self.useless_nodes.contains(&n.id));

		for node in nodes {
			// discard contact points older that aren't recent
			match node.last_contact.as_ref().and_then(|c| c.recent()) {
				Some(&NodeContact::Success(_)) => {
					success.push(node);
				},
				Some(&NodeContact::Failure(_)) => {
					failures.push(node);
				},
				None => {
					unknown.push(node);
				},
			}
		}

		success.sort_by(|a, b| {
			let a = a.last_contact.expect("vector only contains values with defined last_contact; qed");
			let b = b.last_contact.expect("vector only contains values with defined last_contact; qed");
			// inverse ordering, most recent successes come first
			b.time().cmp(&a.time())
		});

		failures.sort_by(|a, b| {
			let a = a.last_contact.expect("vector only contains values with defined last_contact; qed");
			let b = b.last_contact.expect("vector only contains values with defined last_contact; qed");
			// normal ordering, most distant failures come first
			a.time().cmp(&b.time())
		});

		rand::thread_rng().shuffle(&mut unknown);

		success.append(&mut unknown);
		success.append(&mut failures);
		success
	}

	/// Returns node ids sorted by failure percentage, for nodes with the same failure percentage the absolute number of
	/// failures is considered.
	pub fn nodes(&self, filter: &IpFilter) -> Vec<NodeId> {
		self.ordered_entries().iter()
			.filter(|n| n.endpoint.is_allowed(&filter))
			.map(|n| n.id)
			.collect()
	}

	/// Ordered list of all entries by failure percentage, for nodes with the same failure percentage the absolute
	/// number of failures is considered.
	pub fn entries(&self) -> Vec<NodeEntry> {
		self.ordered_entries().iter().map(|n| NodeEntry {
			endpoint: n.endpoint.clone(),
			id: n.id,
		}).collect()
	}

	/// Get particular node
	pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
		self.nodes.get_mut(id)
	}

	/// Check if a node exists in the table.
	pub fn contains(&self, id: &NodeId) -> bool {
		self.nodes.contains_key(id)
	}

	/// Apply table changes coming from discovery
	pub fn update(&mut self, mut update: TableUpdates, reserved: &HashSet<NodeId>) {
		for (_, node) in update.added.drain() {
			let entry = self.nodes.entry(node.id).or_insert_with(|| Node::new(node.id, node.endpoint.clone()));
			entry.endpoint = node.endpoint;
		}
		for r in update.removed {
			if !reserved.contains(&r) {
				self.nodes.remove(&r);
			}
		}
	}

	/// Set last contact as failure for a node
	pub fn note_failure(&mut self, id: &NodeId) {
		if let Some(node) = self.nodes.get_mut(id) {
			node.last_contact = Some(NodeContact::failure());
		}
	}

	/// Set last contact as success for a node
	pub fn note_success(&mut self, id: &NodeId) {
		if let Some(node) = self.nodes.get_mut(id) {
			node.last_contact = Some(NodeContact::success());
		}
	}

	/// Mark as useless, no further attempts to connect until next call to `clear_useless`.
	pub fn mark_as_useless(&mut self, id: &NodeId) {
		self.useless_nodes.insert(id.clone());
	}

	/// Attempt to connect to useless nodes again.
	pub fn clear_useless(&mut self) {
		self.useless_nodes.clear();
	}

	/// Save the nodes.json file.
	pub fn save(&self) {
		let mut path = match self.path {
			Some(ref path) => PathBuf::from(path),
			None => return,
		};
		if let Err(e) = fs::create_dir_all(&path) {
			warn!("Error creating node table directory: {:?}", e);
			return;
		}
		path.push(NODES_FILE);
		let node_ids = self.nodes(&IpFilter::default());
		let nodes = node_ids.into_iter()
			.map(|id| self.nodes.get(&id).expect("self.nodes() only returns node IDs from self.nodes"))
			.take(MAX_NODES)
			.map(Into::into)
			.collect();
		let table = json::NodeTable { nodes };

		match fs::File::create(&path) {
			Ok(file) => {
				if let Err(e) = serde_json::to_writer_pretty(file, &table) {
					warn!("Error writing node table file: {:?}", e);
				}
			},
			Err(e) => {
				warn!("Error creating node table file: {:?}", e);
			}
		}
	}

	fn load(path: Option<String>) -> HashMap<NodeId, Node> {
		let path = match path {
			Some(path) => PathBuf::from(path).join(NODES_FILE),
			None => return Default::default(),
		};

		let file = match fs::File::open(&path) {
			Ok(file) => file,
			Err(e) => {
				debug!("Error opening node table file: {:?}", e);
				return Default::default();
			},
		};
		let res: Result<json::NodeTable, _> = serde_json::from_reader(file);
		match res {
			Ok(table) => {
				table.nodes.into_iter()
					.filter_map(|n| n.into_node())
					.map(|n| (n.id, n))
					.collect()
			},
			Err(e) => {
				warn!("Error reading node table file: {:?}", e);
				Default::default()
			},
		}
	}
}

impl Drop for NodeTable {
	fn drop(&mut self) {
		self.save();
	}
}

/// Check if node url is valid
pub fn validate_node_url(url: &str) -> Option<Error> {
	match Node::from_str(url) {
		Ok(_) => None,
		Err(e) => Some(e)
	}
}

mod json {
	use super::*;

	#[derive(Serialize, Deserialize)]
	pub struct NodeTable {
		pub nodes: Vec<Node>,
	}

	#[derive(Serialize, Deserialize)]
	pub enum NodeContact {
		#[serde(rename = "success")]
		Success(u64),
		#[serde(rename = "failure")]
		Failure(u64),
	}

	impl NodeContact {
		pub fn into_node_contact(self) -> super::NodeContact {
			match self {
				NodeContact::Success(s) => super::NodeContact::Success(
					time::UNIX_EPOCH + Duration::from_secs(s)
				),
				NodeContact::Failure(s) => super::NodeContact::Failure(
					time::UNIX_EPOCH + Duration::from_secs(s)
				),
			}
		}
	}

	#[derive(Serialize, Deserialize)]
	pub struct Node {
		pub url: String,
		pub last_contact: Option<NodeContact>,
	}

	impl Node {
		pub fn into_node(self) -> Option<super::Node> {
			match super::Node::from_str(&self.url) {
				Ok(mut node) => {
					node.last_contact = self.last_contact.map(|c| c.into_node_contact());
					Some(node)
				},
				_ => None,
			}
		}
	}

	impl<'a> From<&'a super::Node> for Node {
		fn from(node: &'a super::Node) -> Self {
			let last_contact = node.last_contact.and_then(|c| {
				match c {
					super::NodeContact::Success(t) =>
						t.duration_since(time::UNIX_EPOCH).ok().map(|d| NodeContact::Success(d.as_secs())),
					super::NodeContact::Failure(t) =>
						t.duration_since(time::UNIX_EPOCH).ok().map(|d| NodeContact::Failure(d.as_secs())),
				}
			});

			Node {
				url: format!("{}", node),
				last_contact
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
	use ethereum_types::H512;
	use std::str::FromStr;
	use tempdir::TempDir;
	use ipnetwork::IpNetwork;

	#[test]
	fn endpoint_parse() {
		let endpoint = NodeEndpoint::from_str("123.99.55.44:7770");
		assert!(endpoint.is_ok());
		let v4 = match endpoint.unwrap().address {
			SocketAddr::V4(v4address) => v4address,
			_ => panic!("should be v4 address")
		};
		assert_eq!(SocketAddrV4::new(Ipv4Addr::new(123, 99, 55, 44), 7770), v4);
	}

	#[test]
	fn endpoint_parse_empty_ip_string_returns_error() {
		let endpoint = NodeEndpoint::from_str("");
		assert!(endpoint.is_err());
		assert_matches!(endpoint.unwrap_err().kind(), &ErrorKind::AddressParse);
	}

	#[test]
	fn endpoint_parse_invalid_ip_string_returns_error() {
		let endpoint = NodeEndpoint::from_str("beef");
		assert!(endpoint.is_err());
		assert_matches!(endpoint.unwrap_err().kind(), &ErrorKind::AddressParse);
	}

	#[test]
	fn endpoint_parse_valid_ip_without_port_returns_error() {
		let endpoint = NodeEndpoint::from_str("123.123.123.123");
		assert!(endpoint.is_err());
		assert_matches!(endpoint.unwrap_err().kind(), &ErrorKind::AddressParse);
		let endpoint = NodeEndpoint::from_str("123.123.123.123:123");
		assert!(endpoint.is_ok())
	}

	#[test]
	fn node_parse() {
		assert!(validate_node_url("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").is_none());
		let node = Node::from_str("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770");
		assert!(node.is_ok());
		let node = node.unwrap();
		let v4 = match node.endpoint.address {
			SocketAddr::V4(v4address) => v4address,
			_ => panic!("should ve v4 address")
		};
		assert_eq!(SocketAddrV4::new(Ipv4Addr::new(22, 99, 55, 44), 7770), v4);
		assert_eq!(
			H512::from_str("a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap(),
			node.id);
	}

	#[test]
	fn node_parse_fails_for_invalid_urls() {
		let node = Node::from_str("foo");
		assert!(node.is_err());
		assert_matches!(node.unwrap_err().kind(), &ErrorKind::AddressParse);

		let node = Node::from_str("enode://foo@bar");
		assert!(node.is_err());
		assert_matches!(node.unwrap_err().kind(), &ErrorKind::AddressParse);
	}

	#[test]
	fn table_last_contact_order() {
		let node1 = Node::from_str("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node2 = Node::from_str("enode://b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node3 = Node::from_str("enode://c979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node4 = Node::from_str("enode://d979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node5 = Node::from_str("enode://e979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node6 = Node::from_str("enode://f979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let id1 = H512::from_str("a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id2 = H512::from_str("b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id3 = H512::from_str("c979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id4 = H512::from_str("d979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id5 = H512::from_str("e979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id6 = H512::from_str("f979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let mut table = NodeTable::new(None);

		table.add_node(node1);
		table.add_node(node2);
		table.add_node(node3);
		table.add_node(node4);
		table.add_node(node5);
		table.add_node(node6);

		// failures - nodes 1 & 2
		table.note_failure(&id1);
		table.note_failure(&id2);

		// success - nodes 3 & 4
		table.note_success(&id3);
		table.note_success(&id4);

		// success - node 5 (old contact)
		table.get_mut(&id5).unwrap().last_contact = Some(NodeContact::Success(time::UNIX_EPOCH));

		// unknown - node 6

		let r = table.nodes(&IpFilter::default());

		assert_eq!(r[0][..], id4[..]); // most recent success
		assert_eq!(r[1][..], id3[..]);

		// unknown (old contacts and new nodes), randomly shuffled
		assert!(
			r[2][..] == id5[..] && r[3][..] == id6[..] ||
			r[2][..] == id6[..] && r[3][..] == id5[..]
		);

		assert_eq!(r[4][..], id1[..]); // oldest failure
		assert_eq!(r[5][..], id2[..]);
	}

	#[test]
	fn table_save_load() {
		let tempdir = TempDir::new("").unwrap();
		let node1 = Node::from_str("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node2 = Node::from_str("enode://b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node3 = Node::from_str("enode://c979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let id1 = H512::from_str("a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id2 = H512::from_str("b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id3 = H512::from_str("c979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();

		{
			let mut table = NodeTable::new(Some(tempdir.path().to_str().unwrap().to_owned()));
			table.add_node(node1);
			table.add_node(node2);
			table.add_node(node3);

			table.note_success(&id2);
			table.note_failure(&id3);
		}

		{
			let table = NodeTable::new(Some(tempdir.path().to_str().unwrap().to_owned()));
			let r = table.nodes(&IpFilter::default());
			assert_eq!(r[0][..], id2[..]); // latest success
			assert_eq!(r[1][..], id1[..]); // unknown
			assert_eq!(r[2][..], id3[..]); // oldest failure
		}
	}

	#[test]
	fn custom_allow() {
		let filter = IpFilter {
			predefined: AllowIP::None,
			custom_allow: vec![IpNetwork::from_str(&"10.0.0.0/8").unwrap(), IpNetwork::from_str(&"1.0.0.0/8").unwrap()],
			custom_block: vec![],
		};
		assert!(!NodeEndpoint::from_str("123.99.55.44:7770").unwrap().is_allowed(&filter));
		assert!(NodeEndpoint::from_str("10.0.0.1:7770").unwrap().is_allowed(&filter));
		assert!(NodeEndpoint::from_str("1.0.0.55:5550").unwrap().is_allowed(&filter));
	}

	#[test]
	fn custom_block() {
		let filter = IpFilter {
			predefined: AllowIP::All,
			custom_allow: vec![],
			custom_block: vec![IpNetwork::from_str(&"10.0.0.0/8").unwrap(), IpNetwork::from_str(&"1.0.0.0/8").unwrap()],
		};
		assert!(NodeEndpoint::from_str("123.99.55.44:7770").unwrap().is_allowed(&filter));
		assert!(!NodeEndpoint::from_str("10.0.0.1:7770").unwrap().is_allowed(&filter));
		assert!(!NodeEndpoint::from_str("1.0.0.55:5550").unwrap().is_allowed(&filter));
	}

	#[test]
	fn custom_allow_ipv6() {
		let filter = IpFilter {
			predefined: AllowIP::None,
			custom_allow: vec![IpNetwork::from_str(&"fc00::/8").unwrap()],
			custom_block: vec![],
		};
		assert!(NodeEndpoint::from_str("[fc00::]:5550").unwrap().is_allowed(&filter));
		assert!(!NodeEndpoint::from_str("[fd00::]:5550").unwrap().is_allowed(&filter));
	}

	#[test]
	fn custom_block_ipv6() {
		let filter = IpFilter {
			predefined: AllowIP::All,
			custom_allow: vec![],
			custom_block: vec![IpNetwork::from_str(&"fc00::/8").unwrap()],
		};
		assert!(!NodeEndpoint::from_str("[fc00::]:5550").unwrap().is_allowed(&filter));
		assert!(NodeEndpoint::from_str("[fd00::]:5550").unwrap().is_allowed(&filter));
	}
}
