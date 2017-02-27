#![allow(dead_code)] // TODO: remove me

use ethkey::{self, Public, Signature};
use super::types::all::DocumentAddress;

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
	EthKey(String),
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
