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
use std::collections::{BTreeSet, BTreeMap};
use ethkey::Secret;
use key_server_cluster::SessionId;
use super::{SerializableH256, SerializablePublic, SerializableSecret, SerializableSignature};

pub type MessageSessionId = SerializableH256;
pub type MessageNodeId = SerializablePublic;

#[derive(Clone, Debug)]
/// All possible messages that can be sent during encryption/decryption sessions.
pub enum Message {
	/// Cluster message.
	Cluster(ClusterMessage),
	/// Encryption message.
	Encryption(EncryptionMessage),
	/// Decryption message.
	Decryption(DecryptionMessage),
}

#[derive(Clone, Debug)]
/// All possible cluster-level messages.
pub enum ClusterMessage {
	/// Introduce node public key.
	NodePublicKey(NodePublicKey),
	/// Confirm that node owns its private key.
	NodePrivateKeySignature(NodePrivateKeySignature),
	/// Keep alive message.
	KeepAlive(KeepAlive),
	/// Keep alive message response.
	KeepAliveResponse(KeepAliveResponse),
}

#[derive(Clone, Debug)]
/// All possible messages that can be sent during encryption session.
pub enum EncryptionMessage {
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
	/// When session error has occured.
	SessionError(SessionError),
	/// When session is completed.
	SessionCompleted(SessionCompleted),
}

#[derive(Clone, Debug)]
/// All possible messages that can be sent during decryption session.
pub enum DecryptionMessage {
	/// Initialize decryption session.
	InitializeDecryptionSession(InitializeDecryptionSession),
	/// Confirm/reject decryption session initialization.
	ConfirmDecryptionInitialization(ConfirmDecryptionInitialization),
	/// Request partial decryption from node.
	RequestPartialDecryption(RequestPartialDecryption),
	/// Partial decryption is completed
	PartialDecryption(PartialDecryption),
	/// When decryption session error has occured.
	DecryptionSessionError(DecryptionSessionError),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Introduce node public key.
pub struct NodePublicKey {
	/// Node identifier (aka node public key).
	pub node_id: MessageNodeId,
	/// Data, which must be signed by peer to prove that he owns the corresponding private key. 
	pub confirmation_plain: SerializableH256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Confirm that node owns the private key of previously passed public key (aka node id).
pub struct NodePrivateKeySignature {
	/// Previously passed `confirmation_plain`, signed with node private key.
	pub confirmation_signed: SerializableSignature,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
/// Ask if the node is still alive.
pub struct KeepAlive {
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Confirm that the node is still alive.
pub struct KeepAliveResponse {
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
	pub derived_point: SerializablePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Confirm DKG session initialization.
pub struct ConfirmInitialization {
	/// Session Id.
	pub session: MessageSessionId,
	/// Derived generation point.
	pub derived_point: SerializablePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Broadcast generated point to every other node.
pub struct CompleteInitialization {
	/// Session Id.
	pub session: MessageSessionId,
	/// All session participants along with their identification numbers.
	pub nodes: BTreeMap<MessageNodeId, SerializableSecret>,
	/// Decryption threshold. During decryption threshold-of-route.len() nodes must came to
	/// consensus to successfully decrypt message.
	pub threshold: usize,
	/// Derived generation point.
	pub derived_point: SerializablePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Generated keys are sent to every node.
pub struct KeysDissemination {
	/// Session Id.
	pub session: MessageSessionId,
	/// Secret 1.
	pub secret1: SerializableSecret,
	/// Secret 2.
	pub secret2: SerializableSecret,
	/// Public values.
	pub publics: Vec<SerializablePublic>,
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
	pub secret1: SerializableSecret,
	/// Secret 2.
	pub secret2: SerializableSecret,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is sharing its public key share.
pub struct PublicKeyShare {
	/// Session Id.
	pub session: MessageSessionId,
	/// Public key share.
	pub public_share: SerializablePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// When session error has occured.
pub struct SessionError {
	/// Session Id.
	pub session: MessageSessionId,
	/// Public key share.
	pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// When session is completed.
pub struct SessionCompleted {
	/// Session Id.
	pub session: MessageSessionId,
	/// Common (shared) encryption point.
	pub common_point: SerializablePublic,
	/// Encrypted point.
	pub encrypted_point: SerializablePublic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is requested to decrypt data, encrypted in given session.
pub struct InitializeDecryptionSession {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Requestor signature.
	pub requestor_signature: SerializableSignature,
	/// Is shadow decryption requested? When true, decryption result
	/// will be visible to the owner of requestor public key only.
	pub is_shadow_decryption: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is responding to decryption request.
pub struct ConfirmDecryptionInitialization {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Is node confirmed to make a decryption?.
	pub is_confirmed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node is requested to do a partial decryption.
pub struct RequestPartialDecryption {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Nodes that are agreed to do a decryption.
	pub nodes: BTreeSet<MessageNodeId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Node has partially decrypted the secret.
pub struct PartialDecryption {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Partially decrypted secret.
	pub shadow_point: SerializablePublic,
	/// Decrypt shadow coefficient (if requested), encrypted with requestor public.
	pub decrypt_shadow: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// When decryption session error has occured.
pub struct DecryptionSessionError {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Public key share.
	pub error: String,
}

impl EncryptionMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			EncryptionMessage::InitializeSession(ref msg) => &msg.session,
			EncryptionMessage::ConfirmInitialization(ref msg) => &msg.session,
			EncryptionMessage::CompleteInitialization(ref msg) => &msg.session,
			EncryptionMessage::KeysDissemination(ref msg) => &msg.session,
			EncryptionMessage::Complaint(ref msg) => &msg.session,
			EncryptionMessage::ComplaintResponse(ref msg) => &msg.session,
			EncryptionMessage::PublicKeyShare(ref msg) => &msg.session,
			EncryptionMessage::SessionError(ref msg) => &msg.session,
			EncryptionMessage::SessionCompleted(ref msg) => &msg.session,
		}
	}
}

impl DecryptionMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			DecryptionMessage::InitializeDecryptionSession(ref msg) => &msg.session,
			DecryptionMessage::ConfirmDecryptionInitialization(ref msg) => &msg.session,
			DecryptionMessage::RequestPartialDecryption(ref msg) => &msg.session,
			DecryptionMessage::PartialDecryption(ref msg) => &msg.session,
			DecryptionMessage::DecryptionSessionError(ref msg) => &msg.session,
		}
	}

