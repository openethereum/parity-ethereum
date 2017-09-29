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

use std::sync::Arc;
use std::collections::{VecDeque, BTreeSet, BTreeMap};
use key_server_cluster::{Error, NodeId, SessionId, KeyStorage, DocumentKeyShare};

/// Session, queued for change.
pub enum QueuedSession {
	/// Session is known on this node.
	Known(SessionId, DocumentKeyShare),
	/// Session is unknown on this node.
	Unknown(SessionId, BTreeSet<NodeId>),
}

/// Queue of share change sessions.
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
	/// Queued session (key) id.
	pub fn id(&self) -> &SessionId {
		match *self {
			QueuedSession::Known(ref session_id, _) => session_id,
			QueuedSession::Unknown(ref session_id, _) => session_id,
		}
	}

	/// OWners of key shares (aka session nodes).
	pub fn nodes(&self) -> BTreeSet<NodeId> {
		match *self {
			QueuedSession::Known(_, ref key_share) => key_share.id_numbers.keys().cloned().collect(),
			QueuedSession::Unknown(_, ref nodes) => nodes.clone(),
		}
	}
}
