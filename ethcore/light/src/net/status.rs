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

//! Peer status and capabilities.

use ethereum_types::{H256, U256};
use rlp::{DecoderError, Encodable, Decodable, RlpStream, Rlp};

use super::request_credits::FlowParams;

// recognized handshake/announcement keys.
// unknown keys are to be skipped, known keys have a defined order.
// their string values are defined in the LES spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum Key {
	ProtocolVersion,
	NetworkId,
	HeadTD,
	HeadHash,
	HeadNum,
	GenesisHash,
	ServeHeaders,
	ServeChainSince,
	ServeStateSince,
	TxRelay,
	BufferLimit,
	BufferCostTable,
	BufferRechargeRate,
}

impl Key {
	// get the string value of this key.
	fn as_str(self) -> &'static str {
		match self {
			Key::ProtocolVersion => "protocolVersion",
			Key::NetworkId => "networkId",
			Key::HeadTD => "headTd",
			Key::HeadHash => "headHash",
			Key::HeadNum => "headNum",
			Key::GenesisHash => "genesisHash",
			Key::ServeHeaders => "serveHeaders",
			Key::ServeChainSince => "serveChainSince",
			Key::ServeStateSince => "serveStateSince",
			Key::TxRelay => "txRelay",
			Key::BufferLimit => "flowControl/BL",
			Key::BufferCostTable => "flowControl/MRC",
			Key::BufferRechargeRate => "flowControl/MRR",
		}
	}

	// try to parse the key value from a string.
	fn from_str(s: &str) -> Option<Self> {
		match s {
			"protocolVersion" => Some(Key::ProtocolVersion),
			"networkId" => Some(Key::NetworkId),
			"headTd" => Some(Key::HeadTD),
			"headHash" => Some(Key::HeadHash),
			"headNum" => Some(Key::HeadNum),
			"genesisHash" => Some(Key::GenesisHash),
			"serveHeaders" => Some(Key::ServeHeaders),
			"serveChainSince" => Some(Key::ServeChainSince),
			"serveStateSince" => Some(Key::ServeStateSince),
			"txRelay" => Some(Key::TxRelay),
			"flowControl/BL" => Some(Key::BufferLimit),
			"flowControl/MRC" => Some(Key::BufferCostTable),
			"flowControl/MRR" => Some(Key::BufferRechargeRate),
			_ => None
		}
	}
}

// helper for decoding key-value pairs in the handshake or an announcement.
struct Parser<'a> {
	pos: usize,
	rlp: &'a Rlp<'a>,
}

impl<'a> Parser<'a> {
	// expect a specific next key, and decode the value.
	// error on unexpected key or invalid value.
	fn expect<T: Decodable>(&mut self, key: Key) -> Result<T, DecoderError> {
		self.expect_raw(key).and_then(|item| item.as_val())
	}

	// expect a specific next key, and get the value's RLP.
	// if the key isn't found, the position isn't advanced.
	fn expect_raw(&mut self, key: Key) -> Result<Rlp<'a>, DecoderError> {
		trace!(target: "les", "Expecting key {}", key.as_str());
		let pre_pos = self.pos;
		if let Some((k, val)) = self.get_next()? {
			if k == key { return Ok(val) }
		}

		self.pos = pre_pos;
		Err(DecoderError::Custom("Missing expected key"))
	}

	// get the next key and value RLP.
	fn get_next(&mut self) -> Result<Option<(Key, Rlp<'a>)>, DecoderError> {
		while self.pos < self.rlp.item_count()? {
			let pair = self.rlp.at(self.pos)?;
			let k: String = pair.val_at(0)?;

			self.pos += 1;
			match Key::from_str(&k) {
				Some(key) => return Ok(Some((key , pair.at(1)?))),
				None => continue,
			}
		}

		Ok(None)
	}
}

// Helper for encoding a key-value pair
fn encode_pair<T: Encodable>(key: Key, val: &T) -> Vec<u8> {
	let mut s = RlpStream::new_list(2);
	s.append(&key.as_str()).append(val);
	s.out()
}

// Helper for encoding a flag.
fn encode_flag(key: Key) -> Vec<u8> {
	let mut s = RlpStream::new_list(2);
	s.append(&key.as_str()).append_empty_data();
	s.out()
}

/// A peer status message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
	/// Protocol version.
	pub protocol_version: u32,
	/// Network id of this peer.
	pub network_id: u64,
	/// Total difficulty of the head of the chain.
	pub head_td: U256,
	/// Hash of the best block.
	pub head_hash: H256,
	/// Number of the best block.
	pub head_num: u64,
	/// Genesis hash
	pub genesis_hash: H256,
	/// Last announced chain head and reorg depth to common ancestor.
	pub last_head: Option<(H256, u64)>,
}

