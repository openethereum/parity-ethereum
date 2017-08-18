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

extern crate parking_lot;
extern crate hidapi;
extern crate libusb;
extern crate ethkey;
extern crate ethcore_bigint as bigint;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[cfg(test)] extern crate rustc_hex;
#[cfg_attr(test, macro_use)] extern crate ethcore_util as util;

mod ledger;
mod keepkey;

use std::fmt;
use std::thread;
use std::sync::atomic;
use std::sync::{Arc, Weak};
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use parking_lot::Mutex;
use ethkey::{Address, Signature};

pub use ledger::KeyPath;

/// Hardware waller error.
#[derive(Debug)]
pub enum Error {
	/// Ledger device error.
	LedgerDevice(ledger::Error),
	/// Keekey device error.
	KeepkeyDevice(keepkey::Error),
	/// USB error.
	Usb(libusb::Error),
	/// Hardware wallet not found for specified key.
	KeyNotFound,
}

/// Hardware waller information.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo<'a> {
	pub nonce: &'a[u8],
	pub gas_limit: &'a[u8],
	pub gas_price: &'a[u8],
	pub value: &'a[u8],
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::KeyNotFound => write!(f, "Key not found for given address."),
			Error::LedgerDevice(ref e) => write!(f, "{}", e),
			Error::KeepkeyDevice(ref e) => write!(f, "{}", e),
			Error::Usb(ref e) => write!(f, "{}", e),
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

impl From<keepkey::Error> for Error {
	fn from(err: keepkey::Error) -> Error {
		Error::KeepkeyDevice(err)
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
	keepkey: Arc<Mutex<keepkey::Manager>>,
	ledger: Arc<Mutex<ledger::Manager>>,
}

struct EventHandler {
	keepkey: Weak<Mutex<keepkey::Manager>>,
	ledger: Weak<Mutex<ledger::Manager>>,
}

impl libusb::Hotplug for EventHandler {
	fn device_arrived(&mut self, _device: libusb::Device) {
		debug!("USB Device arrived");
		let l = self.ledger.upgrade();
		let k = self.keepkey.upgrade();

		if l.is_some() && k.is_some() {
			let l = l.unwrap();
			let k = k.unwrap();
			for _ in 0..10 {
				// The device might not be visible right away. Try a few times.
				if l.lock().update_devices().unwrap_or_else(|e| {
					debug!("Error enumerating Ledger devices: {}", e);
					0
				}) + k.lock().update_devices().unwrap_or_else(|e| {
					println!("Error enumerating Keepkey devices: {:?}", e);
					0
				}) > 0 {
					break;
				}
				thread::sleep(Duration::from_millis(200));
			}
		}
	}

	fn device_left(&mut self, _device: libusb::Device) {
		debug!("USB Device lost");
		if let Some(l) = self.ledger.upgrade() {
			if let Err(e) = l.lock().update_devices() {
				debug!("Error enumerating Ledger devices: {}", e);
			}
		}
		if let Some(k) = self.keepkey.upgrade() {
			println!("DEVICE_LEFT");
			if let Err(e) = k.lock().update_devices() {
				println!("Error enumerating Keepkey devices: {}", e);
			}
		}
	}
}

impl HardwareWalletManager {
	pub fn new() -> Result<HardwareWalletManager, Error> {
		let usb_context = Arc::new(libusb::Context::new()?);
		let hidapi = Arc::new(Mutex::new(hidapi::HidApi::new().unwrap()));
		let keepkey = Arc::new(Mutex::new(keepkey::Manager::new(hidapi.clone())));
		let ledger = Arc::new(Mutex::new(ledger::Manager::new(hidapi.clone())));
		usb_context.register_callback(None, None, None, Box::new(EventHandler { keepkey: Arc::downgrade(&keepkey), ledger: Arc::downgrade(&ledger) }))?;
		let exiting = Arc::new(AtomicBool::new(false));
		let thread_exiting = exiting.clone();
		let k = keepkey.clone();
		let l = ledger.clone();
		let thread = thread::Builder::new().name("hw_wallet".to_string()).spawn(move || {
			if let Err(e) = l.lock().update_devices() {
				println!("Error updating ledger devices: {}", e);
			}
			if let Err(e) = k.lock().update_devices() {
				println!("Error updating keepkey devices: {}", e);
			}
			loop {
				usb_context.handle_events(Some(Duration::from_millis(500))).unwrap_or_else(|e| println!("Error processing USB events: {}", e));
				if thread_exiting.load(atomic::Ordering::Acquire) {
					break;
				}
			}
		}).ok();
		Ok(HardwareWalletManager {
			update_thread: thread,
			exiting: exiting,
			keepkey: keepkey,
			ledger: ledger,
		})
	}

	/// Select key derivation path for a chain.
	pub fn set_key_path(&self, key_path: KeyPath) {
		self.ledger.lock().set_key_path(key_path);
	}

	/// List connected wallets. This only returns wallets that are ready to be used.
	pub fn list_wallets(&self) -> Vec<WalletInfo> {
		self.ledger.lock().list_devices()
	}

	/// Get connected wallet info.
	pub fn wallet_info(&self, address: &Address) -> Option<WalletInfo> {
		let l = self.ledger.lock().device_info(address);
		if l.is_some() {
			return l;
		}
		self.keepkey.lock().path_from_address(address).map(|_| WalletInfo {
			name: "".to_string(),
			manufacturer: "keepkey".to_string(),
			serial: "".to_string(),
			address: address.to_owned(),
		})
	}

	/// Sign transaction data with wallet managing `address`.
	pub fn sign_transaction(&self, address: &Address, t: TransactionInfo, data: &[u8]) -> Result<Signature, Error> {
		if self.wallet_info(address).is_some() {
			return Ok(self.ledger.lock().sign_transaction(address, data)?);
		}
		Ok(self.keepkey.lock().sign_transaction(address, t)?)
	}

	/// Keepkey message.
	pub fn keepkey_message(&self, message_type: String, path: Option<String>, message: Option<String>) -> Result<String, Error> {
		Ok(self.keepkey.lock().message(message_type, path, message)?)
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
