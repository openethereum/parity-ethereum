use std::{fs, ffi, io};
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use ethkey::Address;
use {libc, json, SafeAccount, Error};
use super::KeyDirectory;

#[cfg(not(windows))]
fn restrict_permissions_to_owner(file_path: &Path) -> Result<(), i32>  {
	let cstr = ffi::CString::new(file_path.to_str().unwrap()).unwrap();
	match unsafe { libc::chmod(cstr.as_ptr(), libc::S_IWUSR | libc::S_IRUSR) } {
		0 => Ok(()),
		x => Err(x),
	}
}

#[cfg(windows)]
fn restrict_permissions_to_owner(file_path: &Path) -> Result<(), i32> {
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

		let accounts = files.into_iter()
			.map(json::KeyFile::load)
			.zip(paths.into_iter())
			.filter_map(|(file, path)| file.ok().map(|file| (path, SafeAccount::from(file))))
			.collect();

		Ok(accounts)
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

	fn insert(&self, account: SafeAccount) -> Result<(), Error> {
		// transform account into key file
		let keyfile: json::KeyFile = account.into();

		// build file path
		let mut keyfile_path = self.path.clone();
		keyfile_path.push(format!("{}", keyfile.id));

		// save the file
		let mut file = try!(fs::File::create(&keyfile_path));
		try!(keyfile.write(&mut file).map_err(|e| Error::Custom(format!("{:?}", e))));

		if let Err(_) = restrict_permissions_to_owner(&keyfile_path) {
			fs::remove_file(&keyfile_path).expect("Expected to remove recently created file");
			return Err(Error::Io(io::Error::last_os_error()));
		}

		Ok(())
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
