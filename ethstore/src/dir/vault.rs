// Copyright 2015, 2016, 2017 Parity Technologies (UK) Ltd.
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

use std::{fs, io};
use std::path::{PathBuf, Path};
use {json, SafeAccount, Error};
use util::sha3::Hashable;
use super::super::account::Crypto;
use super::{KeyDirectory, VaultKeyDirectory, VaultKey, SetKeyError};
use super::disk::{DiskDirectory, KeyFileManager};

const VAULT_FILE_NAME: &'static str = "vault.json";

/// Vault directory implementation
pub type VaultDiskDirectory = DiskDirectory<VaultKeyFileManager>;

/// Vault key file manager
pub struct VaultKeyFileManager {
	key: VaultKey,
}

impl VaultDiskDirectory {
	/// Create new vault directory with given key
	pub fn create<P>(root: P, name: &str, key: VaultKey) -> Result<Self, Error> where P: AsRef<Path> {
		// check that vault directory does not exists
		let vault_dir_path = make_vault_dir_path(root, name, true)?;
		if vault_dir_path.exists() {
			return Err(Error::CreationFailed);
		}

		// create vault && vault file
		fs::create_dir_all(&vault_dir_path)?;
		if let Err(err) = create_vault_file(&vault_dir_path, &key) {
			let _ = fs::remove_dir_all(&vault_dir_path); // can't do anything with this
			return Err(err);
		}

		Ok(DiskDirectory::new(vault_dir_path, VaultKeyFileManager::new(key)))
	}

	/// Open existing vault directory with given key
	pub fn at<P>(root: P, name: &str, key: VaultKey) -> Result<Self, Error> where P: AsRef<Path> {
		// check that vault directory exists
		let vault_dir_path = make_vault_dir_path(root, name, true)?;
		if !vault_dir_path.is_dir() {
			return Err(Error::CreationFailed);
		}

		// check that passed key matches vault file
		check_vault_file(&vault_dir_path, &key)?;

		Ok(DiskDirectory::new(vault_dir_path, VaultKeyFileManager::new(key)))
	}

	fn create_temp_vault(&self, key: VaultKey) -> Result<VaultDiskDirectory, Error> {
		let original_path = self.path().expect("self is instance of DiskDirectory; DiskDirectory always returns path; qed");
		let mut path: PathBuf = original_path.clone();
		let name = self.name();

		path.push(name); // to jump to the next level

		let mut index = 0;
		loop {
			let name = format!("{}_temp_{}", name, index);
			path.set_file_name(&name);
			if !path.exists() {
				return VaultDiskDirectory::create(original_path, &name, key);
			}

			index += 1;
		}
	}

	fn copy_to_vault(&self, vault: &VaultDiskDirectory, vault_key: &VaultKey) -> Result<(), Error> {
		let password = &self.key_manager().key.password;
		for account in self.load()? {
			let filename = account.filename.clone().expect("self is instance of DiskDirectory; DiskDirectory fills filename in load; qed");
			let new_account = account.change_password(password, &vault_key.password, vault_key.iterations)?;
			vault.insert_with_filename(new_account, filename)?;
		}

		Ok(())
	}

	fn delete(&self) -> Result<(), Error> {
		let path = self.path().expect("self is instance of DiskDirectory; DiskDirectory always returns path; qed");
		fs::remove_dir_all(path).map_err(Into::into)
	}
}

impl VaultKeyDirectory for VaultDiskDirectory {
	fn as_key_directory(&self) -> &KeyDirectory {
		self
	}

	fn name(&self) -> &str {
		self.path()
			.expect("self is instance of DiskDirectory; DiskDirectory always returns path; qed")
			.file_name()
			.expect("last component of path is checked in make_vault_dir_path; it contains no fs-specific characters; file_name only returns None if last component is fs-specific; qed")
			.to_str()
			.expect("last component of path is checked in make_vault_dir_path; it contains only valid unicode characters; to_str fails when file_name is not valid unicode; qed")
	}

