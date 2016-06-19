use ethkey::Address;
use {SafeAccount, Error};

mod disk;
mod geth;
mod parity;

pub enum DirectoryType {
	Testnet,
	Main,
}

pub trait KeyDirectory: Send + Sync {
	fn load(&self) -> Result<Vec<SafeAccount>, Error>;
	fn insert(&self, account: SafeAccount) -> Result<(), Error>;
	fn remove(&self, address: &Address) -> Result<(), Error>;
}

pub use self::disk::DiskDirectory;
pub use self::geth::GethDirectory;
pub use self::parity::ParityDirectory;
