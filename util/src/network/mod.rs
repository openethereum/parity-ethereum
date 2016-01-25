/// Network and general IO module. 
/// Example usage for craeting a network service and adding an IO handler:
///
/// ```rust
/// extern crate ethcore_util as util;
/// use util::*;
///
/// struct MyHandler;
///
/// #[derive(Clone)]
/// struct MyMessage {
/// 	data: u32
/// }
///
/// impl NetworkProtocolHandler<MyMessage> for MyHandler {
///		fn initialize(&self, io: &NetworkContext<MyMessage>) {
///			io.register_timer(0, 1000);
///		}
///
///		fn read(&self, io: &NetworkContext<MyMessage>, peer: &PeerId, packet_id: u8, data: &[u8]) {
///			println!("Received {} ({} bytes) from {}", packet_id, data.len(), peer);
///		}
///
///		fn connected(&self, io: &NetworkContext<MyMessage>, peer: &PeerId) {
///			println!("Connected {}", peer);
///		}
///
///		fn disconnected(&self, io: &NetworkContext<MyMessage>, peer: &PeerId) {
///			println!("Disconnected {}", peer);
///		}
///
///		fn timeout(&self, io: &NetworkContext<MyMessage>, timer: TimerToken) {
///			println!("Timeout {}", timer);
///		}
///
///		fn message(&self, io: &NetworkContext<MyMessage>, message: &MyMessage) {
///			println!("Message {}", message.data);
///		}
/// }
///
/// fn main () {
/// 	let mut service = NetworkService::<MyMessage>::start(NetworkConfiguration::new()).expect("Error creating network service");
/// 	service.register_protocol(Arc::new(MyHandler), "myproto", &[1u8]);
///
/// 	// Wait for quit condition
/// 	// ...
/// 	// Drop the service
/// }
/// ```
mod host;
mod connection;
mod handshake;
mod session;
mod discovery;
mod service;
mod error;
mod node;
mod stats;

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

