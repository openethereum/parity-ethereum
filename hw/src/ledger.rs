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

//! Ledger hardware wallet module. Supports Ledger Blue and Nano S.
/// See https://github.com/LedgerHQ/blue-app-eth/blob/master/doc/ethapp.asc for protocol details.

use ethereum_types::{H256, Address};
use ethkey::Signature;
use hidapi;
use libusb;
use parking_lot::{Mutex, RwLock};
use semver::Version as FirmwareVersion;
use std::cmp::min;
use std::str::FromStr;
use std::sync::{atomic, atomic::AtomicBool, Arc, Weak};
use std::time::{Duration, Instant};
use std::{fmt, thread};
use super::{WalletInfo, KeyPath, Device, DeviceDirection, Wallet, USB_DEVICE_CLASS_DEVICE, POLLING_DURATION};

const APDU_TAG: u8 = 0x05;
const APDU_CLA: u8 = 0xe0;
const APDU_PAYLOAD_HEADER_LEN: usize = 7;

const ETH_DERIVATION_PATH_BE: [u8; 17] = [4, 0x80, 0, 0, 44, 0x80, 0, 0, 60, 0x80, 0, 0, 0, 0, 0, 0, 0]; // 44'/60'/0'/0
const ETC_DERIVATION_PATH_BE: [u8; 21] = [5, 0x80, 0, 0, 44, 0x80, 0, 0, 60, 0x80, 0x02, 0x73, 0xd0, 0x80, 0, 0, 0, 0, 0, 0, 0]; // 44'/60'/160720'/0'/0

/// Ledger vendor ID
const LEDGER_VID: u16 = 0x2c97;
/// Ledger product IDs: [Nano S and Blue]
const LEDGER_PIDS: [u16; 2] = [0x0000, 0x0001];
const LEDGER_TRANSPORT_HEADER_LEN: usize = 5;

const MAX_CHUNK_SIZE: usize = 255;

const HID_PACKET_SIZE: usize = 64 + HID_PREFIX_ZERO;

#[cfg(windows)] const HID_PREFIX_ZERO: usize = 1;
#[cfg(not(windows))] const HID_PREFIX_ZERO: usize = 0;

mod commands {
	pub const GET_APP_CONFIGURATION: u8 = 0x06;
	pub const GET_ETH_PUBLIC_ADDRESS: u8 = 0x02;
	pub const SIGN_ETH_TRANSACTION: u8 = 0x04;
	pub const SIGN_ETH_PERSONAL_MESSAGE: u8 = 0x08;
}

/// Hardware wallet error.
#[derive(Debug)]
pub enum Error {
	/// Ethereum wallet protocol error.
	Protocol(&'static str),
	/// Hidapi error.
	Usb(hidapi::HidError),
	/// Libusb error
	LibUsb(libusb::Error),
	/// Device with request key is not available.
	KeyNotFound,
	/// Signing has been cancelled by user.
	UserCancel,
	/// Invalid device
	InvalidDevice,
	/// Impossible error
	Impossible,
	/// No device arrived
	NoDeviceArrived,
	/// No device left
	NoDeviceLeft,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::Protocol(ref s) => write!(f, "Ledger protocol error: {}", s),
			Error::Usb(ref e) => write!(f, "USB communication error: {}", e),
			Error::LibUsb(ref e) => write!(f, "LibUSB communication error: {}", e),
			Error::KeyNotFound => write!(f, "Key not found"),
			Error::UserCancel => write!(f, "Operation has been cancelled"),
			Error::InvalidDevice => write!(f, "Unsupported product was entered"),
			Error::Impossible => write!(f, "Placeholder error"),
			Error::NoDeviceArrived => write!(f, "No device arrived"),
			Error::NoDeviceLeft=> write!(f, "No device left"),
		}
	}
}

impl From<hidapi::HidError> for Error {
	fn from(err: hidapi::HidError) -> Error {
		Error::Usb(err)
	}
}

impl From<libusb::Error> for Error {
	fn from(err: libusb::Error) -> Error {
		Error::LibUsb(err)
	}
}

/// Ledger device manager.
pub (crate) struct Manager {
	usb: Arc<Mutex<hidapi::HidApi>>,
	devices: RwLock<Vec<Device>>,
	key_path: RwLock<KeyPath>,
}

impl Manager {
	/// Create a new instance.
	pub fn new(hidapi: Arc<Mutex<hidapi::HidApi>>, exiting: Arc<AtomicBool>) -> Result<Arc<Manager>, libusb::Error> {
		let manager = Arc::new(Manager {
			usb: hidapi,
			devices: RwLock::new(Vec::new()),
			key_path: RwLock::new(KeyPath::Ethereum),
		});

		let usb_context = Arc::new(libusb::Context::new()?);
		let m = manager.clone();

		// Subscribe to all Ledger devices
		// This means that we need to check that the given productID is supported
		// None => LIBUSB_HOTPLUG_MATCH_ANY, in other words that all are subscribed to
		// More info can be found: <http://libusb.sourceforge.net/api-1.0/group__hotplug.html#gae6c5f1add6cc754005549c7259dc35ea>
		usb_context.register_callback(
			Some(LEDGER_VID), None, Some(USB_DEVICE_CLASS_DEVICE),
			Box::new(EventHandler::new(Arc::downgrade(&manager)))).expect("usb_callback");

		// Ledger event handler thread
		thread::Builder::new()
			.spawn(move || {
				if let Err(e) = m.update_devices(DeviceDirection::Arrived) {
					debug!(target: "hw", "Ledger couldn't connect at startup, error: {}", e);
				}
				loop {
					usb_context.handle_events(Some(Duration::from_millis(500)))
						.unwrap_or_else(|e| debug!(target: "hw", "Ledger event handler error: {}", e));
					if exiting.load(atomic::Ordering::Acquire) {
						break;
					}
				}
			})
			.ok();

		Ok(manager)
	}

	// Transport Protocol:
	//		* Communication Channel Id		(2 bytes big endian )
	//		* Command Tag					(1 byte) 
	//		* Packet Sequence ID			(2 bytes big endian)
	//		* Payload						(Optional)
	//
	// Payload
	//		* APDU Total Length				(2 bytes big endian)
	//		* APDU_CLA						(1 byte)
	//		* APDU_INS						(1 byte)
	//		* APDU_P1						(1 byte)
	//		* APDU_P2						(1 byte)
	//		* APDU_LENGTH					(1 byte)
	//		* APDU_Payload					(Variable)
	// 
	fn write(handle: &hidapi::HidDevice, command: u8, p1: u8, p2: u8, data: &[u8]) -> Result<(), Error> {
		let data_len = data.len();
		let mut offset = 0;
		let mut sequence_number = 0;
		let mut hid_chunk = [0_u8; HID_PACKET_SIZE];
		
		while sequence_number == 0 || offset < data_len {
			let header = if sequence_number == 0 { LEDGER_TRANSPORT_HEADER_LEN + APDU_PAYLOAD_HEADER_LEN } else { LEDGER_TRANSPORT_HEADER_LEN };
			let size = min(64 - header, data_len - offset);
			{
				let chunk = &mut hid_chunk[HID_PREFIX_ZERO..];
				&mut chunk[0..5].copy_from_slice(&[0x01, 0x01, APDU_TAG, (sequence_number >> 8) as u8, (sequence_number & 0xff) as u8 ]);

				if sequence_number == 0 {
					let data_len = data.len() + 5;
					&mut chunk[5..12].copy_from_slice(&[(data_len >> 8) as u8, (data_len & 0xff) as u8, APDU_CLA, command, p1, p2, data.len() as u8]);
				}

				&mut chunk[header..header + size].copy_from_slice(&data[offset..offset + size]);
			}
			trace!(target: "hw", "Ledger write {:?}", &hid_chunk[..]);
			let n = handle.write(&hid_chunk[..])?;
			if n < size + header {
				return Err(Error::Protocol("Write data size mismatch"));
			}
			offset += size;
			sequence_number += 1;
			if sequence_number >= 0xffff {
				return Err(Error::Protocol("Maximum sequence number reached"));
			}
		}
		Ok(())
	}
	
	// Transport Protocol:
	//		* Communication Channel Id		(2 bytes big endian )
	//		* Command Tag					(1 byte) 
	//		* Packet Sequence ID			(2 bytes big endian)
	//		* Payload						(Optional)
	//
	// Payload
	//		* APDU Total Length				(2 bytes big endian)
	//		* APDU_CLA						(1 byte)
	//		* APDU_INS						(1 byte)
	//		* APDU_P1						(1 byte)
	//		* APDU_P2						(1 byte)
	//		* APDU_LENGTH					(1 byte)
	//		* APDU_Payload					(Variable)
	// 
	fn read(handle: &hidapi::HidDevice) -> Result<Vec<u8>, Error> {
		let mut message_size = 0;
		let mut message = Vec::new();

		// terminate the loop if `sequence_number` reaches its max_value and report error
		for chunk_index in 0..=0xffff {
			let mut chunk: [u8; HID_PACKET_SIZE] = [0; HID_PACKET_SIZE];
			let chunk_size = handle.read(&mut chunk)?;
			trace!(target: "hw", "Ledger read {:?}", &chunk[..]);
			if chunk_size < LEDGER_TRANSPORT_HEADER_LEN || chunk[0] != 0x01 || chunk[1] != 0x01 || chunk[2] != APDU_TAG {
				return Err(Error::Protocol("Unexpected chunk header"));
			}
			let seq = (chunk[3] as usize) << 8 | (chunk[4] as usize);
			if seq != chunk_index {
				return Err(Error::Protocol("Unexpected chunk header"));
			}

			let mut offset = 5;
			if seq == 0 {
				// Read message size and status word.
				if chunk_size < 7 {
					return Err(Error::Protocol("Unexpected chunk header"));
				}
				message_size = (chunk[5] as usize) << 8 | (chunk[6] as usize);
				offset += 2;
			}
			message.extend_from_slice(&chunk[offset..chunk_size]);
			message.truncate(message_size);
			if message.len() == message_size {
				break;
			}
		}
		if message.len() < 2 {
			return Err(Error::Protocol("No status word"));
		}
		let status = (message[message.len() - 2] as usize) << 8 | (message[message.len() - 1] as usize);
		debug!(target: "hw", "Read status {:x}", status);
		match status {
			0x6700 => Err(Error::Protocol("Incorrect length")),
			0x6982 => Err(Error::Protocol("Security status not satisfied (Canceled by user)")),
			0x6a80 => Err(Error::Protocol("Invalid data")),
			0x6a82 => Err(Error::Protocol("File not found")),
			0x6a85 => Err(Error::UserCancel),
			0x6b00 => Err(Error::Protocol("Incorrect parameters")),
			0x6d00 => Err(Error::Protocol("Not implemented. Make sure Ethereum app is running.")),
			0x6faa => Err(Error::Protocol("Your Ledger need to be unplugged")),
			0x6f00...0x6fff => Err(Error::Protocol("Internal error")),
			0x9000 => Ok(()),
			_ => Err(Error::Protocol("Unknown error")),

		}?;
		let new_len = message.len() - 2;
		message.truncate(new_len);
		Ok(message)
	}

