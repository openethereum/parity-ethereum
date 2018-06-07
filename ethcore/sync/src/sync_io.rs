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

use std::collections::HashMap;
use network::{NetworkContext, PeerId, PacketId, Error, SessionInfo, ProtocolId};
use bytes::Bytes;
use ethcore::client::BlockChainClient;
use ethcore::header::BlockNumber;
use ethcore::snapshot::SnapshotService;
use parking_lot::RwLock;

/// IO interface for the syncing handler.
/// Provides peer connection management and an interface to the blockchain client.
// TODO: ratings
pub trait SyncIo {
	/// Disable a peer
	fn disable_peer(&mut self, peer_id: PeerId);
	/// Disconnect peer
	fn disconnect_peer(&mut self, peer_id: PeerId);
	/// Respond to current request with a packet. Can be called from an IO handler for incoming packet.
	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error>;
	/// Send a packet to a peer.
	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error>;
	/// Send a packet to a peer using specified protocol.
	fn send_protocol(&mut self, protocol: ProtocolId, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error>;
	/// Get the blockchain
	fn chain(&self) -> &BlockChainClient;
	/// Get the snapshot service.
	fn snapshot_service(&self) -> &SnapshotService;
	/// Returns peer identifier string
	fn peer_info(&self, peer_id: PeerId) -> String {
		peer_id.to_string()
	}
	/// Returns information on p2p session
	fn peer_session_info(&self, peer_id: PeerId) -> Option<SessionInfo>;
	/// Maximum mutually supported ETH protocol version
	fn eth_protocol_version(&self, peer_id: PeerId) -> u8;
	/// Maximum mutually supported version of a gien protocol.
	fn protocol_version(&self, protocol: &ProtocolId, peer_id: PeerId) -> u8;
	/// Returns if the chain block queue empty
	fn is_chain_queue_empty(&self) -> bool {
		self.chain().queue_info().is_empty()
	}
	/// Check if the session is expired
	fn is_expired(&self) -> bool;
	/// Return sync overlay
	fn chain_overlay(&self) -> &RwLock<HashMap<BlockNumber, Bytes>>;
}

/// Wraps `NetworkContext` and the blockchain client
pub struct NetSyncIo<'s> {
	network: &'s NetworkContext,
	chain: &'s BlockChainClient,
	snapshot_service: &'s SnapshotService,
	chain_overlay: &'s RwLock<HashMap<BlockNumber, Bytes>>,
}

impl<'s> NetSyncIo<'s> {
	/// Creates a new instance from the `NetworkContext` and the blockchain client reference.
	pub fn new(network: &'s NetworkContext,
		chain: &'s BlockChainClient,
		snapshot_service: &'s SnapshotService,
		chain_overlay: &'s RwLock<HashMap<BlockNumber, Bytes>>) -> NetSyncIo<'s> {
		NetSyncIo {
			network: network,
			chain: chain,
			snapshot_service: snapshot_service,
			chain_overlay: chain_overlay,
		}
	}
}

impl<'s> SyncIo for NetSyncIo<'s> {
	fn disable_peer(&mut self, peer_id: PeerId) {
		self.network.disable_peer(peer_id);
	}

	fn disconnect_peer(&mut self, peer_id: PeerId) {
		self.network.disconnect_peer(peer_id);
	}

	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error>{
		self.network.respond(packet_id, data)
	}

	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error>{
		self.network.send(peer_id, packet_id, data)
	}

	fn send_protocol(&mut self, protocol: ProtocolId, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error>{
		self.network.send_protocol(protocol, peer_id, packet_id, data)
	}

	fn chain(&self) -> &BlockChainClient {
		self.chain
	}

	fn chain_overlay(&self) -> &RwLock<HashMap<BlockNumber, Bytes>> {
		self.chain_overlay
	}

	fn snapshot_service(&self) -> &SnapshotService {
		self.snapshot_service
	}

	fn peer_session_info(&self, peer_id: PeerId) -> Option<SessionInfo> {
		self.network.session_info(peer_id)
	}

	fn is_expired(&self) -> bool {
		self.network.is_expired()
	}

	fn eth_protocol_version(&self, peer_id: PeerId) -> u8 {
		self.network.protocol_version(self.network.subprotocol_name(), peer_id).unwrap_or(0)
	}

	fn protocol_version(&self, protocol: &ProtocolId, peer_id: PeerId) -> u8 {
		self.network.protocol_version(*protocol, peer_id).unwrap_or(0)
	}

	fn peer_info(&self, peer_id: PeerId) -> String {
		self.network.peer_client_version(peer_id)
	}
}
