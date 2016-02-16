// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
use jsonrpc_core::*;
use ethsync::EthSync;
use v1::traits::Net;

/// Net rpc implementation.
pub struct NetClient {
	sync: Arc<EthSync>,
}

impl NetClient {
	/// Creates new NetClient.
	pub fn new(sync: Arc<EthSync>) -> Self {
		NetClient { sync: sync }
	}
}

impl Net for NetClient {
	fn version(&self, _: Params) -> Result<Value, Error> {
		Ok(Value::U64(self.sync.status().protocol_version as u64))
	}

	fn peer_count(&self, _params: Params) -> Result<Value, Error> {
		Ok(Value::U64(self.sync.status().num_peers as u64))
	}
}
