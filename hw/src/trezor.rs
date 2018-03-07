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

//! Trezor hardware wallet module. Supports Trezor v1.
//! See http://doc.satoshilabs.com/trezor-tech/api-protobuf.html
//! and https://github.com/trezor/trezor-common/blob/master/protob/protocol.md
//! for protocol details.

use super::{WalletInfo, TransactionInfo, KeyPath, Wallet, Device, USB_DEVICE_CLASS_DEVICE};

use std::cmp::{min, max};
use std::fmt;
use std::sync::{Arc, Weak};
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};
use std::thread;

use ethereum_types::{U256, H256, Address};
use ethkey::Signature;
use hidapi;
use libusb;
use parking_lot::{Mutex, RwLock};
use protobuf;
use protobuf::{Message, ProtobufEnum};

use trezor_sys::messages::{EthereumAddress, PinMatrixAck, MessageType, EthereumTxRequest, EthereumSignTx, EthereumGetAddress, EthereumTxAck, ButtonAck};

const TREZOR_VID: u16 = 0x534c;
const TREZOR_PIDS: [u16; 1] = [0x0001];

const ETH_DERIVATION_PATH: [u32; 5] = [0x8000002C, 0x8000003C, 0x80000000, 0, 0]; // m/44'/60'/0'/0/0
const ETC_DERIVATION_PATH: [u32; 5] = [0x8000002C, 0x8000003D, 0x80000000, 0, 0]; // m/44'/61'/0'/0/0

/// Hardware wallet error.
#[derive(Debug)]
pub enum Error {
	/// Ethereum wallet protocol error.
	Protocol(&'static str),
	/// Hidapi error.
	Usb(hidapi::HidError),
	/// Device with request key is not available.
	KeyNotFound,
	/// Signing has been cancelled by user.
	UserCancel,
	/// The Message Type given in the trezor RPC call is not something we recognize
	BadMessageType,
	/// Trying to read from a closed device at the given path
	LockedDevice(String),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::Protocol(ref s) => write!(f, "Trezor protocol error: {}", s),
			Error::Usb(ref e) => write!(f, "USB communication error: {}", e),
			Error::KeyNotFound => write!(f, "Key not found"),
			Error::UserCancel => write!(f, "Operation has been cancelled"),
			Error::BadMessageType => write!(f, "Bad Message Type in RPC call"),
			Error::LockedDevice(ref s) => write!(f, "Device is locked, needs PIN to perform operations: {}", s),
		}
	}
}

impl From<hidapi::HidError> for Error {
	fn from(err: hidapi::HidError) -> Error {
		Error::Usb(err)
	}
}

impl From<protobuf::ProtobufError> for Error {
	fn from(_: protobuf::ProtobufError) -> Error {
		Error::Protocol(&"Could not read response from Trezor Device")
	}
}

/// Ledger device manager
pub struct Manager {
	usb: Arc<Mutex<hidapi::HidApi>>,
	devices: RwLock<Vec<Device>>,
	locked_devices: RwLock<Vec<String>>,
	key_path: RwLock<KeyPath>,
}


/// HID Version used for the Trezor device
enum HidVersion {
	V1,
	V2,
}

