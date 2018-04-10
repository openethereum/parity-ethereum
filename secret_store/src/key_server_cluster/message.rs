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
use super::{Error, SerializableH256, SerializablePublic, SerializableSecret,
	SerializableSignature, SerializableMessageHash, SerializableRequester, SerializableAddress};

pub type MessageSessionId = SerializableH256;
pub type MessageNodeId = SerializablePublic;

/// All possible messages that can be sent during encryption/decryption sessions.
#[derive(Clone, Debug)]
pub enum Message {
	/// Cluster message.
	Cluster(ClusterMessage),
	/// Key generation message.
	Generation(GenerationMessage),
	/// Encryption message.
	Encryption(EncryptionMessage),
	/// Decryption message.
	Decryption(DecryptionMessage),
	/// Schnorr signing message.
	SchnorrSigning(SchnorrSigningMessage),
	/// ECDSA signing message.
	EcdsaSigning(EcdsaSigningMessage),
	/// Key version negotiation message.
	KeyVersionNegotiation(KeyVersionNegotiationMessage),
	/// Share add message.
	ShareAdd(ShareAddMessage),
	/// Servers set change message.
	ServersSetChange(ServersSetChangeMessage),
}

/// All possible cluster-level messages.
#[derive(Clone, Debug)]
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

/// All possible messages that can be sent during key generation session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GenerationMessage {
	/// Initialize new DKG session.
	InitializeSession(InitializeSession),
	/// Confirm DKG session initialization.
	ConfirmInitialization(ConfirmInitialization),
	/// Broadcast data, calculated during session initialization phase.
	CompleteInitialization(CompleteInitialization),
	/// Generated keys are sent to every node.
	KeysDissemination(KeysDissemination),
	/// Broadcast self public key portion.
	PublicKeyShare(PublicKeyShare),
	/// When session error has occured.
	SessionError(SessionError),
	/// When session is completed.
	SessionCompleted(SessionCompleted),
}

/// All possible messages that can be sent during encryption session.
#[derive(Clone, Debug)]
pub enum EncryptionMessage {
	/// Initialize encryption session.
	InitializeEncryptionSession(InitializeEncryptionSession),
	/// Confirm/reject encryption session initialization.
	ConfirmEncryptionInitialization(ConfirmEncryptionInitialization),
	/// When encryption session error has occured.
	EncryptionSessionError(EncryptionSessionError),
}

/// All possible messages that can be sent during consensus establishing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsensusMessage {
	/// Initialize consensus session.
	InitializeConsensusSession(InitializeConsensusSession),
	/// Confirm/reject consensus session initialization.
	ConfirmConsensusInitialization(ConfirmConsensusInitialization),
}

/// All possible messages that can be sent during servers-set consensus establishing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsensusMessageWithServersSet {
	/// Initialize consensus session.
	InitializeConsensusSession(InitializeConsensusSessionWithServersSet),
	/// Confirm/reject consensus session initialization.
	ConfirmConsensusInitialization(ConfirmConsensusInitialization),
}

/// All possible messages that can be sent during share add consensus establishing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsensusMessageOfShareAdd {
	/// Initialize consensus session.
	InitializeConsensusSession(InitializeConsensusSessionOfShareAdd),
	/// Confirm/reject consensus session initialization.
	ConfirmConsensusInitialization(ConfirmConsensusInitialization),
}

/// All possible messages that can be sent during decryption session.
#[derive(Clone, Debug)]
pub enum DecryptionMessage {
	/// Consensus establishing message.
	DecryptionConsensusMessage(DecryptionConsensusMessage),
	/// Request partial decryption from node.
	RequestPartialDecryption(RequestPartialDecryption),
	/// Partial decryption is completed.
	PartialDecryption(PartialDecryption),
	/// When decryption session error has occured.
	DecryptionSessionError(DecryptionSessionError),
	/// When decryption session is completed.
	DecryptionSessionCompleted(DecryptionSessionCompleted),
	/// When decryption session is delegated to another node.
	DecryptionSessionDelegation(DecryptionSessionDelegation),
	/// When delegated decryption session is completed.
	DecryptionSessionDelegationCompleted(DecryptionSessionDelegationCompleted),
}

/// All possible messages that can be sent during Schnorr signing session.
#[derive(Clone, Debug)]
pub enum SchnorrSigningMessage {
	/// Consensus establishing message.
	SchnorrSigningConsensusMessage(SchnorrSigningConsensusMessage),
	/// Session key generation message.
	SchnorrSigningGenerationMessage(SchnorrSigningGenerationMessage),
	/// Request partial signature from node.
	SchnorrRequestPartialSignature(SchnorrRequestPartialSignature),
	/// Partial signature is generated.
	SchnorrPartialSignature(SchnorrPartialSignature),
	/// Signing error occured.
	SchnorrSigningSessionError(SchnorrSigningSessionError),
	/// Signing session completed.
	SchnorrSigningSessionCompleted(SchnorrSigningSessionCompleted),
	/// When signing session is delegated to another node.
	SchnorrSigningSessionDelegation(SchnorrSigningSessionDelegation),
	/// When delegated signing session is completed.
	SchnorrSigningSessionDelegationCompleted(SchnorrSigningSessionDelegationCompleted),
}

