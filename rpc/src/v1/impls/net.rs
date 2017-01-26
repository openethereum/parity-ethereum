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

//! Net rpc implementation.
use std::sync::{Arc, Weak};
use jsonrpc_core::Error;
use ethsync::SyncProvider;
use v1::traits::Net;

/// Net rpc implementation.
pub struct NetClient<S: ?Sized> where S: SyncProvider {
	sync: Weak<S>
}

impl<S: ?Sized> NetClient<S> where S: SyncProvider {
	/// Creates new NetClient.
	pub fn new(sync: &Arc<S>) -> Self {
		NetClient {
			sync: Arc::downgrade(sync)
		}
	}
}

impl<S: ?Sized> Net for NetClient<S> where S: SyncProvider + 'static {
	fn version(&self) -> Result<String, Error> {
		Ok(format!("{}", take_weak!(self.sync).status().network_id).to_owned())
	}

	fn peer_count(&self) -> Result<String, Error> {
		Ok(format!("0x{:x}", take_weak!(self.sync).status().num_peers as u64).to_owned())
	}

	fn is_listening(&self) -> Result<bool, Error> {
		// right now (11 march 2016), we are always listening for incoming connections
		//
		// (this may not be true now -- 26 september 2016)
		Ok(true)
	}

}