impl Status {
	/// Update the status from an announcement.
	pub fn update_from(&mut self, announcement: &Announcement) {
		self.last_head = Some((self.head_hash, announcement.reorg_depth));
		self.head_td = announcement.head_td;
		self.head_hash = announcement.head_hash;
		self.head_num = announcement.head_num;
	}
}

/// Peer capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
	/// Whether this peer can serve headers
	pub serve_headers: bool,
	/// Earliest block number it can serve block/receipt requests for.
	/// `None` means no requests will be servable.
	pub serve_chain_since: Option<u64>,
	/// Earliest block number it can serve state requests for.
	/// `None` means no requests will be servable.
	pub serve_state_since: Option<u64>,
	/// Whether it can relay transactions to the eth network.
	pub tx_relay: bool,
}

impl Default for Capabilities {
	fn default() -> Self {
		Capabilities {
			serve_headers: true,
			serve_chain_since: None,
			serve_state_since: None,
			tx_relay: false,
		}
	}
}

impl Capabilities {
	/// Update the capabilities from an announcement.
	pub fn update_from(&mut self, announcement: &Announcement) {
		self.serve_headers = self.serve_headers || announcement.serve_headers;
		self.serve_state_since = self.serve_state_since.or(announcement.serve_state_since);
		self.serve_chain_since = self.serve_chain_since.or(announcement.serve_chain_since);
		self.tx_relay = self.tx_relay || announcement.tx_relay;
	}
}

/// Attempt to parse a handshake message into its three parts:
///   - chain status
///   - serving capabilities
///   - request credit parameters
pub fn parse_handshake(rlp: &Rlp) -> Result<(Status, Capabilities, Option<FlowParams>), DecoderError> {
	let mut parser = Parser {
		pos: 0,
		rlp,
	};

	let status = Status {
		protocol_version: parser.expect(Key::ProtocolVersion)?,
		network_id: parser.expect(Key::NetworkId)?,
		head_td: parser.expect(Key::HeadTD)?,
		head_hash: parser.expect(Key::HeadHash)?,
		head_num: parser.expect(Key::HeadNum)?,
		genesis_hash: parser.expect(Key::GenesisHash)?,
		last_head: None,
	};

	let capabilities = Capabilities {
		serve_headers: parser.expect_raw(Key::ServeHeaders).is_ok(),
		serve_chain_since: parser.expect(Key::ServeChainSince).ok(),
		serve_state_since: parser.expect(Key::ServeStateSince).ok(),
		tx_relay: parser.expect_raw(Key::TxRelay).is_ok(),
	};

	let flow_params = match (
		parser.expect(Key::BufferLimit),
		parser.expect(Key::BufferCostTable),
		parser.expect(Key::BufferRechargeRate)
	) {
		(Ok(bl), Ok(bct), Ok(brr)) => Some(FlowParams::new(bl, bct, brr)),
		_ => None,
	};

	Ok((status, capabilities, flow_params))
}

/// Write a handshake, given status, capabilities, and flow parameters.
pub fn write_handshake(status: &Status, capabilities: &Capabilities, flow_params: Option<&FlowParams>) -> Vec<u8> {
	let mut pairs = Vec::new();
	pairs.push(encode_pair(Key::ProtocolVersion, &status.protocol_version));
	pairs.push(encode_pair(Key::NetworkId, &(status.network_id as u64)));
	pairs.push(encode_pair(Key::HeadTD, &status.head_td));
	pairs.push(encode_pair(Key::HeadHash, &status.head_hash));
	pairs.push(encode_pair(Key::HeadNum, &status.head_num));
	pairs.push(encode_pair(Key::GenesisHash, &status.genesis_hash));

	if capabilities.serve_headers {
		pairs.push(encode_flag(Key::ServeHeaders));
	}
	if let Some(ref serve_chain_since) = capabilities.serve_chain_since {
		pairs.push(encode_pair(Key::ServeChainSince, serve_chain_since));
	}
	if let Some(ref serve_state_since) = capabilities.serve_state_since {
		pairs.push(encode_pair(Key::ServeStateSince, serve_state_since));
	}
	if capabilities.tx_relay {
		pairs.push(encode_flag(Key::TxRelay));
	}

	if let Some(flow_params) = flow_params {
		pairs.push(encode_pair(Key::BufferLimit, flow_params.limit()));
		pairs.push(encode_pair(Key::BufferCostTable, flow_params.cost_table()));
		pairs.push(encode_pair(Key::BufferRechargeRate, flow_params.recharge_rate()));
	}

	let mut stream = RlpStream::new_list(pairs.len());

	for pair in pairs {
		stream.append_raw(&pair, 1);
	}

	stream.out()
}

