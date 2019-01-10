// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use std::collections::{VecDeque, BTreeSet};
use key_server_cluster::{Error, SessionId, KeyStorage};

/// Queue of share change sessions.
pub struct SessionsQueue {
	/// Sessions, known on this node.
	known_sessions: VecDeque<SessionId>,
	/// Unknown sessions.
	unknown_sessions: VecDeque<SessionId>,
}

impl SessionsQueue {
	/// Create new sessions queue.
	pub fn new(key_storage: &Arc<KeyStorage>, unknown_sessions: BTreeSet<SessionId>) -> Self {
		// TODO [Opt]:
		// 1) known sessions - change to iter
		// 2) unknown sesions - request chunk-by-chunk
		SessionsQueue {
			known_sessions: key_storage.iter().map(|(k, _)| k).collect(),
			unknown_sessions: unknown_sessions.into_iter().collect(),
		}
	}
}

impl Iterator for SessionsQueue {
	type Item = Result<SessionId, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(known_session) = self.known_sessions.pop_front() {
			return Some(Ok(known_session));
		}

		if let Some(unknown_session) = self.unknown_sessions.pop_front() {
			return Some(Ok(unknown_session));
		}

		None
	}
}