	fn send_apdu(handle: &hidapi::HidDevice, command: u8, p1: u8, p2: u8, data: &[u8]) -> Result<Vec<u8>, Error> {
		Self::write(&handle, command, p1, p2, data)?;
		Self::read(&handle)
	}

	fn is_valid_ledger(device: &libusb::Device) -> Result<(), Error> {
		let desc = device.device_descriptor()?;
		let vendor_id = desc.vendor_id();
		let product_id = desc.product_id();

		if vendor_id == LEDGER_VID && LEDGER_PIDS.contains(&product_id) {
			Ok(())
		} else {
			Err(Error::InvalidDevice)
		}
	}

	fn get_firmware_version(handle: &hidapi::HidDevice) -> Result<FirmwareVersion, Error> {
		let ver = Self::send_apdu(&handle, commands::GET_APP_CONFIGURATION, 0, 0, &[])?;
		if ver.len() != 4 {
			return Err(Error::Protocol("Version packet size mismatch"));
		}
		Ok(FirmwareVersion::new(ver[1].into(), ver[2].into(), ver[3].into()))
	}

	fn get_derivation_path(&self) -> &[u8] {
		match *self.key_path.read() {
			KeyPath::Ethereum => &ETH_DERIVATION_PATH_BE,
			KeyPath::EthereumClassic => &ETC_DERIVATION_PATH_BE,
		}
	}
	
	fn signer_helper(&self, address: &Address, data: &[u8], command: u8) -> Result<Signature, Error> {
		let usb = self.usb.lock();
		let devices = self.devices.read();
		let device = devices.iter().find(|d| &d.info.address == address).ok_or(Error::KeyNotFound)?;
		let handle = self.open_path(|| usb.open_path(&device.path))?;

		// Signing personal messages are only support by Ledger firmware version 1.0.8 or newer
		if command == commands::SIGN_ETH_PERSONAL_MESSAGE {
			let version = Self::get_firmware_version(&handle)?;
			if version < FirmwareVersion::new(1, 0, 8) {
				return Err(Error::Protocol("Signing personal messages with Ledger requires version 1.0.8"));
			}
		}

		let mut chunk= [0_u8; MAX_CHUNK_SIZE];
		let derivation_path = self.get_derivation_path();

		// Copy the address of the key (only done once)
		&mut chunk[0..derivation_path.len()].copy_from_slice(derivation_path);
		
		let key_length = derivation_path.len();
		let max_payload_size = MAX_CHUNK_SIZE - key_length;
		let data_len = data.len();
		
		let mut result = Vec::new();
		let mut offset = 0;
		
		while offset < data_len {
			let p1 = if offset == 0 { 0 } else { 0x80 };
			let take = min(max_payload_size, data_len - offset);

			// Fetch piece of data and copy it!
			{
				let (_key, d) = &mut chunk.split_at_mut(key_length);
				let (dst, _rem) = &mut d.split_at_mut(take);
				dst.copy_from_slice(&data[offset..(offset + take)]);
			}

			result = Self::send_apdu(&handle, command, p1, 0, &chunk[0..(key_length + take)])?;
			offset += take;
		}

		if result.len() != 65 {
			return Err(Error::Protocol("Signature packet size mismatch"));
		}
		let v = (result[0] + 1) % 2;
		let r = H256::from_slice(&result[1..33]);
		let s = H256::from_slice(&result[33..65]);
		Ok(Signature::from_rsv(&r, &s, v))
	}

	pub fn sign_message(&self, address: &Address, msg: &[u8]) -> Result<Signature, Error> {
		self.signer_helper(address, msg, commands::SIGN_ETH_PERSONAL_MESSAGE)
	}
}

// Try to connect to the device using polling in at most the time specified by the `timeout`
fn try_connect_polling(ledger: Arc<Manager>, timeout: &Duration, device_direction: DeviceDirection) -> bool {
	let start_time = Instant::now();
	while start_time.elapsed() <= *timeout {
		if let Ok(num_devices) = ledger.update_devices(device_direction) {
			trace!(target: "hw", "{} number of Ledger(s) {}", num_devices, device_direction);
			return true;
		}
	}
	false
}

impl <'a>Wallet<'a> for Manager {
	type Error = Error;
	type Transaction = &'a [u8];

	fn sign_transaction(&self, address: &Address, transaction: Self::Transaction) -> Result<Signature, Self::Error> {
		self.signer_helper(address, transaction, commands::SIGN_ETH_TRANSACTION)
	}
	
	fn set_key_path(&self, key_path: KeyPath) {
		*self.key_path.write() = key_path;
	}

	fn update_devices(&self, device_direction: DeviceDirection) -> Result<usize, Self::Error> {
		let mut usb = self.usb.lock();
		usb.refresh_devices();
		let devices = usb.devices();
		let num_prev_devices = self.devices.read().len();

		let detected_devices = devices.iter()
			.filter(|&d| d.vendor_id == LEDGER_VID && LEDGER_PIDS.contains(&d.product_id))
			.fold(Vec::new(), |mut v, d| {
				match self.read_device(&usb, &d) {
					Ok(info) => {
						trace!(target: "hw", "Found device: {:?}", info);
						v.push(info);
					}
					Err(e) => trace!(target: "hw", "Error reading device info: {}", e),
				};
				v
			});

		let num_curr_devices = detected_devices.len();
		*self.devices.write() = detected_devices;

		match device_direction {
			DeviceDirection::Arrived => {
				if num_curr_devices > num_prev_devices {
					Ok(num_curr_devices - num_prev_devices)
				} else {
					Err(Error::NoDeviceArrived)
				}
			}
			DeviceDirection::Left => {
				if num_prev_devices > num_curr_devices {
					Ok(num_prev_devices- num_curr_devices)
				} else {
					Err(Error::NoDeviceLeft)
				}
			}
		}
	}

	fn read_device(&self, usb: &hidapi::HidApi, dev_info: &hidapi::HidDeviceInfo) -> Result<Device, Self::Error> {
		let handle = self.open_path(|| usb.open_path(&dev_info.path))?;
		let manufacturer = dev_info.manufacturer_string.clone().unwrap_or_else(|| "Unknown".to_owned());
		let name = dev_info.product_string.clone().unwrap_or_else(|| "Unknown".to_owned());
		let serial = dev_info.serial_number.clone().unwrap_or_else(|| "Unknown".to_owned());
		match self.get_address(&handle) {
			Ok(Some(addr)) => {
				Ok(Device {
					path: dev_info.path.clone(),
					info: WalletInfo {
						name: name,
						manufacturer: manufacturer,
						serial: serial,
						address: addr,
					},
				})
			}
			// This variant is not possible, but the trait forces this return type
			Ok(None) => Err(Error::Impossible),
			Err(e) => Err(e),
		}
	}

	fn list_devices(&self) -> Vec<WalletInfo> {
		self.devices.read().iter().map(|d| d.info.clone()).collect()
	}

	// Not used because it is not supported by Ledger
	fn list_locked_devices(&self) -> Vec<String> {
		vec![]
	}

	fn get_wallet(&self, address: &Address) -> Option<WalletInfo> {
		self.devices.read().iter().find(|d| &d.info.address == address).map(|d| d.info.clone())
	}

	fn get_address(&self, device: &hidapi::HidDevice) -> Result<Option<Address>, Self::Error> {
		let ledger_version = Self::get_firmware_version(&device)?;
		if ledger_version < FirmwareVersion::new(1, 0, 3) {
			return Err(Error::Protocol("Ledger version 1.0.3 is required"));
		}

		let derivation_path = self.get_derivation_path();

		let key_and_address = Self::send_apdu(device, commands::GET_ETH_PUBLIC_ADDRESS, 0, 0, derivation_path)?;
		if key_and_address.len() != 107 { // 1 + 65 PK + 1 + 40 Addr (ascii-hex)
			return Err(Error::Protocol("Key packet size mismatch"));
		}
		let address_string = ::std::str::from_utf8(&key_and_address[67..107])
			.map_err(|_| Error::Protocol("Invalid address string"))?;

		let address = Address::from_str(&address_string)
			.map_err(|_| Error::Protocol("Invalid address string"))?;

		Ok(Some(address))
	}

	fn open_path<R, F>(&self, f: F) -> Result<R, Self::Error>
		where F: Fn() -> Result<R, &'static str>
	{
		f().map_err(Into::into)
	}
}

/// Ledger event handler
/// A separate thread is handling incoming events
///
/// Note, that this run to completion and race-conditions can't occur but this can
/// therefore starve other events for being process with a spinlock or similar
struct EventHandler {
	ledger: Weak<Manager>,
}

impl EventHandler {
	/// Ledger event handler constructor
	fn new(ledger: Weak<Manager>) -> Self {
		Self { ledger: ledger }
	}
}

impl libusb::Hotplug for EventHandler {
	fn device_arrived(&mut self, device: libusb::Device) {
		debug!(target: "hw", "Ledger arrived");
		if let (Some(ledger), Ok(_)) = (self.ledger.upgrade(), Manager::is_valid_ledger(&device)) {
			if try_connect_polling(ledger, &POLLING_DURATION, DeviceDirection::Arrived) != true {
				debug!(target: "hw", "No Ledger device was connected");
			}
		}
	}

