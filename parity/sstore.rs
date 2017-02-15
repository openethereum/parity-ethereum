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

#[cfg(not(feature = "sstore"))]
mod server {
	use super::{Configuration, Dependencies};

	/// Noop key server implementation
	pub struct KeyServer {
	}

	impl KeyServer {
		/// Create new noop key server
		pub fn new(_conf: Configuration, _deps: Dependencies) -> Result<Self, String> {
			Ok(KeyServer {})
		}
	}
}

#[cfg(feature="sstore")]
mod server {
	use ethcore_secstore;
	use super::{Configuration, Dependencies};

	/// Key server
	pub struct KeyServer {
		_key_server: Box<ethcore_secstore::KeyServer>,
	}

	impl KeyServer {
		/// Create new key server
		pub fn new(conf: Configuration, _deps: Dependencies) -> Result<Self, String> {
			let conf = ethcore_secstore::traits::ServiceConfiguration {
				listener_addr: conf.interface,
				listener_port: conf.port,
				data_path: conf.data_path,
			};

			let key_server = ethcore_secstore::start(conf)
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
			data_path: replace_home(&data_dir, "$BASE/sstore"),
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
