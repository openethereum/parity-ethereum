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

extern crate app_dirs;
extern crate ethcore_bigint as bigint;
extern crate journaldb;

pub mod helpers;
use std::{env, fs};
use std::path::{PathBuf, Path};
use bigint::hash::{H64, H256};
use journaldb::Algorithm;
use helpers::{replace_home, replace_home_and_local};
use app_dirs::{AppInfo, get_app_root, AppDataType};

#[cfg(target_os = "macos")] const AUTHOR: &'static str = "Parity";
#[cfg(target_os = "macos")] const PRODUCT: &'static str = "io.parity.ethereum";
#[cfg(target_os = "macos")] const PRODUCT_HYPERVISOR: &'static str = "io.parity.ethereum-updates";
#[cfg(target_os = "windows")] const AUTHOR: &'static str = "Parity";
#[cfg(target_os = "windows")] const PRODUCT: &'static str = "Ethereum";
#[cfg(target_os = "windows")] const PRODUCT_HYPERVISOR: &'static str = "EthereumUpdates";
#[cfg(not(any(target_os = "windows", target_os = "macos")))] const AUTHOR: &'static str = "parity";
#[cfg(not(any(target_os = "windows", target_os = "macos")))] const PRODUCT: &'static str = "io.parity.ethereum";
#[cfg(not(any(target_os = "windows", target_os = "macos")))] const PRODUCT_HYPERVISOR: &'static str = "io.parity.ethereum-updates";

#[cfg(target_os = "windows")] pub const CHAINS_PATH: &'static str = "$LOCAL/chains";
#[cfg(not(target_os = "windows"))] pub const CHAINS_PATH: &'static str = "$BASE/chains";

#[cfg(target_os = "windows")] pub const CACHE_PATH: &'static str = "$LOCAL/cache";
#[cfg(not(target_os = "windows"))] pub const CACHE_PATH: &'static str = "$BASE/cache";

// this const is irrelevent cause we do have migrations now,
// but we still use it for backwards compatibility
const LEGACY_CLIENT_DB_VER_STR: &'static str = "5.3";

#[derive(Debug, PartialEq)]
pub struct Directories {
	pub base: String,
	pub db: String,
	pub cache: String,
	pub keys: String,
	pub signer: String,
	pub dapps: String,
	pub secretstore: String,
}

impl Default for Directories {
	fn default() -> Self {
		let data_dir = default_data_path();
		let local_dir = default_local_path();
		Directories {
			base: replace_home(&data_dir, "$BASE"),
			db: replace_home_and_local(&data_dir, &local_dir, CHAINS_PATH),
			cache: replace_home_and_local(&data_dir, &local_dir, CACHE_PATH),
			keys: replace_home(&data_dir, "$BASE/keys"),
			signer: replace_home(&data_dir, "$BASE/signer"),
			dapps: replace_home(&data_dir, "$BASE/dapps"),
			secretstore: replace_home(&data_dir, "$BASE/secretstore"),
		}
	}
}

impl Directories {
	pub fn create_dirs(&self, dapps_enabled: bool, signer_enabled: bool, secretstore_enabled: bool) -> Result<(), String> {
		fs::create_dir_all(&self.base).map_err(|e| e.to_string())?;
		fs::create_dir_all(&self.db).map_err(|e| e.to_string())?;
		fs::create_dir_all(&self.cache).map_err(|e| e.to_string())?;
		fs::create_dir_all(&self.keys).map_err(|e| e.to_string())?;
		if signer_enabled {
			fs::create_dir_all(&self.signer).map_err(|e| e.to_string())?;
		}
		if dapps_enabled {
			fs::create_dir_all(&self.dapps).map_err(|e| e.to_string())?;
		}
		if secretstore_enabled {
			fs::create_dir_all(&self.secretstore).map_err(|e| e.to_string())?;
		}
		Ok(())
	}

	/// Database paths.
	pub fn database(&self, genesis_hash: H256, fork_name: Option<String>, spec_name: String) -> DatabaseDirectories {
		DatabaseDirectories {
			path: self.db.clone(),
			legacy_path: self.base.clone(),
			genesis_hash: genesis_hash,
			fork_name: fork_name,
			spec_name: spec_name,
		}
	}

	/// Get the ipc sockets path
	pub fn ipc_path(&self) -> PathBuf {
		let mut dir = Path::new(&self.base).to_path_buf();
		dir.push("ipc");
		dir
	}

	// TODO: remove in 1.7
	pub fn legacy_keys_path(&self, testnet: bool) -> PathBuf {
		let mut dir = Path::new(&self.base).to_path_buf();
		if testnet {
			dir.push("testnet_keys");
		} else {
			dir.push("keys");
		}
		dir
	}

	pub fn keys_path(&self, spec_name: &str) -> PathBuf {
		let mut dir = PathBuf::from(&self.keys);
		dir.push(spec_name);
		dir
	}
}

#[derive(Debug, PartialEq)]
pub struct DatabaseDirectories {
	pub path: String,
	pub legacy_path: String,
	pub genesis_hash: H256,
	pub fork_name: Option<String>,
	pub spec_name: String,
}

impl DatabaseDirectories {
	/// Base DB directory for the given fork.
	// TODO: remove in 1.7
	pub fn legacy_fork_path(&self) -> PathBuf {
		let mut dir = Path::new(&self.legacy_path).to_path_buf();
		dir.push(format!("{:?}{}", H64::from(self.genesis_hash), self.fork_name.as_ref().map(|f| format!("-{}", f)).unwrap_or_default()));
		dir
	}

	pub fn spec_root_path(&self) -> PathBuf {
		let mut dir = Path::new(&self.path).to_path_buf();
		dir.push(&self.spec_name);
		dir
	}