/// All possible messages that can be sent during ECDSA signing session.
#[derive(Clone, Debug)]
pub enum EcdsaSigningMessage {
	/// Consensus establishing message.
	EcdsaSigningConsensusMessage(EcdsaSigningConsensusMessage),
	/// Signature nonce generation message.
	EcdsaSignatureNonceGenerationMessage(EcdsaSignatureNonceGenerationMessage),
	/// Inversion nonce generation message.
	EcdsaInversionNonceGenerationMessage(EcdsaInversionNonceGenerationMessage),
	/// Inversion zero generation message.
	EcdsaInversionZeroGenerationMessage(EcdsaInversionZeroGenerationMessage),
	/// Inversed nonce coefficient share.
	EcdsaSigningInversedNonceCoeffShare(EcdsaSigningInversedNonceCoeffShare),
	/// Request partial signature from node.
	EcdsaRequestPartialSignature(EcdsaRequestPartialSignature),
	/// Partial signature is generated.
	EcdsaPartialSignature(EcdsaPartialSignature),
	/// Signing error occured.
	EcdsaSigningSessionError(EcdsaSigningSessionError),
	/// Signing session completed.
	EcdsaSigningSessionCompleted(EcdsaSigningSessionCompleted),
	/// When signing session is delegated to another node.
	EcdsaSigningSessionDelegation(EcdsaSigningSessionDelegation),
	/// When delegated signing session is completed.
	EcdsaSigningSessionDelegationCompleted(EcdsaSigningSessionDelegationCompleted),
}

/// All possible messages that can be sent during servers set change session.
#[derive(Clone, Debug)]
pub enum ServersSetChangeMessage {
	/// Consensus establishing message.
	ServersSetChangeConsensusMessage(ServersSetChangeConsensusMessage),
	/// Unknown sessions ids request.
	UnknownSessionsRequest(UnknownSessionsRequest),
	/// Unknown sessions ids.
	UnknownSessions(UnknownSessions),
	/// Negotiating key version to use as a base for ShareAdd session.
	ShareChangeKeyVersionNegotiation(ShareChangeKeyVersionNegotiation),
	/// Initialize share change session(s).
	InitializeShareChangeSession(InitializeShareChangeSession),
	/// Confirm share change session(s) initialization.
	ConfirmShareChangeSessionInitialization(ConfirmShareChangeSessionInitialization),
	/// Share change session delegation.
	ServersSetChangeDelegate(ServersSetChangeDelegate),
	/// Share change session delegation response.
	ServersSetChangeDelegateResponse(ServersSetChangeDelegateResponse),
	/// Share add message.
	ServersSetChangeShareAddMessage(ServersSetChangeShareAddMessage),
	/// Servers set change session completed.
	ServersSetChangeError(ServersSetChangeError),
	/// Servers set change session completed.
	ServersSetChangeCompleted(ServersSetChangeCompleted),
}

/// All possible messages that can be sent during share add session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShareAddMessage {
	/// Consensus establishing message.
	ShareAddConsensusMessage(ShareAddConsensusMessage),
	/// Common key share data is sent to new node.
	KeyShareCommon(KeyShareCommon),
	/// Generated keys are sent to every node.
	NewKeysDissemination(NewKeysDissemination),
	/// When session error has occured.
	ShareAddError(ShareAddError),
}

/// All possible messages that can be sent during key version negotiation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KeyVersionNegotiationMessage {
	/// Request key versions.
	RequestKeyVersions(RequestKeyVersions),
	/// Key versions.
	KeyVersions(KeyVersions),
	/// When session error has occured.
	KeyVersionsError(KeyVersionsError),
}

/// Introduce node public key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodePublicKey {
	/// Node identifier (aka node public key).
	pub node_id: MessageNodeId,
	/// Random data, which must be signed by peer to prove that he owns the corresponding private key. 
	pub confirmation_plain: SerializableH256,
	/// The same random `confirmation_plain`, signed with one-time session key.
	pub confirmation_signed_session: SerializableSignature,
}

/// Confirm that node owns the private key of previously passed public key (aka node id).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodePrivateKeySignature {
	/// Previously passed `confirmation_plain`, signed with node private key.
	pub confirmation_signed: SerializableSignature,
}

/// Ask if the node is still alive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeepAlive {
}

/// Confirm that the node is still alive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeepAliveResponse {
	/// Session id, if used for session-level keep alive.
	pub session_id: Option<MessageSessionId>,
}

/// Initialize new DKG session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeSession {
	/// Session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Session origin address (if any).
	pub origin: Option<SerializableAddress>,
	/// Session author.
	pub author: SerializableAddress,
	/// All session participants along with their identification numbers.
	pub nodes: BTreeMap<MessageNodeId, SerializableSecret>,
	/// Is zero secret generation session?
	pub is_zero: bool,
	/// Decryption threshold. During decryption threshold-of-route.len() nodes must came to
	/// consensus to successfully decrypt message.
	pub threshold: usize,
	/// Derived generation point. Starting from originator, every node must multiply this
	/// point by random scalar (unknown by other nodes). At the end of initialization
	/// `point` will be some (k1 * k2 * ... * kn) * G = `point` where `(k1 * k2 * ... * kn)`
	/// is unknown for every node.
	pub derived_point: SerializablePublic,
}

/// Confirm DKG session initialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmInitialization {
	/// Session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Derived generation point.
	pub derived_point: SerializablePublic,
}

/// Broadcast generated point to every other node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompleteInitialization {
	/// Session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Derived generation point.
	pub derived_point: SerializablePublic,
}

/// Generated keys are sent to every node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeysDissemination {
	/// Session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Secret 1.
	pub secret1: SerializableSecret,
	/// Secret 2.
	pub secret2: SerializableSecret,
	/// Public values.
	pub publics: Vec<SerializablePublic>,
}

/// Node is sharing its public key share.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKeyShare {
	/// Session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Public key share.
	pub public_share: SerializablePublic,
}

