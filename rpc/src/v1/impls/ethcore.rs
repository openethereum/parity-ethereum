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
use std::{fs, io};
use std::sync::{mpsc, Arc, Weak};
use std::str::FromStr;

use util::{RotatingLogger, Address, Mutex, sha3};
use util::misc::version_data;

use crypto::ecies;
use fetch::{Client as FetchClient, Fetch};
use ethkey::{Brain, Generator};
use ethstore::random_phrase;
use ethsync::{SyncProvider, ManageNetwork};
use ethcore::miner::MinerService;
use ethcore::client::{MiningBlockChainClient};
use ethcore::ids::BlockID;

use jsonrpc_core::Error;
use v1::traits::Ethcore;
use v1::types::{Bytes, U256, H160, H256, H512, Peers, Transaction, RpcSettings};
use v1::helpers::{errors, SigningQueue, SignerService, NetworkSettings};
use v1::helpers::dispatch::DEFAULT_MAC;
use v1::helpers::auto_args::Ready;

/// Ethcore implementation.
pub struct EthcoreClient<C, M, S: ?Sized, F=FetchClient> where
	C: MiningBlockChainClient,
	M: MinerService,
	S: SyncProvider,
	F: Fetch {

	client: Weak<C>,
	miner: Weak<M>,
	sync: Weak<S>,
	net: Weak<ManageNetwork>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
	signer: Option<Arc<SignerService>>,
	fetch: Mutex<F>
}

impl<C, M, S: ?Sized> EthcoreClient<C, M, S> where
	C: MiningBlockChainClient,
	M: MinerService,
	S: SyncProvider, {
	/// Creates new `EthcoreClient` with default `Fetch`.
	pub fn new(
		client: &Arc<C>,
		miner: &Arc<M>,
		sync: &Arc<S>,
		net: &Arc<ManageNetwork>,
		logger: Arc<RotatingLogger>,
		settings: Arc<NetworkSettings>,
		signer: Option<Arc<SignerService>>
	) -> Self {
		Self::with_fetch(client, miner, sync, net, logger, settings, signer)
	}
}

