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

use std::sync::Arc;
use dir::default_data_path;
use ethcore::client::Client;
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

/// Secret store dependencies
pub struct Dependencies {
	/// Blockchain client.
	pub client: Arc<Client>,
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
	use ethkey;
	use ethcore_secretstore;
	use super::{Configuration, Dependencies};

	/// Key server
	pub struct KeyServer {
		_key_server: Box<ethcore_secretstore::KeyServer>,
	}

	impl KeyServer {
		/// Create new key server
		pub fn new(conf: Configuration, deps: Dependencies) -> Result<Self, String> {
			let key_pairs = vec![
				ethkey::KeyPair::from_secret("6c26a76e9b31048d170873a791401c7e799a11f0cefc0171cc31a49800967509".parse().unwrap()).unwrap(),
				ethkey::KeyPair::from_secret("7e94018b3731afdb3b4e6f4c3e179475640166da12e1d1b0c7d80729b1a5b452".parse().unwrap()).unwrap(),
				ethkey::KeyPair::from_secret("5ab6ed2a52c33142380032c39a03a86b12eacb3fa4b53bc16d84f51318156f8c".parse().unwrap()).unwrap(),
			];
			let conf = ethcore_secretstore::ServiceConfiguration {
				listener_address: ethcore_secretstore::NodeAddress {
					address: conf.interface.clone(),
					port: conf.port,
				},
				data_path: conf.data_path.clone(),
				// TODO: this is test configuration. how it will be configured in production?
				cluster_config: ethcore_secretstore::ClusterConfiguration {
					threads: 4,
					self_private: (***key_pairs[(conf.port - 8082) as usize].secret()).into(),
					listener_address: ethcore_secretstore::NodeAddress {
						address: conf.interface.clone(),
						port: conf.port + 10,
					},
					nodes: key_pairs.iter().enumerate().map(|(i, kp)| (kp.public().clone(),
						ethcore_secretstore::NodeAddress {
							address: conf.interface.clone(),
							port: 8082 + 10 + (i as u16),
						})).collect(),
					allow_connecting_to_higher_nodes: true,
					encryption_config: ethcore_secretstore::EncryptionConfiguration {
						key_check_timeout_ms: 1000,
					},
				}
			};

			let key_server = ethcore_secretstore::start(deps.client, conf)
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