/// When session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionError {
	/// Session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: Error,
}

/// When session is completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionCompleted {
	/// Session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// Node is requested to prepare for saving encrypted data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeEncryptionSession {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Requester.
	pub requester: SerializableRequester,
	/// Common point.
	pub common_point: SerializablePublic,
	/// Encrypted data.
	pub encrypted_point: SerializablePublic,
}

/// Node is responding to encryption initialization request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmEncryptionInitialization {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// When encryption session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptionSessionError {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: Error,
}

/// Node is asked to be part of consensus group.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeConsensusSession {
	/// Requester.
	pub requester: SerializableRequester,
	/// Key version.
	pub version: SerializableH256,
}

/// Node is responding to consensus initialization request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmConsensusInitialization {
	/// Is node confirmed consensus participation.
	pub is_confirmed: bool,
}

/// Node is asked to be part of servers-set consensus group.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeConsensusSessionWithServersSet {
	/// Migration id (if any).
	pub migration_id: Option<SerializableH256>,
	/// Old nodes set.
	pub old_nodes_set: BTreeSet<MessageNodeId>,
	/// New nodes set.
	pub new_nodes_set: BTreeSet<MessageNodeId>,
	/// Old server set, signed by requester.
	pub old_set_signature: SerializableSignature,
	/// New server set, signed by requester.
	pub new_set_signature: SerializableSignature,
}

/// Node is asked to be part of servers-set consensus group.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeConsensusSessionOfShareAdd {
	/// Key version.
	pub version: SerializableH256,
	/// threshold+1 nodes from old_nodes_set selected for shares redistribution.
	pub consensus_group: BTreeSet<MessageNodeId>,
	/// Old nodes set: all non-isolated owners of selected key share version.
	pub old_nodes_set: BTreeSet<MessageNodeId>,
	/// New nodes map: node id => node id number.
	pub new_nodes_map: BTreeMap<MessageNodeId, SerializableSecret>,
	/// Old server set, signed by requester.
	pub old_set_signature: SerializableSignature,
	/// New server set, signed by requester.
	pub new_set_signature: SerializableSignature,
}

/// Consensus-related Schnorr signing message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrSigningConsensusMessage {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Consensus message.
	pub message: ConsensusMessage,
}

/// Session key generation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrSigningGenerationMessage {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Generation message.
	pub message: GenerationMessage,
}

/// Request partial Schnorr signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrRequestPartialSignature {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Request id.
	pub request_id: SerializableSecret,
	/// Message hash.
	pub message_hash: SerializableMessageHash,
	/// Selected nodes.
	pub nodes: BTreeSet<MessageNodeId>,
}

/// Partial Schnorr signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrPartialSignature {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Request id.
	pub request_id: SerializableSecret,
	/// S part of signature.
	pub partial_signature: SerializableSecret,
}

/// When Schnorr signing session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrSigningSessionError {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: Error,
}

/// Schnorr signing session completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrSigningSessionCompleted {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// When Schnorr signing session is delegated to another node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrSigningSessionDelegation {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Requester.
	pub requester: SerializableRequester,
	/// Key version.
	pub version: SerializableH256,
	/// Message hash.
	pub message_hash: SerializableH256,
}

/// When delegated Schnorr signing session is completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrSigningSessionDelegationCompleted {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// S-portion of signature.
	pub signature_s: SerializableSecret,
	/// C-portion of signature.
	pub signature_c: SerializableSecret,
}

/// Consensus-related ECDSA signing message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaSigningConsensusMessage {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Consensus message.
	pub message: ConsensusMessage,
}

/// ECDSA signature nonce generation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaSignatureNonceGenerationMessage {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Generation message.
	pub message: GenerationMessage,
}

/// ECDSA inversion nonce generation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaInversionNonceGenerationMessage {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Generation message.
	pub message: GenerationMessage,
}

/// ECDSA inversed nonce share message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaSigningInversedNonceCoeffShare {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Inversed nonce coefficient share.
	pub inversed_nonce_coeff_share: SerializableSecret,
}

/// ECDSA inversion zero generation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaInversionZeroGenerationMessage {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Generation message.
	pub message: GenerationMessage,
}

/// Request partial ECDSA signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaRequestPartialSignature {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Request id.
	pub request_id: SerializableSecret,
	///
	pub inversed_nonce_coeff: SerializableSecret,
	/// Message hash.
	pub message_hash: SerializableMessageHash,
}

/// Partial ECDSA signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaPartialSignature {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Request id.
	pub request_id: SerializableSecret,
	/// Partial S part of signature.
	pub partial_signature_s: SerializableSecret,
}

/// When ECDSA signing session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaSigningSessionError {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: Error,
}

/// ECDSA signing session completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaSigningSessionCompleted {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// When ECDSA signing session is delegated to another node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaSigningSessionDelegation {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Requestor signature.
	pub requester: SerializableRequester,
	/// Key version.
	pub version: SerializableH256,
	/// Message hash.
	pub message_hash: SerializableH256,
}

/// When delegated ECDSA signing session is completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaSigningSessionDelegationCompleted {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Signature.
	pub signature: SerializableSignature,
}

/// Consensus-related decryption message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecryptionConsensusMessage {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Session origin (in consensus initialization message).
	pub origin: Option<SerializableAddress>,
	/// Consensus message.
	pub message: ConsensusMessage,
}

