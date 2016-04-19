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
use util::{U256, Address};
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

	fn set_min_gas_price(&self, params: Params) -> Result<Value, Error> {
		from_params::<(U256,)>(params).and_then(|(gas_price,)| {
			take_weak!(self.miner).set_minimal_gas_price(gas_price);
			to_value(&true)
		})
	}

	fn set_gas_floor_target(&self, params: Params) -> Result<Value, Error> {
		from_params::<(U256,)>(params).and_then(|(gas_floor_target,)| {
			take_weak!(self.miner).set_gas_floor_target(gas_floor_target);
			to_value(&true)
		})
	}

	fn set_extra_data(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Bytes,)>(params).and_then(|(extra_data,)| {
			take_weak!(self.miner).set_extra_data(extra_data.to_vec());
			to_value(&true)
		})
	}

	fn set_author(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address,)>(params).and_then(|(author,)| {
			take_weak!(self.miner).set_author(author);
			to_value(&true)
		})
	}

	fn set_transactions_limit(&self, params: Params) -> Result<Value, Error> {
		from_params::<(usize,)>(params).and_then(|(limit,)| {
			take_weak!(self.miner).set_transactions_limit(limit);
			to_value(&true)
		})
	}

	fn transactions_limit(&self, _: Params) -> Result<Value, Error> {
		to_value(&take_weak!(self.miner).transactions_limit())
	}

	fn min_gas_price(&self, _: Params) -> Result<Value, Error> {
		to_value(&take_weak!(self.miner).minimal_gas_price())
	}

	fn extra_data(&self, _: Params) -> Result<Value, Error> {
		to_value(&Bytes::new(take_weak!(self.miner).extra_data()))
	}

	fn gas_floor_target(&self, _: Params) -> Result<Value, Error> {
		to_value(&take_weak!(self.miner).gas_floor_target())
	}
}