impl<C, M, S: ?Sized, F> EthcoreClient<C, M, S, F> where
	C: MiningBlockChainClient,
	M: MinerService,
	S: SyncProvider,
	F: Fetch, {

	/// Creates new `EthcoreClient` with customizable `Fetch`.
	pub fn with_fetch(
		client: &Arc<C>,
		miner: &Arc<M>,
		sync: &Arc<S>,
		net: &Arc<ManageNetwork>,
		logger: Arc<RotatingLogger>,
		settings: Arc<NetworkSettings>,
		signer: Option<Arc<SignerService>>
		) -> Self {
		EthcoreClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			sync: Arc::downgrade(sync),
			net: Arc::downgrade(net),
			logger: logger,
			settings: settings,
			signer: signer,
			fetch: Mutex::new(F::default()),
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C, M, S: ?Sized, F> Ethcore for EthcoreClient<C, M, S, F> where
	M: MinerService + 'static,
	C: MiningBlockChainClient + 'static,
	S: SyncProvider + 'static,
	F: Fetch + 'static {

	fn transactions_limit(&self) -> Result<usize, Error> {
		try!(self.active());

		Ok(take_weak!(self.miner).transactions_limit())
	}

	fn min_gas_price(&self) -> Result<U256, Error> {
		try!(self.active());

		Ok(U256::from(take_weak!(self.miner).minimal_gas_price()))
	}

	fn extra_data(&self) -> Result<Bytes, Error> {
		try!(self.active());

		Ok(Bytes::new(take_weak!(self.miner).extra_data()))
	}

	fn gas_floor_target(&self) -> Result<U256, Error> {
		try!(self.active());

		Ok(U256::from(take_weak!(self.miner).gas_floor_target()))
	}

	fn gas_ceil_target(&self) -> Result<U256, Error> {
		try!(self.active());

		Ok(U256::from(take_weak!(self.miner).gas_ceil_target()))
	}

	fn dev_logs(&self) -> Result<Vec<String>, Error> {
		try!(self.active());

		let logs = self.logger.logs();
		Ok(logs.as_slice().to_owned())
	}

	fn dev_logs_levels(&self) -> Result<String, Error> {
		try!(self.active());

		Ok(self.logger.levels().to_owned())
	}

	fn net_chain(&self) -> Result<String, Error> {
		try!(self.active());

		Ok(self.settings.chain.clone())
	}

	fn net_peers(&self) -> Result<Peers, Error> {
		try!(self.active());

		let sync = take_weak!(self.sync);
		let sync_status = sync.status();
		let net_config = take_weak!(self.net).network_config();
		let peers = sync.peers().into_iter().map(Into::into).collect();

		Ok(Peers {
			active: sync_status.num_active_peers,
			connected: sync_status.num_peers,
			max: sync_status.current_max_peers(net_config.min_peers, net_config.max_peers),
			peers: peers
		})
	}

	fn net_port(&self) -> Result<u16, Error> {
		try!(self.active());

		Ok(self.settings.network_port)
	}

	fn node_name(&self) -> Result<String, Error> {
		try!(self.active());

		Ok(self.settings.name.clone())
	}

	fn registry_address(&self) -> Result<Option<H160>, Error> {
		try!(self.active());

		Ok(
			take_weak!(self.client)
				.additional_params()
				.get("registrar")
				.and_then(|s| Address::from_str(s).ok())
				.map(|s| H160::from(s))
		)
	}

	fn rpc_settings(&self) -> Result<RpcSettings, Error> {
		try!(self.active());
		Ok(RpcSettings {
			enabled: self.settings.rpc_enabled,
			interface: self.settings.rpc_interface.clone(),
			port: self.settings.rpc_port as u64,
		})
	}

	fn default_extra_data(&self) -> Result<Bytes, Error> {
		try!(self.active());

		Ok(Bytes::new(version_data()))
	}

	fn gas_price_statistics(&self) -> Result<Vec<U256>, Error> {
		try!(self.active());

		match take_weak!(self.client).gas_price_statistics(100, 8) {
			Ok(stats) => Ok(stats.into_iter().map(Into::into).collect()),
			_ => Err(Error::internal_error()),
		}
	}

	fn unsigned_transactions_count(&self) -> Result<usize, Error> {
		try!(self.active());

		match self.signer {
			None => Err(errors::signer_disabled()),
			Some(ref signer) => Ok(signer.len()),
		}
	}

	fn generate_secret_phrase(&self) -> Result<String, Error> {
		try!(self.active());

		Ok(random_phrase(12))
	}

	fn phrase_to_address(&self, phrase: String) -> Result<H160, Error> {
		try!(self.active());

		Ok(Brain::new(phrase).generate().unwrap().address().into())
	}

	fn list_accounts(&self) -> Result<Option<Vec<H160>>, Error> {
		try!(self.active());

		Ok(take_weak!(self.client)
			.list_accounts(BlockID::Latest)
			.map(|a| a.into_iter().map(Into::into).collect()))
	}

	fn list_storage_keys(&self, _address: H160) -> Result<Option<Vec<H256>>, Error> {
		try!(self.active());

		// TODO: implement this
		Ok(None)
	}

	fn encrypt_message(&self, key: H512, phrase: Bytes) -> Result<Bytes, Error> {
		try!(self.active());

		ecies::encrypt(&key.into(), &DEFAULT_MAC, &phrase.0)
			.map_err(errors::encryption_error)
			.map(Into::into)
	}

	fn pending_transactions(&self) -> Result<Vec<Transaction>, Error> {
		try!(self.active());

		Ok(take_weak!(self.miner).all_transactions().into_iter().map(Into::into).collect::<Vec<_>>())
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
					let ready: Ready<H256> = rx.try_recv().expect("When on_done is invoked ready object is always sent.");
					ready.ready(result);
				}));

				// Either invoke ready right away or transfer it to the closure.
				if let Err(e) = res {
					ready.ready(Err(errors::from_fetch_error(e)));
				} else {
					tx.send(ready).expect("Rx end is sent to on_done closure.");
				}
			}
		}
	}
}
