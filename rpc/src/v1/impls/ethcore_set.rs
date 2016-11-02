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

/// Ethcore-specific rpc interface for operations altering the settings.
use std::sync::{Arc, Weak};
use jsonrpc_core::*;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use ethcore::mode::Mode;
use ethsync::ManageNetwork;
use v1::helpers::errors;
use v1::traits::EthcoreSet;
use v1::types::{Bytes, H160, U256};

/// Ethcore-specific rpc interface for operations altering the settings.
pub struct EthcoreSetClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService
{
	client: Weak<C>,
	miner: Weak<M>,
	net: Weak<ManageNetwork>,
}

impl<C, M> EthcoreSetClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService {
	/// Creates new `EthcoreSetClient`.
	pub fn new(client: &Arc<C>, miner: &Arc<M>, net: &Arc<ManageNetwork>) -> Self {
		EthcoreSetClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			net: Arc::downgrade(net),
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C, M> EthcoreSet for EthcoreSetClient<C, M> where
	C: MiningBlockChainClient + 'static,
	M: MinerService + 'static {

	fn set_min_gas_price(&self, gas_price: U256) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.miner).set_minimal_gas_price(gas_price.into());
		Ok(true)
	}

	fn set_gas_floor_target(&self, target: U256) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.miner).set_gas_floor_target(target.into());
		Ok(true)
	}

	fn set_gas_ceil_target(&self, target: U256) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.miner).set_gas_ceil_target(target.into());
		Ok(true)
	}

	fn set_extra_data(&self, extra_data: Bytes) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.miner).set_extra_data(extra_data.to_vec());
		Ok(true)
	}

	fn set_author(&self, author: H160) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.miner).set_author(author.into());
		Ok(true)
	}

	fn set_transactions_limit(&self, limit: usize) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.miner).set_transactions_limit(limit);
		Ok(true)
	}

	fn set_tx_gas_limit(&self, limit: U256) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.miner).set_tx_gas_limit(limit.into());
		Ok(true)
	}

	fn add_reserved_peer(&self, peer: String) -> Result<bool, Error> {
		try!(self.active());

		match take_weak!(self.net).add_reserved_peer(peer) {
			Ok(()) => Ok(true),
			Err(e) => Err(errors::invalid_params("Peer address", e)),
		}
	}

	fn remove_reserved_peer(&self, peer: String) -> Result<bool, Error> {
		try!(self.active());

		match take_weak!(self.net).remove_reserved_peer(peer) {
			Ok(()) => Ok(true),
			Err(e) => Err(errors::invalid_params("Peer address", e)),
		}
	}

	fn drop_non_reserved_peers(&self) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.net).deny_unreserved_peers();
		Ok(true)
	}

	fn accept_non_reserved_peers(&self) -> Result<bool, Error> {
		try!(self.active());

		take_weak!(self.net).accept_unreserved_peers();
		Ok(true)
	}

	fn start_network(&self) -> Result<bool, Error> {
		take_weak!(self.net).start_network();
		Ok(true)
	}

	fn stop_network(&self) -> Result<bool, Error> {
		take_weak!(self.net).stop_network();
		Ok(true)
	}

	fn set_mode(&self, mode: String) -> Result<bool, Error> {
		take_weak!(self.client).set_mode(match mode.as_str() {
			"off" => Mode::Off,
			"dark" => Mode::Dark(300),
			"passive" => Mode::Passive(300, 3600),
			"active" => Mode::Active,
			e => { return Err(errors::invalid_params("mode", e.to_owned())); },
		});
		Ok(true)
	}
}