impl Manager {
	/// Create a new instance.
	pub fn new(hidapi: Arc<Mutex<hidapi::HidApi>>, exiting: Arc<AtomicBool>) -> Result<Arc<Manager>, libusb::Error> {
		let manager = Arc::new(Manager {
			usb: hidapi,
			devices: RwLock::new(Vec::new()),
			locked_devices: RwLock::new(Vec::new()),
			key_path: RwLock::new(KeyPath::Ethereum),
		});

		let usb_context = Arc::new(libusb::Context::new()?);
		let m = manager.clone();

		// Subscribe to TREZOR V1
		// Note, this support only TREZOR V1 because TREZOR V2 has a different vendorID for some reason
		// Also, we now only support one product as the second argument specifies
		usb_context.register_callback(
			Some(TREZOR_VID), Some(TREZOR_PIDS[0]), Some(USB_DEVICE_CLASS_DEVICE),
			Box::new(EventHandler::new(Arc::downgrade(&manager))))?;

		// Trezor event thread
		thread::Builder::new()
			.name("hw_wallet_trezor".to_string())
			.spawn(move || {
				if let Err(e) = m.update_devices() {
					debug!(target: "hw", "Trezor couldn't connect at startup, error: {}", e);
				}
				loop {
					usb_context.handle_events(Some(Duration::from_millis(500)))
							   .unwrap_or_else(|e| debug!(target: "hw", "Trezor event handler error: {}", e));
					if exiting.load(atomic::Ordering::Acquire) {
						break;
					}
				}
			})
			.ok();

		Ok(manager)
	}

	pub fn pin_matrix_ack(&self, device_path: &str, pin: &str) -> Result<bool, Error> {
		let unlocked = {
			let usb = self.usb.lock();
			let device = self.open_path(|| usb.open_path(&device_path))?;
			let t = MessageType::MessageType_PinMatrixAck;
			let mut m = PinMatrixAck::new();
			m.set_pin(pin.to_string());
			self.send_device_message(&device, &t, &m)?;
			let (resp_type, _) = self.read_device_response(&device)?;
			match resp_type {
				// Getting an Address back means it's unlocked, this is undocumented behavior
				MessageType::MessageType_EthereumAddress => Ok(true),
				// Getting anything else means we didn't unlock it
				_ => Ok(false),

			}
		};
		self.update_devices()?;
		unlocked
	}


	fn u256_to_be_vec(&self, val: &U256) -> Vec<u8> {
		let mut buf = [0u8; 32];
		val.to_big_endian(&mut buf);
		buf.iter().skip_while(|x| **x == 0).cloned().collect()
	}

	fn signing_loop(&self, handle: &hidapi::HidDevice, chain_id: &Option<u64>, data: &[u8]) -> Result<Signature, Error> {
		let (resp_type, bytes) = self.read_device_response(&handle)?;
		match resp_type {
			MessageType::MessageType_Cancel => Err(Error::UserCancel),
			MessageType::MessageType_ButtonRequest => {
				self.send_device_message(handle, &MessageType::MessageType_ButtonAck, &ButtonAck::new())?;
				// Signing loop goes back to the top and reading blocks
				// for up to 5 minutes waiting for response from the device
				// if the user doesn't click any button within 5 minutes you
				// get a signing error and the device sort of locks up on the signing screen
				self.signing_loop(handle, chain_id, data)
			}
			MessageType::MessageType_EthereumTxRequest => {
				let resp: EthereumTxRequest = protobuf::core::parse_from_bytes(&bytes)?;
				if resp.has_data_length() {
					let mut msg = EthereumTxAck::new();
					let len = resp.get_data_length() as usize;
					msg.set_data_chunk(data[..len].to_vec());
					self.send_device_message(handle, &MessageType::MessageType_EthereumTxAck, &msg)?;
					self.signing_loop(handle, chain_id, &data[len..])
				} else {
					let v = resp.get_signature_v();
					let r = H256::from_slice(resp.get_signature_r());
					let s = H256::from_slice(resp.get_signature_s());
					if let Some(c_id) = *chain_id {
						// If there is a chain_id supplied, Trezor will return a v
						// part of the signature that is already adjusted for EIP-155,
						// so v' = v + 2 * chain_id + 35, but code further down the
						// pipeline will already do this transformation, so remove it here
						let adjustment = 35 + 2 * c_id as u32;
						Ok(Signature::from_rsv(&r, &s, (max(v, adjustment) - adjustment) as u8))
					} else {
						// If there isn't a chain_id, v will be returned as v + 27
						let adjusted_v = if v < 27 { v } else { v - 27 };
						Ok(Signature::from_rsv(&r, &s, adjusted_v as u8))
					}
				}
			}
			MessageType::MessageType_Failure => Err(Error::Protocol("Last message sent to Trezor failed")),
			_ => Err(Error::Protocol("Unexpected response from Trezor device.")),
		}
	}

