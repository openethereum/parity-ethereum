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

use syncpacket::SyncPackets;
use crate::api::{ETH_PROTOCOL, WARP_SYNC_PROTOCOL_ID};

/// An enum that defines all known packet ids in the context of
/// synchronization and provides a mechanism to convert from
/// packet ids (of type PacketId or u8) directly read from the network
/// to enum variants. This implicitly provides a mechanism to
/// check whether a given packet id is known, and to prevent
/// packet id clashes when defining new ids.

#[derive(SyncPackets, Clone, Copy)]
pub enum SyncPacket {
	#[protocol(ETH_PROTOCOL)] StatusPacket = 0x00,
	#[protocol(ETH_PROTOCOL)] NewBlockHashesPacket = 0x01,
	#[protocol(ETH_PROTOCOL)] TransactionsPacket = 0x02,
	#[protocol(ETH_PROTOCOL)] GetBlockHeadersPacket = 0x03,
	#[protocol(ETH_PROTOCOL)] BlockHeadersPacket = 0x04,
	#[protocol(ETH_PROTOCOL)] GetBlockBodiesPacket = 0x05,
	#[protocol(ETH_PROTOCOL)] BlockBodiesPacket = 0x06,
	#[protocol(ETH_PROTOCOL)] NewBlockPacket = 0x07,

	#[protocol(ETH_PROTOCOL)] GetNodeDataPacket = 0x0d,
	#[protocol(ETH_PROTOCOL)] NodeDataPacket = 0x0e,
	#[protocol(ETH_PROTOCOL)] GetReceiptsPacket = 0x0f,
	#[protocol(ETH_PROTOCOL)] ReceiptsPacket = 0x10,

	#[protocol(WARP_SYNC_PROTOCOL_ID)] GetSnapshotManifestPacket = 0x11,
	#[protocol(WARP_SYNC_PROTOCOL_ID)] SnapshotManifestPacket = 0x12,
	#[protocol(WARP_SYNC_PROTOCOL_ID)] GetSnapshotDataPacket = 0x13,
	#[protocol(WARP_SYNC_PROTOCOL_ID)] SnapshotDataPacket = 0x14,
	#[protocol(WARP_SYNC_PROTOCOL_ID)] ConsensusDataPacket = 0x15,
	#[protocol(WARP_SYNC_PROTOCOL_ID)] PrivateTransactionPacket = 0x16,
	#[protocol(WARP_SYNC_PROTOCOL_ID)] SignedPrivateTransactionPacket = 0x17,
}
