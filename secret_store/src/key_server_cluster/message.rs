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
use super::{SerializableH256, SerializablePublic, SerializableSecret, SerializableSignature, SerializableMessageHash};

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
	/// Signing message.
	Signing(SigningMessage),
	/// Share add message.
	ShareAdd(ShareAddMessage),
	/// Share move message.
	ShareMove(ShareMoveMessage),
	/// Share add message.
	ShareRemove(ShareRemoveMessage),
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
pub enum ConsensusMessageWithServersMap {
	/// Initialize consensus session.
	InitializeConsensusSession(InitializeConsensusSessionWithServersMap),
	/// Confirm/reject consensus session initialization.
	ConfirmConsensusInitialization(ConfirmConsensusInitialization),
}

/// All possible messages that can be sent during share add consensus establishing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsensusMessageWithServersSecretMap {
	/// Initialize consensus session.
	InitializeConsensusSession(InitializeConsensusSessionWithServersSecretMap),
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
}

/// All possible messages that can be sent during signing session.
#[derive(Clone, Debug)]
pub enum SigningMessage {
	/// Consensus establishing message.
	SigningConsensusMessage(SigningConsensusMessage),
	/// Session key generation message.
	SigningGenerationMessage(SigningGenerationMessage),
	/// Request partial signature from node.
	RequestPartialSignature(RequestPartialSignature),
	/// Partial signature is generated.
	PartialSignature(PartialSignature),
	/// Signing error occured.
	SigningSessionError(SigningSessionError),
	/// Signing session completed.
	SigningSessionCompleted(SigningSessionCompleted),
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
	/// Share move message.
	ServersSetChangeShareMoveMessage(ServersSetChangeShareMoveMessage),
	/// Share remove message.
	ServersSetChangeShareRemoveMessage(ServersSetChangeShareRemoveMessage),
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
	/// Absolute term share of secret polynom is sent to new node.
	NewAbsoluteTermShare(NewAbsoluteTermShare),
	/// Generated keys are sent to every node.
	NewKeysDissemination(NewKeysDissemination),
	/// When session error has occured.
	ShareAddError(ShareAddError),
}

/// All possible messages that can be sent during share move session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShareMoveMessage {
	/// Consensus establishing message.
	ShareMoveConsensusMessage(ShareMoveConsensusMessage),
	/// Share move request.
	ShareMoveRequest(ShareMoveRequest),
	/// Share move.
	ShareMove(ShareMove),
	/// Share move confirmation.
	ShareMoveConfirm(ShareMoveConfirm),
	/// When session error has occured.
	ShareMoveError(ShareMoveError),
}

/// All possible messages that can be sent during share remove session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShareRemoveMessage {
	/// Consensus establishing message.
	ShareRemoveConsensusMessage(ShareRemoveConsensusMessage),
	/// Share remove request.
	ShareRemoveRequest(ShareRemoveRequest),
	/// Share remove confirmation.
	ShareRemoveConfirm(ShareRemoveConfirm),
	/// When session error has occured.
	ShareRemoveError(ShareRemoveError),
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
}

/// Initialize new DKG session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeSession {
	/// Session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Session author.
	pub author: SerializablePublic,
	/// All session participants along with their identification numbers.
	pub nodes: BTreeMap<MessageNodeId, SerializableSecret>,
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
	pub error: String,
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
	/// Requestor signature.
	pub requestor_signature: SerializableSignature,
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
	pub error: String,
}

/// Node is asked to be part of consensus group.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeConsensusSession {
	/// Requestor signature.
	pub requestor_signature: SerializableSignature,
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
pub struct InitializeConsensusSessionWithServersSecretMap {
	/// Old nodes set.
	pub old_nodes_set: BTreeSet<MessageNodeId>,
	/// New nodes set.
	pub new_nodes_set: BTreeMap<MessageNodeId, SerializableSecret>,
	/// Old server set, signed by requester.
	pub old_set_signature: SerializableSignature,
	/// New server set, signed by requester.
	pub new_set_signature: SerializableSignature,
}

