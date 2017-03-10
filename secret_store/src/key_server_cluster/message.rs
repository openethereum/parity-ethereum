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
use std::cmp::{Ord, PartialOrd, Ordering};
use std::ops::Deref;
use std::collections::{BTreeSet, BTreeMap};
use rustc_serialize::hex::{FromHex, ToHex};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, Error as SerdeError};
use ethkey::{Public, Secret, Signature};
use util::H256;
use key_server_cluster::{NodeId, SessionId};

pub type MessageSessionId = MessageH256;
pub type MessageNodeId = MessagePublic;

#[derive(Clone, Debug)]
/// All possible messages that can be sent during encryption/decryption sessions.
pub enum Message {
	/// Introduce node public key.
	NodePublicKey(NodePublicKey),
	/// Confirm that node owns its private key.
	NodePrivateKeySignature(NodePrivateKeySignature),

	/// Initialize new DKG session.
	InitializeSession(InitializeSession),
	/// Confirm DKG session initialization.
	ConfirmInitialization(ConfirmInitialization),
	/// Broadcast data, calculated during session initialization phase.
	CompleteInitialization(CompleteInitialization),
	/// Generated keys are sent to every node.
	KeysDissemination(KeysDissemination),
	/// Complaint against another node is broadcasted.
	Complaint(Complaint),
	/// Complaint response is broadcasted.
	ComplaintResponse(ComplaintResponse),
	/// Broadcast self public key portion.
	PublicKeyShare(PublicKeyShare),

	/// Initialize decryption session.
	InitializeDecryptionSession(InitializeDecryptionSession),
	/// Confirm/reject decryption session initialization.
	ConfirmDecryptionInitialization(ConfirmDecryptionInitialization),
	/// Request partial decryption from node.
	RequestPartialDecryption(RequestPartialDecryption),
	/// Partial decryption is completed
	PartialDecryption(PartialDecryption),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Introduce node public key.
pub struct NodePublicKey {
	/// Node identifier (aka node public key).
	pub node_id: MessageNodeId,
	/// Data, which must be signed by peer to prove that he owns the corresponding private key. 
	pub confirmation_plain: MessageH256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Confirm that node owns the private key of previously passed public key (aka node id).
pub struct NodePrivateKeySignature {
	/// Previously passed `confirmation_plain`, signed with node private key.
	pub confirmation_signed: MessageSignature,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Initialize new DKG session.
pub struct InitializeSession {
	/// Session Id.
	pub session: MessageSessionId,
	/// Derived generation point. Starting from originator, every node must multiply this
	/// point by random scalar (unknown by other nodes). At the end of initialization
	/// `point` will be some (k1 * k2 * ... * kn) * G = `point` where `(k1 * k2 * ... * kn)`
	/// is unknown for every node.
	pub derived_point: MessagePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Confirm DKG session initialization.
pub struct ConfirmInitialization {
	/// Session Id.
	pub session: MessageSessionId,
	/// Derived generation point.
	pub derived_point: MessagePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Broadcast generated point to every other node.
pub struct CompleteInitialization {
	/// Session Id.
	pub session: MessageSessionId,
	/// All session participants along with their identification numbers.
	pub nodes: BTreeMap<MessageNodeId, MessageSecret>,
	/// Decryption threshold. During decryption threshold-of-route.len() nodes must came to
	/// consensus to successfully decrypt message.
	pub threshold: usize,
	/// Derived generation point.
	pub derived_point: MessagePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Generated keys are sent to every node.
pub struct KeysDissemination {
	/// Session Id.
	pub session: MessageSessionId,
	/// Secret 1.
	pub secret1: MessageSecret,
	/// Secret 2.
	pub secret2: MessageSecret,
	/// Public values.
	pub publics: Vec<MessagePublic>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Complaint against node is broadcasted.
pub struct Complaint {
	/// Session Id.
	pub session: MessageSessionId,
	/// Public values.
	pub against: MessageNodeId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is responding to complaint.
pub struct ComplaintResponse {
	/// Session Id.
	pub session: MessageSessionId,
	/// Secret 1.
	pub secret1: MessageSecret,
	/// Secret 2.
	pub secret2: MessageSecret,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is sharing its public key share.
pub struct PublicKeyShare {
	/// Session Id.
	pub session: MessageSessionId,
	/// Public key share.
	pub public_share: MessagePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is requested to decrypt data, encrypted in given session.
pub struct InitializeDecryptionSession {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: MessageSecret,
	/// Requestor signature.
	pub requestor_signature: MessageSignature,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is responding to decryption request.
pub struct ConfirmDecryptionInitialization {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: MessageSecret,
	/// Is node confirmed to make a decryption?.
	pub is_confirmed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is requested to do a partial decryption.
pub struct RequestPartialDecryption {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: MessageSecret,
	/// Nodes that are agreed to do a decryption.
	pub nodes: BTreeSet<MessageNodeId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node has partially decrypted the secret.
pub struct PartialDecryption {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: MessageSecret,
	/// Partially decrypted secret.
	pub shadow_point: MessagePublic,
}

#[derive(Clone, Debug)]
/// Serializable Signature.
pub struct MessageSignature(Signature);

impl<T> From<T> for MessageSignature where Signature: From<T> {
	fn from(s: T) -> MessageSignature {
		MessageSignature(s.into())
	}
}

impl Into<Signature> for MessageSignature {
	fn into(self) -> Signature {
		self.0
	}
}

impl Deref for MessageSignature {
	type Target = Signature;