	pub fn sub_session_id(&self) -> &Secret {
		match *self {
			DecryptionMessage::InitializeDecryptionSession(ref msg) => &msg.sub_session,
			DecryptionMessage::ConfirmDecryptionInitialization(ref msg) => &msg.sub_session,
			DecryptionMessage::RequestPartialDecryption(ref msg) => &msg.sub_session,
			DecryptionMessage::PartialDecryption(ref msg) => &msg.sub_session,
			DecryptionMessage::DecryptionSessionError(ref msg) => &msg.sub_session,
		}
	}
}

impl fmt::Display for Message {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Message::Cluster(ref message) => write!(f, "Cluster.{}", message),
			Message::Encryption(ref message) => write!(f, "Encryption.{}", message),
			Message::Decryption(ref message) => write!(f, "Decryption.{}", message),
		}
	}
}

impl fmt::Display for ClusterMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ClusterMessage::NodePublicKey(_) => write!(f, "NodePublicKey"),
			ClusterMessage::NodePrivateKeySignature(_) => write!(f, "NodePrivateKeySignature"),
			ClusterMessage::KeepAlive(_) => write!(f, "KeepAlive"),
			ClusterMessage::KeepAliveResponse(_) => write!(f, "KeepAliveResponse"),
		}
	}
}

impl fmt::Display for EncryptionMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			EncryptionMessage::InitializeSession(_) => write!(f, "InitializeSession"),
			EncryptionMessage::ConfirmInitialization(_) => write!(f, "ConfirmInitialization"),
			EncryptionMessage::CompleteInitialization(_) => write!(f, "CompleteInitialization"),
			EncryptionMessage::KeysDissemination(_) => write!(f, "KeysDissemination"),
			EncryptionMessage::Complaint(_) => write!(f, "Complaint"),
			EncryptionMessage::ComplaintResponse(_) => write!(f, "ComplaintResponse"),
			EncryptionMessage::PublicKeyShare(_) => write!(f, "PublicKeyShare"),
			EncryptionMessage::SessionError(ref msg) => write!(f, "SessionError({})", msg.error),
			EncryptionMessage::SessionCompleted(_) => write!(f, "SessionCompleted"),
		}
	}
}

impl fmt::Display for DecryptionMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			DecryptionMessage::InitializeDecryptionSession(_) => write!(f, "InitializeDecryptionSession"),
			DecryptionMessage::ConfirmDecryptionInitialization(_) => write!(f, "ConfirmDecryptionInitialization"),
			DecryptionMessage::RequestPartialDecryption(_) => write!(f, "RequestPartialDecryption"),
			DecryptionMessage::PartialDecryption(_) => write!(f, "PartialDecryption"),
			DecryptionMessage::DecryptionSessionError(_) => write!(f, "DecryptionSessionError"),
		}
	}
}