	fn device_left(&mut self, device: libusb::Device) {
		debug!(target: "hw", "Ledger left");
		if let (Some(ledger), Ok(_)) = (self.ledger.upgrade(), Manager::is_valid_ledger(&device)) {
			if try_connect_polling(ledger, &POLLING_DURATION, DeviceDirection::Left) != true {
				debug!(target: "hw", "No Ledger device was disconnected");
			}
		}
	}
}


#[cfg(test)]
mod tests {
	use rustc_hex::FromHex;
	use super::*;

	/// This test can't be run without an actual ledger device connected with the `Ledger Wallet Ethereum application` running
	#[test]
	#[ignore]
	fn sign_personal_message() {
		let manager = Manager::new(
			Arc::new(Mutex::new(hidapi::HidApi::new().expect("HidApi"))),
			Arc::new(AtomicBool::new(false))
		).expect("HardwareWalletManager");

		// Update device list
		manager.update_devices(DeviceDirection::Arrived).expect("No Ledger found, make sure you have a unlocked Ledger connected with the Ledger Wallet Ethereum running");

		// Fetch the ethereum address of a connected ledger device
		let address = manager.list_devices()
			.iter()
			.filter(|d| d.manufacturer == "Ledger".to_string())
			.nth(0)
			.map(|d| d.address.clone())
			.expect("No ledger device detected");

		// 44 bytes transaction
		let tx = FromHex::from_hex("eb018504a817c80082520894a6ca2e6707f2cc189794a9dd459d5b05ed1bcd1c8703f26fcfb7a22480018080").unwrap();
		let signature = manager.sign_transaction(&address, &tx);
		assert!(signature.is_ok());
	}