	fn deref(&self) -> &Signature {
		&self.0
	}
}

impl Serialize for MessageSignature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&(*self.0).to_hex())
	}
}

impl Deserialize for MessageSignature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		struct HashVisitor;

		impl Visitor for HashVisitor {
			type Value = MessageSignature;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded Signature")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				value.parse().map(|s| MessageSignature(s)).map_err(SerdeError::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize(HashVisitor)
	}
}

#[derive(Clone, Debug)]
/// Serializable H256.
pub struct MessageH256(H256);

impl<T> From<T> for MessageH256 where H256: From<T> {
	fn from(s: T) -> MessageH256 {
		MessageH256(s.into())
	}
}

impl Into<H256> for MessageH256 {
	fn into(self) -> H256 {
		self.0
	}
}

impl Deref for MessageH256 {
	type Target = H256;

	fn deref(&self) -> &H256 {
		&self.0
	}
}

impl Serialize for MessageH256 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&(*self.0).to_hex())
	}
}

impl Deserialize for MessageH256 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		struct HashVisitor;

		impl Visitor for HashVisitor {
			type Value = MessageH256;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded H256")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				value.parse().map(|s| MessageH256(s)).map_err(SerdeError::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize(HashVisitor)
	}
}

#[derive(Clone, Debug)]
/// Serializable EC scalar/secret key.
pub struct MessageSecret(Secret);

impl<T> From<T> for MessageSecret where Secret: From<T> {
	fn from(s: T) -> MessageSecret {
		MessageSecret(s.into())
	}
}

impl Into<Secret> for MessageSecret {
	fn into(self) -> Secret {
		self.0
	}
}

impl Deref for MessageSecret {
	type Target = Secret;

	fn deref(&self) -> &Secret {
		&self.0
	}
}

impl Serialize for MessageSecret {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&(*self.0).to_hex())
	}
}

impl Deserialize for MessageSecret {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		struct HashVisitor;

		impl Visitor for HashVisitor {
			type Value = MessageSecret;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded EC scalar")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				value.parse().map(|s| MessageSecret(s)).map_err(SerdeError::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize(HashVisitor)
	}
}

#[derive(Clone, Debug)]
/// Serializable EC point/public key.
pub struct MessagePublic(Public);

impl<T> From<T> for MessagePublic where Public: From<T> {
	fn from(p: T) -> MessagePublic {
		MessagePublic(p.into())
	}
}

impl Into<Public> for MessagePublic {
	fn into(self) -> Public {
		self.0
	}
}

impl Deref for MessagePublic {
	type Target = Public;

	fn deref(&self) -> &Public {
		&self.0
	}
}

impl Eq for MessagePublic { }

impl PartialEq for MessagePublic {
	fn eq(&self, other: &MessagePublic) -> bool {
		self.0.eq(&other.0)
	}
}

impl Ord for MessagePublic {
	fn cmp(&self, other: &MessagePublic) -> Ordering {
		self.0.cmp(&other.0)
	}
}

impl PartialOrd for MessagePublic {
	fn partial_cmp(&self, other: &MessagePublic) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl Serialize for MessagePublic {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&(*self.0).to_hex())
	}
}

impl Deserialize for MessagePublic {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		struct HashVisitor;

		impl Visitor for HashVisitor {
			type Value = MessagePublic;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded EC point")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				value.parse().map(|s| MessagePublic(s)).map_err(SerdeError::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize(HashVisitor)
	}
}
