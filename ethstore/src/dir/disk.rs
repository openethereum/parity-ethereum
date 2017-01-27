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

use std::{fs, io};
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use time;
use {json, SafeAccount, Error};
use json::Uuid;
use super::KeyDirectory;

const IGNORED_FILES: &'static [&'static str] = &[
	"thumbs.db",
	"address_book.json",
	"dapps_policy.json",
	"dapps_accounts.json",
	"dapps_history.json",
];

#[cfg(not(windows))]
fn restrict_permissions_to_owner(file_path: &Path) -> Result<(), i32>  {
	use std::ffi;
	use libc;

	let cstr = ffi::CString::new(&*file_path.to_string_lossy())
		.map_err(|_| -1)?;
	match unsafe { libc::chmod(cstr.as_ptr(), libc::S_IWUSR | libc::S_IRUSR) } {
		0 => Ok(()),
		x => Err(x),
	}
}

#[cfg(windows)]
fn restrict_permissions_to_owner(_file_path: &Path) -> Result<(), i32> {
	Ok(())
}

pub struct DiskDirectory {
	path: PathBuf,
}

impl DiskDirectory {
	pub fn create<P>(path: P) -> Result<Self, Error> where P: AsRef<Path> {
		fs::create_dir_all(&path)?;
		Ok(Self::at(path))
	}

	pub fn at<P>(path: P) -> Self where P: AsRef<Path> {
		DiskDirectory {
			path: path.as_ref().to_path_buf(),
		}
	}

	/// all accounts found in keys directory
	fn files(&self) -> Result<HashMap<PathBuf, SafeAccount>, Error> {
		// it's not done using one iterator cause
		// there is an issue with rustc and it takes tooo much time to compile
		let paths = fs::read_dir(&self.path)?
			.flat_map(Result::ok)
			.filter(|entry| {
				let metadata = entry.metadata().ok();
				let file_name = entry.file_name();
				let name = file_name.to_string_lossy();
				// filter directories
				metadata.map_or(false, |m| !m.is_dir()) &&
				// hidden files
				!name.starts_with(".") &&
				// other ignored files
				!IGNORED_FILES.contains(&&*name)
			})
			.map(|entry| entry.path())
			.collect::<Vec<PathBuf>>();

		Ok(paths
			.iter()
			.map(|p| (
				fs::File::open(p)
					.map_err(Error::from)
					.and_then(|r| json::KeyFile::load(r).map_err(|e| Error::Custom(format!("{:?}", e)))),
				p
			))
			.filter_map(|(file, path)| match file {
				Ok(file) => Some((path.clone(), SafeAccount::from_file(
					file, Some(path.file_name().and_then(|n| n.to_str()).expect("Keys have valid UTF8 names only.").to_owned())
				))),
				Err(err) => {
					warn!("Invalid key file: {:?} ({})", path, err);
					None
				},
			})
			.collect()
		)
	}
}

impl KeyDirectory for DiskDirectory {
	fn load(&self) -> Result<Vec<SafeAccount>, Error> {
		let accounts = self.files()?
			.into_iter()
			.map(|(_, account)| account)
			.collect();
		Ok(accounts)
	}

	fn update(&self, account: SafeAccount) -> Result<SafeAccount, Error> {
		// Disk store handles updates correctly iff filename is the same
		self.insert(account)
	}

	fn insert(&self, account: SafeAccount) -> Result<SafeAccount, Error> {
		// transform account into key file
		let keyfile: json::KeyFile = account.clone().into();

		// build file path
		let filename = account.filename.as_ref().cloned().unwrap_or_else(|| {
			let timestamp = time::strftime("%Y-%m-%dT%H-%M-%S", &time::now_utc()).expect("Time-format string is valid.");
			format!("UTC--{}Z--{}", timestamp, Uuid::from(account.id))
		});

		// update account filename
		let mut account = account;
		account.filename = Some(filename.clone());

		{
			// Path to keyfile
			let mut keyfile_path = self.path.clone();
			keyfile_path.push(filename.as_str());

			// save the file
			let mut file = fs::File::create(&keyfile_path)?;
			keyfile.write(&mut file).map_err(|e| Error::Custom(format!("{:?}", e)))?;

			if let Err(_) = restrict_permissions_to_owner(keyfile_path.as_path()) {
				fs::remove_file(keyfile_path).expect("Expected to remove recently created file");
				return Err(Error::Io(io::Error::last_os_error()));
			}
		}

		Ok(account)
	}

	fn remove(&self, account: &SafeAccount) -> Result<(), Error> {
		// enumerate all entries in keystore
		// and find entry with given address
		let to_remove = self.files()?
			.into_iter()
			.find(|&(_, ref acc)| acc == account);

		// remove it
		match to_remove {
			None => Err(Error::InvalidAccount),
			Some((path, _)) => fs::remove_file(path).map_err(From::from)
		}
	}

	fn path(&self) -> Option<&PathBuf> { Some(&self.path) }
}


#[cfg(test)]
mod test {
	use std::{env, fs};
	use super::DiskDirectory;
	use dir::KeyDirectory;
	use account::SafeAccount;
	use ethkey::{Random, Generator};

	#[test]
	fn should_create_new_account() {
		// given
		let mut dir = env::temp_dir();
		dir.push("ethstore_should_create_new_account");
		let keypair = Random.generate().unwrap();
		let password = "hello world";
		let directory = DiskDirectory::create(dir.clone()).unwrap();

		// when
		let account = SafeAccount::create(&keypair, [0u8; 16], password, 1024, "Test".to_owned(), "{}".to_owned());
		let res = directory.insert(account);


		// then
		assert!(res.is_ok(), "Should save account succesfuly.");
		assert!(res.unwrap().filename.is_some(), "Filename has been assigned.");

		// cleanup
		let _ = fs::remove_dir_all(dir);
	}
}