/// An announcement of new chain head or capabilities made by a peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
	/// Hash of the best block.
	pub head_hash: H256,
	/// Number of the best block.
	pub head_num: u64,
	/// Head total difficulty
	pub head_td: U256,
	/// reorg depth to common ancestor of last announced head.
	pub reorg_depth: u64,
	/// optional new header-serving capability. false means "no change"
	pub serve_headers: bool,
	/// optional new state-serving capability
	pub serve_state_since: Option<u64>,
	/// optional new chain-serving capability
	pub serve_chain_since: Option<u64>,
	/// optional new transaction-relay capability. false means "no change"
	pub tx_relay: bool,
	// TODO: changes in request credits.
}

/// Parse an announcement.
pub fn parse_announcement(rlp: &Rlp) -> Result<Announcement, DecoderError> {
	let mut last_key = None;

	let mut announcement = Announcement {
		head_hash: rlp.val_at(0)?,
		head_num: rlp.val_at(1)?,
		head_td: rlp.val_at(2)?,
		reorg_depth: rlp.val_at(3)?,
		serve_headers: false,
		serve_state_since: None,
		serve_chain_since: None,
		tx_relay: false,
	};

	let mut parser = Parser {
		pos: 4,
		rlp,
	};

	while let Some((key, item)) = parser.get_next()? {
		if Some(key) <= last_key { return Err(DecoderError::Custom("Invalid announcement key ordering")) }
		last_key = Some(key);

		match key {
			Key::ServeHeaders => announcement.serve_headers = true,
			Key::ServeStateSince => announcement.serve_state_since = Some(item.as_val()?),
			Key::ServeChainSince => announcement.serve_chain_since = Some(item.as_val()?),
			Key::TxRelay => announcement.tx_relay = true,
			_ => return Err(DecoderError::Custom("Nonsensical key in announcement")),
		}
	}

	Ok(announcement)
}

/// Write an announcement out.
pub fn write_announcement(announcement: &Announcement) -> Vec<u8> {
	let mut pairs = Vec::new();
	if announcement.serve_headers {
		pairs.push(encode_flag(Key::ServeHeaders));
	}
	if let Some(ref serve_chain_since) = announcement.serve_chain_since {
		pairs.push(encode_pair(Key::ServeChainSince, serve_chain_since));
	}
	if let Some(ref serve_state_since) = announcement.serve_state_since {
		pairs.push(encode_pair(Key::ServeStateSince, serve_state_since));
	}
	if announcement.tx_relay {
		pairs.push(encode_flag(Key::TxRelay));
	}

	let mut stream = RlpStream::new_list(4 + pairs.len());
	stream
		.append(&announcement.head_hash)
		.append(&announcement.head_num)
		.append(&announcement.head_td)
		.append(&announcement.reorg_depth);

	for item in pairs {
		stream.append_raw(&item, 1);
	}

	stream.out()
}

#[cfg(test)]
mod tests {
	use super::*;
	use super::super::request_credits::FlowParams;
	use ethereum_types::{U256, H256};
	use rlp::{RlpStream, Rlp};

	#[test]
	fn full_handshake() {
		let status = Status {
			protocol_version: 1,
			network_id: 1,
			head_td: U256::default(),
			head_hash: H256::default(),
			head_num: 10,
			genesis_hash: H256::zero(),
			last_head: None,
		};

		let capabilities = Capabilities {
			serve_headers: true,
			serve_chain_since: Some(5),
			serve_state_since: Some(8),
			tx_relay: true,
		};

		let flow_params = FlowParams::new(
			1_000_000.into(),
			Default::default(),
			1000.into(),
		);

		let handshake = write_handshake(&status, &capabilities, Some(&flow_params));

		let (read_status, read_capabilities, read_flow)
			= parse_handshake(&Rlp::new(&handshake)).unwrap();

		assert_eq!(read_status, status);
		assert_eq!(read_capabilities, capabilities);
		assert_eq!(read_flow.unwrap(), flow_params);
	}

