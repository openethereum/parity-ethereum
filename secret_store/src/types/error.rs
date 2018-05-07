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

use std::fmt;
use std::net;
use std::io::Error as IoError;

use {ethkey, crypto, kvdb};

/// Secret store error.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Error {
	/// Invalid node address has been passed.
	InvalidNodeAddress,
	/// Invalid node id has been passed.
	InvalidNodeId,
	/// Session with the given id already exists.
	DuplicateSessionId,
	/// No active session with given id.
	NoActiveSessionWithId,
	/// Invalid threshold value has been passed.
	/// Threshold value must be in [0; n - 1], where n is a number of nodes participating in the encryption.
	NotEnoughNodesForThreshold,
	/// Current state of encryption/decryption session does not allow to proceed request.
	/// Reschedule this request for later processing.
	TooEarlyForRequest,
	/// Current state of encryption/decryption session does not allow to proceed request.
	/// This means that either there is some comm-failure or node is misbehaving/cheating.
	InvalidStateForRequest,
	/// Request cannot be sent/received from this node.
	InvalidNodeForRequest,
	/// Message or some data in the message was recognized as invalid.
	/// This means that node is misbehaving/cheating.
	InvalidMessage,
	/// Message version is not supported.
	InvalidMessageVersion,
	/// Message is invalid because of replay-attack protection.
	ReplayProtection,
	/// Connection to node, required for this session is not established.
	NodeDisconnected,
	/// Server key with this ID is already generated.
	ServerKeyAlreadyGenerated,
	/// Server key with this ID is not yet generated.
	ServerKeyIsNotFound,
	/// Document key with this ID is already stored.
	DocumentKeyAlreadyStored,
	/// Document key with this ID is not yet stored.
	DocumentKeyIsNotFound,
	/// Consensus is temporary unreachable. Means that something is currently blocking us from either forming
	/// consensus group (like disconnecting from too many nodes, which are AGREE to partticipate in consensus)
	/// or from rejecting request (disconnecting from AccessDenied-nodes).
	ConsensusTemporaryUnreachable,
	/// Consensus is unreachable. It doesn't mean that it will ALWAYS remain unreachable, but right NOW we have
	/// enough nodes confirmed that they do not want to be a part of consensus. Example: we're connected to 10
	/// of 100 nodes. Key threshold is 6 (i.e. 7 nodes are required for consensus). 4 nodes are responding with
	/// reject => consensus is considered unreachable, even though another 90 nodes still can respond with OK.
	ConsensusUnreachable,
	/// Acl storage error.
	AccessDenied,
	/// Can't start session, because exclusive session is active.
	ExclusiveSessionActive,
	/// Can't start exclusive session, because there are other active sessions.
	HasActiveSessions,
	/// Insufficient requester data.
	InsufficientRequesterData(String),
	/// Cryptographic error.
	EthKey(String),
	/// I/O error has occured.
	Io(String),
	/// Deserialization error has occured.
	Serde(String),
	/// Hyper error.
	Hyper(String),
	/// Database-related error.
	Database(String),
	/// Internal error.
	Internal(String),
}