/// Node is requested to do a partial decryption.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestPartialDecryption {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Request id.
	pub request_id: SerializableSecret,
	/// Is shadow decryption requested? When true, decryption result
	/// will be visible to the owner of requestor public key only.
	pub is_shadow_decryption: bool,
	/// Decryption result must be reconstructed on all participating nodes. This is useful
	/// for service contract API so that all nodes from consensus group can confirm decryption.
	pub is_broadcast_session: bool,
	/// Nodes that are agreed to do a decryption.
	pub nodes: BTreeSet<MessageNodeId>,
}

/// Node has partially decrypted the secret.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartialDecryption {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Request id.
	pub request_id: SerializableSecret,
	/// Partially decrypted secret.
	pub shadow_point: SerializablePublic,
	/// Decrypt shadow coefficient (if requested), encrypted with requestor public.
	pub decrypt_shadow: Option<Vec<u8>>,
}

/// When decryption session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecryptionSessionError {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: Error,
}

/// When decryption session is completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecryptionSessionCompleted {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// When decryption session is delegated to another node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecryptionSessionDelegation {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Session origin.
	pub origin: Option<SerializableAddress>,
	/// Requester.
	pub requester: SerializableRequester,
	/// Key version.
	pub version: SerializableH256,
	/// Is shadow decryption requested? When true, decryption result
	/// will be visible to the owner of requestor public key only.
	pub is_shadow_decryption: bool,
	/// Decryption result must be reconstructed on all participating nodes. This is useful
	/// for service contract API so that all nodes from consensus group can confirm decryption.
	pub is_broadcast_session: bool,
}

/// When delegated decryption session is completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecryptionSessionDelegationCompleted {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Decryption session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Decrypted secret point. It is partially decrypted if shadow decrpytion was requested.
	pub decrypted_secret: SerializablePublic,
	/// Shared common point.
	pub common_point: Option<SerializablePublic>,
	/// If shadow decryption was requested: shadow decryption coefficients, encrypted with requestor public.
	pub decrypt_shadows: Option<Vec<Vec<u8>>>,
}

/// Consensus-related servers set change message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeConsensusMessage {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Consensus message.
	pub message: ConsensusMessageWithServersSet,
}

/// Unknown sessions ids request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnknownSessionsRequest {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// Unknown session ids.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnknownSessions {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Unknown session id.
	pub unknown_sessions: BTreeSet<MessageSessionId>,
}

/// Key version negotiation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareChangeKeyVersionNegotiation {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Key version negotiation message.
	pub message: KeyVersionNegotiationMessage,
}

/// Master node opens share initialize session on other nodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeShareChangeSession {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Key id.
	pub key_id: MessageSessionId,
	/// Key vesion to use in ShareAdd session.
	pub version: SerializableH256,
	/// Master node.
	pub master_node_id: MessageNodeId,
	/// Consensus group to use in ShareAdd session.
	pub consensus_group: BTreeSet<MessageNodeId>,
	/// Shares to add. Values are filled for new nodes only.
	pub new_nodes_map: BTreeMap<MessageNodeId, Option<SerializableSecret>>,
}

/// Slave node confirms session initialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmShareChangeSessionInitialization {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Sessions that are confirmed.
	pub key_id: MessageSessionId,
}

/// Share change is requested.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeDelegate {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Key id.
	pub key_id: MessageSessionId,
}

/// Share change is completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeDelegateResponse {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Key id.
	pub key_id: MessageSessionId,
}

/// Servers set change share add message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeShareAddMessage {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Unknown session id.
	pub message: ShareAddMessage,
}

/// When servers set change session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeError {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: Error,
}

/// When servers set change session is completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeCompleted {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// Consensus-related share add session message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareAddConsensusMessage {
	/// Share add session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Consensus message.
	pub message: ConsensusMessageOfShareAdd,
}

/// Key share common data is passed to new node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyShareCommon {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Key threshold.
	pub threshold: usize,
	/// Author of key share entry.
	pub author: SerializableAddress,
	/// Joint public.
	pub joint_public: SerializablePublic,
	/// Common (shared) encryption point.
	pub common_point: Option<SerializablePublic>,
	/// Encrypted point.
	pub encrypted_point: Option<SerializablePublic>,
	/// Selected version id numbers.
	pub id_numbers: BTreeMap<MessageNodeId, SerializableSecret>,
}

/// Generated keys are sent to every node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewKeysDissemination {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Sub share of rcevier' secret share.
	pub secret_subshare: SerializableSecret,
}

/// When share add session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareAddError {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: Error,
}

/// Key versions are requested.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestKeyVersions {
	/// Generation session id.
	pub session: MessageSessionId,
	/// Version negotiation session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// Key versions are sent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyVersions {
	/// Generation session id.
	pub session: MessageSessionId,
	/// Version negotiation session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Key threshold.
	pub threshold: Option<usize>,
	/// Key versions.
	pub versions: Vec<SerializableH256>,
}

/// When key versions error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyVersionsError {
	/// Generation session id.
	pub session: MessageSessionId,
	/// Version negotiation session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: Error,
}

