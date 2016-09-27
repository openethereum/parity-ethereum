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
use std::collections::{BTreeMap};
use util::{RotatingLogger, Address, Mutex, sha3};
use util::misc::version_data;

use crypto::ecies;
use fetch::{Client as FetchClient, Fetch};
use ethkey::{Brain, Generator};
use ethstore::random_phrase;
use ethsync::{SyncProvider, ManageNetwork};
use ethcore::miner::MinerService;
use ethcore::client::{MiningBlockChainClient};

use jsonrpc_core::{from_params, to_value, Value, Error, Params, Ready};
use v1::traits::Ethcore;
use v1::types::{Bytes, U256, H160, H256, H512, Peers, Transaction};
use v1::helpers::{errors, SigningQueue, SignerService, NetworkSettings};
use v1::helpers::params::expect_no_params;

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

	fn transactions_limit(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&take_weak!(self.miner).transactions_limit()))
	}

	fn min_gas_price(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&U256::from(take_weak!(self.miner).minimal_gas_price())))
	}

	fn extra_data(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&Bytes::new(take_weak!(self.miner).extra_data())))
	}

	fn gas_floor_target(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&U256::from(take_weak!(self.miner).gas_floor_target())))
	}

	fn gas_ceil_target(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&U256::from(take_weak!(self.miner).gas_ceil_target())))
	}

	fn dev_logs(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		let logs = self.logger.logs();
		Ok(to_value(&logs.as_slice()))
	}

	fn dev_logs_levels(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&self.logger.levels()))
	}

	fn net_chain(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&self.settings.chain))
	}

	fn net_peers(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		let sync_status = take_weak!(self.sync).status();
		let net_config = take_weak!(self.net).network_config();

		Ok(to_value(&Peers {
			active: sync_status.num_active_peers,
			connected: sync_status.num_peers,
			max: sync_status.current_max_peers(net_config.min_peers, net_config.max_peers),
		}))
	}

	fn net_port(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&self.settings.network_port))
	}

	fn node_name(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		Ok(to_value(&self.settings.name))
	}

	fn registry_address(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		let r = take_weak!(self.client)
			.additional_params()
			.get("registrar")
			.and_then(|s| Address::from_str(s).ok())
			.map(|s| H160::from(s));
		Ok(to_value(&r))
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
		Ok(to_value(&Bytes::new(version_data())))
	}

	fn gas_price_statistics(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		match take_weak!(self.client).gas_price_statistics(100, 8) {
			Ok(stats) => Ok(to_value(&stats
				.into_iter()
				.map(|x| to_value(&U256::from(x)))
				.collect::<Vec<_>>())),
			_ => Err(Error::internal_error()),
		}
	}

	fn unsigned_transactions_count(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		match self.signer {
			None => Err(errors::signer_disabled()),
			Some(ref signer) => Ok(to_value(&signer.len())),
		}
	}

	fn generate_secret_phrase(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		Ok(to_value(&random_phrase(12)))
	}

	fn phrase_to_address(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(String,)>(params).map(|(phrase,)|
			to_value(&H160::from(Brain::new(phrase).generate().unwrap().address()))
		)
	}

	fn encrypt_message(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(H512, Bytes)>(params).and_then(|(key, phrase)| {
			let s = try!(ecies::encrypt(&key.into(), &[0; 0], &phrase.0).map_err(|_| Error::internal_error()));
			Ok(to_value(&Bytes::from(s)))
		})
	}

	fn pending_transactions(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		Ok(to_value(&take_weak!(self.miner).all_transactions().into_iter().map(Into::into).collect::<Vec<Transaction>>()))
	}

	fn hash_content(&self, params: Params, ready: Ready) {
		let res = self.active().and_then(|_| from_params::<(String,)>(params));

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
			Ok((url, )) => {
				let (tx, rx) = mpsc::channel();
				let res = self.fetch.lock().request_async(&url, Default::default(), Box::new(move |result| {
					let result = hash_content(result)
							.map_err(errors::from_fetch_error)
							.map(|hash| to_value(H256::from(hash)));

					// Receive ready and invoke with result.
					let ready: Ready = rx.try_recv().expect("When on_done is invoked ready object is always sent.");
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
