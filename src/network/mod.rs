extern crate mio;
mod host;
mod connection;
mod handshake;
mod session;

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

pub fn start_host()
{
	let _ = host::Host::start();
}
