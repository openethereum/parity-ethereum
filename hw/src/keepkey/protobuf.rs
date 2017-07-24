//! Automatically generated rust module for 'messages.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::io::Write;
use std::borrow::Cow;
use keepkey::quick_protobuf::{MessageWrite, BytesReader, Writer, Result};
use keepkey::quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Initialize { }

impl Initialize {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for Initialize { }

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Ping<'a> {
    pub message: Option<Cow<'a, str>>,
    pub button_protection: Option<bool>,
    pub pin_protection: Option<bool>,
    pub passphrase_protection: Option<bool>,
}

impl<'a> Ping<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(16) => msg.button_protection = Some(r.read_bool(bytes)?),
                Ok(24) => msg.pin_protection = Some(r.read_bool(bytes)?),
                Ok(32) => msg.passphrase_protection = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Ping<'a> {
    fn get_size(&self) -> usize {
        0
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.button_protection.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.pin_protection.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.passphrase_protection.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.message { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.button_protection { w.write_with_tag(16, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.pin_protection { w.write_with_tag(24, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.passphrase_protection { w.write_with_tag(32, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Features<'a> {
    pub vendor: Option<Cow<'a, str>>,
    pub major_version: Option<u32>,
    pub minor_version: Option<u32>,
    pub patch_version: Option<u32>,
    pub bootloader_mode: Option<bool>,
    pub device_id: Option<Cow<'a, str>>,
    pub pin_protection: Option<bool>,
    pub passphrase_protection: Option<bool>,
    pub language: Option<Cow<'a, str>>,
    pub label: Option<Cow<'a, str>>,
    pub coins: Vec<CoinType<'a>>,
    pub initialized: Option<bool>,
    pub revision: Option<Cow<'a, [u8]>>,
    pub bootloader_hash: Option<Cow<'a, [u8]>>,
    pub imported: Option<bool>,
    pub pin_cached: Option<bool>,
    pub passphrase_cached: Option<bool>,
}

impl<'a> Features<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.vendor = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(16) => msg.major_version = Some(r.read_uint32(bytes)?),
                Ok(24) => msg.minor_version = Some(r.read_uint32(bytes)?),
                Ok(32) => msg.patch_version = Some(r.read_uint32(bytes)?),
                Ok(40) => msg.bootloader_mode = Some(r.read_bool(bytes)?),
                Ok(50) => msg.device_id = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(56) => msg.pin_protection = Some(r.read_bool(bytes)?),
                Ok(64) => msg.passphrase_protection = Some(r.read_bool(bytes)?),
                Ok(74) => msg.language = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(82) => msg.label = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(90) => msg.coins.push(r.read_message(bytes, CoinType::from_reader)?),
                Ok(96) => msg.initialized = Some(r.read_bool(bytes)?),
                Ok(106) => msg.revision = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(114) => msg.bootloader_hash = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(120) => msg.imported = Some(r.read_bool(bytes)?),
                Ok(128) => msg.pin_cached = Some(r.read_bool(bytes)?),
                Ok(136) => msg.passphrase_cached = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Features<'a> {
    fn get_size(&self) -> usize {
        0
        + self.vendor.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.major_version.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.minor_version.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.patch_version.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.bootloader_mode.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.device_id.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.pin_protection.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.passphrase_protection.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.language.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.label.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.coins.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.initialized.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.revision.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.bootloader_hash.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.imported.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.pin_cached.as_ref().map_or(0, |m| 2 + sizeof_varint(*(m) as u64))
        + self.passphrase_cached.as_ref().map_or(0, |m| 2 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.vendor { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.major_version { w.write_with_tag(16, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.minor_version { w.write_with_tag(24, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.patch_version { w.write_with_tag(32, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.bootloader_mode { w.write_with_tag(40, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.device_id { w.write_with_tag(50, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.pin_protection { w.write_with_tag(56, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.passphrase_protection { w.write_with_tag(64, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.language { w.write_with_tag(74, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.label { w.write_with_tag(82, |w| w.write_string(&**s))?; }
        for s in &self.coins { w.write_with_tag(90, |w| w.write_message(s))?; }
        if let Some(ref s) = self.initialized { w.write_with_tag(96, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.revision { w.write_with_tag(106, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.bootloader_hash { w.write_with_tag(114, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.imported { w.write_with_tag(120, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.pin_cached { w.write_with_tag(128, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.passphrase_cached { w.write_with_tag(136, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CoinType<'a> {
    pub coin_name: Option<Cow<'a, str>>,
    pub coin_shortcut: Option<Cow<'a, str>>,
    pub address_type: u32,
    pub maxfee_kb: Option<u64>,
    pub address_type_p2sh: u32,
}

impl<'a> CoinType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = CoinType {
            address_type_p2sh: 5u32,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.coin_name = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.coin_shortcut = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.address_type = r.read_uint32(bytes)?,
                Ok(32) => msg.maxfee_kb = Some(r.read_uint64(bytes)?),
                Ok(40) => msg.address_type_p2sh = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for CoinType<'a> {
    fn get_size(&self) -> usize {
        0
        + self.coin_name.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.coin_shortcut.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + if self.address_type == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.address_type) as u64) }
        + self.maxfee_kb.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + if self.address_type_p2sh == 5u32 { 0 } else { 1 + sizeof_varint(*(&self.address_type_p2sh) as u64) }
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.coin_name { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.coin_shortcut { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if self.address_type != 0u32 { w.write_with_tag(24, |w| w.write_uint32(*&self.address_type))?; }
        if let Some(ref s) = self.maxfee_kb { w.write_with_tag(32, |w| w.write_uint64(*s))?; }
        if self.address_type_p2sh != 5u32 { w.write_with_tag(40, |w| w.write_uint32(*&self.address_type_p2sh))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct GetAddress<'a> {
    pub address_n: Option<u32>,
    pub coin_name: Option<Cow<'a, str>>,
    pub show_display: Option<bool>,
    pub multisig: Option<MultisigRedeemScriptType<'a>>,
}

impl<'a> GetAddress<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.address_n = Some(r.read_uint32(bytes)?),
                Ok(18) => msg.coin_name = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.show_display = Some(r.read_bool(bytes)?),
                Ok(34) => msg.multisig = Some(r.read_message(bytes, MultisigRedeemScriptType::from_reader)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for GetAddress<'a> {
    fn get_size(&self) -> usize {
        0
        + self.address_n.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.coin_name.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.show_display.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.multisig.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.address_n { w.write_with_tag(8, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.coin_name { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.show_display { w.write_with_tag(24, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.multisig { w.write_with_tag(34, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct MultisigRedeemScriptType<'a> {
    pub pubkeys: Vec<HDNodePathType<'a>>,
    pub signatures: Vec<Cow<'a, [u8]>>,
    pub m: Option<u32>,
}

impl<'a> MultisigRedeemScriptType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.pubkeys.push(r.read_message(bytes, HDNodePathType::from_reader)?),
                Ok(18) => msg.signatures.push(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.m = Some(r.read_uint32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for MultisigRedeemScriptType<'a> {
    fn get_size(&self) -> usize {
        0
        + self.pubkeys.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.signatures.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
        + self.m.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.pubkeys { w.write_with_tag(10, |w| w.write_message(s))?; }
        for s in &self.signatures { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.m { w.write_with_tag(24, |w| w.write_uint32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct HDNodePathType<'a> {
    pub node: HDNodeType<'a>,
    pub address_n: Vec<u32>,
}

impl<'a> HDNodePathType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.node = r.read_message(bytes, HDNodeType::from_reader)?,
                Ok(16) => msg.address_n.push(r.read_uint32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for HDNodePathType<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.node).get_size())
        + self.address_n.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_message(&self.node))?;
        for s in &self.address_n { w.write_with_tag(16, |w| w.write_uint32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Address<'a> {
    pub address: Cow<'a, str>,
}

impl<'a> Address<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.address = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Address<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.address).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.address))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct GetPublicKey<'a> {
    pub address_n: Vec<u32>,
    pub ecdsa_curve_name: Option<Cow<'a, str>>,
    pub show_display: Option<bool>,
}

impl<'a> GetPublicKey<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.address_n.push(r.read_uint32(bytes)?),
                Ok(18) => msg.ecdsa_curve_name = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.show_display = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for GetPublicKey<'a> {
    fn get_size(&self) -> usize {
        0
        + self.address_n.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
        + self.ecdsa_curve_name.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.show_display.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.address_n { w.write_with_tag(8, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.ecdsa_curve_name { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.show_display { w.write_with_tag(24, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PublicKey<'a> {
    pub node: HDNodeType<'a>,
    pub xpub: Option<Cow<'a, str>>,
}

impl<'a> PublicKey<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.node = r.read_message(bytes, HDNodeType::from_reader)?,
                Ok(18) => msg.xpub = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PublicKey<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.node).get_size())
        + self.xpub.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_message(&self.node))?;
        if let Some(ref s) = self.xpub { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct HDNodeType<'a> {
    pub depth: u32,
    pub fingerprint: u32,
    pub child_num: u32,
    pub chain_code: Cow<'a, [u8]>,
    pub private_key: Option<Cow<'a, [u8]>>,
    pub public_key: Option<Cow<'a, [u8]>>,
}

impl<'a> HDNodeType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.depth = r.read_uint32(bytes)?,
                Ok(16) => msg.fingerprint = r.read_uint32(bytes)?,
                Ok(24) => msg.child_num = r.read_uint32(bytes)?,
                Ok(34) => msg.chain_code = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(42) => msg.private_key = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(50) => msg.public_key = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for HDNodeType<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.depth) as u64)
        + 1 + sizeof_varint(*(&self.fingerprint) as u64)
        + 1 + sizeof_varint(*(&self.child_num) as u64)
        + 1 + sizeof_len((&self.chain_code).len())
        + self.private_key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.public_key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.depth))?;
        w.write_with_tag(16, |w| w.write_uint32(*&self.fingerprint))?;
        w.write_with_tag(24, |w| w.write_uint32(*&self.child_num))?;
        w.write_with_tag(34, |w| w.write_bytes(&**&self.chain_code))?;
        if let Some(ref s) = self.private_key { w.write_with_tag(42, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.public_key { w.write_with_tag(50, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ButtonRequest<'a> {
    pub code: Option<mod_ButtonRequest::ButtonRequestType>,
    pub data: Option<Cow<'a, str>>,
}

impl<'a> ButtonRequest<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.code = Some(r.read_enum(bytes)?),
                Ok(18) => msg.data = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ButtonRequest<'a> {
    fn get_size(&self) -> usize {
        0
        + self.code.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.data.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.code { w.write_with_tag(8, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.data { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

pub mod mod_ButtonRequest {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ButtonRequestType {
    ButtonRequest_Other = 1,
    ButtonRequest_FeeOverThreshold = 2,
    ButtonRequest_ConfirmOutput = 3,
    ButtonRequest_ResetDevice = 4,
    ButtonRequest_ConfirmWord = 5,
    ButtonRequest_WipeDevice = 6,
    ButtonRequest_ProtectCall = 7,
    ButtonRequest_SignTx = 8,
    ButtonRequest_FirmwareCheck = 9,
    ButtonRequest_Address = 10,
    ButtonRequest_FirmwareErase = 11,
    ButtonRequest_ConfirmTransferToAccount = 12,
    ButtonRequest_ConfirmTransferToNodePath = 13,
}

impl Default for ButtonRequestType {
    fn default() -> Self {
        ButtonRequestType::ButtonRequest_Other
    }
}

impl From<i32> for ButtonRequestType {
    fn from(i: i32) -> Self {
        match i {
            1 => ButtonRequestType::ButtonRequest_Other,
            2 => ButtonRequestType::ButtonRequest_FeeOverThreshold,
            3 => ButtonRequestType::ButtonRequest_ConfirmOutput,
            4 => ButtonRequestType::ButtonRequest_ResetDevice,
            5 => ButtonRequestType::ButtonRequest_ConfirmWord,
            6 => ButtonRequestType::ButtonRequest_WipeDevice,
            7 => ButtonRequestType::ButtonRequest_ProtectCall,
            8 => ButtonRequestType::ButtonRequest_SignTx,
            9 => ButtonRequestType::ButtonRequest_FirmwareCheck,
            10 => ButtonRequestType::ButtonRequest_Address,
            11 => ButtonRequestType::ButtonRequest_FirmwareErase,
            12 => ButtonRequestType::ButtonRequest_ConfirmTransferToAccount,
            13 => ButtonRequestType::ButtonRequest_ConfirmTransferToNodePath,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ButtonAck { }

impl ButtonAck {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for ButtonAck { }

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PinMatrixRequest {
    pub type_pb: Option<mod_PinMatrixRequest::PinMatrixRequestType>,
}

impl PinMatrixRequest {
    pub fn from_reader(r: &mut BytesReader, bytes: &[u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = Some(r.read_enum(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for PinMatrixRequest {
    fn get_size(&self) -> usize {
        0
        + self.type_pb.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.type_pb { w.write_with_tag(8, |w| w.write_enum(*s as i32))?; }
        Ok(())
    }
}

pub mod mod_PinMatrixRequest {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PinMatrixRequestType {
    PinMatrixRequestType_Current = 1,
    PinMatrixRequestType_NewFirst = 2,
    PinMatrixRequestType_NewSecond = 3,
}

impl Default for PinMatrixRequestType {
    fn default() -> Self {
        PinMatrixRequestType::PinMatrixRequestType_Current
    }
}

impl From<i32> for PinMatrixRequestType {
    fn from(i: i32) -> Self {
        match i {
            1 => PinMatrixRequestType::PinMatrixRequestType_Current,
            2 => PinMatrixRequestType::PinMatrixRequestType_NewFirst,
            3 => PinMatrixRequestType::PinMatrixRequestType_NewSecond,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PinMatrixAck<'a> {
    pub pin: Cow<'a, str>,
}

impl<'a> PinMatrixAck<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.pin = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PinMatrixAck<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.pin).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.pin))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Success<'a> {
    pub message: Option<Cow<'a, str>>,
}

impl<'a> Success<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Success<'a> {
    fn get_size(&self) -> usize {
        0
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.message { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Failure<'a> {
    pub code: Option<mod_Failure::FailureType>,
    pub message: Option<Cow<'a, str>>,
}

impl<'a> Failure<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.code = Some(r.read_enum(bytes)?),
                Ok(18) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Failure<'a> {
    fn get_size(&self) -> usize {
        0
        + self.code.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.code { w.write_with_tag(8, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.message { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

pub mod mod_Failure {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FailureType {
    Failure_UnexpectedMessage = 1,
    Failure_ButtonExpected = 2,
    Failure_SyntaxError = 3,
    Failure_ActionCancelled = 4,
    Failure_PinExpected = 5,
    Failure_PinCancelled = 6,
    Failure_InvalidSignature = 8,
    Failure_Other = 9,
    Failure_NotEnoughFunds = 10,
    Failure_NotInitialized = 11,
    Failure_FirmwareError = 99,
}

impl Default for FailureType {
    fn default() -> Self {
        FailureType::Failure_UnexpectedMessage
    }
}

impl From<i32> for FailureType {
    fn from(i: i32) -> Self {
        match i {
            1 => FailureType::Failure_UnexpectedMessage,
            2 => FailureType::Failure_ButtonExpected,
            3 => FailureType::Failure_SyntaxError,
            4 => FailureType::Failure_ActionCancelled,
            5 => FailureType::Failure_PinExpected,
            6 => FailureType::Failure_PinCancelled,
            8 => FailureType::Failure_InvalidSignature,
            9 => FailureType::Failure_Other,
            10 => FailureType::Failure_NotEnoughFunds,
            11 => FailureType::Failure_NotInitialized,
            99 => FailureType::Failure_FirmwareError,
            _ => Self::default(),
        }
    }
}

}
