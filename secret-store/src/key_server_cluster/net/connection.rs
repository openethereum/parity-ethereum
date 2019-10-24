// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::net;
use crypto::publickey::KeyPair;
use key_server_cluster::NodeId;
use key_server_cluster::io::SharedTcpStream;

/// Established connection data
pub struct Connection {
	/// Peer address.
	pub address: net::SocketAddr,
	/// Connection stream.
	pub stream: SharedTcpStream,
	/// Peer node id.
	pub node_id: NodeId,
	/// Encryption key.
	pub key: KeyPair,
}
