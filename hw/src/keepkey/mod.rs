#![allow(dead_code)]

mod protobuf;

extern crate libusb;
extern crate quick_protobuf;
extern crate byteorder;
use super::TransactionInfo;
use keepkey::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use keepkey::quick_protobuf::{BytesReader, Writer};

use std::fmt;
use fmt::Write;
use std::io::{Cursor};
use std::mem::transmute;
use std::borrow::Cow;
use std::sync::{Arc};
use std::time::Duration;
use parking_lot::Mutex;
use serde_json;
use hidapi;
use ethkey::{Address, Signature};
use bigint::hash::H256;
use self::protobuf::{
    EthereumAddress, EthereumGetAddress, EthereumSignTx, EthereumTxRequest,
    EthereumTxAck, Failure, Features, PinMatrixAck, Success,
};

/// Hardware waller error.
#[derive(Debug)]
pub enum Error {
	/// Keepkey device error.
	KeepkeyDevice,
	/// USB error.
	Usb(hidapi::HidError),
	/// Hardware wallet not found for specified key.
	PathNotFound,
    /// Message is unusable
    BadMessageType,
    /// Keepkey is responding with gibberish
    BadStartCode,
    /// Serde Derive Error
    SerdeError(serde_json::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::KeepkeyDevice => write!(f, "Keepkey protocol error"),
			Error::Usb(ref e) => write!(f, "USB communication error: {}", e),
			Error::PathNotFound => write!(f, "Path not found"),
            Error::BadMessageType => write!(f, "Bad message type"),
            Error::BadStartCode => write!(f, "The device is not responding properly"),
			Error::SerdeError(ref s) => write!(f, "Serde derive failure: {}", s),
		}
	}
}

impl From<hidapi::HidError> for Error {
	fn from(err: hidapi::HidError) -> Error {
		Error::Usb(err)
	}
}

#[derive(Debug)]
pub enum MessageType {
    Initialize = 0,
    Success = 2,
    Failure = 3,
    // PublicKey = 12,
    Features = 17,
    PinMatrixRequest = 18,
    PinMatrixAck = 19,
    Cancel = 20,
    ButtonRequest = 26,
    ButtonAck = 27,
    // PassphraseRequest = 41,
    // PassphraseAck = 42,
    EthereumGetAddress = 56,
    EthereumAddress = 57,
    EthereumSignTx = 58,
    EthereumTxRequest = 59,
}

