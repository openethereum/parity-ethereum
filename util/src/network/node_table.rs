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

use std::mem;
use std::slice::from_raw_parts;
use std::net::{SocketAddr, ToSocketAddrs, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr};
use std::hash::{Hash, Hasher};
use std::str::{FromStr};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::path::{PathBuf};
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use hash::*;
use rlp::*;
use time::Tm;
use error::*;
use network::discovery::{TableUpdates, NodeEntry};
use network::ip_utils::*;
pub use rustc_serialize::json::Json;

/// Node public key
pub type NodeId = H512;

#[derive(Debug, Clone)]
/// Node address info
pub struct NodeEndpoint {
	/// IP(V4 or V6) address
	pub address: SocketAddr,
	/// Conneciton port.
	pub udp_port: u16
}

impl NodeEndpoint {
	pub fn udp_address(&self) -> SocketAddr {
		match self.address {
			SocketAddr::V4(a) => SocketAddr::V4(SocketAddrV4::new(a.ip().clone(), self.udp_port)),
			SocketAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(a.ip().clone(), self.udp_port, a.flowinfo(), a.scope_id())),
		}
	}
}

impl NodeEndpoint {
	pub fn from_rlp(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		let tcp_port = try!(rlp.val_at::<u16>(2));
		let udp_port = try!(rlp.val_at::<u16>(1));
		let addr_bytes = try!(try!(rlp.at(0)).data());
		let address = try!(match addr_bytes.len() {
			4 => Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(addr_bytes[0], addr_bytes[1], addr_bytes[2], addr_bytes[3]), tcp_port))),
			16 => unsafe {
				let o: *const u16 = mem::transmute(addr_bytes.as_ptr());
				let o = from_raw_parts(o, 8);
				Ok(SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(o[0], o[1], o[2], o[3], o[4], o[5], o[6], o[7]), tcp_port, 0, 0)))
			},
			_ => Err(DecoderError::RlpInconsistentLengthAndData)
		});
		Ok(NodeEndpoint { address: address, udp_port: udp_port })
	}

	pub fn to_rlp(&self, rlp: &mut RlpStream) {
		match self.address {
			SocketAddr::V4(a) => {
				rlp.append(&(&a.ip().octets()[..]));
			}
			SocketAddr::V6(a) => unsafe {
				let o: *const u8 = mem::transmute(a.ip().segments().as_ptr());
				rlp.append(&from_raw_parts(o, 16));
			}
		};
		rlp.append(&self.udp_port);
		rlp.append(&self.address.port());
	}

	pub fn to_rlp_list(&self, rlp: &mut RlpStream) {
		rlp.begin_list(3);
		self.to_rlp(rlp);
	}

	pub fn is_valid(&self) -> bool {
		self.udp_port != 0 && self.address.port() != 0 &&
		match self.address {
			SocketAddr::V4(a) => !a.ip().is_unspecified_s(),
			SocketAddr::V6(a) => !a.ip().is_unspecified_s()
		}
	}

	pub fn is_global(&self) -> bool {
		match self.address {
			SocketAddr::V4(a) => a.ip().is_global_s(),
			SocketAddr::V6(a) => a.ip().is_global_s()
		}
	}
}

impl FromStr for NodeEndpoint {
	type Err = UtilError;

