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

//! Parity-specific rpc interface for operations altering the settings.
//! Implementation for light client.

use std::io;
use std::sync::Arc;

use ethsync::ManageNetwork;
use fetch::Fetch;
use futures::{BoxFuture, Future};
use util::sha3;

use jsonrpc_core::Error;
use v1::helpers::errors;
use v1::traits::ParitySet;
use v1::types::{Bytes, H160, H256, U256, ReleaseInfo};

/// Parity-specific rpc interface for operations altering the settings.
pub struct ParitySetClient<F> {
	net: Arc<ManageNetwork>,
	fetch: F,
}

impl<F: Fetch> ParitySetClient<F> {
	/// Creates new `ParitySetClient` with given `Fetch`.
	pub fn new(net: Arc<ManageNetwork>, fetch: F) -> Self {
		ParitySetClient {
			net: net,
			fetch: fetch,
		}
	}
}

impl<F: Fetch> ParitySet for ParitySetClient<F> {
	fn set_min_gas_price(&self, _gas_price: U256) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn set_gas_floor_target(&self, _target: U256) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn set_gas_ceil_target(&self, _target: U256) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn set_extra_data(&self, _extra_data: Bytes) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn set_author(&self, _author: H160) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn set_engine_signer(&self, _address: H160, _password: String) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn set_transactions_limit(&self, _limit: usize) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn set_tx_gas_limit(&self, _limit: U256) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
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

	fn set_mode(&self, _mode: String) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn hash_content(&self, url: String) -> BoxFuture<H256, Error> {
		self.fetch.process(self.fetch.fetch(&url).then(move |result| {
			result
				.map_err(errors::from_fetch_error)
				.and_then(|response| {
					sha3(&mut io::BufReader::new(response)).map_err(errors::from_fetch_error)
				})
				.map(Into::into)
		}))
	}

	fn upgrade_ready(&self) -> Result<Option<ReleaseInfo>, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn execute_upgrade(&self) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}
}