/// Node is asked to be part of servers-set consensus group.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeConsensusSessionWithServersMap {
	/// Old nodes set.
	pub old_nodes_set: BTreeSet<MessageNodeId>,
	/// New nodes set (keys() = new_nodes_set, values = old nodes [differs from new if share is moved]).
	pub new_nodes_set: BTreeMap<MessageNodeId, MessageNodeId>,
	/// Old server set, signed by requester.
	pub old_set_signature: SerializableSignature,
	/// New server set, signed by requester.
	pub new_set_signature: SerializableSignature,
}

/// Consensus-related signing message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigningConsensusMessage {
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
pub struct SigningGenerationMessage {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Generation message.
	pub message: GenerationMessage,
}

/// Request partial signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestPartialSignature {
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

/// Partial signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartialSignature {
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

/// When signing session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigningSessionError {
	/// Encryption session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: String,
}

/// Signing session completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigningSessionCompleted {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Signing session Id.
	pub sub_session: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
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
	pub error: String,
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

/// Master node opens share initialize session on other nodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeShareChangeSession {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Key id.
	pub key_id: MessageSessionId,
	/// Master node.
	pub master_node_id: MessageNodeId,
	/// Old nodes set.
	pub old_shares_set: BTreeSet<MessageNodeId>,
	/// Shares to add. Values are filled for new nodes only.
	pub shares_to_add: BTreeMap<MessageNodeId, SerializableSecret>,
	/// Shares to move.
	pub shares_to_move: BTreeMap<MessageNodeId, MessageNodeId>,
	/// Shares to remove.
	pub shares_to_remove: BTreeSet<MessageNodeId>,
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

/// Servers set change share move message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeShareMoveMessage {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Unknown session id.
	pub message: ShareMoveMessage,
}

/// Servers set change share remove message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeShareRemoveMessage {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Unknown session id.
	pub message: ShareRemoveMessage,
}

/// When servers set change session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServersSetChangeError {
	/// Servers set change session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: String,
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
	pub message: ConsensusMessageWithServersSecretMap,
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
	pub author: SerializablePublic,
	/// Common (shared) encryption point.
	pub common_point: Option<SerializablePublic>,
	/// Encrypted point.
	pub encrypted_point: Option<SerializablePublic>,
}

/// Absolute term share is passed to new node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewAbsoluteTermShare {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Sender id number.
	pub sender_id: SerializableSecret,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Absolute term share.
	pub absolute_term_share: SerializableSecret,
}

/// Generated keys are sent to every node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewKeysDissemination {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Refreshed secret1 value.
	pub refreshed_secret1: SerializableSecret,
	/// Refreshed public values.
	pub refreshed_publics: Vec<SerializablePublic>,
}

/// When share add session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareAddError {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: String,
}

/// Consensus-related share move session message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareMoveConsensusMessage {
	/// Share move session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Consensus message.
	pub message: ConsensusMessageWithServersMap,
}

/// Share move is requested.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareMoveRequest {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// Share is moved from source to destination.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareMove {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Author of the entry.
	pub author: SerializablePublic,
	/// Decryption threshold.
	pub threshold: usize,
	/// Nodes ids numbers.
	pub id_numbers: BTreeMap<MessageNodeId, SerializableSecret>,
	/// Polynom1.
	pub polynom1: Vec<SerializableSecret>,
	/// Node secret share.
	pub secret_share: SerializableSecret,
	/// Common (shared) encryption point.
	pub common_point: Option<SerializablePublic>,
	/// Encrypted point.
	pub encrypted_point: Option<SerializablePublic>,
}

/// Share move is confirmed (destination node confirms to all other nodes).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareMoveConfirm {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// When share move session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareMoveError {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: String,
}

/// Consensus-related share remove session message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareRemoveConsensusMessage {
	/// Share move session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Consensus message.
	pub message: ConsensusMessageWithServersSet,
}