	/// Create endpoint from string. Performs name resolution if given a host name.
	fn from_str(s: &str) -> Result<NodeEndpoint, UtilError> {
		let address = s.to_socket_addrs().map(|mut i| i.next());
		match address {
			Ok(Some(a)) => Ok(NodeEndpoint {
				address: a,
				udp_port: a.port()
			}),
			Ok(_) => Err(UtilError::AddressResolve(None)),
			Err(e) => Err(UtilError::AddressResolve(Some(e)))
		}
	}
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PeerType {
	_Required,
	Optional
}

pub struct Node {
	pub id: NodeId,
	pub endpoint: NodeEndpoint,
	pub peer_type: PeerType,
	pub failures: u32,
	pub last_attempted: Option<Tm>,
}

impl Node {
	pub fn new(id: NodeId, endpoint: NodeEndpoint) -> Node {
		Node {
			id: id,
			endpoint: endpoint,
			peer_type: PeerType::Optional,
			failures: 0,
			last_attempted: None,
		}
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.endpoint.udp_port != self.endpoint.address.port() {
			try!(write!(f, "enode://{}@{}+{}", self.id.hex(), self.endpoint.address, self.endpoint.udp_port));
		} else {
			try!(write!(f, "enode://{}@{}", self.id.hex(), self.endpoint.address));
		}
		Ok(())
	}
}

impl FromStr for Node {
	type Err = UtilError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (id, endpoint) = if s.len() > 136 && &s[0..8] == "enode://" && &s[136..137] == "@" {
			(try!(NodeId::from_str(&s[8..136])), try!(NodeEndpoint::from_str(&s[137..])))
		}
		else {
			(NodeId::new(), try!(NodeEndpoint::from_str(s)))
		};

		Ok(Node {
			id: id,
			endpoint: endpoint,
			peer_type: PeerType::Optional,
			last_attempted: None,
			failures: 0,
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
		// preserve failure counter
		let failures = self.nodes.get(&node.id).map_or(0, |n| n.failures);
		node.failures = failures;
		self.nodes.insert(node.id.clone(), node);
	}

	/// Returns node ids sorted by number of failures
	pub fn nodes(&self) -> Vec<NodeId> {
		let mut refs: Vec<&Node> = self.nodes.values().filter(|n| !self.useless_nodes.contains(&n.id)).collect();
		refs.sort_by(|a, b| a.failures.cmp(&b.failures));
		refs.iter().map(|n| n.id.clone()).collect()
	}

	/// Unordered list of all entries
	pub fn unordered_entries(&self) -> Vec<NodeEntry> {
		// preserve failure counter
		self.nodes.values().map(|n| NodeEntry { endpoint: n.endpoint.clone(), id: n.id.clone() }).collect()
	}

	/// Get particular node
	pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
		self.nodes.get_mut(id)
	}

	/// Apply table changes coming from discovery
	pub fn update(&mut self, mut update: TableUpdates) {
		for (_, node) in update.added.drain() {
			let mut entry = self.nodes.entry(node.id.clone()).or_insert_with(|| Node::new(node.id.clone(), node.endpoint.clone()));
			entry.endpoint = node.endpoint;
		}
		for r in update.removed {
			self.nodes.remove(&r);
		}
	}

	/// Increase failure counte for a node
	pub fn note_failure(&mut self, id: &NodeId) {
		if let Some(node) = self.nodes.get_mut(id) {
			node.failures += 1;
		}
	}

	/// Mark as useless, no furter attempts to connect until next call to `clear_useless`.
	pub fn mark_as_useless(&mut self, id: &NodeId) {
		self.useless_nodes.insert(id.clone());
	}

	/// Atempt to connect to useless nodes again.
	pub fn clear_useless(&mut self) {
		self.useless_nodes.clear();
	}

	fn save(&self) {
		if let Some(ref path) = self.path {
			let mut path_buf = PathBuf::from(path);
			if let Err(e) = fs::create_dir_all(path_buf.as_path()) {
				warn!("Error creating node table directory: {:?}", e);
				return;
			};
			path_buf.push("nodes.json");
			let mut json = String::new();
			json.push_str("{\n");
			json.push_str("\"nodes\": [\n");
			let node_ids = self.nodes();
			for i in 0 .. node_ids.len() {
				let node = self.nodes.get(&node_ids[i]).unwrap();
				json.push_str(&format!("\t{{ \"url\": \"{}\", \"failures\": {} }}{}\n", node, node.failures, if i == node_ids.len() - 1 {""} else {","}))
			}
			json.push_str("]\n");
			json.push_str("}");
			let mut file = match fs::File::create(path_buf.as_path()) {
				Ok(file) => file,
				Err(e) => {
					warn!("Error creating node table file: {:?}", e);
					return;
				}
			};
			if let Err(e) = file.write(&json.into_bytes()) {
					warn!("Error writing node table file: {:?}", e);
			}
		}
	}

