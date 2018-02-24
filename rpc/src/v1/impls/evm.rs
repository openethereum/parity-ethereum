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

//! Evm rpc implementation.

use std::sync::Arc;
//use ethereum_types::{U256};

use ethcore::miner::MinerService;

use jsonrpc_core::{Result};

use v1::traits::Evm;
use v1::types::{U256};


/// Evm rpc implementation.
pub struct EvmClient<M> where
	M: MinerService {

	miner: Arc<M>
}

impl<M> EvmClient<M> where
	M: MinerService {

	/// Creates new EvmClient.
	pub fn new(
		miner: &Arc<M>,
	) -> Self {
		EvmClient {
			miner: miner.clone(),
		}
	}

}

impl<M> Evm for EvmClient<M> where
	M: MinerService + 'static,
{
	fn increase_time(&self, increase: U256) -> Result<bool> {
		self.miner.increase_time(increase.into());
		Ok(true)
	}

}