	/// This test can't be run without an actual ledger device connected with the `Ledger Wallet Ethereum application` running
	#[test]
	#[ignore]
	fn smoke() {
		let manager = Manager::new(
			Arc::new(Mutex::new(hidapi::HidApi::new().expect("HidApi"))),
			Arc::new(AtomicBool::new(false))
		).expect("HardwareWalletManager");

		// Update device list
		manager.update_devices(DeviceDirection::Arrived).expect("No Ledger found, make sure you have a unlocked Ledger connected with the Ledger Wallet Ethereum running");

		// Fetch the ethereum address of a connected ledger device
		let address = manager.list_devices()
			.iter()
			.filter(|d| d.manufacturer == "Ledger".to_string())
			.nth(0)
			.map(|d| d.address.clone())
			.expect("No ledger device detected");

		// 44 bytes transaction
		let tx = FromHex::from_hex("eb018504a817c80082520894a6ca2e6707f2cc189794a9dd459d5b05ed1bcd1c8703f26fcfb7a22480018080").unwrap();
		let signature = manager.sign_transaction(&address, &tx);
		println!("Got {:?}", signature);
		assert!(signature.is_ok());


		// 218 bytes transaction
		let large_tx = FromHex::from_hex("f86b028511cfc15d00825208940975ca9f986eee35f5cbba2d672ad9bc8d2a08448766c92c5cf830008026a0d2b0d401b543872d2a6a50de92455decbb868440321bf63a13b310c069e2ba5ba03c6d51bcb2e1653be86546b87f8a12ddb45b6d4e568420299b96f64c19701040f86b028511cfc15d00825208940975ca9f986eee35f5cbba2d672ad9bc8d2a08448766c92c5cf830008026a0d2b0d401b543872d2a6a50de92455decbb868440321bf63a13b310c069e2ba5ba03c6d51bcb2e1653be86546b87f8a12ddb45b6d4e568420299b96f64c19701040").unwrap();
		let signature = manager.sign_transaction(&address, &large_tx);
		println!("Got {:?}", signature);
		assert!(signature.is_ok());


		// 36206 bytes transaction (You need to confirm many transaction on your `Ledger` for this)
		let huge_tx = FromHex::from_hex("f86b028511cfc15d00825208940975ca9f986eee35f5cbba2d672ad9bc8d2a08448766c92c5cf830008026a0d2b0d401b543872d2a6a50de92455decbb868440321bf63a13b310c069e2ba5ba03c6d51bcb2e1653be86546b87f8a12ddb45b6d4e568420299b96f64c19701040f86b028511cfc15d00825208940975ca9f986eee35f5cbba2d672ad9bc8d2a08448766c92c5cf830008026a0d2b0d401b543872d2a6a50de92455decbb868440321bf63a13b310c069e2ba5ba03c6d51bcb2e1653be86546b87f8a12ddb45b6d4e568420299b96f64c1970104000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7cd58ab9190c2792714ab06df5b67e66d9e3873eed251d7beb4fa252d6fed6a0ab1e5fabd284f40878d38f6e63d72eec55c6e1aa8d79c06adf714e3523a1f83da763f4bcc9d34424aba82981534066379c1cba244352042de13168556be761f8b1000807b6a6cd340b97a93cd850ee54335b1043bac153c1b0736a88919bb1a21d6befba34d9af51a9b3eb39164c64fe88efe62f136d0bc83cad1f963aec6344b9e406f7381ad2462dcf1434c90c426ee907e6a05abe39c2b36d1dfb966bcf5a4de5af9f07819256357489365c96b21d92a103a776b656fc10ad1083cf679d240bf09bf2eb7635d7bfa969ce7fbb4e0cd5835f79ca9f5583e3a9eca219fab2f773d9c7e838a7a9ef8755dc22e4880367c2b5e40795fe526fc5d1461e50d5cb053e001206460fc6617a38499db525112a7edde38b9547853ad6e5ab359233611148f196501deafae414acde9df81efd7c4144b8fd27f63ac252ecede9609b3f9e634ae95c13058ad2b4529bbb07b5d7ac567c2da994084c3c73ef7c453fc139fcdb3939461da5bf0fa3f2a83517463d02b903af5d845929cf12c9a1479f6801f20085887a94d72814671dac994e14b2faa3251d465ce16d855f33259d94fcc9553b25b488d5c45fe74de60c303bc75bcdde9374ca268767f5767638d1aec5f6f95cab8e9e27b9a80ddf3dbbe24f790debd9e3baa30145d499dd1afb5662a11788b1bb3dedc1ebc5eff9641fa6918d958e4738bae3854e4cd43f9173cd4c9c821190ec287c18035a530c2dc63077d292b3a35b3756ba9e08295a02e37d332552f9f4fdbb945df004aa5b072f9f0e9fc2e4ed6fe455d95b003e5e593dcbfad0b3b47aa855b34008e0e9a2e1cc23b975a3e6808be59dcaa8a87145c1d5183c799d06100d500227e6a758757b4f7d042b3485aa0ce5e91b2b2e67d3cfdf1c226b7ab90e40f0a0d30cbbf425f495bd5a80202909ad419745a59210e2c42a1846e656f67a764ee307abbd76fbb0c99a702253b7a753c3b93e974881f3c97987856b57449e92ffa759da041a2acac59ea2d53836098196355ae0aa2a185dbb002a67c1a278a6032f156bc1e6d7f4ff6c674126af272fdfd1dcd6a810f42878164f1c7ae346b0dd91b678b363d0e33f4b81f2d7cc14da555dcbe4b9f80ac0fed6265a6ecce278888c9794373dcb0d20aa811a9fe9864fab25eaf12764bb2f1a68cd8756cd0b3583f6e5ec74ca5c327b3f6599fa9ec32ccd1831ae323689ef4a1b1a587cbbd2120e0bb8e59f9fc87d93e0365eb36557be6c45c30c1baeba33cdaa877a87e51fd70f2b5521078607d012d65f1fcca8051a01004a6d10f662dfa6445b2ac015cb3ce8fde56bbff93f5d620171e638c6e05504c2aeeeb74c7667aee1709846cb84d345a011c21c1b4e3fd09774ab4dcc63bda04bb0f4fc49d6145d202d807cc2d8eab29b3babe15e53a3656daf0b022ac37513f77660d43d60bdd3e882eef239bfe13dba2e12707733d56e49f638005e06019a7335d8184f1039ab18084de896a946c23045e5c164dc9d32f2f227c89f717a87d1243516b922e5f270c751f1bdb2b1d3a38a15b18a7b8b7e0818573f31320d496e14a348f979b7606c5124e007493f2f40c931f68e3483a46ab2b853a90bd38ae85e6252fece6fd36f7dad0d07b6763d8001a0d6abee62452904f979cc52fa15001b06eef08f17d6e16d493d227ce9277392337a1c71713603e03803d38d1c24184b52049bc029f4f00b22d2acdef91c776a74aa184cc84b0e764f463ed05c2e16a7a0dcb6c27dd4aeca8aeac1545b48896775ba3fe9de4ea36e946d8f4ec16ca7ae58165e8ddc9189d5cc569888a59733529add4b213ea5c00ad3ed3709c0175b542513c90e18f2d4fa2301389102839d969e9f0d614943fe489750f27382f7ab273f51fcb995f449fa5fba108ad0955ed0819a0a62308021ac4ab0c97f04de9fb8533489b2685447ad71c7f9a9bc89975f9cdde87a3af89ae5bff37d1f192a31b7c5aad50486931bc07820d7dae398960965baba6cfc05c56df18b8ef0f5db488eb87be803fc94e3ad3bd6e4f358fe7ce15ca21c9a4752ddfa98337177a7c096d829886e8d71340a01644c64090c84e88235b11bd1fefe506d59733cdd82286fb466ee215914b06a138356e82c0ae6d5fd8e5fb310eb375540308d95b5d53832a5dae9652f91c1e8c14402991e38836813604dcaf272fc552e7682a6eaa7aacfd4ed1c7107b0232cdee00aef865c5577f2391937b76e34810f9d49fe31e54425b6f5e1d0e436e1366e9762d8295877e27ae495ace18fccfaafd850544c9be949d15d421cf6f4bb180225f7f86ca64480975c486df0eeb4fa80a4632cff28d36585cb5dc534553454ea810260983d02060caf6b1eb2b9443b1552ff73d243fecc9779635ed137a3bc8c04ef13f0329a7a5a54b2af0738218cc91be0ee63512f009435d8623ff4e8cdaf743818510b22e42b586a7e5e75525bb61dd2deb96adc95e07998a265d58fe4df4b9ead5b5f15b9daee510558fbdfae7a56931a6f4c729c18e0d29c467fed504810b7d9dfa0613d1657d9bfa5887e3f327cf46d7059a8a0fd654c60cb9c683c55439cd5186d1615f45f7108f261aff77791cf24c975120acf2b357dfbd2defafac0016525cff9400e0feeddff27910fbf2fa84c35fcaaec90863b605db5adbad0593601447605d68b943249861f8cd33c6419c7611403376a6bb438ee857ced2e6842f99ed1b4a9dc79f835813a4f8d07c14f1ef98773286e79cec1c9ce8c26e00418f1b27c7ef104fc96ea2b2ddefb46e2fec4feef2771a1d7e2643586b6fb97094a8d298de12a6f8f78d88e5d67442ed3310fb40aa6439b89c834e43ecd4a80c0a1d74ce6a90a67bcc996a7e93b6f397fe7ab2fa43711a72b84f8c94bd1e4ac62657b98a4b814d8ef2bb469165464a90d5353aa95d09b6ef4ffef081cab5e9dc12d743364f06d4118a585f7d455fd6e3b01434a728a768987c181409eb939e9396666560d394fb151fc67cb9cddea0a94d3e33382bd0617c95304da97994f110eafaaaff6eecb54421e01dc850dc73d77df18bbf68ecc8b37ee2fff7b6f88c139f7d88d763248deb8b4e16a8fab216c0ce88faea030f3a5c994c6e4ef6a9a68cbc9310787232198b020a7c014a1fa32c1736885603dd4921cd360bfb7dca7aafcbe81d7621dbeb4e5c094c2584c339ce70176d7fd2a6cfc4bbea6b433377eff7320d412947ac774688010369b197ec4d0471b9cc73cf9a3e71bd10901beefb10ca1c53428b89ea63427aae9ede5ba104d3fb54d0447458dd9780cd4e925f1edad33f6f0884cc47da562a3c6e2f5a958a8d8723919c4b88d067343a246c6722b6f9f82018d5213648792f38fa8ea1e635b3983dc1f941630fb3762ef1814ee3f41691b24583ddca585289568b4e64f82448b54797d382916e562b3f4795e2d726facea988249e2c3f72d44ec7197b6f783c6c7a133004d5e131b7b4d6a9557c56942ca4bd1f070a2b46c3a6b81bb9a4d570ac6afea75de65ecd331dff1e0252e0f9095f974f47b2d340d67704343b2e8832232210d2f79665bebccab528745c1dc3b28a78aafa3785c29ce2eb6a8403e4d8eded1cc2554ece0a542aa2febd711164f7d7e3a492a87b01d6b4206e593b3aa6d431e908282fcfee0d14dae4b99176a16fa32f730c2d336dcfe7eff84a7aaab1fc32ac8c2e9ab6ebb72c0306bc6998ec22d6cf20c2b6660cfbbeb064b3047c1cf650df12bd153cd7eec5dc181e46575f07c8e292cc191117cd28302d1f9c72d79b1f4062dd683ca95c3a744ac310764e56b2f02a0c2850a2f24c1b298e712374e9adfe68e5414386d7671bd52f6f472eebfdf51677ce379afe7b8085459fb1e6966f5cef45b256489b7ec8a8939cd931009c8a26642f1ff78cab06a5d25522a922cd5e4541dcdbde4848177a42476b141ce9ea035d28742cee0e5e85eb78ceb2b720e112aeb76cd0eb3fc34574c7476110b3b9dff5c19fceae816715b31fc289c0e7149e8488a59e075ac6683f237886a63a25ad23bf903480b9acf3f724d5ace0ca3a842939d4828910cc735e6513dfc4055624d68a048a626fab6b910eaf558c1b43daf1cf26338bca68b5e308b734b61624c97bf70a82430d586a6c3cf59e1bab2532fd9fa1f6fe4f757c7ede0cabea52f2cbf00cc88ca7db4ccc0ff92c0836e7405ebef2ad2e4b7d3b455d8e4d9ae575d884347bdadb67f5e24058a44ae1335280b671ec3bb9d8247e28fecedf5c151fe892bb0f6e67351752e4b1bf75dcd5af3e62ab4aedc5aa32a1606b4a0de3156b356b0fe74e898065d1e720b81663453fc97f935da3b5755a0629f38d6ae5f8e5e77eb64bbef5fc70d4081ebee7a9f7169df4f0e11796f7a79e9128ec996b6fbd8f6fa56e11f17db4925c27f4cd3ddbdee8a50e0b0d4d8f6e527302cbc4dbeef4b0338e6ac7515c1e796b39c8e83f457b50925c39d405f4cd3c1aaf3188c5ac62bf1dd362bc8c9d4e49d3d2b7c2dd2291fa4bb22d7cbe7963b654d92643b789366d1dce842f47919a1cf5073da8916701f907c4d2f8a710c58e85b59f590123d3f8e57cdc14df41a1481a893b9f9505dc0637ba9b27657b0ceab87b0e4bc742924e6d8bf895b407c54df8622018417f9e543fe49f5b10a7a5fc66e5589304af33a20ea108ddf63facebcb20d22eac2fdf4a97285ae6d3f87865fae1331d00e631dfe5366345e0d78bb39a8077484a941176bc63f469f001cfd230347580b6226d6adff5ab112dcd53e7118925296b1a05978a703e383e6ffa5158fc36781f74501564992ab244d3475e1ee8e7146033da2dc116489b84c378e4a750947eb9ccb982a197f13976bb105c81624618c697f32a5b9e03f3675b2315fe773e4922c2e3da7f68ac225107405ece58dc6bbe2bd8947f3e4269ce245589497cd892c750f9ace0440f48057090c8a6cbd5046d3d982d634b4ad6ba41c7a38b7b8b0f91cb6898e769479fc3c7e7d2010b7fb38ef13c17db705a36455a34969803323806009a4e141a5c42da0f7a5e4760d07250d7e483ca6274e57cc2885e5728c24c8b5102845e8bb74b1c394fa7a206ec052c953967380d64c148ca480ab0edbc5da1a7a1e649c2ebfd19fefc52d81aeed7cd83f3c1d2128bd66feb99d5d8fbced01383d2abbf9be47f3390dd336c22b533a731d1c59c3bc5361d781ca15430d84f3c67d6981ab99100f53b6b5623df9d8eecc99d24e02d9301d636c2d5988e98a54339d5b516379a67d50dd9994a28fae5b806c56b353a84cb31729487a6d9851960b83ebc5178be689720a80c5c412e67f8ed55724534c92ab15c3bbc5bf13dfbff02d41ce4c9bc112746b62dea2b21d034e9a31e276eacfeeafc672b95e701ec0fc7ebd4b020a73fc37361b3f136246a0e3a8378442eb5e60abd7da2032dca9b5556aa22e5007c901f438c5e1baeb5d3ec6128a84d310363c6ec17d4ffece27f502b5c63d20cb1d11d0cfc316074faa820a03e6c577389e5e82ebe5f0976b6f5266618f5eb56986714d5cc75fe87176e92dcf01c58029d2b838022c0812c933db17dc4566d233720075065fda26f44b0ed3a46b6143fe180b7a1e6c1558f87b875aedf8c2fa968e2c925f0c08c7e0f23a9cf1b46f7955d9f1db300dab801f5672e2a7231bb2b622b0dc0dd9f2ec64a5f10c239e613247f8685369ed60b2d262c038fcc43924c5aca318385c12412b10d89753f9dfca43eff5f2be7d7d7b2788b877efa8b46ec5c9e99f922839bef71c613cd44cba597cf68de366eaa8874032c14d8012b41e72fd66422f7031d26be0dc4fef8f36a3c124e4ae767a665a94233812984c4466f5bd698b5fc22153c9c2f4110d9defb23c00e722692983b32ee0e84514169910bb21b14066d048960b29b3ff4c090dd5723ca4dcdebd207d4f88da831f0ee7de4aa302a06589a4aba3ca696e7d3c3e9a93af79db91f7a06b0ad825a8652f74bdb72f580e9afb31aae58807e24067f08dd719abb4e6e458bc8aa272d7a5bbd00710c43a1fea220b9022a26b574997517d04573786a4c3e09d30f3ec32f328462e26d4f7ff015121758ce1a2fd51e7f419eb6d8ac04497ab812aa6ba2e981a312ca16c38ed887b2342b0a91348198797919671a23e2b0634b523f931e48ce0d8eb840c54045d9193afec069803901e5ec1108782503cabd0f43373a85acacfa8af44ef2b1d09e4589d2dd4fdcefbf435cb61254f189ad433fa6a4e190627732ae4ef2b0c85cfcbbbaa0137033034e70a3906112dc76ec101f3198e25fb38aad46261d6019690dbf059d66c44e7ada244589c55edfc2e7d18c0ddfcd2d3841bd54d8502763cd0f4696d44686ae3be29ba3063ff6e7aee14de126dc43302f7c0b57d59eb4fdfc4903ccbd3f7309225dd90b5f25c5ade49c14334c0e00fd18b1dc611b10fbbb98c560ad4908842e765c661b9bce005aeede6461254338b8dad3203ee1b58bac1062c7e02e2aa6d420283ed81525839f2c8ff54ac71cc105042c594fb7fd7b55c14cd1247347a197ea8f93c1bbeada1dbf3e59b798c9b15765ab23f856fcf4eeaa5892c3857646bcfd8ad2bf0a15607e0d6696a8548da32955f1f8476f8a20fe4f59b3e9bf4468730b8d46c824a370d37695d1bdcac521032804c5cc66505637701e653ccbddb052f4ecf185b3605d0ba3a4fd99161973e36a35bf79571841ef7506db822dd2a5c959f36418a8dd8acb5b3ecbf3e7918a73695501ef8f440aba43c6e4575880ba3bb83e0a839254fd8d8c6b979d79337a68d218565a5dcb1518c6c82aa73ce7f54a9434ceb5f5fd503137164d74a230e46ce298b98576fea88806bc51e393acdb2abac1da23219b4dbcfba366d834d40dd8e616d214c3478136050555539eba776bf506870c3d20c4a4645b9a7c4ffa976534068009840aadae71f578ef1a325717f64dff840b9dda81b123086a47a172e6793e68af6140b1492058fecd68c4c23db1cc13d2b57f52d0cba89cd4c26d1bd580dd2a054a1d934a80b9eda8ffb503b7e3e62d00a3d075235410149e976529d8029595e4daaae1aa685f3cbdac9b26916320e75b0846d2de8673600212bb648b26e3f1709df425136f33f46129afc90839d24de1e9fee51c685db8a280a5dd4c3ac1539664cc36ffd4537af480d4082146e7395cd6de1f8b652bca8853ec742366702afd6ed79a5920e4ad1317545266f6dbb796ace0fdc731997cd94e1bd8e6689c856adcf153909cfe882b9b02650f4f9eb8620983f0c6b95b3558682d8134a9ec8fa97e174173041115b2eae21fa0b72d0a3c7c2bf9b022fa141a8b495de8321c152b0a9a942c5baf290a234ade4e8b579238a627196fa5621b196ecbe31583517ec4ed82e8d3fb21a892dfd65ccfccd2d36c5d32afa4d4bf201d684c4b1c8c1207db455dede5b908ac63d5fc0bd2b36e11df53bd52e5ce27a9af9444a8cc4391ccc82914b79ba2971ef4ea5d5c30372e7cdbe9bedfcea9ccc8140f8c3ad1bcda29d11fe51affc74f17c9832798e10222701e0d6e93fd109cc9a12df4ee5d38c531574d39a9f4357a60f8150ee509c68e469b4eb0e9be2e6ef9099f1bb949f738fa801d223316fbb1e179b74445228c8b3c40440306e4821077860c37d6b8c17230fcf7ea48d0bb0d98fd3f1f00655e11a8b2e0a7d5da8427784a8fc6d1a2d4d1d3adcc02030b50a700788ce4078c199fc733e2ad469dd9c775d7a8025b4db9b960619f0263b7f09d038cdf85045ac2a1cc5a18364048bf242af713ac4db889489d781ff16b1dcdf66acd89bd6c7651f25a17ce751b67697739dc4d1a125fdd5a8ecbb0cfaf31cd4179249e91171ef3e628dda697afed9d09b53260ae475d59ccb45a6ffd85a2c4241fd134462cf2ec21b51422439aac77954d1b2396761f16e1c6e3242b538f23f584b95cd4b811e35a526748050a7eaa02cebdf8887d94287c99500bf9c2afb7f36ff47e17906534097b02f10620958e889d2392d30660e513c22f580a505314eea4a865d97adb9136c495403e321f425348b56ce8f8e8e91ccd702ade0bbd1efdebef8344bb9defd471ef4b214976556f59f679e0fa39a2007bb9902f5a60ba044c4316c27f6b634241acdc3ce437c4fad599aabba291bfd71c05eca6d9df49abc33ae7709f6622e516c22418e7ab86144f6baf3697bfeeee65294175e5dc9ce5ec82da64537f5f5b83f5a938e41fa8f6f97f9102fda8bcbfb6a5c58f79648b97e948a074e459b9b75a1793cf7d9ca5d7ab27cf7035ece0612d348a23c0fed509c5e18d19b1e659af237c3b9aba4fa8477de805c5f8ccd0cbf3846b6ee1bc9ef76a190952115bd08a5108c8bba76d8d762184c122d081c6dc8b4c49a7f0e16ad4cbed86c6818d4f22c03a100c9afe3675a2f354bf1c2cde1f5e5a63b95761e10d27c9482539387e3aeeaadaeab59faaa20cf595d4d8c57509c751446282581ed28cc55736211e6fabb63d0f299e39ac1cd2af1431bfb03f86e5e59691dffad4e275d4611cb2d7d3be3defcb77907c94db86d989a2ca7e19729e3454eef23b0d58bff8203b08f41b40913f2d2dd2e8c98af09e5aaee76030d8201640d78e7bcfc6c1171e04cb39a6bd060ca41ebbfd090883d8b3569c39fc19cb5d87c15062c9f09138d4e3d3f3421227fb2ac48b224438b12702cb67e2db161a3c771d866c3cc55d15a094f72fe314092e846256e44a1dc513b02bbdd976321f470f81f36e719b9acf22179855d36ad0c50dab79da662e9ea7f9685ec0b44817271ffe2b7254ab7f3ddc389847e17edbd33fbf789bcd604ccca0c01c60deca286858b16dfa17c5875916e0159dfd4f0495c08bf6de51365e2175e47325d5ee71c96ea8ce24c4541886e0854bf7dd8a980aea1aba9add0316f3d052a2eea95c02c241523f3274ee62c883c4ac440d7626cdb4f0aba7a4ea686b2778cd7d7be220357de63cce55a3928aab4c200a2cd65b04d831ba0b54dc91cd6ea410359512130d2a0122f3c9752ba6210ea3b115caf891f0a0a7ef210d1988324a9af926cea8487640a473aefb2e3b4b9259ca4da66089d7f7800f87cb2bd068b8c268dfac897b9a2dd1ff4ac2b19a48b7e95a39ebc6afa2dceca7928ed8e43630d673e5c7ba1fb4afbbd40243ed411b6519420e738c24ab183f900872f10248190358636c789b842f156987d0593fa7cb813f5c688652f871aada7cb5a9c2e15ddedac147151b4d5a7bc4b33cecac961a3487984918868515ca73ebc647945fd9044f3c085b184b3f9d333a7b74927fbbe4a0d846744e0fd6bc36f9381f76422633946fe79e64c3fd63e30096ef400df8cd8c884bad1955b82c013c1a190db92699d39217e46d3db284f35b18b782e791d722d12b85c8a26ac98e9dea8356f9d3ca58833aef4ffd883953f24c96f5351438dccf33693230db5d72389905b49d7308cc30b805fa968532a976009a527bfce9ea921ff4ea9723be5b5972ace8553441a4dac7f0b2114edd3a25666d70c4f94131a63f4521dbd004309157bb32f9fc649058ffbe747bc3addc523f805f1b34787b0f446c9ed1d1966550c7d0c10e342316c6b34899064d0d2dcbb09087ac20572103ee01193a3eab06c06e3206cd60bdbe367af81dee5ab3e5dde9836c558e54c9bb6aa306a609225cf25a65b575fa97d9c962b72b798e9a7fd8192ba879964cedf623d544c8929af5c8dea56721d25578434e2b234289895c697c9c1bc4556e4f6df479a837d1e9132c011e47f9e23fd27b70e7601fdd24f28937efb9e46673b9f56914638c793f5c3b625664f2b221afb3fce5aee92a84d45bab5cda58c49777f82b2b1c8293d727fec90dd73581b087367add474dc7b4cad75cea1e43619ef3fa1b35175f5f0889c031c2083e764b0f4389fffeb307831b73763e73d2c3112adff579d4dcfa1c09d3f2c5927568a70027242e6bec83c5e2cf7e125d8b5e4ad2ec339fb79bb15b8b9a6db0ea9408fc6fb8ca6efe9ae0c8c25900d859b17fc44c4a262c7a5e06ae9e2083fc6dc36bd08d648e9a1a3d8fcbedf12777d690ff15dc7096e7c8b33e71b19005c9e1b20d2c2b6f5c7c1204edc691b389b6ad04f896ed297922bb92b9e6d10a2df2a83dc71c15d2010b595c72d5677017d6d7938ca3538d671e13b8496583b4f9fa59fd481f1f438f92b01a6c5f7169d44b93c0b6863c1a183e871e7f50e26e6d41243a1c509d423309dc886dbb9ac245263ae9d6024456e72b57e17cb08ef00f4fa4dd9fd27de0685c4c6c680ad654e3d81dbb450f0a5e7821412d442c2034093e3fb10234e6a51b98fd388eafd0eec66b42c275a3547f72c7f3d16ed81395e9a2664faacdf99bb22327280e518e4ff047451e6f7420b562c68877c96e129d0cbe18896aff48d49da028dc97aa0108da9b29c540c5238d676dccafdf463694aea34ad4f513b6c7a58d071c335ff1313d41b7cdd902904b8c9fbea2ed34878b407ebb8144f603683ce4ce61eb0690a00d492978aac3a0f3010b7479667811c3332c06553c14809c723316c84d084530e93a63bf0b7658f7bf367d29577236e23ab658a685f2612f0216a932a24aa4f70b8d0609aa9ca14e4d91b8ed9fb62864ded646012ef675ea359117c07f528d7dfb742aab9ac892851e97c94f72d5c34d4feebc7f67e09fdc6f633f050833192f15a7acf4f8c8beb3adf3860fb26fee39a416ec362e4b6d9ced09fa57b3d5b7fb7de018e4fd93eb65634c08f6d4f1e2f490c2a8b1be2794a27de0dbecc9949fd1d5eefa0fc6f0033a2bdecfcaa267280b445e92385d2edd4c2b31bdc5d54ddd6cb30b3c370a893c217945d346d1c5b8b98ac754a01afeba6f5526939ccfe9f2432461a99c7b9b44a3983eb65fb064c32f8c72e18b8f6e42e72a1bac21b3cf94526f81089b235794412d1aed20f48324d742d4079e9546f495248cf7f42839852d604598ca2079fe44b125ae9970973b57c156e83fabe6d64c9aaab5c243d1dc71520d45317b913205979fe5bc075b0068d8a5ceb7c8ff9149c763c22b08d35a09feb8156bf7d8eda212a102906e251efcef1ebed894556f18444a0938b4c050f2b873505bdce97cd4fe539a944b94e281292f38850dec9e9f108d3b2d5a83837d114bcb3d6e6511629f310d194328eb05a7b88e7a053e97dd92881c89a1169e7d23a4fa1ebf532eed2579fc4482b9c93da2b5e9619f289f346160996cc61a3f380ea71b25e777af37dce79039cf90a2bf16ddd46733fe9c1cddbe7a42fc5faa7869c96ec463e9817495bc24a23cd9968213927522ddb0d6ba5db92f5736a5723135305a6c083a9bb54da7e43da3ebb07066ad94e597706062118fef17e9e65363f71d8859d30527a495f06bb025c1d26c6fc80e9b140c7108c57ee5583063bd8d2a7efe6a3026a79f2294e09ce980be8ce1a017132ccf48a63eb32454b12506a6099d4e310f07612e77da46aa0caed8fb0446fd6091140db2cb1432bb93cbf681cefae9d849fee6b0d87898d52d31a209ca6f168b6305011e2c9a55fc5ad2237d7c2d06b98e0703ff2a89fc7af8471aecd2a6cc0a4745082db863bc8d46209d51135333a03b328345b86d6cfc23d6d7384fae5d8546f05725ab139e2c25b0dd9b2113b2774391aa058cf90915bc97a94e74ca0ff6785243122f12decdc48aaa8ff27200007f35e928e62269f7f07407802c9a10648a91180d559c5c37cf3f425c9949b9e38ce4c99b71810babe45344d929906776a66fab175e20bc5930f1dc4b5b888301028b6e0f92293e468d0c6b191f0840ed822c036e6257bbd4f0db8e931463826c0be855add67bff5fdc6d4de7347fa07e63d68f4b6876774a39dff1ae927614f8a879f128713e24b263850f1ab3176ed0e9ca9369af947bb8e862e927cf803ea7b53b68eb8c5f87f1cde2399122b7892ccd4071610f0873981ece2ed719bebb0d508037e46b95610d14e9a826549cfedecea1d32074aa439592929873b49d9434f35646adeabc8b52e323ec2dd6d0d6e27b530361fd8bf9e4e3a0a58e3079dc63156a684bd5cde53ba8c9c51da274bd61cdab187a3fc0a84d5005319f05fc7ddbda575f73f3178336413f8ba0b99cbfdd5c350a3a925260284d75fe06371716f951d76078df7cbe6f25beab46b8f4222c74f68822d6747314b688839540d3bb9bd0f45a028e780fe2b5c78e28dbce66680f1e57b68d6088101146aa9f976bad10933e4f5481444a46d40413ae5d00044a29dd3760c712c04771976280f793ac5bf8cc1187976096e4620d646358f207a9166b9d27030721fc00688a0df926e6f4944ba6e78dc862a8e55e3d1a20d2993d8c8410548e9bf1b6efa181daf8bc060bd1af3dbd8853d6d3f54bdd1f6270b20fcf7f90310109b98f6b366a4ebc6f717962e408bf865d0128fc9ed607f848d376ab1c50e66152f74916a28539a762c75387d144bdaf4a0b8b0e7baec532e8d531501674a8727547916fbcb2e45f9c7d41063bcfec3de1b0adee000e555397ab16fb0977a8c3ac1385dfc89eb7db5cceb9109077d36ca9ff5fcf9feed6b985693746a95ba34f7d2875f61ee8606302b6470f8ad17b781daab036e288e5ee083a3a36eb116a34f5ad97e1675181818289f514efe868feeec3b48b1a574b9405668aa536e572f0e2b46fdfccaea5b2f65285f6a9a05c020bf440f5db912c8ac289c67b9d724225eff88366992f08711f35112e66b765872d39b54cdb5c4c0719b2c17dfade7e2f19281e6ae7885708ee8a8f6f90ce79387e6e47b33f15f212c5b386a5aa5f93cb597698dae4b5999ccb4d652a08c41ed27c45d2ecbd112a679374ddd6606ca76ceca9ab08f7f648d248622ddd633dfc121f9470930ae058cfa9455ddbd25a38aaf48f242ab6e0dc895c5b2af0d9ab0c996df526f144cce6297af5f3ac5fa1d159f52e072b827dbd273afcc6e3b8fa1151acaaca5965a4b6cf5b0ea6275da3208159c6bd6d716eb61309eb4ddfe1bbc4ef8d013d477668cb3506ebb4724ccc72affdab79dcdfaaee55a5946b4a3f768dae9fedddedc6c5712296f26c025ed2ee299cd15b1e692c616094f500fc53fcd9838401c0ea6b6ccb883c149a52d875501ec2e647b1d6720a8227e33cbc1f429ef60103f3334e3de2e40ed4a59d811b8cc51a695de25ebc66eca519222dafa22dbca634220097b1d3f9aeddc91d11019d7215629122b4dc6e3211ad842288b581c31e44fa79e1f7855d8fa77e7a224cf571aa3c16b5f4fe5feb16d7d1bdecc543b0e8ff01c677ec6801e87241ddaa02a5c83bbfd1d84c62e269f6ce8a708e693b86d8e5439f129431a4c1c0bc6ad47784c38e1cacf6c523da23f65a76c264b96aabb50aa9e299be6abd1c9d078ac3b2c5f2c3986b5707f143513b4ea91a2052731ef5b48780dd0cc6626a0f0c358454f6eb36df7caee6f8dfb3ea19a0ae79c0d1587140147be3efb2a0da1305d5fe056010c518e3471572d889304c4ce00acc78fed04a4b888d5e7e57d6cb5cf4e5cf1f8782e1b25ad948eb3e443db75af9233aaaf6659adbe0ef33d4b3ba5214b85e656719df2eba42235b2e268f80e3c5971d28957f8e93f5b04a3d5eaa607fd4bb838ae48661bd093342762cfd1ed60b21f04f5b95c3e5426ca6127b04810e2ee25bb56ae81d7840328d8d4f7d1bd341ed58b102d9860806f4a4d117c044f472c85ba422eab084faf8994cfe0a880bc46dc9c1a8c11995610756e2ac50c5fea8ebbcb53dcc76b1944ce364f8878f42310fe0f8cc211c62f627d12b20527dfd84b78c98b1122050cbcbdb70e08010f68294a6a805d3fab97e76cd695f918e73763ac2c3dfe4a8d75db87dc37e2399fd854f3284d29c7bae3d3e31c4375ad9e047f03a5204c2ba93b6025c112ea2c9fcd731e380a8aaa42860c859c2e2cfd333f0bee741e21f78776defea86e862711f0d0bbf64003ee848a8d1a12dd00c024cbee343d1093e653555c033c198401caeb951860392b5b1eed6200828aa310ed466e41d855dc4231464adc2b6b6fd66e03fd42736fb791387efec28b37d0686272a6bb181a621aae7be06866bdc1c4be69e94642c8d3782f5ab7cc8c890699008b52a11b149a517771b93bc2ae597dedaf0237ea8d9674e26fc75c3b468e04e2fc317d03484a75fb274f7ba1617bbb72ec16da1fd4109952d052e9de7c00761736dd17e70db0976692626ccf8bc9e88ad6c25ed88a2f7c2750add4ceb95744f690ee5f2fa423a2b62ae57c1105958bd8e81025c9412fa71f5d1e81bd6cffa01f489fab7e90ab8a3c8aaffc8e3d594beb254c460347196473117ec2a416dea464eaff95da6cec26b5535954901298f11932ebeca52aded139f2d5aa2c24174e2f6c701ce1f4564c60861ce3b9cdac1cfecf071295c5ec581f0f075096fa457373c124b6c8cae3aaf915e4701ad94ec9c01e5ca0552019bd7f107a7d5afab9e4a5e7cc7b4c5416656ad064f4a0f89afbf7c5b884b69a12fbce8aa73a49b2e5c5728c67a7396bb8341afdf52213b2f7f8e84962cccbeaea63a3c7b24881ecdde39cc57b4f211cd57c6f982217758042f61b648496e62b612b7b8bbe1b9f15d237aeac42b54d15166b5c71eb27ccca1fc9e050adc62a267eb82ca2144ba323a73aa11e2fdaa87695c70316754faf7aec44a49b668362b0b35e884019227e7b9a35e8841e64e0009c713d7f3e4a74cc3feaecf4c99b8d0ecd85c8ff89771b63a38e3af990641f28fa7e4ea560577d600f43ccd467d6a347fef04d392d42f8e97659348c68b41299f94db4b713d61868adbd20a4db74f61bd0d1e7846bfc8b8f8bb50bf50c2fbfdaa87328933741aa2b1ca50cb759c1276f1a7930952ed656921f5ce5569ed16b31b2a1b6009c784199ae60ce2e35d573808a195974536f220cd14dd634bd06800435cf1219047f6246c2d9bdea5e489ab4862f0cb0f01439ad2ad1e2042b3f63b8611a87efbe842613c21761de4c79291a8491092c20134252b8e900e5d3cc70e75d32cc41452c5c33b66087213c34f67ae73fd56a183be858f1c3bcd73d814bb9e3f78cd18992b0ea401d8f25c3b60c055df8e6430b62899bc86167d0b5e2bbf16d75bf3f2b94c26542202bbfa0abe99be1a07c78140f42c12f51576007bb5439966a47cadf5c4ea624a75e7a4f01d8733aee57e3497c013de4a33cf54a94acad9b1aad837865a6881db9a725310eed49581d2223f2b0984757bf3fc5122c5dd572ecc781b48fc508122775779d2b2849e11684a585ce844d21352f8d35ea53f0f34d772bd9ca76cc4dc33aa3f2e72418c097614fa5260eaf3c2d724d3599dfa0991a9c0eec9c4d550886c85e1ab2541e9868a36afbe0d9c07c93e44c4c73c66f88e770e5d4e4ac331fafc6870c928fca85756c444c6e8f6cf75865859abf0cfecc8e89b8c806a2e6af7cb752215bec6201eeb41759b27d599931dc2ae75d605b3e387bf263ebfd09ce2154b81479675555ec74ad85150f8eb8c1b3c4f31f6409648f9c1b4678c82e8e2afa9c887f3210afffed160d1634ab0259e1bf5565d8598605a435bd289afbbc12034f67199b67bb0fddb4b9180908c483ae5a8eed16221687e1f524d010ce5db78d1b999069f225479fd6bf0681c7ee95d4665925bc96399989b85284087e67d5a070f2713feb78bcb91bc019f3f19bf3abb7cf36ebb98f09fd64b61e2bddc9ae6335da48ba85b62562726e142bb9d9e5c8f278dbaa0657dfe3e410f03211a072555624d98790aefe8e7b0281ff6af3de79dd5a414632f9d4913a480e9cd6990f94350304f853ba5679a4cb3a647b98bf1eee6cf70f77581a1ff82a9ffd7296e8fd172d37b1b0d1621692cbfeff8de18658f04af5d5be08bce66e5dfec5989b674219f9ceb6a1037c80a8febdfac63d482debd34c3057a677420f0bdd66e2c2b25a9c1d34b76b4a998ad3ee21d1e49f812422c83016c12c201ac2b0f07ddc00638846f215bfa6c575cbfd577178eb0282ade2c459a13386f5dee8a7502321292a7de077f4fd12967b8c8055596e7a43287639843b6ebee58d463fa044562ec2da7f9c2a7f28cce685178eddd3b9fe7b10202997b6b170555a71555cfebd06cba6bb019f8cfac2ec5db3b1d1ca88acef9accf76b6a74600e590a0eba1c839d6a577d3877e7d6d010b04fc58e160ec9733bf200a9e0b24fe8ef32613cf2c7b1515008b8833e34d3967ccbc8bbe30fd1810f23bb153b814392eb37d8917e96260b3cb16895ef13b96d72c81a14b908224571680dd56d04a59a6583a232ec58e8cff16f6428b5e3dd19f362992608aba912b642aac9950777627ffa4eadfe9f31b73c3fbca11d2abb623b732f3d7c296806151257c9f2306dee1c84eb05d586e7a82a8750905716b3e51600250a1e3b4bf274130a1bfa47117cc8b6db3741ba04d977015b8ee250c3ffaf859fdf0372b88fec188830b5870f251889584333547f3436a548801fd3236da2ccb2b504f85ef1d259bc3e00f0ced934a4b297ecce0d668fb3ecb524d3ff4380a7856c7060006de31931d0b26ec1d084e0dce3b9a123741cdc326b441131d777799623c6340410c331c7e8a4a8175d7d250274cc4ffebd5d46d855bf90842888893c348f0a447998e3aaffc81c9b65e3a772eca5c2f0907ee13ab6a2babe99f388755fa3ac9dc79a2ba4ad7a869a876448ed1d4dd6a8c678065cfc90df8470b29c83719bfcbec7c5e3244a665a28593ad42ab84663bccf570a8e8b783565f909b5e6e8cd69ed6f79fc945ce5d845c998f25b9dc118c96dd2c0f592a73497dbd9e050632c8d82656a71460d0ae7f5f38636692a78083b2fffaa517dc2dfe18ae020e6a5562be54ed9046c7129b3a57dcbd1917efb0579fa9a3978690fded8e52e4860db75b2a93c77316a6e84df4965291a7531e2abc0fcc0d0016acc29680baa575cb7be1a03206236310eb5120ab4069e0f8f0cc3f6bd188ca91963eafc2bc66b1a42f8c49359cf3171a72eef94eddd8aab03f770cb2f489aece4e09a85fe6b9790ced5feced19e4cfe6bcafd1a5d99fe56b78f7a14fdea11fd5e331e23191a3f74b32d8ff2740409f346aedf469eb8aca16b43dcc44c400ae3e6d1c4717ae1f18a2f70830aa0c4d5734922374dad8c006ab97e02a4263999ecad0b1e9f24ed0b599467c962932ec610e63c0b3ac845f5d4d10979c92bd884669908696172609e0da039728baa1f0dca8885d5439ca420e87f5c449908b2a5f69b65b60adbf5d74b21eb1f4e0d79558c59b4499c245a9952de8d3a51021f2e77c44e06a489df3b72d28e5d03ddd358ced4f5a1fe057e58b86f9e717cb9001cec6d6665cc0f5b9cf89873e6e7d10355746e99494766c937683684312b630337d1c411f3f2eddc52a8267e19d38ee12c810cc4e33193e26790b13d1847c56282ac86697996daa386b06ec2ceaa97fac9c018baf644622c74546177267b053a82292c1a1cf194909beba3f2670acf1d095b0caed4b8da2fe48c9da3dc61969d938707a62ce9cf55b89ceaa04a9069d38f4e89db794a335933c5b45fe215976e76dc71b7719c2ef29d06d2dbcfce0470007331a221dbce6baa3f418f989d7dd927d343152ee310d084799300e8d3801f9d464d9bbd5687e3203cfb8e589fbab39ad4851b07bd13b29d7f4b767858d13c5937a482207470f673593aa9abe339b3d63b7ea4ad60e51e7f9080381eb07213ad1996ba7bd28f8b44b7ea037e0bf9716f56820f908fd4027249df11aea06df25b3860cb18b68a7df5ed0d14730035291346049e1e5cbdefb30719548fde4f986bd9871a71b5bc7f6e03ea4fcf1c6ddfecb06413832ac27b08d203070acdaf432bafdb288908dfd673caddbfe41af8255ff7106d39db8d003ec1abcc3000bd7fe1daec2624bbe8417f81150f20a8a48324100ef1570a6de7c0a21e16f6991b23016671bc96ee55e99a97a5a0120af8ecb816137d5f40b9e71d56cbecf61569dcd2f850ede77437be06fd85b54d7220b9bcd13e682a8227c7a05a4efc8d258b0331b0f47cf45ec370b491d6b2e4e601e50483480d9437fdf570b6be69b28b964972fac047f8aaecbe567c8ee3d583a46d5b58fa3c361dd3ad73c91727e4d0594f428acfa977206c20995612834497928d507eb62aca1752a8f3048c932b9f0f80f7c627a87f2b50d581961b8739bddfe2afabb1c757f366acd1e639de808409f598755dad254c60b5aefbbdcbad52f72c756e5e4b286a6866af769593f66256fadc939d3d23d1db9096038b40ed224ace023f2e3ea84fb4092c974cb44ffbe489f0ddbdd79e66281ef9c44e81781b849b0d3101c17e54ebf8bd69393b9220c75c7d3c564862ef35d7dfedc855e2ea15a6159c6c2bd01d2c4f3c316ddc43f937cc295fe35365a69ffe68a2a3bfa7eff90c2fe8563f6438117c31ab48cbd5a3ef1c7a03a03a048be4a9fe0de1d6a86feb144731f4e84f1b509db65d35b1b8ec3d0f462392da10694b207ef1d9fa2581b572f9c45012151f039ebed848b3fc211b2b4d6d48266e8bf800e68cb1165cfb17cb14af4fff107e57bc90b9e32006dd090ae12ff39b000c474f77da32549f51d07bb23d233485be9143c55849b5fa241337c050d48d88e4723f7f1032120cb609c584cb10cd777404556df84cd095c4a9668d392cb9a6197ce04e4234d48b47f8deaad83ee95292c9a9e9d42838c12e34046483ebd821284ac349fddb3d89c0e9a85716ca5f2c60569686d3580c6c7bce0a0ec4183fea724ad02763f66f85992fedf49c67a54c8ecc5b47d6e00cfeaf23b2425b795be93d65d92fe0ac761cca8b2feb4fd7a4bd21bc98a7328f178a61aabc2edf843e23ee94c757a457d448f3588b4e39cb14d855c35372c2060966df0e3382afe2d18988ee7676511e43afae09d6e16b50bfd290c1202c5c82520bfadb7b9eff22c2e9d202e7606f23182c08f0d405cfda6e8bf4b222a14a96015602cd77b2e0af5027938348075115b146166990bdccdaefa94626e140f8ea6fe6b51fb38fbf7ec39b89e68174db08d243a5da08a573545993db451bcd7462ba2c308849e6f54fd68eac003dff1971d19a00ae1d326d9db706197ce15397066ca114645ee39bb1a950c068908be503b2cf3ee74048dd92808e07172ba1362b3ad4103953c990e19b4581c54b5a240d90ec56150fdd5d9d1e497090941b541a9fa202d09f2790bd29f53fcf2adeddd4b4ecbff252921feca36cbe51e5185234641c8df314dec556280e408ad6605cb82f9fa5cbec32b2d478e876b4c3bc5019c344ee2f0bc33d26ae3b69e349771a8069f38f879d82e1c68f84d44516db921ca606b6e310e9ef0729b9fc76eaff94d3e44f865a6943eecc5ea1dc097e69e91344f7b287223fdf25ed3512e1fa34b0879ade1a2786571435e71d3fab19a6ba93b5d83e20f05afba10ab48ddee2c6feee813635318ac35bece3a339fb5c2278df5b9a6b7859343ff5530a2dbeda669a47a5eb0efc46c148ab00165563023536cf71f189c6b855ca6aaa056233ba82edf29e82d96c6118a0e6bf37d2ab2945ed1904f1dfd19ede3dfcf257aea6d560e3776159ffc384b3540deb1cc38d1022e530c2d46557a21eeb744ed5c00843f7b6d5953f1ff4770d26dde34c4cfbd308074e0df53264afc5a3a7ab8a57dae296c39bd72b88ad988319ba9e13ea529783d5c926d2f48599720695fd174f8873d0f660f002d8d0ee134271450c12e9dddb641b240795c2c09b958778e16081bc9180442c45fa916de16c83f16c50092eef58a56191bbcd906eb475b97d37b7f5cb00a79a9ad66a636e1052f9dd1e75d02a5af4840dfda7eac68c749bb857675e67b450a484d3e7b13a77fdabff0e97dfb705e5f4f6cf1e95a5f6cc38e099634a020087f868580ce2ec0837525b8c58f08444d7fd4333a589c0356de22568b4fad8766ee3325cbd65843f2c713ecdb44c96411ea871c039915b546ed6fbafbd51805ac48d06c6924d3f7036e1814250f50f27342c8c4ded3e68b6b3f161d46379c1088a7a123f48f0e7cb5a348f472eb155956fe232fd301e64f341041683ce3b25bba7f290a10282a8dba3a2a3da24461a5be148c2241d627889adca5acad981583fac81d0ee4ef77038c1f80db9dfe740720904512691a9c8545a9d173c08c2e8599010c972c2c34287d91ac7803a5700a0d6e29b7774f8f487b70cf8d0ec9474443e2c0c051116b16aef491c3945a65e6ddcd7931a7259e56902a2866b95d3c0bb7a3ea61b1f3b54ae56e6a7366ea895056ea0d1c251cd74f7b82b0d47464826f4aca77434df3d909271a825b57890cd830011981d95229cc0427cdc97758ddbc76d6cc77ba06c92d19daac8bbecbf55535e98bd4754ec06a6e632225c43bc46068baa688636eaba53926ca093a7addcd6a696a902ac35631aa43d9d66f77270cc7bf66140dac239034ba304e1aa0a265131e9fb2b7f079861b0e4cb9c911ce82ef0b685002476baf26401dc8cc444543129f82ac6b103881c596b19d9eba8ed6b230c17914d5c34a0040c18dc54d8c4b637ee683637fc5a82ac1cf12691bb28fc0bbb307fc032ec3d2b06eaec56ed769b5e892816c7350dce89551e87918f67a117c39f256a368586c78c2e9614e9658161511a8dad53afe8cb9eebe67c6596a90eeea1d3d2466a4d77a1129c0a4409b98d8ac0b925c4b2b3500665a3cf4ceb82cb0b6732eea8a796f9b79d2ea49be97066bc1f606d9f1f59f41d2acbb878a0783093fc4ab0ef866ff60a6a1a58d3cee90307f09247b5212f8709856251ff5d8fb77657110bbb3f3aeff07898f049c821a82c11e27b0c176a9feb12de5d08498018f7607156c5065cb56bf9d6867a4495f26a07e0f01312c2ee897b82d8eba0cbc473da402814dba727521cfec6afac2cc59cdd6a75e1f8f40585e5cda51a7434a81ccf4b7de33c663dc174ba973cebc5a56831005d231c719ea34ce42999c471fccfdbdbaf1acd2f9c16f258e32c70511c475ab264173246ebf31459a05ecb4df443066b61a243903e80ff907af17a96d7afd9763df8f8c4fc49775bc805e2dc165bd6f1c4e06688521557ed9ddb6860fbed1e32957bea1174b3a9aa809d7fa6301fbbb6b3774cd856095f14c6378cfe98f05d4f06fae91769165dd0adfc51bf8f57d701ef14a99d608db0a104ea78fe5b13794cb8529afa5352d1dbc8235d96148c8f9c2e29d6e2359a8dbeba56c9376b26f8384c66548979f4d982fe0652cd86bb60e6f2463ec63dcdd5f93d4bfaefe48f8012c63b32ad3c02ec9088896f6a0c8b1097c1ad911ada7a2d6f0d201a28b70752182885464dd688535bdfc045e8dafbe34b20eca00848e757b4a37de219be5a5fe7a4bc5cfaad29ed92e9eda2bed08407e0d0f53caf6b3590210067d8b9ef16f9a8f5612315dfa415f1efc8d7349394143a149480ce3ccd60ccaff0d9a8a797820f41b431ce3afc4adb2e07cde16015087e09e08bd13471dee960db35cbc3b53c187a5bca7ed50017e09b2ae2c837b1f6557753c7f5b004332ffa2b52d8a2269e7cf9cc397c6079aa5add61d7a560a894e71510e104f52a93622e34037b1db70a05bcfc546ea2ec7153e69a8df18fa9eadaae2c1438710477a9a23e0f7092c310c5288e2d39d362a0a33f9e3d8d9792b51a71d9014abcff66ee509baa3dad341b1e4b6c601a2966f77172a4df0f32170f3386a6600b0b63699fe21e26eeb475507e99f666e0ac349b9e23463450f4fa4498356887d9e1c5f7d18ade51e526d27ccae799d6775336ca9ca8e54d707639ecb0618a3c675533494e2435c0b3780a66defddd217d2cc464014bef8a051d8f292abf9e5cafa78c600c21ed3d40ede937b1e162a1e14757d39d77d4fad8711b6b46ae707b82ced0739f9fb6bcd9b557982e89bb3af5f3fb5448ea960f454f4475ee78970acda37501a8825a04cecf3e544651eea8933379da3c3e7de0a875d689003c00d276470fda3b6ed6473cf8094ab91784d1c0f9468379e8e9729dc1032a5ca14378f8147409f13cd6994de961e2245b35c814596087625d3d3267fc0c1e5614a4af94993091ead40bc9e1d3093228b70c188855ae9e914b15aacfd4f83fde83072af92b2cc968c93cac74e15322eaff32a7bbbb982fb725aeb71f34bf16323d9c0a11dbaf3ab676a9cd1dcfc3f8a0c66d1f082f23806133002c50b59d4513dbe3419d5002263287ff47abdba0862341effe669f26b375337170c8e0742113e1063e8141c4aa9eb4970471f3187f581b71e6f7fe2f8043d065620da8a066d112fedeb33525eb1061c0d0fe9fb415bddae8ed2eb5c3ae6aa0549230e436afacaddc389b2c66499d7fdec2090e7e13560ca0a64803554c7cd9cfcc1cb48427cf9ccd954bb7446c887e2756db2882ff12eaa64efae3a24b35d1d0402922efe90319510495420301d3360f4486d3f87e3dc4f9337bf3fb4e3c6a82850a840153a1936e7cf74086757b72a8db19d33a62a29f3dd4fdef454d9222031aa0958af21851b66aebc09a5c08efd204f3ff18cb1055e8181d6630309fcc91c0d6daef19e618a3ee23e817a586d02364710cfab0b9f2cf18502a34e67d112f1730d44ccae54dc221d7f3877bb828e7109878109f8e95e2e1407df4e588801d25d9c2a1c501e74890631e9a92d823ebbe6b5635488f7d48788ef77658e3bbaf287536b37d3a7ab1ec1749656f2ebfe562765e71dd3e1b895d9b5c315fcf2b3a063c57e74ad1e7586b293ede4c77732f38d316c14210a121153fc50007f78ed64a8e207e9d04b312ae7f97a946c74d2a1181b67e845c3ac6e340b2428c8a5546679707fded3406fc221900b118a3279e13b74926c793e27fc4cc32ae478b4421d6eef75d3a273ff61d0e95b4981e8dd57e16bb00e09bfbbc2ce60cd844a9abb839b8b671fabddfd6e86a30c0a24e73c3c17770f34641951e5dc73ca11d8f8419a7407d483e0f5f1714df0a1775574b5500e8a5a28c655dbc28d7a1ca4b83fd4ebcc7ef2e4994c97c87659681acebe7417328c8612e8570e7ade7ead7f4fc711c9c539362779e6be525bdf5ec037f670b5235c06a1acd89b4ffc21668a7269cc73bf6d1399852eebb8b1dde8ef072e8d80832ba32c8e9480da2c4f5c3209c557f31beef41c00d22ee7c7e2c1bf9952ba8a03c1afae9b4aa63135d2b131f2b2804afcdcc762e1bcec8c8151f471572888933ce97dd787121ced446aa9718bf3766bb6d8a752692c59489d5b565e1693aa0f67b352f915808e415cba13a9864bbd33ebc97dfdc0d357d6769f2f545cc6529c0f634da901ae63bfcbab0a3896bc43faed6a6c23bb4e92f3d669d2e0ff485287cce322b98d02866f026cc556ec8aba6608ac2b5dbc29e104ef2e28d7b51ce63110025bdbfc5d44e8aa7a04ecece07b9860618a162e7289e8d672bb9b15b6ffc87f738b0c7a2b733c5794afe58b1beee4b6780ed453bf2ef2b584dcf32bf732c98fe359abced05fc115e531b088c61b0d5d5058af10120581d7db192e13a5b7b17874f000343aecc8d5005b91b13720bc831de5f1de5e3ddce27ba05213cd126a7cda0afa9745f498200269a5736f63b0faec36bbd646a868100c17cb7f6639f2f14b6c52198fab04c1645bed8763799acf8fef62b82fda1825a3379c000255002788d686695b4c17be3931e69db8980d0216024e9b7b0588cdf8c8102d11f55f971b3163c392cfaa796e0b85dd0bbacd6ca50b3ab80c2e90fa0c18d3526e05b2a46c2eab823c0511b43c71122d533e27ee6d6e34706fc411c67a3b87440a3429df3009996743ed3e4dc244fac98a789f17818a926a0aae81ecde260982b80acc299f57a570a86ee28d0414edc91fb6d5f9a88aeb31bf22270bf3517aefe1140b05be97123cc43df6e8e8e4df96803fdd59715c87afcf0189fb5448663eb35d2c4e5b13dd0233a95f8d6187bf0d5d3ba35adba59e162e877d5a0397d9495ebfc771ae68283be15d883e91b81b1bb0cd8da6c300df7e2bc8a21094cadc974c8270d8ee37fc7e7501a57eaecbc244ed61cfc8d556e38c0611a5269c3b930ee5f37a9771f0c152a5e28df07a104360c973b9a83d3ec5c0aa012bff141842e9b68222647c7d022753dbaae024877f421ff36b3721c26a39b3009683c8c510ba0ba8b5dc1033f9b56e9a43b3141a92599378622a2ca8136f5f1f51cf7b7dce7d043f65f8562b33c4864adc30e7d4c808b10abbbd92f94272b68b063f7d7baf7fd6eb31cc76690042233bc8dee7253f89ce23de7a535af022dae95ac321694d6ce311744d9c152e4424a0a502d221b2e602ada71c60a2f15b7086d75867476b0633063297681fbb0a3e154efe552cdbd9d3203f2e447b60b643b823ea12f504f33f6b6c3bd20e54cf38e3c45c5d472814db60741687894e6cc3c78196d5e722499d202334fb742f14dc2ccb7d114ae0c4cd61ce2ed0cc7fe25a395d6b73c1dfee9174e59d129e7f3c42f93a246d918028d4e2dc804438799
").unwrap();
		let signature = manager.sign_transaction(&address, &huge_tx);
		println!("Got {:?}", signature);
		assert!(signature.is_ok());
	}
}
