/// Network and general IO module.
///
/// Example usage for craeting a network service and adding an IO handler:
///
/// ```rust
/// extern crate ethcore_util as util;
/// use util::network::*;
///
/// struct MyHandler;
///
/// impl ProtocolHandler for MyHandler {
///		fn initialize(&mut self, io: &mut HandlerIo) {
///			io.register_timer(1000);
///		}
///
///		fn read(&mut self, io: &mut HandlerIo, peer: &PeerId, packet_id: u8, data: &[u8]) {
///			println!("Received {} ({} bytes) from {}", packet_id, data.len(), peer);
///		}
///
///		fn connected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
///			println!("Connected {}", peer);
///		}
///
///		fn disconnected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
///			println!("Disconnected {}", peer);
///		}
///
///		fn timeout(&mut self, io: &mut HandlerIo, timer: TimerToken) {
///			println!("Timeout {}", timer);
///		}
///
///		fn message(&mut self, io: &mut HandlerIo, message: &Message) {
///			println!("Message {}:{}", message.protocol, message.id);
///		}
/// }
///
/// fn main () {
/// 	let mut service = NetworkService::start().expect("Error creating network service");
/// 	service.register_protocol(Box::new(MyHandler), "myproto", &[1u8]);
///
/// 	// Wait for quit condition
/// 	// ...
/// 	// Drop the service
/// }
/// ```
extern crate mio;
mod host;
mod connection;
mod handshake;
mod session;
mod discovery;
mod service;

#[derive(Debug, Copy, Clone)]
pub enum DisconnectReason
{
	DisconnectRequested,
	TCPError,
	BadProtocol,
	UselessPeer,
	TooManyPeers,
	DuplicatePeer,
	IncompatibleProtocol,
	NullIdentity,
	ClientQuit,
	UnexpectedIdentity,
	LocalIdentity,
	PingTimeout,
}

#[derive(Debug)]
pub enum Error {
	Crypto(::crypto::CryptoError),
	Io(::std::io::Error),
	Auth,
	BadProtocol,
	AddressParse(::std::net::AddrParseError),
	AddressResolve(Option<::std::io::Error>),
	NodeIdParse(::error::EthcoreError),
	PeerNotFound,
	Disconnect(DisconnectReason)
}

impl From<::std::io::Error> for Error {
	fn from(err: ::std::io::Error) -> Error {
		Error::Io(err)
	}
}

impl From<::crypto::CryptoError> for Error {
	fn from(err: ::crypto::CryptoError) -> Error {
		Error::Crypto(err)
	}
}

impl From<::std::net::AddrParseError> for Error {
	fn from(err: ::std::net::AddrParseError) -> Error {
		Error::AddressParse(err)
	}
}
impl From<::error::EthcoreError> for Error {
	fn from(err: ::error::EthcoreError) -> Error {
		Error::NodeIdParse(err)
	}
}
impl From<::rlp::DecoderError> for Error {
	fn from(_err: ::rlp::DecoderError) -> Error {
		Error::Auth
	}
}

impl From<::mio::NotifyError<host::HostMessage>> for Error {
	fn from(_err: ::mio::NotifyError<host::HostMessage>) -> Error {
		Error::Io(::std::io::Error::new(::std::io::ErrorKind::ConnectionAborted, "Network IO notification error"))
	}
}

pub type PeerId = host::PeerId;
pub type PacketId = host::PacketId;
pub type TimerToken = host::TimerToken;
pub type HandlerIo<'s> = host::HostIo<'s>;
pub type Message = host::UserMessage;
pub type MessageId = host::UserMessageId;

/// Network IO protocol handler. This needs to be implemented for each new subprotocol.
/// TODO: Separate p2p networking IO from IPC IO. `timeout` and `message` should go to a more genera IO provider.
/// All the handler function are called from within IO event loop.
pub trait ProtocolHandler: Send {
	/// Initialize the hadler
	fn initialize(&mut self, io: &mut HandlerIo);
	/// Called when new network packet received.
	fn read(&mut self, io: &mut HandlerIo, peer: &PeerId, packet_id: u8, data: &[u8]);
	/// Called when new peer is connected. Only called when peer supports the same protocol.
	fn connected(&mut self, io: &mut HandlerIo, peer: &PeerId);
	/// Called when a previously connected peer disconnects.
	fn disconnected(&mut self, io: &mut HandlerIo, peer: &PeerId);
	/// Timer function called after a timeout created with `HandlerIo::timeout`.
	fn timeout(&mut self, io: &mut HandlerIo, timer: TimerToken);
	/// Called when a broadcasted message is received. The message can only be sent from a different protocol handler.
	fn message(&mut self, io: &mut HandlerIo, message: &Message);
}

pub type NetworkService = service::NetworkService;

