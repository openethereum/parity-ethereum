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

use std::collections::{BTreeSet, BTreeMap};
use ethkey::{Public, Secret, Signature};
use key_server_cluster::{NodeId, SessionId};

#[derive(Clone, Debug)]
/// All possible messages that can be sent during DKG.
pub enum Message {
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

#[derive(Clone, Debug)]
/// Initialize new DKG session.
pub struct InitializeSession {
	/// Session Id.
	pub session: SessionId,
	/// Derived generation point. Starting from originator, every node must multiply this
	/// point by random scalar (unknown by other nodes). At the end of initialization
	/// `point` will be some (k1 * k2 * ... * kn) * G = `point` where `(k1 * k2 * ... * kn)`
	/// is unknown for every node.
	pub derived_point: Public,
}

#[derive(Clone, Debug)]
/// Confirm DKG session initialization.
pub struct ConfirmInitialization {
	/// Session Id.
	pub session: SessionId,
	/// Derived generation point.
	pub derived_point: Public,
}

#[derive(Clone, Debug)]
/// Broadcast generated point to every other node.
pub struct CompleteInitialization {
	/// Session Id.
	pub session: SessionId,
	/// All session participants along with their identification numbers.
	pub nodes: BTreeMap<NodeId, Secret>,
	/// Decryption threshold. During decryption threshold-of-route.len() nodes must came to
	/// consensus to successfully decrypt message.
	pub threshold: usize,
	/// Derived generation point.
	pub derived_point: Public,
}

#[derive(Clone, Debug)]
/// Generated keys are sent to every node.
pub struct KeysDissemination {
	/// Session Id.
	pub session: SessionId,
	/// Secret 1.
	pub secret1: Secret,
	/// Secret 2.
	pub secret2: Secret,
	/// Public values.
	pub publics: Vec<Public>,
}

#[derive(Clone, Debug)]
/// Complaint against node is broadcasted.
pub struct Complaint {
	/// Session Id.
	pub session: SessionId,
	/// Public values.
	pub against: NodeId,
}

#[derive(Clone, Debug)]
/// Node is responding to complaint.
pub struct ComplaintResponse {
	/// Session Id.
	pub session: SessionId,
	/// Secret 1.
	pub secret1: Secret,
	/// Secret 2.
	pub secret2: Secret,
}

#[derive(Clone, Debug)]
/// Node is sharing its public key share.
pub struct PublicKeyShare {
	/// Session Id.
	pub session: SessionId,
	/// Public key share.
	pub public_share: Public,
}

#[derive(Clone, Debug)]
/// Node is requested to decrypt data, encrypted in given session.
pub struct InitializeDecryptionSession {
	/// Encryption session Id.
	pub session: SessionId,
	/// Decryption session Id.
	pub sub_session: Secret,
	/// Requestor signature.
	pub requestor_signature: Signature,
}

#[derive(Clone, Debug)]
/// Node is responding to decryption request.
pub struct ConfirmDecryptionInitialization {
	/// Encryption session Id.
	pub session: SessionId,
	/// Decryption session Id.
	pub sub_session: Secret,
	/// Is node confirmed to make a decryption?.
	pub is_confirmed: bool,
}

#[derive(Clone, Debug)]
/// Node is requested to do a partial decryption.
pub struct RequestPartialDecryption {
	/// Encryption session Id.
	pub session: SessionId,
	/// Decryption session Id.
	pub sub_session: Secret,
	/// Nodes that are agreed to do a decryption.
	pub nodes: BTreeSet<NodeId>,
}

#[derive(Clone, Debug)]
/// Node has partially decrypted the secret.
pub struct PartialDecryption {
	/// Encryption session Id.
	pub session: SessionId,
	/// Decryption session Id.
	pub sub_session: Secret,
	/// Partially decrypted secret.
	pub shadow_point: Public,
}
