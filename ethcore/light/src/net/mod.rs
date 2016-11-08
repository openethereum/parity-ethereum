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

//! LES Protocol Version 1 implementation.
//!
//! This uses a "Provider" to answer requests.
//! See https://github.com/ethcore/parity/wiki/Light-Ethereum-Subprotocol-(LES)

use io::TimerToken;
use network::{NetworkProtocolHandler, NetworkService, NetworkContext, NetworkError, PeerId};
use rlp::{DecoderError, RlpStream, Stream, UntrustedRlp, View};
use util::hash::H256;
use util::{U256, Mutex, RwLock};

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use provider::Provider;
use request::{self, Request};
use self::buffer_flow::FlowParams;

mod buffer_flow;
mod status;

const TIMEOUT: TimerToken = 0;
const TIMEOUT_INTERVAL_MS: u64 = 1000;

// LPV1
const PROTOCOL_VERSION: u32 = 1;

// TODO [rob] make configurable.
const PROTOCOL_ID: [u8; 3] = *b"les";

// packet ID definitions.
mod packet {
	// the status packet.
	pub const STATUS: u8 = 0x00;

	// announcement of new block hashes or capabilities.
	pub const ANNOUNCE: u8 = 0x01;

	// request and response for block headers
	pub const GET_BLOCK_HEADERS: u8 = 0x02;
	pub const BLOCK_HEADERS: u8 = 0x03;

	// request and response for block bodies
	pub const GET_BLOCK_BODIES: u8 = 0x04;
	pub const BLOCK_BODIES: u8 = 0x05;

	// request and response for transaction receipts.
	pub const GET_RECEIPTS: u8 = 0x06;
	pub const RECEIPTS: u8 = 0x07;

	// request and response for merkle proofs.
	pub const GET_PROOFS: u8 = 0x08;
	pub const PROOFS: u8 = 0x09;

	// request and response for contract code.
	pub const GET_CONTRACT_CODES: u8 = 0x0a;
	pub const CONTRACT_CODES: u8 = 0x0b;

	// relay transactions to peers.
	pub const SEND_TRANSACTIONS: u8 = 0x0c;

	// request and response for header proofs in a CHT.
	pub const GET_HEADER_PROOFS: u8 = 0x0d;
	pub const HEADER_PROOFS: u8 = 0x0e;
}

// data about each peer.
struct Peer {
	buffer: u64, // remaining buffer value.
	current_asking: HashSet<usize>, // pending request ids.
}

/// This is an implementation of the light ethereum network protocol, abstracted
/// over a `Provider` of data and a p2p network.
///
/// This is simply designed for request-response purposes. Higher level uses
/// of the protocol, such as synchronization, will function as wrappers around
/// this system.
pub struct LightProtocol {
	provider: Box<Provider>,
	genesis_hash: H256,
	mainnet: bool,
	peers: RwLock<HashMap<PeerId, Peer>>,
	pending_requests: RwLock<HashMap<usize, Request>>,
	req_id: AtomicUsize,
}

impl LightProtocol {
	// make a request to a given peer.
	fn request_from(&self, peer: &PeerId, req: Request) {
		unimplemented!()
	}

	// called when a peer connects.
	fn on_connect(&self, peer: &PeerId, io: &NetworkContext) {
		let peer = *peer;
		match self.send_status(peer, io) {
			Ok(()) => {
				self.peers.write().insert(peer, Peer {
					buffer: 0,
					current_asking: HashSet::new(),
				});
			}
			Err(e) => {
				trace!(target: "les", "Error while sending status: {}", e);
				io.disable_peer(peer);
			}
		}
	}

	// called when a peer disconnects.
	fn on_disconnect(&self, peer: PeerId, io: &NetworkContext) {
		// TODO: reassign all requests assigned to this peer.
		self.peers.write().remove(&peer);
	}

