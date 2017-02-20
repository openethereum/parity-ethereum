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

use dir::default_data_path;
use helpers::replace_home;

#[derive(Debug, PartialEq, Clone)]
/// Secret store configuration
pub struct Configuration {
	/// Is secret store functionality enabled?
	pub enabled: bool,
	/// Interface to listen to
	pub interface: String,
	/// Port to listen to
	pub port: u16,
	/// Data directory path for secret store
	pub data_path: String,
}

#[derive(Debug, PartialEq, Clone)]
/// Secret store dependencies
pub struct Dependencies {
	// the only dependency will be BlockChainClient
}

#[cfg(not(feature = "secretstore"))]
mod server {
	use super::{Configuration, Dependencies};

	/// Noop key server implementation
	pub struct KeyServer;

	impl KeyServer {
		/// Create new noop key server
		pub fn new(_conf: Configuration, _deps: Dependencies) -> Result<Self, String> {
			Ok(KeyServer)
		}
	}
}

#[cfg(feature="secretstore")]
mod server {
	use ethcore_secretstore;
	use super::{Configuration, Dependencies};

	/// Key server
	pub struct KeyServer {
		_key_server: Box<ethcore_secretstore::KeyServer>,
	}

	impl KeyServer {
		/// Create new key server
		pub fn new(conf: Configuration, _deps: Dependencies) -> Result<Self, String> {
			let conf = ethcore_secretstore::ServiceConfiguration {
				listener_addr: conf.interface,
				listener_port: conf.port,
				data_path: conf.data_path,
			};

			let key_server = ethcore_secretstore::start(conf)
				.map_err(Into::<String>::into)?;

			Ok(KeyServer {
				_key_server: key_server,
			})
		}
	}
}

pub use self::server::KeyServer;

impl Default for Configuration {
	fn default() -> Self {
		let data_dir = default_data_path();
		Configuration {
			enabled: true,
			interface: "127.0.0.1".to_owned(),
			port: 8082,
			data_path: replace_home(&data_dir, "$BASE/secretstore"),
		}
	}
}

/// Start secret store-related functionality
pub fn start(conf: Configuration, deps: Dependencies) -> Result<Option<KeyServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	KeyServer::new(conf, deps)
		.map(|s| Some(s))
}
