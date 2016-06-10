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
use util::RotatingLogger;
use util::network_settings::NetworkSettings;
use util::misc::version_data;
use std::sync::{Arc, Weak};
use std::ops::Deref;
use std::collections::BTreeMap;
use jsonrpc_core::*;
use ethcore::miner::MinerService;
use v1::traits::Ethcore;
use v1::types::{Bytes};

/// Ethcore implementation.
pub struct EthcoreClient<M> where
	M: MinerService {

	miner: Weak<M>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
}

impl<M> EthcoreClient<M> where M: MinerService {
	/// Creates new `EthcoreClient`.
	pub fn new(miner: &Arc<M>, logger: Arc<RotatingLogger>, settings: Arc<NetworkSettings>) -> Self {
		EthcoreClient {
			miner: Arc::downgrade(miner),
			logger: logger,
			settings: settings,
		}
	}
}

impl<M> Ethcore for EthcoreClient<M> where M: MinerService + 'static {

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

	fn dev_logs(&self, _params: Params) -> Result<Value, Error> {
		let logs = self.logger.logs();
		to_value(&logs.deref().as_slice())
	}

	fn dev_logs_levels(&self, _params: Params) -> Result<Value, Error> {
		to_value(&self.logger.levels())
	}

	fn net_chain(&self, _params: Params) -> Result<Value, Error> {
		to_value(&self.settings.chain)
	}

	fn net_max_peers(&self, _params: Params) -> Result<Value, Error> {
		to_value(&self.settings.max_peers)
	}

	fn net_port(&self, _params: Params) -> Result<Value, Error> {
		to_value(&self.settings.network_port)
	}

	fn node_name(&self, _params: Params) -> Result<Value, Error> {
		to_value(&self.settings.name)
	}

	fn rpc_settings(&self, _params: Params) -> Result<Value, Error> {
		let mut map = BTreeMap::new();
		map.insert("enabled".to_owned(), Value::Bool(self.settings.rpc_enabled));
		map.insert("interface".to_owned(), Value::String(self.settings.rpc_interface.clone()));
		map.insert("port".to_owned(), Value::U64(self.settings.rpc_port as u64));
		Ok(Value::Object(map))
	}

	fn default_extra_data(&self, _params: Params) -> Result<Value, Error> {
		let version = version_data();
		to_value(&Bytes::new(version))
	}
}
