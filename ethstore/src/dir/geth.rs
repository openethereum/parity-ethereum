use std::env;
use std::path::PathBuf;
use ethkey::Address;
use {SafeAccount, Error};
use super::{KeyDirectory, DiskDirectory, DirectoryType};

#[cfg(target_os = "macos")]
fn geth_dir_path() -> PathBuf {
	let mut home = env::home_dir().expect("Failed to get home dir");
	home.push("Library");
	home.push("Ethereum");
	home
}

#[cfg(windows)]
/// Default path for ethereum installation on Windows
pub fn geth_dir_path() -> PathBuf {
	let mut home = env::home_dir().expect("Failed to get home dir");
	home.push("AppData");
	home.push("Roaming");
	home.push("Ethereum");
	home
}

#[cfg(not(any(target_os = "macos", windows)))]
/// Default path for ethereum installation on posix system which is not Mac OS
pub fn geth_dir_path() -> PathBuf {
	let mut home = env::home_dir().expect("Failed to get home dir");
	home.push(".ethereum");
	home
}

fn geth_keystore(t: DirectoryType) -> PathBuf {
	let mut dir = geth_dir_path();
	match t {
		DirectoryType::Testnet => {
			dir.push("testnet");
			dir.push("keystore");
		},
		DirectoryType::Main => {
			dir.push("keystore");
		}
	}
	dir
}

pub struct GethDirectory {
	dir: DiskDirectory,
}

impl GethDirectory {
	pub fn create(t: DirectoryType) -> Result<Self, Error> {
		let result = GethDirectory {
			dir: try!(DiskDirectory::create(geth_keystore(t))),
		};

		Ok(result)
	}

	pub fn open(t: DirectoryType) -> Self {
		GethDirectory {
			dir: DiskDirectory::at(geth_keystore(t)),
		}
	}
}

impl KeyDirectory for GethDirectory {
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
