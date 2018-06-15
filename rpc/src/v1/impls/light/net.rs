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

//! Net rpc implementation.
use std::sync::Arc;
use jsonrpc_core::Result;
use sync::LightSyncProvider;
use v1::traits::Net;

/// Net rpc implementation.
pub struct NetClient<S: ?Sized> {
	sync: Arc<S>
}

impl<S: ?Sized> NetClient<S> where S: LightSyncProvider {
	/// Creates new NetClient.
	pub fn new(sync: Arc<S>) -> Self {
		NetClient {
			sync: sync,
		}
	}
}

impl<S: ?Sized + Sync + Send + 'static> Net for NetClient<S> where S: LightSyncProvider {
	fn version(&self) -> Result<String> {
		Ok(format!("{}", self.sync.network_id()).to_owned())
	}

	fn peer_count(&self) -> Result<String> {
		Ok(format!("0x{:x}", self.sync.peer_numbers().connected as u64).to_owned())
	}

	fn is_listening(&self) -> Result<bool> {
		Ok(true)
	}
}