impl MessageType {
    fn get_type(value: u16) -> MessageType {
        unsafe { transmute(value as u8) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Info {
    major_version:         Option<u32>,
    minor_version:         Option<u32>,
    patch_version:         Option<u32>,
    device_id:             String,
    pin_protection:        Option<bool>,
    passphrase_protection: Option<bool>,
    language:              String,
    label:                 String,
    coin:                  Option<Coin>,
    initialized:           Option<bool>,
    pin_cached:            Option<bool>,
    passphrase_cached:     Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceDetails {
    device_path: String,
    device_info: Option<Info>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Coin {
    coin_name: Option<String>,
    coin_shortcut: Option<String>,
    address_type: u32,
    maxfee_kb: Option<u64>,
    address_type_p2sh: u32,
}

/// Keepkey device manager.
pub struct Manager {
    api:     Arc<Mutex<hidapi::HidApi>>,
	devices: Vec<Device>,
}
impl Manager {
    /// Create a new instance
    pub fn new(hidapi: Arc<Mutex<hidapi::HidApi>>) -> Manager {
        println!("NEW KEEPKEY MANAGER");
        Manager {
            api: hidapi,
            devices: Vec::new(),
        }
    }

    /// Called when looking for new devices or removing dead ones
    pub fn update_devices(&mut self) -> Result<usize, Error> {
        println!("UPDATE DEVICES KEEPKEY");
        let mut num_new_devices = 0;
        let mut path = Vec::new();
        {
            let mut api = self.api.lock();
            api.refresh_devices();
            // get list of connected devices. If there is a keepkey, and we dont already own it, add it to the list.
            for device in api.devices() {
                if device.vendor_id == 11044 && device.product_id == 1 {
                    path.push(device.path.clone());
                    num_new_devices += 1;
                }
            }
        }
        let new_devices = path.iter().map(|p| Device::new(self.api.clone(), p.to_string())).collect();
        self.devices = new_devices;
        Ok(num_new_devices)
    }

    /// Get the device path from address
    pub fn path_from_address(&self, address: &Address) -> Option<String> {
        self.devices.iter().find(|d| d.address.clone().unwrap() == address.as_ref()).map(|d| d.path.clone())
    }

    /// Sign the transaction given an address and
    pub fn sign_transaction(&self, address: &Address, t: TransactionInfo) -> Result<Signature, Error> {
        let tx = serde_json::to_string(&t).map_err(Error::SerdeError).unwrap();
        let path = self.path_from_address(address);
        let res: Vec<u8> = self._message(path, |mut d| d.sign_transaction(&tx)).unwrap()
            .split("").map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect();
        let v = (&res[0] + 1) % 2;
        let r = H256::from_slice(&res[1..33]);
		let s = H256::from_slice(&res[33..65]);
        Ok(Signature::from_rsv(&r, &s, v))
    }

    pub fn message(&self, message_type: String, path: Option<String>, message: Option<String>) -> Result<String, Error> {
        println!("message_type: {}", message_type);
        match message_type.as_ref() {
            "init" => {
                let devices: Vec<DeviceDetails> = self.devices.iter()
                    .map(|device| DeviceDetails {
                        device_info: device.info.clone(),
                        device_path: device.path.clone()
                    }).collect();
                match serde_json::to_string(&devices) {
                    Ok(t) => Ok(t),
                    Err(e) => Err(Error::SerdeError(e)),
                }
            }
            "cancel" => {
                self._message(path, |mut d| d.cancel())
            }
            "get_address" => {
                self._message(path, |mut d| d.get_address())
            }
            "pin_matrix_ack" => {
                let message = match message {
                    None => return Err(Error::BadMessageType),
                    Some(m) => m,
                };
                self._message(path, |mut d| d.pin_matrix_ack(message.as_ref()))
            }
            _ => {
                Err(Error::BadMessageType)
            }
        }
    }

    fn _message<F>(&self, path: Option<String>, f: F) -> Result<String, Error>
    where F: Fn(Device) -> Result<String, Error> {
        for device in self.devices.clone() {
            if device.path == path.clone().unwrap_or("".to_owned()) {
                return f(device);
            }
        }
        Err(Error::PathNotFound)
    }
}

/// Keepkey device
#[derive(Clone)]
struct Device {
    api:     Arc<Mutex<hidapi::HidApi>>,
    address: Option<Vec<u8>>,
    info:    Option<Info>,
    path:    String,
}
impl Device {
    pub fn new(api: Arc<Mutex<hidapi::HidApi>>, path: String) -> Device {
        println!("NEW Device");
        let mut device = Device {
            api:     api,
            address: None,
            info:    None,
            path:    path.clone(),
        };
        device.initialize();
        device
    }

    fn cancel(&mut self) -> Result<String, Error> {
        println!("CANCEL");
        self.write(MessageType::Cancel as u16, &[])
    }

    fn initialize(&mut self) {
        // Send a hello to device
        println!("INIT");
        self.write(MessageType::Initialize as u16, &[]).unwrap();
    }

    fn get_address(&mut self) -> Result<String, Error> {
        println!("GET_ADDRESS");
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf);
            let get_address = EthereumGetAddress {
                address_n: vec![0],
                show_display: Some(false),
            };
            writer.write_message(&get_address).unwrap();
        }
        self.write(MessageType::EthereumGetAddress as u16, &buf)
    }

    fn pin_matrix_ack(&mut self, pin: &str) -> Result<String, Error> {
        println!("PIN_MATRIX_ACK");
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf);
            let pin_matrix_ack = PinMatrixAck {
                pin: Cow::from(pin),
            };
            writer.write_message(&pin_matrix_ack).unwrap();
        }
        self.write(MessageType::PinMatrixAck as u16, &buf)
    }

    fn sign_tx(&mut self) {
        println!("SIGN_TX");
        let mut buf = Vec::new();
        // if data, we need to send data in 1024bit chunks
        {
            let mut writer = Writer::new(&mut buf);
            let sign_tx = EthereumSignTx {
                address_n: vec![0],
                nonce: Some(Cow::from(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
                gas_price: Some(Cow::from(vec![0, 0, 0, 5, 210, 29, 186, 0])),
                gas_limit: Some(Cow::from(vec![0, 0, 0, 23, 72, 118, 232, 0])),
                to: Some(Cow::from(vec![95, 89, 79, 26, 128, 97, 158, 107, 167, 231, 0, 62, 153, 208, 33, 157, 49, 44, 157, 44])),
                value: Some(Cow::from(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 224, 182, 179, 167, 100, 0, 0])),
                data_initial_chunk: None,
                data_length: None,
                to_address_n: vec![0],
                address_type: Some(OutputAddressType::SPEND),
                exchange_type: None,
                chain_id: Some(1),
            };
            writer.write_message(&sign_tx).expect("Cannot write sign_tx");
        }
        self.write(MessageType::EthereumSignTx as u16, &buf);
        self.ethereum_tx_ack(None);
    }

    fn ethereum_tx_ack(&mut self, data: Option<Vec<u8>>) {
        println!("PIN_MATRIX_ACK");
        let mut buf = Vec::new();
        let data_chunk = match data {
            Some(ref d) => Some(Cow::from(&d[..])),
            None => None
        };
        {
            let mut writer = Writer::new(&mut buf);
            let ethereum_tx_ack = EthereumTxAck {
                data_chunk,
            };
            writer.write_message(&ethereum_tx_ack).expect("Cannot write pin_matrix_ack");
        }
        self.write(MessageType::EthereumTxAck as u16, &buf);
    }

    fn write(&mut self, msg_type: u16, buf: &[u8]) -> Result<String, Error> {
        // write data to device
        let mut msg = vec![35, 35];
        msg.write_u16::<BigEndian>(msg_type).unwrap();
        msg.extend_from_slice(&[0, 0, 0]);
        msg.extend_from_slice(&buf[..]);
        self._write(&mut msg)
    }

    fn open_path<R, F>(&self, f: F) -> Result<R, Error>
    where F: Fn() -> Result<R, &'static str> {
    	let mut err = Error::PathNotFound;
    	/// Try to open device a few times.
    	for _ in 0..10 {
    		match f() {
    			Ok(h) => return Ok(h),
    			Err(e) => err = From::from(e),
    		}
    		::std::thread::sleep(Duration::from_millis(200));
    	}
        Err(err)
    }

    fn _write(&mut self, msg_in: &[u8]) -> Result<String, Error> {
        // Read data from device
        let mut msg = vec![];
        let mut buf = [0u8; 64];
        let msg_type: u16;
        {
            let api = self.api.lock();
            let handle = self.open_path(|| api.open_path(&self.path))?;
            for chunk in msg_in.chunks(63) {
                let mut msg_chunk = vec![63];
                msg_chunk.extend_from_slice(&chunk);
                handle.write(&msg_chunk).unwrap();
            }
            handle.read(&mut buf[..]).unwrap();
            msg_type        = Cursor::new(&buf[3..5]).read_u16::<BigEndian>().unwrap();
            let mut msg_len = Cursor::new(&buf[7..9]).read_i16::<BigEndian>().unwrap();
            let l           = msg_len as usize;
            msg.extend_from_slice(&buf[9..]);

            // Manage incoming packets that do not have the proper start code
            if !&buf.starts_with(&[63]) {
                return Err(Error::BadStartCode);
            }
            msg_len -= msg.len() as i16;
            while msg_len > 0 {
                handle.read(&mut buf[..]).unwrap();
                msg_len -= buf.len() as i16;
                msg.extend_from_slice(&buf[1..]);
            }
            msg = (&msg[..l]).to_vec();
        }
        self.parse_msg(msg_type, msg)
    }

    fn parse_msg(&mut self, msg_type: u16, msg: Vec<u8>) -> Result<String, Error> {
        let mut reader = BytesReader::from_bytes(&msg);

        match MessageType::get_type(msg_type) {
            MessageType::Success => {
                // Success
                let success = Success::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
                serde_json::to_string(&success).map_err(Error::SerdeError)
            }
            MessageType::Failure => {
                // Failure
                let failure = Failure::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
                println!("Failure: {:?}", failure);
                serde_json::to_string(&failure).map_err(Error::SerdeError)
            }
            MessageType::Features => {
                // Features
                println!("FEATURES");
                let features = Features::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
                let ref coin = features.coins.into_iter().filter(|&ref x| x.clone().coin_name.unwrap() == "Ethereum").collect::<Vec<_>>()[0];
                self.info = Some(Info {
                    major_version: features.major_version,
                    minor_version: features.minor_version,
                    patch_version: features.patch_version,
                    device_id: features.device_id.unwrap().to_mut().clone(),
                    pin_protection: features.pin_protection,
                    passphrase_protection: features.passphrase_protection,
                    language: features.language.unwrap().to_mut().clone(),
                    label: features.label.unwrap().to_mut().clone(),
                    coin: Some(Coin {
                        coin_name: Some(coin.clone().coin_name.unwrap().to_mut().clone()),
                        coin_shortcut: Some(coin.clone().coin_name.unwrap().to_mut().clone()),
                        address_type: coin.address_type,
                        maxfee_kb: coin.maxfee_kb,
                        address_type_p2sh: coin.address_type_p2sh,
                    }),
                    initialized: features.initialized,
                    pin_cached: features.pin_cached,
                    passphrase_cached: features.passphrase_cached,
                });
                println!("self.info: {:?}", self.info);
                serde_json::to_string(&self.info).map_err(Error::SerdeError)
            }
            MessageType::PinMatrixRequest => {
                // Pin Matrix Request
                println!("Please enter your pin...");
                Ok("pin_matrix_request".to_string())
            }
            MessageType::ButtonRequest => {
                // Button Request
                println!("Button Request");
                // let the device know we got the request and send an ack (code 27)
                self.write(MessageType::ButtonAck as u16, &[])
            }
            MessageType::EthereumAddress => {
                // Address
                let mut address = EthereumAddress::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
                self.address = Some(address.address.to_mut().clone());
                let mut s = String::new();
                for byte in &self.address.clone().unwrap()[..] {
                    write!(&mut s, "{:02x}", byte).expect("whatever");
                }
                println!("ADDRESS: {:?}", s);
                Ok(s)
            }
            MessageType::EthereumTxRequest => {
                // Transaction Signing Responce
                let request = EthereumTxRequest::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
                println!("REQUEST: {:?}", request);
                Ok("res".to_string())
            }
            _ => {
                // Messages we don't care about
                println!("MESSAGE: {:?}", msg_type);
                Err(Error::BadMessageType)
            }
        }
    }
}
