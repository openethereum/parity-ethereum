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

use common::*;
use keys::store::SecretStore;
use keys::directory::KeyFileContent;
use std::path::PathBuf;

/// Enumerates all geth keys in the directory and returns collection of tuples `(accountId, filename)`
pub fn enumerate_geth_keys(path: &Path) -> Result<Vec<(Address, String)>, ImportError> {
	let mut entries = Vec::new();
	for entry in try!(fs::read_dir(path)) {
		let entry = try!(entry);
		if !try!(fs::metadata(entry.path())).is_dir() {
			match entry.file_name().to_str() {
				Some(name) => {
					let parts: Vec<&str> = name.split("--").collect();
					if parts.len() != 3 { continue; }
					let account_id = try!(Address::from_str(parts[2]).map_err(|_| ImportError::Format));
					entries.push((account_id, name.to_owned()));
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
	Io(io::Error),
	/// format error
	Format,
}

impl From<io::Error> for ImportError {
	fn from (err: io::Error) -> ImportError {
		ImportError::Io(err)
	}
}

/// Imports one geth key to the store
pub fn import_geth_key(secret_store: &mut SecretStore, geth_keyfile_path: &Path) -> Result<(), ImportError> {
	let mut file = try!(fs::File::open(geth_keyfile_path));
	let mut buf = String::new();
	try!(file.read_to_string(&mut buf));

	let mut json_result = Json::from_str(&buf);
	let mut json = match json_result {
		Ok(ref mut parsed_json) => try!(parsed_json.as_object_mut().ok_or(ImportError::Format)),
		Err(_) => { return Err(ImportError::Format); }
	};
	if let Some(crypto_object) = json.get("Crypto").and_then(|crypto| crypto.as_object()).cloned() {
		json.insert("crypto".to_owned(), Json::Object(crypto_object));
		json.remove("Crypto");
	}
	match KeyFileContent::load(&Json::Object(json.clone())) {
		Ok(key_file) => try!(secret_store.import_key(key_file)),
		Err(_) => { return Err(ImportError::Format); }
	};
	Ok(())
}

/// Imports all geth keys in the directory
pub fn import_geth_keys(secret_store: &mut SecretStore, geth_keyfiles_directory: &Path) -> Result<(), ImportError> {
	use std::path::PathBuf;
	let geth_files = try!(enumerate_geth_keys(geth_keyfiles_directory));
	for &(ref address, ref file_path) in &geth_files {
		let mut path = PathBuf::new();
		path.push(geth_keyfiles_directory);
		path.push(file_path);
		if let Err(e) = import_geth_key(secret_store, Path::new(&path)) {
			warn!("Skipped geth address {}, error importing: {:?}", address, e)
		}
	}
	Ok(())
}


/// Gets the default geth keystore directory.
///
/// Based on https://github.com/ethereum/go-ethereum/blob/e553215/common/path.go#L75
pub fn keystore_dir() -> PathBuf {
	#[cfg(target_os = "macos")]
	fn data_dir(mut home: PathBuf) -> PathBuf {
		home.push("Library");
		home.push("Ethereum");
		home
	}
	
	#[cfg(windows)]
	fn data_dir(mut home: PathBuf) -> PathBuf {
		home.push("AppData");
		home.push("Roaming");
		home.push("Ethereum");
		home	
	}
	
	#[cfg(not(any(target_os = "macos", windows)))]
	fn data_dir(mut home: PathBuf) -> PathBuf {
		home.push(".ethereum");
        home
	}
	
	let mut data_dir = data_dir(::std::env::home_dir().expect("Failed to get home dir"));
	data_dir.push("keystore");
	data_dir
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::*;
	use keys::store::SecretStore;

	fn test_path() -> &'static str {
		match ::std::fs::metadata("res") {
			Ok(_) => "res/geth_keystore",
			Err(_) => "util/res/geth_keystore"
		}
	}

	fn test_path_param(param_val: &'static str) -> String {
		test_path().to_owned() + param_val
	}

	#[test]
	fn can_enumerate() {
		let keys = enumerate_geth_keys(Path::new(test_path())).unwrap();
		assert_eq!(3, keys.len());
	}

	#[test]
	fn can_import_geth_old() {
		let temp = ::devtools::RandomTempPath::create_dir();
		let mut secret_store = SecretStore::new_in(temp.as_path());
		import_geth_key(&mut secret_store, Path::new(&test_path_param("/UTC--2016-02-17T09-20-45.721400158Z--3f49624084b67849c7b4e805c5988c21a430f9d9"))).unwrap();
		let key = secret_store.account(&Address::from_str("3f49624084b67849c7b4e805c5988c21a430f9d9").unwrap());
		assert!(key.is_some());
	}

	#[test]
	fn can_import_geth140() {
		let temp = ::devtools::RandomTempPath::create_dir();
		let mut secret_store = SecretStore::new_in(temp.as_path());
		import_geth_key(&mut secret_store, Path::new(&test_path_param("/UTC--2016-04-03T08-58-49.834202900Z--63121b431a52f8043c16fcf0d1df9cb7b5f66649"))).unwrap();
		let key = secret_store.account(&Address::from_str("63121b431a52f8043c16fcf0d1df9cb7b5f66649").unwrap());
		assert!(key.is_some());
	}

	#[test]
	fn can_import_directory() {
		let temp = ::devtools::RandomTempPath::create_dir();
		let mut secret_store = SecretStore::new_in(temp.as_path());
		import_geth_keys(&mut secret_store, Path::new(test_path())).unwrap();

		let key = secret_store.account(&Address::from_str("3f49624084b67849c7b4e805c5988c21a430f9d9").unwrap());
		assert!(key.is_some());

		let key = secret_store.account(&Address::from_str("5ba4dcf897e97c2bdf8315b9ef26c13c085988cf").unwrap());
		assert!(key.is_some());
	}

	#[test]
	fn imports_as_scrypt_keys() {
		use keys::directory::{KeyDirectory, KeyFileKdf};
		let temp = ::devtools::RandomTempPath::create_dir();
		{
			let mut secret_store = SecretStore::new_in(temp.as_path());
			import_geth_keys(&mut secret_store, Path::new(test_path())).unwrap();
		}

		let key_directory = KeyDirectory::new(&temp.as_path());
		let key_file = key_directory.get(&H128::from_str("62a0ad73556d496a8e1c0783d30d3ace").unwrap()).unwrap();

		match key_file.crypto.kdf {
			KeyFileKdf::Scrypt(scrypt_params) => {
				assert_eq!(262144, scrypt_params.n);
				assert_eq!(8, scrypt_params.r);
				assert_eq!(1, scrypt_params.p);
			},
			_ => { panic!("expected kdf params of crypto to be of scrypt type" ); }
		}
	}

	#[test]
	#[cfg(feature="heavy-tests")]
	fn can_decrypt_with_imported() {
		use keys::store::EncryptedHashMap;

		let temp = ::devtools::RandomTempPath::create_dir();
		let mut secret_store = SecretStore::new_in(temp.as_path());
		import_geth_keys(&mut secret_store, Path::new(test_path())).unwrap();

		let val = secret_store.get::<Bytes>(&H128::from_str("62a0ad73556d496a8e1c0783d30d3ace").unwrap(), "123");
		assert!(val.is_ok());
		assert_eq!(32, val.unwrap().len());
	}
}
