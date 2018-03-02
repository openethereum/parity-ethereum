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

//! Hardware wallet management.

#[deny(missing_docs)]
#[deny(warnings)]

extern crate ethereum_types;
extern crate ethkey;
extern crate hidapi;
extern crate libusb;
extern crate parking_lot;
extern crate protobuf;
extern crate trezor_sys;
#[macro_use] extern crate log;
#[cfg(test)] extern crate rustc_hex;

mod ledger;
mod trezor;

use ethkey::{Address, Signature};

use parking_lot::Mutex;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;
use ethereum_types::U256;

#[derive(Debug)]
pub struct Device {
	path: String,
	info: WalletInfo,
}

// The goal with this is to replace the Hardware Wallet Manager completly
// Because it more or less acts as wrapper on top of the different wallets
// Also, because it doesn't care about the event handler threads
// It doesn't make sense to keep it only adds complexity in terms of
// more code
pub trait Foo<'a> {
	/// Error
	type Error;
	/// Transaction format
	type Transaction;

	/// USB Device Class
	const USB_DEVICE_CLASS_DEVICE: u8 = 0;

	/// Sign transaction data with wallet managing `address`.
	fn sign_transaction(&self, address: &Address, transaction: Self::Transaction) -> Result<Signature, Self::Error>;

	/// Set key derivation path for a chain.
	// TODO: add return value
	fn set_key_path(&self, key_path: KeyPath);

	/// Re-populate device list
	/// Note, this assumes all devices are iterated over and updated
	fn update_devices(&self) -> Result<usize, Self::Error>;

	/// Read device info
	fn read_device(&self, usb: &hidapi::HidApi, dev_info: &hidapi::HidDeviceInfo) -> Result<Device, Self::Error>;

	/// List connected and acknowledged wallets
	fn list_devices(&self) -> Vec<WalletInfo>;

	/// List locked wallets
	fn list_locked_devices(&self) -> Vec<String>;

	/// Get wallet info.
	fn get_wallet(&self, address: &Address) -> Option<WalletInfo>;

	/// Generate ethereum address for a Wallet
	fn get_address(&self, device: &hidapi::HidDevice) -> Result<Option<Address>, Self::Error>;

	/// Open a device using path
	/// Note, f - is a closure that borrows HidResult<HidDevice>
	/// HidDevice is in turn as type alias for a `c_void function pointer`
	/// For further information see:
	///		* [hidapi-rs](https://github.com/paritytech/hidapi-rs)
	///		* [libc](https://github.com/rust-lang/libc)
	fn open_path<R, F>(&self, f: F) -> Result<R, Self::Error>
		where F: Fn() -> Result<R, &'static str>;
}

const USB_DEVICE_CLASS_DEVICE: u8 = 0;

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
/// or less a duplicate of ethcore::transaction::Transaction, but we can't
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
	fn from(err: ledger::Error) -> Error {
		match err {
			ledger::Error::KeyNotFound => Error::KeyNotFound,
			_ => Error::LedgerDevice(err),
		}
	}
}

impl From<trezor::Error> for Error {
	fn from(err: trezor::Error) -> Error {
		match err {
			trezor::Error::KeyNotFound => Error::KeyNotFound,
			_ => Error::TrezorDevice(err),
		}
	}
}

impl From<libusb::Error> for Error {
	fn from(err: libusb::Error) -> Error {
		Error::Usb(err)
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
	pub fn new() -> Result<HardwareWalletManager, Error> {
		let usb_context_trezor = Arc::new(libusb::Context::new()?);
		let usb_context_ledger = Arc::new(libusb::Context::new()?);
		let hidapi = Arc::new(Mutex::new(hidapi::HidApi::new().map_err(|e| Error::Hid(e.to_string().clone()))?));
		let ledger = Arc::new(ledger::Manager::new(hidapi.clone()));
		let trezor = Arc::new(trezor::Manager::new(hidapi.clone()));

		// Subscribe to TREZOR V1
		// Note, this support only TREZOR V1 becasue TREZOR V2 has another vendorID for some reason
		// Also, we now only support one product as the second argument specifies
		usb_context_trezor.register_callback(
			Some(trezor::TREZOR_VID), Some(trezor::TREZOR_PIDS[0]), Some(USB_DEVICE_CLASS_DEVICE),
			Box::new(trezor::EventHandler::new(Arc::downgrade(&trezor))))?;

		// Subscribe to all Ledger Devices
		// This means that we need to check that the given productID is supported
		// None => LIBUSB_HOTPLUG_MATCH_ANY, in other words that all are subscribed to
		// More info can be found: http://libusb.sourceforge.net/api-1.0/group__hotplug.html#gae6c5f1add6cc754005549c7259dc35ea
		usb_context_ledger.register_callback(
			Some(ledger::LEDGER_VID), None, Some(USB_DEVICE_CLASS_DEVICE),
			Box::new(ledger::EventHandler::new(Arc::downgrade(&ledger))))?;

		let exiting = Arc::new(AtomicBool::new(false));
		let thread_exiting_ledger = exiting.clone();
		let thread_exiting_trezor = exiting.clone();
		let l = ledger.clone();
		let t = trezor.clone();

		// Ledger event thread
		thread::Builder::new()
			.name("hw_wallet_ledger".to_string())
			.spawn(move || {
				if let Err(e) = l.update_devices() {
					debug!(target: "hw", "Ledger couldn't connect at startup, error: {}", e);
				}
				loop {
					usb_context_ledger.handle_events(Some(Duration::from_millis(500)))
					           .unwrap_or_else(|e| debug!(target: "hw", "Ledger event handler error: {}", e));
					if thread_exiting_ledger.load(atomic::Ordering::Acquire) {
						break;
					}
				}
			})
			.ok();

		// Trezor event thread
		thread::Builder::new()
			.name("hw_wallet_trezor".to_string())
			.spawn(move || {
				if let Err(e) = t.update_devices() {
					debug!(target: "hw", "Trezor couldn't connect at startup, error: {}", e);
				}
				loop {
					usb_context_trezor.handle_events(Some(Duration::from_millis(500)))
					           .unwrap_or_else(|e| debug!(target: "hw", "Trezor event handler error: {}", e));
					if thread_exiting_trezor.load(atomic::Ordering::Acquire) {
						break;
					}
				}
			})
			.ok();

		Ok(HardwareWalletManager {
			exiting: exiting,
			ledger: ledger,
			trezor: trezor,
		})
	}

	/// Select key derivation path for a chain.
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
	pub fn list_locked_wallets(&self) -> Result<Vec<String>, Error> {
		Ok(self.trezor.list_locked_devices())
	}

	/// Get connected wallet info.
	//TODO: modify with trait
	pub fn wallet_info(&self, address: &Address) -> Option<WalletInfo> {
		if let Some(info) = self.ledger.get_wallet(address) {
			Some(info)
		} else {
			self.trezor.get_wallet(address)
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
	/// This is Trezor specific!!!
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
