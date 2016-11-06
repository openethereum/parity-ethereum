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

/// Parity-specific rpc interface for operations altering the settings.
use std::{fs, io};
use std::sync::{Arc, Weak, mpsc};

use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use ethcore::mode::Mode;
use ethsync::ManageNetwork;
use fetch::{Client as FetchClient, Fetch};
use util::{Mutex, sha3};

use jsonrpc_core::Error;
use v1::helpers::auto_args::Ready;
use v1::helpers::errors;
use v1::traits::ParitySet;
use v1::types::{Bytes, H160, H256, U256};

/// Parity-specific rpc interface for operations altering the settings.
pub struct ParitySetClient<C, M, F=FetchClient> where
	C: MiningBlockChainClient,
	M: MinerService,
	F: Fetch,
{
	client: Weak<C>,
	miner: Weak<M>,
	net: Weak<ManageNetwork>,
	fetch: Mutex<F>,
}

impl<C, M> ParitySetClient<C, M, FetchClient> where
	C: MiningBlockChainClient,
	M: MinerService
{
	/// Creates new `ParitySetClient` with default `FetchClient`.
	pub fn new(client: &Arc<C>, miner: &Arc<M>, net: &Arc<ManageNetwork>) -> Self {
		Self::with_fetch(client, miner, net)
	}
}

impl<C, M, F> ParitySetClient<C, M, F> where
	C: MiningBlockChainClient,
	M: MinerService,
	F: Fetch,
{
	/// Creates new `ParitySetClient` with default `FetchClient`.
	pub fn with_fetch(client: &Arc<C>, miner: &Arc<M>, net: &Arc<ManageNetwork>) -> Self {
		ParitySetClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			net: Arc::downgrade(net),
			fetch: Mutex::new(F::default()),
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C, M, F> ParitySet for ParitySetClient<C, M, F> where
	C: MiningBlockChainClient + 'static,
	M: MinerService + 'static,
	F: Fetch + 'static,
{

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
			"offline" => Mode::Off,
			"dark" => Mode::Dark(300),
			"passive" => Mode::Passive(300, 3600),
			"active" => Mode::Active,
			e => { return Err(errors::invalid_params("mode", e.to_owned())); },
		});
		Ok(true)
	}

	fn hash_content(&self, ready: Ready<H256>, url: String) {
		let res = self.active();

		let hash_content = |result| {
			let path = try!(result);
			let mut file = io::BufReader::new(try!(fs::File::open(&path)));
			// Try to hash
			let result = sha3(&mut file);
			// Remove file (always)
			try!(fs::remove_file(&path));
			// Return the result
			Ok(try!(result))
		};

		match res {
			Err(e) => ready.ready(Err(e)),
			Ok(()) => {
				let (tx, rx) = mpsc::channel();
				let res = self.fetch.lock().request_async(&url, Default::default(), Box::new(move |result| {
					let result = hash_content(result)
							.map_err(errors::from_fetch_error)
							.map(Into::into);

					// Receive ready and invoke with result.
					let ready: Ready<H256> = rx.recv().expect(
						"recv() fails when `tx` has been dropped, if this closure is invoked `tx` is not dropped (`res == Ok()`); qed"
					);
					ready.ready(result);
				}));

				// Either invoke ready right away or transfer it to the closure.
				if let Err(e) = res {
					ready.ready(Err(errors::from_fetch_error(e)));
				} else {
					tx.send(ready).expect(
						"send() fails when `rx` end is dropped, if `res == Ok()`: `rx` is moved to the closure; qed"
					);
				}
			}
		}
	}
}