	fn send_device_message(&self, device: &hidapi::HidDevice, msg_type: &MessageType, msg: &Message) -> Result<usize, Error> {
		let msg_id = *msg_type as u16;
		let mut message = msg.write_to_bytes()?;
		let msg_size = message.len();
		let mut data = Vec::new();
		let hid_version = self.probe_hid_version(device)?;
		// Magic constants
		data.push('#' as u8);
		data.push('#' as u8);
		// Convert msg_id to BE and split into bytes
		data.push(((msg_id >> 8) & 0xFF) as u8);
		data.push((msg_id & 0xFF) as u8);
		// Convert msg_size to BE and split into bytes
		data.push(((msg_size >> 24) & 0xFF) as u8);
		data.push(((msg_size >> 16) & 0xFF) as u8);
		data.push(((msg_size >> 8) & 0xFF) as u8);
		data.push((msg_size & 0xFF) as u8);
		data.append(&mut message);
		while data.len() % 63 > 0 {
			data.push(0);
		}
		let mut total_written = 0;
		for chunk in data.chunks(63) {
			let mut padded_chunk = match hid_version {
				HidVersion::V1 => vec!['?' as u8],
				HidVersion::V2 => vec![0, '?' as u8],
			};
			padded_chunk.extend_from_slice(&chunk);
			total_written += device.write(&padded_chunk)?;
		}
		Ok(total_written)
	}

	fn probe_hid_version(&self, device: &hidapi::HidDevice) -> Result<HidVersion, Error> {
		let mut buf2 = [0xFFu8; 65];
		buf2[0] = 0;
		buf2[1] = 63;
		let mut buf1 = [0xFFu8; 64];
		buf1[0] = 63;
		if device.write(&buf2)? == 65 {
			Ok(HidVersion::V2)
		} else if device.write(&buf1)? == 64 {
			Ok(HidVersion::V1)
		} else {
			Err(Error::Usb("Unable to determine HID Version"))
		}
	}

	fn read_device_response(&self, device: &hidapi::HidDevice) -> Result<(MessageType, Vec<u8>), Error> {
		let protocol_err = Error::Protocol(&"Unexpected wire response from Trezor Device");
		let mut buf = vec![0; 64];

		let first_chunk = device.read_timeout(&mut buf, 300_000)?;
		if first_chunk < 9 || buf[0] != '?' as u8 || buf[1] != '#' as u8 || buf[2] != '#' as u8 {
			return Err(protocol_err);
		}
		let msg_type = MessageType::from_i32(((buf[3] as i32 & 0xFF) << 8) + (buf[4] as i32 & 0xFF)).ok_or(protocol_err)?;
		let msg_size = ((buf[5] as u32 & 0xFF) << 24) + ((buf[6] as u32 & 0xFF) << 16) + ((buf[7] as u32 & 0xFF) << 8) + (buf[8] as u32 & 0xFF);
		let mut data = Vec::new();
		data.extend_from_slice(&buf[9..]);
		while data.len() < (msg_size as usize) {
			device.read_timeout(&mut buf, 10_000)?;
			data.extend_from_slice(&buf[1..]);
		}
		Ok((msg_type, data[..msg_size as usize].to_vec()))
	}
}

impl <'a>Wallet<'a> for Manager {
	type Error = Error;
	type Transaction = &'a TransactionInfo;

