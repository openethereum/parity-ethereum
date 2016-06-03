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
use util::panics::PanicHandler;
use die::*;
use rpc_apis;

#[cfg(feature = "dapps")]
pub use ethcore_dapps::Server as WebappServer;
#[cfg(not(feature = "dapps"))]
pub struct WebappServer;

pub struct Configuration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub user: Option<String>,
	pub pass: Option<String>,
	pub dapps_path: String,
}

pub struct Dependencies {
	pub panic_handler: Arc<PanicHandler>,
	pub apis: Arc<rpc_apis::Dependencies>,
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

	Some(setup_dapps_server(deps, configuration.dapps_path, &addr, auth))
}

#[cfg(not(feature = "dapps"))]
pub fn setup_dapps_server(
	_deps: Dependencies,
	_dapps_path: String,
	_url: &SocketAddr,
	_auth: Option<(String, String)>,
) -> ! {
	die!("Your Parity version has been compiled without WebApps support.")
}

#[cfg(feature = "dapps")]
pub fn setup_dapps_server(
	deps: Dependencies,
	dapps_path: String,
	url: &SocketAddr,
	auth: Option<(String, String)>
) -> WebappServer {
	use ethcore_dapps as dapps;

	let server = dapps::ServerBuilder::new(dapps_path);
	let server = rpc_apis::setup_rpc(server, deps.apis.clone(), rpc_apis::ApiSet::UnsafeContext);
	let start_result = match auth {
		None => {
			server.start_unsecure_http(url)
		},
		Some((username, password)) => {
			server.start_basic_auth_http(url, &username, &password)
		},
	};

	match start_result {
		Err(dapps::ServerError::IoError(err)) => die_with_io_error("WebApps", err),
		Err(e) => die!("WebApps: {:?}", e),
		Ok(server) => {
			server.set_panic_handler(move || {
				deps.panic_handler.notify_all("Panic in WebApp thread.".to_owned());
			});
			server
		},
	}

}

