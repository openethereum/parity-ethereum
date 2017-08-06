//! Automatically generated rust module for 'messages2.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use std::io::Write;
use std::borrow::Cow;
use serde_json;
use serde_derive;
use self::quick_protobuf::{MessageWrite, BytesReader, Writer, Result};
use self::quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum OutputScriptType {
    PAYTOADDRESS = 0,
    PAYTOSCRIPTHASH = 1,
    PAYTOMULTISIG = 2,
    PAYTOOPRETURN = 3,
    PAYTOWITNESS = 4,
    PAYTOP2SHWITNESS = 5,
}

impl Default for OutputScriptType {
    fn default() -> Self {
        OutputScriptType::PAYTOADDRESS
    }
}

impl From<i32> for OutputScriptType {
    fn from(i: i32) -> Self {
        match i {
            0 => OutputScriptType::PAYTOADDRESS,
            1 => OutputScriptType::PAYTOSCRIPTHASH,
            2 => OutputScriptType::PAYTOMULTISIG,
            3 => OutputScriptType::PAYTOOPRETURN,
            4 => OutputScriptType::PAYTOWITNESS,
            5 => OutputScriptType::PAYTOP2SHWITNESS,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum OutputAddressType {
    SPEND = 0,
    TRANSFER = 1,
    CHANGE = 2,
    EXCHANGE = 3,
}

impl Default for OutputAddressType {
    fn default() -> Self {
        OutputAddressType::SPEND
    }
}

impl From<i32> for OutputAddressType {
    fn from(i: i32) -> Self {
        match i {
            0 => OutputAddressType::SPEND,
            1 => OutputAddressType::TRANSFER,
            2 => OutputAddressType::CHANGE,
            3 => OutputAddressType::EXCHANGE,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum RequestType {
    TXINPUT = 0,
    TXOUTPUT = 1,
    TXMETA = 2,
    TXFINISHED = 3,
    TXEXTRADATA = 4,
}

impl Default for RequestType {
    fn default() -> Self {
        RequestType::TXINPUT
    }
}

impl From<i32> for RequestType {
    fn from(i: i32) -> Self {
        match i {
            0 => RequestType::TXINPUT,
            1 => RequestType::TXOUTPUT,
            2 => RequestType::TXMETA,
            3 => RequestType::TXFINISHED,
            4 => RequestType::TXEXTRADATA,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
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
    ButtonRequest_ChangeLabel = 14,
    ButtonRequest_ChangeLanguage = 15,
    ButtonRequest_EnablePassphrase = 16,
    ButtonRequest_DisablePassphrase = 17,
    ButtonRequest_EncryptAndSignMessage = 18,
    ButtonRequest_EncryptMessage = 19,
    ButtonRequest_ImportPrivateKey = 20,
    ButtonRequest_ImportRecoverySentence = 21,
    ButtonRequest_SignIdentity = 22,
    ButtonRequest_Ping = 23,
    ButtonRequest_RemovePin = 24,
    ButtonRequest_ChangePin = 25,
    ButtonRequest_CreatePin = 26,
    ButtonRequest_GetEntropy = 27,
    ButtonRequest_SignMessage = 28,
    ButtonRequest_ApplyPolicies = 29,
    ButtonRequest_SignExchange = 30,
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
            14 => ButtonRequestType::ButtonRequest_ChangeLabel,
            15 => ButtonRequestType::ButtonRequest_ChangeLanguage,
            16 => ButtonRequestType::ButtonRequest_EnablePassphrase,
            17 => ButtonRequestType::ButtonRequest_DisablePassphrase,
            18 => ButtonRequestType::ButtonRequest_EncryptAndSignMessage,
            19 => ButtonRequestType::ButtonRequest_EncryptMessage,
            20 => ButtonRequestType::ButtonRequest_ImportPrivateKey,
            21 => ButtonRequestType::ButtonRequest_ImportRecoverySentence,
            22 => ButtonRequestType::ButtonRequest_SignIdentity,
            23 => ButtonRequestType::ButtonRequest_Ping,
            24 => ButtonRequestType::ButtonRequest_RemovePin,
            25 => ButtonRequestType::ButtonRequest_ChangePin,
            26 => ButtonRequestType::ButtonRequest_CreatePin,
            27 => ButtonRequestType::ButtonRequest_GetEntropy,
            28 => ButtonRequestType::ButtonRequest_SignMessage,
            29 => ButtonRequestType::ButtonRequest_ApplyPolicies,
            30 => ButtonRequestType::ButtonRequest_SignExchange,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum FailureType {
    Failure_UnexpectedMessage = 1,
    Failure_ButtonExpected = 2,
    Failure_SyntaxError = 3,
    Failure_ActionCancelled = 4,
    Failure_PinExpected = 5,
    Failure_PinCancelled = 6,
    Failure_PinInvalid = 7,
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
            7 => FailureType::Failure_PinInvalid,
            8 => FailureType::Failure_InvalidSignature,
            9 => FailureType::Failure_Other,
            10 => FailureType::Failure_NotEnoughFunds,
            11 => FailureType::Failure_NotInitialized,
            99 => FailureType::Failure_FirmwareError,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum InputScriptType {
    SPENDADDRESS = 0,
    SPENDMULTISIG = 1,
    EXTERNAL = 2,
    SPENDWITNESS = 3,
    SPENDP2SHWITNESS = 4,
}

impl Default for InputScriptType {
    fn default() -> Self {
        InputScriptType::SPENDADDRESS
    }
}

impl From<i32> for InputScriptType {
    fn from(i: i32) -> Self {
        match i {
            0 => InputScriptType::SPENDADDRESS,
            1 => InputScriptType::SPENDMULTISIG,
            2 => InputScriptType::EXTERNAL,
            3 => InputScriptType::SPENDWITNESS,
            4 => InputScriptType::SPENDP2SHWITNESS,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MessageType {
    MessageType_Initialize = 0,
    MessageType_Ping = 1,
    MessageType_Success = 2,
    MessageType_Failure = 3,
    MessageType_ChangePin = 4,
    MessageType_WipeDevice = 5,
    MessageType_FirmwareErase = 6,
    MessageType_FirmwareUpload = 7,
    MessageType_GetEntropy = 9,
    MessageType_Entropy = 10,
    MessageType_GetPublicKey = 11,
    MessageType_PublicKey = 12,
    MessageType_LoadDevice = 13,
    MessageType_ResetDevice = 14,
    MessageType_SignTx = 15,
    MessageType_SimpleSignTx = 16,
    MessageType_Features = 17,
    MessageType_PinMatrixRequest = 18,
    MessageType_PinMatrixAck = 19,
    MessageType_Cancel = 20,
    MessageType_TxRequest = 21,
    MessageType_TxAck = 22,
    MessageType_CipherKeyValue = 23,
    MessageType_ClearSession = 24,
    MessageType_ApplySettings = 25,
    MessageType_ButtonRequest = 26,
    MessageType_ButtonAck = 27,
    MessageType_GetAddress = 29,
    MessageType_Address = 30,
    MessageType_EntropyRequest = 35,
    MessageType_EntropyAck = 36,
    MessageType_SignMessage = 38,
    MessageType_VerifyMessage = 39,
    MessageType_MessageSignature = 40,
    MessageType_PassphraseRequest = 41,
    MessageType_PassphraseAck = 42,
    MessageType_EstimateTxSize = 43,
    MessageType_TxSize = 44,
    MessageType_RecoveryDevice = 45,
    MessageType_WordRequest = 46,
    MessageType_WordAck = 47,
    MessageType_CipheredKeyValue = 48,
    MessageType_EncryptMessage = 49,
    MessageType_EncryptedMessage = 50,
    MessageType_DecryptMessage = 51,
    MessageType_DecryptedMessage = 52,
    MessageType_SignIdentity = 53,
    MessageType_SignedIdentity = 54,
    MessageType_GetFeatures = 55,
    MessageType_EthereumGetAddress = 56,
    MessageType_EthereumAddress = 57,
    MessageType_EthereumSignTx = 58,
    MessageType_EthereumTxRequest = 59,
    MessageType_EthereumTxAck = 60,
    MessageType_CharacterRequest = 80,
    MessageType_CharacterAck = 81,
    MessageType_RawTxAck = 82,
    MessageType_ApplyPolicies = 83,
    MessageType_DebugLinkDecision = 100,
    MessageType_DebugLinkGetState = 101,
    MessageType_DebugLinkState = 102,
    MessageType_DebugLinkStop = 103,
    MessageType_DebugLinkLog = 104,
    MessageType_DebugLinkFillConfig = 105,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::MessageType_Initialize
    }
}

impl From<i32> for MessageType {
    fn from(i: i32) -> Self {
        match i {
            0 => MessageType::MessageType_Initialize,
            1 => MessageType::MessageType_Ping,
            2 => MessageType::MessageType_Success,
            3 => MessageType::MessageType_Failure,
            4 => MessageType::MessageType_ChangePin,
            5 => MessageType::MessageType_WipeDevice,
            6 => MessageType::MessageType_FirmwareErase,
            7 => MessageType::MessageType_FirmwareUpload,
            9 => MessageType::MessageType_GetEntropy,
            10 => MessageType::MessageType_Entropy,
            11 => MessageType::MessageType_GetPublicKey,
            12 => MessageType::MessageType_PublicKey,
            13 => MessageType::MessageType_LoadDevice,
            14 => MessageType::MessageType_ResetDevice,
            15 => MessageType::MessageType_SignTx,
            16 => MessageType::MessageType_SimpleSignTx,
            17 => MessageType::MessageType_Features,
            18 => MessageType::MessageType_PinMatrixRequest,
            19 => MessageType::MessageType_PinMatrixAck,
            20 => MessageType::MessageType_Cancel,
            21 => MessageType::MessageType_TxRequest,
            22 => MessageType::MessageType_TxAck,
            23 => MessageType::MessageType_CipherKeyValue,
            24 => MessageType::MessageType_ClearSession,
            25 => MessageType::MessageType_ApplySettings,
            26 => MessageType::MessageType_ButtonRequest,
            27 => MessageType::MessageType_ButtonAck,
            29 => MessageType::MessageType_GetAddress,
            30 => MessageType::MessageType_Address,
            35 => MessageType::MessageType_EntropyRequest,
            36 => MessageType::MessageType_EntropyAck,
            38 => MessageType::MessageType_SignMessage,
            39 => MessageType::MessageType_VerifyMessage,
            40 => MessageType::MessageType_MessageSignature,
            41 => MessageType::MessageType_PassphraseRequest,
            42 => MessageType::MessageType_PassphraseAck,
            43 => MessageType::MessageType_EstimateTxSize,
            44 => MessageType::MessageType_TxSize,
            45 => MessageType::MessageType_RecoveryDevice,
            46 => MessageType::MessageType_WordRequest,
            47 => MessageType::MessageType_WordAck,
            48 => MessageType::MessageType_CipheredKeyValue,
            49 => MessageType::MessageType_EncryptMessage,
            50 => MessageType::MessageType_EncryptedMessage,
            51 => MessageType::MessageType_DecryptMessage,
            52 => MessageType::MessageType_DecryptedMessage,
            53 => MessageType::MessageType_SignIdentity,
            54 => MessageType::MessageType_SignedIdentity,
            55 => MessageType::MessageType_GetFeatures,
            56 => MessageType::MessageType_EthereumGetAddress,
            57 => MessageType::MessageType_EthereumAddress,
            58 => MessageType::MessageType_EthereumSignTx,
            59 => MessageType::MessageType_EthereumTxRequest,
            60 => MessageType::MessageType_EthereumTxAck,
            80 => MessageType::MessageType_CharacterRequest,
            81 => MessageType::MessageType_CharacterAck,
            82 => MessageType::MessageType_RawTxAck,
            83 => MessageType::MessageType_ApplyPolicies,
            100 => MessageType::MessageType_DebugLinkDecision,
            101 => MessageType::MessageType_DebugLinkGetState,
            102 => MessageType::MessageType_DebugLinkState,
            103 => MessageType::MessageType_DebugLinkStop,
            104 => MessageType::MessageType_DebugLinkLog,
            105 => MessageType::MessageType_DebugLinkFillConfig,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct Initialize { }

impl Initialize {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for Initialize { }

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct GetFeatures { }

impl GetFeatures {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for GetFeatures { }

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
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
    pub policies: Vec<PolicyType<'a>>,
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
                Ok(146) => msg.policies.push(r.read_message(bytes, PolicyType::from_reader)?),
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
        + self.policies.iter().map(|s| 2 + sizeof_len((s).get_size())).sum::<usize>()
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
        for s in &self.policies { w.write_with_tag(146, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct PolicyType<'a> {
    pub policy_name: Option<Cow<'a, str>>,
    pub enabled: Option<bool>,
}

impl<'a> PolicyType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.policy_name = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(16) => msg.enabled = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PolicyType<'a> {
    fn get_size(&self) -> usize {
        0
        + self.policy_name.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.enabled.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.policy_name { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.enabled { w.write_with_tag(16, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ClearSession { }

impl ClearSession {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for ClearSession { }

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct Failure<'a> {
    pub code: Option<FailureType>,
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ButtonRequest<'a> {
    pub code: Option<ButtonRequestType>,
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ButtonAck { }

impl ButtonAck {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for ButtonAck { }

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct PinMatrixRequest {
    pub type_pb: Option<PinMatrixRequestType>,
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct Cancel { }

impl Cancel {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for Cancel { }

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct PassphraseRequest { }

impl PassphraseRequest {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for PassphraseRequest { }

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct PassphraseAck<'a> {
    pub passphrase: Cow<'a, str>,
}

impl<'a> PassphraseAck<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.passphrase = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PassphraseAck<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.passphrase).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.passphrase))?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct EthereumGetAddress {
    pub address_n: Vec<u32>,
    pub show_display: Option<bool>,
}

impl EthereumGetAddress {
    pub fn from_reader(r: &mut BytesReader, bytes: &[u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.address_n.push(r.read_uint32(bytes)?),
                Ok(16) => msg.show_display = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for EthereumGetAddress {
    fn get_size(&self) -> usize {
        0
        + self.address_n.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
        + self.show_display.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.address_n { w.write_with_tag(8, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.show_display { w.write_with_tag(16, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct EthereumAddress<'a> {
    pub address: Cow<'a, [u8]>,
}

impl<'a> EthereumAddress<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.address = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for EthereumAddress<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.address).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_bytes(&**&self.address))?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct LoadDevice<'a> {
    pub mnemonic: Option<Cow<'a, str>>,
    pub node: Option<HDNodeType<'a>>,
    pub pin: Option<Cow<'a, str>>,
    pub passphrase_protection: Option<bool>,
    pub language: Cow<'a, str>,
    pub label: Option<Cow<'a, str>>,
    pub skip_checksum: Option<bool>,
}

impl<'a> LoadDevice<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = LoadDevice {
            language: Cow::Borrowed("english"),
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.mnemonic = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.node = Some(r.read_message(bytes, HDNodeType::from_reader)?),
                Ok(26) => msg.pin = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(32) => msg.passphrase_protection = Some(r.read_bool(bytes)?),
                Ok(42) => msg.language = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(50) => msg.label = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(56) => msg.skip_checksum = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for LoadDevice<'a> {
    fn get_size(&self) -> usize {
        0
        + self.mnemonic.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.node.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.pin.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.passphrase_protection.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + if self.language == Cow::Borrowed("english") { 0 } else { 1 + sizeof_len((&self.language).len()) }
        + self.label.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.skip_checksum.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.mnemonic { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.node { w.write_with_tag(18, |w| w.write_message(s))?; }
        if let Some(ref s) = self.pin { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.passphrase_protection { w.write_with_tag(32, |w| w.write_bool(*s))?; }
        if self.language != Cow::Borrowed("english") { w.write_with_tag(42, |w| w.write_string(&**&self.language))?; }
        if let Some(ref s) = self.label { w.write_with_tag(50, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.skip_checksum { w.write_with_tag(56, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct EthereumSignTx<'a> {
    pub address_n: Vec<u32>,
    pub nonce: Option<Cow<'a, [u8]>>,
    pub gas_price: Option<Cow<'a, [u8]>>,
    pub gas_limit: Option<Cow<'a, [u8]>>,
    pub to: Option<Cow<'a, [u8]>>,
    pub value: Option<Cow<'a, [u8]>>,
    pub data_initial_chunk: Option<Cow<'a, [u8]>>,
    pub data_length: Option<u32>,
    pub to_address_n: Vec<u32>,
    pub address_type: Option<OutputAddressType>,
    pub exchange_type: Option<ExchangeType<'a>>,
    pub chain_id: Option<u32>,
}

impl<'a> EthereumSignTx<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.address_n.push(r.read_uint32(bytes)?),
                Ok(18) => msg.nonce = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.gas_price = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(34) => msg.gas_limit = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(42) => msg.to = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(50) => msg.value = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(58) => msg.data_initial_chunk = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(64) => msg.data_length = Some(r.read_uint32(bytes)?),
                Ok(72) => msg.to_address_n.push(r.read_uint32(bytes)?),
                Ok(80) => msg.address_type = Some(r.read_enum(bytes)?),
                Ok(90) => msg.exchange_type = Some(r.read_message(bytes, ExchangeType::from_reader)?),
                Ok(96) => msg.chain_id = Some(r.read_uint32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for EthereumSignTx<'a> {
    fn get_size(&self) -> usize {
        0
        + self.address_n.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
        + self.nonce.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.gas_price.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.gas_limit.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.to.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.value.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.data_initial_chunk.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.data_length.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.to_address_n.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
        + self.address_type.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.exchange_type.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.chain_id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.address_n { w.write_with_tag(8, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.nonce { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.gas_price { w.write_with_tag(26, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.gas_limit { w.write_with_tag(34, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.to { w.write_with_tag(42, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.value { w.write_with_tag(50, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.data_initial_chunk { w.write_with_tag(58, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.data_length { w.write_with_tag(64, |w| w.write_uint32(*s))?; }
        for s in &self.to_address_n { w.write_with_tag(72, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.address_type { w.write_with_tag(80, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.exchange_type { w.write_with_tag(90, |w| w.write_message(s))?; }
        if let Some(ref s) = self.chain_id { w.write_with_tag(96, |w| w.write_uint32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ExchangeType<'a> {
    pub signed_exchange_response: Option<SignedExchangeResponse<'a>>,
    pub withdrawal_coin_name: Cow<'a, str>,
    pub withdrawal_address_n: Vec<u32>,
    pub return_address_n: Vec<u32>,
}

impl<'a> ExchangeType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = ExchangeType {
            withdrawal_coin_name: Cow::Borrowed("Ethereum"),
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.signed_exchange_response = Some(r.read_message(bytes, SignedExchangeResponse::from_reader)?),
                Ok(18) => msg.withdrawal_coin_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.withdrawal_address_n.push(r.read_uint32(bytes)?),
                Ok(32) => msg.return_address_n.push(r.read_uint32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ExchangeType<'a> {
    fn get_size(&self) -> usize {
        0
        + self.signed_exchange_response.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + if self.withdrawal_coin_name == Cow::Borrowed("Ethereum") { 0 } else { 1 + sizeof_len((&self.withdrawal_coin_name).len()) }
        + self.withdrawal_address_n.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
        + self.return_address_n.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.signed_exchange_response { w.write_with_tag(10, |w| w.write_message(s))?; }
        if self.withdrawal_coin_name != Cow::Borrowed("Ethereum") { w.write_with_tag(18, |w| w.write_string(&**&self.withdrawal_coin_name))?; }
        for s in &self.withdrawal_address_n { w.write_with_tag(24, |w| w.write_uint32(*s))?; }
        for s in &self.return_address_n { w.write_with_tag(32, |w| w.write_uint32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct SignedExchangeResponse<'a> {
    pub response: Option<ExchangeResponse<'a>>,
    pub signature: Option<Cow<'a, [u8]>>,
    pub responseV2: Option<ExchangeResponseV2<'a>>,
}

impl<'a> SignedExchangeResponse<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.response = Some(r.read_message(bytes, ExchangeResponse::from_reader)?),
                Ok(18) => msg.signature = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.responseV2 = Some(r.read_message(bytes, ExchangeResponseV2::from_reader)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for SignedExchangeResponse<'a> {
    fn get_size(&self) -> usize {
        0
        + self.response.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.signature.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.responseV2.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.response { w.write_with_tag(10, |w| w.write_message(s))?; }
        if let Some(ref s) = self.signature { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.responseV2 { w.write_with_tag(26, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ExchangeResponse<'a> {
    pub deposit_address: Option<ExchangeAddress<'a>>,
    pub deposit_amount: Option<u64>,
    pub expiration: Option<i64>,
    pub quoted_rate: Option<u64>,
    pub withdrawal_address: Option<ExchangeAddress<'a>>,
    pub withdrawal_amount: Option<u64>,
    pub return_address: Option<ExchangeAddress<'a>>,
    pub api_key: Option<Cow<'a, [u8]>>,
    pub miner_fee: Option<u64>,
    pub order_id: Option<Cow<'a, [u8]>>,
}

impl<'a> ExchangeResponse<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.deposit_address = Some(r.read_message(bytes, ExchangeAddress::from_reader)?),
                Ok(16) => msg.deposit_amount = Some(r.read_uint64(bytes)?),
                Ok(24) => msg.expiration = Some(r.read_int64(bytes)?),
                Ok(32) => msg.quoted_rate = Some(r.read_uint64(bytes)?),
                Ok(42) => msg.withdrawal_address = Some(r.read_message(bytes, ExchangeAddress::from_reader)?),
                Ok(48) => msg.withdrawal_amount = Some(r.read_uint64(bytes)?),
                Ok(58) => msg.return_address = Some(r.read_message(bytes, ExchangeAddress::from_reader)?),
                Ok(66) => msg.api_key = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(72) => msg.miner_fee = Some(r.read_uint64(bytes)?),
                Ok(82) => msg.order_id = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ExchangeResponse<'a> {
    fn get_size(&self) -> usize {
        0
        + self.deposit_address.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.deposit_amount.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.expiration.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.quoted_rate.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.withdrawal_address.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.withdrawal_amount.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.return_address.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.api_key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.miner_fee.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.order_id.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.deposit_address { w.write_with_tag(10, |w| w.write_message(s))?; }
        if let Some(ref s) = self.deposit_amount { w.write_with_tag(16, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.expiration { w.write_with_tag(24, |w| w.write_int64(*s))?; }
        if let Some(ref s) = self.quoted_rate { w.write_with_tag(32, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.withdrawal_address { w.write_with_tag(42, |w| w.write_message(s))?; }
        if let Some(ref s) = self.withdrawal_amount { w.write_with_tag(48, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.return_address { w.write_with_tag(58, |w| w.write_message(s))?; }
        if let Some(ref s) = self.api_key { w.write_with_tag(66, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.miner_fee { w.write_with_tag(72, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.order_id { w.write_with_tag(82, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ExchangeResponseV2<'a> {
    pub deposit_address: Option<ExchangeAddress<'a>>,
    pub deposit_amount: Option<Cow<'a, [u8]>>,
    pub expiration: Option<i64>,
    pub quoted_rate: Option<Cow<'a, [u8]>>,
    pub withdrawal_address: Option<ExchangeAddress<'a>>,
    pub withdrawal_amount: Option<Cow<'a, [u8]>>,
    pub return_address: Option<ExchangeAddress<'a>>,
    pub api_key: Option<Cow<'a, [u8]>>,
    pub miner_fee: Option<Cow<'a, [u8]>>,
    pub order_id: Option<Cow<'a, [u8]>>,
}

impl<'a> ExchangeResponseV2<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.deposit_address = Some(r.read_message(bytes, ExchangeAddress::from_reader)?),
                Ok(18) => msg.deposit_amount = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.expiration = Some(r.read_int64(bytes)?),
                Ok(34) => msg.quoted_rate = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(42) => msg.withdrawal_address = Some(r.read_message(bytes, ExchangeAddress::from_reader)?),
                Ok(50) => msg.withdrawal_amount = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(58) => msg.return_address = Some(r.read_message(bytes, ExchangeAddress::from_reader)?),
                Ok(66) => msg.api_key = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(74) => msg.miner_fee = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(82) => msg.order_id = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ExchangeResponseV2<'a> {
    fn get_size(&self) -> usize {
        0
        + self.deposit_address.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.deposit_amount.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.expiration.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.quoted_rate.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.withdrawal_address.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.withdrawal_amount.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.return_address.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.api_key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.miner_fee.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.order_id.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.deposit_address { w.write_with_tag(10, |w| w.write_message(s))?; }
        if let Some(ref s) = self.deposit_amount { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.expiration { w.write_with_tag(24, |w| w.write_int64(*s))?; }
        if let Some(ref s) = self.quoted_rate { w.write_with_tag(34, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.withdrawal_address { w.write_with_tag(42, |w| w.write_message(s))?; }
        if let Some(ref s) = self.withdrawal_amount { w.write_with_tag(50, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.return_address { w.write_with_tag(58, |w| w.write_message(s))?; }
        if let Some(ref s) = self.api_key { w.write_with_tag(66, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.miner_fee { w.write_with_tag(74, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.order_id { w.write_with_tag(82, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ExchangeAddress<'a> {
    pub coin_type: Option<Cow<'a, str>>,
    pub address: Option<Cow<'a, str>>,
    pub dest_tag: Option<Cow<'a, str>>,
    pub rs_address: Option<Cow<'a, str>>,
}

impl<'a> ExchangeAddress<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.coin_type = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.address = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.dest_tag = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(34) => msg.rs_address = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ExchangeAddress<'a> {
    fn get_size(&self) -> usize {
        0
        + self.coin_type.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.address.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.dest_tag.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.rs_address.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.coin_type { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.address { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.dest_tag { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.rs_address { w.write_with_tag(34, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct EthereumTxRequest<'a> {
    pub data_length: Option<u32>,
    pub signature_v: Option<u32>,
    pub signature_r: Option<Cow<'a, [u8]>>,
    pub signature_s: Option<Cow<'a, [u8]>>,
    pub hash: Option<Cow<'a, [u8]>>,
    pub signature_der: Option<Cow<'a, [u8]>>,
}

impl<'a> EthereumTxRequest<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.data_length = Some(r.read_uint32(bytes)?),
                Ok(16) => msg.signature_v = Some(r.read_uint32(bytes)?),
                Ok(26) => msg.signature_r = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(34) => msg.signature_s = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(42) => msg.hash = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(50) => msg.signature_der = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for EthereumTxRequest<'a> {
    fn get_size(&self) -> usize {
        0
        + self.data_length.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.signature_v.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.signature_r.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.signature_s.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.hash.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.signature_der.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.data_length { w.write_with_tag(8, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.signature_v { w.write_with_tag(16, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.signature_r { w.write_with_tag(26, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.signature_s { w.write_with_tag(34, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.hash { w.write_with_tag(42, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.signature_der { w.write_with_tag(50, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct EthereumTxAck<'a> {
    pub data_chunk: Option<Cow<'a, [u8]>>,
}

impl<'a> EthereumTxAck<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.data_chunk = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for EthereumTxAck<'a> {
    fn get_size(&self) -> usize {
        0
        + self.data_chunk.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.data_chunk { w.write_with_tag(10, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct CoinType<'a> {
    pub coin_name: Option<Cow<'a, str>>,
    pub coin_shortcut: Option<Cow<'a, str>>,
    pub address_type: u32,
    pub maxfee_kb: Option<u64>,
    pub address_type_p2sh: u32,
    pub address_type_p2wpkh: u32,
    pub address_type_p2wsh: u32,
    pub signed_message_header: Option<Cow<'a, str>>,
    pub bip44_account_path: Option<u32>,
}

impl<'a> CoinType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = CoinType {
            address_type_p2sh: 5u32,
            address_type_p2wpkh: 6u32,
            address_type_p2wsh: 10u32,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.coin_name = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.coin_shortcut = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.address_type = r.read_uint32(bytes)?,
                Ok(32) => msg.maxfee_kb = Some(r.read_uint64(bytes)?),
                Ok(40) => msg.address_type_p2sh = r.read_uint32(bytes)?,
                Ok(48) => msg.address_type_p2wpkh = r.read_uint32(bytes)?,
                Ok(56) => msg.address_type_p2wsh = r.read_uint32(bytes)?,
                Ok(66) => msg.signed_message_header = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(72) => msg.bip44_account_path = Some(r.read_uint32(bytes)?),
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
        + if self.address_type_p2wpkh == 6u32 { 0 } else { 1 + sizeof_varint(*(&self.address_type_p2wpkh) as u64) }
        + if self.address_type_p2wsh == 10u32 { 0 } else { 1 + sizeof_varint(*(&self.address_type_p2wsh) as u64) }
        + self.signed_message_header.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.bip44_account_path.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.coin_name { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.coin_shortcut { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if self.address_type != 0u32 { w.write_with_tag(24, |w| w.write_uint32(*&self.address_type))?; }
        if let Some(ref s) = self.maxfee_kb { w.write_with_tag(32, |w| w.write_uint64(*s))?; }
        if self.address_type_p2sh != 5u32 { w.write_with_tag(40, |w| w.write_uint32(*&self.address_type_p2sh))?; }
        if self.address_type_p2wpkh != 6u32 { w.write_with_tag(48, |w| w.write_uint32(*&self.address_type_p2wpkh))?; }
        if self.address_type_p2wsh != 10u32 { w.write_with_tag(56, |w| w.write_uint32(*&self.address_type_p2wsh))?; }
        if let Some(ref s) = self.signed_message_header { w.write_with_tag(66, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.bip44_account_path { w.write_with_tag(72, |w| w.write_uint32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct TxRequestDetailsType<'a> {
    pub request_index: Option<u32>,
    pub tx_hash: Option<Cow<'a, [u8]>>,
    pub extra_data_len: Option<u32>,
    pub extra_data_offset: Option<u32>,
}

impl<'a> TxRequestDetailsType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.request_index = Some(r.read_uint32(bytes)?),
                Ok(18) => msg.tx_hash = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.extra_data_len = Some(r.read_uint32(bytes)?),
                Ok(32) => msg.extra_data_offset = Some(r.read_uint32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TxRequestDetailsType<'a> {
    fn get_size(&self) -> usize {
        0
        + self.request_index.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.tx_hash.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.extra_data_len.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.extra_data_offset.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.request_index { w.write_with_tag(8, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.tx_hash { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.extra_data_len { w.write_with_tag(24, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.extra_data_offset { w.write_with_tag(32, |w| w.write_uint32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct TxRequestSerializedType<'a> {
    pub signature_index: Option<u32>,
    pub signature: Option<Cow<'a, [u8]>>,
    pub serialized_tx: Option<Cow<'a, [u8]>>,
}

impl<'a> TxRequestSerializedType<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.signature_index = Some(r.read_uint32(bytes)?),
                Ok(18) => msg.signature = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.serialized_tx = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TxRequestSerializedType<'a> {
    fn get_size(&self) -> usize {
        0
        + self.signature_index.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.signature.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.serialized_tx.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.signature_index { w.write_with_tag(8, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.signature { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.serialized_tx { w.write_with_tag(26, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}
