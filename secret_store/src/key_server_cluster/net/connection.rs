use std::net;
use key_server_cluster::NodeId;
use key_server_cluster::io::SharedTcpStream;

pub struct Connection {
	pub address: net::SocketAddr,
	pub stream: SharedTcpStream,
	pub node_id: NodeId,
}