impl Message {
	pub fn is_initialization_message(&self) -> bool {
		match *self {
			Message::Generation(GenerationMessage::InitializeSession(_)) => true,
			Message::Encryption(EncryptionMessage::InitializeEncryptionSession(_)) => true,
			Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(ref msg)) => match msg.message {
				ConsensusMessage::InitializeConsensusSession(_) => true,
				_ => false
			},
			Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningConsensusMessage(ref msg)) => match msg.message {
				ConsensusMessage::InitializeConsensusSession(_) => true,
				_ => false
			},
			Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningConsensusMessage(ref msg)) => match msg.message {
				ConsensusMessage::InitializeConsensusSession(_) => true,
				_ => false
			},
			Message::KeyVersionNegotiation(KeyVersionNegotiationMessage::RequestKeyVersions(_)) => true,
			Message::ShareAdd(ShareAddMessage::ShareAddConsensusMessage(ref msg)) => match msg.message {
				ConsensusMessageOfShareAdd::InitializeConsensusSession(_) => true,
				_ => false
			},
			Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref msg)) => match msg.message {
				ConsensusMessageWithServersSet::InitializeConsensusSession(_) => true,
				_ => false
			},
			_ => false,
		}
	}

	pub fn is_delegation_message(&self) -> bool {
		match *self {
			Message::Decryption(DecryptionMessage::DecryptionSessionDelegation(_)) => true,
			Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningSessionDelegation(_)) => true,
			Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningSessionDelegation(_)) => true,
			_ => false,
		}
	}

	pub fn is_error_message(&self) -> bool {
		match *self {
			Message::Generation(GenerationMessage::SessionError(_)) => true,
			Message::Encryption(EncryptionMessage::EncryptionSessionError(_)) => true,
			Message::Decryption(DecryptionMessage::DecryptionSessionError(_)) => true,
			Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningSessionError(_)) => true,
			Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningSessionError(_)) => true,
			Message::KeyVersionNegotiation(KeyVersionNegotiationMessage::KeyVersionsError(_)) => true,
			Message::ShareAdd(ShareAddMessage::ShareAddError(_)) => true,
			Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeError(_)) => true,
			_ => false,
		}
	}

	pub fn is_exclusive_session_message(&self) -> bool {
		match *self {
			Message::ServersSetChange(_) => true,
			_ => false,
		}
	}

	pub fn session_nonce(&self) -> Option<u64> {
		match *self {
			Message::Cluster(_) => None,
			Message::Generation(ref message) => Some(message.session_nonce()),
			Message::Encryption(ref message) => Some(message.session_nonce()),
			Message::Decryption(ref message) => Some(message.session_nonce()),
			Message::SchnorrSigning(ref message) => Some(message.session_nonce()),
			Message::EcdsaSigning(ref message) => Some(message.session_nonce()),
			Message::ShareAdd(ref message) => Some(message.session_nonce()),
			Message::ServersSetChange(ref message) => Some(message.session_nonce()),
			Message::KeyVersionNegotiation(ref message) => Some(message.session_nonce()),
		}
	}
}

impl GenerationMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			GenerationMessage::InitializeSession(ref msg) => &msg.session,
			GenerationMessage::ConfirmInitialization(ref msg) => &msg.session,
			GenerationMessage::CompleteInitialization(ref msg) => &msg.session,
			GenerationMessage::KeysDissemination(ref msg) => &msg.session,
			GenerationMessage::PublicKeyShare(ref msg) => &msg.session,
			GenerationMessage::SessionError(ref msg) => &msg.session,
			GenerationMessage::SessionCompleted(ref msg) => &msg.session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			GenerationMessage::InitializeSession(ref msg) => msg.session_nonce,
			GenerationMessage::ConfirmInitialization(ref msg) => msg.session_nonce,
			GenerationMessage::CompleteInitialization(ref msg) => msg.session_nonce,
			GenerationMessage::KeysDissemination(ref msg) => msg.session_nonce,
			GenerationMessage::PublicKeyShare(ref msg) => msg.session_nonce,
			GenerationMessage::SessionError(ref msg) => msg.session_nonce,
			GenerationMessage::SessionCompleted(ref msg) => msg.session_nonce,
		}
	}
}

impl EncryptionMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			EncryptionMessage::InitializeEncryptionSession(ref msg) => &msg.session,
			EncryptionMessage::ConfirmEncryptionInitialization(ref msg) => &msg.session,
			EncryptionMessage::EncryptionSessionError(ref msg) => &msg.session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			EncryptionMessage::InitializeEncryptionSession(ref msg) => msg.session_nonce,
			EncryptionMessage::ConfirmEncryptionInitialization(ref msg) => msg.session_nonce,
			EncryptionMessage::EncryptionSessionError(ref msg) => msg.session_nonce,
		}
	}
}

impl DecryptionMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			DecryptionMessage::DecryptionConsensusMessage(ref msg) => &msg.session,
			DecryptionMessage::RequestPartialDecryption(ref msg) => &msg.session,
			DecryptionMessage::PartialDecryption(ref msg) => &msg.session,
			DecryptionMessage::DecryptionSessionError(ref msg) => &msg.session,
			DecryptionMessage::DecryptionSessionCompleted(ref msg) => &msg.session,
			DecryptionMessage::DecryptionSessionDelegation(ref msg) => &msg.session,
			DecryptionMessage::DecryptionSessionDelegationCompleted(ref msg) => &msg.session,
		}
	}

	pub fn sub_session_id(&self) -> &Secret {
		match *self {
			DecryptionMessage::DecryptionConsensusMessage(ref msg) => &msg.sub_session,
			DecryptionMessage::RequestPartialDecryption(ref msg) => &msg.sub_session,
			DecryptionMessage::PartialDecryption(ref msg) => &msg.sub_session,
			DecryptionMessage::DecryptionSessionError(ref msg) => &msg.sub_session,
			DecryptionMessage::DecryptionSessionCompleted(ref msg) => &msg.sub_session,
			DecryptionMessage::DecryptionSessionDelegation(ref msg) => &msg.sub_session,
			DecryptionMessage::DecryptionSessionDelegationCompleted(ref msg) => &msg.sub_session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			DecryptionMessage::DecryptionConsensusMessage(ref msg) => msg.session_nonce,
			DecryptionMessage::RequestPartialDecryption(ref msg) => msg.session_nonce,
			DecryptionMessage::PartialDecryption(ref msg) => msg.session_nonce,
			DecryptionMessage::DecryptionSessionError(ref msg) => msg.session_nonce,
			DecryptionMessage::DecryptionSessionCompleted(ref msg) => msg.session_nonce,
			DecryptionMessage::DecryptionSessionDelegation(ref msg) => msg.session_nonce,
			DecryptionMessage::DecryptionSessionDelegationCompleted(ref msg) => msg.session_nonce,
		}
	}
}

