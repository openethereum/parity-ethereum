#![allow(dead_code)]

mod protobuf;

extern crate libusb;
extern crate hidapi;
extern crate quick_protobuf;
extern crate byteorder;
use keepkey::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use std::io::{self, BufRead};
use std::mem::transmute;
use keepkey::quick_protobuf::{BytesReader, Writer};
use self::protobuf::{
    Address, ButtonAck, ButtonRequest, Failure, Features,
    GetAddress, GetPublicKey, Initialize, Ping, PinMatrixAck,
    PinMatrixRequest, PublicKey, Success,
};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::io::Cursor;

/// Hardware waller error.
#[derive(Debug)]
pub enum Error {
	/// Keepkey device error.
	KeepkeyDevice,
	/// USB error.
	Usb(libusb::Error),
	/// Hardware wallet not found for specified key.
	KeyNotFound,
}

#[derive(Debug)]
pub enum MessageType {
    Initialize = 0,
    Ping = 1,
    Success = 2,
    Failure = 3,
    ChangePin = 4,
    WipeDevice = 5,
    FirmwareErase = 6,
    FirmwareUpload = 7,
    GetEntropy = 9,
    Entropy = 10,
    GetPublicKey = 11,
    PublicKey = 12,
    LoadDevice = 13,
    ResetDevice = 14,
    SignTx = 15,
    SimpleSignTx = 16,
    Features = 17,
    PinMatrixRequest = 18,
    PinMatrixAck = 19,
    Cancel = 20,
    TxRequest = 21,
    TxAck = 22,
    CipherKeyValue = 23,
    ClearSession = 24,
    ApplySettings = 25,
    ButtonRequest = 26,
    ButtonAck = 27,
    GetAddress = 29,
    Address = 30,
    EntropyRequest = 35,
    EntropyAck = 36,
    SignMessage = 38,
    VerifyMessage = 39,
    MessageSignature = 40,
    PassphraseRequest = 41,
    PassphraseAck = 42,
    EstimateTxSize = 43,
    TxSize = 44,
    RecoveryDevice = 45,
    WordRequest = 46,
    WordAck = 47,
    CipheredKeyValue = 48,
    EncryptMessage = 49,
    EncryptedMessage = 50,
    DecryptMessage = 51,
    DecryptedMessage = 52,
    SignIdentity = 53,
    SignedIdentity = 54,
    GetFeatures = 55,
    CharacterRequest = 80,
    CharacterAck = 81,
    DebugLinkDecision = 100,
    DebugLinkGetState = 101,
    DebugLinkState = 102,
    DebugLinkStop = 103,
    DebugLinkLog = 104,
    DebugLinkFillConfig = 105,
}

impl MessageType {
    fn get_type(value: u16) -> MessageType {
        unsafe { transmute(value as u8) }
    }
}

/// Keepkey device manager.
pub struct Manager {
    api:     Arc<Mutex<hidapi::HidApi>>,
	devices: Vec<Device>,
}
impl Manager {
    pub fn new() -> Manager {
        let api  = Arc::new(Mutex::new(hidapi::HidApi::new().unwrap()));
        let mut list = Vec::new();
        // get list of connected devices. If there is a keepkey, add it.
        for device in &api.lock().unwrap().devices() {
            if device.vendor_id == 11044 && device.product_id == 1 {
                list.push(device.path.clone());
            }
        }
        Manager {
            api: api.clone(),
            devices: list.iter().map(|path| Device::new(api.clone(), path.to_string())).collect()
        }
    }

    pub fn update_devices(&mut self) -> Result<usize, Error> {
        let api  = self.api.lock().unwrap();
        let mut num_new_devices = 0;
        // get list of connected devices. If there is a keepkey, add it to the list.
        for device in &api.lock().unwrap().devices() {
            if device.vendor_id == 11044
                && device.product_id == 1
                && !devices.contains() // device doesn't already exist doesn't exist
            {
                self.devices.push(Device::new(api.clone(), device.path.clone().to_string()));
                num_new_devices += 1;
            }
        }
        Ok(num_new_devices)
    }
}

/// Keepkey device
struct Device {
    api:        Arc<Mutex<hidapi::HidApi>>,
    address:    Option<String>,
    info:       Option<Info>,
    path:       String,
}
impl Device {
    pub fn new(api: Arc<Mutex<hidapi::HidApi>>, path: String) -> Device {
        let mut device = Device {
            api:        api,
            address:    None,
            info:       None,
            path:       path.clone(),
        };
        device.initialize();
        device
    }

    fn initialize(&mut self) {
        // Send a hello to device
        self.write(MessageType::Initialize as u16, &[]);
        // Get Address
        self.get_address();
    }

