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

/// Parity-specific rpc interface for operations altering the settings.
use std::io;
use std::sync::Arc;

use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use ethcore::mode::Mode;
use ethsync::ManageNetwork;
use fetch::{self, Fetch};
use hash::keccak_buffer;
use updater::{Service as UpdateService};

use jsonrpc_core::{BoxFuture, Error};
use jsonrpc_core::futures::Future;
use v1::helpers::dapps::DappsService;
use v1::helpers::errors;
use v1::traits::ParitySet;
use v1::types::{Bytes, H160, H256, U256, ReleaseInfo, Transaction, LocalDapp};

/// Parity-specific rpc interface for operations altering the settings.
pub struct ParitySetClient<C, M, U, F = fetch::Client> {
	client: Arc<C>,
	miner: Arc<M>,
	updater: Arc<U>,
	net: Arc<ManageNetwork>,
	dapps: Option<Arc<DappsService>>,
	fetch: F,
	eip86_transition: u64,
}

impl<C, M, U, F> ParitySetClient<C, M, U, F>
	where C: MiningBlockChainClient + 'static,
{
	/// Creates new `ParitySetClient` with given `Fetch`.
	pub fn new(
		client: &Arc<C>,
		miner: &Arc<M>,
		updater: &Arc<U>,
		net: &Arc<ManageNetwork>,
		dapps: Option<Arc<DappsService>>,
		fetch: F,
	) -> Self {
		ParitySetClient {
			client: client.clone(),
			miner: miner.clone(),
			updater: updater.clone(),
			net: net.clone(),
			dapps: dapps,
			fetch: fetch,
			eip86_transition: client.eip86_transition(),
		}
	}
}

impl<C, M, U, F> ParitySet for ParitySetClient<C, M, U, F> where
	C: MiningBlockChainClient + 'static,
	M: MinerService + 'static,
	U: UpdateService + 'static,
	F: Fetch + 'static,
{

	fn set_min_gas_price(&self, gas_price: U256) -> Result<bool, Error> {
		self.miner.set_minimal_gas_price(gas_price.into());
		Ok(true)
	}

	fn set_gas_floor_target(&self, target: U256) -> Result<bool, Error> {
		self.miner.set_gas_floor_target(target.into());
		Ok(true)
	}

	fn set_gas_ceil_target(&self, target: U256) -> Result<bool, Error> {
		self.miner.set_gas_ceil_target(target.into());
		Ok(true)
	}

	fn set_extra_data(&self, extra_data: Bytes) -> Result<bool, Error> {
		self.miner.set_extra_data(extra_data.into_vec());
		Ok(true)
	}

	fn set_author(&self, author: H160) -> Result<bool, Error> {
		self.miner.set_author(author.into());
		Ok(true)
	}

	fn set_engine_signer(&self, address: H160, password: String) -> Result<bool, Error> {
		self.miner.set_engine_signer(address.into(), password).map_err(Into::into).map_err(errors::password)?;
		Ok(true)
	}

	fn set_transactions_limit(&self, limit: usize) -> Result<bool, Error> {
		self.miner.set_transactions_limit(limit);
		Ok(true)
	}

	fn set_tx_gas_limit(&self, limit: U256) -> Result<bool, Error> {
		self.miner.set_tx_gas_limit(limit.into());
		Ok(true)
	}

	fn add_reserved_peer(&self, peer: String) -> Result<bool, Error> {
		match self.net.add_reserved_peer(peer) {
			Ok(()) => Ok(true),
			Err(e) => Err(errors::invalid_params("Peer address", e)),
		}
	}

	fn remove_reserved_peer(&self, peer: String) -> Result<bool, Error> {
		match self.net.remove_reserved_peer(peer) {
			Ok(()) => Ok(true),
			Err(e) => Err(errors::invalid_params("Peer address", e)),
		}
	}

	fn drop_non_reserved_peers(&self) -> Result<bool, Error> {
		self.net.deny_unreserved_peers();
		Ok(true)
	}

	fn accept_non_reserved_peers(&self) -> Result<bool, Error> {
		self.net.accept_unreserved_peers();
		Ok(true)
	}

	fn start_network(&self) -> Result<bool, Error> {
		self.net.start_network();
		Ok(true)
	}

	fn stop_network(&self) -> Result<bool, Error> {
		self.net.stop_network();
		Ok(true)
	}

	fn set_mode(&self, mode: String) -> Result<bool, Error> {
		self.client.set_mode(match mode.as_str() {
			"offline" => Mode::Off,
			"dark" => Mode::Dark(300),
			"passive" => Mode::Passive(300, 3600),
			"active" => Mode::Active,
			e => { return Err(errors::invalid_params("mode", e.to_owned())); },
		});
		Ok(true)
	}

	fn set_spec_name(&self, spec_name: String) -> Result<bool, Error> {
		self.client.set_spec_name(spec_name);
		Ok(true)
	}

	fn hash_content(&self, url: String) -> BoxFuture<H256, Error> {
		self.fetch.process(self.fetch.fetch(&url).then(move |result| {
			result
				.map_err(errors::fetch)
				.and_then(|response| {
					keccak_buffer(&mut io::BufReader::new(response)).map_err(errors::fetch)
				})
				.map(Into::into)
		}))
	}

	fn dapps_refresh(&self) -> Result<bool, Error> {
		self.dapps.as_ref().map(|dapps| dapps.refresh_local_dapps()).ok_or_else(errors::dapps_disabled)
	}

	fn dapps_list(&self) -> Result<Vec<LocalDapp>, Error> {
		self.dapps.as_ref().map(|dapps| dapps.list_dapps()).ok_or_else(errors::dapps_disabled)
	}

	fn upgrade_ready(&self) -> Result<Option<ReleaseInfo>, Error> {
		Ok(self.updater.upgrade_ready().map(Into::into))
	}

	fn execute_upgrade(&self) -> Result<bool, Error> {
		Ok(self.updater.execute_upgrade())
	}

	fn remove_transaction(&self, hash: H256) -> Result<Option<Transaction>, Error> {
		let block_number = self.client.chain_info().best_block_number;
		let hash = hash.into();

		Ok(self.miner.remove_pending_transaction(&*self.client, &hash).map(|t| Transaction::from_pending(t, block_number, self.eip86_transition)))
	}
}
