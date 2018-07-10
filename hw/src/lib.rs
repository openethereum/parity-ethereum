// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Hardware wallet management.

#![warn(missing_docs)]
#![warn(warnings)]

extern crate ethereum_types;
extern crate ethkey;
extern crate hidapi;
extern crate libusb;
extern crate parking_lot;
extern crate protobuf;
extern crate semver;
extern crate trezor_sys;

#[macro_use] extern crate log;
#[cfg(test)] extern crate rustc_hex;

mod ledger;
mod trezor;

use std::sync::{Arc, atomic, atomic::AtomicBool};
use std::{fmt, time::Duration};

use ethereum_types::U256;
use ethkey::{Address, Signature};
use parking_lot::Mutex;

const USB_DEVICE_CLASS_DEVICE: u8 = 0;
const POLLING_DURATION: Duration = Duration::from_millis(500);

/// `HardwareWallet` device
#[derive(Debug)]
pub struct Device {
	path: String,
	info: WalletInfo,
}

/// `Wallet` trait
pub trait Wallet<'a> {
	/// Error
	type Error;
	/// Transaction data format
	type Transaction;

	/// Sign transaction data with wallet managing `address`.
	fn sign_transaction(&self, address: &Address, transaction: Self::Transaction) -> Result<Signature, Self::Error>;
	
	/// Set key derivation path for a chain.
	fn set_key_path(&self, key_path: KeyPath);

	/// Re-populate device list
	/// Note, this assumes all devices are iterated over and updated
	fn update_devices(&self, device_direction: DeviceDirection) -> Result<usize, Self::Error>;

	/// Read device info
	fn read_device(&self, usb: &hidapi::HidApi, dev_info: &hidapi::HidDeviceInfo) -> Result<Device, Self::Error>;

	/// List connected and acknowledged wallets
	fn list_devices(&self) -> Vec<WalletInfo>;

	/// List locked wallets
	/// This may be moved if it is the wrong assumption, for example this is not supported by Ledger
	/// Then this method return a empty vector
	fn list_locked_devices(&self) -> Vec<String>;

	/// Get wallet info.
	fn get_wallet(&self, address: &Address) -> Option<WalletInfo>;

	/// Generate ethereum address for a Wallet
	fn get_address(&self, device: &hidapi::HidDevice) -> Result<Option<Address>, Self::Error>;

	/// Open a device using `device path`
	/// Note, f - is a closure that borrows HidResult<HidDevice>
	/// HidDevice is in turn a type alias for a `c_void function pointer`
	/// For further information see:
	///		* <https://github.com/paritytech/hidapi-rs>
	///		* <https://github.com/rust-lang/libc>
	fn open_path<R, F>(&self, f: F) -> Result<R, Self::Error>
		where F: Fn() -> Result<R, &'static str>;
}

/// Hardware wallet error.
#[derive(Debug)]
pub enum Error {
	/// Ledger device error.
	LedgerDevice(ledger::Error),
	/// Trezor device error
	TrezorDevice(trezor::Error),
	/// USB error.
	Usb(libusb::Error),
	/// HID error
	Hid(String),
	/// Hardware wallet not found for specified key.
	KeyNotFound,
}

/// This is the transaction info we need to supply to Trezor message. It's more
/// or less a duplicate of `ethcore::transaction::Transaction`, but we can't
/// import ethcore here as that would be a circular dependency.
pub struct TransactionInfo {
	/// Nonce
	pub nonce: U256,
	/// Gas price
	pub gas_price: U256,
	/// Gas limit
	pub gas_limit: U256,
	/// Receiver
	pub to: Option<Address>,
	/// Value
	pub value: U256,
	/// Data
	pub data: Vec<u8>,
	/// Chain ID
	pub chain_id: Option<u64>,
}

/// Hardware wallet information.
#[derive(Debug, Clone)]
pub struct WalletInfo {
	/// Wallet device name.
	pub name: String,
	/// Wallet device manufacturer.
	pub manufacturer: String,
	/// Wallet device serial number.
	pub serial: String,
	/// Ethereum address.
	pub address: Address,
}

/// Key derivation paths used on hardware wallets.
#[derive(Debug, Clone, Copy)]
pub enum KeyPath {
	/// Ethereum.
	Ethereum,
	/// Ethereum classic.
	EthereumClassic,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::KeyNotFound => write!(f, "Key not found for given address."),
			Error::LedgerDevice(ref e) => write!(f, "{}", e),
			Error::TrezorDevice(ref e) => write!(f, "{}", e),
			Error::Usb(ref e) => write!(f, "{}", e),
			Error::Hid(ref e) => write!(f, "{}", e),
		}
	}
}

