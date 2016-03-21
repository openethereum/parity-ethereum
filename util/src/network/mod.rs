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

//! Network and general IO module. 
//! 
//! Example usage for craeting a network service and adding an IO handler:
//!
//! ```rust
//! extern crate ethcore_util as util;
//! use util::*;
//!
//! struct MyHandler;
//!
//! #[derive(Clone)]
//! struct MyMessage {
//! 	data: u32
//! }
//!
//! impl NetworkProtocolHandler<MyMessage> for MyHandler {
//!		fn initialize(&self, io: &NetworkContext<MyMessage>) {
//!			io.register_timer(0, 1000);
//!		}
//!
//!		fn read(&self, io: &NetworkContext<MyMessage>, peer: &PeerId, packet_id: u8, data: &[u8]) {
//!			println!("Received {} ({} bytes) from {}", packet_id, data.len(), peer);
//!		}
//!
//!		fn connected(&self, io: &NetworkContext<MyMessage>, peer: &PeerId) {
//!			println!("Connected {}", peer);
//!		}
//!
//!		fn disconnected(&self, io: &NetworkContext<MyMessage>, peer: &PeerId) {
//!			println!("Disconnected {}", peer);
//!		}
//!
//!		fn timeout(&self, io: &NetworkContext<MyMessage>, timer: TimerToken) {
//!			println!("Timeout {}", timer);
//!		}
//!
//!		fn message(&self, io: &NetworkContext<MyMessage>, message: &MyMessage) {
//!			println!("Message {}", message.data);
//!		}
//! }
//!
//! fn main () {
//! 	let mut service = NetworkService::<MyMessage>::start(NetworkConfiguration::new_local()).expect("Error creating network service");
//! 	service.register_protocol(Arc::new(MyHandler), "myproto", &[1u8]);
//!
//! 	// Wait for quit condition
//! 	// ...
//! 	// Drop the service
//! }
//! ```
mod host;
mod connection;
mod handshake;
mod session;
mod discovery;
mod service;
mod error;
mod node_table;
mod stats;
mod ip_utils;

#[cfg(test)]
mod tests;

pub use network::host::PeerId;
pub use network::host::PacketId;
pub use network::host::NetworkContext;
pub use network::service::NetworkService;
pub use network::host::NetworkIoMessage;
pub use network::host::NetworkIoMessage::User as UserMessage;
pub use network::error::NetworkError;
pub use network::host::NetworkConfiguration;
pub use network::stats::NetworkStats;

use io::TimerToken;
pub use network::node_table::is_valid_node_url;

const PROTOCOL_VERSION: u32 = 4;

/// Network IO protocol handler. This needs to be implemented for each new subprotocol.
/// All the handler function are called from within IO event loop.
/// `Message` is the type for message data.
pub trait NetworkProtocolHandler<Message>: Sync + Send where Message: Send + Sync + Clone {
	/// Initialize the handler
	fn initialize(&self, _io: &NetworkContext<Message>) {}
	/// Called when new network packet received.
	fn read(&self, io: &NetworkContext<Message>, peer: &PeerId, packet_id: u8, data: &[u8]);
	/// Called when new peer is connected. Only called when peer supports the same protocol.
	fn connected(&self, io: &NetworkContext<Message>, peer: &PeerId);
	/// Called when a previously connected peer disconnects.
	fn disconnected(&self, io: &NetworkContext<Message>, peer: &PeerId);
	/// Timer function called after a timeout created with `NetworkContext::timeout`.
	fn timeout(&self, _io: &NetworkContext<Message>, _timer: TimerToken) {}
	/// Called when a broadcasted message is received. The message can only be sent from a different IO handler.
	fn message(&self, _io: &NetworkContext<Message>, _message: &Message) {}
}

