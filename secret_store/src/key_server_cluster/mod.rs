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

#![allow(dead_code)] // TODO: remove me

use std::collections::BTreeMap;
use ethkey::{self, Public, Secret, Signature};
use super::types::all::DocumentAddress;

pub use super::acl_storage::AclStorage;

pub type NodeId = Public;
pub type SessionId = DocumentAddress;
pub type SessionIdSignature = Signature;

#[derive(Debug, PartialEq)]
/// Errors which can occur during encryption/decryption session
pub enum Error {
	/// Invalid number of nodes.
	/// There must be at least two nodes participating in encryption.
	/// There must be at least one node participating in decryption.
	InvalidNodesCount,
	/// Node which is required to start encryption/decryption session is not a part of cluster.
	InvalidNodesConfiguration,
	/// Invalid threshold value has been passed.
	/// Threshold value must be in [0; n - 1], where n is a number of nodes participating in the encryption.
	InvalidThreshold,
	/// Current state of encryption/decryption session does not allow to proceed request.
	/// This means that either there is some comm-failure or node is misbehaving/cheating.
	InvalidStateForRequest,
	/// Some data in passed message was recognized as invalid.
	/// This means that node is misbehaving/cheating.
	InvalidMessage,
	/// Cryptographic error.
	EthKey(String),
}

#[derive(Debug, Clone)]
/// Data, which is stored on every node after DKG && encryption is completed.
pub struct EncryptedData {
	/// Decryption threshold (at least threshold + 1 nodes are required to decrypt data).
	threshold: usize,
	/// Nodes ids numbers.
	id_numbers: BTreeMap<NodeId, Secret>,
	/// Node secret share.
	secret_share: Secret,
	/// Common (shared) encryption point.
	common_point: Public,
	/// Encrypted point.
	encrypted_point: Public,
}

impl From<ethkey::Error> for Error {
	fn from(err: ethkey::Error) -> Self {
		Error::EthKey(err.into())
	}
}

mod cluster;
mod decryption_session;
mod encryption_session;
mod math;
mod message;
