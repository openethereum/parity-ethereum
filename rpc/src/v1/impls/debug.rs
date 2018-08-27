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

//! Debug APIs RPC implementation

use std::sync::Arc;

use ethcore::client::BlockChainClient;

use jsonrpc_core::Result;
use v1::traits::Debug;
use v1::types::RichBlock;

/// Debug rpc implementation.
pub struct DebugClient<C> {
	client: Arc<C>,
}

impl<C> DebugClient<C> {
	/// Creates new debug client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
		}
	}
}

impl<C: BlockChainClient + 'static> Debug for DebugClient<C> {
	fn bad_blocks(&self) -> Result<Vec<RichBlock>> {
		unimplemented!()
	}
}
