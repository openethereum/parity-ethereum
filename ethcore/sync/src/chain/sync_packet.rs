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

//! When sending packets over p2p we specify both which subprotocol
//! to use and what kind of packet we are sending (through a packet id).
//! Likewise when receiving packets from other peers we decode the
//! subprotocol and the packet id. This module helps coupling both
//! pieces of information together and provides an easy mechanism
//! to convert to/from the packet id values transmitted over the
//! wire.

/// An enum that defines all known packet ids in the context of
/// synchronization and provides a mechanism to convert from
/// packet ids (of type PacketId or u8) directly read from the network
/// to enum variants. This implicitly provides a mechanism to
/// check whether a given packet id is known, and to prevent
/// packet id clashes when defining new ids.
#[derive(SyncPackets, Clone, Copy, Debug, PartialEq)]
pub enum SyncPacket {
	#[eth] StatusPacket = 0x00,
	#[eth] NewBlockHashesPacket = 0x01,
	#[eth] TransactionsPacket = 0x02,
	#[eth] GetBlockHeadersPacket = 0x03,
	#[eth] BlockHeadersPacket = 0x04,
	#[eth] GetBlockBodiesPacket = 0x05,
	#[eth] BlockBodiesPacket = 0x06,
	#[eth] NewBlockPacket = 0x07,

	#[eth] GetNodeDataPacket = 0x0d,
	#[eth] NodeDataPacket = 0x0e,
	#[eth] GetReceiptsPacket = 0x0f,
	#[eth] ReceiptsPacket = 0x10,

	#[par] GetSnapshotManifestPacket = 0x11,
	#[par] SnapshotManifestPacket = 0x12,
	#[par] GetSnapshotDataPacket = 0x13,
	#[par] SnapshotDataPacket = 0x14,
	#[par] ConsensusDataPacket = 0x15,
	#[par] PrivateTransactionPacket = 0x16,
	#[par] SignedPrivateTransactionPacket = 0x17,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn packet_ids_from_u8_when_from_primitive_zero_then_equals_status_packet() {
		assert_eq!(SyncPacket::from_u8(0x00), Some(StatusPacket));
	}

	#[test]
	fn packet_ids_from_u8_when_from_primitive_eleven_then_equals_get_snapshot_manifest_packet() {
		assert_eq!(SyncPacket::from_u8(0x11), Some(GetSnapshotManifestPacket));
	}

	#[test]
	fn packet_ids_from_u8_when_invalid_packet_id_then_none() {
		assert!(SyncPacket::from_u8(0x99).is_none());
	}

	#[test]
	fn when_status_packet_then_id_and_protocol_match() {
		assert_eq!(StatusPacket.id(), StatusPacket as PacketId);
		assert_eq!(StatusPacket.protocol(), ETH_PROTOCOL);
	}

	#[test]
	fn when_consensus_data_packet_then_id_and_protocol_match() {
		assert_eq!(ConsensusDataPacket.id(), ConsensusDataPacket as PacketId);
		assert_eq!(ConsensusDataPacket.protocol(), WARP_SYNC_PROTOCOL_ID);
	}
}