	fn set_key(&self, key: VaultKey, new_key: VaultKey) -> Result<(), SetKeyError> {
		if self.key_manager().key != key {
			return Err(SetKeyError::NonFatalOld(Error::InvalidPassword));
		}

		let temp_vault = VaultDiskDirectory::create_temp_vault(self, new_key.clone()).map_err(|err| SetKeyError::NonFatalOld(err))?;
		let mut source_path = temp_vault.path().expect("temp_vault is instance of DiskDirectory; DiskDirectory always returns path; qed").clone();
		let mut target_path = self.path().expect("self is instance of DiskDirectory; DiskDirectory always returns path; qed").clone();
		// jump to next fs level
		source_path.push("next");
		target_path.push("next");

		let temp_accounts = self.copy_to_vault(&temp_vault, &new_key)
			.and_then(|_| temp_vault.load())
			.map_err(|err| {
				// ignore error, as we already processing error
				let _ = temp_vault.delete();
				SetKeyError::NonFatalOld(err)
			})?;

		// we can't just delete temp vault until all files moved, because
		// original vault content has already been partially replaced
		// => when error or crash happens here, we can't do anything
		for temp_account in temp_accounts {
			let filename = temp_account.filename.expect("self is instance of DiskDirectory; DiskDirectory fills filename in load; qed");
			source_path.set_file_name(&filename);
			target_path.set_file_name(&filename);
			fs::rename(&source_path, &target_path).map_err(|err| SetKeyError::Fatal(err.into()))?;
		}
		source_path.set_file_name(VAULT_FILE_NAME);
		target_path.set_file_name(VAULT_FILE_NAME);
		fs::rename(source_path, target_path).map_err(|err| SetKeyError::Fatal(err.into()))?;

		temp_vault.delete().map_err(|err| SetKeyError::NonFatalNew(err))
	}
}

impl VaultKeyFileManager {
	pub fn new(key: VaultKey) -> Self {
		VaultKeyFileManager {
			key: key,
		}
	}
}

impl KeyFileManager for VaultKeyFileManager {
	fn read<T>(&self, filename: Option<String>, reader: T) -> Result<SafeAccount, Error> where T: io::Read {
		let vault_file = json::VaultKeyFile::load(reader).map_err(|e| Error::Custom(format!("{:?}", e)))?;
		let safe_account = SafeAccount::from_vault_file(&self.key.password, vault_file, filename.clone())?;
		if !safe_account.check_password(&self.key.password) {
			warn!("Invalid vault key file: {:?}", filename);
			return Err(Error::InvalidPassword);
		}

		Ok(safe_account)
	}

	fn write<T>(&self, account: SafeAccount, writer: &mut T) -> Result<(), Error> where T: io::Write {
		// all accounts share the same password
		if !account.check_password(&self.key.password) {
			return Err(Error::InvalidPassword);
		}

		let vault_file: json::VaultKeyFile = account.into_vault_file(self.key.iterations, &self.key.password)?;
		vault_file.write(writer).map_err(|e| Error::Custom(format!("{:?}", e)))
	}
}

/// Makes path to vault directory, checking that vault name is appropriate
fn make_vault_dir_path<P>(root: P, name: &str, check_name: bool) -> Result<PathBuf, Error> where P: AsRef<Path> {
	// check vault name
	if check_name && !check_vault_name(name) {
		return Err(Error::InvalidVaultName);
	}

	let mut vault_dir_path: PathBuf = root.as_ref().into();
	vault_dir_path.push(name);
	Ok(vault_dir_path)
}

/// Every vault must have unique name => we rely on filesystem to check this
/// => vault name must not contain any fs-special characters to avoid directory traversal
/// => we only allow alphanumeric + separator characters in vault name.
fn check_vault_name(name: &str) -> bool {
	!name.is_empty()
	&& name.chars()
		.all(|c| c.is_alphanumeric()
			|| c.is_whitespace()
			|| c == '-' || c == '_')
}