/// Share remove is requested.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareRemoveRequest {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// Share remove is confirmed (destination node confirms to all other nodes).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareRemoveConfirm {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
}

/// When share remove session error has occured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareRemoveError {
	/// Generation session Id.
	pub session: MessageSessionId,
	/// Session-level nonce.
	pub session_nonce: u64,
	/// Error message.
	pub error: String,
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
		}
	}

	pub fn sub_session_id(&self) -> &Secret {
		match *self {
			DecryptionMessage::DecryptionConsensusMessage(ref msg) => &msg.sub_session,
			DecryptionMessage::RequestPartialDecryption(ref msg) => &msg.sub_session,
			DecryptionMessage::PartialDecryption(ref msg) => &msg.sub_session,
			DecryptionMessage::DecryptionSessionError(ref msg) => &msg.sub_session,
			DecryptionMessage::DecryptionSessionCompleted(ref msg) => &msg.sub_session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			DecryptionMessage::DecryptionConsensusMessage(ref msg) => msg.session_nonce,
			DecryptionMessage::RequestPartialDecryption(ref msg) => msg.session_nonce,
			DecryptionMessage::PartialDecryption(ref msg) => msg.session_nonce,
			DecryptionMessage::DecryptionSessionError(ref msg) => msg.session_nonce,
			DecryptionMessage::DecryptionSessionCompleted(ref msg) => msg.session_nonce,
		}
	}
}

impl SigningMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			SigningMessage::SigningConsensusMessage(ref msg) => &msg.session,
			SigningMessage::SigningGenerationMessage(ref msg) => &msg.session,
			SigningMessage::RequestPartialSignature(ref msg) => &msg.session,
			SigningMessage::PartialSignature(ref msg) => &msg.session,
			SigningMessage::SigningSessionError(ref msg) => &msg.session,
			SigningMessage::SigningSessionCompleted(ref msg) => &msg.session,
		}
	}

	pub fn sub_session_id(&self) -> &Secret {
		match *self {
			SigningMessage::SigningConsensusMessage(ref msg) => &msg.sub_session,
			SigningMessage::SigningGenerationMessage(ref msg) => &msg.sub_session,
			SigningMessage::RequestPartialSignature(ref msg) => &msg.sub_session,
			SigningMessage::PartialSignature(ref msg) => &msg.sub_session,
			SigningMessage::SigningSessionError(ref msg) => &msg.sub_session,
			SigningMessage::SigningSessionCompleted(ref msg) => &msg.sub_session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			SigningMessage::SigningConsensusMessage(ref msg) => msg.session_nonce,
			SigningMessage::SigningGenerationMessage(ref msg) => msg.session_nonce,
			SigningMessage::RequestPartialSignature(ref msg) => msg.session_nonce,
			SigningMessage::PartialSignature(ref msg) => msg.session_nonce,
			SigningMessage::SigningSessionError(ref msg) => msg.session_nonce,
			SigningMessage::SigningSessionCompleted(ref msg) => msg.session_nonce,
		}
	}
}

