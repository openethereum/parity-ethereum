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

use ethkey::Signature;
use key_server_cluster::{Error};

/// Share addition session API.
pub trait Session: Send + Sync + 'static {
}

/// Share addition session.
pub struct SessionImpl {
}

/// Immutable session data.
struct SessionCore {
}

/// Mutable session data.
struct SessionData {
}

/// SessionImpl creation parameters
pub struct SessionParams {
}

impl SessionImpl {
	/// Create new share addition session.
	pub fn new(params: SessionParams, requester_signature: Option<Signature>) -> Result<Self, Error> {
		unimplemented!()
	}
}
