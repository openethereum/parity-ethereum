use api::{ETH_PROTOCOL, WARP_SYNC_PROTOCOL_ID};
use network::{PacketId, ProtocolId};

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SyncPacketId {
	StatusPacket = 0x00,
	NewBlockHashesPacket = 0x01,
	TransactionsPacket = 0x02,
	GetBlockHeadersPacket = 0x03,
	BlockHeadersPacket = 0x04,
	GetBlockBodiesPacket = 0x05,
	BlockBodiesPacket = 0x06,
	NewBlockPacket = 0x07,

	GetNodeDataPacket = 0x0d,
	NodeDataPacket = 0x0e,
	GetReceiptsPacket = 0x0f,
	ReceiptsPacket = 0x10,

	GetSnapshotManifestPacket = 0x11,
	SnapshotManifestPacket = 0x12,
	GetSnapshotDataPacket = 0x13,
	SnapshotDataPacket = 0x14,
	ConsensusDataPacket = 0x15,
	PrivateTransactionPacket = 0x16,
	SignedPrivateTransactionPacket = 0x17,
}
}


use self::SyncPacketId::*;

pub trait PacketInfo {
	fn id(&self) -> PacketId;
	fn protocol(&self) -> ProtocolId;
}

impl PacketInfo for SyncPacketId {
	fn protocol(&self) -> ProtocolId {
		match self {
			StatusPacket |
			NewBlockHashesPacket |
			TransactionsPacket |
			GetBlockHeadersPacket |
			BlockHeadersPacket |
			GetBlockBodiesPacket |
			BlockBodiesPacket |
			NewBlockPacket |

			GetNodeDataPacket|
			NodeDataPacket |
			GetReceiptsPacket |
			ReceiptsPacket

				=> ETH_PROTOCOL,

			GetSnapshotManifestPacket|
			SnapshotManifestPacket |
			GetSnapshotDataPacket |
			SnapshotDataPacket |
			ConsensusDataPacket |
			PrivateTransactionPacket |
			SignedPrivateTransactionPacket

				=> WARP_SYNC_PROTOCOL_ID,
		}
	}

	fn id(&self) -> PacketId {
		(*self) as PacketId
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	use enum_primitive::FromPrimitive;

	#[test]
	fn packet_ids_from_u8_when_from_primitive_zero_then_equals_status_packet() {
		assert_eq!(SyncPacketId::from_u8(0x00), Some(StatusPacket));
	}

	#[test]
	fn packet_ids_from_u8_when_from_primitive_eleven_then_equals_get_snapshot_manifest_packet() {
		assert_eq!(SyncPacketId::from_u8(0x11), Some(GetSnapshotManifestPacket));
	}

	#[test]
	fn packet_ids_from_u8_when_invalid_packet_id_then_none() {
		assert!(SyncPacketId::from_u8(0x99).is_none());
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
