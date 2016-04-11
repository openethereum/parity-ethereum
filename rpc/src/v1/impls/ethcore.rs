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

//! Ethcore-specific rpc implementation.
use std::sync::{Arc, Weak};
use jsonrpc_core::*;
use ethminer::{MinerService};
use v1::traits::Ethcore;
use v1::types::Bytes;

/// Ethcore implementation.
pub struct EthcoreClient<M>
	where M: MinerService {
	miner: Weak<M>,
}

impl<M> EthcoreClient<M> where M: MinerService {
	/// Creates new `EthcoreClient`.
	pub fn new(miner: &Arc<M>) -> Self {
		EthcoreClient {
			miner: Arc::downgrade(miner)
		}
	}
}

impl<M> Ethcore for EthcoreClient<M> where M: MinerService + 'static {
	fn extra_data(&self, _: Params) -> Result<Value, Error> {
		to_value(&Bytes::new(take_weak!(self.miner).extra_data()))
	}

	fn gas_floor_target(&self, _: Params) -> Result<Value, Error> {
		to_value(&take_weak!(self.miner).gas_floor_target())
	}
}
