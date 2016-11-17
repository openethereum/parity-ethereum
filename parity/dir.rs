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

use std::fs;
use std::path::{PathBuf, Path};
use util::{H64, H256};
use util::journaldb::Algorithm;
use helpers::replace_home;

// this const is irrelevent cause we do have migrations now,
// but we still use it for backwards compatibility
const LEGACY_CLIENT_DB_VER_STR: &'static str = "5.3";

#[derive(Debug, PartialEq)]
pub struct Directories {
	pub db: String,
	pub keys: String,
	pub signer: String,
	pub dapps: String,
}

impl Default for Directories {
	fn default() -> Self {
		Directories {
			db: replace_home("$HOME/.parity"),
			keys: replace_home("$HOME/.parity/keys"),
			signer: replace_home("$HOME/.parity/signer"),
			dapps: replace_home("$HOME/.parity/dapps"),
		}
	}
}

impl Directories {
	pub fn create_dirs(&self, dapps_enabled: bool, signer_enabled: bool) -> Result<(), String> {
		try!(fs::create_dir_all(&self.db).map_err(|e| e.to_string()));
		try!(fs::create_dir_all(&self.keys).map_err(|e| e.to_string()));
		if signer_enabled {
			try!(fs::create_dir_all(&self.signer).map_err(|e| e.to_string()));
		}
		if dapps_enabled {
			try!(fs::create_dir_all(&self.dapps).map_err(|e| e.to_string()));
		}
		Ok(())
	}

	/// Database paths.
	pub fn database(&self, genesis_hash: H256, fork_name: Option<String>) -> DatabaseDirectories {
		DatabaseDirectories {
			path: self.db.clone(),
			genesis_hash: genesis_hash,
			fork_name: fork_name,
		}
	}

	/// Get the ipc sockets path
	pub fn ipc_path(&self) -> PathBuf {
		let mut dir = Path::new(&self.db).to_path_buf();
		dir.push("ipc");
		dir
	}
}

#[derive(Debug, PartialEq)]
pub struct DatabaseDirectories {
	pub path: String,
	pub genesis_hash: H256,
	pub fork_name: Option<String>,
}

impl DatabaseDirectories {
	/// Base DB directory for the given fork.
	pub fn fork_path(&self) -> PathBuf {
		let mut dir = Path::new(&self.path).to_path_buf();
		dir.push(format!("{:?}{}", H64::from(self.genesis_hash), self.fork_name.as_ref().map(|f| format!("-{}", f)).unwrap_or_default()));
		dir
	}

	/// Get the root path for database
	pub fn version_path(&self, pruning: Algorithm) -> PathBuf {
		let mut dir = self.fork_path();
		dir.push(format!("v{}-sec-{}", LEGACY_CLIENT_DB_VER_STR, pruning.as_internal_name_str()));
		dir
	}

	/// Get the path for the databases given the genesis_hash and information on the databases.
	pub fn client_path(&self, pruning: Algorithm) -> PathBuf {
		let mut dir = self.version_path(pruning);
		dir.push("db");
		dir
	}

	/// Get user defaults path
	pub fn user_defaults_path(&self) -> PathBuf {
		let mut dir = self.fork_path();
		dir.push("user_defaults");
		dir
	}

	/// Get the path for the snapshot directory given the genesis hash and fork name.
	pub fn snapshot_path(&self) -> PathBuf {
		let mut dir = self.fork_path();
		dir.push("snapshot");
		dir
	}

	/// Get the path for the network directory.
	pub fn network_path(&self) -> PathBuf {
		let mut dir = self.fork_path();
		dir.push("network");
		dir
	}
}

#[cfg(test)]
mod tests {
	use super::Directories;
	use helpers::replace_home;

	#[test]
	fn test_default_directories() {
		let expected = Directories {
			db: replace_home("$HOME/.parity"),
			keys: replace_home("$HOME/.parity/keys"),
			signer: replace_home("$HOME/.parity/signer"),
			dapps: replace_home("$HOME/.parity/dapps"),
		};
		assert_eq!(expected, Directories::default());
	}
}
