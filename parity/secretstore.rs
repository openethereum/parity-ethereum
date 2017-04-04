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

use std::collections::BTreeMap;
use std::sync::Arc;
use dir::default_data_path;
use ethcore::client::Client;
use ethkey::{Secret, Public};
use helpers::replace_home;

#[derive(Debug, PartialEq, Clone)]
/// Secret store configuration
pub struct Configuration {
	/// Is secret store functionality enabled?
	pub enabled: bool,
	/// This node secret.
	pub self_secret: Option<Secret>,
	/// Other nodes IDs + addresses.
	pub nodes: BTreeMap<Public, (String, u16)>,
	/// Interface to listen to
	pub interface: String,
	/// Port to listen to
	pub port: u16,
	/// Interface to listen to
	pub http_interface: String,
	/// Port to listen to
	pub http_port: u16,
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
	use ethcore_secretstore;
	use ethkey::KeyPair;
	use super::{Configuration, Dependencies};

	/// Key server
	pub struct KeyServer {
		_key_server: Box<ethcore_secretstore::KeyServer>,
	}

	impl KeyServer {
		/// Create new key server
		pub fn new(conf: Configuration, deps: Dependencies) -> Result<Self, String> {
			let self_secret = conf.self_secret.ok_or("self secret is required when using secretstore")?;
			let mut conf = ethcore_secretstore::ServiceConfiguration {
				listener_address: ethcore_secretstore::NodeAddress {
					address: conf.http_interface.clone(),
					port: conf.http_port,
				},
				data_path: conf.data_path.clone(),
				cluster_config: ethcore_secretstore::ClusterConfiguration {
					threads: 4,
					self_private: (**self_secret).into(),
					listener_address: ethcore_secretstore::NodeAddress {
						address: conf.interface.clone(),
						port: conf.port,
					},
					nodes: conf.nodes.into_iter().map(|(p, (ip, port))| (p, ethcore_secretstore::NodeAddress {
						address: ip,
						port: port,
					})).collect(),
					allow_connecting_to_higher_nodes: true,
					encryption_config: ethcore_secretstore::EncryptionConfiguration {
						key_check_timeout_ms: 1000,
					},
				},
			};

			let self_key_pair = KeyPair::from_secret(self_secret.clone())
				.map_err(|e| format!("valid secret is required when using secretstore. Error: {}", e))?;
			conf.cluster_config.nodes.insert(self_key_pair.public().clone(), conf.cluster_config.listener_address.clone());

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
			self_secret: None,
			nodes: BTreeMap::new(),
			interface: "127.0.0.1".to_owned(),
			port: 8083,
			http_interface: "127.0.0.1".to_owned(),
			http_port: 8082,
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
