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

use io::IoError;
use rlp::*;
use util::UtilError;
use std::fmt;
use ethkey::Error as KeyError;
use crypto::Error as CryptoError;

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

impl fmt::Display for DisconnectReason {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::DisconnectReason::*;

		let msg = match *self {
			DisconnectRequested => "disconnect requested",
			TCPError => "TCP error",
			BadProtocol => "bad protocol",
			UselessPeer => "useless peer",
			TooManyPeers => "too many peers",
			DuplicatePeer => "duplicate peer",
			IncompatibleProtocol => "incompatible protocol",
			NullIdentity => "null identity",
			ClientQuit => "client quit",
			UnexpectedIdentity => "unexpected identity",
			LocalIdentity => "local identity",
			PingTimeout => "ping timeout",
			Unknown => "unknown",
		};

		f.write_str(msg)
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
	/// Util error.
	Util(UtilError),
	/// Socket IO error.
	Io(IoError),
	/// Error concerning the network address parsing subsystem.
	AddressParse(::std::net::AddrParseError),
	/// Error concerning the network address resolution subsystem.
	AddressResolve(Option<::std::io::Error>),
	/// Error concerning the Rust standard library's IO subsystem.
	StdIo(::std::io::Error),
	/// Packet size is over the protocol limit.
	OversizedPacket,
}

impl fmt::Display for NetworkError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::NetworkError::*;

		let msg = match *self {
			Auth => "Authentication failure".into(),
			BadProtocol => "Bad protocol".into(),
			Expired => "Expired message".into(),
			PeerNotFound => "Peer not found".into(),
			Disconnect(ref reason) => format!("Peer disconnected: {}", reason),
			Io(ref err) => format!("Socket I/O error: {}", err),
			AddressParse(ref err) => format!("{}", err),
			AddressResolve(Some(ref err)) => format!("{}", err),
			AddressResolve(_) => "Failed to resolve network address.".into(),
			StdIo(ref err) => format!("{}", err),
			Util(ref err) => format!("{}", err),
			OversizedPacket => "Packet is too large".into(),
		};

		f.write_fmt(format_args!("Network error ({})", msg))
	}
}

impl From<DecoderError> for NetworkError {
	fn from(_err: DecoderError) -> NetworkError {
		NetworkError::Auth
	}
}

impl From<::std::io::Error> for NetworkError {
	fn from(err: ::std::io::Error) -> NetworkError {
		NetworkError::StdIo(err)
	}
}

impl From<IoError> for NetworkError {
	fn from(err: IoError) -> NetworkError {
		NetworkError::Io(err)
	}
}

impl From<UtilError> for NetworkError {
	fn from(err: UtilError) -> NetworkError {
		NetworkError::Util(err)
	}
}

impl From<KeyError> for NetworkError {
	fn from(_err: KeyError) -> Self {
		NetworkError::Auth
	}
}

impl From<CryptoError> for NetworkError {
	fn from(_err: CryptoError) -> NetworkError {
		NetworkError::Auth
	}
}

impl From<::std::net::AddrParseError> for NetworkError {
	fn from(err: ::std::net::AddrParseError) -> NetworkError {
		NetworkError::AddressParse(err)
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

	match <NetworkError as From<CryptoError>>::from(CryptoError::InvalidMessage) {
		NetworkError::Auth => {},
		_ => panic!("Unexpeceted error"),
	}
}