impl From<ledger::Error> for Error {
	fn from(err: ledger::Error) -> Self {
		match err {
			ledger::Error::KeyNotFound => Error::KeyNotFound,
			_ => Error::LedgerDevice(err),
		}
	}
}

impl From<trezor::Error> for Error {
	fn from(err: trezor::Error) -> Self {
		match err {
			trezor::Error::KeyNotFound => Error::KeyNotFound,
			_ => Error::TrezorDevice(err),
		}
	}
}

impl From<libusb::Error> for Error {
	fn from(err: libusb::Error) -> Self {
		Error::Usb(err)
	}
}

/// Specifies the direction of the `HardwareWallet` i.e, whether it arrived or left
#[derive(Debug, Copy, Clone)]
pub enum DeviceDirection {
	/// Device arrived
	Arrived,
	/// Device left
	Left,
}

impl fmt::Display for DeviceDirection {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			DeviceDirection::Arrived => write!(f, "arrived"),
			DeviceDirection::Left => write!(f, "left"),
		}
	}
}

/// Hardware wallet management interface.
pub struct HardwareWalletManager {
	exiting: Arc<AtomicBool>,
	ledger: Arc<ledger::Manager>,
	trezor: Arc<trezor::Manager>,
}

impl HardwareWalletManager {
	/// Hardware wallet constructor
	pub fn new() -> Result<Self, Error> {
		let exiting = Arc::new(AtomicBool::new(false));
		let hidapi = Arc::new(Mutex::new(hidapi::HidApi::new().map_err(|e| Error::Hid(e.to_string().clone()))?));
		let ledger = ledger::Manager::new(hidapi.clone(), exiting.clone())?;
		let trezor = trezor::Manager::new(hidapi.clone(), exiting.clone())?;

		Ok(Self {
			exiting,
			ledger,
			trezor,
		})
	}

	/// Select key derivation path for a chain.
	/// Currently, only one hard-coded keypath is supported
	/// It is managed by `ethcore/account_provider`
	pub fn set_key_path(&self, key_path: KeyPath) {
		self.ledger.set_key_path(key_path);
		self.trezor.set_key_path(key_path);
	}

	/// List connected wallets. This only returns wallets that are ready to be used.
	pub fn list_wallets(&self) -> Vec<WalletInfo> {
		let mut wallets = Vec::new();
		wallets.extend(self.ledger.list_devices());
		wallets.extend(self.trezor.list_devices());
		wallets
	}

	/// Return a list of paths to locked hardware wallets
	/// This is only applicable to Trezor because Ledger only appears as
	/// a device when it is unlocked
	pub fn list_locked_wallets(&self) -> Result<Vec<String>, Error> {
		Ok(self.trezor.list_locked_devices())
	}

	/// Get connected wallet info.
	pub fn wallet_info(&self, address: &Address) -> Option<WalletInfo> {
		if let Some(info) = self.ledger.get_wallet(address) {
			Some(info)
		} else {
			self.trezor.get_wallet(address)
		}
	}

	/// Sign a message with the wallet (only supported by Ledger)
	pub fn sign_message(&self, address: &Address, msg: &[u8]) -> Result<Signature, Error> {
		if self.ledger.get_wallet(address).is_some() {
			Ok(self.ledger.sign_message(address, msg)?)
		} else if self.trezor.get_wallet(address).is_some() {
			Err(Error::TrezorDevice(trezor::Error::NoSigningMessage))
		} else {
			Err(Error::KeyNotFound)
		}
	}

	/// Sign transaction data with wallet managing `address`.
	pub fn sign_transaction(&self, address: &Address, t_info: &TransactionInfo, encoded_transaction: &[u8]) -> Result<Signature, Error> {
		if self.ledger.get_wallet(address).is_some() {
			Ok(self.ledger.sign_transaction(address, encoded_transaction)?)
		} else if self.trezor.get_wallet(address).is_some() {
			Ok(self.trezor.sign_transaction(address, t_info)?)
		} else {
			Err(Error::KeyNotFound)
		}
	}

	/// Send a pin to a device at a certain path to unlock it
	/// This is only applicable to Trezor because Ledger only appears as
	/// a device when it is unlocked
	pub fn pin_matrix_ack(&self, path: &str, pin: &str) -> Result<bool, Error> {
		self.trezor.pin_matrix_ack(path, pin).map_err(Error::TrezorDevice)
	}
}

impl Drop for HardwareWalletManager {
	fn drop(&mut self) {
		// Indicate to the USB Hotplug handlers that they
		// shall terminate but don't wait for them to terminate.
		// If they don't terminate for some reason USB Hotplug events will be handled
		// even if the HardwareWalletManger has been dropped
		self.exiting.store(true, atomic::Ordering::Release);
	}
}
