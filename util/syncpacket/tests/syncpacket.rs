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

#[macro_use]
extern crate syncpacket;

const MY_PROTOCOL0: ProtocolId = *b"mp0";
const MY_PROTOCOL1: ProtocolId = *b"mp1";

#[derive(SyncPackets, Clone, Copy, Debug, PartialEq)]
enum Packets {
	#[protocol(MY_PROTOCOL0)] Packet0 = 0x00,
	#[protocol(MY_PROTOCOL1)] Packet1 = 0x01,
}

#[test]
fn packet_ids_from_u8_when_from_primitive_zero_then_equals_status_packet() {
	assert_eq!(Packets::from_u8(0x00), Some(Packets::Packet0));
}

#[test]
fn packet_ids_from_u8_when_from_primitive_eleven_then_equals_get_snapshot_manifest_packet() {
	assert_eq!(Packets::from_u8(0x01), Some(Packets::Packet1));
}

#[test]
fn packet_ids_from_u8_when_invalid_packet_id_then_none() {
	assert!(Packets::from_u8(0x99).is_none());
}

#[test]
fn when_status_packet_then_id_and_protocol_match() {
	assert_eq!(Packets::Packet0.id(), Packets::Packet0 as PacketId);
	assert_eq!(Packets::Packet0.protocol(), MY_PROTOCOL0);

	assert_eq!(Packets::Packet1.id(), Packets::Packet1 as PacketId);
	assert_eq!(Packets::Packet1.protocol(), MY_PROTOCOL1);
}
