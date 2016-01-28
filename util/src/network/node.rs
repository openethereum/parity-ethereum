use std::net::{SocketAddr, ToSocketAddrs};
use std::hash::{Hash, Hasher};
use std::str::{FromStr};
use hash::*;
use rlp::*;
use time::Tm;
use error::*;

/// Node public key
pub type NodeId = H512;

#[derive(Debug)]
/// Noe address info
pub struct NodeEndpoint {
	/// IP(V4 or V6) address
	pub address: SocketAddr,
	/// Address as string (can be host name).
	pub address_str: String,
	/// Conneciton port.
	pub udp_port: u16
}

impl FromStr for NodeEndpoint {
	type Err = UtilError;

	/// Create endpoint from string. Performs name resolution if given a host name.
	fn from_str(s: &str) -> Result<NodeEndpoint, UtilError> {
		let address = s.to_socket_addrs().map(|mut i| i.next());
		match address {
			Ok(Some(a)) => Ok(NodeEndpoint {
				address: a,
				address_str: s.to_owned(),
				udp_port: a.port()
			}),
			Ok(_) => Err(UtilError::AddressResolve(None)),
			Err(e) => Err(UtilError::AddressResolve(Some(e)))
		}
	}
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PeerType {
	Required,
	Optional
}

pub struct Node {
	pub id: NodeId,
	pub endpoint: NodeEndpoint,
	pub peer_type: PeerType,
	pub last_attempted: Option<Tm>,
}

impl FromStr for Node {
	type Err = UtilError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (id, endpoint) = if &s[0..8] == "enode://" && s.len() > 136 && &s[136..137] == "@" {
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
		})
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}
impl Eq for Node { }

impl Hash for Node {
	fn hash<H>(&self, state: &mut H) where H: Hasher {
		self.id.hash(state)
	}
}

