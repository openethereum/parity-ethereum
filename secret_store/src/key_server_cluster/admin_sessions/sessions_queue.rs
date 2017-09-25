use std::sync::Arc;
use std::collections::{VecDeque, BTreeSet, BTreeMap};
use key_server_cluster::{Error, NodeId, SessionId, KeyStorage, DocumentKeyShare};

pub enum QueuedSession {
	Known(SessionId, DocumentKeyShare),
	Unknown(SessionId, BTreeSet<NodeId>),
}

pub struct SessionsQueue {
	/// Key storage.
	key_storage: Arc<KeyStorage>,
	/// Sessions, known on this node.
	known_sessions: VecDeque<SessionId>,
	/// Unknown sessions.
	unknown_sessions: VecDeque<(SessionId, BTreeSet<NodeId>)>,
}

impl SessionsQueue {
	/// Create new sessions queue.
	pub fn new(key_storage: Arc<KeyStorage>, unknown_sessions: BTreeMap<SessionId, BTreeSet<NodeId>>) -> Self {
		// TODO: optimizations:
		// 1) known sessions - change to iter
		// 2) unknown sesions - request chunk-by-chunk
		SessionsQueue {
			key_storage: key_storage.clone(),
			known_sessions: key_storage.iter().map(|(k, _)| k).collect(),
			unknown_sessions: unknown_sessions.into_iter().collect(),
		}
	}
}

impl Iterator for SessionsQueue {
	type Item = Result<QueuedSession, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(known_session) = self.known_sessions.pop_front() {
			return Some(self.key_storage.get(&known_session)
				.map(|session| QueuedSession::Known(known_session, session))
				.map_err(|e| Error::KeyStorage(e.into())));
		}

		if let Some(unknown_session) = self.unknown_sessions.pop_front() {
			return Some(Ok(QueuedSession::Unknown(unknown_session.0, unknown_session.1)));
		}

		None
	}
}

impl QueuedSession {
	pub fn id(&self) -> &SessionId {
		match *self {
			QueuedSession::Known(ref session_id, _) => session_id,
			QueuedSession::Unknown(ref session_id, _) => session_id,
		}
	}

	pub fn nodes(&self) -> BTreeSet<NodeId> {
		match *self {
			QueuedSession::Known(_, ref key_share) => key_share.id_numbers.keys().cloned().collect(),
			QueuedSession::Unknown(_, ref nodes) => nodes.clone(),
		}
	}
}