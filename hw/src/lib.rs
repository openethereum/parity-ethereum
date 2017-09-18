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

extern crate ethcore_bigint as bigint;
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
use std::sync::{Arc, Weak};
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;
use bigint::prelude::uint::U256;

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
	pub nonce: U256,
	pub gas_price: U256,
	pub gas_limit: U256,
	pub to: Option<Address>,
	pub value: U256,
	pub data: Vec<u8>,
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
	update_thread: Option<thread::JoinHandle<()>>,
	exiting: Arc<AtomicBool>,
	ledger: Arc<ledger::Manager>,
	trezor: Arc<trezor::Manager>,
}

struct EventHandler {
	ledger: Weak<ledger::Manager>,
	trezor: Weak<trezor::Manager>,
}

impl libusb::Hotplug for EventHandler {
	fn device_arrived(&mut self, _device: libusb::Device) {
		debug!("USB Device arrived");
		if let (Some(l), Some(t)) = (self.ledger.upgrade(), self.trezor.upgrade()) {
			for _ in 0..10 {
				let l_devices = l.update_devices().unwrap_or_else(|e| {
					debug!("Error enumerating Ledger devices: {}", e);
					0
				});
				let t_devices = t.update_devices().unwrap_or_else(|e| {
					debug!("Error enumerating Trezor devices: {}", e);
					0
				});
				if l_devices + t_devices > 0 {
					break;
				}
				thread::sleep(Duration::from_millis(200));
			}
		}
	}

	fn device_left(&mut self, _device: libusb::Device) {
		debug!("USB Device lost");
		if let (Some(l), Some(t)) = (self.ledger.upgrade(), self.trezor.upgrade()) {
			l.update_devices().unwrap_or_else(|e| {debug!("Error enumerating Ledger devices: {}", e); 0});
			t.update_devices().unwrap_or_else(|e| {debug!("Error enumerating Trezor devices: {}", e); 0});
		}
	}
}

impl HardwareWalletManager {
	pub fn new() -> Result<HardwareWalletManager, Error> {
		let usb_context = Arc::new(libusb::Context::new()?);
		let hidapi = Arc::new(Mutex::new(hidapi::HidApi::new().map_err(|e| Error::Hid(e.to_string().clone()))?));
		let ledger = Arc::new(ledger::Manager::new(hidapi.clone()));
		let trezor = Arc::new(trezor::Manager::new(hidapi.clone()));
		usb_context.register_callback(
			None, None, None,
			Box::new(EventHandler {
				ledger: Arc::downgrade(&ledger),
				trezor: Arc::downgrade(&trezor),
			}),
		)?;
		let exiting = Arc::new(AtomicBool::new(false));
		let thread_exiting = exiting.clone();
		let l = ledger.clone();
		let t = trezor.clone();
		let thread = thread::Builder::new()
			.name("hw_wallet".to_string())
			.spawn(move || {
				if let Err(e) = l.update_devices() {
					debug!("Error updating ledger devices: {}", e);
				}
				if let Err(e) = t.update_devices() {
					debug!("Error updating trezor devices: {}", e);
				}
				loop {
					usb_context.handle_events(Some(Duration::from_millis(500)))
					           .unwrap_or_else(|e| debug!("Error processing USB events: {}", e));
					if thread_exiting.load(atomic::Ordering::Acquire) {
						break;
					}
				}
			})
			.ok();
		Ok(HardwareWalletManager {
			update_thread: thread,
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
	pub fn wallet_info(&self, address: &Address) -> Option<WalletInfo> {
		if let Some(info) = self.ledger.device_info(address) {
			Some(info)
		} else {
			self.trezor.device_info(address)
		}
	}

	/// Sign transaction data with wallet managing `address`.
	pub fn sign_transaction(&self, address: &Address, t_info: &TransactionInfo, encoded_transaction: &[u8]) -> Result<Signature, Error> {
		if self.ledger.device_info(address).is_some() {
			Ok(self.ledger.sign_transaction(address, encoded_transaction)?)
		} else if self.trezor.device_info(address).is_some() {
			Ok(self.trezor.sign_transaction(address, t_info)?)
		} else {
			Err(Error::KeyNotFound)
		}
	}

	/// Send a pin to a device at a certain path to unlock it
	pub fn pin_matrix_ack(&self, path: &str, pin: &str) -> Result<bool, Error> {
		self.trezor.pin_matrix_ack(path, pin).map_err(Error::TrezorDevice)
	}
}

impl Drop for HardwareWalletManager {
	fn drop(&mut self) {
		self.exiting.store(true, atomic::Ordering::Release);
		if let Some(thread) = self.update_thread.take() {
			thread.thread().unpark();
			thread.join().ok();
		}
	}
}