	fn load(path: Option<String>) -> HashMap<NodeId, Node> {
		let mut nodes: HashMap<NodeId, Node> = HashMap::new();
		if let Some(path) = path {
			let mut path_buf = PathBuf::from(path);
			path_buf.push("nodes.json");
			let mut file = match fs::File::open(path_buf.as_path()) {
				Ok(file) => file,
				Err(e) => {
					debug!("Error opening node table file: {:?}", e);
					return nodes;
				}
			};
			let mut buf = String::new();
			match file.read_to_string(&mut buf) {
				Ok(_) => {},
				Err(e) => {
					warn!("Error reading node table file: {:?}", e);
					return nodes;
				}
			}
			let json = match Json::from_str(&buf) {
				Ok(json) => json,
				Err(e) => {
					warn!("Error parsing node table file: {:?}", e);
					return nodes;
				}
			};
			if let Some(list) = json.as_object().and_then(|o| o.get("nodes")).and_then(|n| n.as_array()) {
				for n in list.iter().filter_map(|n| n.as_object()) {
					if let Some(url) = n.get("url").and_then(|u| u.as_string()) {
						if let Ok(mut node) = Node::from_str(url) {
							if let Some(failures) = n.get("failures").and_then(|f| f.as_u64()) {
								node.failures = failures as u32;
							}
							nodes.insert(node.id.clone(), node);
						}
					}
				}
			}
		}
		nodes
	}
}

impl Drop for NodeTable {
	fn drop(&mut self) {
		self.save();
	}
}

/// Check if node url is valid
pub fn is_valid_node_url(url: &str) -> bool {
	use std::str::FromStr;
	Node::from_str(url).is_ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;
	use std::net::*;
	use hash::*;
	use devtools::*;

	#[test]
	fn endpoint_parse() {
		let endpoint = NodeEndpoint::from_str("123.99.55.44:7770");
		assert!(endpoint.is_ok());
		let v4 = match endpoint.unwrap().address {
			SocketAddr::V4(v4address) => v4address,
			_ => panic!("should ve v4 address")
		};
		assert_eq!(SocketAddrV4::new(Ipv4Addr::new(123, 99, 55, 44), 7770), v4);
	}

	#[test]
	fn node_parse() {
		assert!(is_valid_node_url("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770"));
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
	fn table_failure_order() {
		let node1 = Node::from_str("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node2 = Node::from_str("enode://b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node3 = Node::from_str("enode://c979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let id1 = H512::from_str("a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id2 = H512::from_str("b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id3 = H512::from_str("c979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let mut table = NodeTable::new(None);
		table.add_node(node3);
		table.add_node(node1);
		table.add_node(node2);

		table.note_failure(&id1);
		table.note_failure(&id1);
		table.note_failure(&id2);

		let r = table.nodes();
		assert_eq!(r[0][..], id3[..]);
		assert_eq!(r[1][..], id2[..]);
		assert_eq!(r[2][..], id1[..]);
	}

	#[test]
	fn table_save_load() {
		let temp_path = RandomTempPath::create_dir();
		let node1 = Node::from_str("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let node2 = Node::from_str("enode://b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770").unwrap();
		let id1 = H512::from_str("a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		let id2 = H512::from_str("b979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c").unwrap();
		{
			let mut table = NodeTable::new(Some(temp_path.as_path().to_str().unwrap().to_owned()));
			table.add_node(node1);
			table.add_node(node2);
			table.note_failure(&id2);
		}

		{
			let table = NodeTable::new(Some(temp_path.as_path().to_str().unwrap().to_owned()));
			let r = table.nodes();
			assert_eq!(r[0][..], id1[..]);
			assert_eq!(r[1][..], id2[..]);
		}
	}
}
