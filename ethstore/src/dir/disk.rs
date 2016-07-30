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

use std::{fs, io};
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use time;
use ethkey::Address;
use {json, SafeAccount, Error};
use super::KeyDirectory;

#[cfg(not(windows))]
fn restrict_permissions_to_owner(file_path: &Path) -> Result<(), i32>  {
	use std::ffi;
	use libc;
	let cstr = ffi::CString::new(file_path.to_str().unwrap()).unwrap();
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
		try!(fs::create_dir_all(&path));
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
		let paths = try!(fs::read_dir(&self.path))
			.flat_map(Result::ok)
			.filter(|entry| {
				let metadata = entry.metadata();
				metadata.is_ok() && !metadata.unwrap().is_dir()
			})
			.map(|entry| entry.path())
			.collect::<Vec<PathBuf>>();

		let files: Result<Vec<_>, _> = paths.iter()
			.map(fs::File::open)
			.collect();

		let files = try!(files);

		files.into_iter()
			.map(json::KeyFile::load)
			.zip(paths.into_iter())
			.map(|(file, path)| match file {
				Ok(file) => Ok((path, file.into())),
				Err(err) => Err(Error::InvalidKeyFile(format!("{:?}: {}", path, err))),
			})
			.collect()
	}
}

impl KeyDirectory for DiskDirectory {
	fn load(&self) -> Result<Vec<SafeAccount>, Error> {
		let accounts = try!(self.files())
			.into_iter()
			.map(|(_, account)| account)
			.collect();
		Ok(accounts)
	}

	fn insert(&self, account: SafeAccount) -> Result<SafeAccount, Error> {
		// transform account into key file
		let keyfile: json::KeyFile = account.clone().into();

		// build file path
		let mut account = account;
		account.path = account.path.or_else(|| {
			let mut keyfile_path = self.path.clone();
			let timestamp = time::strftime("%Y-%m-%d_%H:%M:%S_%Z", &time::now()).unwrap_or("???".to_owned());
			keyfile_path.push(format!("{}-{}.json", keyfile.id, timestamp));
			Some(keyfile_path)
		});

		{
			// save the file
			let path = account.path.as_ref().expect("build-file-path ensures is not None; qed");
			let mut file = try!(fs::File::create(path));
			try!(keyfile.write(&mut file).map_err(|e| Error::Custom(format!("{:?}", e))));

			if let Err(_) = restrict_permissions_to_owner(path) {
				fs::remove_file(path).expect("Expected to remove recently created file");
				return Err(Error::Io(io::Error::last_os_error()));
			}
		}

		Ok(account)
	}

	fn remove(&self, address: &Address) -> Result<(), Error> {
		// enumerate all entries in keystore
		// and find entry with given address
		let to_remove = try!(self.files())
			.into_iter()
			.find(|&(_, ref account)| &account.address == address);

		// remove it
		match to_remove {
			None => Err(Error::InvalidAccount),
			Some((path, _)) => fs::remove_file(path).map_err(From::from)
		}
	}
}
