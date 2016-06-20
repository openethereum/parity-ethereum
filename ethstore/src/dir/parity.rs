use std::env;
use std::path::PathBuf;
use ethkey::Address;
use {SafeAccount, Error};
use super::{KeyDirectory, DiskDirectory, DirectoryType};

fn parity_dir_path() -> PathBuf {
	let mut home = env::home_dir().expect("Failed to get home dir");
	home.push(".parity");
	home
}

fn parity_keystore(t: DirectoryType) -> PathBuf {
	let mut dir = parity_dir_path();
	match t {
		DirectoryType::Testnet => {
			dir.push("testnet_keys");
		},
		DirectoryType::Main => {
			dir.push("keys");
		}
	}
	dir
}

pub struct ParityDirectory {
	dir: DiskDirectory,
}

impl ParityDirectory {
	pub fn create(t: DirectoryType) -> Result<Self, Error> {
		let result = ParityDirectory {
			dir: try!(DiskDirectory::create(parity_keystore(t))),
		};

		Ok(result)
	}

	pub fn open(t: DirectoryType) -> Self {
		ParityDirectory {
			dir: DiskDirectory::at(parity_keystore(t)),
		}
	}
}

impl KeyDirectory for ParityDirectory {
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