	pub fn client_path(&self, pruning: Algorithm) -> PathBuf {
		let mut dir = self.db_root_path();
		dir.push(pruning.as_internal_name_str());
		dir.push("db");
		dir
	}

	pub fn db_root_path(&self) -> PathBuf {
		let mut dir = self.spec_root_path();
		dir.push("db");
		dir.push(H64::from(self.genesis_hash).hex());
		dir
	}

	pub fn db_path(&self, pruning: Algorithm) -> PathBuf {
		let mut dir = self.db_root_path();
		dir.push(pruning.as_internal_name_str());
		dir
	}

	/// Get the root path for database
	// TODO: remove in 1.7
	pub fn legacy_version_path(&self, pruning: Algorithm) -> PathBuf {
		let mut dir = self.legacy_fork_path();
		dir.push(format!("v{}-sec-{}", LEGACY_CLIENT_DB_VER_STR, pruning.as_internal_name_str()));
		dir
	}

	/// Get user defaults path
	// TODO: remove in 1.7
	pub fn legacy_user_defaults_path(&self) -> PathBuf {
		let mut dir = self.legacy_fork_path();
		dir.push("user_defaults");
		dir
	}

	/// Get user defaults path
	// TODO: remove in 1.7
	pub fn legacy_snapshot_path(&self) -> PathBuf {
		let mut dir = self.legacy_fork_path();
		dir.push("snapshot");
		dir
	}

	/// Get user defaults path
	// TODO: remove in 1.7
	pub fn legacy_network_path(&self) -> PathBuf {
		let mut dir = self.legacy_fork_path();
		dir.push("network");
		dir
	}

	pub fn user_defaults_path(&self) -> PathBuf {
		let mut dir = self.spec_root_path();
		dir.push("user_defaults");
		dir
	}

	/// Get the path for the snapshot directory given the genesis hash and fork name.
	pub fn snapshot_path(&self) -> PathBuf {
		let mut dir = self.db_root_path();
		dir.push("snapshot");
		dir
	}

	/// Get the path for the network directory.
	pub fn network_path(&self) -> PathBuf {
		let mut dir = self.spec_root_path();
		dir.push("network");
		dir
	}
}

pub fn default_data_path() -> String {
	let app_info = AppInfo { name: PRODUCT, author: AUTHOR };
	get_app_root(AppDataType::UserData, &app_info).map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| "$HOME/.parity".to_owned())
}

pub fn default_local_path() -> String {
	let app_info = AppInfo { name: PRODUCT, author: AUTHOR };
	get_app_root(AppDataType::UserCache, &app_info).map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| "$HOME/.parity".to_owned())
}

pub fn default_hypervisor_path() -> String {
	let app_info = AppInfo { name: PRODUCT_HYPERVISOR, author: AUTHOR };
	get_app_root(AppDataType::UserData, &app_info).map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| "$HOME/.parity-hypervisor".to_owned())
}

fn home() -> PathBuf {
	env::home_dir().expect("Failed to get home dir")
}

/// Geth path
pub fn geth(testnet: bool) -> PathBuf {
	let mut base = geth_base();
	if testnet {
		base.push("testnet");
	}
	base.push("keystore");
	base
}

/// Parity path for specific chain
pub fn parity(chain: &str) -> PathBuf {
	let mut base = parity_base();
	base.push(chain);
	base
}

#[cfg(target_os = "macos")]
fn parity_base() -> PathBuf {
	let mut home = home();
	home.push("Library");
	home.push("Application Support");
	home.push("io.parity.ethereum");
	home.push("keys");
	home
}

#[cfg(windows)]
fn parity_base() -> PathBuf {
	let mut home = home();
	home.push("AppData");
	home.push("Roaming");
	home.push("Parity");
	home.push("Ethereum");
	home.push("keys");
	home
}

#[cfg(not(any(target_os = "macos", windows)))]
fn parity_base() -> PathBuf {
	let mut home = home();
	home.push(".local");
	home.push("share");
	home.push("io.parity.ethereum");
	home.push("keys");
	home
}

#[cfg(target_os = "macos")]
fn geth_base() -> PathBuf {
	let mut home = home();
	home.push("Library");
	home.push("Ethereum");
	home
}

#[cfg(windows)]
fn geth_base() -> PathBuf {
	let mut home = home();
	home.push("AppData");
	home.push("Roaming");
	home.push("Ethereum");
	home
}

#[cfg(not(any(target_os = "macos", windows)))]
fn geth_base() -> PathBuf {
	let mut home = home();
	home.push(".ethereum");
	home
}
#[cfg(test)]
mod tests {
	use super::Directories;
	use helpers::{replace_home, replace_home_and_local};

	#[test]
	fn test_default_directories() {
		let data_dir = super::default_data_path();
		let local_dir = super::default_local_path();
		let expected = Directories {
			base: replace_home(&data_dir, "$BASE"),
			db: replace_home_and_local(&data_dir, &local_dir,
				if cfg!(target_os = "windows") { "$LOCAL/chains" }
				else { "$BASE/chains" }
			),
			cache: replace_home_and_local(&data_dir, &local_dir,
				if cfg!(target_os = "windows") { "$LOCAL/cache" }
				else { "$BASE/cache" }
			),
			keys: replace_home(&data_dir, "$BASE/keys"),
			signer: replace_home(&data_dir, "$BASE/signer"),
			dapps: replace_home(&data_dir, "$BASE/dapps"),
			secretstore: replace_home(&data_dir, "$BASE/secretstore"),
		};
		assert_eq!(expected, Directories::default());
	}
}
