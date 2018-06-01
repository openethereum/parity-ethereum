//! Dummy module for platforms that does not provide support for hardware wallets (libusb)

use super::{fmt, Address};

pub struct WalletInfo {
	pub address: Address,
	pub name: String,
	pub manufacturer: String,
}

#[derive(Debug)]
/// `ErrorType` for devices with no `hardware wallet`
pub enum HardwareError {
	NoWallet,
}

/// `HardwareWalletManager` for devices with no `hardware wallet`
pub struct HardwareWalletManager;

impl HardwareWalletManager {
	pub fn wallet_info(&self, _: &Address) -> Option<WalletInfo> { 
	None 
	}

	pub fn list_wallets(&self) -> Vec<WalletInfo> {
		Vec::with_capacity(0)
	}

	pub fn list_locked_wallets(&self) -> Result<Vec<String>, HardwareError> {
	Err(HardwareError::NoWallet)
	}

	pub fn pin_matrix_ack(&self, _: &str, _: &str) -> Result<bool, HardwareError> { 
	Err(HardwareError::NoWallet)
	}
}

impl fmt::Display for HardwareError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
		write!(f, "") 
	}
}
