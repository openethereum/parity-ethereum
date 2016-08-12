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
use util::{RotatingLogger, KeyPair};
use util::misc::version_data;
use std::sync::{Arc, Weak};
use std::collections::{BTreeMap};

use ethstore::random_phrase;
use ethsync::{SyncProvider, ManageNetwork};
use ethcore::miner::MinerService;
use ethcore::client::{MiningBlockChainClient};

use jsonrpc_core::*;
use v1::traits::Ethcore;
use v1::types::{Bytes, U256, H160, Peers};
use v1::helpers::{errors, SigningQueue, ConfirmationsQueue, NetworkSettings};
use v1::helpers::params::expect_no_params;

/// Ethcore implementation.
pub struct EthcoreClient<C, M, S: ?Sized> where
	C: MiningBlockChainClient,
	M: MinerService,
	S: SyncProvider {

	client: Weak<C>,
	miner: Weak<M>,
	sync: Weak<S>,
	net: Weak<ManageNetwork>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
	confirmations_queue: Option<Arc<ConfirmationsQueue>>,
}

impl<C, M, S: ?Sized> EthcoreClient<C, M, S> where C: MiningBlockChainClient, M: MinerService, S: SyncProvider {
	/// Creates new `EthcoreClient`.
	pub fn new(
		client: &Arc<C>,
		miner: &Arc<M>,
		sync: &Arc<S>,
		net: &Arc<ManageNetwork>,
		logger: Arc<RotatingLogger>,
		settings: Arc<NetworkSettings>,
		queue: Option<Arc<ConfirmationsQueue>>
		) -> Self {
		EthcoreClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			sync: Arc::downgrade(sync),
			net: Arc::downgrade(net),
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

impl<C, M, S: ?Sized> Ethcore for EthcoreClient<C, M, S> where M: MinerService + 'static, C: MiningBlockChainClient + 'static, S: SyncProvider + 'static {

	fn transactions_limit(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&take_weak!(self.miner).transactions_limit())
	}

	fn min_gas_price(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&U256::from(take_weak!(self.miner).minimal_gas_price()))
	}

	fn extra_data(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&Bytes::new(take_weak!(self.miner).extra_data()))
	}

	fn gas_floor_target(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&U256::from(take_weak!(self.miner).gas_floor_target()))
	}

	fn gas_ceil_target(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&U256::from(take_weak!(self.miner).gas_ceil_target()))
	}

	fn dev_logs(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		let logs = self.logger.logs();
		to_value(&logs.as_slice())
	}

	fn dev_logs_levels(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&self.logger.levels())
	}

	fn net_chain(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&self.settings.chain)
	}

	fn net_peers(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		let sync_status = take_weak!(self.sync).status();
		let net_config = take_weak!(self.net).network_config();

		to_value(&Peers {
			active: sync_status.num_active_peers,
			connected: sync_status.num_peers,
			max: sync_status.current_max_peers(net_config.min_peers, net_config.max_peers),
		})
	}

	fn net_port(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&self.settings.network_port)
	}

	fn node_name(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&self.settings.name)
	}

	fn rpc_settings(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		let mut map = BTreeMap::new();
		map.insert("enabled".to_owned(), Value::Bool(self.settings.rpc_enabled));
		map.insert("interface".to_owned(), Value::String(self.settings.rpc_interface.clone()));
		map.insert("port".to_owned(), Value::U64(self.settings.rpc_port as u64));
		Ok(Value::Object(map))
	}

	fn default_extra_data(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		to_value(&Bytes::new(version_data()))
	}

	fn gas_price_statistics(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		match take_weak!(self.client).gas_price_statistics(100, 8) {
			Ok(stats) => to_value(&stats
				.into_iter()
				.map(|x| to_value(&U256::from(x)).expect("x must be U256; qed"))
				.collect::<Vec<_>>()),
			_ => Err(Error::internal_error()),
		}
	}

	fn unsigned_transactions_count(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		match self.confirmations_queue {
			None => Err(errors::signer_disabled()),
			Some(ref queue) => to_value(&queue.len()),
		}
	}

	fn generate_secret_phrase(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		to_value(&random_phrase(12))
	}

	fn phrase_to_address(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(String,)>(params).and_then(|(phrase,)|
			to_value(&H160::from(KeyPair::from_phrase(&phrase).address()))
		)
	}
}
