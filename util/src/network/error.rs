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
use rlp::*;

#[derive(Debug, Copy, Clone)]
pub enum DisconnectReason {
	DisconnectRequested,
	_TCPError,
	_BadProtocol,
	UselessPeer,
	_TooManyPeers,
	_DuplicatePeer,
	_IncompatibleProtocol,
	_NullIdentity,
	_ClientQuit,
	_UnexpectedIdentity,
	_LocalIdentity,
	PingTimeout,
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
