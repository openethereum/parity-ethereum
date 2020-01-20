// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Net rpc implementation.
use std::sync::Arc;
use jsonrpc_core::Result;
use sync::SyncProvider;
use v1::traits::Net;

/// Net rpc implementation.
pub struct NetClient<S: ?Sized> {
	sync: Arc<S>,
	/// Cached `network_id`.
	///
	/// We cache it to avoid redundant aquire of sync read lock.
	/// https://github.com/paritytech/parity-ethereum/issues/8746
	network_id: u64,
}

impl<S: ?Sized> NetClient<S> where S: SyncProvider {
	/// Creates new NetClient.
	pub fn new(sync: &Arc<S>) -> Self {
		NetClient {
			sync: sync.clone(),
			network_id: sync.status().network_id,
		}
	}
}

impl<S: ?Sized> Net for NetClient<S> where S: SyncProvider + 'static {
	fn version(&self) -> Result<String> {
		Ok(format!("{}", self.network_id))
	}

	fn peer_count(&self) -> Result<String> {
		Ok(format!("{:#x}", self.sync.status().num_peers as u64))
	}

	fn is_listening(&self) -> Result<bool> {
		// right now (11 march 2016), we are always listening for incoming connections
		//
		// (this may not be true now -- 26 september 2016)
		Ok(true)
	}

}
