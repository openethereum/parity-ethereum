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
use util::{RotatingLogger};
use util::misc::version_data;
use std::sync::{Arc, Weak};
use std::ops::Deref;
use std::collections::{BTreeMap};
use ethcore::client::{MiningBlockChainClient};
use jsonrpc_core::*;
use ethcore::miner::MinerService;
use v1::traits::Ethcore;
use v1::types::{Bytes, U256};
use v1::helpers::{SigningQueue, ConfirmationsQueue, NetworkSettings};
use v1::impls::signer_disabled_error;

/// Ethcore implementation.
pub struct EthcoreClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService {

	client: Weak<C>,
	miner: Weak<M>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
	confirmations_queue: Option<Arc<ConfirmationsQueue>>,
}

impl<C, M> EthcoreClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	/// Creates new `EthcoreClient`.
	pub fn new(client: &Arc<C>, miner: &Arc<M>, logger: Arc<RotatingLogger>, settings: Arc<NetworkSettings>, queue: Option<Arc<ConfirmationsQueue>>) -> Self {
		EthcoreClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			logger: logger,
			settings: settings,
			confirmations_queue: queue,
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C, M> Ethcore for EthcoreClient<C, M> where M: MinerService + 'static, C: MiningBlockChainClient + 'static {

	fn transactions_limit(&self, _: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&take_weak!(self.miner).transactions_limit())
	}

	fn min_gas_price(&self, _: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&U256::from(take_weak!(self.miner).minimal_gas_price()))
	}

	fn extra_data(&self, _: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&Bytes::new(take_weak!(self.miner).extra_data()))
	}

	fn gas_floor_target(&self, _: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&U256::from(take_weak!(self.miner).gas_floor_target()))
	}

	fn gas_ceil_target(&self, _: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&U256::from(take_weak!(self.miner).gas_ceil_target()))
	}

	fn dev_logs(&self, _params: Params) -> Result<Value, Error> {
		try!(self.active());
		let logs = self.logger.logs();
		to_value(&logs.deref().as_slice())
	}

	fn dev_logs_levels(&self, _params: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&self.logger.levels())
	}

	fn net_chain(&self, _params: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&self.settings.chain)
	}

	fn net_max_peers(&self, _params: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&self.settings.max_peers)
	}

	fn net_port(&self, _params: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&self.settings.network_port)
	}

	fn node_name(&self, _params: Params) -> Result<Value, Error> {
		try!(self.active());
		to_value(&self.settings.name)
	}

	fn rpc_settings(&self, _params: Params) -> Result<Value, Error> {
		try!(self.active());
		let mut map = BTreeMap::new();
		map.insert("enabled".to_owned(), Value::Bool(self.settings.rpc_enabled));
		map.insert("interface".to_owned(), Value::String(self.settings.rpc_interface.clone()));
		map.insert("port".to_owned(), Value::U64(self.settings.rpc_port as u64));
		Ok(Value::Object(map))
	}

	fn default_extra_data(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		match params {
			Params::None => to_value(&Bytes::new(version_data())),
			_ => Err(Error::invalid_params()),
		}
	}

	fn gas_price_statistics(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		match params {
			Params::None => match take_weak!(self.client).gas_price_statistics(100, 8) {
				Ok(stats) => to_value(&stats
					.into_iter()
					.map(|x| to_value(&U256::from(x)).expect("x must be U256; qed"))
					.collect::<Vec<_>>()),
				_ => Err(Error::internal_error()),
			},
			_ => Err(Error::invalid_params()),
		}
	}

	fn unsigned_transactions_count(&self, _params: Params) -> Result<Value, Error> {
		try!(self.active());
		match self.confirmations_queue {
			None => Err(signer_disabled_error()),
			Some(ref queue) => to_value(&queue.len()),
		}
	}
}
