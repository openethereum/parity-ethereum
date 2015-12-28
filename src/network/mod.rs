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

pub trait ProtocolHandler: Send {
	fn initialize(&mut self, io: &mut HandlerIo);
	fn read(&mut self, io: &mut HandlerIo, peer: &PeerId, packet_id: u8, data: &[u8]);
	fn connected(&mut self, io: &mut HandlerIo, peer: &PeerId);
	fn disconnected(&mut self, io: &mut HandlerIo, peer: &PeerId);
	fn timeout(&mut self, io: &mut HandlerIo, timer: TimerToken);
	fn message(&mut self, io: &mut HandlerIo, message: &Message);
}

pub struct NetworkClient;
pub type NetworkService = service::NetworkService;