	fn sign_transaction(&self, address: &Address, t_info: Self::Transaction) ->
		Result<Signature, Error> {
		let usb = self.usb.lock();
		let devices = self.devices.read();
		let device = devices.iter().find(|d| &d.info.address == address).ok_or(Error::KeyNotFound)?;
		let handle = self.open_path(|| usb.open_path(&device.path))?;
		let msg_type = MessageType::MessageType_EthereumSignTx;
		let mut message = EthereumSignTx::new();
		match *self.key_path.read() {
			KeyPath::Ethereum => message.set_address_n(ETH_DERIVATION_PATH.to_vec()),
			KeyPath::EthereumClassic => message.set_address_n(ETC_DERIVATION_PATH.to_vec()),
		}
		message.set_nonce(self.u256_to_be_vec(&t_info.nonce));
		message.set_gas_limit(self.u256_to_be_vec(&t_info.gas_limit));
		message.set_gas_price(self.u256_to_be_vec(&t_info.gas_price));
		message.set_value(self.u256_to_be_vec(&t_info.value));

		match t_info.to {
			Some(addr) => {
				message.set_to(addr.to_vec())
			}
			None => (),
		}
		let first_chunk_length = min(t_info.data.len(), 1024);
		let chunk = &t_info.data[0..first_chunk_length];
		message.set_data_initial_chunk(chunk.to_vec());
		message.set_data_length(t_info.data.len() as u32);
		if let Some(c_id) = t_info.chain_id {
			message.set_chain_id(c_id as u32);
		}

		self.send_device_message(&handle, &msg_type, &message)?;

		self.signing_loop(&handle, &t_info.chain_id, &t_info.data[first_chunk_length..])
	}

	fn set_key_path(&self, key_path: KeyPath) {
		*self.key_path.write() = key_path;
	}

	fn update_devices(&self) -> Result<usize, Error> {
		let mut usb = self.usb.lock();
		usb.refresh_devices();
		let devices = usb.devices();
		let mut new_devices = Vec::new();
		let mut locked_devices = Vec::new();
		let mut error = None;
		for usb_device in devices {
			let is_trezor = usb_device.vendor_id == TREZOR_VID;
			let is_supported_product = TREZOR_PIDS.contains(&usb_device.product_id);
			let is_valid = usb_device.usage_page == 0xFF00 || usb_device.interface_number == 0;

			trace!(
				"Checking device: {:?}, trezor: {:?}, prod: {:?}, valid: {:?}",
				usb_device,
				is_trezor,
				is_supported_product,
				is_valid,
			);
			if !is_trezor || !is_supported_product || !is_valid {
				continue;
			}
			match self.read_device(&usb, &usb_device) {
				Ok(device) => new_devices.push(device),
				Err(Error::LockedDevice(path)) => locked_devices.push(path.to_string()),
				Err(e) => {
					warn!("Error reading device: {:?}", e);
					error = Some(e);
				}
			}
		}
		let count = new_devices.len();
		trace!("Got devices: {:?}, closed: {:?}", new_devices, locked_devices);
		*self.devices.write() = new_devices;
		*self.locked_devices.write() = locked_devices;
		match error {
			Some(e) => Err(e),
			None => Ok(count),
		}
	}