    fn get_address(&mut self) {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf);
            let get_address = GetAddress {
                address_n: Some(0),
                coin_name: Some(Cow::from("Ethereum")),
                show_display: Some(false),
                multisig: None,
            };
            writer.write_message(&get_address).expect("Cannot write get_address");
        }
        self.write(MessageType::GetAddress as u16, &buf);
    }

    fn pin_matrix_ack(&mut self, pin: &str) {
        println!("PIN_MATRIX_ACK");
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf);
            let pin_matrix_ack = PinMatrixAck {
                pin: Cow::from(pin),
            };
            writer.write_message(&pin_matrix_ack).expect("Cannot write pin_matrix_ack");
        }
        self.write(MessageType::PinMatrixAck as u16, &buf);
    }

    fn write(&mut self, msg_type: u16, buf: &[u8]) {
        // write data to device
        let mut msg = vec![63, 35, 35];
        msg.write_u16::<BigEndian>(msg_type).unwrap();
        msg.extend_from_slice(&[0, 0, 0]);
        msg.extend_from_slice(&buf[..]);
        self._write(&mut msg);
    }

    fn _write(&mut self, msg_in: &[u8]) {
        // Read data from device
        let mut msg      = vec![];
        let mut buf      = [0u8; 64];
        let msg_type: u16;
        {
            let api    = self.api.lock().unwrap();
            let handle = api.open_path(&self.path).unwrap();
            handle.write(&msg_in).unwrap();
            handle.read(&mut buf[..]).unwrap();
            msg_type        = Cursor::new(&buf[3..5]).read_u16::<BigEndian>().unwrap();
            let mut msg_len = Cursor::new(&buf[7..9]).read_i16::<BigEndian>().unwrap();
            msg.extend_from_slice(&buf[9..]);

            // TODO: Manage incoming packets that do not have the proper start code
            // if !&buf.starts_with(63) {
            //
            // }
            msg_len -= msg.len() as i16;

            while msg_len > 0 {
                handle.read(&mut buf[..]).unwrap();
                msg_len -= buf.len() as i16;
                msg.extend_from_slice(&buf[1..]);
            }
            let l = (msg.len() as i16 + msg_len + 10) as usize;
            msg = (&msg[..l]).to_vec();
        }
        self.parse_msg(msg_type, msg);
    }

    fn parse_msg(&mut self, msg_type: u16, msg: Vec<u8>) {
        let mut reader = BytesReader::from_bytes(&msg);

        match MessageType::get_type(msg_type) {
            MessageType::Ping => {
                // Ping
                let ping = Ping::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
            }
            MessageType::Failure => {
                // Failure
                let failure = Failure::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
                println!("Failure: {:?}", failure);
            }
            MessageType::Features => {
                // Features
                let features = Features::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
                let ref coin = features.coins.into_iter().filter(|&ref x| x.clone().coin_name.unwrap() == "Ethereum").collect::<Vec<_>>()[0];
                self.info = Some(Info {
                    major_version: features.major_version.clone(),
                    minor_version: features.minor_version.clone(),
                    patch_version: features.patch_version.clone(),
                    device_id: features.device_id.unwrap().to_mut().clone(),
                    pin_protection: features.pin_protection.clone(),
                    passphrase_protection: features.passphrase_protection.clone(),
                    language: features.language.unwrap().to_mut().clone(),
                    label: features.label.unwrap().to_mut().clone(),
                    coin: Some(Coin {
                        coin_name: Some(coin.clone().coin_name.unwrap().to_mut().clone()),
                        coin_shortcut: Some(coin.clone().coin_name.unwrap().to_mut().clone()),
                        address_type: coin.address_type,
                        maxfee_kb: coin.maxfee_kb,
                        address_type_p2sh: coin.address_type_p2sh,
                    }),
                    initialized: features.initialized.clone(),
                    pin_cached: features.pin_cached.clone(),
                    passphrase_cached: features.passphrase_cached.clone(),
                });
                println!("self.info: {:?}", self.info);
            }
            MessageType::PinMatrixRequest => {
                // Pin Matrix Request
                println!("Please enter your pin: ");
                let mut line = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut line).expect("Could not read line");
                line.pop(); // delete new line character
                self.pin_matrix_ack(&line);
            }
            MessageType::ButtonRequest => {
                // Button Request
                println!("Button Request");
                // let the device know we got the request and send an ack (code 27)
                self.write(MessageType::ButtonAck as u16, &[]);
            }
            MessageType::Address => {
                // Address
                println!("ADDRESS!!!!!");
                let mut address = Address::from_reader(&mut reader, &msg).expect("Cannot read Message -_-");
                self.address = Some(address.address.to_mut().clone());
                println!("ADDRESS: {:?}", self.address);
            }
            _ => {
                // Messages we don't care about
                println!("MESSAGE: {:?}", msg_type);
            }
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
struct Coin {
    coin_name: Option<String>,
    coin_shortcut: Option<String>,
    address_type: u32,
    maxfee_kb: Option<u64>,
    address_type_p2sh: u32,
}
