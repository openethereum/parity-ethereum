use io::IoError;
use rlp::*;

#[derive(Debug, Copy, Clone)]
pub enum DisconnectReason
{
	DisconnectRequested,
	//TCPError,
	//BadProtocol,
	UselessPeer,
	//TooManyPeers,
	//DuplicatePeer,
	//IncompatibleProtocol,
	//NullIdentity,
	//ClientQuit,
	//UnexpectedIdentity,
	//LocalIdentity,
	//PingTimeout,
}

#[derive(Debug)]
/// Network error.
pub enum NetworkError {
	/// Authentication error.
	Auth,
	/// Unrecognised protocol.
	BadProtocol,
	/// Peer not found.
	PeerNotFound,
	/// Peer is diconnected.
	Disconnect(DisconnectReason),
	/// Socket IO error.
	Io(IoError),
}

impl From<DecoderError> for NetworkError {
	fn from(_err: DecoderError) -> NetworkError {
		NetworkError::Auth
	}
}

impl From<IoError> for NetworkError {
	fn from(err: IoError) -> NetworkError {
		NetworkError::Io(err)
	}
}