impl SchnorrSigningMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			SchnorrSigningMessage::SchnorrSigningConsensusMessage(ref msg) => &msg.session,
			SchnorrSigningMessage::SchnorrSigningGenerationMessage(ref msg) => &msg.session,
			SchnorrSigningMessage::SchnorrRequestPartialSignature(ref msg) => &msg.session,
			SchnorrSigningMessage::SchnorrPartialSignature(ref msg) => &msg.session,
			SchnorrSigningMessage::SchnorrSigningSessionError(ref msg) => &msg.session,
			SchnorrSigningMessage::SchnorrSigningSessionCompleted(ref msg) => &msg.session,
			SchnorrSigningMessage::SchnorrSigningSessionDelegation(ref msg) => &msg.session,
			SchnorrSigningMessage::SchnorrSigningSessionDelegationCompleted(ref msg) => &msg.session,
		}
	}

	pub fn sub_session_id(&self) -> &Secret {
		match *self {
			SchnorrSigningMessage::SchnorrSigningConsensusMessage(ref msg) => &msg.sub_session,
			SchnorrSigningMessage::SchnorrSigningGenerationMessage(ref msg) => &msg.sub_session,
			SchnorrSigningMessage::SchnorrRequestPartialSignature(ref msg) => &msg.sub_session,
			SchnorrSigningMessage::SchnorrPartialSignature(ref msg) => &msg.sub_session,
			SchnorrSigningMessage::SchnorrSigningSessionError(ref msg) => &msg.sub_session,
			SchnorrSigningMessage::SchnorrSigningSessionCompleted(ref msg) => &msg.sub_session,
			SchnorrSigningMessage::SchnorrSigningSessionDelegation(ref msg) => &msg.sub_session,
			SchnorrSigningMessage::SchnorrSigningSessionDelegationCompleted(ref msg) => &msg.sub_session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			SchnorrSigningMessage::SchnorrSigningConsensusMessage(ref msg) => msg.session_nonce,
			SchnorrSigningMessage::SchnorrSigningGenerationMessage(ref msg) => msg.session_nonce,
			SchnorrSigningMessage::SchnorrRequestPartialSignature(ref msg) => msg.session_nonce,
			SchnorrSigningMessage::SchnorrPartialSignature(ref msg) => msg.session_nonce,
			SchnorrSigningMessage::SchnorrSigningSessionError(ref msg) => msg.session_nonce,
			SchnorrSigningMessage::SchnorrSigningSessionCompleted(ref msg) => msg.session_nonce,
			SchnorrSigningMessage::SchnorrSigningSessionDelegation(ref msg) => msg.session_nonce,
			SchnorrSigningMessage::SchnorrSigningSessionDelegationCompleted(ref msg) => msg.session_nonce,
		}
	}
}