impl ServersSetChangeMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref msg) => &msg.session,
			ServersSetChangeMessage::UnknownSessionsRequest(ref msg) => &msg.session,
			ServersSetChangeMessage::UnknownSessions(ref msg) => &msg.session,
			ServersSetChangeMessage::InitializeShareChangeSession(ref msg) => &msg.session,
			ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeDelegate(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeDelegateResponse(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeShareAddMessage(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeShareMoveMessage(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeShareRemoveMessage(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeError(ref msg) => &msg.session,
			ServersSetChangeMessage::ServersSetChangeCompleted(ref msg) => &msg.session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::UnknownSessionsRequest(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::UnknownSessions(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::InitializeShareChangeSession(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeDelegate(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeDelegateResponse(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeShareAddMessage(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeShareMoveMessage(ref msg) => msg.session_nonce,
			ServersSetChangeMessage::ServersSetChangeShareRemoveMessage(ref msg) => msg.session_nonce,
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
			ShareAddMessage::NewAbsoluteTermShare(ref msg) => &msg.session,
			ShareAddMessage::NewKeysDissemination(ref msg) => &msg.session,
			ShareAddMessage::ShareAddError(ref msg) => &msg.session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			ShareAddMessage::ShareAddConsensusMessage(ref msg) => msg.session_nonce,
			ShareAddMessage::KeyShareCommon(ref msg) => msg.session_nonce,
			ShareAddMessage::NewAbsoluteTermShare(ref msg) => msg.session_nonce,
			ShareAddMessage::NewKeysDissemination(ref msg) => msg.session_nonce,
			ShareAddMessage::ShareAddError(ref msg) => msg.session_nonce,
		}
	}
}

impl ShareMoveMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			ShareMoveMessage::ShareMoveConsensusMessage(ref msg) => &msg.session,
			ShareMoveMessage::ShareMoveRequest(ref msg) => &msg.session,
			ShareMoveMessage::ShareMove(ref msg) => &msg.session,
			ShareMoveMessage::ShareMoveConfirm(ref msg) => &msg.session,
			ShareMoveMessage::ShareMoveError(ref msg) => &msg.session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			ShareMoveMessage::ShareMoveConsensusMessage(ref msg) => msg.session_nonce,
			ShareMoveMessage::ShareMoveRequest(ref msg) => msg.session_nonce,
			ShareMoveMessage::ShareMove(ref msg) => msg.session_nonce,
			ShareMoveMessage::ShareMoveConfirm(ref msg) => msg.session_nonce,
			ShareMoveMessage::ShareMoveError(ref msg) => msg.session_nonce,
		}
	}
}

impl ShareRemoveMessage {
	pub fn session_id(&self) -> &SessionId {
		match *self {
			ShareRemoveMessage::ShareRemoveConsensusMessage(ref msg) => &msg.session,
			ShareRemoveMessage::ShareRemoveRequest(ref msg) => &msg.session,
			ShareRemoveMessage::ShareRemoveConfirm(ref msg) => &msg.session,
			ShareRemoveMessage::ShareRemoveError(ref msg) => &msg.session,
		}
	}

	pub fn session_nonce(&self) -> u64 {
		match *self {
			ShareRemoveMessage::ShareRemoveConsensusMessage(ref msg) => msg.session_nonce,
			ShareRemoveMessage::ShareRemoveRequest(ref msg) => msg.session_nonce,
			ShareRemoveMessage::ShareRemoveConfirm(ref msg) => msg.session_nonce,
			ShareRemoveMessage::ShareRemoveError(ref msg) => msg.session_nonce,
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
			Message::Signing(ref message) => write!(f, "Signing.{}", message),
			Message::ServersSetChange(ref message) => write!(f, "ServersSetChange.{}", message),
			Message::ShareAdd(ref message) => write!(f, "ShareAdd.{}", message),
			Message::ShareMove(ref message) => write!(f, "ShareMove.{}", message),
			Message::ShareRemove(ref message) => write!(f, "ShareRemove.{}", message),
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
			ConsensusMessage::ConfirmConsensusInitialization(_) => write!(f, "ConfirmConsensusInitialization"),
		}
	}
}

impl fmt::Display for ConsensusMessageWithServersSet {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ConsensusMessageWithServersSet::InitializeConsensusSession(_) => write!(f, "InitializeConsensusSession"),
			ConsensusMessageWithServersSet::ConfirmConsensusInitialization(_) => write!(f, "ConfirmConsensusInitialization"),
		}
	}
}

impl fmt::Display for ConsensusMessageWithServersMap {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ConsensusMessageWithServersMap::InitializeConsensusSession(_) => write!(f, "InitializeConsensusSession"),
			ConsensusMessageWithServersMap::ConfirmConsensusInitialization(_) => write!(f, "ConfirmConsensusInitialization"),
		}
	}
}

impl fmt::Display for ConsensusMessageWithServersSecretMap {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ConsensusMessageWithServersSecretMap::InitializeConsensusSession(_) => write!(f, "InitializeConsensusSession"),
			ConsensusMessageWithServersSecretMap::ConfirmConsensusInitialization(_) => write!(f, "ConfirmConsensusInitialization"),
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
		}
	}
}

