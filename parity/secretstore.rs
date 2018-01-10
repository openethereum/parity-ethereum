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
use dir::helpers::replace_home;
use ethcore::account_provider::AccountProvider;
use ethcore::client::Client;
use ethkey::{Secret, Public};
use ethsync::SyncProvider;
use util::Address;

/// This node secret key.
#[derive(Debug, PartialEq, Clone)]
pub enum NodeSecretKey {
	/// Stored as plain text in configuration file.
	Plain(Secret),
	/// Stored as account in key store.
	KeyStore(Address),
}

/// Secret store service contract address.
#[derive(Debug, PartialEq, Clone)]
pub enum ContractAddress {
	/// Contract address is read from registry.
	Registry,
	/// Contract address is specified.
	Address(Address),
}

#[derive(Debug, PartialEq, Clone)]
/// Secret store configuration
pub struct Configuration {
	/// Is secret store functionality enabled?
	pub enabled: bool,
	/// Is HTTP API enabled?
	pub http_enabled: bool,
	/// Is ACL check enabled.
	pub acl_check_enabled: bool,
	/// Is auto migrate enabled.
	pub auto_migrate_enabled: bool,
	/// Service contract address.
	pub service_contract_address: Option<ContractAddress>,
	/// This node secret.
	pub self_secret: Option<NodeSecretKey>,
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
	/// Administrator public key.
	pub admin_public: Option<Public>,
}

/// Secret store dependencies
pub struct Dependencies<'a> {
	/// Blockchain client.
	pub client: Arc<Client>,
	/// Sync provider.
	pub sync: Arc<SyncProvider>,
	/// Account provider.
	pub account_provider: Arc<AccountProvider>,
	/// Passed accounts passwords.
	pub accounts_passwords: &'a [String],
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
	use std::sync::Arc;
	use ethcore_secretstore;
	use ethkey::KeyPair;
	use ansi_term::Colour::Red;
	use super::{Configuration, Dependencies, NodeSecretKey, ContractAddress};

	/// Key server
	pub struct KeyServer {
		_key_server: Box<ethcore_secretstore::KeyServer>,
	}

	impl KeyServer {
		/// Create new key server
		pub fn new(mut conf: Configuration, deps: Dependencies) -> Result<Self, String> {
			if !conf.acl_check_enabled {
				warn!("Running SecretStore with disabled ACL check: {}", Red.bold().paint("everyone has access to stored keys"));
			}

			let self_secret: Arc<ethcore_secretstore::NodeKeyPair> = match conf.self_secret.take() {
				Some(NodeSecretKey::Plain(secret)) => Arc::new(ethcore_secretstore::PlainNodeKeyPair::new(
					KeyPair::from_secret(secret).map_err(|e| format!("invalid secret: {}", e))?)),
				Some(NodeSecretKey::KeyStore(account)) => {
					// Check if account exists
					if !deps.account_provider.has_account(account.clone()).unwrap_or(false) {
						return Err(format!("Account {} passed as secret store node key is not found", account));
					}

					// Check if any passwords have been read from the password file(s)
					if deps.accounts_passwords.is_empty() {
						return Err(format!("No password found for the secret store node account {}", account));
					}

					// Attempt to sign in the engine signer.
					let password = deps.accounts_passwords.iter()
						.find(|p| deps.account_provider.sign(account.clone(), Some((*p).clone()), Default::default()).is_ok())
						.ok_or_else(|| format!("No valid password for the secret store node account {}", account))?;
					Arc::new(ethcore_secretstore::KeyStoreNodeKeyPair::new(deps.account_provider, account, password.clone())
						.map_err(|e| format!("{}", e))?)
				},
				None => return Err("self secret is required when using secretstore".into()),
			};

			let key_server_name = format!("{}:{}", conf.interface, conf.port);
			let mut cconf = ethcore_secretstore::ServiceConfiguration {
				listener_address: if conf.http_enabled { Some(ethcore_secretstore::NodeAddress {
					address: conf.http_interface.clone(),
					port: conf.http_port,
				}) } else { None },
				service_contract_address: conf.service_contract_address.map(|c| match c {
					ContractAddress::Registry => ethcore_secretstore::ContractAddress::Registry,
					ContractAddress::Address(address) => ethcore_secretstore::ContractAddress::Address(address),
				}),
				data_path: conf.data_path.clone(),
				acl_check_enabled: conf.acl_check_enabled,
				cluster_config: ethcore_secretstore::ClusterConfiguration {
					threads: 4,
					listener_address: ethcore_secretstore::NodeAddress {
						address: conf.interface.clone(),
						port: conf.port,
					},
					nodes: conf.nodes.into_iter().map(|(p, (ip, port))| (p, ethcore_secretstore::NodeAddress {
						address: ip,
						port: port,
					})).collect(),
					allow_connecting_to_higher_nodes: true,
					admin_public: conf.admin_public,
					auto_migrate_enabled: conf.auto_migrate_enabled,
				},
			};

			cconf.cluster_config.nodes.insert(self_secret.public().clone(), cconf.cluster_config.listener_address.clone());

			let key_server = ethcore_secretstore::start(deps.client, deps.sync, self_secret, cconf)
				.map_err(|e| format!("Error starting KeyServer {}: {}", key_server_name, e))?;

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
			http_enabled: true,
			acl_check_enabled: true,
			auto_migrate_enabled: true,
			service_contract_address: None,
			self_secret: None,
			admin_public: None,
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
