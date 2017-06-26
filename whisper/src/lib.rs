// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Whisper messaging system as a DevP2P subprotocol and RPC interface.

extern crate ethcore_bigint as bigint;
extern crate ethcore_network as network;
extern crate parking_lot;
extern crate rlp;

use std::time::{self, Duration, SystemTime};

use bigint::{H32, H512};
use network::{NetworkContext, PeerId};
use rlp::{self, DecoderError, RlpStream, UntrustedRlp};

const PROTOCOL_ID: [u8; 3] = *b"shh";

struct Topic([u8; 4]);

impl Topic {
	fn bloom(&self) -> H512 {
		unimplemented!()
	}
}

impl rlp::Encodable for Topic {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&H32(self.0))
	}
}

impl rlp::Decodable for Topic {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		rlp.as_val::<H32>().map(|h| Topic(h.0))
	}
}

// raw envelope struct.
struct Envelope {
	expiry: u64,
	ttl: u64,
	topics: Vec<Topic>,
	data: Vec<u8>,
	nonce: u64,
}

impl rlp::Encodable for Envelope {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(5)
			.append(&self.expiry)
			.append(&self.ttl)
			.append_list(&self.topics)
			.append(&self.data)
			.append(&self.nonce)
	}
}

impl rlp::Decodable for Envelope {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		Ok(Envelope {
			expiry: rlp.val_at(0)?,
			ttl: rlp.val_at(1)?,
			topics: rlp.list_at(2)?,
			data: rlp.val_at(3)?,
			nonce: rlp.val_at(4)?,
		})
	}
}

struct Message {
	envelope: Envelope,
	bloom: H512,
}

/// The whisper network protocol handler.
pub struct Handler {

}

impl ::network::NetworkProtocolHandler for Handler {
	fn initialize(&self, _io: &NetworkContext) {
		// set up broadcast timer (< 1s)
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		// handle packet and punish peers.
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		// peer with higher ID should begin rallying.
	}

	fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
	}

	fn timeout(&self, _io: &NetworkContext, _timer: TimerToken) {
		// rally with each peer and handle timeouts.
	}
}

#[cfg(test)]
mod tests {
}