/// Vault can be empty, but still must be pluggable => we store vault password in separate file
fn create_vault_file<P>(vault_dir_path: P, key: &VaultKey) -> Result<(), Error> where P: AsRef<Path> {
	let password_hash = key.password.sha3();
	let crypto = Crypto::with_plain(&password_hash, &key.password, key.iterations);

	let mut vault_file_path: PathBuf = vault_dir_path.as_ref().into();
	vault_file_path.push(VAULT_FILE_NAME);

	let mut vault_file = fs::File::create(vault_file_path)?;
	let vault_file_contents = json::VaultFile {
		crypto: crypto.into(),
	};
	vault_file_contents.write(&mut vault_file).map_err(|e| Error::Custom(format!("{:?}", e)))?;

	Ok(())
}

/// When vault is opened => we must check that password matches
fn check_vault_file<P>(vault_dir_path: P, key: &VaultKey) -> Result<(), Error> where P: AsRef<Path> {
	let mut vault_file_path: PathBuf = vault_dir_path.as_ref().into();
	vault_file_path.push(VAULT_FILE_NAME);

	let vault_file = fs::File::open(vault_file_path)?;
	let vault_file_contents = json::VaultFile::load(vault_file).map_err(|e| Error::Custom(format!("{:?}", e)))?;
	let vault_file_crypto: Crypto = vault_file_contents.crypto.into();

	let password_bytes = vault_file_crypto.decrypt(&key.password)?;
	let password_hash = key.password.sha3();
	if &*password_hash != password_bytes.as_slice() {
		return Err(Error::InvalidPassword);
	}

	Ok(())
}

#[cfg(test)]
mod test {
	use std::{env, fs};
	use std::io::Write;
	use dir::VaultKey;
	use super::{VAULT_FILE_NAME, check_vault_name, make_vault_dir_path, create_vault_file, check_vault_file, VaultDiskDirectory};

	#[test]
	fn check_vault_name_succeeds() {
		assert!(check_vault_name("vault"));
		assert!(check_vault_name("vault with spaces"));
		assert!(check_vault_name("vault	with	tabs"));
		assert!(check_vault_name("vault_with_underscores"));
		assert!(check_vault_name("vault-with-dashes"));
		assert!(check_vault_name("vault-with-digits-123"));
	}

	#[test]
	fn check_vault_name_fails() {
		assert!(!check_vault_name(""));
		assert!(!check_vault_name("."));
		assert!(!check_vault_name("*"));
		assert!(!check_vault_name("../.bash_history"));
		assert!(!check_vault_name("/etc/passwd"));
		assert!(!check_vault_name("c:\\windows"));
	}

	#[test]
	fn make_vault_dir_path_succeeds() {
		assert_eq!(make_vault_dir_path("/home/user/parity", "vault", true).unwrap().to_str().unwrap(), "/home/user/parity/vault");
		assert_eq!(make_vault_dir_path("/home/user/parity", "*bad-name*", false).unwrap().to_str().unwrap(), "/home/user/parity/*bad-name*");
	}

	#[test]
	fn make_vault_dir_path_fails() {
		assert!(make_vault_dir_path("/home/user/parity", "*bad-name*", true).is_err());
	}

	#[test]
	fn create_vault_file_succeeds() {
		// given
		let key = VaultKey::new("password", 1024);
		let mut dir = env::temp_dir();
		dir.push("create_vault_file_succeeds");
		let mut vault_dir = dir.clone();
		vault_dir.push("vault");
		fs::create_dir_all(&vault_dir).unwrap();

		// when
		let result = create_vault_file(&vault_dir, &key);

		// then
		assert!(result.is_ok());
		let mut vault_file_path = vault_dir.clone();
		vault_file_path.push(VAULT_FILE_NAME);
		assert!(vault_file_path.exists() && vault_file_path.is_file());

		// cleanup
		let _ = fs::remove_dir_all(dir);
	}

