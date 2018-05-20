// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Parity RPC requests Metadata.
use std::sync::Arc;

use jsonrpc_core;
use jsonrpc_pubsub::{Session, PubSubMetadata};

use v1::types::{DappId, Origin};

/// RPC methods metadata.
#[derive(Clone, Default, Debug)]
pub struct Metadata {
	/// Request origin
	pub origin: Origin,
	/// Request PubSub Session
	pub session: Option<Arc<Session>>,
}

impl Metadata {
	/// Returns dapp id if this request is coming from a Dapp or default `DappId` otherwise.
	pub fn dapp_id(&self) -> DappId {
		// TODO [ToDr] Extract dapp info from Ws connections.
		match self.origin {
			Origin::Dapps(ref dapp) => dapp.clone(),
			Origin::Ws { ref dapp, .. } => dapp.clone(),
			Origin::Signer { ref dapp, .. } => dapp.clone(),
			_ => DappId::default(),
		}
	}

	/// Returns true if the request originates from a Dapp.
	pub fn is_dapp(&self) -> bool {
		if let Origin::Dapps(_) = self.origin {
			true
		} else {
			false
		}
	}
}

impl jsonrpc_core::Metadata for Metadata {}
impl PubSubMetadata for Metadata {
	fn session(&self) -> Option<Arc<Session>> {
		self.session.clone()
	}
}
