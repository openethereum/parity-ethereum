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

use api::WARP_SYNC_PROTOCOL_ID;
use block_sync::BlockRequest;
use bytes::Bytes;
use ethcore::header::BlockNumber;
use ethereum_types::H256;
use network::{PeerId, PacketId};
use rlp::RlpStream;
use std::time::Instant;
use sync_io::SyncIo;

use super::{
	BlockSet,
	ChainSync,
	PeerAsking,
	ETH_PROTOCOL_VERSION_63,
	GET_BLOCK_BODIES_PACKET,
	GET_BLOCK_HEADERS_PACKET,
	GET_RECEIPTS_PACKET,
	GET_SNAPSHOT_DATA_PACKET,
	GET_SNAPSHOT_MANIFEST_PACKET,
};

/// The Chain Sync Requester: requesting data to other peers
pub struct SyncRequester;

impl SyncRequester {
	/// Perform block download request`
	pub fn request_blocks(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId, request: BlockRequest, block_set: BlockSet) {
		match request {
			BlockRequest::Headers { start, count, skip } => {
				SyncRequester::request_headers_by_hash(sync, io, peer_id, &start, count, skip, false, block_set);
			},
			BlockRequest::Bodies { hashes } => {
				SyncRequester::request_bodies(sync, io, peer_id, hashes, block_set);
			},
			BlockRequest::Receipts { hashes } => {
				SyncRequester::request_receipts(sync, io, peer_id, hashes, block_set);
			},
		}
	}

	/// Request block bodies from a peer
	fn request_bodies(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId, hashes: Vec<H256>, set: BlockSet) {
		let mut rlp = RlpStream::new_list(hashes.len());
		trace!(target: "sync", "{} <- GetBlockBodies: {} entries starting from {:?}, set = {:?}", peer_id, hashes.len(), hashes.first(), set);
		for h in &hashes {
			rlp.append(&h.clone());
		}
		SyncRequester::send_request(sync, io, peer_id, PeerAsking::BlockBodies, GET_BLOCK_BODIES_PACKET, rlp.out());
		let peer = sync.peers.get_mut(&peer_id).expect("peer_id may originate either from on_packet, where it is already validated or from enumerating self.peers. qed");
		peer.asking_blocks = hashes;
		peer.block_set = Some(set);
	}

	/// Request headers from a peer by block number
	pub fn request_fork_header(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId, n: BlockNumber) {
		trace!(target: "sync", "{} <- GetForkHeader: at {}", peer_id, n);
		let mut rlp = RlpStream::new_list(4);
		rlp.append(&n);
		rlp.append(&1u32);
		rlp.append(&0u32);
		rlp.append(&0u32);
		SyncRequester::send_request(sync, io, peer_id, PeerAsking::ForkHeader, GET_BLOCK_HEADERS_PACKET, rlp.out());
	}

	/// Find some headers or blocks to download for a peer.
	pub fn request_snapshot_data(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId) {
		// find chunk data to download
		if let Some(hash) = sync.snapshot.needed_chunk() {
			if let Some(ref mut peer) = sync.peers.get_mut(&peer_id) {
				peer.asking_snapshot_data = Some(hash.clone());
			}
			SyncRequester::request_snapshot_chunk(sync, io, peer_id, &hash);
		}
	}

	/// Request snapshot manifest from a peer.
	pub fn request_snapshot_manifest(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId) {
		trace!(target: "sync", "{} <- GetSnapshotManifest", peer_id);
		let rlp = RlpStream::new_list(0);
		SyncRequester::send_request(sync, io, peer_id, PeerAsking::SnapshotManifest, GET_SNAPSHOT_MANIFEST_PACKET, rlp.out());
	}

	/// Request headers from a peer by block hash
	fn request_headers_by_hash(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId, h: &H256, count: u64, skip: u64, reverse: bool, set: BlockSet) {
		trace!(target: "sync", "{} <- GetBlockHeaders: {} entries starting from {}, set = {:?}", peer_id, count, h, set);
		let mut rlp = RlpStream::new_list(4);
		rlp.append(h);
		rlp.append(&count);
		rlp.append(&skip);
		rlp.append(&if reverse {1u32} else {0u32});
		SyncRequester::send_request(sync, io, peer_id, PeerAsking::BlockHeaders, GET_BLOCK_HEADERS_PACKET, rlp.out());
		let peer = sync.peers.get_mut(&peer_id).expect("peer_id may originate either from on_packet, where it is already validated or from enumerating self.peers. qed");
		peer.asking_hash = Some(h.clone());
		peer.block_set = Some(set);
	}

	/// Request block receipts from a peer
	fn request_receipts(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId, hashes: Vec<H256>, set: BlockSet) {
		let mut rlp = RlpStream::new_list(hashes.len());
		trace!(target: "sync", "{} <- GetBlockReceipts: {} entries starting from {:?}, set = {:?}", peer_id, hashes.len(), hashes.first(), set);
		for h in &hashes {
			rlp.append(&h.clone());
		}
		SyncRequester::send_request(sync, io, peer_id, PeerAsking::BlockReceipts, GET_RECEIPTS_PACKET, rlp.out());
		let peer = sync.peers.get_mut(&peer_id).expect("peer_id may originate either from on_packet, where it is already validated or from enumerating self.peers. qed");
		peer.asking_blocks = hashes;
		peer.block_set = Some(set);
	}

	/// Request snapshot chunk from a peer.
	fn request_snapshot_chunk(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId, chunk: &H256) {
		trace!(target: "sync", "{} <- GetSnapshotData {:?}", peer_id, chunk);
		let mut rlp = RlpStream::new_list(1);
		rlp.append(chunk);
		SyncRequester::send_request(sync, io, peer_id, PeerAsking::SnapshotData, GET_SNAPSHOT_DATA_PACKET, rlp.out());
	}

	/// Generic request sender
	fn send_request(sync: &mut ChainSync, io: &mut SyncIo, peer_id: PeerId, asking: PeerAsking,  packet_id: PacketId, packet: Bytes) {
		if let Some(ref mut peer) = sync.peers.get_mut(&peer_id) {
			if peer.asking != PeerAsking::Nothing {
				warn!(target:"sync", "Asking {:?} while requesting {:?}", peer.asking, asking);
			}
			peer.asking = asking;
			peer.ask_time = Instant::now();
			// TODO [ToDr] This seems quite fragile. Be careful when protocol is updated.
			let result = if packet_id >= ETH_PROTOCOL_VERSION_63.1 {
				io.send_protocol(WARP_SYNC_PROTOCOL_ID, peer_id, packet_id, packet)
			} else {
				io.send(peer_id, packet_id, packet)
			};
			if let Err(e) = result {
				debug!(target:"sync", "Error sending request: {:?}", e);
				io.disconnect_peer(peer_id);
			}
		}
	}
}