	#[test]
	fn check_vault_file_succeeds() {
		// given
		let key = VaultKey::new("password", 1024);
		let vault_file_contents = r#"{"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"758696c8dc6378ab9b25bb42790da2f5"},"ciphertext":"54eb50683717d41caaeb12ea969f2c159daada5907383f26f327606a37dc7168","kdf":"pbkdf2","kdfparams":{"c":1024,"dklen":32,"prf":"hmac-sha256","salt":"3c320fa566a1a7963ac8df68a19548d27c8f40bf92ef87c84594dcd5bbc402b6"},"mac":"9e5c2314c2a0781962db85611417c614bd6756666b6b1e93840f5b6ed895f003"}}"#;
		let mut dir = env::temp_dir();
		dir.push("check_vault_file_succeeds");
		fs::create_dir_all(&dir).unwrap();
		let mut vault_file_path = dir.clone();
		vault_file_path.push(VAULT_FILE_NAME);
		{
			let mut vault_file = fs::File::create(vault_file_path).unwrap();
			vault_file.write_all(vault_file_contents.as_bytes()).unwrap();
		}

		// when
		let result = check_vault_file(&dir, &key);

		// then
		assert!(result.is_ok());

		// cleanup
		let _ = fs::remove_dir_all(dir);
	}

	#[test]
	fn check_vault_file_fails() {
		// given
		let key = VaultKey::new("password1", 1024);
		let mut dir = env::temp_dir();
		dir.push("check_vault_file_fails");
		let mut vault_file_path = dir.clone();
		vault_file_path.push(VAULT_FILE_NAME);
		fs::create_dir_all(&dir).unwrap();

		// when
		let result = check_vault_file(&dir, &key);

		// then
		assert!(result.is_err());

		// and when given
		let vault_file_contents = r#"{"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"0155e3690be19fbfbecabcd440aa284b"},"ciphertext":"4d6938a1f49b7782","kdf":"pbkdf2","kdfparams":{"c":1024,"dklen":32,"prf":"hmac-sha256","salt":"b6a9338a7ccd39288a86dba73bfecd9101b4f3db9c9830e7c76afdbd4f6872e5"},"mac":"16381463ea11c6eb2239a9f339c2e780516d29d234ce30ac5f166f9080b5a262"}}"#;
		{
			let mut vault_file = fs::File::create(vault_file_path).unwrap();
			vault_file.write_all(vault_file_contents.as_bytes()).unwrap();
		}

		// when
		let result = check_vault_file(&dir, &key);

		// then
		assert!(result.is_err());

		// cleanup
		let _ = fs::remove_dir_all(dir);
	}

	#[test]
	fn vault_directory_can_be_created() {
		// given
		let key = VaultKey::new("password", 1024);
		let mut dir = env::temp_dir();
		dir.push("vault_directory_can_be_created");

		// when
		let vault = VaultDiskDirectory::create(&dir, "vault", key.clone());

		// then
		assert!(vault.is_ok());

		// and when
		let vault = VaultDiskDirectory::at(&dir, "vault", key);

		// then
		assert!(vault.is_ok());

		// cleanup
		let _ = fs::remove_dir_all(dir);
	}

	#[test]
	fn vault_directory_cannot_be_created_if_already_exists() {
		// given
		let key = VaultKey::new("password", 1024);
		let mut dir = env::temp_dir();
		dir.push("vault_directory_cannot_be_created_if_already_exists");
		let mut vault_dir = dir.clone();
		vault_dir.push("vault");
		fs::create_dir_all(&vault_dir).unwrap();

		// when
		let vault = VaultDiskDirectory::create(&dir, "vault", key);

		// then
		assert!(vault.is_err());

		// cleanup
		let _ = fs::remove_dir_all(dir);
	}

	#[test]
	fn vault_directory_cannot_be_opened_if_not_exists() {
		// given
		let key = VaultKey::new("password", 1024);
		let mut dir = env::temp_dir();
		dir.push("vault_directory_cannot_be_opened_if_not_exists");

		// when
		let vault = VaultDiskDirectory::at(&dir, "vault", key);

		// then
		assert!(vault.is_err());

		// cleanup
		let _ = fs::remove_dir_all(dir);
	}
}