	fn send_status(&self, peer: PeerId, io: &NetworkContext) -> Result<(), NetworkError> {
		let chain_info = self.provider.chain_info();

		// TODO [rob] use optional keys too.
		let mut stream = RlpStream::new_list(6);
		stream
			.begin_list(2)
				.append(&"protocolVersion")
				.append(&PROTOCOL_VERSION)
			.begin_list(2)
				.append(&"networkId")
				.append(&(self.mainnet as u8))
			.begin_list(2)
				.append(&"headTd")
				.append(&chain_info.total_difficulty)
			.begin_list(2)
				.append(&"headHash")
				.append(&chain_info.best_block_hash)
			.begin_list(2)
				.append(&"headNum")
				.append(&chain_info.best_block_number)
			.begin_list(2)
				.append(&"genesisHash")
				.append(&self.genesis_hash);

		io.send(peer, packet::STATUS, stream.out())
	}

	/// Check on the status of all pending requests.
	fn check_pending_requests(&self) {
		unimplemented!()
	}

	fn status(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Handle an announcement.
	fn announcement(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		const MAX_NEW_HASHES: usize = 256;

		unimplemented!()
	}

	// Handle a request for block headers.
	fn get_block_headers(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		const MAX_HEADERS: u64 = 512;

		unimplemented!()
	}

	// Receive a response for block headers.
	fn block_headers(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Handle a request for block bodies.
	fn get_block_bodies(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		const MAX_BODIES: usize = 512;

		unimplemented!()
	}

	// Receive a response for block bodies.
	fn block_bodies(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Handle a request for receipts.
	fn get_receipts(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Receive a response for receipts.
	fn receipts(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Handle a request for proofs.
	fn get_proofs(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Receive a response for proofs.
	fn proofs(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Handle a request for contract code.
	fn get_contract_code(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Receive a response for contract code.
	fn contract_code(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Handle a request for header proofs
	fn get_header_proofs(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Receive a response for header proofs
	fn header_proofs(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}

	// Receive a set of transactions to relay.
	fn relay_transactions(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) {
		unimplemented!()
	}
}

impl NetworkProtocolHandler for LightProtocol {
	fn initialize(&self, io: &NetworkContext) {
		io.register_timer(TIMEOUT, TIMEOUT_INTERVAL_MS).expect("Error registering sync timer.");
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		let rlp = UntrustedRlp::new(data);
		match packet_id {
			packet::STATUS => self.status(peer, io, rlp),
			packet::ANNOUNCE => self.announcement(peer, io, rlp),

			packet::GET_BLOCK_HEADERS => self.get_block_headers(peer, io, rlp),
			packet::BLOCK_HEADERS => self.block_headers(peer, io, rlp),

			packet::GET_BLOCK_BODIES => self.get_block_bodies(peer, io, rlp),
			packet::BLOCK_BODIES => self.block_bodies(peer, io, rlp),

			packet::GET_RECEIPTS => self.get_receipts(peer, io, rlp),
			packet::RECEIPTS => self.receipts(peer, io, rlp),

			packet::GET_PROOFS => self.get_proofs(peer, io, rlp),
			packet::PROOFS => self.proofs(peer, io, rlp),

			packet::GET_CONTRACT_CODES => self.get_contract_code(peer, io, rlp),
			packet::CONTRACT_CODES => self.contract_code(peer, io, rlp),

			packet::SEND_TRANSACTIONS => self.relay_transactions(peer, io, rlp),

			other => {
				debug!(target: "les", "Disconnecting peer {} on unexpected packet {}", peer, other);
				io.disconnect_peer(*peer);
			}
		}
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		self.on_connect(peer, io);
	}

	fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
		self.on_disconnect(*peer, io);
	}

	fn timeout(&self, io: &NetworkContext, timer: TimerToken) {
		match timer {
			TIMEOUT => {
				// broadcast transactions to peers.
			}
			_ => warn!(target: "les", "received timeout on unknown token {}", timer),
		}
	}
}