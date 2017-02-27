use std::collections::BTreeMap;
use ethkey::{Public, Secret};
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
pub struct ComplaintResponse {
	/// Session Id.
	pub session: SessionId,
	/// Secret 1.
	pub secret1: Secret,
	/// Secret 2.
	pub secret2: Secret,
}

#[derive(Clone, Debug)]
pub struct PublicKeyShare {
	/// Session Id.
	pub session: SessionId,
	/// Public key share.
	pub public_share: Public,
}