	fn read_device(&self, usb: &hidapi::HidApi, dev_info: &hidapi::HidDeviceInfo) -> Result<Device, Error> {
		let handle = self.open_path(|| usb.open_path(&dev_info.path))?;
		let manufacturer = dev_info.manufacturer_string.clone().unwrap_or("Unknown".to_owned());
		let name = dev_info.product_string.clone().unwrap_or("Unknown".to_owned());
		let serial = dev_info.serial_number.clone().unwrap_or("Unknown".to_owned());
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
			Ok(None) => Err(Error::LockedDevice(dev_info.path.clone())),
			Err(e) => Err(e),
		}
	}

	fn list_devices(&self) -> Vec<WalletInfo> {
		self.devices.read().iter().map(|d| d.info.clone()).collect()
	}

	fn list_locked_devices(&self) -> Vec<String> {
		(*self.locked_devices.read()).clone()
	}

	fn get_wallet(&self, address: &Address) -> Option<WalletInfo> {
		self.devices.read().iter().find(|d| &d.info.address == address).map(|d| d.info.clone())
	}

	fn get_address(&self, device: &hidapi::HidDevice) -> Result<Option<Address>, Error> {
		let typ = MessageType::MessageType_EthereumGetAddress;
		let mut message = EthereumGetAddress::new();
		match *self.key_path.read() {
			KeyPath::Ethereum => message.set_address_n(ETH_DERIVATION_PATH.to_vec()),
			KeyPath::EthereumClassic => message.set_address_n(ETC_DERIVATION_PATH.to_vec()),
		}
		message.set_show_display(false);
		self.send_device_message(&device, &typ, &message)?;

		let (resp_type, bytes) = self.read_device_response(&device)?;
		match resp_type {
			MessageType::MessageType_EthereumAddress => {
				let response: EthereumAddress = protobuf::core::parse_from_bytes(&bytes)?;
				Ok(Some(From::from(response.get_address())))
			}
			_ => Ok(None),
		}
	}

	fn open_path<R, F>(&self, f: F) -> Result<R, Error>
		where F: Fn() -> Result<R, &'static str>
	{
		f().map_err(Into::into)
	}
}

// Try to connect to the device using polling in at most the time specified by the `timeout`
fn try_connect_polling(trezor: Arc<Manager>, duration: Duration) -> bool {
	let start_time = Instant::now();
	while start_time.elapsed() <= duration {
		if let Ok(_) = trezor.update_devices() {
			return true
		}
	}
	false
}

/// Trezor event handler
/// A separate thread is handeling incoming events
///
/// Note, that this run to completion and race-conditions can't occur but this can
/// therefore starve other events for being process with a spinlock or similar
struct EventHandler {
	trezor: Weak<Manager>,
}

impl EventHandler {
	/// Trezor event handler constructor
	pub fn new(trezor: Weak<Manager>) -> Self {
		Self { trezor: trezor }
	}
}

impl libusb::Hotplug for EventHandler {
	fn device_arrived(&mut self, _device: libusb::Device) {
		debug!(target: "hw", "Trezor V1 arrived");
		if let Some(trezor) = self.trezor.upgrade() {
			if try_connect_polling(trezor, Duration::from_millis(500)) != true {
				debug!(target: "hw", "Ledger connect timeout");
			}
		}
	}

	fn device_left(&mut self, _device: libusb::Device) {
		debug!(target: "hw", "Trezor V1 left");
		if let Some(trezor) = self.trezor.upgrade() {
			if try_connect_polling(trezor, Duration::from_millis(500)) != true {
				debug!(target: "hw", "Ledger disconnect timeout");
			}
		}
	}
}

#[test]
#[ignore]
/// This test can't be run without an actual trezor device connected
/// (and unlocked) attached to the machine that's running the test
fn test_signature() {
	use ethereum_types::{H160, H256, U256};

	let hidapi = Arc::new(Mutex::new(hidapi::HidApi::new().unwrap()));
	let manager = Manager::new(hidapi.clone(), Arc::new(AtomicBool::new(false))).unwrap();
	let addr: Address = H160::from("some_addr");

	assert_eq!(try_connect_polling(manager.clone(), Duration::from_millis(500)), true);

	let t_info = TransactionInfo {
		nonce: U256::from(1),
		gas_price: U256::from(100),
		gas_limit: U256::from(21_000),
		to: Some(H160::from("some_other_addr")),
		chain_id: Some(17),
		value: U256::from(1_000_000),
		data: (&[1u8; 3000]).to_vec(),
	};
	let signature = manager.sign_transaction(&addr, &t_info).unwrap();
	let expected = Signature::from_rsv(
		&H256::from("device_specific_r"),
		&H256::from("device_specific_s"),
		0x01
		);

	assert_eq!(signature, expected)
}
