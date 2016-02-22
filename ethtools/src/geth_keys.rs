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

//! Geth keys import/export tool

use util::hash::*;
use util::keys::store::SecretStore;
use util::keys::directory::KeyFileContent;
use std::path::{Path, PathBuf};
use std::result::*;
use std::fs;
use std::str::FromStr;
use std::io;
use std::io::Read;
use rustc_serialize::json::Json;

/// Enumerates all geth keys in the directory and returns collection of tuples `(accountId, filename)`
pub fn enumerate_geth_keys(path: &Path) -> Result<Vec<(Address, String)>, io::Error> {
	let mut entries = Vec::new();
	for entry in try!(fs::read_dir(path)) {
		let entry = try!(entry);
		if !try!(fs::metadata(entry.path())).is_dir() {
			match entry.file_name().to_str() {
				Some(name) => {
					let parts: Vec<&str> = name.split("--").collect();
					if parts.len() != 3 { continue; }
					match Address::from_str(parts[2]) {
						Ok(account_id) => { entries.push((account_id, name.to_owned())); }
						Err(e) => { panic!("error: {:?}", e); }
					}
				},
				None => { continue; }
			};
		}
	}
	Ok(entries)
}

/// Geth import error
#[derive(Debug)]
pub enum ImportError {
	/// Io error reading geth file
	IoError(io::Error),
	/// format error
	FormatError,
}

impl From<io::Error> for ImportError {
	fn from (err: io::Error) -> ImportError {
		ImportError::IoError(err)
	}
}

/// Imports one geth key to the store
pub fn import_geth_key(secret_store: &mut SecretStore, geth_keyfile_path: &Path) -> Result<(), ImportError> {
	let mut file = try!(fs::File::open(geth_keyfile_path));
	let mut buf = String::new();
	try!(file.read_to_string(&mut buf));

	let mut json = match Json::from_str(&buf) {
		Ok(parsed_json) => try!(parsed_json.as_object().ok_or(ImportError::FormatError)).clone(),
		Err(_) => { return Err(ImportError::FormatError); }
	};
	let crypto_object = try!(json.get("Crypto").and_then(|crypto| crypto.as_object()).ok_or(ImportError::FormatError)).clone();
	json.insert("crypto".to_owned(), Json::Object(crypto_object.clone()));
	json.remove("Crypto");
	match KeyFileContent::load(&Json::Object(json.clone())) {
		Ok(key_file) => try!(secret_store.import_key(key_file)),
		Err(_) => { return Err(ImportError::FormatError); }
	};
	Ok(())
}

/// Imports all geth keys in the directory
pub fn import_geth_keys(secret_store: &mut SecretStore, geth_keyfiles_directory: &Path) -> Result<(), ImportError> {
	let geth_files = try!(enumerate_geth_keys(geth_keyfiles_directory));
	for &(ref address, ref file_path) in geth_files.iter() {
		let mut path = PathBuf::new();
		path.push(geth_keyfiles_directory);
		path.push(file_path);
		if let Err(e) = import_geth_key(secret_store, Path::new(&path)) {
			warn!("Skipped geth address {}, error importing: {:?}", address, e)
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;
	use util::hash::*;
	use util::keys::store::SecretStore;
	use std::str::FromStr;

	#[test]
	fn can_enumerate() {
		let keys = enumerate_geth_keys(Path::new("res/geth_keystore")).unwrap();
		assert_eq!(2, keys.len());
	}

	#[test]
	fn can_import() {
		let temp = ::devtools::RandomTempPath::create_dir();
		let mut secret_store = SecretStore::new_in(temp.as_path());
		import_geth_key(&mut secret_store, Path::new("res/geth_keystore/UTC--2016-02-17T09-20-45.721400158Z--3f49624084b67849c7b4e805c5988c21a430f9d9")).unwrap();
		let key = secret_store.account(&Address::from_str("3f49624084b67849c7b4e805c5988c21a430f9d9").unwrap());
		assert!(key.is_some());
	}

	#[test]
	fn can_import_directory() {
		let temp = ::devtools::RandomTempPath::create_dir();
		let mut secret_store = SecretStore::new_in(temp.as_path());
		import_geth_keys(&mut secret_store, Path::new("res/geth_keystore")).unwrap();

		let key = secret_store.account(&Address::from_str("3f49624084b67849c7b4e805c5988c21a430f9d9").unwrap());
		assert!(key.is_some());

		let key = secret_store.account(&Address::from_str("5ba4dcf897e97c2bdf8315b9ef26c13c085988cf").unwrap());
		assert!(key.is_some());
	}

	#[test]
	fn can_decrypt_with_imported() {
		use util::keys::store::EncryptedHashMap;
		use util::bytes::*;

		let temp = ::devtools::RandomTempPath::create_dir();
		let mut secret_store = SecretStore::new_in(temp.as_path());
		import_geth_keys(&mut secret_store, Path::new("res/geth_keystore")).unwrap();

		let val = secret_store.get::<Bytes>(&H128::from_str("62a0ad73556d496a8e1c0783d30d3ace").unwrap(), "123");
		assert!(val.is_ok());
		assert_eq!(vec![0u8, 10], val.unwrap());
	}
}
