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

/// Infinity networking

use util::*;
use network::*;
use rlp::{UntrustedRlp, DecoderError, RlpStream, View, Stream};
use ethcore::client::{BlockChainClient};
use sync_io::SyncIo;
use super::SyncConfig;

known_heap_size!(0, PeerInfo);

type PacketDecodeError = DecoderError;

const PROTOCOL_VERSION: u8 = 1u8;

const STATUS_PACKET: u8 = 0x00;
const GENERIC_PACKET: u8 = 0x01;

/// Syncing status and statistics
#[derive(Clone)]
pub struct NetworkStatus {
	pub protocol_version: u8,
	/// The underlying p2p network version.
	pub network_id: usize,
	/// Total number of connected peers
	pub num_peers: usize,
	/// Total number of active peers
	pub num_active_peers: usize,
}

#[derive(Clone)]
/// Inf peer information
struct PeerInfo {
	/// inf protocol version
	protocol_version: u32,
	/// Peer chain genesis hash
	genesis: H256,
	/// Peer network id
	network_id: usize,
}

/// Infinity protocol handler.
pub struct InfinitySync {
	chain: Arc<BlockChainClient>,
	/// All connected peers
	peers: HashMap<PeerId, PeerInfo>,
	/// Network ID
	network_id: usize,
}

impl InfinitySync {
	/// Create a new instance of syncing strategy.
	pub fn new(config: &SyncConfig, chain: Arc<BlockChainClient>) -> InfinitySync {
		let mut sync = InfinitySync {
			chain: chain,
			peers: HashMap::new(),
			network_id: config.network_id,
		};
		sync.reset();
		sync
	}

	/// @returns Synchonization status
	pub fn _status(&self) -> NetworkStatus {
		NetworkStatus {
			protocol_version: 1,
			network_id: self.network_id,
			num_peers: self.peers.len(),
			num_active_peers: 0,
		}
	}

	#[cfg_attr(feature="dev", allow(for_kv_map))] // Because it's not possible to get `values_mut()`
	/// Reset sync. Clear all downloaded data but keep the queue
	fn reset(&mut self) {
	}

	/// Called by peer to report status
	fn on_peer_status(&mut self, io: &mut SyncIo, peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		let peer = PeerInfo {
			protocol_version: try!(r.val_at(0)),
			network_id: try!(r.val_at(1)),
			genesis: try!(r.val_at(2)),
		};
		trace!(target: "inf", "New peer {} (protocol: {}, network: {:?}, genesis:{})", peer_id, peer.protocol_version, peer.network_id, peer.genesis);
		if self.peers.contains_key(&peer_id) {
			debug!(target: "inf", "Unexpected status packet from {}:{}", peer_id, io.peer_info(peer_id));
			return Ok(());
		}
		let chain_info = io.chain().chain_info();
		if peer.genesis != chain_info.genesis_hash {
			io.disable_peer(peer_id);
			trace!(target: "inf", "Peer {} genesis hash mismatch (ours: {}, theirs: {})", peer_id, chain_info.genesis_hash, peer.genesis);
			return Ok(());
		}
		if peer.network_id != self.network_id {
			io.disable_peer(peer_id);
			trace!(target: "inf", "Peer {} network id mismatch (ours: {}, theirs: {})", peer_id, self.network_id, peer.network_id);
			return Ok(());
		}

		self.peers.insert(peer_id.clone(), peer);
		Ok(())
	}

	/// Called when a new peer is connected
	pub fn on_peer_connected(&mut self, io: &mut SyncIo, peer: PeerId) {
		trace!(target: "inf", "== Connected {}: {}", peer, io.peer_info(peer));
		if let Err(e) = self.send_status(io) {
			debug!(target:"inf", "Error sending status request: {:?}", e);
			io.disable_peer(peer);
		}
	}

	/// Generic packet sender
	fn send_packet(&mut self, sync: &mut SyncIo, peer_id: PeerId, packet_id: PacketId, packet: Bytes) {
		if self.peers.contains_key(&peer_id) {
			if let Err(e) = sync.send(peer_id, packet_id, packet) {
				debug!(target:"inf", "Error sending request: {:?}", e);
				sync.disable_peer(peer_id);
			}
		}
	}

	/// Called when peer sends us new transactions
	fn on_peer_packet(&mut self, _io: &mut SyncIo, _peer_id: PeerId, r: &UntrustedRlp) -> Result<(), PacketDecodeError> {
		self.chain.queue_infinity_message(r.as_raw().to_vec());
		Ok(())
	}

	/// Called by peer when it is disconnecting
	pub fn on_peer_aborting(&mut self, io: &mut SyncIo, peer: PeerId) {
		trace!(target: "inf", "== Disconnecting {}: {}", peer, io.peer_info(peer));
		if self.peers.contains_key(&peer) {
			debug!(target: "inf", "Disconnected {}", peer);
			self.peers.remove(&peer);
		}
	}

	/// Send Status message
	fn send_status(&mut self, io: &mut SyncIo) -> Result<(), NetworkError> {
		let mut packet = RlpStream::new_list(5);
		let chain = io.chain().chain_info();
		packet.append(&(PROTOCOL_VERSION as u32));
		packet.append(&self.network_id);
		packet.append(&chain.total_difficulty);
		packet.append(&chain.best_block_hash);
		packet.append(&chain.genesis_hash);
		io.respond(STATUS_PACKET, packet.out())
	}

	pub fn dispatch_packet(sync: &RwLock<InfinitySync>, io: &mut SyncIo, peer: PeerId, packet_id: u8, data: &[u8]) {
		let rlp = UntrustedRlp::new(data);
		match packet_id {
			STATUS_PACKET => sync.write().on_peer_status(io, peer, &rlp).unwrap_or_else(
				|e| trace!(target: "inf", "Error processing packet: {:?}", e)),
			GENERIC_PACKET => sync.write().on_peer_packet(io, peer, &rlp).unwrap_or_else(
				|e| warn!(target: "inf", "Error queueing packet: {:?}", e)),
			p @ _ => trace!(target: "inf", "Unexpected packet {} from {}", p, peer),
		};
	}

	pub fn propagate_packet(&mut self, io: &mut SyncIo, packet: Bytes) {
		let lucky_peers: Vec<_> = self.peers.keys().cloned().collect();
		trace!(target: "inf", "Sending packets to {:?}", lucky_peers);
		for peer_id in lucky_peers {
			self.send_packet(io, peer_id, GENERIC_PACKET, packet.clone());
		}
	}
}

#[cfg(test)]
mod tests {
}