impl fmt::Display for SigningMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			SigningMessage::SigningConsensusMessage(ref m) => write!(f, "SigningConsensusMessage.{}", m.message),
			SigningMessage::SigningGenerationMessage(ref m) => write!(f, "SigningGenerationMessage.{}", m.message),
			SigningMessage::RequestPartialSignature(_) => write!(f, "RequestPartialSignature"),
			SigningMessage::PartialSignature(_) => write!(f, "PartialSignature"),
			SigningMessage::SigningSessionError(_) => write!(f, "SigningSessionError"),
			SigningMessage::SigningSessionCompleted(_) => write!(f, "SigningSessionCompleted"),
		}
	}
}

impl fmt::Display for ServersSetChangeMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref m) => write!(f, "ServersSetChangeConsensusMessage.{}", m.message),
			ServersSetChangeMessage::UnknownSessionsRequest(_) => write!(f, "UnknownSessionsRequest"),
			ServersSetChangeMessage::UnknownSessions(_) => write!(f, "UnknownSessions"),
			ServersSetChangeMessage::InitializeShareChangeSession(_) => write!(f, "InitializeShareChangeSession"),
			ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(_) => write!(f, "ConfirmShareChangeSessionInitialization"),
			ServersSetChangeMessage::ServersSetChangeDelegate(_) => write!(f, "ServersSetChangeDelegate"),
			ServersSetChangeMessage::ServersSetChangeDelegateResponse(_) => write!(f, "ServersSetChangeDelegateResponse"),
			ServersSetChangeMessage::ServersSetChangeShareAddMessage(ref m) => write!(f, "ServersSetChangeShareAddMessage.{}", m.message),
			ServersSetChangeMessage::ServersSetChangeShareMoveMessage(ref m) => write!(f, "ServersSetChangeShareMoveMessage.{}", m.message),
			ServersSetChangeMessage::ServersSetChangeShareRemoveMessage(ref m) => write!(f, "ServersSetChangeShareRemoveMessage.{}", m.message),
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
			ShareAddMessage::NewAbsoluteTermShare(_) => write!(f, "NewAbsoluteTermShare"),
			ShareAddMessage::NewKeysDissemination(_) => write!(f, "NewKeysDissemination"),
			ShareAddMessage::ShareAddError(_) => write!(f, "ShareAddError"),

		}
	}
}

impl fmt::Display for ShareMoveMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ShareMoveMessage::ShareMoveConsensusMessage(ref m) => write!(f, "ShareMoveConsensusMessage.{}", m.message),
			ShareMoveMessage::ShareMoveRequest(_) => write!(f, "ShareMoveRequest"),
			ShareMoveMessage::ShareMove(_) => write!(f, "ShareMove"),
			ShareMoveMessage::ShareMoveConfirm(_) => write!(f, "ShareMoveConfirm"),
			ShareMoveMessage::ShareMoveError(_) => write!(f, "ShareMoveError"),
		}
	}
}

impl fmt::Display for ShareRemoveMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ShareRemoveMessage::ShareRemoveConsensusMessage(ref m) => write!(f, "InitializeShareRemoveSession.{}", m.message),
			ShareRemoveMessage::ShareRemoveRequest(_) => write!(f, "ShareRemoveRequest"),
			ShareRemoveMessage::ShareRemoveConfirm(_) => write!(f, "ShareRemoveConfirm"),
			ShareRemoveMessage::ShareRemoveError(_) => write!(f, "ShareRemoveError"),
		}
	}
}
