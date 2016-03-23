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

use io::IoError;
use crypto::CryptoError;
use rlp::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
	Unknown,
}

impl DisconnectReason {
	pub fn from_u8(n: u8) -> DisconnectReason {
		match n {
			0 => DisconnectReason::DisconnectRequested,
			1 => DisconnectReason::TCPError,
			2 => DisconnectReason::BadProtocol,
			3 => DisconnectReason::UselessPeer,
			4 => DisconnectReason::TooManyPeers,
			5 => DisconnectReason::DuplicatePeer,
			6 => DisconnectReason::IncompatibleProtocol,
			7 => DisconnectReason::NullIdentity,
			8 => DisconnectReason::ClientQuit,
			9 => DisconnectReason::UnexpectedIdentity,
			10 => DisconnectReason::LocalIdentity,
			11 => DisconnectReason::PingTimeout,
			_ => DisconnectReason::Unknown,
		}
	}
}

#[derive(Debug)]
/// Network error.
pub enum NetworkError {
	/// Authentication error.
	Auth,
	/// Unrecognised protocol.
	BadProtocol,
	/// Message expired.
	Expired,
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

impl From<CryptoError> for NetworkError {
	fn from(_err: CryptoError) -> NetworkError {
		NetworkError::Auth
	}
}

#[test]
fn test_errors() {
	assert_eq!(DisconnectReason::ClientQuit, DisconnectReason::from_u8(8));
	let mut r = DisconnectReason::DisconnectRequested;
	for i in 0 .. 20 {
		r = DisconnectReason::from_u8(i);
	}
	assert_eq!(DisconnectReason::Unknown, r);

	match <NetworkError as From<DecoderError>>::from(DecoderError::RlpIsTooBig) {
		NetworkError::Auth => {},
		_ => panic!("Unexpeceted error"),
	}

	match <NetworkError as From<CryptoError>>::from(CryptoError::InvalidSecret) {
		NetworkError::Auth => {},
		_ => panic!("Unexpeceted error"),
	}
}
