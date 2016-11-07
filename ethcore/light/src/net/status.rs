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

//! Peer status and capabilities.

use rlp::{RlpStream, Stream, UntrustedRlp, View};
use util::{H256, U256};

/// Network ID structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NetworkId {
	/// ID for the mainnet
	Mainnet = 1,
	/// ID for the testnet
	Testnet = 0,
}

/// A peer status message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
	/// Protocol version.
	pub protocol_version: u32,
	/// Network id of this peer.
	pub network_id: NetworkId,
	/// Total difficulty of the head of the chain.
	pub head_td: U256,
	/// Hash of the best block.
	pub head_hash: H256,
	/// Number of the best block.
	pub head_num: u64,
	/// Genesis hash
	pub genesis_hash: Option<H256>,
	/// Last announced chain head and reorg depth to common ancestor.
	pub last_head: Option<(H256, u64)>,
}

/// Peer capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
	/// Whether this peer can serve headers
	pub serve_headers: bool,
	/// Earliest block number it can serve chain requests for.
	pub serve_chain_since: Option<u64>,
	/// Earliest block number it can serve state requests for.
	pub serve_state_since: Option<u64>,
	/// Whether it can relay transactions to the eth network.
	pub tx_relay: bool,
}

impl Default for Capabilities {
	fn default() -> Self {
		Capabilities {
			serve_headers: false,
			serve_chain_since: None,
			serve_state_since: None,
			tx_relay: false,
		}
	}
}

impl Capabilities {
	/// Decode capabilities from the given rlp stream, starting from the given
	/// index.
	fn decode_from(rlp: &UntrustedRlp, start_idx: usize) -> Result<Self, DecoderError> {
		let mut caps = Capabilities::default();

		for item in rlp.iter().skip(start_idx).take(4) {
			let key: String = try!(item.val_at(0));

			match &*key {
				"serveHeaders" => caps.serve_headers = true,
				"serveChainSince" => caps.serve_chain_since = Some(try!(item.val_at(1))),
				"serveStateSince" => caps.serve_state_since = Some(try!(item.val_at(1))),
				"txRelay" => caps.tx_relay = true,
				_ => continue,
			}
		}

		Ok(caps)
	}
}

/// An announcement of new chain head or capabilities made by a peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
	/// Hash of the best block.
	head_hash: H256,
	/// Number of the best block.
	head_num: u64,
	/// Head total difficulty
	head_td: U256,
	/// reorg depth to common ancestor of last announced head.
	reorg_depth: u64,
	/// updated capabilities.
	new_capabilities: Capabilities,
}