impl EcdsaSigningMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			EcdsaSigningMessage::EcdsaSigningConsensusMessage(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaSignatureNonceGenerationMessage(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaInversionNonceGenerationMessage(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaInversionZeroGenerationMessage(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaSigningInversedNonceCoeffShare(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaRequestPartialSignature(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaPartialSignature(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaSigningSessionError(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaSigningSessionCompleted(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaSigningSessionDelegation(ref msg) => &msg.session,
			EcdsaSigningMessage::EcdsaSigningSessionDelegationCompleted(ref msg) => &msg.session,
		}
	}

	pub fn sub_session_id(&self) -> &Secret {
		match *self {
			EcdsaSigningMessage::EcdsaSigningConsensusMessage(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaSignatureNonceGenerationMessage(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaInversionNonceGenerationMessage(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaInversionZeroGenerationMessage(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaSigningInversedNonceCoeffShare(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaRequestPartialSignature(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaPartialSignature(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaSigningSessionError(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaSigningSessionCompleted(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaSigningSessionDelegation(ref msg) => &msg.sub_session,
			EcdsaSigningMessage::EcdsaSigningSessionDelegationCompleted(ref msg) => &msg.sub_session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			EcdsaSigningMessage::EcdsaSigningConsensusMessage(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaSignatureNonceGenerationMessage(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaInversionNonceGenerationMessage(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaInversionZeroGenerationMessage(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaSigningInversedNonceCoeffShare(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaRequestPartialSignature(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaPartialSignature(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaSigningSessionError(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaSigningSessionCompleted(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaSigningSessionDelegation(ref msg) => msg.session_nonce,
			EcdsaSigningMessage::EcdsaSigningSessionDelegationCompleted(ref msg) => msg.session_nonce,
		}
	}
}

impl ServersSetChangeMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref msg) => &msg.session,
			ServersSetChangeMessage::UnknownSessionsRequest(ref msg) => &msg.session,
			ServersSetChangeMessage::UnknownSessions(ref msg) => &msg.session,
			ServersSetChangeMessage::ShareChangeKeyVersionNegotiation(ref msg) => &msg.session,
			ServersSetChangeMessage::InitializeShareChangeSession(ref msg) => &msg.session,
			ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeDelegate(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeDelegateResponse(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeShareAddMessage(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeError(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeCompleted(ref msg) => &msg.session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::UnknownSessionsRequest(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::UnknownSessions(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ShareChangeKeyVersionNegotiation(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::InitializeShareChangeSession(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeDelegate(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeDelegateResponse(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeShareAddMessage(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeError(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeCompleted(ref msg) => msg.session_nonce,
		}
	}
}

impl ShareAddMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			ShareAddMessage::ShareAddConsensusMessage(ref msg) => &msg.session,
			ShareAddMessage::KeyShareCommon(ref msg) => &msg.session,
			ShareAddMessage::NewKeysDissemination(ref msg) => &msg.session,
			ShareAddMessage::ShareAddError(ref msg) => &msg.session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			ShareAddMessage::ShareAddConsensusMessage(ref msg) => msg.session_nonce,
			ShareAddMessage::KeyShareCommon(ref msg) => msg.session_nonce,
			ShareAddMessage::NewKeysDissemination(ref msg) => msg.session_nonce,
			ShareAddMessage::ShareAddError(ref msg) => msg.session_nonce,
		}
	}
}

impl KeyVersionNegotiationMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			KeyVersionNegotiationMessage::RequestKeyVersions(ref msg) => &msg.session,
			KeyVersionNegotiationMessage::KeyVersions(ref msg) => &msg.session,
			KeyVersionNegotiationMessage::KeyVersionsError(ref msg) => &msg.session,
		}
	}

	pub fn sub_session_id(&self) -> &Secret {
		match *self {
			KeyVersionNegotiationMessage::RequestKeyVersions(ref msg) => &msg.sub_session,
			KeyVersionNegotiationMessage::KeyVersions(ref msg) => &msg.sub_session,
			KeyVersionNegotiationMessage::KeyVersionsError(ref msg) => &msg.sub_session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			KeyVersionNegotiationMessage::RequestKeyVersions(ref msg) => msg.session_nonce,
			KeyVersionNegotiationMessage::KeyVersions(ref msg) => msg.session_nonce,
			KeyVersionNegotiationMessage::KeyVersionsError(ref msg) => msg.session_nonce,
		}
	}
}

impl fmt::Display for Message {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Message::Cluster(ref message) => write!(f, "Cluster.{}", message),
			Message::Generation(ref message) => write!(f, "Generation.{}", message),
			Message::Encryption(ref message) => write!(f, "Encryption.{}", message),
			Message::Decryption(ref message) => write!(f, "Decryption.{}", message),
			Message::SchnorrSigning(ref message) => write!(f, "SchnorrSigning.{}", message),
			Message::EcdsaSigning(ref message) => write!(f, "EcdsaSigning.{}", message),
			Message::ServersSetChange(ref message) => write!(f, "ServersSetChange.{}", message),
			Message::ShareAdd(ref message) => write!(f, "ShareAdd.{}", message),
			Message::KeyVersionNegotiation(ref message) => write!(f, "KeyVersionNegotiation.{}", message),
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

impl fmt::Display for GenerationMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			GenerationMessage::InitializeSession(_) => write!(f, "InitializeSession"),
			GenerationMessage::ConfirmInitialization(_) => write!(f, "ConfirmInitialization"),
			GenerationMessage::CompleteInitialization(_) => write!(f, "CompleteInitialization"),
			GenerationMessage::KeysDissemination(_) => write!(f, "KeysDissemination"),
			GenerationMessage::PublicKeyShare(_) => write!(f, "PublicKeyShare"),
			GenerationMessage::SessionError(ref msg) => write!(f, "SessionError({})", msg.error),
			GenerationMessage::SessionCompleted(_) => write!(f, "SessionCompleted"),
		}
	}
}

impl fmt::Display for EncryptionMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			EncryptionMessage::InitializeEncryptionSession(_) => write!(f, "InitializeEncryptionSession"),
			EncryptionMessage::ConfirmEncryptionInitialization(_) => write!(f, "ConfirmEncryptionInitialization"),
			EncryptionMessage::EncryptionSessionError(ref msg) => write!(f, "EncryptionSessionError({})", msg.error),
		}
	}
}

impl fmt::Display for ConsensusMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ConsensusMessage::InitializeConsensusSession(_) => write!(f, "InitializeConsensusSession"),
			ConsensusMessage::ConfirmConsensusInitialization(ref msg) => write!(f, "ConfirmConsensusInitialization({})", msg.is_confirmed),
		}
	}
}

impl fmt::Display for ConsensusMessageWithServersSet {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ConsensusMessageWithServersSet::InitializeConsensusSession(_) => write!(f, "InitializeConsensusSession"),
			ConsensusMessageWithServersSet::ConfirmConsensusInitialization(ref msg) => write!(f, "ConfirmConsensusInitialization({})", msg.is_confirmed),
		}
	}
}

impl fmt::Display for ConsensusMessageOfShareAdd {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ConsensusMessageOfShareAdd::InitializeConsensusSession(_) => write!(f, "InitializeConsensusSession"),
			ConsensusMessageOfShareAdd::ConfirmConsensusInitialization(ref msg) => write!(f, "ConfirmConsensusInitialization({})", msg.is_confirmed),
		}
	}
}

impl fmt::Display for DecryptionMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			DecryptionMessage::DecryptionConsensusMessage(ref m) => write!(f, "DecryptionConsensusMessage.{}", m.message),
			DecryptionMessage::RequestPartialDecryption(_) => write!(f, "RequestPartialDecryption"),
			DecryptionMessage::PartialDecryption(_) => write!(f, "PartialDecryption"),
			DecryptionMessage::DecryptionSessionError(_) => write!(f, "DecryptionSessionError"),
			DecryptionMessage::DecryptionSessionCompleted(_) => write!(f, "DecryptionSessionCompleted"),
			DecryptionMessage::DecryptionSessionDelegation(_) => write!(f, "DecryptionSessionDelegation"),
			DecryptionMessage::DecryptionSessionDelegationCompleted(_) => write!(f, "DecryptionSessionDelegationCompleted"),
		}
	}
}

impl fmt::Display for SchnorrSigningMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			SchnorrSigningMessage::SchnorrSigningConsensusMessage(ref m) => write!(f, "SchnorrSigningConsensusMessage.{}", m.message),
			SchnorrSigningMessage::SchnorrSigningGenerationMessage(ref m) => write!(f, "SchnorrSigningGenerationMessage.{}", m.message),
			SchnorrSigningMessage::SchnorrRequestPartialSignature(_) => write!(f, "SchnorrRequestPartialSignature"),
			SchnorrSigningMessage::SchnorrPartialSignature(_) => write!(f, "SchnorrPartialSignature"),
			SchnorrSigningMessage::SchnorrSigningSessionError(_) => write!(f, "SchnorrSigningSessionError"),
			SchnorrSigningMessage::SchnorrSigningSessionCompleted(_) => write!(f, "SchnorrSigningSessionCompleted"),
			SchnorrSigningMessage::SchnorrSigningSessionDelegation(_) => write!(f, "SchnorrSigningSessionDelegation"),
			SchnorrSigningMessage::SchnorrSigningSessionDelegationCompleted(_) => write!(f, "SchnorrSigningSessionDelegationCompleted"),
		}
	}
}

impl fmt::Display for EcdsaSigningMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			EcdsaSigningMessage::EcdsaSigningConsensusMessage(ref m) => write!(f, "EcdsaSigningConsensusMessage.{}", m.message),
			EcdsaSigningMessage::EcdsaSignatureNonceGenerationMessage(ref m) => write!(f, "EcdsaSignatureNonceGenerationMessage.{}", m.message),
			EcdsaSigningMessage::EcdsaInversionNonceGenerationMessage(ref m) => write!(f, "EcdsaInversionNonceGenerationMessage.{}", m.message),
			EcdsaSigningMessage::EcdsaInversionZeroGenerationMessage(ref m) => write!(f, "EcdsaInversionZeroGenerationMessage.{}", m.message),
			EcdsaSigningMessage::EcdsaSigningInversedNonceCoeffShare(_) => write!(f, "EcdsaSigningInversedNonceCoeffShare"),
			EcdsaSigningMessage::EcdsaRequestPartialSignature(_) => write!(f, "EcdsaRequestPartialSignature"),
			EcdsaSigningMessage::EcdsaPartialSignature(_) => write!(f, "EcdsaPartialSignature"),
			EcdsaSigningMessage::EcdsaSigningSessionError(_) => write!(f, "EcdsaSigningSessionError"),
			EcdsaSigningMessage::EcdsaSigningSessionCompleted(_) => write!(f, "EcdsaSigningSessionCompleted"),
			EcdsaSigningMessage::EcdsaSigningSessionDelegation(_) => write!(f, "EcdsaSigningSessionDelegation"),
			EcdsaSigningMessage::EcdsaSigningSessionDelegationCompleted(_) => write!(f, "EcdsaSigningSessionDelegationCompleted"),
		}
	}
}

impl fmt::Display for ServersSetChangeMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref m) => write!(f, "ServersSetChangeConsensusMessage.{}", m.message),
			ServersSetChangeMessage::UnknownSessionsRequest(_) => write!(f, "UnknownSessionsRequest"),
			ServersSetChangeMessage::UnknownSessions(_) => write!(f, "UnknownSessions"),
			ServersSetChangeMessage::ShareChangeKeyVersionNegotiation(ref m) => write!(f, "ShareChangeKeyVersionNegotiation.{}", m.message),
			ServersSetChangeMessage::InitializeShareChangeSession(_) => write!(f, "InitializeShareChangeSession"),
			ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(_) => write!(f, "ConfirmShareChangeSessionInitialization"),
			ServersSetChangeMessage::ServersSetChangeDelegate(_) => write!(f, "ServersSetChangeDelegate"),
			ServersSetChangeMessage::ServersSetChangeDelegateResponse(_) => write!(f, "ServersSetChangeDelegateResponse"),
			ServersSetChangeMessage::ServersSetChangeShareAddMessage(ref m) => write!(f, "ServersSetChangeShareAddMessage.{}", m.message),
			ServersSetChangeMessage::ServersSetChangeError(_) => write!(f, "ServersSetChangeError"),
			ServersSetChangeMessage::ServersSetChangeCompleted(_) => write!(f, "ServersSetChangeCompleted"),
		}
	}
}

impl fmt::Display for ShareAddMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ShareAddMessage::ShareAddConsensusMessage(ref m) => write!(f, "ShareAddConsensusMessage.{}", m.message),
			ShareAddMessage::KeyShareCommon(_) => write!(f, "KeyShareCommon"),
			ShareAddMessage::NewKeysDissemination(_) => write!(f, "NewKeysDissemination"),
			ShareAddMessage::ShareAddError(_) => write!(f, "ShareAddError"),

		}
	}
}

impl fmt::Display for KeyVersionNegotiationMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			KeyVersionNegotiationMessage::RequestKeyVersions(_) => write!(f, "RequestKeyVersions"),
			KeyVersionNegotiationMessage::KeyVersions(_) => write!(f, "KeyVersions"),
			KeyVersionNegotiationMessage::KeyVersionsError(_) => write!(f, "KeyVersionsError"),
		}
	}
}
