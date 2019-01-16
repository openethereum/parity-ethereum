// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

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

use std::sync::{Arc, atomic, atomic::AtomicBool, Weak};
use std::{fmt, time::Duration};
use std::thread;

use ethereum_types::U256;
use ethkey::{Address, Signature};
use parking_lot::Mutex;

const HID_GLOBAL_USAGE_PAGE: u16 = 0xFF00;
const HID_USB_DEVICE_CLASS: u8 = 0;
const MAX_POLLING_DURATION: Duration = Duration::from_millis(500);
const USB_EVENT_POLLING_INTERVAL: Duration = Duration::from_millis(500);

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
#[derive(Debug, Copy, Clone, PartialEq)]
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
		let ledger = ledger::Manager::new(hidapi.clone());
		let trezor = trezor::Manager::new(hidapi.clone());
		let usb_context = Arc::new(libusb::Context::new()?);

		let l = ledger.clone();
		let t = trezor.clone();
		let exit = exiting.clone();

		// Subscribe to all vendor IDs (VIDs) and product IDs (PIDs)
		// This means that the `HardwareWalletManager` is responsible to validate the detected device
		usb_context.register_callback(
			None, None, Some(HID_USB_DEVICE_CLASS),
			Box::new(EventHandler::new(
				Arc::downgrade(&ledger),
				Arc::downgrade(&trezor)
			))
		)?;

		// Hardware event subscriber thread
		thread::Builder::new()
			.name("hw_wallet_manager".to_string())
			.spawn(move || {
				if let Err(e) = l.update_devices(DeviceDirection::Arrived) {
					debug!(target: "hw", "Ledger couldn't connect at startup, error: {}", e);
				}
				if let Err(e) = t.update_devices(DeviceDirection::Arrived) {
					debug!(target: "hw", "Trezor couldn't connect at startup, error: {}", e);
				}

				while !exit.load(atomic::Ordering::Acquire) {
					if let Err(e) = usb_context.handle_events(Some(USB_EVENT_POLLING_INTERVAL)) {
						debug!(target: "hw", "HardwareWalletManager event handler error: {}", e);
					}
				}
			})
			.ok();

		Ok(Self {
			exiting,
			trezor,
			ledger,
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
		// Indicate to the USB Hotplug handler that it
		// shall terminate but don't wait for it to terminate.
		// If it doesn't terminate for some reason USB Hotplug events will be handled
		// even if the HardwareWalletManger has been dropped
		self.exiting.store(true, atomic::Ordering::Release);
	}
}

/// Hardware wallet event handler
///
/// Note, that this runs to completion and race-conditions can't occur but it can
/// stop other events for being processed with an infinite loop or similar
struct EventHandler {
	ledger: Weak<ledger::Manager>,
	trezor: Weak<trezor::Manager>,
}

impl EventHandler {
	/// Trezor event handler constructor
	pub fn new(ledger: Weak<ledger::Manager>, trezor: Weak<trezor::Manager>) -> Self {
		Self { ledger, trezor }
	}

	fn extract_device_info(device: &libusb::Device) -> Result<(u16, u16), Error> {
		let desc = device.device_descriptor()?;
		Ok((desc.vendor_id(), desc.product_id()))
	}
}

impl libusb::Hotplug for EventHandler {
	fn device_arrived(&mut self, device: libusb::Device) {
		// Upgrade reference to an Arc
		if let (Some(ledger), Some(trezor)) = (self.ledger.upgrade(), self.trezor.upgrade()) {
			// Version ID and Product ID are available
			if let Ok((vid, pid)) = Self::extract_device_info(&device) {
				if trezor::is_valid_trezor(vid, pid) {
					if !trezor::try_connect_polling(&trezor, &MAX_POLLING_DURATION, DeviceDirection::Arrived) {
						trace!(target: "hw", "Trezor device was detected but connection failed");
					}
				} else if ledger::is_valid_ledger(vid, pid) {
					if !ledger::try_connect_polling(&ledger, &MAX_POLLING_DURATION, DeviceDirection::Arrived) {
						trace!(target: "hw", "Ledger device was detected but connection failed");
					}
				}
			}
		}
	}

	fn device_left(&mut self, device: libusb::Device) {
		// Upgrade reference to an Arc
		if let (Some(ledger), Some(trezor)) = (self.ledger.upgrade(), self.trezor.upgrade()) {
			// Version ID and Product ID are available
			if let Ok((vid, pid)) = Self::extract_device_info(&device) {
				if trezor::is_valid_trezor(vid, pid) {
					if !trezor::try_connect_polling(&trezor, &MAX_POLLING_DURATION, DeviceDirection::Left) {
						trace!(target: "hw", "Trezor device was detected but disconnection failed");
					}
				} else if ledger::is_valid_ledger(vid, pid) {
					if !ledger::try_connect_polling(&ledger, &MAX_POLLING_DURATION, DeviceDirection::Left) {
						trace!(target: "hw", "Ledger device was detected but disconnection failed");
					}
				}
			}
		}
	}
}

/// Helper to determine if a device is a valid HID
pub fn is_valid_hid_device(usage_page: u16, interface_number: i32) -> bool {
	usage_page == HID_GLOBAL_USAGE_PAGE || interface_number == HID_USB_DEVICE_CLASS as i32
}
