// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! I/O and event context generalizations.

use network::{NetworkContext, PeerId, NodeId};

use super::{Announcement, LightProtocol, ReqId};
use super::error::Error;
use request::NetworkRequests as Requests;

/// An I/O context which allows sending and receiving packets as well as
/// disconnecting peers. This is used as a generalization of the portions
/// of a p2p network which the light protocol structure makes use of.
pub trait IoContext {
	/// Send a packet to a specific peer.
	fn send(&self, peer: PeerId, packet_id: u8, packet_body: Vec<u8>);

	/// Respond to a peer's message. Only works if this context is a byproduct
	/// of a packet handler.
	fn respond(&self, packet_id: u8, packet_body: Vec<u8>);

	/// Disconnect a peer.
	fn disconnect_peer(&self, peer: PeerId);

	/// Disable a peer -- this is a disconnect + a time-out.
	fn disable_peer(&self, peer: PeerId);

	/// Get a peer's protocol version.
	fn protocol_version(&self, peer: PeerId) -> Option<u8>;

	/// Persistent peer id
	fn persistent_peer_id(&self, peer: PeerId) -> Option<NodeId>;

	/// Whether given peer id is reserved peer
	fn is_reserved_peer(&self, peer: PeerId) -> bool;
}

impl<T> IoContext for T where T: ?Sized + NetworkContext {
	fn send(&self, peer: PeerId, packet_id: u8, packet_body: Vec<u8>) {
		if let Err(e) = self.send(peer, packet_id, packet_body) {
			debug!(target: "pip", "Error sending packet to peer {}: {}", peer, e);
		}
	}

	fn respond(&self, packet_id: u8, packet_body: Vec<u8>) {
		if let Err(e) = self.respond(packet_id, packet_body) {
			debug!(target: "pip", "Error responding to peer message: {}", e);
		}
	}

	fn disconnect_peer(&self, peer: PeerId) {
		trace!(target: "pip", "Initiating disconnect of peer {}", peer);
		NetworkContext::disconnect_peer(self, peer);
	}

	fn disable_peer(&self, peer: PeerId) {
		trace!(target: "pip", "Initiating disable of peer {}", peer);
		NetworkContext::disable_peer(self, peer);
	}

	fn protocol_version(&self, peer: PeerId) -> Option<u8> {
		self.protocol_version(self.subprotocol_name(), peer)
	}

	fn persistent_peer_id(&self, peer: PeerId) -> Option<NodeId> {
		self.session_info(peer).and_then(|info| info.id)
	}

	fn is_reserved_peer(&self, peer: PeerId) -> bool {
		NetworkContext::is_reserved_peer(self, peer)
	}
}

/// Basic context for the protocol.
pub trait BasicContext {
	/// Returns the relevant's peer persistent Id (aka NodeId).
	fn persistent_peer_id(&self, peer: PeerId) -> Option<NodeId>;

	/// Make a request from a peer.
	///
	/// Fails on: nonexistent peer, network error, peer not server,
	/// insufficient credits. Does not check capabilities before sending.
	/// On success, returns a request id which can later be coordinated
	/// with an event.
	fn request_from(&self, peer: PeerId, request: Requests) -> Result<ReqId, Error>;

	/// Make an announcement of new capabilities to the rest of the peers.
	// TODO: maybe just put this on a timer in LightProtocol?
	fn make_announcement(&self, announcement: Announcement);

	/// Disconnect a peer.
	fn disconnect_peer(&self, peer: PeerId);

	/// Disable a peer.
	fn disable_peer(&self, peer: PeerId);
}

/// Context for a protocol event which has a peer ID attached.
pub trait EventContext: BasicContext {
	/// Get the peer relevant to the event e.g. message sender,
	/// disconnected/connected peer.
	fn peer(&self) -> PeerId;

	/// Treat the event context as a basic context.
	fn as_basic(&self) -> &dyn BasicContext;
}

/// Basic context.
pub struct TickCtx<'a> {
	/// Io context to enable dispatch.
	pub io: &'a dyn IoContext,
	/// Protocol implementation.
	pub proto: &'a LightProtocol,
}

impl<'a> BasicContext for TickCtx<'a> {
	fn persistent_peer_id(&self, id: PeerId) -> Option<NodeId> {
		self.io.persistent_peer_id(id)
	}

	fn request_from(&self, peer: PeerId, requests: Requests) -> Result<ReqId, Error> {
		self.proto.request_from(self.io, peer, requests)
	}

	fn make_announcement(&self, announcement: Announcement) {
		self.proto.make_announcement(self.io, announcement);
	}

	fn disconnect_peer(&self, peer: PeerId) {
		self.io.disconnect_peer(peer);
	}

	fn disable_peer(&self, peer: PeerId) {
		self.io.disable_peer(peer);
	}
}

/// Concrete implementation of `EventContext` over the light protocol struct and
/// an io context.
pub struct Ctx<'a> {
	/// Io context to enable immediate response to events.
	pub io: &'a dyn IoContext,
	/// Protocol implementation.
	pub proto: &'a LightProtocol,
	/// Relevant peer for event.
	pub peer: PeerId,
}

impl<'a> BasicContext for Ctx<'a> {
	fn persistent_peer_id(&self, id: PeerId) -> Option<NodeId> {
		self.io.persistent_peer_id(id)
	}

	fn request_from(&self, peer: PeerId, requests: Requests) -> Result<ReqId, Error> {
		self.proto.request_from(self.io, peer, requests)
	}

	fn make_announcement(&self, announcement: Announcement) {
		self.proto.make_announcement(self.io, announcement);
	}

	fn disconnect_peer(&self, peer: PeerId) {
		self.io.disconnect_peer(peer);
	}

	fn disable_peer(&self, peer: PeerId) {
		self.io.disable_peer(peer);
	}
}

impl<'a> EventContext for Ctx<'a> {
	fn peer(&self) -> PeerId {
		self.peer
	}

	fn as_basic(&self) -> &dyn BasicContext {
		&*self
	}
}
