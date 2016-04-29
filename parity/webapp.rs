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

use std::sync::Arc;
use std::str::FromStr;
use std::net::SocketAddr;
use ethcore::client::Client;
use ethsync::EthSync;
use ethminer::{Miner, ExternalMiner};
use util::RotatingLogger;
use util::panics::PanicHandler;
use util::keys::store::{AccountService};
use util::network_settings::NetworkSettings;
use die::*;

#[cfg(feature = "webapp")]
pub use ethcore_webapp::Server as WebappServer;
#[cfg(not(feature = "webapp"))]
pub struct WebappServer;

pub struct Configuration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub user: Option<String>,
	pub pass: Option<String>,
}

pub struct Dependencies {
	pub panic_handler: Arc<PanicHandler>,
	pub client: Arc<Client>,
	pub sync: Arc<EthSync>,
	pub secret_store: Arc<AccountService>,
	pub miner: Arc<Miner>,
	pub external_miner: Arc<ExternalMiner>,
	pub logger: Arc<RotatingLogger>,
	pub settings: Arc<NetworkSettings>,
}

pub fn new(configuration: Configuration, deps: Dependencies) -> Option<WebappServer> {
	if !configuration.enabled {
		return None;
	}

	let interface = match configuration.interface.as_str() {
		"all" => "0.0.0.0",
		"local" => "127.0.0.1",
		x => x,
	};
	let url = format!("{}:{}", interface, configuration.port);
	let addr = SocketAddr::from_str(&url).unwrap_or_else(|_| die!("{}: Invalid Webapps listen host/port given.", url));

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

	Some(setup_webapp_server(deps, &addr, auth))
}

#[cfg(not(feature = "webapp"))]
pub fn setup_webapp_server(
	_deps: Dependencies,
	_url: &SocketAddr,
	_auth: Option<(String, String)>,
) -> ! {
	die!("Your Parity version has been compiled without WebApps support.")
}

#[cfg(feature = "webapp")]
pub fn setup_webapp_server(
	deps: Dependencies,
	url: &SocketAddr,
	auth: Option<(String, String)>
) -> WebappServer {
	use ethcore_rpc::v1::*;
	use ethcore_webapp as webapp;

	let server = webapp::ServerBuilder::new();
	server.add_delegate(Web3Client::new().to_delegate());
	server.add_delegate(NetClient::new(&deps.sync).to_delegate());
	server.add_delegate(EthClient::new(&deps.client, &deps.sync, &deps.secret_store, &deps.miner, &deps.external_miner).to_delegate());
	server.add_delegate(EthFilterClient::new(&deps.client, &deps.miner).to_delegate());
	server.add_delegate(PersonalClient::new(&deps.secret_store).to_delegate());
	server.add_delegate(EthcoreClient::new(&deps.miner, deps.logger.clone(), deps.settings.clone()).to_delegate());

	let start_result = match auth {
		None => {
			server.start_unsecure_http(url)
		},
		Some((username, password)) => {
			server.start_basic_auth_http(url, &username, &password)
		},
	};

	match start_result {
		Err(webapp::ServerError::IoError(err)) => die_with_io_error("WebApps", err),
		Err(e) => die!("WebApps: {:?}", e),
		Ok(server) => {
			server.set_panic_handler(move || {
				deps.panic_handler.notify_all("Panic in WebApp thread.".to_owned());
			});
			server
		},
	}

}

