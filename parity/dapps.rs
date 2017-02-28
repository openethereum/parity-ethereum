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
use ethcore_rpc::informant::RpcStats;
use ethsync::SyncProvider;
use hash_fetch::fetch::Client as FetchClient;
use helpers::replace_home;
use io::PanicHandler;
use jsonrpc_core::reactor::Remote;
use rpc_apis::{self, SignerService};

#[derive(Debug, PartialEq, Clone)]
pub struct Configuration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub hosts: Option<Vec<String>>,
	pub cors: Option<Vec<String>>,
	pub user: Option<String>,
	pub pass: Option<String>,
	pub dapps_path: PathBuf,
	pub extra_dapps: Vec<PathBuf>,
	pub all_apis: bool,
}

impl Default for Configuration {
	fn default() -> Self {
		let data_dir = default_data_path();
		Configuration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8080,
			hosts: Some(Vec::new()),
			cors: None,
			user: None,
			pass: None,
			dapps_path: replace_home(&data_dir, "$BASE/dapps").into(),
			extra_dapps: vec![],
			all_apis: false,
		}
	}
}

pub struct Dependencies {
	pub panic_handler: Arc<PanicHandler>,
	pub apis: Arc<rpc_apis::Dependencies>,
	pub client: Arc<Client>,
	pub sync: Arc<SyncProvider>,
	pub remote: Remote,
	pub fetch: FetchClient,
	pub signer: Arc<SignerService>,
	pub stats: Arc<RpcStats>,
}

pub fn new(configuration: Configuration, deps: Dependencies) -> Result<Option<WebappServer>, String> {
	if !configuration.enabled {
		return Ok(None);
	}

	let url = format!("{}:{}", configuration.interface, configuration.port);
	let addr = url.parse().map_err(|_| format!("Invalid Webapps listen host/port given: {}", url))?;

	let auth = configuration.user.as_ref().map(|username| {
		let password = configuration.pass.as_ref().map_or_else(|| {
			use rpassword::read_password;
			println!("Type password for WebApps server (user: {}): ", username);
			let pass = read_password().unwrap();
			println!("OK, got it. Starting server...");
			pass
		}, |pass| pass.to_owned());
		(username.to_owned(), password)
	});

	Ok(Some(setup_dapps_server(
		deps,
		configuration.dapps_path,
		configuration.extra_dapps,
		&addr,
		configuration.hosts,
		configuration.cors,
		auth,
		configuration.all_apis,
	)?))
}

pub use self::server::WebappServer;
pub use self::server::setup_dapps_server;

#[cfg(not(feature = "dapps"))]
mod server {
	use super::Dependencies;
	use std::net::SocketAddr;
	use std::path::PathBuf;

	pub struct WebappServer;
	pub fn setup_dapps_server(
		_deps: Dependencies,
		_dapps_path: PathBuf,
		_extra_dapps: Vec<PathBuf>,
		_url: &SocketAddr,
		_allowed_hosts: Option<Vec<String>>,
		_cors: Option<Vec<String>>,
		_auth: Option<(String, String)>,
		_all_apis: bool,
	) -> Result<WebappServer, String> {
		Err("Your Parity version has been compiled without WebApps support.".into())
	}
}

#[cfg(feature = "dapps")]
mod server {
	use super::Dependencies;
	use std::path::PathBuf;
	use std::sync::Arc;
	use std::net::SocketAddr;
	use std::io;
	use util::{Bytes, Address, U256};

	use ansi_term::Colour;
	use ethcore::transaction::{Transaction, Action};
	use ethcore::client::{Client, BlockChainClient, BlockId};
	use ethcore_rpc::is_major_importing;
	use hash_fetch::urlhint::ContractClient;
	use jsonrpc_core::reactor::RpcHandler;
	use parity_reactor;
	use rpc_apis;

	pub use ethcore_dapps::Server as WebappServer;

	pub fn setup_dapps_server(
		deps: Dependencies,
		dapps_path: PathBuf,
		extra_dapps: Vec<PathBuf>,
		url: &SocketAddr,
		allowed_hosts: Option<Vec<String>>,
		cors: Option<Vec<String>>,
		auth: Option<(String, String)>,
		all_apis: bool,
	) -> Result<WebappServer, String> {
		use ethcore_dapps as dapps;

		let server = dapps::ServerBuilder::new(
			&dapps_path,
			Arc::new(Registrar { client: deps.client.clone() }),
			parity_reactor::Remote::new(deps.remote.clone()),
		);

		let sync = deps.sync.clone();
		let client = deps.client.clone();
		let signer = deps.signer.clone();
		let server = server
			.fetch(deps.fetch.clone())
			.sync_status(Arc::new(move || is_major_importing(Some(sync.status().state), client.queue_info())))
			.web_proxy_tokens(Arc::new(move |token| signer.is_valid_web_proxy_access_token(&token)))
			.extra_dapps(&extra_dapps)
			.signer_address(deps.signer.address())
			.allowed_hosts(allowed_hosts)
			.extra_cors_headers(cors);

		let api_set = if all_apis {
			warn!("{}", Colour::Red.bold().paint("*** INSECURE *** Running Dapps with all APIs exposed."));
			info!("If you do not intend this, exit now.");
			rpc_apis::ApiSet::SafeContext
		} else {
			rpc_apis::ApiSet::UnsafeContext
		};
		let apis = rpc_apis::setup_rpc(deps.stats, deps.apis.clone(), api_set);
		let handler = RpcHandler::new(Arc::new(apis), deps.remote);
		let start_result = match auth {
			None => {
				server.start_unsecured_http(url, handler)
			},
			Some((username, password)) => {
				server.start_basic_auth_http(url, &username, &password, handler)
			},
		};

		match start_result {
			Err(dapps::ServerError::IoError(err)) => match err.kind() {
				io::ErrorKind::AddrInUse => Err(format!("WebApps address {} is already in use, make sure that another instance of an Ethereum client is not running or change the address using the --dapps-port and --dapps-interface options.", url)),
				_ => Err(format!("WebApps io error: {}", err)),
			},
			Err(e) => Err(format!("WebApps error: {:?}", e)),
			Ok(server) => {
				let ph = deps.panic_handler;
				server.set_panic_handler(move || {
					ph.notify_all("Panic in WebApp thread.".to_owned());
				});
				Ok(server)
			},
		}
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
