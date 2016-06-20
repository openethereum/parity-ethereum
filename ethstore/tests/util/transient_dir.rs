use std::path::PathBuf;
use std::{env, fs};
use rand::{Rng, OsRng};
use ethstore::dir::{KeyDirectory, DiskDirectory};
use ethstore::ethkey::Address;
use ethstore::{Error, SafeAccount};

pub fn random_dir() -> PathBuf {
	let mut rng = OsRng::new().unwrap();
	let mut dir = env::temp_dir();
	dir.push(format!("{:x}-{:x}", rng.next_u64(), rng.next_u64()));
	dir
}

pub struct TransientDir {
	dir: DiskDirectory,
	path: PathBuf,
}

impl TransientDir {
	pub fn create() -> Result<Self, Error> {
		let path = random_dir();
		let result = TransientDir {
			dir: try!(DiskDirectory::create(&path)),
			path: path,
		};

		Ok(result)
	}

	pub fn open() -> Self {
		let path = random_dir();
		TransientDir {
			dir: DiskDirectory::at(&path),
			path: path,
		}
	}
}

impl Drop for TransientDir {
	fn drop(&mut self) {
		fs::remove_dir_all(&self.path).expect("Expected to remove temp dir");
	}
}

impl KeyDirectory for TransientDir {
	fn load(&self) -> Result<Vec<SafeAccount>, Error> {
		self.dir.load()
	}

	fn insert(&self, account: SafeAccount) -> Result<(), Error> {
		self.dir.insert(account)
	}

	fn remove(&self, address: &Address) -> Result<(), Error> {
		self.dir.remove(address)
	}
}