impl Error {
	/// Is this a fatal error? Non-fatal means that it is possible to replay the same request with a non-zero
	/// chance to success. I.e. the error is not about request itself (or current environment factors that
	/// are affecting request processing), but about current SecretStore state.
	pub fn is_non_fatal(&self) -> bool {
		match *self {
			// non-fatal errors:

			// session start errors => restarting session is a solution
			Error::DuplicateSessionId | Error::NoActiveSessionWithId |
			// unexpected message errors => restarting session/excluding node is a solution
			Error::TooEarlyForRequest | Error::InvalidStateForRequest | Error::InvalidNodeForRequest |
			// invalid message errors => restarting/updating/excluding node is a solution
			Error::InvalidMessage | Error::InvalidMessageVersion | Error::ReplayProtection |
			// connectivity problems => waiting for reconnect && restarting session is a solution
			Error::NodeDisconnected |
			// temporary (?) consensus problems, related to other non-fatal errors => restarting is probably (!) a solution
			Error::ConsensusTemporaryUnreachable |
			// exclusive session errors => waiting && restarting is a solution
			Error::ExclusiveSessionActive | Error::HasActiveSessions => true,

			// fatal errors:

			// config-related errors
			Error::InvalidNodeAddress | Error::InvalidNodeId |
			// wrong session input params errors
			Error::NotEnoughNodesForThreshold | Error::ServerKeyAlreadyGenerated | Error::ServerKeyIsNotFound |
				Error::DocumentKeyAlreadyStored | Error::DocumentKeyIsNotFound | Error::InsufficientRequesterData(_) |
			// access denied/consensus error
			Error::AccessDenied | Error::ConsensusUnreachable |
			// indeterminate internal errors, which could be either fatal (db failure, invalid request), or not (network error),
			// but we still consider these errors as fatal
			Error::EthKey(_) | Error::Serde(_) | Error::Hyper(_) | Error::Database(_) | Error::Internal(_) | Error::Io(_) => false,
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::InvalidNodeAddress => write!(f, "invalid node address has been passed"),
			Error::InvalidNodeId => write!(f, "invalid node id has been passed"),
			Error::DuplicateSessionId => write!(f, "session with the same id is already registered"),
			Error::NoActiveSessionWithId => write!(f, "no active session with given id"),
			Error::NotEnoughNodesForThreshold => write!(f, "not enough nodes for passed threshold"),
			Error::TooEarlyForRequest => write!(f, "session is not yet ready to process this request"),
			Error::InvalidStateForRequest => write!(f, "session is in invalid state for processing this request"),
			Error::InvalidNodeForRequest => write!(f, "invalid node for this request"),
			Error::InvalidMessage => write!(f, "invalid message is received"),
			Error::InvalidMessageVersion => write!(f, "unsupported message is received"),
			Error::ReplayProtection => write!(f, "replay message is received"),
			Error::NodeDisconnected => write!(f, "node required for this operation is currently disconnected"),
			Error::ServerKeyAlreadyGenerated => write!(f, "Server key with this ID is already generated"),
			Error::ServerKeyIsNotFound => write!(f, "Server key with this ID is not found"),
			Error::DocumentKeyAlreadyStored => write!(f, "Document key with this ID is already stored"),
			Error::DocumentKeyIsNotFound => write!(f, "Document key with this ID is not found"),
			Error::ConsensusUnreachable => write!(f, "Consensus unreachable"),
			Error::ConsensusTemporaryUnreachable => write!(f, "Consensus temporary unreachable"),
			Error::AccessDenied => write!(f, "Access dened"),
			Error::ExclusiveSessionActive => write!(f, "Exclusive session active"),
			Error::HasActiveSessions => write!(f, "Unable to start exclusive session"),
			Error::InsufficientRequesterData(ref e) => write!(f, "Insufficient requester data: {}", e),
			Error::EthKey(ref e) => write!(f, "cryptographic error {}", e),
			Error::Hyper(ref msg) => write!(f, "Hyper error: {}", msg),
			Error::Serde(ref msg) => write!(f, "Serialization error: {}", msg),
			Error::Database(ref msg) => write!(f, "Database error: {}", msg),
			Error::Internal(ref msg) => write!(f, "Internal error: {}", msg),
			Error::Io(ref msg) => write!(f, "IO error: {}", msg),
		}
	}
}

impl From<ethkey::Error> for Error {
	fn from(err: ethkey::Error) -> Self {
		Error::EthKey(err.into())
	}
}

impl From<ethkey::crypto::Error> for Error {
	fn from(err: ethkey::crypto::Error) -> Self {
		Error::EthKey(err.to_string())
	}
}

impl From<kvdb::Error> for Error {
	fn from(err: kvdb::Error) -> Self {
		Error::Database(err.to_string())
	}
}

impl From<crypto::Error> for Error {
	fn from(err: crypto::Error) -> Self {
		Error::EthKey(err.to_string())
	}
}

impl From<IoError> for Error {
	fn from(err: IoError) -> Self {
		Error::Io(err.to_string())
	}
}

impl Into<String> for Error {
	fn into(self) -> String {
		format!("{}", self)
	}
}

impl From<net::AddrParseError> for Error {
	fn from(err: net::AddrParseError) -> Error {
		Error::Internal(err.to_string())
	}
}