	#[test]
	fn partial_handshake() {
		let status = Status {
			protocol_version: 1,
			network_id: 1,
			head_td: U256::default(),
			head_hash: H256::default(),
			head_num: 10,
			genesis_hash: H256::zero(),
			last_head: None,
		};

		let capabilities = Capabilities {
			serve_headers: false,
			serve_chain_since: Some(5),
			serve_state_since: None,
			tx_relay: true,
		};

		let flow_params = FlowParams::new(
			1_000_000.into(),
			Default::default(),
			1000.into(),
		);

		let handshake = write_handshake(&status, &capabilities, Some(&flow_params));

		let (read_status, read_capabilities, read_flow)
			= parse_handshake(&Rlp::new(&handshake)).unwrap();

		assert_eq!(read_status, status);
		assert_eq!(read_capabilities, capabilities);
		assert_eq!(read_flow.unwrap(), flow_params);
	}

	#[test]
	fn skip_unknown_keys() {
		let status = Status {
			protocol_version: 1,
			network_id: 1,
			head_td: U256::default(),
			head_hash: H256::default(),
			head_num: 10,
			genesis_hash: H256::zero(),
			last_head: None,
		};

		let capabilities = Capabilities {
			serve_headers: false,
			serve_chain_since: Some(5),
			serve_state_since: None,
			tx_relay: true,
		};

		let flow_params = FlowParams::new(
			1_000_000.into(),
			Default::default(),
			1000.into(),
		);

		let handshake = write_handshake(&status, &capabilities, Some(&flow_params));
		let interleaved = {
			let handshake = Rlp::new(&handshake);
			let mut stream = RlpStream::new_list(handshake.item_count().unwrap_or(0) * 3);

			for item in handshake.iter() {
				stream.append_raw(item.as_raw(), 1);
				let (mut s1, mut s2) = (RlpStream::new_list(2), RlpStream::new_list(2));
				s1.append(&"foo").append_empty_data();
				s2.append(&"bar").append_empty_data();
				stream.append_raw(&s1.out(), 1);
				stream.append_raw(&s2.out(), 1);
			}

			stream.out()
		};

		let (read_status, read_capabilities, read_flow)
			= parse_handshake(&Rlp::new(&interleaved)).unwrap();

		assert_eq!(read_status, status);
		assert_eq!(read_capabilities, capabilities);
		assert_eq!(read_flow.unwrap(), flow_params);
	}

	#[test]
	fn announcement_roundtrip() {
		let announcement = Announcement {
			head_hash: H256::random(),
			head_num: 100_000,
			head_td: 1_000_000.into(),
			reorg_depth: 4,
			serve_headers: false,
			serve_state_since: Some(99_000),
			serve_chain_since: Some(1),
			tx_relay: true,
		};

		let serialized = write_announcement(&announcement);
		let read = parse_announcement(&Rlp::new(&serialized)).unwrap();

		assert_eq!(read, announcement);
	}

	#[test]
	fn keys_out_of_order() {
		use super::{Key, encode_pair, encode_flag};

		let mut stream = RlpStream::new_list(6);
		stream
			.append(&H256::zero())
			.append(&10_u64)
			.append(&100_000_u64)
			.append(&2_u64)
			.append_raw(&encode_pair(Key::ServeStateSince, &44_u64), 1)
			.append_raw(&encode_flag(Key::ServeHeaders), 1);

		let out = stream.drain();
		assert!(parse_announcement(&Rlp::new(&out)).is_err());

		let mut stream = RlpStream::new_list(6);
		stream
			.append(&H256::zero())
			.append(&10_u64)
			.append(&100_000_u64)
			.append(&2_u64)
			.append_raw(&encode_flag(Key::ServeHeaders), 1)
			.append_raw(&encode_pair(Key::ServeStateSince, &44_u64), 1);

		let out = stream.drain();
		assert!(parse_announcement(&Rlp::new(&out)).is_ok());
	}

	#[test]
	fn optional_flow() {
		let status = Status {
			protocol_version: 1,
			network_id: 1,
			head_td: U256::default(),
			head_hash: H256::default(),
			head_num: 10,
			genesis_hash: H256::zero(),
			last_head: None,
		};

		let capabilities = Capabilities {
			serve_headers: true,
			serve_chain_since: Some(5),
			serve_state_since: Some(8),
			tx_relay: true,
		};

		let handshake = write_handshake(&status, &capabilities, None);

		let (read_status, read_capabilities, read_flow)
			= parse_handshake(&Rlp::new(&handshake)).unwrap();

		assert_eq!(read_status, status);
		assert_eq!(read_capabilities, capabilities);
		assert!(read_flow.is_none());
	}
}
