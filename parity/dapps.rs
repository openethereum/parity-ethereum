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

use std::path::PathBuf;
use std::sync::Arc;

use dir::default_data_path;
use ethcore::client::Client;
use ethsync::SyncProvider;
use hash_fetch::fetch::Client as FetchClient;
use helpers::replace_home;
use rpc_apis::SignerService;
use parity_reactor;

#[derive(Debug, PartialEq, Clone)]
pub struct Configuration {
	pub enabled: bool,
	pub dapps_path: PathBuf,
	pub extra_dapps: Vec<PathBuf>,
}

impl Default for Configuration {
	fn default() -> Self {
		let data_dir = default_data_path();
		Configuration {
			enabled: true,
			dapps_path: replace_home(&data_dir, "$BASE/dapps").into(),
			extra_dapps: vec![],
		}
	}
}

pub struct Dependencies {
	pub client: Arc<Client>,
	pub sync: Arc<SyncProvider>,
	pub remote: parity_reactor::TokioRemote,
	pub fetch: FetchClient,
	pub signer: Arc<SignerService>,
}

pub fn new(configuration: Configuration, deps: Dependencies) -> Result<Option<Middleware>, String> {
	if !configuration.enabled {
		return Ok(None);
	}

	dapps_middleware(
		deps,
		configuration.dapps_path,
		configuration.extra_dapps,
	).map(Some)
}

pub use self::server::Middleware;
pub use self::server::dapps_middleware;

#[cfg(not(feature = "dapps"))]
mod server {
	use super::Dependencies;
	use std::path::PathBuf;
	use ethcore_rpc::{hyper, RequestMiddleware, RequestMiddlewareAction};

	pub struct Middleware;

	impl RequestMiddleware for Middleware {
		fn on_request(
			&self, req: &hyper::server::Request<hyper::net::HttpStream>, control: &hyper::Control
		) -> RequestMiddlewareAction {
			unreachable!()
		}
	}

	pub fn dapps_middleware(
		_deps: Dependencies,
		_dapps_path: PathBuf,
		_extra_dapps: Vec<PathBuf>,
	) -> Result<Middleware, String> {
		Err("Your Parity version has been compiled without WebApps support.".into())
	}
}

#[cfg(feature = "dapps")]
mod server {
	use super::Dependencies;
	use std::path::PathBuf;
	use std::sync::Arc;
	use util::{Bytes, Address, U256};

	use ethcore::transaction::{Transaction, Action};
	use ethcore::client::{Client, BlockChainClient, BlockId};
	use ethcore_rpc::is_major_importing;
	use hash_fetch::urlhint::ContractClient;
	use parity_dapps;
	use parity_reactor;

	pub use parity_dapps::Middleware;

	pub fn dapps_middleware(
		deps: Dependencies,
		dapps_path: PathBuf,
		extra_dapps: Vec<PathBuf>,
	) -> Result<Middleware, String> {
		let sync = deps.sync;
		let signer = deps.signer.clone();
		let client = deps.client;
		let parity_remote = parity_reactor::Remote::new(deps.remote.clone());
		let registrar = Arc::new(Registrar { client: client.clone() });
		let sync_status = Arc::new(move || is_major_importing(Some(sync.status().state), client.queue_info()));
		let web_proxy_tokens = Arc::new(move |token| signer.is_valid_web_proxy_access_token(&token));

		Ok(parity_dapps::Middleware::new(
			parity_remote,
			deps.signer.address(),
			dapps_path,
			extra_dapps,
			registrar,
			sync_status,
			web_proxy_tokens,
			deps.fetch.clone(),
		))
	}

	struct Registrar {
		client: Arc<Client>,
	}

	impl ContractClient for Registrar {
		fn registrar(&self) -> Result<Address, String> {
			self.client.additional_params().get("registrar")
				 .ok_or_else(|| "Registrar not defined.".into())
				 .and_then(|registrar| {
					 registrar.parse().map_err(|e| format!("Invalid registrar address: {:?}", e))
				 })
		}

		fn call(&self, address: Address, data: Bytes) -> Result<Bytes, String> {
			let from = Address::default();
			let transaction = Transaction {
				nonce: self.client.latest_nonce(&from),
				action: Action::Call(address),
				gas: U256::from(50_000_000),
				gas_price: U256::default(),
				value: U256::default(),
				data: data,
			}.fake_sign(from);

			self.client.call(&transaction, BlockId::Latest, Default::default())
				.map_err(|e| format!("{:?}", e))
				.map(|executed| {
					executed.output
				})
		}
	}
}
