// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct Initialize {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Initialize {}

impl Initialize {
    pub fn new() -> Initialize {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Initialize {
        static mut instance: ::protobuf::lazy::Lazy<Initialize> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Initialize,
        };
        unsafe {
            instance.get(Initialize::new)
        }
    }
}

impl ::protobuf::Message for Initialize {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Initialize {
    fn new() -> Initialize {
        Initialize::new()
    }

    fn descriptor_static(_: ::std::option::Option<Initialize>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<Initialize>(
                    "Initialize",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Initialize {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Initialize {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Initialize {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct GetFeatures {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for GetFeatures {}

impl GetFeatures {
    pub fn new() -> GetFeatures {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static GetFeatures {
        static mut instance: ::protobuf::lazy::Lazy<GetFeatures> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const GetFeatures,
        };
        unsafe {
            instance.get(GetFeatures::new)
        }
    }
}

impl ::protobuf::Message for GetFeatures {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for GetFeatures {
    fn new() -> GetFeatures {
        GetFeatures::new()
    }

    fn descriptor_static(_: ::std::option::Option<GetFeatures>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<GetFeatures>(
                    "GetFeatures",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for GetFeatures {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for GetFeatures {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for GetFeatures {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Features {
    // message fields
    vendor: ::protobuf::SingularField<::std::string::String>,
    major_version: ::std::option::Option<u32>,
    minor_version: ::std::option::Option<u32>,
    patch_version: ::std::option::Option<u32>,
    bootloader_mode: ::std::option::Option<bool>,
    device_id: ::protobuf::SingularField<::std::string::String>,
    pin_protection: ::std::option::Option<bool>,
    passphrase_protection: ::std::option::Option<bool>,
    language: ::protobuf::SingularField<::std::string::String>,
    label: ::protobuf::SingularField<::std::string::String>,
    coins: ::protobuf::RepeatedField<super::types::CoinType>,
    initialized: ::std::option::Option<bool>,
    revision: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    bootloader_hash: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    imported: ::std::option::Option<bool>,
    pin_cached: ::std::option::Option<bool>,
    passphrase_cached: ::std::option::Option<bool>,
    firmware_present: ::std::option::Option<bool>,
    needs_backup: ::std::option::Option<bool>,
    flags: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Features {}

impl Features {
    pub fn new() -> Features {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Features {
        static mut instance: ::protobuf::lazy::Lazy<Features> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Features,
        };
        unsafe {
            instance.get(Features::new)
        }
    }

    // optional string vendor = 1;

    pub fn clear_vendor(&mut self) {
        self.vendor.clear();
    }

    pub fn has_vendor(&self) -> bool {
        self.vendor.is_some()
    }

    // Param is passed by value, moved
    pub fn set_vendor(&mut self, v: ::std::string::String) {
        self.vendor = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_vendor(&mut self) -> &mut ::std::string::String {
        if self.vendor.is_none() {
            self.vendor.set_default();
        }
        self.vendor.as_mut().unwrap()
    }

    // Take field
    pub fn take_vendor(&mut self) -> ::std::string::String {
        self.vendor.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_vendor(&self) -> &str {
        match self.vendor.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_vendor_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.vendor
    }

    fn mut_vendor_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.vendor
    }

    // optional uint32 major_version = 2;

    pub fn clear_major_version(&mut self) {
        self.major_version = ::std::option::Option::None;
    }

    pub fn has_major_version(&self) -> bool {
        self.major_version.is_some()
    }

    // Param is passed by value, moved
    pub fn set_major_version(&mut self, v: u32) {
        self.major_version = ::std::option::Option::Some(v);
    }

    pub fn get_major_version(&self) -> u32 {
        self.major_version.unwrap_or(0)
    }

    fn get_major_version_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.major_version
    }

    fn mut_major_version_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.major_version
    }

    // optional uint32 minor_version = 3;

    pub fn clear_minor_version(&mut self) {
        self.minor_version = ::std::option::Option::None;
    }

    pub fn has_minor_version(&self) -> bool {
        self.minor_version.is_some()
    }

    // Param is passed by value, moved
    pub fn set_minor_version(&mut self, v: u32) {
        self.minor_version = ::std::option::Option::Some(v);
    }

    pub fn get_minor_version(&self) -> u32 {
        self.minor_version.unwrap_or(0)
    }

    fn get_minor_version_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.minor_version
    }

    fn mut_minor_version_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.minor_version
    }

    // optional uint32 patch_version = 4;

    pub fn clear_patch_version(&mut self) {
        self.patch_version = ::std::option::Option::None;
    }

    pub fn has_patch_version(&self) -> bool {
        self.patch_version.is_some()
    }

    // Param is passed by value, moved
    pub fn set_patch_version(&mut self, v: u32) {
        self.patch_version = ::std::option::Option::Some(v);
    }

    pub fn get_patch_version(&self) -> u32 {
        self.patch_version.unwrap_or(0)
    }

    fn get_patch_version_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.patch_version
    }

    fn mut_patch_version_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.patch_version
    }

    // optional bool bootloader_mode = 5;

    pub fn clear_bootloader_mode(&mut self) {
        self.bootloader_mode = ::std::option::Option::None;
    }

    pub fn has_bootloader_mode(&self) -> bool {
        self.bootloader_mode.is_some()
    }

    // Param is passed by value, moved
    pub fn set_bootloader_mode(&mut self, v: bool) {
        self.bootloader_mode = ::std::option::Option::Some(v);
    }

    pub fn get_bootloader_mode(&self) -> bool {
        self.bootloader_mode.unwrap_or(false)
    }

    fn get_bootloader_mode_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.bootloader_mode
    }

    fn mut_bootloader_mode_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.bootloader_mode
    }

    // optional string device_id = 6;

    pub fn clear_device_id(&mut self) {
        self.device_id.clear();
    }

    pub fn has_device_id(&self) -> bool {
        self.device_id.is_some()
    }

    // Param is passed by value, moved
    pub fn set_device_id(&mut self, v: ::std::string::String) {
        self.device_id = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_device_id(&mut self) -> &mut ::std::string::String {
        if self.device_id.is_none() {
            self.device_id.set_default();
        }
        self.device_id.as_mut().unwrap()
    }

    // Take field
    pub fn take_device_id(&mut self) -> ::std::string::String {
        self.device_id.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_device_id(&self) -> &str {
        match self.device_id.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_device_id_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.device_id
    }

    fn mut_device_id_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.device_id
    }

    // optional bool pin_protection = 7;

    pub fn clear_pin_protection(&mut self) {
        self.pin_protection = ::std::option::Option::None;
    }

    pub fn has_pin_protection(&self) -> bool {
        self.pin_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pin_protection(&mut self, v: bool) {
        self.pin_protection = ::std::option::Option::Some(v);
    }

    pub fn get_pin_protection(&self) -> bool {
        self.pin_protection.unwrap_or(false)
    }

    fn get_pin_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.pin_protection
    }

    fn mut_pin_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.pin_protection
    }

    // optional bool passphrase_protection = 8;

    pub fn clear_passphrase_protection(&mut self) {
        self.passphrase_protection = ::std::option::Option::None;
    }

    pub fn has_passphrase_protection(&self) -> bool {
        self.passphrase_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_passphrase_protection(&mut self, v: bool) {
        self.passphrase_protection = ::std::option::Option::Some(v);
    }

    pub fn get_passphrase_protection(&self) -> bool {
        self.passphrase_protection.unwrap_or(false)
    }

    fn get_passphrase_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.passphrase_protection
    }

    fn mut_passphrase_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.passphrase_protection
    }

    // optional string language = 9;

    pub fn clear_language(&mut self) {
        self.language.clear();
    }

    pub fn has_language(&self) -> bool {
        self.language.is_some()
    }

    // Param is passed by value, moved
    pub fn set_language(&mut self, v: ::std::string::String) {
        self.language = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_language(&mut self) -> &mut ::std::string::String {
        if self.language.is_none() {
            self.language.set_default();
        }
        self.language.as_mut().unwrap()
    }

    // Take field
    pub fn take_language(&mut self) -> ::std::string::String {
        self.language.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_language(&self) -> &str {
        match self.language.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_language_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.language
    }

    fn mut_language_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.language
    }

    // optional string label = 10;

    pub fn clear_label(&mut self) {
        self.label.clear();
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    // Param is passed by value, moved
    pub fn set_label(&mut self, v: ::std::string::String) {
        self.label = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_label(&mut self) -> &mut ::std::string::String {
        if self.label.is_none() {
            self.label.set_default();
        }
        self.label.as_mut().unwrap()
    }

    // Take field
    pub fn take_label(&mut self) -> ::std::string::String {
        self.label.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_label(&self) -> &str {
        match self.label.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_label_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.label
    }

    fn mut_label_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.label
    }

    // repeated .CoinType coins = 11;

    pub fn clear_coins(&mut self) {
        self.coins.clear();
    }

    // Param is passed by value, moved
    pub fn set_coins(&mut self, v: ::protobuf::RepeatedField<super::types::CoinType>) {
        self.coins = v;
    }

    // Mutable pointer to the field.
    pub fn mut_coins(&mut self) -> &mut ::protobuf::RepeatedField<super::types::CoinType> {
        &mut self.coins
    }

    // Take field
    pub fn take_coins(&mut self) -> ::protobuf::RepeatedField<super::types::CoinType> {
        ::std::mem::replace(&mut self.coins, ::protobuf::RepeatedField::new())
    }

    pub fn get_coins(&self) -> &[super::types::CoinType] {
        &self.coins
    }

    fn get_coins_for_reflect(&self) -> &::protobuf::RepeatedField<super::types::CoinType> {
        &self.coins
    }

    fn mut_coins_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<super::types::CoinType> {
        &mut self.coins
    }

    // optional bool initialized = 12;

    pub fn clear_initialized(&mut self) {
        self.initialized = ::std::option::Option::None;
    }

    pub fn has_initialized(&self) -> bool {
        self.initialized.is_some()
    }

    // Param is passed by value, moved
    pub fn set_initialized(&mut self, v: bool) {
        self.initialized = ::std::option::Option::Some(v);
    }

    pub fn get_initialized(&self) -> bool {
        self.initialized.unwrap_or(false)
    }

    fn get_initialized_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.initialized
    }

    fn mut_initialized_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.initialized
    }

    // optional bytes revision = 13;

    pub fn clear_revision(&mut self) {
        self.revision.clear();
    }

    pub fn has_revision(&self) -> bool {
        self.revision.is_some()
    }

    // Param is passed by value, moved
    pub fn set_revision(&mut self, v: ::std::vec::Vec<u8>) {
        self.revision = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_revision(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.revision.is_none() {
            self.revision.set_default();
        }
        self.revision.as_mut().unwrap()
    }

    // Take field
    pub fn take_revision(&mut self) -> ::std::vec::Vec<u8> {
        self.revision.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_revision(&self) -> &[u8] {
        match self.revision.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_revision_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.revision
    }

    fn mut_revision_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.revision
    }

    // optional bytes bootloader_hash = 14;

    pub fn clear_bootloader_hash(&mut self) {
        self.bootloader_hash.clear();
    }

    pub fn has_bootloader_hash(&self) -> bool {
        self.bootloader_hash.is_some()
    }

    // Param is passed by value, moved
    pub fn set_bootloader_hash(&mut self, v: ::std::vec::Vec<u8>) {
        self.bootloader_hash = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_bootloader_hash(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.bootloader_hash.is_none() {
            self.bootloader_hash.set_default();
        }
        self.bootloader_hash.as_mut().unwrap()
    }

    // Take field
    pub fn take_bootloader_hash(&mut self) -> ::std::vec::Vec<u8> {
        self.bootloader_hash.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_bootloader_hash(&self) -> &[u8] {
        match self.bootloader_hash.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_bootloader_hash_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.bootloader_hash
    }

    fn mut_bootloader_hash_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.bootloader_hash
    }

    // optional bool imported = 15;

    pub fn clear_imported(&mut self) {
        self.imported = ::std::option::Option::None;
    }

    pub fn has_imported(&self) -> bool {
        self.imported.is_some()
    }

    // Param is passed by value, moved
    pub fn set_imported(&mut self, v: bool) {
        self.imported = ::std::option::Option::Some(v);
    }

    pub fn get_imported(&self) -> bool {
        self.imported.unwrap_or(false)
    }

    fn get_imported_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.imported
    }

    fn mut_imported_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.imported
    }

    // optional bool pin_cached = 16;

    pub fn clear_pin_cached(&mut self) {
        self.pin_cached = ::std::option::Option::None;
    }

    pub fn has_pin_cached(&self) -> bool {
        self.pin_cached.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pin_cached(&mut self, v: bool) {
        self.pin_cached = ::std::option::Option::Some(v);
    }

    pub fn get_pin_cached(&self) -> bool {
        self.pin_cached.unwrap_or(false)
    }

    fn get_pin_cached_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.pin_cached
    }

    fn mut_pin_cached_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.pin_cached
    }

    // optional bool passphrase_cached = 17;

    pub fn clear_passphrase_cached(&mut self) {
        self.passphrase_cached = ::std::option::Option::None;
    }

    pub fn has_passphrase_cached(&self) -> bool {
        self.passphrase_cached.is_some()
    }

    // Param is passed by value, moved
    pub fn set_passphrase_cached(&mut self, v: bool) {
        self.passphrase_cached = ::std::option::Option::Some(v);
    }

    pub fn get_passphrase_cached(&self) -> bool {
        self.passphrase_cached.unwrap_or(false)
    }

    fn get_passphrase_cached_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.passphrase_cached
    }

    fn mut_passphrase_cached_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.passphrase_cached
    }

    // optional bool firmware_present = 18;

    pub fn clear_firmware_present(&mut self) {
        self.firmware_present = ::std::option::Option::None;
    }

    pub fn has_firmware_present(&self) -> bool {
        self.firmware_present.is_some()
    }

    // Param is passed by value, moved
    pub fn set_firmware_present(&mut self, v: bool) {
        self.firmware_present = ::std::option::Option::Some(v);
    }

    pub fn get_firmware_present(&self) -> bool {
        self.firmware_present.unwrap_or(false)
    }

    fn get_firmware_present_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.firmware_present
    }

    fn mut_firmware_present_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.firmware_present
    }

    // optional bool needs_backup = 19;

    pub fn clear_needs_backup(&mut self) {
        self.needs_backup = ::std::option::Option::None;
    }

    pub fn has_needs_backup(&self) -> bool {
        self.needs_backup.is_some()
    }

    // Param is passed by value, moved
    pub fn set_needs_backup(&mut self, v: bool) {
        self.needs_backup = ::std::option::Option::Some(v);
    }

    pub fn get_needs_backup(&self) -> bool {
        self.needs_backup.unwrap_or(false)
    }

    fn get_needs_backup_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.needs_backup
    }

    fn mut_needs_backup_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.needs_backup
    }

    // optional uint32 flags = 20;

    pub fn clear_flags(&mut self) {
        self.flags = ::std::option::Option::None;
    }

    pub fn has_flags(&self) -> bool {
        self.flags.is_some()
    }

    // Param is passed by value, moved
    pub fn set_flags(&mut self, v: u32) {
        self.flags = ::std::option::Option::Some(v);
    }

    pub fn get_flags(&self) -> u32 {
        self.flags.unwrap_or(0)
    }

    fn get_flags_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.flags
    }

    fn mut_flags_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.flags
    }
}

impl ::protobuf::Message for Features {
    fn is_initialized(&self) -> bool {
        for v in &self.coins {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.vendor)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.major_version = ::std::option::Option::Some(tmp);
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.minor_version = ::std::option::Option::Some(tmp);
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.patch_version = ::std::option::Option::Some(tmp);
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.bootloader_mode = ::std::option::Option::Some(tmp);
                },
                6 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.device_id)?;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.pin_protection = ::std::option::Option::Some(tmp);
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.passphrase_protection = ::std::option::Option::Some(tmp);
                },
                9 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.language)?;
                },
                10 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.label)?;
                },
                11 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.coins)?;
                },
                12 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.initialized = ::std::option::Option::Some(tmp);
                },
                13 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.revision)?;
                },
                14 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.bootloader_hash)?;
                },
                15 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.imported = ::std::option::Option::Some(tmp);
                },
                16 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.pin_cached = ::std::option::Option::Some(tmp);
                },
                17 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.passphrase_cached = ::std::option::Option::Some(tmp);
                },
                18 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.firmware_present = ::std::option::Option::Some(tmp);
                },
                19 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.needs_backup = ::std::option::Option::Some(tmp);
                },
                20 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.flags = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.vendor.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(v) = self.major_version {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.minor_version {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.patch_version {
            my_size += ::protobuf::rt::value_size(4, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.bootloader_mode {
            my_size += 2;
        }
        if let Some(ref v) = self.device_id.as_ref() {
            my_size += ::protobuf::rt::string_size(6, &v);
        }
        if let Some(v) = self.pin_protection {
            my_size += 2;
        }
        if let Some(v) = self.passphrase_protection {
            my_size += 2;
        }
        if let Some(ref v) = self.language.as_ref() {
            my_size += ::protobuf::rt::string_size(9, &v);
        }
        if let Some(ref v) = self.label.as_ref() {
            my_size += ::protobuf::rt::string_size(10, &v);
        }
        for value in &self.coins {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        if let Some(v) = self.initialized {
            my_size += 2;
        }
        if let Some(ref v) = self.revision.as_ref() {
            my_size += ::protobuf::rt::bytes_size(13, &v);
        }
        if let Some(ref v) = self.bootloader_hash.as_ref() {
            my_size += ::protobuf::rt::bytes_size(14, &v);
        }
        if let Some(v) = self.imported {
            my_size += 2;
        }
        if let Some(v) = self.pin_cached {
            my_size += 3;
        }
        if let Some(v) = self.passphrase_cached {
            my_size += 3;
        }
        if let Some(v) = self.firmware_present {
            my_size += 3;
        }
        if let Some(v) = self.needs_backup {
            my_size += 3;
        }
        if let Some(v) = self.flags {
            my_size += ::protobuf::rt::value_size(20, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.vendor.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(v) = self.major_version {
            os.write_uint32(2, v)?;
        }
        if let Some(v) = self.minor_version {
            os.write_uint32(3, v)?;
        }
        if let Some(v) = self.patch_version {
            os.write_uint32(4, v)?;
        }
        if let Some(v) = self.bootloader_mode {
            os.write_bool(5, v)?;
        }
        if let Some(ref v) = self.device_id.as_ref() {
            os.write_string(6, &v)?;
        }
        if let Some(v) = self.pin_protection {
            os.write_bool(7, v)?;
        }
        if let Some(v) = self.passphrase_protection {
            os.write_bool(8, v)?;
        }
        if let Some(ref v) = self.language.as_ref() {
            os.write_string(9, &v)?;
        }
        if let Some(ref v) = self.label.as_ref() {
            os.write_string(10, &v)?;
        }
        for v in &self.coins {
            os.write_tag(11, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        if let Some(v) = self.initialized {
            os.write_bool(12, v)?;
        }
        if let Some(ref v) = self.revision.as_ref() {
            os.write_bytes(13, &v)?;
        }
        if let Some(ref v) = self.bootloader_hash.as_ref() {
            os.write_bytes(14, &v)?;
        }
        if let Some(v) = self.imported {
            os.write_bool(15, v)?;
        }
        if let Some(v) = self.pin_cached {
            os.write_bool(16, v)?;
        }
        if let Some(v) = self.passphrase_cached {
            os.write_bool(17, v)?;
        }
        if let Some(v) = self.firmware_present {
            os.write_bool(18, v)?;
        }
        if let Some(v) = self.needs_backup {
            os.write_bool(19, v)?;
        }
        if let Some(v) = self.flags {
            os.write_uint32(20, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Features {
    fn new() -> Features {
        Features::new()
    }

    fn descriptor_static(_: ::std::option::Option<Features>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "vendor",
                    Features::get_vendor_for_reflect,
                    Features::mut_vendor_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "major_version",
                    Features::get_major_version_for_reflect,
                    Features::mut_major_version_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "minor_version",
                    Features::get_minor_version_for_reflect,
                    Features::mut_minor_version_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "patch_version",
                    Features::get_patch_version_for_reflect,
                    Features::mut_patch_version_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "bootloader_mode",
                    Features::get_bootloader_mode_for_reflect,
                    Features::mut_bootloader_mode_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "device_id",
                    Features::get_device_id_for_reflect,
                    Features::mut_device_id_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "pin_protection",
                    Features::get_pin_protection_for_reflect,
                    Features::mut_pin_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "passphrase_protection",
                    Features::get_passphrase_protection_for_reflect,
                    Features::mut_passphrase_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "language",
                    Features::get_language_for_reflect,
                    Features::mut_language_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "label",
                    Features::get_label_for_reflect,
                    Features::mut_label_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::CoinType>>(
                    "coins",
                    Features::get_coins_for_reflect,
                    Features::mut_coins_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "initialized",
                    Features::get_initialized_for_reflect,
                    Features::mut_initialized_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "revision",
                    Features::get_revision_for_reflect,
                    Features::mut_revision_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "bootloader_hash",
                    Features::get_bootloader_hash_for_reflect,
                    Features::mut_bootloader_hash_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "imported",
                    Features::get_imported_for_reflect,
                    Features::mut_imported_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "pin_cached",
                    Features::get_pin_cached_for_reflect,
                    Features::mut_pin_cached_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "passphrase_cached",
                    Features::get_passphrase_cached_for_reflect,
                    Features::mut_passphrase_cached_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "firmware_present",
                    Features::get_firmware_present_for_reflect,
                    Features::mut_firmware_present_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "needs_backup",
                    Features::get_needs_backup_for_reflect,
                    Features::mut_needs_backup_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "flags",
                    Features::get_flags_for_reflect,
                    Features::mut_flags_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Features>(
                    "Features",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Features {
    fn clear(&mut self) {
        self.clear_vendor();
        self.clear_major_version();
        self.clear_minor_version();
        self.clear_patch_version();
        self.clear_bootloader_mode();
        self.clear_device_id();
        self.clear_pin_protection();
        self.clear_passphrase_protection();
        self.clear_language();
        self.clear_label();
        self.clear_coins();
        self.clear_initialized();
        self.clear_revision();
        self.clear_bootloader_hash();
        self.clear_imported();
        self.clear_pin_cached();
        self.clear_passphrase_cached();
        self.clear_firmware_present();
        self.clear_needs_backup();
        self.clear_flags();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Features {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Features {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ClearSession {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ClearSession {}

impl ClearSession {
    pub fn new() -> ClearSession {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ClearSession {
        static mut instance: ::protobuf::lazy::Lazy<ClearSession> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ClearSession,
        };
        unsafe {
            instance.get(ClearSession::new)
        }
    }
}

impl ::protobuf::Message for ClearSession {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ClearSession {
    fn new() -> ClearSession {
        ClearSession::new()
    }

    fn descriptor_static(_: ::std::option::Option<ClearSession>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<ClearSession>(
                    "ClearSession",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ClearSession {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ClearSession {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ClearSession {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ApplySettings {
    // message fields
    language: ::protobuf::SingularField<::std::string::String>,
    label: ::protobuf::SingularField<::std::string::String>,
    use_passphrase: ::std::option::Option<bool>,
    homescreen: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ApplySettings {}

impl ApplySettings {
    pub fn new() -> ApplySettings {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ApplySettings {
        static mut instance: ::protobuf::lazy::Lazy<ApplySettings> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ApplySettings,
        };
        unsafe {
            instance.get(ApplySettings::new)
        }
    }

    // optional string language = 1;

    pub fn clear_language(&mut self) {
        self.language.clear();
    }

    pub fn has_language(&self) -> bool {
        self.language.is_some()
    }

    // Param is passed by value, moved
    pub fn set_language(&mut self, v: ::std::string::String) {
        self.language = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_language(&mut self) -> &mut ::std::string::String {
        if self.language.is_none() {
            self.language.set_default();
        }
        self.language.as_mut().unwrap()
    }

    // Take field
    pub fn take_language(&mut self) -> ::std::string::String {
        self.language.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_language(&self) -> &str {
        match self.language.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_language_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.language
    }

    fn mut_language_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.language
    }

    // optional string label = 2;

    pub fn clear_label(&mut self) {
        self.label.clear();
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    // Param is passed by value, moved
    pub fn set_label(&mut self, v: ::std::string::String) {
        self.label = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_label(&mut self) -> &mut ::std::string::String {
        if self.label.is_none() {
            self.label.set_default();
        }
        self.label.as_mut().unwrap()
    }

    // Take field
    pub fn take_label(&mut self) -> ::std::string::String {
        self.label.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_label(&self) -> &str {
        match self.label.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_label_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.label
    }

    fn mut_label_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.label
    }

    // optional bool use_passphrase = 3;

    pub fn clear_use_passphrase(&mut self) {
        self.use_passphrase = ::std::option::Option::None;
    }

    pub fn has_use_passphrase(&self) -> bool {
        self.use_passphrase.is_some()
    }

    // Param is passed by value, moved
    pub fn set_use_passphrase(&mut self, v: bool) {
        self.use_passphrase = ::std::option::Option::Some(v);
    }

    pub fn get_use_passphrase(&self) -> bool {
        self.use_passphrase.unwrap_or(false)
    }

    fn get_use_passphrase_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.use_passphrase
    }

    fn mut_use_passphrase_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.use_passphrase
    }

    // optional bytes homescreen = 4;

    pub fn clear_homescreen(&mut self) {
        self.homescreen.clear();
    }

    pub fn has_homescreen(&self) -> bool {
        self.homescreen.is_some()
    }

    // Param is passed by value, moved
    pub fn set_homescreen(&mut self, v: ::std::vec::Vec<u8>) {
        self.homescreen = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_homescreen(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.homescreen.is_none() {
            self.homescreen.set_default();
        }
        self.homescreen.as_mut().unwrap()
    }

    // Take field
    pub fn take_homescreen(&mut self) -> ::std::vec::Vec<u8> {
        self.homescreen.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_homescreen(&self) -> &[u8] {
        match self.homescreen.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_homescreen_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.homescreen
    }

    fn mut_homescreen_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.homescreen
    }
}

impl ::protobuf::Message for ApplySettings {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.language)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.label)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.use_passphrase = ::std::option::Option::Some(tmp);
                },
                4 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.homescreen)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.language.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(ref v) = self.label.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        if let Some(v) = self.use_passphrase {
            my_size += 2;
        }
        if let Some(ref v) = self.homescreen.as_ref() {
            my_size += ::protobuf::rt::bytes_size(4, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.language.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(ref v) = self.label.as_ref() {
            os.write_string(2, &v)?;
        }
        if let Some(v) = self.use_passphrase {
            os.write_bool(3, v)?;
        }
        if let Some(ref v) = self.homescreen.as_ref() {
            os.write_bytes(4, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ApplySettings {
    fn new() -> ApplySettings {
        ApplySettings::new()
    }

    fn descriptor_static(_: ::std::option::Option<ApplySettings>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "language",
                    ApplySettings::get_language_for_reflect,
                    ApplySettings::mut_language_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "label",
                    ApplySettings::get_label_for_reflect,
                    ApplySettings::mut_label_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "use_passphrase",
                    ApplySettings::get_use_passphrase_for_reflect,
                    ApplySettings::mut_use_passphrase_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "homescreen",
                    ApplySettings::get_homescreen_for_reflect,
                    ApplySettings::mut_homescreen_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ApplySettings>(
                    "ApplySettings",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ApplySettings {
    fn clear(&mut self) {
        self.clear_language();
        self.clear_label();
        self.clear_use_passphrase();
        self.clear_homescreen();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ApplySettings {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ApplySettings {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ApplyFlags {
    // message fields
    flags: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ApplyFlags {}

impl ApplyFlags {
    pub fn new() -> ApplyFlags {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ApplyFlags {
        static mut instance: ::protobuf::lazy::Lazy<ApplyFlags> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ApplyFlags,
        };
        unsafe {
            instance.get(ApplyFlags::new)
        }
    }

    // optional uint32 flags = 1;

    pub fn clear_flags(&mut self) {
        self.flags = ::std::option::Option::None;
    }

    pub fn has_flags(&self) -> bool {
        self.flags.is_some()
    }

    // Param is passed by value, moved
    pub fn set_flags(&mut self, v: u32) {
        self.flags = ::std::option::Option::Some(v);
    }

    pub fn get_flags(&self) -> u32 {
        self.flags.unwrap_or(0)
    }

    fn get_flags_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.flags
    }

    fn mut_flags_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.flags
    }
}

impl ::protobuf::Message for ApplyFlags {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.flags = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.flags {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.flags {
            os.write_uint32(1, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ApplyFlags {
    fn new() -> ApplyFlags {
        ApplyFlags::new()
    }

    fn descriptor_static(_: ::std::option::Option<ApplyFlags>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "flags",
                    ApplyFlags::get_flags_for_reflect,
                    ApplyFlags::mut_flags_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ApplyFlags>(
                    "ApplyFlags",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ApplyFlags {
    fn clear(&mut self) {
        self.clear_flags();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ApplyFlags {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ApplyFlags {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ChangePin {
    // message fields
    remove: ::std::option::Option<bool>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ChangePin {}

impl ChangePin {
    pub fn new() -> ChangePin {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ChangePin {
        static mut instance: ::protobuf::lazy::Lazy<ChangePin> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ChangePin,
        };
        unsafe {
            instance.get(ChangePin::new)
        }
    }

    // optional bool remove = 1;

    pub fn clear_remove(&mut self) {
        self.remove = ::std::option::Option::None;
    }

    pub fn has_remove(&self) -> bool {
        self.remove.is_some()
    }

    // Param is passed by value, moved
    pub fn set_remove(&mut self, v: bool) {
        self.remove = ::std::option::Option::Some(v);
    }

    pub fn get_remove(&self) -> bool {
        self.remove.unwrap_or(false)
    }

    fn get_remove_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.remove
    }

    fn mut_remove_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.remove
    }
}

impl ::protobuf::Message for ChangePin {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.remove = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.remove {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.remove {
            os.write_bool(1, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ChangePin {
    fn new() -> ChangePin {
        ChangePin::new()
    }

    fn descriptor_static(_: ::std::option::Option<ChangePin>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "remove",
                    ChangePin::get_remove_for_reflect,
                    ChangePin::mut_remove_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ChangePin>(
                    "ChangePin",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ChangePin {
    fn clear(&mut self) {
        self.clear_remove();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ChangePin {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ChangePin {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Ping {
    // message fields
    message: ::protobuf::SingularField<::std::string::String>,
    button_protection: ::std::option::Option<bool>,
    pin_protection: ::std::option::Option<bool>,
    passphrase_protection: ::std::option::Option<bool>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Ping {}

impl Ping {
    pub fn new() -> Ping {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Ping {
        static mut instance: ::protobuf::lazy::Lazy<Ping> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Ping,
        };
        unsafe {
            instance.get(Ping::new)
        }
    }

    // optional string message = 1;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::string::String) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::string::String {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::string::String {
        self.message.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_message(&self) -> &str {
        match self.message.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.message
    }

    // optional bool button_protection = 2;

    pub fn clear_button_protection(&mut self) {
        self.button_protection = ::std::option::Option::None;
    }

    pub fn has_button_protection(&self) -> bool {
        self.button_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_button_protection(&mut self, v: bool) {
        self.button_protection = ::std::option::Option::Some(v);
    }

    pub fn get_button_protection(&self) -> bool {
        self.button_protection.unwrap_or(false)
    }

    fn get_button_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.button_protection
    }

    fn mut_button_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.button_protection
    }

    // optional bool pin_protection = 3;

    pub fn clear_pin_protection(&mut self) {
        self.pin_protection = ::std::option::Option::None;
    }

    pub fn has_pin_protection(&self) -> bool {
        self.pin_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pin_protection(&mut self, v: bool) {
        self.pin_protection = ::std::option::Option::Some(v);
    }

    pub fn get_pin_protection(&self) -> bool {
        self.pin_protection.unwrap_or(false)
    }

    fn get_pin_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.pin_protection
    }

    fn mut_pin_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.pin_protection
    }

    // optional bool passphrase_protection = 4;

    pub fn clear_passphrase_protection(&mut self) {
        self.passphrase_protection = ::std::option::Option::None;
    }

    pub fn has_passphrase_protection(&self) -> bool {
        self.passphrase_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_passphrase_protection(&mut self, v: bool) {
        self.passphrase_protection = ::std::option::Option::Some(v);
    }

    pub fn get_passphrase_protection(&self) -> bool {
        self.passphrase_protection.unwrap_or(false)
    }

    fn get_passphrase_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.passphrase_protection
    }

    fn mut_passphrase_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.passphrase_protection
    }
}

impl ::protobuf::Message for Ping {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.message)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.button_protection = ::std::option::Option::Some(tmp);
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.pin_protection = ::std::option::Option::Some(tmp);
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.passphrase_protection = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(v) = self.button_protection {
            my_size += 2;
        }
        if let Some(v) = self.pin_protection {
            my_size += 2;
        }
        if let Some(v) = self.passphrase_protection {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.message.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(v) = self.button_protection {
            os.write_bool(2, v)?;
        }
        if let Some(v) = self.pin_protection {
            os.write_bool(3, v)?;
        }
        if let Some(v) = self.passphrase_protection {
            os.write_bool(4, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Ping {
    fn new() -> Ping {
        Ping::new()
    }

    fn descriptor_static(_: ::std::option::Option<Ping>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "message",
                    Ping::get_message_for_reflect,
                    Ping::mut_message_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "button_protection",
                    Ping::get_button_protection_for_reflect,
                    Ping::mut_button_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "pin_protection",
                    Ping::get_pin_protection_for_reflect,
                    Ping::mut_pin_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "passphrase_protection",
                    Ping::get_passphrase_protection_for_reflect,
                    Ping::mut_passphrase_protection_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Ping>(
                    "Ping",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Ping {
    fn clear(&mut self) {
        self.clear_message();
        self.clear_button_protection();
        self.clear_pin_protection();
        self.clear_passphrase_protection();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Ping {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Ping {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Success {
    // message fields
    message: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Success {}

impl Success {
    pub fn new() -> Success {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Success {
        static mut instance: ::protobuf::lazy::Lazy<Success> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Success,
        };
        unsafe {
            instance.get(Success::new)
        }
    }

    // optional string message = 1;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::string::String) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::string::String {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::string::String {
        self.message.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_message(&self) -> &str {
        match self.message.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.message
    }
}

impl ::protobuf::Message for Success {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.message)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.message.as_ref() {
            os.write_string(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Success {
    fn new() -> Success {
        Success::new()
    }

    fn descriptor_static(_: ::std::option::Option<Success>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "message",
                    Success::get_message_for_reflect,
                    Success::mut_message_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Success>(
                    "Success",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Success {
    fn clear(&mut self) {
        self.clear_message();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Success {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Success {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Failure {
    // message fields
    code: ::std::option::Option<super::types::FailureType>,
    message: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Failure {}

impl Failure {
    pub fn new() -> Failure {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Failure {
        static mut instance: ::protobuf::lazy::Lazy<Failure> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Failure,
        };
        unsafe {
            instance.get(Failure::new)
        }
    }

    // optional .FailureType code = 1;

    pub fn clear_code(&mut self) {
        self.code = ::std::option::Option::None;
    }

    pub fn has_code(&self) -> bool {
        self.code.is_some()
    }

    // Param is passed by value, moved
    pub fn set_code(&mut self, v: super::types::FailureType) {
        self.code = ::std::option::Option::Some(v);
    }

    pub fn get_code(&self) -> super::types::FailureType {
        self.code.unwrap_or(super::types::FailureType::Failure_UnexpectedMessage)
    }

    fn get_code_for_reflect(&self) -> &::std::option::Option<super::types::FailureType> {
        &self.code
    }

    fn mut_code_for_reflect(&mut self) -> &mut ::std::option::Option<super::types::FailureType> {
        &mut self.code
    }

    // optional string message = 2;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::string::String) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::string::String {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::string::String {
        self.message.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_message(&self) -> &str {
        match self.message.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.message
    }
}

impl ::protobuf::Message for Failure {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.code = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.message)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.code {
            my_size += ::protobuf::rt::enum_size(1, v);
        }
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.code {
            os.write_enum(1, v.value())?;
        }
        if let Some(ref v) = self.message.as_ref() {
            os.write_string(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Failure {
    fn new() -> Failure {
        Failure::new()
    }

    fn descriptor_static(_: ::std::option::Option<Failure>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<super::types::FailureType>>(
                    "code",
                    Failure::get_code_for_reflect,
                    Failure::mut_code_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "message",
                    Failure::get_message_for_reflect,
                    Failure::mut_message_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Failure>(
                    "Failure",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Failure {
    fn clear(&mut self) {
        self.clear_code();
        self.clear_message();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Failure {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ButtonRequest {
    // message fields
    code: ::std::option::Option<super::types::ButtonRequestType>,
    data: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ButtonRequest {}

impl ButtonRequest {
    pub fn new() -> ButtonRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ButtonRequest {
        static mut instance: ::protobuf::lazy::Lazy<ButtonRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ButtonRequest,
        };
        unsafe {
            instance.get(ButtonRequest::new)
        }
    }

    // optional .ButtonRequestType code = 1;

    pub fn clear_code(&mut self) {
        self.code = ::std::option::Option::None;
    }

    pub fn has_code(&self) -> bool {
        self.code.is_some()
    }

    // Param is passed by value, moved
    pub fn set_code(&mut self, v: super::types::ButtonRequestType) {
        self.code = ::std::option::Option::Some(v);
    }

    pub fn get_code(&self) -> super::types::ButtonRequestType {
        self.code.unwrap_or(super::types::ButtonRequestType::ButtonRequest_Other)
    }

    fn get_code_for_reflect(&self) -> &::std::option::Option<super::types::ButtonRequestType> {
        &self.code
    }

    fn mut_code_for_reflect(&mut self) -> &mut ::std::option::Option<super::types::ButtonRequestType> {
        &mut self.code
    }

    // optional string data = 2;

    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    // Param is passed by value, moved
    pub fn set_data(&mut self, v: ::std::string::String) {
        self.data = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_data(&mut self) -> &mut ::std::string::String {
        if self.data.is_none() {
            self.data.set_default();
        }
        self.data.as_mut().unwrap()
    }

    // Take field
    pub fn take_data(&mut self) -> ::std::string::String {
        self.data.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_data(&self) -> &str {
        match self.data.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_data_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.data
    }

    fn mut_data_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.data
    }
}

impl ::protobuf::Message for ButtonRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.code = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.data)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.code {
            my_size += ::protobuf::rt::enum_size(1, v);
        }
        if let Some(ref v) = self.data.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.code {
            os.write_enum(1, v.value())?;
        }
        if let Some(ref v) = self.data.as_ref() {
            os.write_string(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ButtonRequest {
    fn new() -> ButtonRequest {
        ButtonRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<ButtonRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<super::types::ButtonRequestType>>(
                    "code",
                    ButtonRequest::get_code_for_reflect,
                    ButtonRequest::mut_code_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "data",
                    ButtonRequest::get_data_for_reflect,
                    ButtonRequest::mut_data_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ButtonRequest>(
                    "ButtonRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ButtonRequest {
    fn clear(&mut self) {
        self.clear_code();
        self.clear_data();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ButtonRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ButtonRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ButtonAck {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ButtonAck {}

impl ButtonAck {
    pub fn new() -> ButtonAck {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ButtonAck {
        static mut instance: ::protobuf::lazy::Lazy<ButtonAck> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ButtonAck,
        };
        unsafe {
            instance.get(ButtonAck::new)
        }
    }
}

impl ::protobuf::Message for ButtonAck {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ButtonAck {
    fn new() -> ButtonAck {
        ButtonAck::new()
    }

    fn descriptor_static(_: ::std::option::Option<ButtonAck>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<ButtonAck>(
                    "ButtonAck",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ButtonAck {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ButtonAck {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ButtonAck {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PinMatrixRequest {
    // message fields
    field_type: ::std::option::Option<super::types::PinMatrixRequestType>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for PinMatrixRequest {}

impl PinMatrixRequest {
    pub fn new() -> PinMatrixRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static PinMatrixRequest {
        static mut instance: ::protobuf::lazy::Lazy<PinMatrixRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const PinMatrixRequest,
        };
        unsafe {
            instance.get(PinMatrixRequest::new)
        }
    }

    // optional .PinMatrixRequestType type = 1;

    pub fn clear_field_type(&mut self) {
        self.field_type = ::std::option::Option::None;
    }

    pub fn has_field_type(&self) -> bool {
        self.field_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_field_type(&mut self, v: super::types::PinMatrixRequestType) {
        self.field_type = ::std::option::Option::Some(v);
    }

    pub fn get_field_type(&self) -> super::types::PinMatrixRequestType {
        self.field_type.unwrap_or(super::types::PinMatrixRequestType::PinMatrixRequestType_Current)
    }

    fn get_field_type_for_reflect(&self) -> &::std::option::Option<super::types::PinMatrixRequestType> {
        &self.field_type
    }

    fn mut_field_type_for_reflect(&mut self) -> &mut ::std::option::Option<super::types::PinMatrixRequestType> {
        &mut self.field_type
    }
}

impl ::protobuf::Message for PinMatrixRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.field_type = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.field_type {
            my_size += ::protobuf::rt::enum_size(1, v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.field_type {
            os.write_enum(1, v.value())?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for PinMatrixRequest {
    fn new() -> PinMatrixRequest {
        PinMatrixRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<PinMatrixRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<super::types::PinMatrixRequestType>>(
                    "type",
                    PinMatrixRequest::get_field_type_for_reflect,
                    PinMatrixRequest::mut_field_type_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<PinMatrixRequest>(
                    "PinMatrixRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for PinMatrixRequest {
    fn clear(&mut self) {
        self.clear_field_type();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PinMatrixRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PinMatrixRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PinMatrixAck {
    // message fields
    pin: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for PinMatrixAck {}

impl PinMatrixAck {
    pub fn new() -> PinMatrixAck {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static PinMatrixAck {
        static mut instance: ::protobuf::lazy::Lazy<PinMatrixAck> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const PinMatrixAck,
        };
        unsafe {
            instance.get(PinMatrixAck::new)
        }
    }

    // required string pin = 1;

    pub fn clear_pin(&mut self) {
        self.pin.clear();
    }

    pub fn has_pin(&self) -> bool {
        self.pin.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pin(&mut self, v: ::std::string::String) {
        self.pin = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_pin(&mut self) -> &mut ::std::string::String {
        if self.pin.is_none() {
            self.pin.set_default();
        }
        self.pin.as_mut().unwrap()
    }

    // Take field
    pub fn take_pin(&mut self) -> ::std::string::String {
        self.pin.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_pin(&self) -> &str {
        match self.pin.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_pin_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.pin
    }

    fn mut_pin_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.pin
    }
}

impl ::protobuf::Message for PinMatrixAck {
    fn is_initialized(&self) -> bool {
        if self.pin.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.pin)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.pin.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.pin.as_ref() {
            os.write_string(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for PinMatrixAck {
    fn new() -> PinMatrixAck {
        PinMatrixAck::new()
    }

    fn descriptor_static(_: ::std::option::Option<PinMatrixAck>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "pin",
                    PinMatrixAck::get_pin_for_reflect,
                    PinMatrixAck::mut_pin_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<PinMatrixAck>(
                    "PinMatrixAck",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for PinMatrixAck {
    fn clear(&mut self) {
        self.clear_pin();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PinMatrixAck {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PinMatrixAck {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Cancel {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Cancel {}

impl Cancel {
    pub fn new() -> Cancel {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Cancel {
        static mut instance: ::protobuf::lazy::Lazy<Cancel> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Cancel,
        };
        unsafe {
            instance.get(Cancel::new)
        }
    }
}

impl ::protobuf::Message for Cancel {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Cancel {
    fn new() -> Cancel {
        Cancel::new()
    }

    fn descriptor_static(_: ::std::option::Option<Cancel>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<Cancel>(
                    "Cancel",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Cancel {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Cancel {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Cancel {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PassphraseRequest {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for PassphraseRequest {}

impl PassphraseRequest {
    pub fn new() -> PassphraseRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static PassphraseRequest {
        static mut instance: ::protobuf::lazy::Lazy<PassphraseRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const PassphraseRequest,
        };
        unsafe {
            instance.get(PassphraseRequest::new)
        }
    }
}

impl ::protobuf::Message for PassphraseRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for PassphraseRequest {
    fn new() -> PassphraseRequest {
        PassphraseRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<PassphraseRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<PassphraseRequest>(
                    "PassphraseRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for PassphraseRequest {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PassphraseRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PassphraseRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PassphraseAck {
    // message fields
    passphrase: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for PassphraseAck {}

impl PassphraseAck {
    pub fn new() -> PassphraseAck {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static PassphraseAck {
        static mut instance: ::protobuf::lazy::Lazy<PassphraseAck> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const PassphraseAck,
        };
        unsafe {
            instance.get(PassphraseAck::new)
        }
    }

    // required string passphrase = 1;

    pub fn clear_passphrase(&mut self) {
        self.passphrase.clear();
    }

    pub fn has_passphrase(&self) -> bool {
        self.passphrase.is_some()
    }

    // Param is passed by value, moved
    pub fn set_passphrase(&mut self, v: ::std::string::String) {
        self.passphrase = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_passphrase(&mut self) -> &mut ::std::string::String {
        if self.passphrase.is_none() {
            self.passphrase.set_default();
        }
        self.passphrase.as_mut().unwrap()
    }

    // Take field
    pub fn take_passphrase(&mut self) -> ::std::string::String {
        self.passphrase.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_passphrase(&self) -> &str {
        match self.passphrase.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_passphrase_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.passphrase
    }

    fn mut_passphrase_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.passphrase
    }
}

impl ::protobuf::Message for PassphraseAck {
    fn is_initialized(&self) -> bool {
        if self.passphrase.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.passphrase)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.passphrase.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.passphrase.as_ref() {
            os.write_string(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for PassphraseAck {
    fn new() -> PassphraseAck {
        PassphraseAck::new()
    }

    fn descriptor_static(_: ::std::option::Option<PassphraseAck>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "passphrase",
                    PassphraseAck::get_passphrase_for_reflect,
                    PassphraseAck::mut_passphrase_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<PassphraseAck>(
                    "PassphraseAck",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for PassphraseAck {
    fn clear(&mut self) {
        self.clear_passphrase();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PassphraseAck {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PassphraseAck {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct GetEntropy {
    // message fields
    size: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for GetEntropy {}

impl GetEntropy {
    pub fn new() -> GetEntropy {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static GetEntropy {
        static mut instance: ::protobuf::lazy::Lazy<GetEntropy> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const GetEntropy,
        };
        unsafe {
            instance.get(GetEntropy::new)
        }
    }

    // required uint32 size = 1;

    pub fn clear_size(&mut self) {
        self.size = ::std::option::Option::None;
    }

    pub fn has_size(&self) -> bool {
        self.size.is_some()
    }

    // Param is passed by value, moved
    pub fn set_size(&mut self, v: u32) {
        self.size = ::std::option::Option::Some(v);
    }

    pub fn get_size(&self) -> u32 {
        self.size.unwrap_or(0)
    }

    fn get_size_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.size
    }

    fn mut_size_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.size
    }
}

impl ::protobuf::Message for GetEntropy {
    fn is_initialized(&self) -> bool {
        if self.size.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.size = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.size {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.size {
            os.write_uint32(1, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for GetEntropy {
    fn new() -> GetEntropy {
        GetEntropy::new()
    }

    fn descriptor_static(_: ::std::option::Option<GetEntropy>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "size",
                    GetEntropy::get_size_for_reflect,
                    GetEntropy::mut_size_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<GetEntropy>(
                    "GetEntropy",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for GetEntropy {
    fn clear(&mut self) {
        self.clear_size();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for GetEntropy {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for GetEntropy {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Entropy {
    // message fields
    entropy: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Entropy {}

impl Entropy {
    pub fn new() -> Entropy {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Entropy {
        static mut instance: ::protobuf::lazy::Lazy<Entropy> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Entropy,
        };
        unsafe {
            instance.get(Entropy::new)
        }
    }

    // required bytes entropy = 1;

    pub fn clear_entropy(&mut self) {
        self.entropy.clear();
    }

    pub fn has_entropy(&self) -> bool {
        self.entropy.is_some()
    }

    // Param is passed by value, moved
    pub fn set_entropy(&mut self, v: ::std::vec::Vec<u8>) {
        self.entropy = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_entropy(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.entropy.is_none() {
            self.entropy.set_default();
        }
        self.entropy.as_mut().unwrap()
    }

    // Take field
    pub fn take_entropy(&mut self) -> ::std::vec::Vec<u8> {
        self.entropy.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_entropy(&self) -> &[u8] {
        match self.entropy.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_entropy_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.entropy
    }

    fn mut_entropy_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.entropy
    }
}

impl ::protobuf::Message for Entropy {
    fn is_initialized(&self) -> bool {
        if self.entropy.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.entropy)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.entropy.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.entropy.as_ref() {
            os.write_bytes(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Entropy {
    fn new() -> Entropy {
        Entropy::new()
    }

    fn descriptor_static(_: ::std::option::Option<Entropy>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "entropy",
                    Entropy::get_entropy_for_reflect,
                    Entropy::mut_entropy_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Entropy>(
                    "Entropy",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Entropy {
    fn clear(&mut self) {
        self.clear_entropy();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Entropy {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Entropy {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct GetPublicKey {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    ecdsa_curve_name: ::protobuf::SingularField<::std::string::String>,
    show_display: ::std::option::Option<bool>,
    coin_name: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for GetPublicKey {}

impl GetPublicKey {
    pub fn new() -> GetPublicKey {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static GetPublicKey {
        static mut instance: ::protobuf::lazy::Lazy<GetPublicKey> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const GetPublicKey,
        };
        unsafe {
            instance.get(GetPublicKey::new)
        }
    }

    // repeated uint32 address_n = 1;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // optional string ecdsa_curve_name = 2;

    pub fn clear_ecdsa_curve_name(&mut self) {
        self.ecdsa_curve_name.clear();
    }

    pub fn has_ecdsa_curve_name(&self) -> bool {
        self.ecdsa_curve_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_ecdsa_curve_name(&mut self, v: ::std::string::String) {
        self.ecdsa_curve_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_ecdsa_curve_name(&mut self) -> &mut ::std::string::String {
        if self.ecdsa_curve_name.is_none() {
            self.ecdsa_curve_name.set_default();
        }
        self.ecdsa_curve_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_ecdsa_curve_name(&mut self) -> ::std::string::String {
        self.ecdsa_curve_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_ecdsa_curve_name(&self) -> &str {
        match self.ecdsa_curve_name.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_ecdsa_curve_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.ecdsa_curve_name
    }

    fn mut_ecdsa_curve_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.ecdsa_curve_name
    }

    // optional bool show_display = 3;

    pub fn clear_show_display(&mut self) {
        self.show_display = ::std::option::Option::None;
    }

    pub fn has_show_display(&self) -> bool {
        self.show_display.is_some()
    }

    // Param is passed by value, moved
    pub fn set_show_display(&mut self, v: bool) {
        self.show_display = ::std::option::Option::Some(v);
    }

    pub fn get_show_display(&self) -> bool {
        self.show_display.unwrap_or(false)
    }

    fn get_show_display_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.show_display
    }

    fn mut_show_display_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.show_display
    }

    // optional string coin_name = 4;

    pub fn clear_coin_name(&mut self) {
        self.coin_name.clear();
    }

    pub fn has_coin_name(&self) -> bool {
        self.coin_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_name(&mut self, v: ::std::string::String) {
        self.coin_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_name(&mut self) -> &mut ::std::string::String {
        if self.coin_name.is_none() {
            self.coin_name.set_default();
        }
        self.coin_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_name(&mut self) -> ::std::string::String {
        self.coin_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_name(&self) -> &str {
        match self.coin_name.as_ref() {
            Some(v) => &v,
            None => "Bitcoin",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }
}

impl ::protobuf::Message for GetPublicKey {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.ecdsa_curve_name)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.show_display = ::std::option::Option::Some(tmp);
                },
                4 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(ref v) = self.ecdsa_curve_name.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        if let Some(v) = self.show_display {
            my_size += 2;
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(4, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(ref v) = self.ecdsa_curve_name.as_ref() {
            os.write_string(2, &v)?;
        }
        if let Some(v) = self.show_display {
            os.write_bool(3, v)?;
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(4, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for GetPublicKey {
    fn new() -> GetPublicKey {
        GetPublicKey::new()
    }

    fn descriptor_static(_: ::std::option::Option<GetPublicKey>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    GetPublicKey::get_address_n_for_reflect,
                    GetPublicKey::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "ecdsa_curve_name",
                    GetPublicKey::get_ecdsa_curve_name_for_reflect,
                    GetPublicKey::mut_ecdsa_curve_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "show_display",
                    GetPublicKey::get_show_display_for_reflect,
                    GetPublicKey::mut_show_display_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    GetPublicKey::get_coin_name_for_reflect,
                    GetPublicKey::mut_coin_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<GetPublicKey>(
                    "GetPublicKey",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for GetPublicKey {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_ecdsa_curve_name();
        self.clear_show_display();
        self.clear_coin_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for GetPublicKey {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for GetPublicKey {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PublicKey {
    // message fields
    node: ::protobuf::SingularPtrField<super::types::HDNodeType>,
    xpub: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for PublicKey {}

impl PublicKey {
    pub fn new() -> PublicKey {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static PublicKey {
        static mut instance: ::protobuf::lazy::Lazy<PublicKey> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const PublicKey,
        };
        unsafe {
            instance.get(PublicKey::new)
        }
    }

    // required .HDNodeType node = 1;

    pub fn clear_node(&mut self) {
        self.node.clear();
    }

    pub fn has_node(&self) -> bool {
        self.node.is_some()
    }

    // Param is passed by value, moved
    pub fn set_node(&mut self, v: super::types::HDNodeType) {
        self.node = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_node(&mut self) -> &mut super::types::HDNodeType {
        if self.node.is_none() {
            self.node.set_default();
        }
        self.node.as_mut().unwrap()
    }

    // Take field
    pub fn take_node(&mut self) -> super::types::HDNodeType {
        self.node.take().unwrap_or_else(|| super::types::HDNodeType::new())
    }

    pub fn get_node(&self) -> &super::types::HDNodeType {
        self.node.as_ref().unwrap_or_else(|| super::types::HDNodeType::default_instance())
    }

    fn get_node_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::HDNodeType> {
        &self.node
    }

    fn mut_node_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::HDNodeType> {
        &mut self.node
    }

    // optional string xpub = 2;

    pub fn clear_xpub(&mut self) {
        self.xpub.clear();
    }

    pub fn has_xpub(&self) -> bool {
        self.xpub.is_some()
    }

    // Param is passed by value, moved
    pub fn set_xpub(&mut self, v: ::std::string::String) {
        self.xpub = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_xpub(&mut self) -> &mut ::std::string::String {
        if self.xpub.is_none() {
            self.xpub.set_default();
        }
        self.xpub.as_mut().unwrap()
    }

    // Take field
    pub fn take_xpub(&mut self) -> ::std::string::String {
        self.xpub.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_xpub(&self) -> &str {
        match self.xpub.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_xpub_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.xpub
    }

    fn mut_xpub_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.xpub
    }
}

impl ::protobuf::Message for PublicKey {
    fn is_initialized(&self) -> bool {
        if self.node.is_none() {
            return false;
        }
        for v in &self.node {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.node)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.xpub)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.node.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.xpub.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.node.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.xpub.as_ref() {
            os.write_string(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for PublicKey {
    fn new() -> PublicKey {
        PublicKey::new()
    }

    fn descriptor_static(_: ::std::option::Option<PublicKey>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::HDNodeType>>(
                    "node",
                    PublicKey::get_node_for_reflect,
                    PublicKey::mut_node_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "xpub",
                    PublicKey::get_xpub_for_reflect,
                    PublicKey::mut_xpub_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<PublicKey>(
                    "PublicKey",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for PublicKey {
    fn clear(&mut self) {
        self.clear_node();
        self.clear_xpub();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PublicKey {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct GetAddress {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    coin_name: ::protobuf::SingularField<::std::string::String>,
    show_display: ::std::option::Option<bool>,
    multisig: ::protobuf::SingularPtrField<super::types::MultisigRedeemScriptType>,
    script_type: ::std::option::Option<super::types::InputScriptType>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for GetAddress {}

impl GetAddress {
    pub fn new() -> GetAddress {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static GetAddress {
        static mut instance: ::protobuf::lazy::Lazy<GetAddress> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const GetAddress,
        };
        unsafe {
            instance.get(GetAddress::new)
        }
    }

    // repeated uint32 address_n = 1;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // optional string coin_name = 2;

    pub fn clear_coin_name(&mut self) {
        self.coin_name.clear();
    }

    pub fn has_coin_name(&self) -> bool {
        self.coin_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_name(&mut self, v: ::std::string::String) {
        self.coin_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_name(&mut self) -> &mut ::std::string::String {
        if self.coin_name.is_none() {
            self.coin_name.set_default();
        }
        self.coin_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_name(&mut self) -> ::std::string::String {
        self.coin_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_name(&self) -> &str {
        match self.coin_name.as_ref() {
            Some(v) => &v,
            None => "Bitcoin",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }

    // optional bool show_display = 3;

    pub fn clear_show_display(&mut self) {
        self.show_display = ::std::option::Option::None;
    }

    pub fn has_show_display(&self) -> bool {
        self.show_display.is_some()
    }

    // Param is passed by value, moved
    pub fn set_show_display(&mut self, v: bool) {
        self.show_display = ::std::option::Option::Some(v);
    }

    pub fn get_show_display(&self) -> bool {
        self.show_display.unwrap_or(false)
    }

    fn get_show_display_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.show_display
    }

    fn mut_show_display_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.show_display
    }

    // optional .MultisigRedeemScriptType multisig = 4;

    pub fn clear_multisig(&mut self) {
        self.multisig.clear();
    }

    pub fn has_multisig(&self) -> bool {
        self.multisig.is_some()
    }

    // Param is passed by value, moved
    pub fn set_multisig(&mut self, v: super::types::MultisigRedeemScriptType) {
        self.multisig = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_multisig(&mut self) -> &mut super::types::MultisigRedeemScriptType {
        if self.multisig.is_none() {
            self.multisig.set_default();
        }
        self.multisig.as_mut().unwrap()
    }

    // Take field
    pub fn take_multisig(&mut self) -> super::types::MultisigRedeemScriptType {
        self.multisig.take().unwrap_or_else(|| super::types::MultisigRedeemScriptType::new())
    }

    pub fn get_multisig(&self) -> &super::types::MultisigRedeemScriptType {
        self.multisig.as_ref().unwrap_or_else(|| super::types::MultisigRedeemScriptType::default_instance())
    }

    fn get_multisig_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::MultisigRedeemScriptType> {
        &self.multisig
    }

    fn mut_multisig_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::MultisigRedeemScriptType> {
        &mut self.multisig
    }

    // optional .InputScriptType script_type = 5;

    pub fn clear_script_type(&mut self) {
        self.script_type = ::std::option::Option::None;
    }

    pub fn has_script_type(&self) -> bool {
        self.script_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_script_type(&mut self, v: super::types::InputScriptType) {
        self.script_type = ::std::option::Option::Some(v);
    }

    pub fn get_script_type(&self) -> super::types::InputScriptType {
        self.script_type.unwrap_or(super::types::InputScriptType::SPENDADDRESS)
    }

    fn get_script_type_for_reflect(&self) -> &::std::option::Option<super::types::InputScriptType> {
        &self.script_type
    }

    fn mut_script_type_for_reflect(&mut self) -> &mut ::std::option::Option<super::types::InputScriptType> {
        &mut self.script_type
    }
}

impl ::protobuf::Message for GetAddress {
    fn is_initialized(&self) -> bool {
        for v in &self.multisig {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.show_display = ::std::option::Option::Some(tmp);
                },
                4 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.multisig)?;
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.script_type = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        if let Some(v) = self.show_display {
            my_size += 2;
        }
        if let Some(ref v) = self.multisig.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(v) = self.script_type {
            my_size += ::protobuf::rt::enum_size(5, v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(2, &v)?;
        }
        if let Some(v) = self.show_display {
            os.write_bool(3, v)?;
        }
        if let Some(ref v) = self.multisig.as_ref() {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(v) = self.script_type {
            os.write_enum(5, v.value())?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for GetAddress {
    fn new() -> GetAddress {
        GetAddress::new()
    }

    fn descriptor_static(_: ::std::option::Option<GetAddress>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    GetAddress::get_address_n_for_reflect,
                    GetAddress::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    GetAddress::get_coin_name_for_reflect,
                    GetAddress::mut_coin_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "show_display",
                    GetAddress::get_show_display_for_reflect,
                    GetAddress::mut_show_display_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::MultisigRedeemScriptType>>(
                    "multisig",
                    GetAddress::get_multisig_for_reflect,
                    GetAddress::mut_multisig_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<super::types::InputScriptType>>(
                    "script_type",
                    GetAddress::get_script_type_for_reflect,
                    GetAddress::mut_script_type_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<GetAddress>(
                    "GetAddress",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for GetAddress {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_coin_name();
        self.clear_show_display();
        self.clear_multisig();
        self.clear_script_type();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for GetAddress {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for GetAddress {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EthereumGetAddress {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    show_display: ::std::option::Option<bool>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EthereumGetAddress {}

impl EthereumGetAddress {
    pub fn new() -> EthereumGetAddress {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EthereumGetAddress {
        static mut instance: ::protobuf::lazy::Lazy<EthereumGetAddress> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EthereumGetAddress,
        };
        unsafe {
            instance.get(EthereumGetAddress::new)
        }
    }

    // repeated uint32 address_n = 1;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // optional bool show_display = 2;

    pub fn clear_show_display(&mut self) {
        self.show_display = ::std::option::Option::None;
    }

    pub fn has_show_display(&self) -> bool {
        self.show_display.is_some()
    }

    // Param is passed by value, moved
    pub fn set_show_display(&mut self, v: bool) {
        self.show_display = ::std::option::Option::Some(v);
    }

    pub fn get_show_display(&self) -> bool {
        self.show_display.unwrap_or(false)
    }

    fn get_show_display_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.show_display
    }

    fn mut_show_display_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.show_display
    }
}

impl ::protobuf::Message for EthereumGetAddress {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.show_display = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(v) = self.show_display {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(v) = self.show_display {
            os.write_bool(2, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EthereumGetAddress {
    fn new() -> EthereumGetAddress {
        EthereumGetAddress::new()
    }

    fn descriptor_static(_: ::std::option::Option<EthereumGetAddress>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    EthereumGetAddress::get_address_n_for_reflect,
                    EthereumGetAddress::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "show_display",
                    EthereumGetAddress::get_show_display_for_reflect,
                    EthereumGetAddress::mut_show_display_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EthereumGetAddress>(
                    "EthereumGetAddress",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EthereumGetAddress {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_show_display();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EthereumGetAddress {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EthereumGetAddress {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Address {
    // message fields
    address: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Address {}

impl Address {
    pub fn new() -> Address {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Address {
        static mut instance: ::protobuf::lazy::Lazy<Address> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Address,
        };
        unsafe {
            instance.get(Address::new)
        }
    }

    // required string address = 1;

    pub fn clear_address(&mut self) {
        self.address.clear();
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: ::std::string::String) {
        self.address = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_address(&mut self) -> &mut ::std::string::String {
        if self.address.is_none() {
            self.address.set_default();
        }
        self.address.as_mut().unwrap()
    }

    // Take field
    pub fn take_address(&mut self) -> ::std::string::String {
        self.address.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_address(&self) -> &str {
        match self.address.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_address_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.address
    }
}

impl ::protobuf::Message for Address {
    fn is_initialized(&self) -> bool {
        if self.address.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.address)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.address.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.address.as_ref() {
            os.write_string(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Address {
    fn new() -> Address {
        Address::new()
    }

    fn descriptor_static(_: ::std::option::Option<Address>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "address",
                    Address::get_address_for_reflect,
                    Address::mut_address_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Address>(
                    "Address",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Address {
    fn clear(&mut self) {
        self.clear_address();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Address {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Address {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EthereumAddress {
    // message fields
    address: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EthereumAddress {}

impl EthereumAddress {
    pub fn new() -> EthereumAddress {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EthereumAddress {
        static mut instance: ::protobuf::lazy::Lazy<EthereumAddress> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EthereumAddress,
        };
        unsafe {
            instance.get(EthereumAddress::new)
        }
    }

    // required bytes address = 1;

    pub fn clear_address(&mut self) {
        self.address.clear();
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: ::std::vec::Vec<u8>) {
        self.address = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_address(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.address.is_none() {
            self.address.set_default();
        }
        self.address.as_mut().unwrap()
    }

    // Take field
    pub fn take_address(&mut self) -> ::std::vec::Vec<u8> {
        self.address.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_address(&self) -> &[u8] {
        match self.address.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_address_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.address
    }
}

impl ::protobuf::Message for EthereumAddress {
    fn is_initialized(&self) -> bool {
        if self.address.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.address)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.address.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.address.as_ref() {
            os.write_bytes(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EthereumAddress {
    fn new() -> EthereumAddress {
        EthereumAddress::new()
    }

    fn descriptor_static(_: ::std::option::Option<EthereumAddress>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "address",
                    EthereumAddress::get_address_for_reflect,
                    EthereumAddress::mut_address_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EthereumAddress>(
                    "EthereumAddress",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EthereumAddress {
    fn clear(&mut self) {
        self.clear_address();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EthereumAddress {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EthereumAddress {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct WipeDevice {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for WipeDevice {}

impl WipeDevice {
    pub fn new() -> WipeDevice {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static WipeDevice {
        static mut instance: ::protobuf::lazy::Lazy<WipeDevice> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const WipeDevice,
        };
        unsafe {
            instance.get(WipeDevice::new)
        }
    }
}

impl ::protobuf::Message for WipeDevice {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for WipeDevice {
    fn new() -> WipeDevice {
        WipeDevice::new()
    }

    fn descriptor_static(_: ::std::option::Option<WipeDevice>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<WipeDevice>(
                    "WipeDevice",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for WipeDevice {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for WipeDevice {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for WipeDevice {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct LoadDevice {
    // message fields
    mnemonic: ::protobuf::SingularField<::std::string::String>,
    node: ::protobuf::SingularPtrField<super::types::HDNodeType>,
    pin: ::protobuf::SingularField<::std::string::String>,
    passphrase_protection: ::std::option::Option<bool>,
    language: ::protobuf::SingularField<::std::string::String>,
    label: ::protobuf::SingularField<::std::string::String>,
    skip_checksum: ::std::option::Option<bool>,
    u2f_counter: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for LoadDevice {}

impl LoadDevice {
    pub fn new() -> LoadDevice {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static LoadDevice {
        static mut instance: ::protobuf::lazy::Lazy<LoadDevice> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const LoadDevice,
        };
        unsafe {
            instance.get(LoadDevice::new)
        }
    }

    // optional string mnemonic = 1;

    pub fn clear_mnemonic(&mut self) {
        self.mnemonic.clear();
    }

    pub fn has_mnemonic(&self) -> bool {
        self.mnemonic.is_some()
    }

    // Param is passed by value, moved
    pub fn set_mnemonic(&mut self, v: ::std::string::String) {
        self.mnemonic = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_mnemonic(&mut self) -> &mut ::std::string::String {
        if self.mnemonic.is_none() {
            self.mnemonic.set_default();
        }
        self.mnemonic.as_mut().unwrap()
    }

    // Take field
    pub fn take_mnemonic(&mut self) -> ::std::string::String {
        self.mnemonic.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_mnemonic(&self) -> &str {
        match self.mnemonic.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_mnemonic_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.mnemonic
    }

    fn mut_mnemonic_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.mnemonic
    }

    // optional .HDNodeType node = 2;

    pub fn clear_node(&mut self) {
        self.node.clear();
    }

    pub fn has_node(&self) -> bool {
        self.node.is_some()
    }

    // Param is passed by value, moved
    pub fn set_node(&mut self, v: super::types::HDNodeType) {
        self.node = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_node(&mut self) -> &mut super::types::HDNodeType {
        if self.node.is_none() {
            self.node.set_default();
        }
        self.node.as_mut().unwrap()
    }

    // Take field
    pub fn take_node(&mut self) -> super::types::HDNodeType {
        self.node.take().unwrap_or_else(|| super::types::HDNodeType::new())
    }

    pub fn get_node(&self) -> &super::types::HDNodeType {
        self.node.as_ref().unwrap_or_else(|| super::types::HDNodeType::default_instance())
    }

    fn get_node_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::HDNodeType> {
        &self.node
    }

    fn mut_node_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::HDNodeType> {
        &mut self.node
    }

    // optional string pin = 3;

    pub fn clear_pin(&mut self) {
        self.pin.clear();
    }

    pub fn has_pin(&self) -> bool {
        self.pin.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pin(&mut self, v: ::std::string::String) {
        self.pin = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_pin(&mut self) -> &mut ::std::string::String {
        if self.pin.is_none() {
            self.pin.set_default();
        }
        self.pin.as_mut().unwrap()
    }

    // Take field
    pub fn take_pin(&mut self) -> ::std::string::String {
        self.pin.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_pin(&self) -> &str {
        match self.pin.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_pin_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.pin
    }

    fn mut_pin_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.pin
    }

    // optional bool passphrase_protection = 4;

    pub fn clear_passphrase_protection(&mut self) {
        self.passphrase_protection = ::std::option::Option::None;
    }

    pub fn has_passphrase_protection(&self) -> bool {
        self.passphrase_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_passphrase_protection(&mut self, v: bool) {
        self.passphrase_protection = ::std::option::Option::Some(v);
    }

    pub fn get_passphrase_protection(&self) -> bool {
        self.passphrase_protection.unwrap_or(false)
    }

    fn get_passphrase_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.passphrase_protection
    }

    fn mut_passphrase_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.passphrase_protection
    }

    // optional string language = 5;

    pub fn clear_language(&mut self) {
        self.language.clear();
    }

    pub fn has_language(&self) -> bool {
        self.language.is_some()
    }

    // Param is passed by value, moved
    pub fn set_language(&mut self, v: ::std::string::String) {
        self.language = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_language(&mut self) -> &mut ::std::string::String {
        if self.language.is_none() {
            self.language.set_default();
        }
        self.language.as_mut().unwrap()
    }

    // Take field
    pub fn take_language(&mut self) -> ::std::string::String {
        self.language.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_language(&self) -> &str {
        match self.language.as_ref() {
            Some(v) => &v,
            None => "english",
        }
    }

    fn get_language_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.language
    }

    fn mut_language_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.language
    }

    // optional string label = 6;

    pub fn clear_label(&mut self) {
        self.label.clear();
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    // Param is passed by value, moved
    pub fn set_label(&mut self, v: ::std::string::String) {
        self.label = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_label(&mut self) -> &mut ::std::string::String {
        if self.label.is_none() {
            self.label.set_default();
        }
        self.label.as_mut().unwrap()
    }

    // Take field
    pub fn take_label(&mut self) -> ::std::string::String {
        self.label.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_label(&self) -> &str {
        match self.label.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_label_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.label
    }

    fn mut_label_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.label
    }

    // optional bool skip_checksum = 7;

    pub fn clear_skip_checksum(&mut self) {
        self.skip_checksum = ::std::option::Option::None;
    }

    pub fn has_skip_checksum(&self) -> bool {
        self.skip_checksum.is_some()
    }

    // Param is passed by value, moved
    pub fn set_skip_checksum(&mut self, v: bool) {
        self.skip_checksum = ::std::option::Option::Some(v);
    }

    pub fn get_skip_checksum(&self) -> bool {
        self.skip_checksum.unwrap_or(false)
    }

    fn get_skip_checksum_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.skip_checksum
    }

    fn mut_skip_checksum_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.skip_checksum
    }

    // optional uint32 u2f_counter = 8;

    pub fn clear_u2f_counter(&mut self) {
        self.u2f_counter = ::std::option::Option::None;
    }

    pub fn has_u2f_counter(&self) -> bool {
        self.u2f_counter.is_some()
    }

    // Param is passed by value, moved
    pub fn set_u2f_counter(&mut self, v: u32) {
        self.u2f_counter = ::std::option::Option::Some(v);
    }

    pub fn get_u2f_counter(&self) -> u32 {
        self.u2f_counter.unwrap_or(0)
    }

    fn get_u2f_counter_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.u2f_counter
    }

    fn mut_u2f_counter_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.u2f_counter
    }
}

impl ::protobuf::Message for LoadDevice {
    fn is_initialized(&self) -> bool {
        for v in &self.node {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.mnemonic)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.node)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.pin)?;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.passphrase_protection = ::std::option::Option::Some(tmp);
                },
                5 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.language)?;
                },
                6 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.label)?;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.skip_checksum = ::std::option::Option::Some(tmp);
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.u2f_counter = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.mnemonic.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(ref v) = self.node.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.pin.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        if let Some(v) = self.passphrase_protection {
            my_size += 2;
        }
        if let Some(ref v) = self.language.as_ref() {
            my_size += ::protobuf::rt::string_size(5, &v);
        }
        if let Some(ref v) = self.label.as_ref() {
            my_size += ::protobuf::rt::string_size(6, &v);
        }
        if let Some(v) = self.skip_checksum {
            my_size += 2;
        }
        if let Some(v) = self.u2f_counter {
            my_size += ::protobuf::rt::value_size(8, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.mnemonic.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(ref v) = self.node.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.pin.as_ref() {
            os.write_string(3, &v)?;
        }
        if let Some(v) = self.passphrase_protection {
            os.write_bool(4, v)?;
        }
        if let Some(ref v) = self.language.as_ref() {
            os.write_string(5, &v)?;
        }
        if let Some(ref v) = self.label.as_ref() {
            os.write_string(6, &v)?;
        }
        if let Some(v) = self.skip_checksum {
            os.write_bool(7, v)?;
        }
        if let Some(v) = self.u2f_counter {
            os.write_uint32(8, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for LoadDevice {
    fn new() -> LoadDevice {
        LoadDevice::new()
    }

    fn descriptor_static(_: ::std::option::Option<LoadDevice>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "mnemonic",
                    LoadDevice::get_mnemonic_for_reflect,
                    LoadDevice::mut_mnemonic_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::HDNodeType>>(
                    "node",
                    LoadDevice::get_node_for_reflect,
                    LoadDevice::mut_node_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "pin",
                    LoadDevice::get_pin_for_reflect,
                    LoadDevice::mut_pin_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "passphrase_protection",
                    LoadDevice::get_passphrase_protection_for_reflect,
                    LoadDevice::mut_passphrase_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "language",
                    LoadDevice::get_language_for_reflect,
                    LoadDevice::mut_language_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "label",
                    LoadDevice::get_label_for_reflect,
                    LoadDevice::mut_label_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "skip_checksum",
                    LoadDevice::get_skip_checksum_for_reflect,
                    LoadDevice::mut_skip_checksum_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "u2f_counter",
                    LoadDevice::get_u2f_counter_for_reflect,
                    LoadDevice::mut_u2f_counter_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<LoadDevice>(
                    "LoadDevice",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for LoadDevice {
    fn clear(&mut self) {
        self.clear_mnemonic();
        self.clear_node();
        self.clear_pin();
        self.clear_passphrase_protection();
        self.clear_language();
        self.clear_label();
        self.clear_skip_checksum();
        self.clear_u2f_counter();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for LoadDevice {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for LoadDevice {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ResetDevice {
    // message fields
    display_random: ::std::option::Option<bool>,
    strength: ::std::option::Option<u32>,
    passphrase_protection: ::std::option::Option<bool>,
    pin_protection: ::std::option::Option<bool>,
    language: ::protobuf::SingularField<::std::string::String>,
    label: ::protobuf::SingularField<::std::string::String>,
    u2f_counter: ::std::option::Option<u32>,
    skip_backup: ::std::option::Option<bool>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ResetDevice {}

impl ResetDevice {
    pub fn new() -> ResetDevice {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ResetDevice {
        static mut instance: ::protobuf::lazy::Lazy<ResetDevice> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ResetDevice,
        };
        unsafe {
            instance.get(ResetDevice::new)
        }
    }

    // optional bool display_random = 1;

    pub fn clear_display_random(&mut self) {
        self.display_random = ::std::option::Option::None;
    }

    pub fn has_display_random(&self) -> bool {
        self.display_random.is_some()
    }

    // Param is passed by value, moved
    pub fn set_display_random(&mut self, v: bool) {
        self.display_random = ::std::option::Option::Some(v);
    }

    pub fn get_display_random(&self) -> bool {
        self.display_random.unwrap_or(false)
    }

    fn get_display_random_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.display_random
    }

    fn mut_display_random_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.display_random
    }

    // optional uint32 strength = 2;

    pub fn clear_strength(&mut self) {
        self.strength = ::std::option::Option::None;
    }

    pub fn has_strength(&self) -> bool {
        self.strength.is_some()
    }

    // Param is passed by value, moved
    pub fn set_strength(&mut self, v: u32) {
        self.strength = ::std::option::Option::Some(v);
    }

    pub fn get_strength(&self) -> u32 {
        self.strength.unwrap_or(256u32)
    }

    fn get_strength_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.strength
    }

    fn mut_strength_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.strength
    }

    // optional bool passphrase_protection = 3;

    pub fn clear_passphrase_protection(&mut self) {
        self.passphrase_protection = ::std::option::Option::None;
    }

    pub fn has_passphrase_protection(&self) -> bool {
        self.passphrase_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_passphrase_protection(&mut self, v: bool) {
        self.passphrase_protection = ::std::option::Option::Some(v);
    }

    pub fn get_passphrase_protection(&self) -> bool {
        self.passphrase_protection.unwrap_or(false)
    }

    fn get_passphrase_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.passphrase_protection
    }

    fn mut_passphrase_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.passphrase_protection
    }

    // optional bool pin_protection = 4;

    pub fn clear_pin_protection(&mut self) {
        self.pin_protection = ::std::option::Option::None;
    }

    pub fn has_pin_protection(&self) -> bool {
        self.pin_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pin_protection(&mut self, v: bool) {
        self.pin_protection = ::std::option::Option::Some(v);
    }

    pub fn get_pin_protection(&self) -> bool {
        self.pin_protection.unwrap_or(false)
    }

    fn get_pin_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.pin_protection
    }

    fn mut_pin_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.pin_protection
    }

    // optional string language = 5;

    pub fn clear_language(&mut self) {
        self.language.clear();
    }

    pub fn has_language(&self) -> bool {
        self.language.is_some()
    }

    // Param is passed by value, moved
    pub fn set_language(&mut self, v: ::std::string::String) {
        self.language = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_language(&mut self) -> &mut ::std::string::String {
        if self.language.is_none() {
            self.language.set_default();
        }
        self.language.as_mut().unwrap()
    }

    // Take field
    pub fn take_language(&mut self) -> ::std::string::String {
        self.language.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_language(&self) -> &str {
        match self.language.as_ref() {
            Some(v) => &v,
            None => "english",
        }
    }

    fn get_language_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.language
    }

    fn mut_language_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.language
    }

    // optional string label = 6;

    pub fn clear_label(&mut self) {
        self.label.clear();
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    // Param is passed by value, moved
    pub fn set_label(&mut self, v: ::std::string::String) {
        self.label = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_label(&mut self) -> &mut ::std::string::String {
        if self.label.is_none() {
            self.label.set_default();
        }
        self.label.as_mut().unwrap()
    }

    // Take field
    pub fn take_label(&mut self) -> ::std::string::String {
        self.label.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_label(&self) -> &str {
        match self.label.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_label_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.label
    }

    fn mut_label_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.label
    }

    // optional uint32 u2f_counter = 7;

    pub fn clear_u2f_counter(&mut self) {
        self.u2f_counter = ::std::option::Option::None;
    }

    pub fn has_u2f_counter(&self) -> bool {
        self.u2f_counter.is_some()
    }

    // Param is passed by value, moved
    pub fn set_u2f_counter(&mut self, v: u32) {
        self.u2f_counter = ::std::option::Option::Some(v);
    }

    pub fn get_u2f_counter(&self) -> u32 {
        self.u2f_counter.unwrap_or(0)
    }

    fn get_u2f_counter_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.u2f_counter
    }

    fn mut_u2f_counter_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.u2f_counter
    }

    // optional bool skip_backup = 8;

    pub fn clear_skip_backup(&mut self) {
        self.skip_backup = ::std::option::Option::None;
    }

    pub fn has_skip_backup(&self) -> bool {
        self.skip_backup.is_some()
    }

    // Param is passed by value, moved
    pub fn set_skip_backup(&mut self, v: bool) {
        self.skip_backup = ::std::option::Option::Some(v);
    }

    pub fn get_skip_backup(&self) -> bool {
        self.skip_backup.unwrap_or(false)
    }

    fn get_skip_backup_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.skip_backup
    }

    fn mut_skip_backup_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.skip_backup
    }
}

impl ::protobuf::Message for ResetDevice {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.display_random = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.strength = ::std::option::Option::Some(tmp);
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.passphrase_protection = ::std::option::Option::Some(tmp);
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.pin_protection = ::std::option::Option::Some(tmp);
                },
                5 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.language)?;
                },
                6 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.label)?;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.u2f_counter = ::std::option::Option::Some(tmp);
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.skip_backup = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.display_random {
            my_size += 2;
        }
        if let Some(v) = self.strength {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.passphrase_protection {
            my_size += 2;
        }
        if let Some(v) = self.pin_protection {
            my_size += 2;
        }
        if let Some(ref v) = self.language.as_ref() {
            my_size += ::protobuf::rt::string_size(5, &v);
        }
        if let Some(ref v) = self.label.as_ref() {
            my_size += ::protobuf::rt::string_size(6, &v);
        }
        if let Some(v) = self.u2f_counter {
            my_size += ::protobuf::rt::value_size(7, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.skip_backup {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.display_random {
            os.write_bool(1, v)?;
        }
        if let Some(v) = self.strength {
            os.write_uint32(2, v)?;
        }
        if let Some(v) = self.passphrase_protection {
            os.write_bool(3, v)?;
        }
        if let Some(v) = self.pin_protection {
            os.write_bool(4, v)?;
        }
        if let Some(ref v) = self.language.as_ref() {
            os.write_string(5, &v)?;
        }
        if let Some(ref v) = self.label.as_ref() {
            os.write_string(6, &v)?;
        }
        if let Some(v) = self.u2f_counter {
            os.write_uint32(7, v)?;
        }
        if let Some(v) = self.skip_backup {
            os.write_bool(8, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ResetDevice {
    fn new() -> ResetDevice {
        ResetDevice::new()
    }

    fn descriptor_static(_: ::std::option::Option<ResetDevice>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "display_random",
                    ResetDevice::get_display_random_for_reflect,
                    ResetDevice::mut_display_random_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "strength",
                    ResetDevice::get_strength_for_reflect,
                    ResetDevice::mut_strength_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "passphrase_protection",
                    ResetDevice::get_passphrase_protection_for_reflect,
                    ResetDevice::mut_passphrase_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "pin_protection",
                    ResetDevice::get_pin_protection_for_reflect,
                    ResetDevice::mut_pin_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "language",
                    ResetDevice::get_language_for_reflect,
                    ResetDevice::mut_language_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "label",
                    ResetDevice::get_label_for_reflect,
                    ResetDevice::mut_label_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "u2f_counter",
                    ResetDevice::get_u2f_counter_for_reflect,
                    ResetDevice::mut_u2f_counter_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "skip_backup",
                    ResetDevice::get_skip_backup_for_reflect,
                    ResetDevice::mut_skip_backup_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ResetDevice>(
                    "ResetDevice",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ResetDevice {
    fn clear(&mut self) {
        self.clear_display_random();
        self.clear_strength();
        self.clear_passphrase_protection();
        self.clear_pin_protection();
        self.clear_language();
        self.clear_label();
        self.clear_u2f_counter();
        self.clear_skip_backup();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ResetDevice {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ResetDevice {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct BackupDevice {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for BackupDevice {}

impl BackupDevice {
    pub fn new() -> BackupDevice {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static BackupDevice {
        static mut instance: ::protobuf::lazy::Lazy<BackupDevice> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const BackupDevice,
        };
        unsafe {
            instance.get(BackupDevice::new)
        }
    }
}

impl ::protobuf::Message for BackupDevice {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for BackupDevice {
    fn new() -> BackupDevice {
        BackupDevice::new()
    }

    fn descriptor_static(_: ::std::option::Option<BackupDevice>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<BackupDevice>(
                    "BackupDevice",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for BackupDevice {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for BackupDevice {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for BackupDevice {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EntropyRequest {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EntropyRequest {}

impl EntropyRequest {
    pub fn new() -> EntropyRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EntropyRequest {
        static mut instance: ::protobuf::lazy::Lazy<EntropyRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EntropyRequest,
        };
        unsafe {
            instance.get(EntropyRequest::new)
        }
    }
}

impl ::protobuf::Message for EntropyRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EntropyRequest {
    fn new() -> EntropyRequest {
        EntropyRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<EntropyRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<EntropyRequest>(
                    "EntropyRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EntropyRequest {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EntropyRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EntropyRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EntropyAck {
    // message fields
    entropy: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EntropyAck {}

impl EntropyAck {
    pub fn new() -> EntropyAck {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EntropyAck {
        static mut instance: ::protobuf::lazy::Lazy<EntropyAck> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EntropyAck,
        };
        unsafe {
            instance.get(EntropyAck::new)
        }
    }

    // optional bytes entropy = 1;

    pub fn clear_entropy(&mut self) {
        self.entropy.clear();
    }

    pub fn has_entropy(&self) -> bool {
        self.entropy.is_some()
    }

    // Param is passed by value, moved
    pub fn set_entropy(&mut self, v: ::std::vec::Vec<u8>) {
        self.entropy = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_entropy(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.entropy.is_none() {
            self.entropy.set_default();
        }
        self.entropy.as_mut().unwrap()
    }

    // Take field
    pub fn take_entropy(&mut self) -> ::std::vec::Vec<u8> {
        self.entropy.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_entropy(&self) -> &[u8] {
        match self.entropy.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_entropy_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.entropy
    }

    fn mut_entropy_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.entropy
    }
}

impl ::protobuf::Message for EntropyAck {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.entropy)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.entropy.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.entropy.as_ref() {
            os.write_bytes(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EntropyAck {
    fn new() -> EntropyAck {
        EntropyAck::new()
    }

    fn descriptor_static(_: ::std::option::Option<EntropyAck>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "entropy",
                    EntropyAck::get_entropy_for_reflect,
                    EntropyAck::mut_entropy_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EntropyAck>(
                    "EntropyAck",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EntropyAck {
    fn clear(&mut self) {
        self.clear_entropy();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EntropyAck {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EntropyAck {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct RecoveryDevice {
    // message fields
    word_count: ::std::option::Option<u32>,
    passphrase_protection: ::std::option::Option<bool>,
    pin_protection: ::std::option::Option<bool>,
    language: ::protobuf::SingularField<::std::string::String>,
    label: ::protobuf::SingularField<::std::string::String>,
    enforce_wordlist: ::std::option::Option<bool>,
    field_type: ::std::option::Option<u32>,
    u2f_counter: ::std::option::Option<u32>,
    dry_run: ::std::option::Option<bool>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for RecoveryDevice {}

impl RecoveryDevice {
    pub fn new() -> RecoveryDevice {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static RecoveryDevice {
        static mut instance: ::protobuf::lazy::Lazy<RecoveryDevice> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RecoveryDevice,
        };
        unsafe {
            instance.get(RecoveryDevice::new)
        }
    }

    // optional uint32 word_count = 1;

    pub fn clear_word_count(&mut self) {
        self.word_count = ::std::option::Option::None;
    }

    pub fn has_word_count(&self) -> bool {
        self.word_count.is_some()
    }

    // Param is passed by value, moved
    pub fn set_word_count(&mut self, v: u32) {
        self.word_count = ::std::option::Option::Some(v);
    }

    pub fn get_word_count(&self) -> u32 {
        self.word_count.unwrap_or(0)
    }

    fn get_word_count_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.word_count
    }

    fn mut_word_count_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.word_count
    }

    // optional bool passphrase_protection = 2;

    pub fn clear_passphrase_protection(&mut self) {
        self.passphrase_protection = ::std::option::Option::None;
    }

    pub fn has_passphrase_protection(&self) -> bool {
        self.passphrase_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_passphrase_protection(&mut self, v: bool) {
        self.passphrase_protection = ::std::option::Option::Some(v);
    }

    pub fn get_passphrase_protection(&self) -> bool {
        self.passphrase_protection.unwrap_or(false)
    }

    fn get_passphrase_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.passphrase_protection
    }

    fn mut_passphrase_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.passphrase_protection
    }

    // optional bool pin_protection = 3;

    pub fn clear_pin_protection(&mut self) {
        self.pin_protection = ::std::option::Option::None;
    }

    pub fn has_pin_protection(&self) -> bool {
        self.pin_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pin_protection(&mut self, v: bool) {
        self.pin_protection = ::std::option::Option::Some(v);
    }

    pub fn get_pin_protection(&self) -> bool {
        self.pin_protection.unwrap_or(false)
    }

    fn get_pin_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.pin_protection
    }

    fn mut_pin_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.pin_protection
    }

    // optional string language = 4;

    pub fn clear_language(&mut self) {
        self.language.clear();
    }

    pub fn has_language(&self) -> bool {
        self.language.is_some()
    }

    // Param is passed by value, moved
    pub fn set_language(&mut self, v: ::std::string::String) {
        self.language = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_language(&mut self) -> &mut ::std::string::String {
        if self.language.is_none() {
            self.language.set_default();
        }
        self.language.as_mut().unwrap()
    }

    // Take field
    pub fn take_language(&mut self) -> ::std::string::String {
        self.language.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_language(&self) -> &str {
        match self.language.as_ref() {
            Some(v) => &v,
            None => "english",
        }
    }

    fn get_language_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.language
    }

    fn mut_language_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.language
    }

    // optional string label = 5;

    pub fn clear_label(&mut self) {
        self.label.clear();
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    // Param is passed by value, moved
    pub fn set_label(&mut self, v: ::std::string::String) {
        self.label = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_label(&mut self) -> &mut ::std::string::String {
        if self.label.is_none() {
            self.label.set_default();
        }
        self.label.as_mut().unwrap()
    }

    // Take field
    pub fn take_label(&mut self) -> ::std::string::String {
        self.label.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_label(&self) -> &str {
        match self.label.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_label_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.label
    }

    fn mut_label_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.label
    }

    // optional bool enforce_wordlist = 6;

    pub fn clear_enforce_wordlist(&mut self) {
        self.enforce_wordlist = ::std::option::Option::None;
    }

    pub fn has_enforce_wordlist(&self) -> bool {
        self.enforce_wordlist.is_some()
    }

    // Param is passed by value, moved
    pub fn set_enforce_wordlist(&mut self, v: bool) {
        self.enforce_wordlist = ::std::option::Option::Some(v);
    }

    pub fn get_enforce_wordlist(&self) -> bool {
        self.enforce_wordlist.unwrap_or(false)
    }

    fn get_enforce_wordlist_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.enforce_wordlist
    }

    fn mut_enforce_wordlist_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.enforce_wordlist
    }

    // optional uint32 type = 8;

    pub fn clear_field_type(&mut self) {
        self.field_type = ::std::option::Option::None;
    }

    pub fn has_field_type(&self) -> bool {
        self.field_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_field_type(&mut self, v: u32) {
        self.field_type = ::std::option::Option::Some(v);
    }

    pub fn get_field_type(&self) -> u32 {
        self.field_type.unwrap_or(0)
    }

    fn get_field_type_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.field_type
    }

    fn mut_field_type_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.field_type
    }

    // optional uint32 u2f_counter = 9;

    pub fn clear_u2f_counter(&mut self) {
        self.u2f_counter = ::std::option::Option::None;
    }

    pub fn has_u2f_counter(&self) -> bool {
        self.u2f_counter.is_some()
    }

    // Param is passed by value, moved
    pub fn set_u2f_counter(&mut self, v: u32) {
        self.u2f_counter = ::std::option::Option::Some(v);
    }

    pub fn get_u2f_counter(&self) -> u32 {
        self.u2f_counter.unwrap_or(0)
    }

    fn get_u2f_counter_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.u2f_counter
    }

    fn mut_u2f_counter_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.u2f_counter
    }

    // optional bool dry_run = 10;

    pub fn clear_dry_run(&mut self) {
        self.dry_run = ::std::option::Option::None;
    }

    pub fn has_dry_run(&self) -> bool {
        self.dry_run.is_some()
    }

    // Param is passed by value, moved
    pub fn set_dry_run(&mut self, v: bool) {
        self.dry_run = ::std::option::Option::Some(v);
    }

    pub fn get_dry_run(&self) -> bool {
        self.dry_run.unwrap_or(false)
    }

    fn get_dry_run_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.dry_run
    }

    fn mut_dry_run_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.dry_run
    }
}

impl ::protobuf::Message for RecoveryDevice {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.word_count = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.passphrase_protection = ::std::option::Option::Some(tmp);
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.pin_protection = ::std::option::Option::Some(tmp);
                },
                4 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.language)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.label)?;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.enforce_wordlist = ::std::option::Option::Some(tmp);
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.field_type = ::std::option::Option::Some(tmp);
                },
                9 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.u2f_counter = ::std::option::Option::Some(tmp);
                },
                10 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.dry_run = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.word_count {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.passphrase_protection {
            my_size += 2;
        }
        if let Some(v) = self.pin_protection {
            my_size += 2;
        }
        if let Some(ref v) = self.language.as_ref() {
            my_size += ::protobuf::rt::string_size(4, &v);
        }
        if let Some(ref v) = self.label.as_ref() {
            my_size += ::protobuf::rt::string_size(5, &v);
        }
        if let Some(v) = self.enforce_wordlist {
            my_size += 2;
        }
        if let Some(v) = self.field_type {
            my_size += ::protobuf::rt::value_size(8, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.u2f_counter {
            my_size += ::protobuf::rt::value_size(9, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.dry_run {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.word_count {
            os.write_uint32(1, v)?;
        }
        if let Some(v) = self.passphrase_protection {
            os.write_bool(2, v)?;
        }
        if let Some(v) = self.pin_protection {
            os.write_bool(3, v)?;
        }
        if let Some(ref v) = self.language.as_ref() {
            os.write_string(4, &v)?;
        }
        if let Some(ref v) = self.label.as_ref() {
            os.write_string(5, &v)?;
        }
        if let Some(v) = self.enforce_wordlist {
            os.write_bool(6, v)?;
        }
        if let Some(v) = self.field_type {
            os.write_uint32(8, v)?;
        }
        if let Some(v) = self.u2f_counter {
            os.write_uint32(9, v)?;
        }
        if let Some(v) = self.dry_run {
            os.write_bool(10, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for RecoveryDevice {
    fn new() -> RecoveryDevice {
        RecoveryDevice::new()
    }

    fn descriptor_static(_: ::std::option::Option<RecoveryDevice>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "word_count",
                    RecoveryDevice::get_word_count_for_reflect,
                    RecoveryDevice::mut_word_count_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "passphrase_protection",
                    RecoveryDevice::get_passphrase_protection_for_reflect,
                    RecoveryDevice::mut_passphrase_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "pin_protection",
                    RecoveryDevice::get_pin_protection_for_reflect,
                    RecoveryDevice::mut_pin_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "language",
                    RecoveryDevice::get_language_for_reflect,
                    RecoveryDevice::mut_language_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "label",
                    RecoveryDevice::get_label_for_reflect,
                    RecoveryDevice::mut_label_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "enforce_wordlist",
                    RecoveryDevice::get_enforce_wordlist_for_reflect,
                    RecoveryDevice::mut_enforce_wordlist_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "type",
                    RecoveryDevice::get_field_type_for_reflect,
                    RecoveryDevice::mut_field_type_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "u2f_counter",
                    RecoveryDevice::get_u2f_counter_for_reflect,
                    RecoveryDevice::mut_u2f_counter_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "dry_run",
                    RecoveryDevice::get_dry_run_for_reflect,
                    RecoveryDevice::mut_dry_run_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RecoveryDevice>(
                    "RecoveryDevice",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for RecoveryDevice {
    fn clear(&mut self) {
        self.clear_word_count();
        self.clear_passphrase_protection();
        self.clear_pin_protection();
        self.clear_language();
        self.clear_label();
        self.clear_enforce_wordlist();
        self.clear_field_type();
        self.clear_u2f_counter();
        self.clear_dry_run();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RecoveryDevice {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RecoveryDevice {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct WordRequest {
    // message fields
    field_type: ::std::option::Option<super::types::WordRequestType>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for WordRequest {}

impl WordRequest {
    pub fn new() -> WordRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static WordRequest {
        static mut instance: ::protobuf::lazy::Lazy<WordRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const WordRequest,
        };
        unsafe {
            instance.get(WordRequest::new)
        }
    }

    // optional .WordRequestType type = 1;

    pub fn clear_field_type(&mut self) {
        self.field_type = ::std::option::Option::None;
    }

    pub fn has_field_type(&self) -> bool {
        self.field_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_field_type(&mut self, v: super::types::WordRequestType) {
        self.field_type = ::std::option::Option::Some(v);
    }

    pub fn get_field_type(&self) -> super::types::WordRequestType {
        self.field_type.unwrap_or(super::types::WordRequestType::WordRequestType_Plain)
    }

    fn get_field_type_for_reflect(&self) -> &::std::option::Option<super::types::WordRequestType> {
        &self.field_type
    }

    fn mut_field_type_for_reflect(&mut self) -> &mut ::std::option::Option<super::types::WordRequestType> {
        &mut self.field_type
    }
}

impl ::protobuf::Message for WordRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.field_type = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.field_type {
            my_size += ::protobuf::rt::enum_size(1, v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.field_type {
            os.write_enum(1, v.value())?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for WordRequest {
    fn new() -> WordRequest {
        WordRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<WordRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<super::types::WordRequestType>>(
                    "type",
                    WordRequest::get_field_type_for_reflect,
                    WordRequest::mut_field_type_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<WordRequest>(
                    "WordRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for WordRequest {
    fn clear(&mut self) {
        self.clear_field_type();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for WordRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for WordRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct WordAck {
    // message fields
    word: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for WordAck {}

impl WordAck {
    pub fn new() -> WordAck {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static WordAck {
        static mut instance: ::protobuf::lazy::Lazy<WordAck> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const WordAck,
        };
        unsafe {
            instance.get(WordAck::new)
        }
    }

    // required string word = 1;

    pub fn clear_word(&mut self) {
        self.word.clear();
    }

    pub fn has_word(&self) -> bool {
        self.word.is_some()
    }

    // Param is passed by value, moved
    pub fn set_word(&mut self, v: ::std::string::String) {
        self.word = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_word(&mut self) -> &mut ::std::string::String {
        if self.word.is_none() {
            self.word.set_default();
        }
        self.word.as_mut().unwrap()
    }

    // Take field
    pub fn take_word(&mut self) -> ::std::string::String {
        self.word.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_word(&self) -> &str {
        match self.word.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_word_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.word
    }

    fn mut_word_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.word
    }
}

impl ::protobuf::Message for WordAck {
    fn is_initialized(&self) -> bool {
        if self.word.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.word)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.word.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.word.as_ref() {
            os.write_string(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for WordAck {
    fn new() -> WordAck {
        WordAck::new()
    }

    fn descriptor_static(_: ::std::option::Option<WordAck>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "word",
                    WordAck::get_word_for_reflect,
                    WordAck::mut_word_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<WordAck>(
                    "WordAck",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for WordAck {
    fn clear(&mut self) {
        self.clear_word();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for WordAck {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for WordAck {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SignMessage {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    message: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    coin_name: ::protobuf::SingularField<::std::string::String>,
    script_type: ::std::option::Option<super::types::InputScriptType>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for SignMessage {}

impl SignMessage {
    pub fn new() -> SignMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static SignMessage {
        static mut instance: ::protobuf::lazy::Lazy<SignMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const SignMessage,
        };
        unsafe {
            instance.get(SignMessage::new)
        }
    }

    // repeated uint32 address_n = 1;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // required bytes message = 2;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::vec::Vec<u8>) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::vec::Vec<u8> {
        self.message.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_message(&self) -> &[u8] {
        match self.message.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.message
    }

    // optional string coin_name = 3;

    pub fn clear_coin_name(&mut self) {
        self.coin_name.clear();
    }

    pub fn has_coin_name(&self) -> bool {
        self.coin_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_name(&mut self, v: ::std::string::String) {
        self.coin_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_name(&mut self) -> &mut ::std::string::String {
        if self.coin_name.is_none() {
            self.coin_name.set_default();
        }
        self.coin_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_name(&mut self) -> ::std::string::String {
        self.coin_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_name(&self) -> &str {
        match self.coin_name.as_ref() {
            Some(v) => &v,
            None => "Bitcoin",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }

    // optional .InputScriptType script_type = 4;

    pub fn clear_script_type(&mut self) {
        self.script_type = ::std::option::Option::None;
    }

    pub fn has_script_type(&self) -> bool {
        self.script_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_script_type(&mut self, v: super::types::InputScriptType) {
        self.script_type = ::std::option::Option::Some(v);
    }

    pub fn get_script_type(&self) -> super::types::InputScriptType {
        self.script_type.unwrap_or(super::types::InputScriptType::SPENDADDRESS)
    }

    fn get_script_type_for_reflect(&self) -> &::std::option::Option<super::types::InputScriptType> {
        &self.script_type
    }

    fn mut_script_type_for_reflect(&mut self) -> &mut ::std::option::Option<super::types::InputScriptType> {
        &mut self.script_type
    }
}

impl ::protobuf::Message for SignMessage {
    fn is_initialized(&self) -> bool {
        if self.message.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.message)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.script_type = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        if let Some(v) = self.script_type {
            my_size += ::protobuf::rt::enum_size(4, v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(ref v) = self.message.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(3, &v)?;
        }
        if let Some(v) = self.script_type {
            os.write_enum(4, v.value())?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for SignMessage {
    fn new() -> SignMessage {
        SignMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<SignMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    SignMessage::get_address_n_for_reflect,
                    SignMessage::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "message",
                    SignMessage::get_message_for_reflect,
                    SignMessage::mut_message_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    SignMessage::get_coin_name_for_reflect,
                    SignMessage::mut_coin_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<super::types::InputScriptType>>(
                    "script_type",
                    SignMessage::get_script_type_for_reflect,
                    SignMessage::mut_script_type_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<SignMessage>(
                    "SignMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for SignMessage {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_message();
        self.clear_coin_name();
        self.clear_script_type();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SignMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SignMessage {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct VerifyMessage {
    // message fields
    address: ::protobuf::SingularField<::std::string::String>,
    signature: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    message: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    coin_name: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for VerifyMessage {}

impl VerifyMessage {
    pub fn new() -> VerifyMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static VerifyMessage {
        static mut instance: ::protobuf::lazy::Lazy<VerifyMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const VerifyMessage,
        };
        unsafe {
            instance.get(VerifyMessage::new)
        }
    }

    // optional string address = 1;

    pub fn clear_address(&mut self) {
        self.address.clear();
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: ::std::string::String) {
        self.address = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_address(&mut self) -> &mut ::std::string::String {
        if self.address.is_none() {
            self.address.set_default();
        }
        self.address.as_mut().unwrap()
    }

    // Take field
    pub fn take_address(&mut self) -> ::std::string::String {
        self.address.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_address(&self) -> &str {
        match self.address.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_address_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.address
    }

    // optional bytes signature = 2;

    pub fn clear_signature(&mut self) {
        self.signature.clear();
    }

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature(&mut self, v: ::std::vec::Vec<u8>) {
        self.signature = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signature(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.signature.is_none() {
            self.signature.set_default();
        }
        self.signature.as_mut().unwrap()
    }

    // Take field
    pub fn take_signature(&mut self) -> ::std::vec::Vec<u8> {
        self.signature.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_signature(&self) -> &[u8] {
        match self.signature.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_signature_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.signature
    }

    fn mut_signature_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.signature
    }

    // optional bytes message = 3;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::vec::Vec<u8>) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::vec::Vec<u8> {
        self.message.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_message(&self) -> &[u8] {
        match self.message.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.message
    }

    // optional string coin_name = 4;

    pub fn clear_coin_name(&mut self) {
        self.coin_name.clear();
    }

    pub fn has_coin_name(&self) -> bool {
        self.coin_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_name(&mut self, v: ::std::string::String) {
        self.coin_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_name(&mut self) -> &mut ::std::string::String {
        if self.coin_name.is_none() {
            self.coin_name.set_default();
        }
        self.coin_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_name(&mut self) -> ::std::string::String {
        self.coin_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_name(&self) -> &str {
        match self.coin_name.as_ref() {
            Some(v) => &v,
            None => "Bitcoin",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }
}

impl ::protobuf::Message for VerifyMessage {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.address)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.signature)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.message)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.address.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(ref v) = self.signature.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(4, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.address.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(ref v) = self.signature.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.message.as_ref() {
            os.write_bytes(3, &v)?;
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(4, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for VerifyMessage {
    fn new() -> VerifyMessage {
        VerifyMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<VerifyMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "address",
                    VerifyMessage::get_address_for_reflect,
                    VerifyMessage::mut_address_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature",
                    VerifyMessage::get_signature_for_reflect,
                    VerifyMessage::mut_signature_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "message",
                    VerifyMessage::get_message_for_reflect,
                    VerifyMessage::mut_message_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    VerifyMessage::get_coin_name_for_reflect,
                    VerifyMessage::mut_coin_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<VerifyMessage>(
                    "VerifyMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for VerifyMessage {
    fn clear(&mut self) {
        self.clear_address();
        self.clear_signature();
        self.clear_message();
        self.clear_coin_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for VerifyMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for VerifyMessage {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct MessageSignature {
    // message fields
    address: ::protobuf::SingularField<::std::string::String>,
    signature: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for MessageSignature {}

impl MessageSignature {
    pub fn new() -> MessageSignature {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static MessageSignature {
        static mut instance: ::protobuf::lazy::Lazy<MessageSignature> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const MessageSignature,
        };
        unsafe {
            instance.get(MessageSignature::new)
        }
    }

    // optional string address = 1;

    pub fn clear_address(&mut self) {
        self.address.clear();
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: ::std::string::String) {
        self.address = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_address(&mut self) -> &mut ::std::string::String {
        if self.address.is_none() {
            self.address.set_default();
        }
        self.address.as_mut().unwrap()
    }

    // Take field
    pub fn take_address(&mut self) -> ::std::string::String {
        self.address.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_address(&self) -> &str {
        match self.address.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_address_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.address
    }

    // optional bytes signature = 2;

    pub fn clear_signature(&mut self) {
        self.signature.clear();
    }

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature(&mut self, v: ::std::vec::Vec<u8>) {
        self.signature = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signature(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.signature.is_none() {
            self.signature.set_default();
        }
        self.signature.as_mut().unwrap()
    }

    // Take field
    pub fn take_signature(&mut self) -> ::std::vec::Vec<u8> {
        self.signature.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_signature(&self) -> &[u8] {
        match self.signature.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_signature_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.signature
    }

    fn mut_signature_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.signature
    }
}

impl ::protobuf::Message for MessageSignature {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.address)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.signature)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.address.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(ref v) = self.signature.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.address.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(ref v) = self.signature.as_ref() {
            os.write_bytes(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for MessageSignature {
    fn new() -> MessageSignature {
        MessageSignature::new()
    }

    fn descriptor_static(_: ::std::option::Option<MessageSignature>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "address",
                    MessageSignature::get_address_for_reflect,
                    MessageSignature::mut_address_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature",
                    MessageSignature::get_signature_for_reflect,
                    MessageSignature::mut_signature_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<MessageSignature>(
                    "MessageSignature",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for MessageSignature {
    fn clear(&mut self) {
        self.clear_address();
        self.clear_signature();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for MessageSignature {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for MessageSignature {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EncryptMessage {
    // message fields
    pubkey: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    message: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    display_only: ::std::option::Option<bool>,
    address_n: ::std::vec::Vec<u32>,
    coin_name: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EncryptMessage {}

impl EncryptMessage {
    pub fn new() -> EncryptMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EncryptMessage {
        static mut instance: ::protobuf::lazy::Lazy<EncryptMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EncryptMessage,
        };
        unsafe {
            instance.get(EncryptMessage::new)
        }
    }

    // optional bytes pubkey = 1;

    pub fn clear_pubkey(&mut self) {
        self.pubkey.clear();
    }

    pub fn has_pubkey(&self) -> bool {
        self.pubkey.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pubkey(&mut self, v: ::std::vec::Vec<u8>) {
        self.pubkey = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_pubkey(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.pubkey.is_none() {
            self.pubkey.set_default();
        }
        self.pubkey.as_mut().unwrap()
    }

    // Take field
    pub fn take_pubkey(&mut self) -> ::std::vec::Vec<u8> {
        self.pubkey.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_pubkey(&self) -> &[u8] {
        match self.pubkey.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_pubkey_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.pubkey
    }

    fn mut_pubkey_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.pubkey
    }

    // optional bytes message = 2;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::vec::Vec<u8>) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::vec::Vec<u8> {
        self.message.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_message(&self) -> &[u8] {
        match self.message.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.message
    }

    // optional bool display_only = 3;

    pub fn clear_display_only(&mut self) {
        self.display_only = ::std::option::Option::None;
    }

    pub fn has_display_only(&self) -> bool {
        self.display_only.is_some()
    }

    // Param is passed by value, moved
    pub fn set_display_only(&mut self, v: bool) {
        self.display_only = ::std::option::Option::Some(v);
    }

    pub fn get_display_only(&self) -> bool {
        self.display_only.unwrap_or(false)
    }

    fn get_display_only_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.display_only
    }

    fn mut_display_only_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.display_only
    }

    // repeated uint32 address_n = 4;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // optional string coin_name = 5;

    pub fn clear_coin_name(&mut self) {
        self.coin_name.clear();
    }

    pub fn has_coin_name(&self) -> bool {
        self.coin_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_name(&mut self, v: ::std::string::String) {
        self.coin_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_name(&mut self) -> &mut ::std::string::String {
        if self.coin_name.is_none() {
            self.coin_name.set_default();
        }
        self.coin_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_name(&mut self) -> ::std::string::String {
        self.coin_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_name(&self) -> &str {
        match self.coin_name.as_ref() {
            Some(v) => &v,
            None => "Bitcoin",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }
}

impl ::protobuf::Message for EncryptMessage {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.pubkey)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.message)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.display_only = ::std::option::Option::Some(tmp);
                },
                4 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.pubkey.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(v) = self.display_only {
            my_size += 2;
        }
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(4, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(5, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.pubkey.as_ref() {
            os.write_bytes(1, &v)?;
        }
        if let Some(ref v) = self.message.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(v) = self.display_only {
            os.write_bool(3, v)?;
        }
        for v in &self.address_n {
            os.write_uint32(4, *v)?;
        };
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(5, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EncryptMessage {
    fn new() -> EncryptMessage {
        EncryptMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<EncryptMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "pubkey",
                    EncryptMessage::get_pubkey_for_reflect,
                    EncryptMessage::mut_pubkey_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "message",
                    EncryptMessage::get_message_for_reflect,
                    EncryptMessage::mut_message_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "display_only",
                    EncryptMessage::get_display_only_for_reflect,
                    EncryptMessage::mut_display_only_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    EncryptMessage::get_address_n_for_reflect,
                    EncryptMessage::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    EncryptMessage::get_coin_name_for_reflect,
                    EncryptMessage::mut_coin_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EncryptMessage>(
                    "EncryptMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EncryptMessage {
    fn clear(&mut self) {
        self.clear_pubkey();
        self.clear_message();
        self.clear_display_only();
        self.clear_address_n();
        self.clear_coin_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EncryptMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EncryptMessage {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EncryptedMessage {
    // message fields
    nonce: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    message: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    hmac: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EncryptedMessage {}

impl EncryptedMessage {
    pub fn new() -> EncryptedMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EncryptedMessage {
        static mut instance: ::protobuf::lazy::Lazy<EncryptedMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EncryptedMessage,
        };
        unsafe {
            instance.get(EncryptedMessage::new)
        }
    }

    // optional bytes nonce = 1;

    pub fn clear_nonce(&mut self) {
        self.nonce.clear();
    }

    pub fn has_nonce(&self) -> bool {
        self.nonce.is_some()
    }

    // Param is passed by value, moved
    pub fn set_nonce(&mut self, v: ::std::vec::Vec<u8>) {
        self.nonce = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_nonce(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.nonce.is_none() {
            self.nonce.set_default();
        }
        self.nonce.as_mut().unwrap()
    }

    // Take field
    pub fn take_nonce(&mut self) -> ::std::vec::Vec<u8> {
        self.nonce.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_nonce(&self) -> &[u8] {
        match self.nonce.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_nonce_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.nonce
    }

    fn mut_nonce_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.nonce
    }

    // optional bytes message = 2;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::vec::Vec<u8>) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::vec::Vec<u8> {
        self.message.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_message(&self) -> &[u8] {
        match self.message.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.message
    }

    // optional bytes hmac = 3;

    pub fn clear_hmac(&mut self) {
        self.hmac.clear();
    }

    pub fn has_hmac(&self) -> bool {
        self.hmac.is_some()
    }

    // Param is passed by value, moved
    pub fn set_hmac(&mut self, v: ::std::vec::Vec<u8>) {
        self.hmac = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_hmac(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.hmac.is_none() {
            self.hmac.set_default();
        }
        self.hmac.as_mut().unwrap()
    }

    // Take field
    pub fn take_hmac(&mut self) -> ::std::vec::Vec<u8> {
        self.hmac.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_hmac(&self) -> &[u8] {
        match self.hmac.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_hmac_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.hmac
    }

    fn mut_hmac_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.hmac
    }
}

impl ::protobuf::Message for EncryptedMessage {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.nonce)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.message)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.hmac)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.nonce.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.hmac.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.nonce.as_ref() {
            os.write_bytes(1, &v)?;
        }
        if let Some(ref v) = self.message.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.hmac.as_ref() {
            os.write_bytes(3, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EncryptedMessage {
    fn new() -> EncryptedMessage {
        EncryptedMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<EncryptedMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "nonce",
                    EncryptedMessage::get_nonce_for_reflect,
                    EncryptedMessage::mut_nonce_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "message",
                    EncryptedMessage::get_message_for_reflect,
                    EncryptedMessage::mut_message_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "hmac",
                    EncryptedMessage::get_hmac_for_reflect,
                    EncryptedMessage::mut_hmac_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EncryptedMessage>(
                    "EncryptedMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EncryptedMessage {
    fn clear(&mut self) {
        self.clear_nonce();
        self.clear_message();
        self.clear_hmac();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EncryptedMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EncryptedMessage {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DecryptMessage {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    nonce: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    message: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    hmac: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DecryptMessage {}

impl DecryptMessage {
    pub fn new() -> DecryptMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DecryptMessage {
        static mut instance: ::protobuf::lazy::Lazy<DecryptMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DecryptMessage,
        };
        unsafe {
            instance.get(DecryptMessage::new)
        }
    }

    // repeated uint32 address_n = 1;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // optional bytes nonce = 2;

    pub fn clear_nonce(&mut self) {
        self.nonce.clear();
    }

    pub fn has_nonce(&self) -> bool {
        self.nonce.is_some()
    }

    // Param is passed by value, moved
    pub fn set_nonce(&mut self, v: ::std::vec::Vec<u8>) {
        self.nonce = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_nonce(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.nonce.is_none() {
            self.nonce.set_default();
        }
        self.nonce.as_mut().unwrap()
    }

    // Take field
    pub fn take_nonce(&mut self) -> ::std::vec::Vec<u8> {
        self.nonce.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_nonce(&self) -> &[u8] {
        match self.nonce.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_nonce_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.nonce
    }

    fn mut_nonce_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.nonce
    }

    // optional bytes message = 3;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::vec::Vec<u8>) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::vec::Vec<u8> {
        self.message.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_message(&self) -> &[u8] {
        match self.message.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.message
    }

    // optional bytes hmac = 4;

    pub fn clear_hmac(&mut self) {
        self.hmac.clear();
    }

    pub fn has_hmac(&self) -> bool {
        self.hmac.is_some()
    }

    // Param is passed by value, moved
    pub fn set_hmac(&mut self, v: ::std::vec::Vec<u8>) {
        self.hmac = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_hmac(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.hmac.is_none() {
            self.hmac.set_default();
        }
        self.hmac.as_mut().unwrap()
    }

    // Take field
    pub fn take_hmac(&mut self) -> ::std::vec::Vec<u8> {
        self.hmac.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_hmac(&self) -> &[u8] {
        match self.hmac.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_hmac_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.hmac
    }

    fn mut_hmac_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.hmac
    }
}

impl ::protobuf::Message for DecryptMessage {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.nonce)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.message)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.hmac)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(ref v) = self.nonce.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        if let Some(ref v) = self.hmac.as_ref() {
            my_size += ::protobuf::rt::bytes_size(4, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(ref v) = self.nonce.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.message.as_ref() {
            os.write_bytes(3, &v)?;
        }
        if let Some(ref v) = self.hmac.as_ref() {
            os.write_bytes(4, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DecryptMessage {
    fn new() -> DecryptMessage {
        DecryptMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<DecryptMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    DecryptMessage::get_address_n_for_reflect,
                    DecryptMessage::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "nonce",
                    DecryptMessage::get_nonce_for_reflect,
                    DecryptMessage::mut_nonce_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "message",
                    DecryptMessage::get_message_for_reflect,
                    DecryptMessage::mut_message_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "hmac",
                    DecryptMessage::get_hmac_for_reflect,
                    DecryptMessage::mut_hmac_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DecryptMessage>(
                    "DecryptMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DecryptMessage {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_nonce();
        self.clear_message();
        self.clear_hmac();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DecryptMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DecryptMessage {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DecryptedMessage {
    // message fields
    message: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    address: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DecryptedMessage {}

impl DecryptedMessage {
    pub fn new() -> DecryptedMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DecryptedMessage {
        static mut instance: ::protobuf::lazy::Lazy<DecryptedMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DecryptedMessage,
        };
        unsafe {
            instance.get(DecryptedMessage::new)
        }
    }

    // optional bytes message = 1;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::vec::Vec<u8>) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::vec::Vec<u8> {
        self.message.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_message(&self) -> &[u8] {
        match self.message.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.message
    }

    // optional string address = 2;

    pub fn clear_address(&mut self) {
        self.address.clear();
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: ::std::string::String) {
        self.address = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_address(&mut self) -> &mut ::std::string::String {
        if self.address.is_none() {
            self.address.set_default();
        }
        self.address.as_mut().unwrap()
    }

    // Take field
    pub fn take_address(&mut self) -> ::std::string::String {
        self.address.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_address(&self) -> &str {
        match self.address.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_address_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.address
    }
}

impl ::protobuf::Message for DecryptedMessage {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.message)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.address)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        if let Some(ref v) = self.address.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.message.as_ref() {
            os.write_bytes(1, &v)?;
        }
        if let Some(ref v) = self.address.as_ref() {
            os.write_string(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DecryptedMessage {
    fn new() -> DecryptedMessage {
        DecryptedMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<DecryptedMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "message",
                    DecryptedMessage::get_message_for_reflect,
                    DecryptedMessage::mut_message_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "address",
                    DecryptedMessage::get_address_for_reflect,
                    DecryptedMessage::mut_address_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DecryptedMessage>(
                    "DecryptedMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DecryptedMessage {
    fn clear(&mut self) {
        self.clear_message();
        self.clear_address();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DecryptedMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DecryptedMessage {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CipherKeyValue {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    key: ::protobuf::SingularField<::std::string::String>,
    value: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    encrypt: ::std::option::Option<bool>,
    ask_on_encrypt: ::std::option::Option<bool>,
    ask_on_decrypt: ::std::option::Option<bool>,
    iv: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CipherKeyValue {}

impl CipherKeyValue {
    pub fn new() -> CipherKeyValue {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CipherKeyValue {
        static mut instance: ::protobuf::lazy::Lazy<CipherKeyValue> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CipherKeyValue,
        };
        unsafe {
            instance.get(CipherKeyValue::new)
        }
    }

    // repeated uint32 address_n = 1;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // optional string key = 2;

    pub fn clear_key(&mut self) {
        self.key.clear();
    }

    pub fn has_key(&self) -> bool {
        self.key.is_some()
    }

    // Param is passed by value, moved
    pub fn set_key(&mut self, v: ::std::string::String) {
        self.key = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_key(&mut self) -> &mut ::std::string::String {
        if self.key.is_none() {
            self.key.set_default();
        }
        self.key.as_mut().unwrap()
    }

    // Take field
    pub fn take_key(&mut self) -> ::std::string::String {
        self.key.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_key(&self) -> &str {
        match self.key.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_key_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.key
    }

    fn mut_key_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.key
    }

    // optional bytes value = 3;

    pub fn clear_value(&mut self) {
        self.value.clear();
    }

    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }

    // Param is passed by value, moved
    pub fn set_value(&mut self, v: ::std::vec::Vec<u8>) {
        self.value = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_value(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.value.is_none() {
            self.value.set_default();
        }
        self.value.as_mut().unwrap()
    }

    // Take field
    pub fn take_value(&mut self) -> ::std::vec::Vec<u8> {
        self.value.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_value(&self) -> &[u8] {
        match self.value.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_value_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.value
    }

    fn mut_value_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.value
    }

    // optional bool encrypt = 4;

    pub fn clear_encrypt(&mut self) {
        self.encrypt = ::std::option::Option::None;
    }

    pub fn has_encrypt(&self) -> bool {
        self.encrypt.is_some()
    }

    // Param is passed by value, moved
    pub fn set_encrypt(&mut self, v: bool) {
        self.encrypt = ::std::option::Option::Some(v);
    }

    pub fn get_encrypt(&self) -> bool {
        self.encrypt.unwrap_or(false)
    }

    fn get_encrypt_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.encrypt
    }

    fn mut_encrypt_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.encrypt
    }

    // optional bool ask_on_encrypt = 5;

    pub fn clear_ask_on_encrypt(&mut self) {
        self.ask_on_encrypt = ::std::option::Option::None;
    }

    pub fn has_ask_on_encrypt(&self) -> bool {
        self.ask_on_encrypt.is_some()
    }

    // Param is passed by value, moved
    pub fn set_ask_on_encrypt(&mut self, v: bool) {
        self.ask_on_encrypt = ::std::option::Option::Some(v);
    }

    pub fn get_ask_on_encrypt(&self) -> bool {
        self.ask_on_encrypt.unwrap_or(false)
    }

    fn get_ask_on_encrypt_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.ask_on_encrypt
    }

    fn mut_ask_on_encrypt_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.ask_on_encrypt
    }

    // optional bool ask_on_decrypt = 6;

    pub fn clear_ask_on_decrypt(&mut self) {
        self.ask_on_decrypt = ::std::option::Option::None;
    }

    pub fn has_ask_on_decrypt(&self) -> bool {
        self.ask_on_decrypt.is_some()
    }

    // Param is passed by value, moved
    pub fn set_ask_on_decrypt(&mut self, v: bool) {
        self.ask_on_decrypt = ::std::option::Option::Some(v);
    }

    pub fn get_ask_on_decrypt(&self) -> bool {
        self.ask_on_decrypt.unwrap_or(false)
    }

    fn get_ask_on_decrypt_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.ask_on_decrypt
    }

    fn mut_ask_on_decrypt_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.ask_on_decrypt
    }

    // optional bytes iv = 7;

    pub fn clear_iv(&mut self) {
        self.iv.clear();
    }

    pub fn has_iv(&self) -> bool {
        self.iv.is_some()
    }

    // Param is passed by value, moved
    pub fn set_iv(&mut self, v: ::std::vec::Vec<u8>) {
        self.iv = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_iv(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.iv.is_none() {
            self.iv.set_default();
        }
        self.iv.as_mut().unwrap()
    }

    // Take field
    pub fn take_iv(&mut self) -> ::std::vec::Vec<u8> {
        self.iv.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_iv(&self) -> &[u8] {
        match self.iv.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_iv_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.iv
    }

    fn mut_iv_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.iv
    }
}

impl ::protobuf::Message for CipherKeyValue {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.key)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.value)?;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.encrypt = ::std::option::Option::Some(tmp);
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.ask_on_encrypt = ::std::option::Option::Some(tmp);
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.ask_on_decrypt = ::std::option::Option::Some(tmp);
                },
                7 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.iv)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(ref v) = self.key.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        if let Some(ref v) = self.value.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        if let Some(v) = self.encrypt {
            my_size += 2;
        }
        if let Some(v) = self.ask_on_encrypt {
            my_size += 2;
        }
        if let Some(v) = self.ask_on_decrypt {
            my_size += 2;
        }
        if let Some(ref v) = self.iv.as_ref() {
            my_size += ::protobuf::rt::bytes_size(7, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(ref v) = self.key.as_ref() {
            os.write_string(2, &v)?;
        }
        if let Some(ref v) = self.value.as_ref() {
            os.write_bytes(3, &v)?;
        }
        if let Some(v) = self.encrypt {
            os.write_bool(4, v)?;
        }
        if let Some(v) = self.ask_on_encrypt {
            os.write_bool(5, v)?;
        }
        if let Some(v) = self.ask_on_decrypt {
            os.write_bool(6, v)?;
        }
        if let Some(ref v) = self.iv.as_ref() {
            os.write_bytes(7, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for CipherKeyValue {
    fn new() -> CipherKeyValue {
        CipherKeyValue::new()
    }

    fn descriptor_static(_: ::std::option::Option<CipherKeyValue>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    CipherKeyValue::get_address_n_for_reflect,
                    CipherKeyValue::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "key",
                    CipherKeyValue::get_key_for_reflect,
                    CipherKeyValue::mut_key_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "value",
                    CipherKeyValue::get_value_for_reflect,
                    CipherKeyValue::mut_value_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "encrypt",
                    CipherKeyValue::get_encrypt_for_reflect,
                    CipherKeyValue::mut_encrypt_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "ask_on_encrypt",
                    CipherKeyValue::get_ask_on_encrypt_for_reflect,
                    CipherKeyValue::mut_ask_on_encrypt_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "ask_on_decrypt",
                    CipherKeyValue::get_ask_on_decrypt_for_reflect,
                    CipherKeyValue::mut_ask_on_decrypt_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "iv",
                    CipherKeyValue::get_iv_for_reflect,
                    CipherKeyValue::mut_iv_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<CipherKeyValue>(
                    "CipherKeyValue",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CipherKeyValue {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_key();
        self.clear_value();
        self.clear_encrypt();
        self.clear_ask_on_encrypt();
        self.clear_ask_on_decrypt();
        self.clear_iv();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CipherKeyValue {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CipherKeyValue {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CipheredKeyValue {
    // message fields
    value: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CipheredKeyValue {}

impl CipheredKeyValue {
    pub fn new() -> CipheredKeyValue {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CipheredKeyValue {
        static mut instance: ::protobuf::lazy::Lazy<CipheredKeyValue> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CipheredKeyValue,
        };
        unsafe {
            instance.get(CipheredKeyValue::new)
        }
    }

    // optional bytes value = 1;

    pub fn clear_value(&mut self) {
        self.value.clear();
    }

    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }

    // Param is passed by value, moved
    pub fn set_value(&mut self, v: ::std::vec::Vec<u8>) {
        self.value = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_value(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.value.is_none() {
            self.value.set_default();
        }
        self.value.as_mut().unwrap()
    }

    // Take field
    pub fn take_value(&mut self) -> ::std::vec::Vec<u8> {
        self.value.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_value(&self) -> &[u8] {
        match self.value.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_value_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.value
    }

    fn mut_value_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.value
    }
}

impl ::protobuf::Message for CipheredKeyValue {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.value)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.value.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.value.as_ref() {
            os.write_bytes(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for CipheredKeyValue {
    fn new() -> CipheredKeyValue {
        CipheredKeyValue::new()
    }

    fn descriptor_static(_: ::std::option::Option<CipheredKeyValue>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "value",
                    CipheredKeyValue::get_value_for_reflect,
                    CipheredKeyValue::mut_value_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<CipheredKeyValue>(
                    "CipheredKeyValue",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CipheredKeyValue {
    fn clear(&mut self) {
        self.clear_value();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CipheredKeyValue {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CipheredKeyValue {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EstimateTxSize {
    // message fields
    outputs_count: ::std::option::Option<u32>,
    inputs_count: ::std::option::Option<u32>,
    coin_name: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EstimateTxSize {}

impl EstimateTxSize {
    pub fn new() -> EstimateTxSize {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EstimateTxSize {
        static mut instance: ::protobuf::lazy::Lazy<EstimateTxSize> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EstimateTxSize,
        };
        unsafe {
            instance.get(EstimateTxSize::new)
        }
    }

    // required uint32 outputs_count = 1;

    pub fn clear_outputs_count(&mut self) {
        self.outputs_count = ::std::option::Option::None;
    }

    pub fn has_outputs_count(&self) -> bool {
        self.outputs_count.is_some()
    }

    // Param is passed by value, moved
    pub fn set_outputs_count(&mut self, v: u32) {
        self.outputs_count = ::std::option::Option::Some(v);
    }

    pub fn get_outputs_count(&self) -> u32 {
        self.outputs_count.unwrap_or(0)
    }

    fn get_outputs_count_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.outputs_count
    }

    fn mut_outputs_count_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.outputs_count
    }

    // required uint32 inputs_count = 2;

    pub fn clear_inputs_count(&mut self) {
        self.inputs_count = ::std::option::Option::None;
    }

    pub fn has_inputs_count(&self) -> bool {
        self.inputs_count.is_some()
    }

    // Param is passed by value, moved
    pub fn set_inputs_count(&mut self, v: u32) {
        self.inputs_count = ::std::option::Option::Some(v);
    }

    pub fn get_inputs_count(&self) -> u32 {
        self.inputs_count.unwrap_or(0)
    }

    fn get_inputs_count_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.inputs_count
    }

    fn mut_inputs_count_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.inputs_count
    }

    // optional string coin_name = 3;

    pub fn clear_coin_name(&mut self) {
        self.coin_name.clear();
    }

    pub fn has_coin_name(&self) -> bool {
        self.coin_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_name(&mut self, v: ::std::string::String) {
        self.coin_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_name(&mut self) -> &mut ::std::string::String {
        if self.coin_name.is_none() {
            self.coin_name.set_default();
        }
        self.coin_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_name(&mut self) -> ::std::string::String {
        self.coin_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_name(&self) -> &str {
        match self.coin_name.as_ref() {
            Some(v) => &v,
            None => "Bitcoin",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }
}

impl ::protobuf::Message for EstimateTxSize {
    fn is_initialized(&self) -> bool {
        if self.outputs_count.is_none() {
            return false;
        }
        if self.inputs_count.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.outputs_count = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.inputs_count = ::std::option::Option::Some(tmp);
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.outputs_count {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.inputs_count {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.outputs_count {
            os.write_uint32(1, v)?;
        }
        if let Some(v) = self.inputs_count {
            os.write_uint32(2, v)?;
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(3, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EstimateTxSize {
    fn new() -> EstimateTxSize {
        EstimateTxSize::new()
    }

    fn descriptor_static(_: ::std::option::Option<EstimateTxSize>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "outputs_count",
                    EstimateTxSize::get_outputs_count_for_reflect,
                    EstimateTxSize::mut_outputs_count_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "inputs_count",
                    EstimateTxSize::get_inputs_count_for_reflect,
                    EstimateTxSize::mut_inputs_count_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    EstimateTxSize::get_coin_name_for_reflect,
                    EstimateTxSize::mut_coin_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EstimateTxSize>(
                    "EstimateTxSize",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EstimateTxSize {
    fn clear(&mut self) {
        self.clear_outputs_count();
        self.clear_inputs_count();
        self.clear_coin_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EstimateTxSize {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EstimateTxSize {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TxSize {
    // message fields
    tx_size: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TxSize {}

impl TxSize {
    pub fn new() -> TxSize {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TxSize {
        static mut instance: ::protobuf::lazy::Lazy<TxSize> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TxSize,
        };
        unsafe {
            instance.get(TxSize::new)
        }
    }

    // optional uint32 tx_size = 1;

    pub fn clear_tx_size(&mut self) {
        self.tx_size = ::std::option::Option::None;
    }

    pub fn has_tx_size(&self) -> bool {
        self.tx_size.is_some()
    }

    // Param is passed by value, moved
    pub fn set_tx_size(&mut self, v: u32) {
        self.tx_size = ::std::option::Option::Some(v);
    }

    pub fn get_tx_size(&self) -> u32 {
        self.tx_size.unwrap_or(0)
    }

    fn get_tx_size_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.tx_size
    }

    fn mut_tx_size_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.tx_size
    }
}

impl ::protobuf::Message for TxSize {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.tx_size = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.tx_size {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.tx_size {
            os.write_uint32(1, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for TxSize {
    fn new() -> TxSize {
        TxSize::new()
    }

    fn descriptor_static(_: ::std::option::Option<TxSize>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "tx_size",
                    TxSize::get_tx_size_for_reflect,
                    TxSize::mut_tx_size_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TxSize>(
                    "TxSize",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TxSize {
    fn clear(&mut self) {
        self.clear_tx_size();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TxSize {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TxSize {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SignTx {
    // message fields
    outputs_count: ::std::option::Option<u32>,
    inputs_count: ::std::option::Option<u32>,
    coin_name: ::protobuf::SingularField<::std::string::String>,
    version: ::std::option::Option<u32>,
    lock_time: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for SignTx {}

impl SignTx {
    pub fn new() -> SignTx {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static SignTx {
        static mut instance: ::protobuf::lazy::Lazy<SignTx> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const SignTx,
        };
        unsafe {
            instance.get(SignTx::new)
        }
    }

    // required uint32 outputs_count = 1;

    pub fn clear_outputs_count(&mut self) {
        self.outputs_count = ::std::option::Option::None;
    }

    pub fn has_outputs_count(&self) -> bool {
        self.outputs_count.is_some()
    }

    // Param is passed by value, moved
    pub fn set_outputs_count(&mut self, v: u32) {
        self.outputs_count = ::std::option::Option::Some(v);
    }

    pub fn get_outputs_count(&self) -> u32 {
        self.outputs_count.unwrap_or(0)
    }

    fn get_outputs_count_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.outputs_count
    }

    fn mut_outputs_count_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.outputs_count
    }

    // required uint32 inputs_count = 2;

    pub fn clear_inputs_count(&mut self) {
        self.inputs_count = ::std::option::Option::None;
    }

    pub fn has_inputs_count(&self) -> bool {
        self.inputs_count.is_some()
    }

    // Param is passed by value, moved
    pub fn set_inputs_count(&mut self, v: u32) {
        self.inputs_count = ::std::option::Option::Some(v);
    }

    pub fn get_inputs_count(&self) -> u32 {
        self.inputs_count.unwrap_or(0)
    }

    fn get_inputs_count_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.inputs_count
    }

    fn mut_inputs_count_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.inputs_count
    }

    // optional string coin_name = 3;

    pub fn clear_coin_name(&mut self) {
        self.coin_name.clear();
    }

    pub fn has_coin_name(&self) -> bool {
        self.coin_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_name(&mut self, v: ::std::string::String) {
        self.coin_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_name(&mut self) -> &mut ::std::string::String {
        if self.coin_name.is_none() {
            self.coin_name.set_default();
        }
        self.coin_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_name(&mut self) -> ::std::string::String {
        self.coin_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_name(&self) -> &str {
        match self.coin_name.as_ref() {
            Some(v) => &v,
            None => "Bitcoin",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }

    // optional uint32 version = 4;

    pub fn clear_version(&mut self) {
        self.version = ::std::option::Option::None;
    }

    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }

    // Param is passed by value, moved
    pub fn set_version(&mut self, v: u32) {
        self.version = ::std::option::Option::Some(v);
    }

    pub fn get_version(&self) -> u32 {
        self.version.unwrap_or(1u32)
    }

    fn get_version_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.version
    }

    fn mut_version_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.version
    }

    // optional uint32 lock_time = 5;

    pub fn clear_lock_time(&mut self) {
        self.lock_time = ::std::option::Option::None;
    }

    pub fn has_lock_time(&self) -> bool {
        self.lock_time.is_some()
    }

    // Param is passed by value, moved
    pub fn set_lock_time(&mut self, v: u32) {
        self.lock_time = ::std::option::Option::Some(v);
    }

    pub fn get_lock_time(&self) -> u32 {
        self.lock_time.unwrap_or(0u32)
    }

    fn get_lock_time_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.lock_time
    }

    fn mut_lock_time_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.lock_time
    }
}

impl ::protobuf::Message for SignTx {
    fn is_initialized(&self) -> bool {
        if self.outputs_count.is_none() {
            return false;
        }
        if self.inputs_count.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.outputs_count = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.inputs_count = ::std::option::Option::Some(tmp);
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.version = ::std::option::Option::Some(tmp);
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.lock_time = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.outputs_count {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.inputs_count {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        if let Some(v) = self.version {
            my_size += ::protobuf::rt::value_size(4, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.lock_time {
            my_size += ::protobuf::rt::value_size(5, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.outputs_count {
            os.write_uint32(1, v)?;
        }
        if let Some(v) = self.inputs_count {
            os.write_uint32(2, v)?;
        }
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(3, &v)?;
        }
        if let Some(v) = self.version {
            os.write_uint32(4, v)?;
        }
        if let Some(v) = self.lock_time {
            os.write_uint32(5, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for SignTx {
    fn new() -> SignTx {
        SignTx::new()
    }

    fn descriptor_static(_: ::std::option::Option<SignTx>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "outputs_count",
                    SignTx::get_outputs_count_for_reflect,
                    SignTx::mut_outputs_count_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "inputs_count",
                    SignTx::get_inputs_count_for_reflect,
                    SignTx::mut_inputs_count_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    SignTx::get_coin_name_for_reflect,
                    SignTx::mut_coin_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "version",
                    SignTx::get_version_for_reflect,
                    SignTx::mut_version_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "lock_time",
                    SignTx::get_lock_time_for_reflect,
                    SignTx::mut_lock_time_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<SignTx>(
                    "SignTx",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for SignTx {
    fn clear(&mut self) {
        self.clear_outputs_count();
        self.clear_inputs_count();
        self.clear_coin_name();
        self.clear_version();
        self.clear_lock_time();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SignTx {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SignTx {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SimpleSignTx {
    // message fields
    inputs: ::protobuf::RepeatedField<super::types::TxInputType>,
    outputs: ::protobuf::RepeatedField<super::types::TxOutputType>,
    transactions: ::protobuf::RepeatedField<super::types::TransactionType>,
    coin_name: ::protobuf::SingularField<::std::string::String>,
    version: ::std::option::Option<u32>,
    lock_time: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for SimpleSignTx {}

impl SimpleSignTx {
    pub fn new() -> SimpleSignTx {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static SimpleSignTx {
        static mut instance: ::protobuf::lazy::Lazy<SimpleSignTx> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const SimpleSignTx,
        };
        unsafe {
            instance.get(SimpleSignTx::new)
        }
    }

    // repeated .TxInputType inputs = 1;

    pub fn clear_inputs(&mut self) {
        self.inputs.clear();
    }

    // Param is passed by value, moved
    pub fn set_inputs(&mut self, v: ::protobuf::RepeatedField<super::types::TxInputType>) {
        self.inputs = v;
    }

    // Mutable pointer to the field.
    pub fn mut_inputs(&mut self) -> &mut ::protobuf::RepeatedField<super::types::TxInputType> {
        &mut self.inputs
    }

    // Take field
    pub fn take_inputs(&mut self) -> ::protobuf::RepeatedField<super::types::TxInputType> {
        ::std::mem::replace(&mut self.inputs, ::protobuf::RepeatedField::new())
    }

    pub fn get_inputs(&self) -> &[super::types::TxInputType] {
        &self.inputs
    }

    fn get_inputs_for_reflect(&self) -> &::protobuf::RepeatedField<super::types::TxInputType> {
        &self.inputs
    }

    fn mut_inputs_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<super::types::TxInputType> {
        &mut self.inputs
    }

    // repeated .TxOutputType outputs = 2;

    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }

    // Param is passed by value, moved
    pub fn set_outputs(&mut self, v: ::protobuf::RepeatedField<super::types::TxOutputType>) {
        self.outputs = v;
    }

    // Mutable pointer to the field.
    pub fn mut_outputs(&mut self) -> &mut ::protobuf::RepeatedField<super::types::TxOutputType> {
        &mut self.outputs
    }

    // Take field
    pub fn take_outputs(&mut self) -> ::protobuf::RepeatedField<super::types::TxOutputType> {
        ::std::mem::replace(&mut self.outputs, ::protobuf::RepeatedField::new())
    }

    pub fn get_outputs(&self) -> &[super::types::TxOutputType] {
        &self.outputs
    }

    fn get_outputs_for_reflect(&self) -> &::protobuf::RepeatedField<super::types::TxOutputType> {
        &self.outputs
    }

    fn mut_outputs_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<super::types::TxOutputType> {
        &mut self.outputs
    }

    // repeated .TransactionType transactions = 3;

    pub fn clear_transactions(&mut self) {
        self.transactions.clear();
    }

    // Param is passed by value, moved
    pub fn set_transactions(&mut self, v: ::protobuf::RepeatedField<super::types::TransactionType>) {
        self.transactions = v;
    }

    // Mutable pointer to the field.
    pub fn mut_transactions(&mut self) -> &mut ::protobuf::RepeatedField<super::types::TransactionType> {
        &mut self.transactions
    }

    // Take field
    pub fn take_transactions(&mut self) -> ::protobuf::RepeatedField<super::types::TransactionType> {
        ::std::mem::replace(&mut self.transactions, ::protobuf::RepeatedField::new())
    }

    pub fn get_transactions(&self) -> &[super::types::TransactionType] {
        &self.transactions
    }

    fn get_transactions_for_reflect(&self) -> &::protobuf::RepeatedField<super::types::TransactionType> {
        &self.transactions
    }

    fn mut_transactions_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<super::types::TransactionType> {
        &mut self.transactions
    }

    // optional string coin_name = 4;

    pub fn clear_coin_name(&mut self) {
        self.coin_name.clear();
    }

    pub fn has_coin_name(&self) -> bool {
        self.coin_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_name(&mut self, v: ::std::string::String) {
        self.coin_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_name(&mut self) -> &mut ::std::string::String {
        if self.coin_name.is_none() {
            self.coin_name.set_default();
        }
        self.coin_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_name(&mut self) -> ::std::string::String {
        self.coin_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_name(&self) -> &str {
        match self.coin_name.as_ref() {
            Some(v) => &v,
            None => "Bitcoin",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }

    // optional uint32 version = 5;

    pub fn clear_version(&mut self) {
        self.version = ::std::option::Option::None;
    }

    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }

    // Param is passed by value, moved
    pub fn set_version(&mut self, v: u32) {
        self.version = ::std::option::Option::Some(v);
    }

    pub fn get_version(&self) -> u32 {
        self.version.unwrap_or(1u32)
    }

    fn get_version_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.version
    }

    fn mut_version_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.version
    }

    // optional uint32 lock_time = 6;

    pub fn clear_lock_time(&mut self) {
        self.lock_time = ::std::option::Option::None;
    }

    pub fn has_lock_time(&self) -> bool {
        self.lock_time.is_some()
    }

    // Param is passed by value, moved
    pub fn set_lock_time(&mut self, v: u32) {
        self.lock_time = ::std::option::Option::Some(v);
    }

    pub fn get_lock_time(&self) -> u32 {
        self.lock_time.unwrap_or(0u32)
    }

    fn get_lock_time_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.lock_time
    }

    fn mut_lock_time_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.lock_time
    }
}

impl ::protobuf::Message for SimpleSignTx {
    fn is_initialized(&self) -> bool {
        for v in &self.inputs {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.outputs {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.transactions {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.inputs)?;
                },
                2 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.outputs)?;
                },
                3 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.transactions)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.version = ::std::option::Option::Some(tmp);
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.lock_time = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.inputs {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        for value in &self.outputs {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        for value in &self.transactions {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(4, &v);
        }
        if let Some(v) = self.version {
            my_size += ::protobuf::rt::value_size(5, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.lock_time {
            my_size += ::protobuf::rt::value_size(6, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.inputs {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        for v in &self.outputs {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        for v in &self.transactions {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(4, &v)?;
        }
        if let Some(v) = self.version {
            os.write_uint32(5, v)?;
        }
        if let Some(v) = self.lock_time {
            os.write_uint32(6, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for SimpleSignTx {
    fn new() -> SimpleSignTx {
        SimpleSignTx::new()
    }

    fn descriptor_static(_: ::std::option::Option<SimpleSignTx>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::TxInputType>>(
                    "inputs",
                    SimpleSignTx::get_inputs_for_reflect,
                    SimpleSignTx::mut_inputs_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::TxOutputType>>(
                    "outputs",
                    SimpleSignTx::get_outputs_for_reflect,
                    SimpleSignTx::mut_outputs_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::TransactionType>>(
                    "transactions",
                    SimpleSignTx::get_transactions_for_reflect,
                    SimpleSignTx::mut_transactions_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    SimpleSignTx::get_coin_name_for_reflect,
                    SimpleSignTx::mut_coin_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "version",
                    SimpleSignTx::get_version_for_reflect,
                    SimpleSignTx::mut_version_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "lock_time",
                    SimpleSignTx::get_lock_time_for_reflect,
                    SimpleSignTx::mut_lock_time_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<SimpleSignTx>(
                    "SimpleSignTx",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for SimpleSignTx {
    fn clear(&mut self) {
        self.clear_inputs();
        self.clear_outputs();
        self.clear_transactions();
        self.clear_coin_name();
        self.clear_version();
        self.clear_lock_time();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SimpleSignTx {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SimpleSignTx {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TxRequest {
    // message fields
    request_type: ::std::option::Option<super::types::RequestType>,
    details: ::protobuf::SingularPtrField<super::types::TxRequestDetailsType>,
    serialized: ::protobuf::SingularPtrField<super::types::TxRequestSerializedType>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TxRequest {}

impl TxRequest {
    pub fn new() -> TxRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TxRequest {
        static mut instance: ::protobuf::lazy::Lazy<TxRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TxRequest,
        };
        unsafe {
            instance.get(TxRequest::new)
        }
    }

    // optional .RequestType request_type = 1;

    pub fn clear_request_type(&mut self) {
        self.request_type = ::std::option::Option::None;
    }

    pub fn has_request_type(&self) -> bool {
        self.request_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_request_type(&mut self, v: super::types::RequestType) {
        self.request_type = ::std::option::Option::Some(v);
    }

    pub fn get_request_type(&self) -> super::types::RequestType {
        self.request_type.unwrap_or(super::types::RequestType::TXINPUT)
    }

    fn get_request_type_for_reflect(&self) -> &::std::option::Option<super::types::RequestType> {
        &self.request_type
    }

    fn mut_request_type_for_reflect(&mut self) -> &mut ::std::option::Option<super::types::RequestType> {
        &mut self.request_type
    }

    // optional .TxRequestDetailsType details = 2;

    pub fn clear_details(&mut self) {
        self.details.clear();
    }

    pub fn has_details(&self) -> bool {
        self.details.is_some()
    }

    // Param is passed by value, moved
    pub fn set_details(&mut self, v: super::types::TxRequestDetailsType) {
        self.details = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_details(&mut self) -> &mut super::types::TxRequestDetailsType {
        if self.details.is_none() {
            self.details.set_default();
        }
        self.details.as_mut().unwrap()
    }

    // Take field
    pub fn take_details(&mut self) -> super::types::TxRequestDetailsType {
        self.details.take().unwrap_or_else(|| super::types::TxRequestDetailsType::new())
    }

    pub fn get_details(&self) -> &super::types::TxRequestDetailsType {
        self.details.as_ref().unwrap_or_else(|| super::types::TxRequestDetailsType::default_instance())
    }

    fn get_details_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::TxRequestDetailsType> {
        &self.details
    }

    fn mut_details_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::TxRequestDetailsType> {
        &mut self.details
    }

    // optional .TxRequestSerializedType serialized = 3;

    pub fn clear_serialized(&mut self) {
        self.serialized.clear();
    }

    pub fn has_serialized(&self) -> bool {
        self.serialized.is_some()
    }

    // Param is passed by value, moved
    pub fn set_serialized(&mut self, v: super::types::TxRequestSerializedType) {
        self.serialized = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_serialized(&mut self) -> &mut super::types::TxRequestSerializedType {
        if self.serialized.is_none() {
            self.serialized.set_default();
        }
        self.serialized.as_mut().unwrap()
    }

    // Take field
    pub fn take_serialized(&mut self) -> super::types::TxRequestSerializedType {
        self.serialized.take().unwrap_or_else(|| super::types::TxRequestSerializedType::new())
    }

    pub fn get_serialized(&self) -> &super::types::TxRequestSerializedType {
        self.serialized.as_ref().unwrap_or_else(|| super::types::TxRequestSerializedType::default_instance())
    }

    fn get_serialized_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::TxRequestSerializedType> {
        &self.serialized
    }

    fn mut_serialized_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::TxRequestSerializedType> {
        &mut self.serialized
    }
}

impl ::protobuf::Message for TxRequest {
    fn is_initialized(&self) -> bool {
        for v in &self.details {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.serialized {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.request_type = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.details)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.serialized)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.request_type {
            my_size += ::protobuf::rt::enum_size(1, v);
        }
        if let Some(ref v) = self.details.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.serialized.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.request_type {
            os.write_enum(1, v.value())?;
        }
        if let Some(ref v) = self.details.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.serialized.as_ref() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for TxRequest {
    fn new() -> TxRequest {
        TxRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<TxRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<super::types::RequestType>>(
                    "request_type",
                    TxRequest::get_request_type_for_reflect,
                    TxRequest::mut_request_type_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::TxRequestDetailsType>>(
                    "details",
                    TxRequest::get_details_for_reflect,
                    TxRequest::mut_details_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::TxRequestSerializedType>>(
                    "serialized",
                    TxRequest::get_serialized_for_reflect,
                    TxRequest::mut_serialized_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TxRequest>(
                    "TxRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TxRequest {
    fn clear(&mut self) {
        self.clear_request_type();
        self.clear_details();
        self.clear_serialized();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TxRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TxRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TxAck {
    // message fields
    tx: ::protobuf::SingularPtrField<super::types::TransactionType>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TxAck {}

impl TxAck {
    pub fn new() -> TxAck {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TxAck {
        static mut instance: ::protobuf::lazy::Lazy<TxAck> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TxAck,
        };
        unsafe {
            instance.get(TxAck::new)
        }
    }

    // optional .TransactionType tx = 1;

    pub fn clear_tx(&mut self) {
        self.tx.clear();
    }

    pub fn has_tx(&self) -> bool {
        self.tx.is_some()
    }

    // Param is passed by value, moved
    pub fn set_tx(&mut self, v: super::types::TransactionType) {
        self.tx = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_tx(&mut self) -> &mut super::types::TransactionType {
        if self.tx.is_none() {
            self.tx.set_default();
        }
        self.tx.as_mut().unwrap()
    }

    // Take field
    pub fn take_tx(&mut self) -> super::types::TransactionType {
        self.tx.take().unwrap_or_else(|| super::types::TransactionType::new())
    }

    pub fn get_tx(&self) -> &super::types::TransactionType {
        self.tx.as_ref().unwrap_or_else(|| super::types::TransactionType::default_instance())
    }

    fn get_tx_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::TransactionType> {
        &self.tx
    }

    fn mut_tx_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::TransactionType> {
        &mut self.tx
    }
}

impl ::protobuf::Message for TxAck {
    fn is_initialized(&self) -> bool {
        for v in &self.tx {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.tx)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.tx.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.tx.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for TxAck {
    fn new() -> TxAck {
        TxAck::new()
    }

    fn descriptor_static(_: ::std::option::Option<TxAck>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::TransactionType>>(
                    "tx",
                    TxAck::get_tx_for_reflect,
                    TxAck::mut_tx_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TxAck>(
                    "TxAck",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TxAck {
    fn clear(&mut self) {
        self.clear_tx();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TxAck {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TxAck {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EthereumSignTx {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    nonce: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    gas_price: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    gas_limit: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    to: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    value: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    data_initial_chunk: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    data_length: ::std::option::Option<u32>,
    chain_id: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EthereumSignTx {}

impl EthereumSignTx {
    pub fn new() -> EthereumSignTx {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EthereumSignTx {
        static mut instance: ::protobuf::lazy::Lazy<EthereumSignTx> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EthereumSignTx,
        };
        unsafe {
            instance.get(EthereumSignTx::new)
        }
    }

    // repeated uint32 address_n = 1;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // optional bytes nonce = 2;

    pub fn clear_nonce(&mut self) {
        self.nonce.clear();
    }

    pub fn has_nonce(&self) -> bool {
        self.nonce.is_some()
    }

    // Param is passed by value, moved
    pub fn set_nonce(&mut self, v: ::std::vec::Vec<u8>) {
        self.nonce = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_nonce(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.nonce.is_none() {
            self.nonce.set_default();
        }
        self.nonce.as_mut().unwrap()
    }

    // Take field
    pub fn take_nonce(&mut self) -> ::std::vec::Vec<u8> {
        self.nonce.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_nonce(&self) -> &[u8] {
        match self.nonce.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_nonce_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.nonce
    }

    fn mut_nonce_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.nonce
    }

    // optional bytes gas_price = 3;

    pub fn clear_gas_price(&mut self) {
        self.gas_price.clear();
    }

    pub fn has_gas_price(&self) -> bool {
        self.gas_price.is_some()
    }

    // Param is passed by value, moved
    pub fn set_gas_price(&mut self, v: ::std::vec::Vec<u8>) {
        self.gas_price = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_gas_price(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.gas_price.is_none() {
            self.gas_price.set_default();
        }
        self.gas_price.as_mut().unwrap()
    }

    // Take field
    pub fn take_gas_price(&mut self) -> ::std::vec::Vec<u8> {
        self.gas_price.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_gas_price(&self) -> &[u8] {
        match self.gas_price.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_gas_price_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.gas_price
    }

    fn mut_gas_price_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.gas_price
    }

    // optional bytes gas_limit = 4;

    pub fn clear_gas_limit(&mut self) {
        self.gas_limit.clear();
    }

    pub fn has_gas_limit(&self) -> bool {
        self.gas_limit.is_some()
    }

    // Param is passed by value, moved
    pub fn set_gas_limit(&mut self, v: ::std::vec::Vec<u8>) {
        self.gas_limit = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_gas_limit(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.gas_limit.is_none() {
            self.gas_limit.set_default();
        }
        self.gas_limit.as_mut().unwrap()
    }

    // Take field
    pub fn take_gas_limit(&mut self) -> ::std::vec::Vec<u8> {
        self.gas_limit.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_gas_limit(&self) -> &[u8] {
        match self.gas_limit.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_gas_limit_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.gas_limit
    }

    fn mut_gas_limit_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.gas_limit
    }

    // optional bytes to = 5;

    pub fn clear_to(&mut self) {
        self.to.clear();
    }

    pub fn has_to(&self) -> bool {
        self.to.is_some()
    }

    // Param is passed by value, moved
    pub fn set_to(&mut self, v: ::std::vec::Vec<u8>) {
        self.to = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_to(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.to.is_none() {
            self.to.set_default();
        }
        self.to.as_mut().unwrap()
    }

    // Take field
    pub fn take_to(&mut self) -> ::std::vec::Vec<u8> {
        self.to.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_to(&self) -> &[u8] {
        match self.to.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_to_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.to
    }

    fn mut_to_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.to
    }

    // optional bytes value = 6;

    pub fn clear_value(&mut self) {
        self.value.clear();
    }

    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }

    // Param is passed by value, moved
    pub fn set_value(&mut self, v: ::std::vec::Vec<u8>) {
        self.value = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_value(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.value.is_none() {
            self.value.set_default();
        }
        self.value.as_mut().unwrap()
    }

    // Take field
    pub fn take_value(&mut self) -> ::std::vec::Vec<u8> {
        self.value.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_value(&self) -> &[u8] {
        match self.value.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_value_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.value
    }

    fn mut_value_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.value
    }

    // optional bytes data_initial_chunk = 7;

    pub fn clear_data_initial_chunk(&mut self) {
        self.data_initial_chunk.clear();
    }

    pub fn has_data_initial_chunk(&self) -> bool {
        self.data_initial_chunk.is_some()
    }

    // Param is passed by value, moved
    pub fn set_data_initial_chunk(&mut self, v: ::std::vec::Vec<u8>) {
        self.data_initial_chunk = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_data_initial_chunk(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.data_initial_chunk.is_none() {
            self.data_initial_chunk.set_default();
        }
        self.data_initial_chunk.as_mut().unwrap()
    }

    // Take field
    pub fn take_data_initial_chunk(&mut self) -> ::std::vec::Vec<u8> {
        self.data_initial_chunk.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_data_initial_chunk(&self) -> &[u8] {
        match self.data_initial_chunk.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_data_initial_chunk_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.data_initial_chunk
    }

    fn mut_data_initial_chunk_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.data_initial_chunk
    }

    // optional uint32 data_length = 8;

    pub fn clear_data_length(&mut self) {
        self.data_length = ::std::option::Option::None;
    }

    pub fn has_data_length(&self) -> bool {
        self.data_length.is_some()
    }

    // Param is passed by value, moved
    pub fn set_data_length(&mut self, v: u32) {
        self.data_length = ::std::option::Option::Some(v);
    }

    pub fn get_data_length(&self) -> u32 {
        self.data_length.unwrap_or(0)
    }

    fn get_data_length_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.data_length
    }

    fn mut_data_length_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.data_length
    }

    // optional uint32 chain_id = 9;

    pub fn clear_chain_id(&mut self) {
        self.chain_id = ::std::option::Option::None;
    }

    pub fn has_chain_id(&self) -> bool {
        self.chain_id.is_some()
    }

    // Param is passed by value, moved
    pub fn set_chain_id(&mut self, v: u32) {
        self.chain_id = ::std::option::Option::Some(v);
    }

    pub fn get_chain_id(&self) -> u32 {
        self.chain_id.unwrap_or(0)
    }

    fn get_chain_id_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.chain_id
    }

    fn mut_chain_id_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.chain_id
    }
}

impl ::protobuf::Message for EthereumSignTx {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.nonce)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.gas_price)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.gas_limit)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.to)?;
                },
                6 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.value)?;
                },
                7 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.data_initial_chunk)?;
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.data_length = ::std::option::Option::Some(tmp);
                },
                9 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.chain_id = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(ref v) = self.nonce.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.gas_price.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        if let Some(ref v) = self.gas_limit.as_ref() {
            my_size += ::protobuf::rt::bytes_size(4, &v);
        }
        if let Some(ref v) = self.to.as_ref() {
            my_size += ::protobuf::rt::bytes_size(5, &v);
        }
        if let Some(ref v) = self.value.as_ref() {
            my_size += ::protobuf::rt::bytes_size(6, &v);
        }
        if let Some(ref v) = self.data_initial_chunk.as_ref() {
            my_size += ::protobuf::rt::bytes_size(7, &v);
        }
        if let Some(v) = self.data_length {
            my_size += ::protobuf::rt::value_size(8, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.chain_id {
            my_size += ::protobuf::rt::value_size(9, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(ref v) = self.nonce.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.gas_price.as_ref() {
            os.write_bytes(3, &v)?;
        }
        if let Some(ref v) = self.gas_limit.as_ref() {
            os.write_bytes(4, &v)?;
        }
        if let Some(ref v) = self.to.as_ref() {
            os.write_bytes(5, &v)?;
        }
        if let Some(ref v) = self.value.as_ref() {
            os.write_bytes(6, &v)?;
        }
        if let Some(ref v) = self.data_initial_chunk.as_ref() {
            os.write_bytes(7, &v)?;
        }
        if let Some(v) = self.data_length {
            os.write_uint32(8, v)?;
        }
        if let Some(v) = self.chain_id {
            os.write_uint32(9, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EthereumSignTx {
    fn new() -> EthereumSignTx {
        EthereumSignTx::new()
    }

    fn descriptor_static(_: ::std::option::Option<EthereumSignTx>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    EthereumSignTx::get_address_n_for_reflect,
                    EthereumSignTx::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "nonce",
                    EthereumSignTx::get_nonce_for_reflect,
                    EthereumSignTx::mut_nonce_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "gas_price",
                    EthereumSignTx::get_gas_price_for_reflect,
                    EthereumSignTx::mut_gas_price_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "gas_limit",
                    EthereumSignTx::get_gas_limit_for_reflect,
                    EthereumSignTx::mut_gas_limit_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "to",
                    EthereumSignTx::get_to_for_reflect,
                    EthereumSignTx::mut_to_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "value",
                    EthereumSignTx::get_value_for_reflect,
                    EthereumSignTx::mut_value_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "data_initial_chunk",
                    EthereumSignTx::get_data_initial_chunk_for_reflect,
                    EthereumSignTx::mut_data_initial_chunk_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "data_length",
                    EthereumSignTx::get_data_length_for_reflect,
                    EthereumSignTx::mut_data_length_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "chain_id",
                    EthereumSignTx::get_chain_id_for_reflect,
                    EthereumSignTx::mut_chain_id_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EthereumSignTx>(
                    "EthereumSignTx",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EthereumSignTx {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_nonce();
        self.clear_gas_price();
        self.clear_gas_limit();
        self.clear_to();
        self.clear_value();
        self.clear_data_initial_chunk();
        self.clear_data_length();
        self.clear_chain_id();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EthereumSignTx {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EthereumSignTx {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EthereumTxRequest {
    // message fields
    data_length: ::std::option::Option<u32>,
    signature_v: ::std::option::Option<u32>,
    signature_r: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    signature_s: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EthereumTxRequest {}

impl EthereumTxRequest {
    pub fn new() -> EthereumTxRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EthereumTxRequest {
        static mut instance: ::protobuf::lazy::Lazy<EthereumTxRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EthereumTxRequest,
        };
        unsafe {
            instance.get(EthereumTxRequest::new)
        }
    }

    // optional uint32 data_length = 1;

    pub fn clear_data_length(&mut self) {
        self.data_length = ::std::option::Option::None;
    }

    pub fn has_data_length(&self) -> bool {
        self.data_length.is_some()
    }

    // Param is passed by value, moved
    pub fn set_data_length(&mut self, v: u32) {
        self.data_length = ::std::option::Option::Some(v);
    }

    pub fn get_data_length(&self) -> u32 {
        self.data_length.unwrap_or(0)
    }

    fn get_data_length_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.data_length
    }

    fn mut_data_length_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.data_length
    }

    // optional uint32 signature_v = 2;

    pub fn clear_signature_v(&mut self) {
        self.signature_v = ::std::option::Option::None;
    }

    pub fn has_signature_v(&self) -> bool {
        self.signature_v.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature_v(&mut self, v: u32) {
        self.signature_v = ::std::option::Option::Some(v);
    }

    pub fn get_signature_v(&self) -> u32 {
        self.signature_v.unwrap_or(0)
    }

    fn get_signature_v_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.signature_v
    }

    fn mut_signature_v_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.signature_v
    }

    // optional bytes signature_r = 3;

    pub fn clear_signature_r(&mut self) {
        self.signature_r.clear();
    }

    pub fn has_signature_r(&self) -> bool {
        self.signature_r.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature_r(&mut self, v: ::std::vec::Vec<u8>) {
        self.signature_r = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signature_r(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.signature_r.is_none() {
            self.signature_r.set_default();
        }
        self.signature_r.as_mut().unwrap()
    }

    // Take field
    pub fn take_signature_r(&mut self) -> ::std::vec::Vec<u8> {
        self.signature_r.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_signature_r(&self) -> &[u8] {
        match self.signature_r.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_signature_r_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.signature_r
    }

    fn mut_signature_r_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.signature_r
    }

    // optional bytes signature_s = 4;

    pub fn clear_signature_s(&mut self) {
        self.signature_s.clear();
    }

    pub fn has_signature_s(&self) -> bool {
        self.signature_s.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature_s(&mut self, v: ::std::vec::Vec<u8>) {
        self.signature_s = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signature_s(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.signature_s.is_none() {
            self.signature_s.set_default();
        }
        self.signature_s.as_mut().unwrap()
    }

    // Take field
    pub fn take_signature_s(&mut self) -> ::std::vec::Vec<u8> {
        self.signature_s.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_signature_s(&self) -> &[u8] {
        match self.signature_s.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_signature_s_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.signature_s
    }

    fn mut_signature_s_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.signature_s
    }
}

impl ::protobuf::Message for EthereumTxRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.data_length = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.signature_v = ::std::option::Option::Some(tmp);
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.signature_r)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.signature_s)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.data_length {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.signature_v {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.signature_r.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        if let Some(ref v) = self.signature_s.as_ref() {
            my_size += ::protobuf::rt::bytes_size(4, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.data_length {
            os.write_uint32(1, v)?;
        }
        if let Some(v) = self.signature_v {
            os.write_uint32(2, v)?;
        }
        if let Some(ref v) = self.signature_r.as_ref() {
            os.write_bytes(3, &v)?;
        }
        if let Some(ref v) = self.signature_s.as_ref() {
            os.write_bytes(4, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EthereumTxRequest {
    fn new() -> EthereumTxRequest {
        EthereumTxRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<EthereumTxRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "data_length",
                    EthereumTxRequest::get_data_length_for_reflect,
                    EthereumTxRequest::mut_data_length_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "signature_v",
                    EthereumTxRequest::get_signature_v_for_reflect,
                    EthereumTxRequest::mut_signature_v_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature_r",
                    EthereumTxRequest::get_signature_r_for_reflect,
                    EthereumTxRequest::mut_signature_r_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature_s",
                    EthereumTxRequest::get_signature_s_for_reflect,
                    EthereumTxRequest::mut_signature_s_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EthereumTxRequest>(
                    "EthereumTxRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EthereumTxRequest {
    fn clear(&mut self) {
        self.clear_data_length();
        self.clear_signature_v();
        self.clear_signature_r();
        self.clear_signature_s();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EthereumTxRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EthereumTxRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EthereumTxAck {
    // message fields
    data_chunk: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EthereumTxAck {}

impl EthereumTxAck {
    pub fn new() -> EthereumTxAck {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EthereumTxAck {
        static mut instance: ::protobuf::lazy::Lazy<EthereumTxAck> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EthereumTxAck,
        };
        unsafe {
            instance.get(EthereumTxAck::new)
        }
    }

    // optional bytes data_chunk = 1;

    pub fn clear_data_chunk(&mut self) {
        self.data_chunk.clear();
    }

    pub fn has_data_chunk(&self) -> bool {
        self.data_chunk.is_some()
    }

    // Param is passed by value, moved
    pub fn set_data_chunk(&mut self, v: ::std::vec::Vec<u8>) {
        self.data_chunk = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_data_chunk(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.data_chunk.is_none() {
            self.data_chunk.set_default();
        }
        self.data_chunk.as_mut().unwrap()
    }

    // Take field
    pub fn take_data_chunk(&mut self) -> ::std::vec::Vec<u8> {
        self.data_chunk.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_data_chunk(&self) -> &[u8] {
        match self.data_chunk.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_data_chunk_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.data_chunk
    }

    fn mut_data_chunk_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.data_chunk
    }
}

impl ::protobuf::Message for EthereumTxAck {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.data_chunk)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.data_chunk.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.data_chunk.as_ref() {
            os.write_bytes(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EthereumTxAck {
    fn new() -> EthereumTxAck {
        EthereumTxAck::new()
    }

    fn descriptor_static(_: ::std::option::Option<EthereumTxAck>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "data_chunk",
                    EthereumTxAck::get_data_chunk_for_reflect,
                    EthereumTxAck::mut_data_chunk_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EthereumTxAck>(
                    "EthereumTxAck",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EthereumTxAck {
    fn clear(&mut self) {
        self.clear_data_chunk();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EthereumTxAck {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EthereumTxAck {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EthereumSignMessage {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    message: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EthereumSignMessage {}

impl EthereumSignMessage {
    pub fn new() -> EthereumSignMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EthereumSignMessage {
        static mut instance: ::protobuf::lazy::Lazy<EthereumSignMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EthereumSignMessage,
        };
        unsafe {
            instance.get(EthereumSignMessage::new)
        }
    }

    // repeated uint32 address_n = 1;

    pub fn clear_address_n(&mut self) {
        self.address_n.clear();
    }

    // Param is passed by value, moved
    pub fn set_address_n(&mut self, v: ::std::vec::Vec<u32>) {
        self.address_n = v;
    }

    // Mutable pointer to the field.
    pub fn mut_address_n(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // Take field
    pub fn take_address_n(&mut self) -> ::std::vec::Vec<u32> {
        ::std::mem::replace(&mut self.address_n, ::std::vec::Vec::new())
    }

    pub fn get_address_n(&self) -> &[u32] {
        &self.address_n
    }

    fn get_address_n_for_reflect(&self) -> &::std::vec::Vec<u32> {
        &self.address_n
    }

    fn mut_address_n_for_reflect(&mut self) -> &mut ::std::vec::Vec<u32> {
        &mut self.address_n
    }

    // required bytes message = 2;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::vec::Vec<u8>) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::vec::Vec<u8> {
        self.message.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_message(&self) -> &[u8] {
        match self.message.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.message
    }
}

impl ::protobuf::Message for EthereumSignMessage {
    fn is_initialized(&self) -> bool {
        if self.message.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.message)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(ref v) = self.message.as_ref() {
            os.write_bytes(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EthereumSignMessage {
    fn new() -> EthereumSignMessage {
        EthereumSignMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<EthereumSignMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    EthereumSignMessage::get_address_n_for_reflect,
                    EthereumSignMessage::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "message",
                    EthereumSignMessage::get_message_for_reflect,
                    EthereumSignMessage::mut_message_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EthereumSignMessage>(
                    "EthereumSignMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EthereumSignMessage {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_message();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EthereumSignMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EthereumSignMessage {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EthereumVerifyMessage {
    // message fields
    address: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    signature: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    message: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EthereumVerifyMessage {}

impl EthereumVerifyMessage {
    pub fn new() -> EthereumVerifyMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EthereumVerifyMessage {
        static mut instance: ::protobuf::lazy::Lazy<EthereumVerifyMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EthereumVerifyMessage,
        };
        unsafe {
            instance.get(EthereumVerifyMessage::new)
        }
    }

    // optional bytes address = 1;

    pub fn clear_address(&mut self) {
        self.address.clear();
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: ::std::vec::Vec<u8>) {
        self.address = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_address(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.address.is_none() {
            self.address.set_default();
        }
        self.address.as_mut().unwrap()
    }

    // Take field
    pub fn take_address(&mut self) -> ::std::vec::Vec<u8> {
        self.address.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_address(&self) -> &[u8] {
        match self.address.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_address_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.address
    }

    // optional bytes signature = 2;

    pub fn clear_signature(&mut self) {
        self.signature.clear();
    }

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature(&mut self, v: ::std::vec::Vec<u8>) {
        self.signature = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signature(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.signature.is_none() {
            self.signature.set_default();
        }
        self.signature.as_mut().unwrap()
    }

    // Take field
    pub fn take_signature(&mut self) -> ::std::vec::Vec<u8> {
        self.signature.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_signature(&self) -> &[u8] {
        match self.signature.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_signature_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.signature
    }

    fn mut_signature_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.signature
    }

    // optional bytes message = 3;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::vec::Vec<u8>) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.message.is_none() {
            self.message.set_default();
        }
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::vec::Vec<u8> {
        self.message.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_message(&self) -> &[u8] {
        match self.message.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_message_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.message
    }
}

impl ::protobuf::Message for EthereumVerifyMessage {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.address)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.signature)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.message)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.address.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        if let Some(ref v) = self.signature.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.message.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.address.as_ref() {
            os.write_bytes(1, &v)?;
        }
        if let Some(ref v) = self.signature.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.message.as_ref() {
            os.write_bytes(3, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EthereumVerifyMessage {
    fn new() -> EthereumVerifyMessage {
        EthereumVerifyMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<EthereumVerifyMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "address",
                    EthereumVerifyMessage::get_address_for_reflect,
                    EthereumVerifyMessage::mut_address_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature",
                    EthereumVerifyMessage::get_signature_for_reflect,
                    EthereumVerifyMessage::mut_signature_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "message",
                    EthereumVerifyMessage::get_message_for_reflect,
                    EthereumVerifyMessage::mut_message_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EthereumVerifyMessage>(
                    "EthereumVerifyMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EthereumVerifyMessage {
    fn clear(&mut self) {
        self.clear_address();
        self.clear_signature();
        self.clear_message();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EthereumVerifyMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EthereumVerifyMessage {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EthereumMessageSignature {
    // message fields
    address: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    signature: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EthereumMessageSignature {}

impl EthereumMessageSignature {
    pub fn new() -> EthereumMessageSignature {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EthereumMessageSignature {
        static mut instance: ::protobuf::lazy::Lazy<EthereumMessageSignature> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EthereumMessageSignature,
        };
        unsafe {
            instance.get(EthereumMessageSignature::new)
        }
    }

    // optional bytes address = 1;

    pub fn clear_address(&mut self) {
        self.address.clear();
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: ::std::vec::Vec<u8>) {
        self.address = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_address(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.address.is_none() {
            self.address.set_default();
        }
        self.address.as_mut().unwrap()
    }

    // Take field
    pub fn take_address(&mut self) -> ::std::vec::Vec<u8> {
        self.address.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_address(&self) -> &[u8] {
        match self.address.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_address_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.address
    }

    // optional bytes signature = 2;

    pub fn clear_signature(&mut self) {
        self.signature.clear();
    }

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature(&mut self, v: ::std::vec::Vec<u8>) {
        self.signature = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signature(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.signature.is_none() {
            self.signature.set_default();
        }
        self.signature.as_mut().unwrap()
    }

    // Take field
    pub fn take_signature(&mut self) -> ::std::vec::Vec<u8> {
        self.signature.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_signature(&self) -> &[u8] {
        match self.signature.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_signature_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.signature
    }

    fn mut_signature_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.signature
    }
}

impl ::protobuf::Message for EthereumMessageSignature {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.address)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.signature)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.address.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        if let Some(ref v) = self.signature.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.address.as_ref() {
            os.write_bytes(1, &v)?;
        }
        if let Some(ref v) = self.signature.as_ref() {
            os.write_bytes(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for EthereumMessageSignature {
    fn new() -> EthereumMessageSignature {
        EthereumMessageSignature::new()
    }

    fn descriptor_static(_: ::std::option::Option<EthereumMessageSignature>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "address",
                    EthereumMessageSignature::get_address_for_reflect,
                    EthereumMessageSignature::mut_address_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature",
                    EthereumMessageSignature::get_signature_for_reflect,
                    EthereumMessageSignature::mut_signature_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EthereumMessageSignature>(
                    "EthereumMessageSignature",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EthereumMessageSignature {
    fn clear(&mut self) {
        self.clear_address();
        self.clear_signature();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EthereumMessageSignature {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EthereumMessageSignature {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SignIdentity {
    // message fields
    identity: ::protobuf::SingularPtrField<super::types::IdentityType>,
    challenge_hidden: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    challenge_visual: ::protobuf::SingularField<::std::string::String>,
    ecdsa_curve_name: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for SignIdentity {}

impl SignIdentity {
    pub fn new() -> SignIdentity {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static SignIdentity {
        static mut instance: ::protobuf::lazy::Lazy<SignIdentity> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const SignIdentity,
        };
        unsafe {
            instance.get(SignIdentity::new)
        }
    }

    // optional .IdentityType identity = 1;

    pub fn clear_identity(&mut self) {
        self.identity.clear();
    }

    pub fn has_identity(&self) -> bool {
        self.identity.is_some()
    }

    // Param is passed by value, moved
    pub fn set_identity(&mut self, v: super::types::IdentityType) {
        self.identity = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_identity(&mut self) -> &mut super::types::IdentityType {
        if self.identity.is_none() {
            self.identity.set_default();
        }
        self.identity.as_mut().unwrap()
    }

    // Take field
    pub fn take_identity(&mut self) -> super::types::IdentityType {
        self.identity.take().unwrap_or_else(|| super::types::IdentityType::new())
    }

    pub fn get_identity(&self) -> &super::types::IdentityType {
        self.identity.as_ref().unwrap_or_else(|| super::types::IdentityType::default_instance())
    }

    fn get_identity_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::IdentityType> {
        &self.identity
    }

    fn mut_identity_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::IdentityType> {
        &mut self.identity
    }

    // optional bytes challenge_hidden = 2;

    pub fn clear_challenge_hidden(&mut self) {
        self.challenge_hidden.clear();
    }

    pub fn has_challenge_hidden(&self) -> bool {
        self.challenge_hidden.is_some()
    }

    // Param is passed by value, moved
    pub fn set_challenge_hidden(&mut self, v: ::std::vec::Vec<u8>) {
        self.challenge_hidden = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_challenge_hidden(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.challenge_hidden.is_none() {
            self.challenge_hidden.set_default();
        }
        self.challenge_hidden.as_mut().unwrap()
    }

    // Take field
    pub fn take_challenge_hidden(&mut self) -> ::std::vec::Vec<u8> {
        self.challenge_hidden.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_challenge_hidden(&self) -> &[u8] {
        match self.challenge_hidden.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_challenge_hidden_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.challenge_hidden
    }

    fn mut_challenge_hidden_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.challenge_hidden
    }

    // optional string challenge_visual = 3;

    pub fn clear_challenge_visual(&mut self) {
        self.challenge_visual.clear();
    }

    pub fn has_challenge_visual(&self) -> bool {
        self.challenge_visual.is_some()
    }

    // Param is passed by value, moved
    pub fn set_challenge_visual(&mut self, v: ::std::string::String) {
        self.challenge_visual = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_challenge_visual(&mut self) -> &mut ::std::string::String {
        if self.challenge_visual.is_none() {
            self.challenge_visual.set_default();
        }
        self.challenge_visual.as_mut().unwrap()
    }

    // Take field
    pub fn take_challenge_visual(&mut self) -> ::std::string::String {
        self.challenge_visual.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_challenge_visual(&self) -> &str {
        match self.challenge_visual.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_challenge_visual_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.challenge_visual
    }

    fn mut_challenge_visual_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.challenge_visual
    }

    // optional string ecdsa_curve_name = 4;

    pub fn clear_ecdsa_curve_name(&mut self) {
        self.ecdsa_curve_name.clear();
    }

    pub fn has_ecdsa_curve_name(&self) -> bool {
        self.ecdsa_curve_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_ecdsa_curve_name(&mut self, v: ::std::string::String) {
        self.ecdsa_curve_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_ecdsa_curve_name(&mut self) -> &mut ::std::string::String {
        if self.ecdsa_curve_name.is_none() {
            self.ecdsa_curve_name.set_default();
        }
        self.ecdsa_curve_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_ecdsa_curve_name(&mut self) -> ::std::string::String {
        self.ecdsa_curve_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_ecdsa_curve_name(&self) -> &str {
        match self.ecdsa_curve_name.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_ecdsa_curve_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.ecdsa_curve_name
    }

    fn mut_ecdsa_curve_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.ecdsa_curve_name
    }
}

impl ::protobuf::Message for SignIdentity {
    fn is_initialized(&self) -> bool {
        for v in &self.identity {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.identity)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.challenge_hidden)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.challenge_visual)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.ecdsa_curve_name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.identity.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.challenge_hidden.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.challenge_visual.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        if let Some(ref v) = self.ecdsa_curve_name.as_ref() {
            my_size += ::protobuf::rt::string_size(4, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.identity.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.challenge_hidden.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.challenge_visual.as_ref() {
            os.write_string(3, &v)?;
        }
        if let Some(ref v) = self.ecdsa_curve_name.as_ref() {
            os.write_string(4, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for SignIdentity {
    fn new() -> SignIdentity {
        SignIdentity::new()
    }

    fn descriptor_static(_: ::std::option::Option<SignIdentity>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::IdentityType>>(
                    "identity",
                    SignIdentity::get_identity_for_reflect,
                    SignIdentity::mut_identity_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "challenge_hidden",
                    SignIdentity::get_challenge_hidden_for_reflect,
                    SignIdentity::mut_challenge_hidden_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "challenge_visual",
                    SignIdentity::get_challenge_visual_for_reflect,
                    SignIdentity::mut_challenge_visual_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "ecdsa_curve_name",
                    SignIdentity::get_ecdsa_curve_name_for_reflect,
                    SignIdentity::mut_ecdsa_curve_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<SignIdentity>(
                    "SignIdentity",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for SignIdentity {
    fn clear(&mut self) {
        self.clear_identity();
        self.clear_challenge_hidden();
        self.clear_challenge_visual();
        self.clear_ecdsa_curve_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SignIdentity {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SignIdentity {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SignedIdentity {
    // message fields
    address: ::protobuf::SingularField<::std::string::String>,
    public_key: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    signature: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for SignedIdentity {}

impl SignedIdentity {
    pub fn new() -> SignedIdentity {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static SignedIdentity {
        static mut instance: ::protobuf::lazy::Lazy<SignedIdentity> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const SignedIdentity,
        };
        unsafe {
            instance.get(SignedIdentity::new)
        }
    }

    // optional string address = 1;

    pub fn clear_address(&mut self) {
        self.address.clear();
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: ::std::string::String) {
        self.address = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_address(&mut self) -> &mut ::std::string::String {
        if self.address.is_none() {
            self.address.set_default();
        }
        self.address.as_mut().unwrap()
    }

    // Take field
    pub fn take_address(&mut self) -> ::std::string::String {
        self.address.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_address(&self) -> &str {
        match self.address.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_address_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.address
    }

    // optional bytes public_key = 2;

    pub fn clear_public_key(&mut self) {
        self.public_key.clear();
    }

    pub fn has_public_key(&self) -> bool {
        self.public_key.is_some()
    }

    // Param is passed by value, moved
    pub fn set_public_key(&mut self, v: ::std::vec::Vec<u8>) {
        self.public_key = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_public_key(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.public_key.is_none() {
            self.public_key.set_default();
        }
        self.public_key.as_mut().unwrap()
    }

    // Take field
    pub fn take_public_key(&mut self) -> ::std::vec::Vec<u8> {
        self.public_key.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_public_key(&self) -> &[u8] {
        match self.public_key.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_public_key_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.public_key
    }

    fn mut_public_key_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.public_key
    }

    // optional bytes signature = 3;

    pub fn clear_signature(&mut self) {
        self.signature.clear();
    }

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature(&mut self, v: ::std::vec::Vec<u8>) {
        self.signature = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signature(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.signature.is_none() {
            self.signature.set_default();
        }
        self.signature.as_mut().unwrap()
    }

    // Take field
    pub fn take_signature(&mut self) -> ::std::vec::Vec<u8> {
        self.signature.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_signature(&self) -> &[u8] {
        match self.signature.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_signature_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.signature
    }

    fn mut_signature_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.signature
    }
}

impl ::protobuf::Message for SignedIdentity {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.address)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.public_key)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.signature)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.address.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(ref v) = self.public_key.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.signature.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.address.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(ref v) = self.public_key.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.signature.as_ref() {
            os.write_bytes(3, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for SignedIdentity {
    fn new() -> SignedIdentity {
        SignedIdentity::new()
    }

    fn descriptor_static(_: ::std::option::Option<SignedIdentity>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "address",
                    SignedIdentity::get_address_for_reflect,
                    SignedIdentity::mut_address_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "public_key",
                    SignedIdentity::get_public_key_for_reflect,
                    SignedIdentity::mut_public_key_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature",
                    SignedIdentity::get_signature_for_reflect,
                    SignedIdentity::mut_signature_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<SignedIdentity>(
                    "SignedIdentity",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for SignedIdentity {
    fn clear(&mut self) {
        self.clear_address();
        self.clear_public_key();
        self.clear_signature();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SignedIdentity {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SignedIdentity {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct GetECDHSessionKey {
    // message fields
    identity: ::protobuf::SingularPtrField<super::types::IdentityType>,
    peer_public_key: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    ecdsa_curve_name: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for GetECDHSessionKey {}

impl GetECDHSessionKey {
    pub fn new() -> GetECDHSessionKey {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static GetECDHSessionKey {
        static mut instance: ::protobuf::lazy::Lazy<GetECDHSessionKey> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const GetECDHSessionKey,
        };
        unsafe {
            instance.get(GetECDHSessionKey::new)
        }
    }

    // optional .IdentityType identity = 1;

    pub fn clear_identity(&mut self) {
        self.identity.clear();
    }

    pub fn has_identity(&self) -> bool {
        self.identity.is_some()
    }

    // Param is passed by value, moved
    pub fn set_identity(&mut self, v: super::types::IdentityType) {
        self.identity = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_identity(&mut self) -> &mut super::types::IdentityType {
        if self.identity.is_none() {
            self.identity.set_default();
        }
        self.identity.as_mut().unwrap()
    }

    // Take field
    pub fn take_identity(&mut self) -> super::types::IdentityType {
        self.identity.take().unwrap_or_else(|| super::types::IdentityType::new())
    }

    pub fn get_identity(&self) -> &super::types::IdentityType {
        self.identity.as_ref().unwrap_or_else(|| super::types::IdentityType::default_instance())
    }

    fn get_identity_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::IdentityType> {
        &self.identity
    }

    fn mut_identity_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::IdentityType> {
        &mut self.identity
    }

    // optional bytes peer_public_key = 2;

    pub fn clear_peer_public_key(&mut self) {
        self.peer_public_key.clear();
    }

    pub fn has_peer_public_key(&self) -> bool {
        self.peer_public_key.is_some()
    }

    // Param is passed by value, moved
    pub fn set_peer_public_key(&mut self, v: ::std::vec::Vec<u8>) {
        self.peer_public_key = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_peer_public_key(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.peer_public_key.is_none() {
            self.peer_public_key.set_default();
        }
        self.peer_public_key.as_mut().unwrap()
    }

    // Take field
    pub fn take_peer_public_key(&mut self) -> ::std::vec::Vec<u8> {
        self.peer_public_key.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_peer_public_key(&self) -> &[u8] {
        match self.peer_public_key.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_peer_public_key_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.peer_public_key
    }

    fn mut_peer_public_key_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.peer_public_key
    }

    // optional string ecdsa_curve_name = 3;

    pub fn clear_ecdsa_curve_name(&mut self) {
        self.ecdsa_curve_name.clear();
    }

    pub fn has_ecdsa_curve_name(&self) -> bool {
        self.ecdsa_curve_name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_ecdsa_curve_name(&mut self, v: ::std::string::String) {
        self.ecdsa_curve_name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_ecdsa_curve_name(&mut self) -> &mut ::std::string::String {
        if self.ecdsa_curve_name.is_none() {
            self.ecdsa_curve_name.set_default();
        }
        self.ecdsa_curve_name.as_mut().unwrap()
    }

    // Take field
    pub fn take_ecdsa_curve_name(&mut self) -> ::std::string::String {
        self.ecdsa_curve_name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_ecdsa_curve_name(&self) -> &str {
        match self.ecdsa_curve_name.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_ecdsa_curve_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.ecdsa_curve_name
    }

    fn mut_ecdsa_curve_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.ecdsa_curve_name
    }
}

impl ::protobuf::Message for GetECDHSessionKey {
    fn is_initialized(&self) -> bool {
        for v in &self.identity {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.identity)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.peer_public_key)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.ecdsa_curve_name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.identity.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.peer_public_key.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.ecdsa_curve_name.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.identity.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.peer_public_key.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.ecdsa_curve_name.as_ref() {
            os.write_string(3, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for GetECDHSessionKey {
    fn new() -> GetECDHSessionKey {
        GetECDHSessionKey::new()
    }

    fn descriptor_static(_: ::std::option::Option<GetECDHSessionKey>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::IdentityType>>(
                    "identity",
                    GetECDHSessionKey::get_identity_for_reflect,
                    GetECDHSessionKey::mut_identity_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "peer_public_key",
                    GetECDHSessionKey::get_peer_public_key_for_reflect,
                    GetECDHSessionKey::mut_peer_public_key_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "ecdsa_curve_name",
                    GetECDHSessionKey::get_ecdsa_curve_name_for_reflect,
                    GetECDHSessionKey::mut_ecdsa_curve_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<GetECDHSessionKey>(
                    "GetECDHSessionKey",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for GetECDHSessionKey {
    fn clear(&mut self) {
        self.clear_identity();
        self.clear_peer_public_key();
        self.clear_ecdsa_curve_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for GetECDHSessionKey {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for GetECDHSessionKey {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ECDHSessionKey {
    // message fields
    session_key: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ECDHSessionKey {}

impl ECDHSessionKey {
    pub fn new() -> ECDHSessionKey {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ECDHSessionKey {
        static mut instance: ::protobuf::lazy::Lazy<ECDHSessionKey> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ECDHSessionKey,
        };
        unsafe {
            instance.get(ECDHSessionKey::new)
        }
    }

    // optional bytes session_key = 1;

    pub fn clear_session_key(&mut self) {
        self.session_key.clear();
    }

    pub fn has_session_key(&self) -> bool {
        self.session_key.is_some()
    }

    // Param is passed by value, moved
    pub fn set_session_key(&mut self, v: ::std::vec::Vec<u8>) {
        self.session_key = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_session_key(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.session_key.is_none() {
            self.session_key.set_default();
        }
        self.session_key.as_mut().unwrap()
    }

    // Take field
    pub fn take_session_key(&mut self) -> ::std::vec::Vec<u8> {
        self.session_key.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_session_key(&self) -> &[u8] {
        match self.session_key.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_session_key_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.session_key
    }

    fn mut_session_key_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.session_key
    }
}

impl ::protobuf::Message for ECDHSessionKey {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.session_key)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.session_key.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.session_key.as_ref() {
            os.write_bytes(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ECDHSessionKey {
    fn new() -> ECDHSessionKey {
        ECDHSessionKey::new()
    }

    fn descriptor_static(_: ::std::option::Option<ECDHSessionKey>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "session_key",
                    ECDHSessionKey::get_session_key_for_reflect,
                    ECDHSessionKey::mut_session_key_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ECDHSessionKey>(
                    "ECDHSessionKey",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ECDHSessionKey {
    fn clear(&mut self) {
        self.clear_session_key();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ECDHSessionKey {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ECDHSessionKey {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SetU2FCounter {
    // message fields
    u2f_counter: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for SetU2FCounter {}

impl SetU2FCounter {
    pub fn new() -> SetU2FCounter {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static SetU2FCounter {
        static mut instance: ::protobuf::lazy::Lazy<SetU2FCounter> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const SetU2FCounter,
        };
        unsafe {
            instance.get(SetU2FCounter::new)
        }
    }

    // optional uint32 u2f_counter = 1;

    pub fn clear_u2f_counter(&mut self) {
        self.u2f_counter = ::std::option::Option::None;
    }

    pub fn has_u2f_counter(&self) -> bool {
        self.u2f_counter.is_some()
    }

    // Param is passed by value, moved
    pub fn set_u2f_counter(&mut self, v: u32) {
        self.u2f_counter = ::std::option::Option::Some(v);
    }

    pub fn get_u2f_counter(&self) -> u32 {
        self.u2f_counter.unwrap_or(0)
    }

    fn get_u2f_counter_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.u2f_counter
    }

    fn mut_u2f_counter_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.u2f_counter
    }
}

impl ::protobuf::Message for SetU2FCounter {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.u2f_counter = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.u2f_counter {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.u2f_counter {
            os.write_uint32(1, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for SetU2FCounter {
    fn new() -> SetU2FCounter {
        SetU2FCounter::new()
    }

    fn descriptor_static(_: ::std::option::Option<SetU2FCounter>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "u2f_counter",
                    SetU2FCounter::get_u2f_counter_for_reflect,
                    SetU2FCounter::mut_u2f_counter_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<SetU2FCounter>(
                    "SetU2FCounter",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for SetU2FCounter {
    fn clear(&mut self) {
        self.clear_u2f_counter();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SetU2FCounter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SetU2FCounter {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct FirmwareErase {
    // message fields
    length: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for FirmwareErase {}

impl FirmwareErase {
    pub fn new() -> FirmwareErase {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static FirmwareErase {
        static mut instance: ::protobuf::lazy::Lazy<FirmwareErase> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const FirmwareErase,
        };
        unsafe {
            instance.get(FirmwareErase::new)
        }
    }

    // optional uint32 length = 1;

    pub fn clear_length(&mut self) {
        self.length = ::std::option::Option::None;
    }

    pub fn has_length(&self) -> bool {
        self.length.is_some()
    }

    // Param is passed by value, moved
    pub fn set_length(&mut self, v: u32) {
        self.length = ::std::option::Option::Some(v);
    }

    pub fn get_length(&self) -> u32 {
        self.length.unwrap_or(0)
    }

    fn get_length_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.length
    }

    fn mut_length_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.length
    }
}

impl ::protobuf::Message for FirmwareErase {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.length = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.length {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.length {
            os.write_uint32(1, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for FirmwareErase {
    fn new() -> FirmwareErase {
        FirmwareErase::new()
    }

    fn descriptor_static(_: ::std::option::Option<FirmwareErase>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "length",
                    FirmwareErase::get_length_for_reflect,
                    FirmwareErase::mut_length_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<FirmwareErase>(
                    "FirmwareErase",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for FirmwareErase {
    fn clear(&mut self) {
        self.clear_length();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for FirmwareErase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for FirmwareErase {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct FirmwareRequest {
    // message fields
    offset: ::std::option::Option<u32>,
    length: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for FirmwareRequest {}

impl FirmwareRequest {
    pub fn new() -> FirmwareRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static FirmwareRequest {
        static mut instance: ::protobuf::lazy::Lazy<FirmwareRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const FirmwareRequest,
        };
        unsafe {
            instance.get(FirmwareRequest::new)
        }
    }

    // optional uint32 offset = 1;

    pub fn clear_offset(&mut self) {
        self.offset = ::std::option::Option::None;
    }

    pub fn has_offset(&self) -> bool {
        self.offset.is_some()
    }

    // Param is passed by value, moved
    pub fn set_offset(&mut self, v: u32) {
        self.offset = ::std::option::Option::Some(v);
    }

    pub fn get_offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }

    fn get_offset_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.offset
    }

    fn mut_offset_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.offset
    }

    // optional uint32 length = 2;

    pub fn clear_length(&mut self) {
        self.length = ::std::option::Option::None;
    }

    pub fn has_length(&self) -> bool {
        self.length.is_some()
    }

    // Param is passed by value, moved
    pub fn set_length(&mut self, v: u32) {
        self.length = ::std::option::Option::Some(v);
    }

    pub fn get_length(&self) -> u32 {
        self.length.unwrap_or(0)
    }

    fn get_length_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.length
    }

    fn mut_length_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.length
    }
}

impl ::protobuf::Message for FirmwareRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.offset = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.length = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.offset {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.length {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.offset {
            os.write_uint32(1, v)?;
        }
        if let Some(v) = self.length {
            os.write_uint32(2, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for FirmwareRequest {
    fn new() -> FirmwareRequest {
        FirmwareRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<FirmwareRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "offset",
                    FirmwareRequest::get_offset_for_reflect,
                    FirmwareRequest::mut_offset_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "length",
                    FirmwareRequest::get_length_for_reflect,
                    FirmwareRequest::mut_length_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<FirmwareRequest>(
                    "FirmwareRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for FirmwareRequest {
    fn clear(&mut self) {
        self.clear_offset();
        self.clear_length();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for FirmwareRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for FirmwareRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct FirmwareUpload {
    // message fields
    payload: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    hash: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for FirmwareUpload {}

impl FirmwareUpload {
    pub fn new() -> FirmwareUpload {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static FirmwareUpload {
        static mut instance: ::protobuf::lazy::Lazy<FirmwareUpload> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const FirmwareUpload,
        };
        unsafe {
            instance.get(FirmwareUpload::new)
        }
    }

    // required bytes payload = 1;

    pub fn clear_payload(&mut self) {
        self.payload.clear();
    }

    pub fn has_payload(&self) -> bool {
        self.payload.is_some()
    }

    // Param is passed by value, moved
    pub fn set_payload(&mut self, v: ::std::vec::Vec<u8>) {
        self.payload = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_payload(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.payload.is_none() {
            self.payload.set_default();
        }
        self.payload.as_mut().unwrap()
    }

    // Take field
    pub fn take_payload(&mut self) -> ::std::vec::Vec<u8> {
        self.payload.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_payload(&self) -> &[u8] {
        match self.payload.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_payload_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.payload
    }

    fn mut_payload_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.payload
    }

    // optional bytes hash = 2;

    pub fn clear_hash(&mut self) {
        self.hash.clear();
    }

    pub fn has_hash(&self) -> bool {
        self.hash.is_some()
    }

    // Param is passed by value, moved
    pub fn set_hash(&mut self, v: ::std::vec::Vec<u8>) {
        self.hash = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_hash(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.hash.is_none() {
            self.hash.set_default();
        }
        self.hash.as_mut().unwrap()
    }

    // Take field
    pub fn take_hash(&mut self) -> ::std::vec::Vec<u8> {
        self.hash.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_hash(&self) -> &[u8] {
        match self.hash.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_hash_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.hash
    }

    fn mut_hash_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.hash
    }
}

impl ::protobuf::Message for FirmwareUpload {
    fn is_initialized(&self) -> bool {
        if self.payload.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.payload)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.hash)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.payload.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        if let Some(ref v) = self.hash.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.payload.as_ref() {
            os.write_bytes(1, &v)?;
        }
        if let Some(ref v) = self.hash.as_ref() {
            os.write_bytes(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for FirmwareUpload {
    fn new() -> FirmwareUpload {
        FirmwareUpload::new()
    }

    fn descriptor_static(_: ::std::option::Option<FirmwareUpload>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "payload",
                    FirmwareUpload::get_payload_for_reflect,
                    FirmwareUpload::mut_payload_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "hash",
                    FirmwareUpload::get_hash_for_reflect,
                    FirmwareUpload::mut_hash_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<FirmwareUpload>(
                    "FirmwareUpload",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for FirmwareUpload {
    fn clear(&mut self) {
        self.clear_payload();
        self.clear_hash();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for FirmwareUpload {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for FirmwareUpload {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SelfTest {
    // message fields
    payload: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for SelfTest {}

impl SelfTest {
    pub fn new() -> SelfTest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static SelfTest {
        static mut instance: ::protobuf::lazy::Lazy<SelfTest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const SelfTest,
        };
        unsafe {
            instance.get(SelfTest::new)
        }
    }

    // optional bytes payload = 1;

    pub fn clear_payload(&mut self) {
        self.payload.clear();
    }

    pub fn has_payload(&self) -> bool {
        self.payload.is_some()
    }

    // Param is passed by value, moved
    pub fn set_payload(&mut self, v: ::std::vec::Vec<u8>) {
        self.payload = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_payload(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.payload.is_none() {
            self.payload.set_default();
        }
        self.payload.as_mut().unwrap()
    }

    // Take field
    pub fn take_payload(&mut self) -> ::std::vec::Vec<u8> {
        self.payload.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_payload(&self) -> &[u8] {
        match self.payload.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_payload_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.payload
    }

    fn mut_payload_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.payload
    }
}

impl ::protobuf::Message for SelfTest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.payload)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.payload.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.payload.as_ref() {
            os.write_bytes(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for SelfTest {
    fn new() -> SelfTest {
        SelfTest::new()
    }

    fn descriptor_static(_: ::std::option::Option<SelfTest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "payload",
                    SelfTest::get_payload_for_reflect,
                    SelfTest::mut_payload_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<SelfTest>(
                    "SelfTest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for SelfTest {
    fn clear(&mut self) {
        self.clear_payload();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SelfTest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SelfTest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkDecision {
    // message fields
    yes_no: ::std::option::Option<bool>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkDecision {}

impl DebugLinkDecision {
    pub fn new() -> DebugLinkDecision {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkDecision {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkDecision> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkDecision,
        };
        unsafe {
            instance.get(DebugLinkDecision::new)
        }
    }

    // required bool yes_no = 1;

    pub fn clear_yes_no(&mut self) {
        self.yes_no = ::std::option::Option::None;
    }

    pub fn has_yes_no(&self) -> bool {
        self.yes_no.is_some()
    }

    // Param is passed by value, moved
    pub fn set_yes_no(&mut self, v: bool) {
        self.yes_no = ::std::option::Option::Some(v);
    }

    pub fn get_yes_no(&self) -> bool {
        self.yes_no.unwrap_or(false)
    }

    fn get_yes_no_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.yes_no
    }

    fn mut_yes_no_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.yes_no
    }
}

impl ::protobuf::Message for DebugLinkDecision {
    fn is_initialized(&self) -> bool {
        if self.yes_no.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.yes_no = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.yes_no {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.yes_no {
            os.write_bool(1, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkDecision {
    fn new() -> DebugLinkDecision {
        DebugLinkDecision::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkDecision>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "yes_no",
                    DebugLinkDecision::get_yes_no_for_reflect,
                    DebugLinkDecision::mut_yes_no_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkDecision>(
                    "DebugLinkDecision",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkDecision {
    fn clear(&mut self) {
        self.clear_yes_no();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkDecision {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkDecision {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkGetState {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkGetState {}

impl DebugLinkGetState {
    pub fn new() -> DebugLinkGetState {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkGetState {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkGetState> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkGetState,
        };
        unsafe {
            instance.get(DebugLinkGetState::new)
        }
    }
}

impl ::protobuf::Message for DebugLinkGetState {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkGetState {
    fn new() -> DebugLinkGetState {
        DebugLinkGetState::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkGetState>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkGetState>(
                    "DebugLinkGetState",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkGetState {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkGetState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkGetState {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkState {
    // message fields
    layout: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    pin: ::protobuf::SingularField<::std::string::String>,
    matrix: ::protobuf::SingularField<::std::string::String>,
    mnemonic: ::protobuf::SingularField<::std::string::String>,
    node: ::protobuf::SingularPtrField<super::types::HDNodeType>,
    passphrase_protection: ::std::option::Option<bool>,
    reset_word: ::protobuf::SingularField<::std::string::String>,
    reset_entropy: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    recovery_fake_word: ::protobuf::SingularField<::std::string::String>,
    recovery_word_pos: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkState {}

impl DebugLinkState {
    pub fn new() -> DebugLinkState {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkState {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkState> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkState,
        };
        unsafe {
            instance.get(DebugLinkState::new)
        }
    }

    // optional bytes layout = 1;

    pub fn clear_layout(&mut self) {
        self.layout.clear();
    }

    pub fn has_layout(&self) -> bool {
        self.layout.is_some()
    }

    // Param is passed by value, moved
    pub fn set_layout(&mut self, v: ::std::vec::Vec<u8>) {
        self.layout = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_layout(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.layout.is_none() {
            self.layout.set_default();
        }
        self.layout.as_mut().unwrap()
    }

    // Take field
    pub fn take_layout(&mut self) -> ::std::vec::Vec<u8> {
        self.layout.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_layout(&self) -> &[u8] {
        match self.layout.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_layout_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.layout
    }

    fn mut_layout_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.layout
    }

    // optional string pin = 2;

    pub fn clear_pin(&mut self) {
        self.pin.clear();
    }

    pub fn has_pin(&self) -> bool {
        self.pin.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pin(&mut self, v: ::std::string::String) {
        self.pin = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_pin(&mut self) -> &mut ::std::string::String {
        if self.pin.is_none() {
            self.pin.set_default();
        }
        self.pin.as_mut().unwrap()
    }

    // Take field
    pub fn take_pin(&mut self) -> ::std::string::String {
        self.pin.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_pin(&self) -> &str {
        match self.pin.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_pin_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.pin
    }

    fn mut_pin_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.pin
    }

    // optional string matrix = 3;

    pub fn clear_matrix(&mut self) {
        self.matrix.clear();
    }

    pub fn has_matrix(&self) -> bool {
        self.matrix.is_some()
    }

    // Param is passed by value, moved
    pub fn set_matrix(&mut self, v: ::std::string::String) {
        self.matrix = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_matrix(&mut self) -> &mut ::std::string::String {
        if self.matrix.is_none() {
            self.matrix.set_default();
        }
        self.matrix.as_mut().unwrap()
    }

    // Take field
    pub fn take_matrix(&mut self) -> ::std::string::String {
        self.matrix.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_matrix(&self) -> &str {
        match self.matrix.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_matrix_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.matrix
    }

    fn mut_matrix_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.matrix
    }

    // optional string mnemonic = 4;

    pub fn clear_mnemonic(&mut self) {
        self.mnemonic.clear();
    }

    pub fn has_mnemonic(&self) -> bool {
        self.mnemonic.is_some()
    }

    // Param is passed by value, moved
    pub fn set_mnemonic(&mut self, v: ::std::string::String) {
        self.mnemonic = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_mnemonic(&mut self) -> &mut ::std::string::String {
        if self.mnemonic.is_none() {
            self.mnemonic.set_default();
        }
        self.mnemonic.as_mut().unwrap()
    }

    // Take field
    pub fn take_mnemonic(&mut self) -> ::std::string::String {
        self.mnemonic.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_mnemonic(&self) -> &str {
        match self.mnemonic.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_mnemonic_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.mnemonic
    }

    fn mut_mnemonic_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.mnemonic
    }

    // optional .HDNodeType node = 5;

    pub fn clear_node(&mut self) {
        self.node.clear();
    }

    pub fn has_node(&self) -> bool {
        self.node.is_some()
    }

    // Param is passed by value, moved
    pub fn set_node(&mut self, v: super::types::HDNodeType) {
        self.node = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_node(&mut self) -> &mut super::types::HDNodeType {
        if self.node.is_none() {
            self.node.set_default();
        }
        self.node.as_mut().unwrap()
    }

    // Take field
    pub fn take_node(&mut self) -> super::types::HDNodeType {
        self.node.take().unwrap_or_else(|| super::types::HDNodeType::new())
    }

    pub fn get_node(&self) -> &super::types::HDNodeType {
        self.node.as_ref().unwrap_or_else(|| super::types::HDNodeType::default_instance())
    }

    fn get_node_for_reflect(&self) -> &::protobuf::SingularPtrField<super::types::HDNodeType> {
        &self.node
    }

    fn mut_node_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::types::HDNodeType> {
        &mut self.node
    }

    // optional bool passphrase_protection = 6;

    pub fn clear_passphrase_protection(&mut self) {
        self.passphrase_protection = ::std::option::Option::None;
    }

    pub fn has_passphrase_protection(&self) -> bool {
        self.passphrase_protection.is_some()
    }

    // Param is passed by value, moved
    pub fn set_passphrase_protection(&mut self, v: bool) {
        self.passphrase_protection = ::std::option::Option::Some(v);
    }

    pub fn get_passphrase_protection(&self) -> bool {
        self.passphrase_protection.unwrap_or(false)
    }

    fn get_passphrase_protection_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.passphrase_protection
    }

    fn mut_passphrase_protection_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.passphrase_protection
    }

    // optional string reset_word = 7;

    pub fn clear_reset_word(&mut self) {
        self.reset_word.clear();
    }

    pub fn has_reset_word(&self) -> bool {
        self.reset_word.is_some()
    }

    // Param is passed by value, moved
    pub fn set_reset_word(&mut self, v: ::std::string::String) {
        self.reset_word = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_reset_word(&mut self) -> &mut ::std::string::String {
        if self.reset_word.is_none() {
            self.reset_word.set_default();
        }
        self.reset_word.as_mut().unwrap()
    }

    // Take field
    pub fn take_reset_word(&mut self) -> ::std::string::String {
        self.reset_word.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_reset_word(&self) -> &str {
        match self.reset_word.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_reset_word_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.reset_word
    }

    fn mut_reset_word_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.reset_word
    }

    // optional bytes reset_entropy = 8;

    pub fn clear_reset_entropy(&mut self) {
        self.reset_entropy.clear();
    }

    pub fn has_reset_entropy(&self) -> bool {
        self.reset_entropy.is_some()
    }

    // Param is passed by value, moved
    pub fn set_reset_entropy(&mut self, v: ::std::vec::Vec<u8>) {
        self.reset_entropy = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_reset_entropy(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.reset_entropy.is_none() {
            self.reset_entropy.set_default();
        }
        self.reset_entropy.as_mut().unwrap()
    }

    // Take field
    pub fn take_reset_entropy(&mut self) -> ::std::vec::Vec<u8> {
        self.reset_entropy.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_reset_entropy(&self) -> &[u8] {
        match self.reset_entropy.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_reset_entropy_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.reset_entropy
    }

    fn mut_reset_entropy_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.reset_entropy
    }

    // optional string recovery_fake_word = 9;

    pub fn clear_recovery_fake_word(&mut self) {
        self.recovery_fake_word.clear();
    }

    pub fn has_recovery_fake_word(&self) -> bool {
        self.recovery_fake_word.is_some()
    }

    // Param is passed by value, moved
    pub fn set_recovery_fake_word(&mut self, v: ::std::string::String) {
        self.recovery_fake_word = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_recovery_fake_word(&mut self) -> &mut ::std::string::String {
        if self.recovery_fake_word.is_none() {
            self.recovery_fake_word.set_default();
        }
        self.recovery_fake_word.as_mut().unwrap()
    }

    // Take field
    pub fn take_recovery_fake_word(&mut self) -> ::std::string::String {
        self.recovery_fake_word.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_recovery_fake_word(&self) -> &str {
        match self.recovery_fake_word.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_recovery_fake_word_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.recovery_fake_word
    }

    fn mut_recovery_fake_word_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.recovery_fake_word
    }

    // optional uint32 recovery_word_pos = 10;

    pub fn clear_recovery_word_pos(&mut self) {
        self.recovery_word_pos = ::std::option::Option::None;
    }

    pub fn has_recovery_word_pos(&self) -> bool {
        self.recovery_word_pos.is_some()
    }

    // Param is passed by value, moved
    pub fn set_recovery_word_pos(&mut self, v: u32) {
        self.recovery_word_pos = ::std::option::Option::Some(v);
    }

    pub fn get_recovery_word_pos(&self) -> u32 {
        self.recovery_word_pos.unwrap_or(0)
    }

    fn get_recovery_word_pos_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.recovery_word_pos
    }

    fn mut_recovery_word_pos_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.recovery_word_pos
    }
}

impl ::protobuf::Message for DebugLinkState {
    fn is_initialized(&self) -> bool {
        for v in &self.node {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.layout)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.pin)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.matrix)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.mnemonic)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.node)?;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.passphrase_protection = ::std::option::Option::Some(tmp);
                },
                7 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.reset_word)?;
                },
                8 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.reset_entropy)?;
                },
                9 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.recovery_fake_word)?;
                },
                10 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.recovery_word_pos = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.layout.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        if let Some(ref v) = self.pin.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        if let Some(ref v) = self.matrix.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        if let Some(ref v) = self.mnemonic.as_ref() {
            my_size += ::protobuf::rt::string_size(4, &v);
        }
        if let Some(ref v) = self.node.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(v) = self.passphrase_protection {
            my_size += 2;
        }
        if let Some(ref v) = self.reset_word.as_ref() {
            my_size += ::protobuf::rt::string_size(7, &v);
        }
        if let Some(ref v) = self.reset_entropy.as_ref() {
            my_size += ::protobuf::rt::bytes_size(8, &v);
        }
        if let Some(ref v) = self.recovery_fake_word.as_ref() {
            my_size += ::protobuf::rt::string_size(9, &v);
        }
        if let Some(v) = self.recovery_word_pos {
            my_size += ::protobuf::rt::value_size(10, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.layout.as_ref() {
            os.write_bytes(1, &v)?;
        }
        if let Some(ref v) = self.pin.as_ref() {
            os.write_string(2, &v)?;
        }
        if let Some(ref v) = self.matrix.as_ref() {
            os.write_string(3, &v)?;
        }
        if let Some(ref v) = self.mnemonic.as_ref() {
            os.write_string(4, &v)?;
        }
        if let Some(ref v) = self.node.as_ref() {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(v) = self.passphrase_protection {
            os.write_bool(6, v)?;
        }
        if let Some(ref v) = self.reset_word.as_ref() {
            os.write_string(7, &v)?;
        }
        if let Some(ref v) = self.reset_entropy.as_ref() {
            os.write_bytes(8, &v)?;
        }
        if let Some(ref v) = self.recovery_fake_word.as_ref() {
            os.write_string(9, &v)?;
        }
        if let Some(v) = self.recovery_word_pos {
            os.write_uint32(10, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkState {
    fn new() -> DebugLinkState {
        DebugLinkState::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkState>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "layout",
                    DebugLinkState::get_layout_for_reflect,
                    DebugLinkState::mut_layout_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "pin",
                    DebugLinkState::get_pin_for_reflect,
                    DebugLinkState::mut_pin_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "matrix",
                    DebugLinkState::get_matrix_for_reflect,
                    DebugLinkState::mut_matrix_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "mnemonic",
                    DebugLinkState::get_mnemonic_for_reflect,
                    DebugLinkState::mut_mnemonic_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::types::HDNodeType>>(
                    "node",
                    DebugLinkState::get_node_for_reflect,
                    DebugLinkState::mut_node_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "passphrase_protection",
                    DebugLinkState::get_passphrase_protection_for_reflect,
                    DebugLinkState::mut_passphrase_protection_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "reset_word",
                    DebugLinkState::get_reset_word_for_reflect,
                    DebugLinkState::mut_reset_word_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "reset_entropy",
                    DebugLinkState::get_reset_entropy_for_reflect,
                    DebugLinkState::mut_reset_entropy_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "recovery_fake_word",
                    DebugLinkState::get_recovery_fake_word_for_reflect,
                    DebugLinkState::mut_recovery_fake_word_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "recovery_word_pos",
                    DebugLinkState::get_recovery_word_pos_for_reflect,
                    DebugLinkState::mut_recovery_word_pos_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkState>(
                    "DebugLinkState",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkState {
    fn clear(&mut self) {
        self.clear_layout();
        self.clear_pin();
        self.clear_matrix();
        self.clear_mnemonic();
        self.clear_node();
        self.clear_passphrase_protection();
        self.clear_reset_word();
        self.clear_reset_entropy();
        self.clear_recovery_fake_word();
        self.clear_recovery_word_pos();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkState {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkStop {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkStop {}

impl DebugLinkStop {
    pub fn new() -> DebugLinkStop {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkStop {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkStop> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkStop,
        };
        unsafe {
            instance.get(DebugLinkStop::new)
        }
    }
}

impl ::protobuf::Message for DebugLinkStop {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkStop {
    fn new() -> DebugLinkStop {
        DebugLinkStop::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkStop>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkStop>(
                    "DebugLinkStop",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkStop {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkStop {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkStop {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkLog {
    // message fields
    level: ::std::option::Option<u32>,
    bucket: ::protobuf::SingularField<::std::string::String>,
    text: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkLog {}

impl DebugLinkLog {
    pub fn new() -> DebugLinkLog {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkLog {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkLog> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkLog,
        };
        unsafe {
            instance.get(DebugLinkLog::new)
        }
    }

    // optional uint32 level = 1;

    pub fn clear_level(&mut self) {
        self.level = ::std::option::Option::None;
    }

    pub fn has_level(&self) -> bool {
        self.level.is_some()
    }

    // Param is passed by value, moved
    pub fn set_level(&mut self, v: u32) {
        self.level = ::std::option::Option::Some(v);
    }

    pub fn get_level(&self) -> u32 {
        self.level.unwrap_or(0)
    }

    fn get_level_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.level
    }

    fn mut_level_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.level
    }

    // optional string bucket = 2;

    pub fn clear_bucket(&mut self) {
        self.bucket.clear();
    }

    pub fn has_bucket(&self) -> bool {
        self.bucket.is_some()
    }

    // Param is passed by value, moved
    pub fn set_bucket(&mut self, v: ::std::string::String) {
        self.bucket = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_bucket(&mut self) -> &mut ::std::string::String {
        if self.bucket.is_none() {
            self.bucket.set_default();
        }
        self.bucket.as_mut().unwrap()
    }

    // Take field
    pub fn take_bucket(&mut self) -> ::std::string::String {
        self.bucket.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_bucket(&self) -> &str {
        match self.bucket.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_bucket_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.bucket
    }

    fn mut_bucket_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.bucket
    }

    // optional string text = 3;

    pub fn clear_text(&mut self) {
        self.text.clear();
    }

    pub fn has_text(&self) -> bool {
        self.text.is_some()
    }

    // Param is passed by value, moved
    pub fn set_text(&mut self, v: ::std::string::String) {
        self.text = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_text(&mut self) -> &mut ::std::string::String {
        if self.text.is_none() {
            self.text.set_default();
        }
        self.text.as_mut().unwrap()
    }

    // Take field
    pub fn take_text(&mut self) -> ::std::string::String {
        self.text.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_text(&self) -> &str {
        match self.text.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_text_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.text
    }

    fn mut_text_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.text
    }
}

impl ::protobuf::Message for DebugLinkLog {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.level = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.bucket)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.text)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.level {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.bucket.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        if let Some(ref v) = self.text.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.level {
            os.write_uint32(1, v)?;
        }
        if let Some(ref v) = self.bucket.as_ref() {
            os.write_string(2, &v)?;
        }
        if let Some(ref v) = self.text.as_ref() {
            os.write_string(3, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkLog {
    fn new() -> DebugLinkLog {
        DebugLinkLog::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkLog>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "level",
                    DebugLinkLog::get_level_for_reflect,
                    DebugLinkLog::mut_level_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "bucket",
                    DebugLinkLog::get_bucket_for_reflect,
                    DebugLinkLog::mut_bucket_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "text",
                    DebugLinkLog::get_text_for_reflect,
                    DebugLinkLog::mut_text_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkLog>(
                    "DebugLinkLog",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkLog {
    fn clear(&mut self) {
        self.clear_level();
        self.clear_bucket();
        self.clear_text();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkLog {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkLog {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkMemoryRead {
    // message fields
    address: ::std::option::Option<u32>,
    length: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkMemoryRead {}

impl DebugLinkMemoryRead {
    pub fn new() -> DebugLinkMemoryRead {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkMemoryRead {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkMemoryRead> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkMemoryRead,
        };
        unsafe {
            instance.get(DebugLinkMemoryRead::new)
        }
    }

    // optional uint32 address = 1;

    pub fn clear_address(&mut self) {
        self.address = ::std::option::Option::None;
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: u32) {
        self.address = ::std::option::Option::Some(v);
    }

    pub fn get_address(&self) -> u32 {
        self.address.unwrap_or(0)
    }

    fn get_address_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.address
    }

    // optional uint32 length = 2;

    pub fn clear_length(&mut self) {
        self.length = ::std::option::Option::None;
    }

    pub fn has_length(&self) -> bool {
        self.length.is_some()
    }

    // Param is passed by value, moved
    pub fn set_length(&mut self, v: u32) {
        self.length = ::std::option::Option::Some(v);
    }

    pub fn get_length(&self) -> u32 {
        self.length.unwrap_or(0)
    }

    fn get_length_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.length
    }

    fn mut_length_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.length
    }
}

impl ::protobuf::Message for DebugLinkMemoryRead {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.address = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.length = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.address {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.length {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.address {
            os.write_uint32(1, v)?;
        }
        if let Some(v) = self.length {
            os.write_uint32(2, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkMemoryRead {
    fn new() -> DebugLinkMemoryRead {
        DebugLinkMemoryRead::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkMemoryRead>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address",
                    DebugLinkMemoryRead::get_address_for_reflect,
                    DebugLinkMemoryRead::mut_address_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "length",
                    DebugLinkMemoryRead::get_length_for_reflect,
                    DebugLinkMemoryRead::mut_length_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkMemoryRead>(
                    "DebugLinkMemoryRead",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkMemoryRead {
    fn clear(&mut self) {
        self.clear_address();
        self.clear_length();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkMemoryRead {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkMemoryRead {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkMemory {
    // message fields
    memory: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkMemory {}

impl DebugLinkMemory {
    pub fn new() -> DebugLinkMemory {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkMemory {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkMemory> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkMemory,
        };
        unsafe {
            instance.get(DebugLinkMemory::new)
        }
    }

    // optional bytes memory = 1;

    pub fn clear_memory(&mut self) {
        self.memory.clear();
    }

    pub fn has_memory(&self) -> bool {
        self.memory.is_some()
    }

    // Param is passed by value, moved
    pub fn set_memory(&mut self, v: ::std::vec::Vec<u8>) {
        self.memory = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_memory(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.memory.is_none() {
            self.memory.set_default();
        }
        self.memory.as_mut().unwrap()
    }

    // Take field
    pub fn take_memory(&mut self) -> ::std::vec::Vec<u8> {
        self.memory.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_memory(&self) -> &[u8] {
        match self.memory.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_memory_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.memory
    }

    fn mut_memory_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.memory
    }
}

impl ::protobuf::Message for DebugLinkMemory {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.memory)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.memory.as_ref() {
            my_size += ::protobuf::rt::bytes_size(1, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.memory.as_ref() {
            os.write_bytes(1, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkMemory {
    fn new() -> DebugLinkMemory {
        DebugLinkMemory::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkMemory>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "memory",
                    DebugLinkMemory::get_memory_for_reflect,
                    DebugLinkMemory::mut_memory_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkMemory>(
                    "DebugLinkMemory",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkMemory {
    fn clear(&mut self) {
        self.clear_memory();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkMemory {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkMemory {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkMemoryWrite {
    // message fields
    address: ::std::option::Option<u32>,
    memory: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    flash: ::std::option::Option<bool>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkMemoryWrite {}

impl DebugLinkMemoryWrite {
    pub fn new() -> DebugLinkMemoryWrite {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkMemoryWrite {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkMemoryWrite> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkMemoryWrite,
        };
        unsafe {
            instance.get(DebugLinkMemoryWrite::new)
        }
    }

    // optional uint32 address = 1;

    pub fn clear_address(&mut self) {
        self.address = ::std::option::Option::None;
    }

    pub fn has_address(&self) -> bool {
        self.address.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address(&mut self, v: u32) {
        self.address = ::std::option::Option::Some(v);
    }

    pub fn get_address(&self) -> u32 {
        self.address.unwrap_or(0)
    }

    fn get_address_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.address
    }

    fn mut_address_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.address
    }

    // optional bytes memory = 2;

    pub fn clear_memory(&mut self) {
        self.memory.clear();
    }

    pub fn has_memory(&self) -> bool {
        self.memory.is_some()
    }

    // Param is passed by value, moved
    pub fn set_memory(&mut self, v: ::std::vec::Vec<u8>) {
        self.memory = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_memory(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.memory.is_none() {
            self.memory.set_default();
        }
        self.memory.as_mut().unwrap()
    }

    // Take field
    pub fn take_memory(&mut self) -> ::std::vec::Vec<u8> {
        self.memory.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_memory(&self) -> &[u8] {
        match self.memory.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_memory_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.memory
    }

    fn mut_memory_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.memory
    }

    // optional bool flash = 3;

    pub fn clear_flash(&mut self) {
        self.flash = ::std::option::Option::None;
    }

    pub fn has_flash(&self) -> bool {
        self.flash.is_some()
    }

    // Param is passed by value, moved
    pub fn set_flash(&mut self, v: bool) {
        self.flash = ::std::option::Option::Some(v);
    }

    pub fn get_flash(&self) -> bool {
        self.flash.unwrap_or(false)
    }

    fn get_flash_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.flash
    }

    fn mut_flash_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.flash
    }
}

impl ::protobuf::Message for DebugLinkMemoryWrite {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.address = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.memory)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.flash = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.address {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.memory.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(v) = self.flash {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.address {
            os.write_uint32(1, v)?;
        }
        if let Some(ref v) = self.memory.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(v) = self.flash {
            os.write_bool(3, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkMemoryWrite {
    fn new() -> DebugLinkMemoryWrite {
        DebugLinkMemoryWrite::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkMemoryWrite>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address",
                    DebugLinkMemoryWrite::get_address_for_reflect,
                    DebugLinkMemoryWrite::mut_address_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "memory",
                    DebugLinkMemoryWrite::get_memory_for_reflect,
                    DebugLinkMemoryWrite::mut_memory_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "flash",
                    DebugLinkMemoryWrite::get_flash_for_reflect,
                    DebugLinkMemoryWrite::mut_flash_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkMemoryWrite>(
                    "DebugLinkMemoryWrite",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkMemoryWrite {
    fn clear(&mut self) {
        self.clear_address();
        self.clear_memory();
        self.clear_flash();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkMemoryWrite {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkMemoryWrite {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DebugLinkFlashErase {
    // message fields
    sector: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DebugLinkFlashErase {}

impl DebugLinkFlashErase {
    pub fn new() -> DebugLinkFlashErase {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DebugLinkFlashErase {
        static mut instance: ::protobuf::lazy::Lazy<DebugLinkFlashErase> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DebugLinkFlashErase,
        };
        unsafe {
            instance.get(DebugLinkFlashErase::new)
        }
    }

    // optional uint32 sector = 1;

    pub fn clear_sector(&mut self) {
        self.sector = ::std::option::Option::None;
    }

    pub fn has_sector(&self) -> bool {
        self.sector.is_some()
    }

    // Param is passed by value, moved
    pub fn set_sector(&mut self, v: u32) {
        self.sector = ::std::option::Option::Some(v);
    }

    pub fn get_sector(&self) -> u32 {
        self.sector.unwrap_or(0)
    }

    fn get_sector_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.sector
    }

    fn mut_sector_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.sector
    }
}

impl ::protobuf::Message for DebugLinkFlashErase {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.sector = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.sector {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.sector {
            os.write_uint32(1, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DebugLinkFlashErase {
    fn new() -> DebugLinkFlashErase {
        DebugLinkFlashErase::new()
    }

    fn descriptor_static(_: ::std::option::Option<DebugLinkFlashErase>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "sector",
                    DebugLinkFlashErase::get_sector_for_reflect,
                    DebugLinkFlashErase::mut_sector_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DebugLinkFlashErase>(
                    "DebugLinkFlashErase",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DebugLinkFlashErase {
    fn clear(&mut self) {
        self.clear_sector();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DebugLinkFlashErase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DebugLinkFlashErase {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum MessageType {
    MessageType_Initialize = 0,
    MessageType_Ping = 1,
    MessageType_Success = 2,
    MessageType_Failure = 3,
    MessageType_ChangePin = 4,
    MessageType_WipeDevice = 5,
    MessageType_FirmwareErase = 6,
    MessageType_FirmwareUpload = 7,
    MessageType_FirmwareRequest = 8,
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
    MessageType_ApplyFlags = 28,
    MessageType_GetAddress = 29,
    MessageType_Address = 30,
    MessageType_SelfTest = 32,
    MessageType_BackupDevice = 34,
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
    MessageType_GetECDHSessionKey = 61,
    MessageType_ECDHSessionKey = 62,
    MessageType_SetU2FCounter = 63,
    MessageType_EthereumSignMessage = 64,
    MessageType_EthereumVerifyMessage = 65,
    MessageType_EthereumMessageSignature = 66,
    MessageType_DebugLinkDecision = 100,
    MessageType_DebugLinkGetState = 101,
    MessageType_DebugLinkState = 102,
    MessageType_DebugLinkStop = 103,
    MessageType_DebugLinkLog = 104,
    MessageType_DebugLinkMemoryRead = 110,
    MessageType_DebugLinkMemory = 111,
    MessageType_DebugLinkMemoryWrite = 112,
    MessageType_DebugLinkFlashErase = 113,
}

impl ::protobuf::ProtobufEnum for MessageType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<MessageType> {
        match value {
            0 => ::std::option::Option::Some(MessageType::MessageType_Initialize),
            1 => ::std::option::Option::Some(MessageType::MessageType_Ping),
            2 => ::std::option::Option::Some(MessageType::MessageType_Success),
            3 => ::std::option::Option::Some(MessageType::MessageType_Failure),
            4 => ::std::option::Option::Some(MessageType::MessageType_ChangePin),
            5 => ::std::option::Option::Some(MessageType::MessageType_WipeDevice),
            6 => ::std::option::Option::Some(MessageType::MessageType_FirmwareErase),
            7 => ::std::option::Option::Some(MessageType::MessageType_FirmwareUpload),
            8 => ::std::option::Option::Some(MessageType::MessageType_FirmwareRequest),
            9 => ::std::option::Option::Some(MessageType::MessageType_GetEntropy),
            10 => ::std::option::Option::Some(MessageType::MessageType_Entropy),
            11 => ::std::option::Option::Some(MessageType::MessageType_GetPublicKey),
            12 => ::std::option::Option::Some(MessageType::MessageType_PublicKey),
            13 => ::std::option::Option::Some(MessageType::MessageType_LoadDevice),
            14 => ::std::option::Option::Some(MessageType::MessageType_ResetDevice),
            15 => ::std::option::Option::Some(MessageType::MessageType_SignTx),
            16 => ::std::option::Option::Some(MessageType::MessageType_SimpleSignTx),
            17 => ::std::option::Option::Some(MessageType::MessageType_Features),
            18 => ::std::option::Option::Some(MessageType::MessageType_PinMatrixRequest),
            19 => ::std::option::Option::Some(MessageType::MessageType_PinMatrixAck),
            20 => ::std::option::Option::Some(MessageType::MessageType_Cancel),
            21 => ::std::option::Option::Some(MessageType::MessageType_TxRequest),
            22 => ::std::option::Option::Some(MessageType::MessageType_TxAck),
            23 => ::std::option::Option::Some(MessageType::MessageType_CipherKeyValue),
            24 => ::std::option::Option::Some(MessageType::MessageType_ClearSession),
            25 => ::std::option::Option::Some(MessageType::MessageType_ApplySettings),
            26 => ::std::option::Option::Some(MessageType::MessageType_ButtonRequest),
            27 => ::std::option::Option::Some(MessageType::MessageType_ButtonAck),
            28 => ::std::option::Option::Some(MessageType::MessageType_ApplyFlags),
            29 => ::std::option::Option::Some(MessageType::MessageType_GetAddress),
            30 => ::std::option::Option::Some(MessageType::MessageType_Address),
            32 => ::std::option::Option::Some(MessageType::MessageType_SelfTest),
            34 => ::std::option::Option::Some(MessageType::MessageType_BackupDevice),
            35 => ::std::option::Option::Some(MessageType::MessageType_EntropyRequest),
            36 => ::std::option::Option::Some(MessageType::MessageType_EntropyAck),
            38 => ::std::option::Option::Some(MessageType::MessageType_SignMessage),
            39 => ::std::option::Option::Some(MessageType::MessageType_VerifyMessage),
            40 => ::std::option::Option::Some(MessageType::MessageType_MessageSignature),
            41 => ::std::option::Option::Some(MessageType::MessageType_PassphraseRequest),
            42 => ::std::option::Option::Some(MessageType::MessageType_PassphraseAck),
            43 => ::std::option::Option::Some(MessageType::MessageType_EstimateTxSize),
            44 => ::std::option::Option::Some(MessageType::MessageType_TxSize),
            45 => ::std::option::Option::Some(MessageType::MessageType_RecoveryDevice),
            46 => ::std::option::Option::Some(MessageType::MessageType_WordRequest),
            47 => ::std::option::Option::Some(MessageType::MessageType_WordAck),
            48 => ::std::option::Option::Some(MessageType::MessageType_CipheredKeyValue),
            49 => ::std::option::Option::Some(MessageType::MessageType_EncryptMessage),
            50 => ::std::option::Option::Some(MessageType::MessageType_EncryptedMessage),
            51 => ::std::option::Option::Some(MessageType::MessageType_DecryptMessage),
            52 => ::std::option::Option::Some(MessageType::MessageType_DecryptedMessage),
            53 => ::std::option::Option::Some(MessageType::MessageType_SignIdentity),
            54 => ::std::option::Option::Some(MessageType::MessageType_SignedIdentity),
            55 => ::std::option::Option::Some(MessageType::MessageType_GetFeatures),
            56 => ::std::option::Option::Some(MessageType::MessageType_EthereumGetAddress),
            57 => ::std::option::Option::Some(MessageType::MessageType_EthereumAddress),
            58 => ::std::option::Option::Some(MessageType::MessageType_EthereumSignTx),
            59 => ::std::option::Option::Some(MessageType::MessageType_EthereumTxRequest),
            60 => ::std::option::Option::Some(MessageType::MessageType_EthereumTxAck),
            61 => ::std::option::Option::Some(MessageType::MessageType_GetECDHSessionKey),
            62 => ::std::option::Option::Some(MessageType::MessageType_ECDHSessionKey),
            63 => ::std::option::Option::Some(MessageType::MessageType_SetU2FCounter),
            64 => ::std::option::Option::Some(MessageType::MessageType_EthereumSignMessage),
            65 => ::std::option::Option::Some(MessageType::MessageType_EthereumVerifyMessage),
            66 => ::std::option::Option::Some(MessageType::MessageType_EthereumMessageSignature),
            100 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkDecision),
            101 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkGetState),
            102 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkState),
            103 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkStop),
            104 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkLog),
            110 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkMemoryRead),
            111 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkMemory),
            112 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkMemoryWrite),
            113 => ::std::option::Option::Some(MessageType::MessageType_DebugLinkFlashErase),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [MessageType] = &[
            MessageType::MessageType_Initialize,
            MessageType::MessageType_Ping,
            MessageType::MessageType_Success,
            MessageType::MessageType_Failure,
            MessageType::MessageType_ChangePin,
            MessageType::MessageType_WipeDevice,
            MessageType::MessageType_FirmwareErase,
            MessageType::MessageType_FirmwareUpload,
            MessageType::MessageType_FirmwareRequest,
            MessageType::MessageType_GetEntropy,
            MessageType::MessageType_Entropy,
            MessageType::MessageType_GetPublicKey,
            MessageType::MessageType_PublicKey,
            MessageType::MessageType_LoadDevice,
            MessageType::MessageType_ResetDevice,
            MessageType::MessageType_SignTx,
            MessageType::MessageType_SimpleSignTx,
            MessageType::MessageType_Features,
            MessageType::MessageType_PinMatrixRequest,
            MessageType::MessageType_PinMatrixAck,
            MessageType::MessageType_Cancel,
            MessageType::MessageType_TxRequest,
            MessageType::MessageType_TxAck,
            MessageType::MessageType_CipherKeyValue,
            MessageType::MessageType_ClearSession,
            MessageType::MessageType_ApplySettings,
            MessageType::MessageType_ButtonRequest,
            MessageType::MessageType_ButtonAck,
            MessageType::MessageType_ApplyFlags,
            MessageType::MessageType_GetAddress,
            MessageType::MessageType_Address,
            MessageType::MessageType_SelfTest,
            MessageType::MessageType_BackupDevice,
            MessageType::MessageType_EntropyRequest,
            MessageType::MessageType_EntropyAck,
            MessageType::MessageType_SignMessage,
            MessageType::MessageType_VerifyMessage,
            MessageType::MessageType_MessageSignature,
            MessageType::MessageType_PassphraseRequest,
            MessageType::MessageType_PassphraseAck,
            MessageType::MessageType_EstimateTxSize,
            MessageType::MessageType_TxSize,
            MessageType::MessageType_RecoveryDevice,
            MessageType::MessageType_WordRequest,
            MessageType::MessageType_WordAck,
            MessageType::MessageType_CipheredKeyValue,
            MessageType::MessageType_EncryptMessage,
            MessageType::MessageType_EncryptedMessage,
            MessageType::MessageType_DecryptMessage,
            MessageType::MessageType_DecryptedMessage,
            MessageType::MessageType_SignIdentity,
            MessageType::MessageType_SignedIdentity,
            MessageType::MessageType_GetFeatures,
            MessageType::MessageType_EthereumGetAddress,
            MessageType::MessageType_EthereumAddress,
            MessageType::MessageType_EthereumSignTx,
            MessageType::MessageType_EthereumTxRequest,
            MessageType::MessageType_EthereumTxAck,
            MessageType::MessageType_GetECDHSessionKey,
            MessageType::MessageType_ECDHSessionKey,
            MessageType::MessageType_SetU2FCounter,
            MessageType::MessageType_EthereumSignMessage,
            MessageType::MessageType_EthereumVerifyMessage,
            MessageType::MessageType_EthereumMessageSignature,
            MessageType::MessageType_DebugLinkDecision,
            MessageType::MessageType_DebugLinkGetState,
            MessageType::MessageType_DebugLinkState,
            MessageType::MessageType_DebugLinkStop,
            MessageType::MessageType_DebugLinkLog,
            MessageType::MessageType_DebugLinkMemoryRead,
            MessageType::MessageType_DebugLinkMemory,
            MessageType::MessageType_DebugLinkMemoryWrite,
            MessageType::MessageType_DebugLinkFlashErase,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<MessageType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("MessageType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for MessageType {
}

impl ::protobuf::reflect::ProtobufValue for MessageType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0emessages.proto\x1a\x0btypes.proto\"\x0c\n\nInitialize\"\r\n\x0bGet\
    Features\"\xb9\x05\n\x08Features\x12\x16\n\x06vendor\x18\x01\x20\x01(\tR\
    \x06vendor\x12#\n\rmajor_version\x18\x02\x20\x01(\rR\x0cmajorVersion\x12\
    #\n\rminor_version\x18\x03\x20\x01(\rR\x0cminorVersion\x12#\n\rpatch_ver\
    sion\x18\x04\x20\x01(\rR\x0cpatchVersion\x12'\n\x0fbootloader_mode\x18\
    \x05\x20\x01(\x08R\x0ebootloaderMode\x12\x1b\n\tdevice_id\x18\x06\x20\
    \x01(\tR\x08deviceId\x12%\n\x0epin_protection\x18\x07\x20\x01(\x08R\rpin\
    Protection\x123\n\x15passphrase_protection\x18\x08\x20\x01(\x08R\x14pass\
    phraseProtection\x12\x1a\n\x08language\x18\t\x20\x01(\tR\x08language\x12\
    \x14\n\x05label\x18\n\x20\x01(\tR\x05label\x12\x1f\n\x05coins\x18\x0b\
    \x20\x03(\x0b2\t.CoinTypeR\x05coins\x12\x20\n\x0binitialized\x18\x0c\x20\
    \x01(\x08R\x0binitialized\x12\x1a\n\x08revision\x18\r\x20\x01(\x0cR\x08r\
    evision\x12'\n\x0fbootloader_hash\x18\x0e\x20\x01(\x0cR\x0ebootloaderHas\
    h\x12\x1a\n\x08imported\x18\x0f\x20\x01(\x08R\x08imported\x12\x1d\n\npin\
    _cached\x18\x10\x20\x01(\x08R\tpinCached\x12+\n\x11passphrase_cached\x18\
    \x11\x20\x01(\x08R\x10passphraseCached\x12)\n\x10firmware_present\x18\
    \x12\x20\x01(\x08R\x0ffirmwarePresent\x12!\n\x0cneeds_backup\x18\x13\x20\
    \x01(\x08R\x0bneedsBackup\x12\x14\n\x05flags\x18\x14\x20\x01(\rR\x05flag\
    s\"\x0e\n\x0cClearSession\"\x88\x01\n\rApplySettings\x12\x1a\n\x08langua\
    ge\x18\x01\x20\x01(\tR\x08language\x12\x14\n\x05label\x18\x02\x20\x01(\t\
    R\x05label\x12%\n\x0euse_passphrase\x18\x03\x20\x01(\x08R\rusePassphrase\
    \x12\x1e\n\nhomescreen\x18\x04\x20\x01(\x0cR\nhomescreen\"\"\n\nApplyFla\
    gs\x12\x14\n\x05flags\x18\x01\x20\x01(\rR\x05flags\"#\n\tChangePin\x12\
    \x16\n\x06remove\x18\x01\x20\x01(\x08R\x06remove\"\xa9\x01\n\x04Ping\x12\
    \x18\n\x07message\x18\x01\x20\x01(\tR\x07message\x12+\n\x11button_protec\
    tion\x18\x02\x20\x01(\x08R\x10buttonProtection\x12%\n\x0epin_protection\
    \x18\x03\x20\x01(\x08R\rpinProtection\x123\n\x15passphrase_protection\
    \x18\x04\x20\x01(\x08R\x14passphraseProtection\"#\n\x07Success\x12\x18\n\
    \x07message\x18\x01\x20\x01(\tR\x07message\"E\n\x07Failure\x12\x20\n\x04\
    code\x18\x01\x20\x01(\x0e2\x0c.FailureTypeR\x04code\x12\x18\n\x07message\
    \x18\x02\x20\x01(\tR\x07message\"K\n\rButtonRequest\x12&\n\x04code\x18\
    \x01\x20\x01(\x0e2\x12.ButtonRequestTypeR\x04code\x12\x12\n\x04data\x18\
    \x02\x20\x01(\tR\x04data\"\x0b\n\tButtonAck\"=\n\x10PinMatrixRequest\x12\
    )\n\x04type\x18\x01\x20\x01(\x0e2\x15.PinMatrixRequestTypeR\x04type\"\
    \x20\n\x0cPinMatrixAck\x12\x10\n\x03pin\x18\x01\x20\x02(\tR\x03pin\"\x08\
    \n\x06Cancel\"\x13\n\x11PassphraseRequest\"/\n\rPassphraseAck\x12\x1e\n\
    \npassphrase\x18\x01\x20\x02(\tR\npassphrase\"\x20\n\nGetEntropy\x12\x12\
    \n\x04size\x18\x01\x20\x02(\rR\x04size\"#\n\x07Entropy\x12\x18\n\x07entr\
    opy\x18\x01\x20\x02(\x0cR\x07entropy\"\x9e\x01\n\x0cGetPublicKey\x12\x1b\
    \n\taddress_n\x18\x01\x20\x03(\rR\x08addressN\x12(\n\x10ecdsa_curve_name\
    \x18\x02\x20\x01(\tR\x0eecdsaCurveName\x12!\n\x0cshow_display\x18\x03\
    \x20\x01(\x08R\x0bshowDisplay\x12$\n\tcoin_name\x18\x04\x20\x01(\t:\x07B\
    itcoinR\x08coinName\"@\n\tPublicKey\x12\x1f\n\x04node\x18\x01\x20\x02(\
    \x0b2\x0b.HDNodeTypeR\x04node\x12\x12\n\x04xpub\x18\x02\x20\x01(\tR\x04x\
    pub\"\xea\x01\n\nGetAddress\x12\x1b\n\taddress_n\x18\x01\x20\x03(\rR\x08\
    addressN\x12$\n\tcoin_name\x18\x02\x20\x01(\t:\x07BitcoinR\x08coinName\
    \x12!\n\x0cshow_display\x18\x03\x20\x01(\x08R\x0bshowDisplay\x125\n\x08m\
    ultisig\x18\x04\x20\x01(\x0b2\x19.MultisigRedeemScriptTypeR\x08multisig\
    \x12?\n\x0bscript_type\x18\x05\x20\x01(\x0e2\x10.InputScriptType:\x0cSPE\
    NDADDRESSR\nscriptType\"T\n\x12EthereumGetAddress\x12\x1b\n\taddress_n\
    \x18\x01\x20\x03(\rR\x08addressN\x12!\n\x0cshow_display\x18\x02\x20\x01(\
    \x08R\x0bshowDisplay\"#\n\x07Address\x12\x18\n\x07address\x18\x01\x20\
    \x02(\tR\x07address\"+\n\x0fEthereumAddress\x12\x18\n\x07address\x18\x01\
    \x20\x02(\x0cR\x07address\"\x0c\n\nWipeDevice\"\x91\x02\n\nLoadDevice\
    \x12\x1a\n\x08mnemonic\x18\x01\x20\x01(\tR\x08mnemonic\x12\x1f\n\x04node\
    \x18\x02\x20\x01(\x0b2\x0b.HDNodeTypeR\x04node\x12\x10\n\x03pin\x18\x03\
    \x20\x01(\tR\x03pin\x123\n\x15passphrase_protection\x18\x04\x20\x01(\x08\
    R\x14passphraseProtection\x12#\n\x08language\x18\x05\x20\x01(\t:\x07engl\
    ishR\x08language\x12\x14\n\x05label\x18\x06\x20\x01(\tR\x05label\x12#\n\
    \rskip_checksum\x18\x07\x20\x01(\x08R\x0cskipChecksum\x12\x1f\n\x0bu2f_c\
    ounter\x18\x08\x20\x01(\rR\nu2fCounter\"\xae\x02\n\x0bResetDevice\x12%\n\
    \x0edisplay_random\x18\x01\x20\x01(\x08R\rdisplayRandom\x12\x1f\n\x08str\
    ength\x18\x02\x20\x01(\r:\x03256R\x08strength\x123\n\x15passphrase_prote\
    ction\x18\x03\x20\x01(\x08R\x14passphraseProtection\x12%\n\x0epin_protec\
    tion\x18\x04\x20\x01(\x08R\rpinProtection\x12#\n\x08language\x18\x05\x20\
    \x01(\t:\x07englishR\x08language\x12\x14\n\x05label\x18\x06\x20\x01(\tR\
    \x05label\x12\x1f\n\x0bu2f_counter\x18\x07\x20\x01(\rR\nu2fCounter\x12\
    \x1f\n\x0bskip_backup\x18\x08\x20\x01(\x08R\nskipBackup\"\x0e\n\x0cBacku\
    pDevice\"\x10\n\x0eEntropyRequest\"&\n\nEntropyAck\x12\x18\n\x07entropy\
    \x18\x01\x20\x01(\x0cR\x07entropy\"\xbf\x02\n\x0eRecoveryDevice\x12\x1d\
    \n\nword_count\x18\x01\x20\x01(\rR\twordCount\x123\n\x15passphrase_prote\
    ction\x18\x02\x20\x01(\x08R\x14passphraseProtection\x12%\n\x0epin_protec\
    tion\x18\x03\x20\x01(\x08R\rpinProtection\x12#\n\x08language\x18\x04\x20\
    \x01(\t:\x07englishR\x08language\x12\x14\n\x05label\x18\x05\x20\x01(\tR\
    \x05label\x12)\n\x10enforce_wordlist\x18\x06\x20\x01(\x08R\x0fenforceWor\
    dlist\x12\x12\n\x04type\x18\x08\x20\x01(\rR\x04type\x12\x1f\n\x0bu2f_cou\
    nter\x18\t\x20\x01(\rR\nu2fCounter\x12\x17\n\x07dry_run\x18\n\x20\x01(\
    \x08R\x06dryRun\"3\n\x0bWordRequest\x12$\n\x04type\x18\x01\x20\x01(\x0e2\
    \x10.WordRequestTypeR\x04type\"\x1d\n\x07WordAck\x12\x12\n\x04word\x18\
    \x01\x20\x02(\tR\x04word\"\xab\x01\n\x0bSignMessage\x12\x1b\n\taddress_n\
    \x18\x01\x20\x03(\rR\x08addressN\x12\x18\n\x07message\x18\x02\x20\x02(\
    \x0cR\x07message\x12$\n\tcoin_name\x18\x03\x20\x01(\t:\x07BitcoinR\x08co\
    inName\x12?\n\x0bscript_type\x18\x04\x20\x01(\x0e2\x10.InputScriptType:\
    \x0cSPENDADDRESSR\nscriptType\"\x87\x01\n\rVerifyMessage\x12\x18\n\x07ad\
    dress\x18\x01\x20\x01(\tR\x07address\x12\x1c\n\tsignature\x18\x02\x20\
    \x01(\x0cR\tsignature\x12\x18\n\x07message\x18\x03\x20\x01(\x0cR\x07mess\
    age\x12$\n\tcoin_name\x18\x04\x20\x01(\t:\x07BitcoinR\x08coinName\"J\n\
    \x10MessageSignature\x12\x18\n\x07address\x18\x01\x20\x01(\tR\x07address\
    \x12\x1c\n\tsignature\x18\x02\x20\x01(\x0cR\tsignature\"\xa8\x01\n\x0eEn\
    cryptMessage\x12\x16\n\x06pubkey\x18\x01\x20\x01(\x0cR\x06pubkey\x12\x18\
    \n\x07message\x18\x02\x20\x01(\x0cR\x07message\x12!\n\x0cdisplay_only\
    \x18\x03\x20\x01(\x08R\x0bdisplayOnly\x12\x1b\n\taddress_n\x18\x04\x20\
    \x03(\rR\x08addressN\x12$\n\tcoin_name\x18\x05\x20\x01(\t:\x07BitcoinR\
    \x08coinName\"V\n\x10EncryptedMessage\x12\x14\n\x05nonce\x18\x01\x20\x01\
    (\x0cR\x05nonce\x12\x18\n\x07message\x18\x02\x20\x01(\x0cR\x07message\
    \x12\x12\n\x04hmac\x18\x03\x20\x01(\x0cR\x04hmac\"q\n\x0eDecryptMessage\
    \x12\x1b\n\taddress_n\x18\x01\x20\x03(\rR\x08addressN\x12\x14\n\x05nonce\
    \x18\x02\x20\x01(\x0cR\x05nonce\x12\x18\n\x07message\x18\x03\x20\x01(\
    \x0cR\x07message\x12\x12\n\x04hmac\x18\x04\x20\x01(\x0cR\x04hmac\"F\n\
    \x10DecryptedMessage\x12\x18\n\x07message\x18\x01\x20\x01(\x0cR\x07messa\
    ge\x12\x18\n\x07address\x18\x02\x20\x01(\tR\x07address\"\xcb\x01\n\x0eCi\
    pherKeyValue\x12\x1b\n\taddress_n\x18\x01\x20\x03(\rR\x08addressN\x12\
    \x10\n\x03key\x18\x02\x20\x01(\tR\x03key\x12\x14\n\x05value\x18\x03\x20\
    \x01(\x0cR\x05value\x12\x18\n\x07encrypt\x18\x04\x20\x01(\x08R\x07encryp\
    t\x12$\n\x0eask_on_encrypt\x18\x05\x20\x01(\x08R\x0caskOnEncrypt\x12$\n\
    \x0eask_on_decrypt\x18\x06\x20\x01(\x08R\x0caskOnDecrypt\x12\x0e\n\x02iv\
    \x18\x07\x20\x01(\x0cR\x02iv\"(\n\x10CipheredKeyValue\x12\x14\n\x05value\
    \x18\x01\x20\x01(\x0cR\x05value\"~\n\x0eEstimateTxSize\x12#\n\routputs_c\
    ount\x18\x01\x20\x02(\rR\x0coutputsCount\x12!\n\x0cinputs_count\x18\x02\
    \x20\x02(\rR\x0binputsCount\x12$\n\tcoin_name\x18\x03\x20\x01(\t:\x07Bit\
    coinR\x08coinName\"!\n\x06TxSize\x12\x17\n\x07tx_size\x18\x01\x20\x01(\r\
    R\x06txSize\"\xb3\x01\n\x06SignTx\x12#\n\routputs_count\x18\x01\x20\x02(\
    \rR\x0coutputsCount\x12!\n\x0cinputs_count\x18\x02\x20\x02(\rR\x0binputs\
    Count\x12$\n\tcoin_name\x18\x03\x20\x01(\t:\x07BitcoinR\x08coinName\x12\
    \x1b\n\x07version\x18\x04\x20\x01(\r:\x011R\x07version\x12\x1e\n\tlock_t\
    ime\x18\x05\x20\x01(\r:\x010R\x08lockTime\"\xf6\x01\n\x0cSimpleSignTx\
    \x12$\n\x06inputs\x18\x01\x20\x03(\x0b2\x0c.TxInputTypeR\x06inputs\x12'\
    \n\x07outputs\x18\x02\x20\x03(\x0b2\r.TxOutputTypeR\x07outputs\x124\n\
    \x0ctransactions\x18\x03\x20\x03(\x0b2\x10.TransactionTypeR\x0ctransacti\
    ons\x12$\n\tcoin_name\x18\x04\x20\x01(\t:\x07BitcoinR\x08coinName\x12\
    \x1b\n\x07version\x18\x05\x20\x01(\r:\x011R\x07version\x12\x1e\n\tlock_t\
    ime\x18\x06\x20\x01(\r:\x010R\x08lockTime\"\xa7\x01\n\tTxRequest\x12/\n\
    \x0crequest_type\x18\x01\x20\x01(\x0e2\x0c.RequestTypeR\x0brequestType\
    \x12/\n\x07details\x18\x02\x20\x01(\x0b2\x15.TxRequestDetailsTypeR\x07de\
    tails\x128\n\nserialized\x18\x03\x20\x01(\x0b2\x18.TxRequestSerializedTy\
    peR\nserialized\")\n\x05TxAck\x12\x20\n\x02tx\x18\x01\x20\x01(\x0b2\x10.\
    TransactionTypeR\x02tx\"\x8d\x02\n\x0eEthereumSignTx\x12\x1b\n\taddress_\
    n\x18\x01\x20\x03(\rR\x08addressN\x12\x14\n\x05nonce\x18\x02\x20\x01(\
    \x0cR\x05nonce\x12\x1b\n\tgas_price\x18\x03\x20\x01(\x0cR\x08gasPrice\
    \x12\x1b\n\tgas_limit\x18\x04\x20\x01(\x0cR\x08gasLimit\x12\x0e\n\x02to\
    \x18\x05\x20\x01(\x0cR\x02to\x12\x14\n\x05value\x18\x06\x20\x01(\x0cR\
    \x05value\x12,\n\x12data_initial_chunk\x18\x07\x20\x01(\x0cR\x10dataInit\
    ialChunk\x12\x1f\n\x0bdata_length\x18\x08\x20\x01(\rR\ndataLength\x12\
    \x19\n\x08chain_id\x18\t\x20\x01(\rR\x07chainId\"\x97\x01\n\x11EthereumT\
    xRequest\x12\x1f\n\x0bdata_length\x18\x01\x20\x01(\rR\ndataLength\x12\
    \x1f\n\x0bsignature_v\x18\x02\x20\x01(\rR\nsignatureV\x12\x1f\n\x0bsigna\
    ture_r\x18\x03\x20\x01(\x0cR\nsignatureR\x12\x1f\n\x0bsignature_s\x18\
    \x04\x20\x01(\x0cR\nsignatureS\".\n\rEthereumTxAck\x12\x1d\n\ndata_chunk\
    \x18\x01\x20\x01(\x0cR\tdataChunk\"L\n\x13EthereumSignMessage\x12\x1b\n\
    \taddress_n\x18\x01\x20\x03(\rR\x08addressN\x12\x18\n\x07message\x18\x02\
    \x20\x02(\x0cR\x07message\"i\n\x15EthereumVerifyMessage\x12\x18\n\x07add\
    ress\x18\x01\x20\x01(\x0cR\x07address\x12\x1c\n\tsignature\x18\x02\x20\
    \x01(\x0cR\tsignature\x12\x18\n\x07message\x18\x03\x20\x01(\x0cR\x07mess\
    age\"R\n\x18EthereumMessageSignature\x12\x18\n\x07address\x18\x01\x20\
    \x01(\x0cR\x07address\x12\x1c\n\tsignature\x18\x02\x20\x01(\x0cR\tsignat\
    ure\"\xb9\x01\n\x0cSignIdentity\x12)\n\x08identity\x18\x01\x20\x01(\x0b2\
    \r.IdentityTypeR\x08identity\x12)\n\x10challenge_hidden\x18\x02\x20\x01(\
    \x0cR\x0fchallengeHidden\x12)\n\x10challenge_visual\x18\x03\x20\x01(\tR\
    \x0fchallengeVisual\x12(\n\x10ecdsa_curve_name\x18\x04\x20\x01(\tR\x0eec\
    dsaCurveName\"g\n\x0eSignedIdentity\x12\x18\n\x07address\x18\x01\x20\x01\
    (\tR\x07address\x12\x1d\n\npublic_key\x18\x02\x20\x01(\x0cR\tpublicKey\
    \x12\x1c\n\tsignature\x18\x03\x20\x01(\x0cR\tsignature\"\x90\x01\n\x11Ge\
    tECDHSessionKey\x12)\n\x08identity\x18\x01\x20\x01(\x0b2\r.IdentityTypeR\
    \x08identity\x12&\n\x0fpeer_public_key\x18\x02\x20\x01(\x0cR\rpeerPublic\
    Key\x12(\n\x10ecdsa_curve_name\x18\x03\x20\x01(\tR\x0eecdsaCurveName\"1\
    \n\x0eECDHSessionKey\x12\x1f\n\x0bsession_key\x18\x01\x20\x01(\x0cR\nses\
    sionKey\"0\n\rSetU2FCounter\x12\x1f\n\x0bu2f_counter\x18\x01\x20\x01(\rR\
    \nu2fCounter\"'\n\rFirmwareErase\x12\x16\n\x06length\x18\x01\x20\x01(\rR\
    \x06length\"A\n\x0fFirmwareRequest\x12\x16\n\x06offset\x18\x01\x20\x01(\
    \rR\x06offset\x12\x16\n\x06length\x18\x02\x20\x01(\rR\x06length\">\n\x0e\
    FirmwareUpload\x12\x18\n\x07payload\x18\x01\x20\x02(\x0cR\x07payload\x12\
    \x12\n\x04hash\x18\x02\x20\x01(\x0cR\x04hash\"$\n\x08SelfTest\x12\x18\n\
    \x07payload\x18\x01\x20\x01(\x0cR\x07payload\"*\n\x11DebugLinkDecision\
    \x12\x15\n\x06yes_no\x18\x01\x20\x02(\x08R\x05yesNo\"\x13\n\x11DebugLink\
    GetState\"\xe2\x02\n\x0eDebugLinkState\x12\x16\n\x06layout\x18\x01\x20\
    \x01(\x0cR\x06layout\x12\x10\n\x03pin\x18\x02\x20\x01(\tR\x03pin\x12\x16\
    \n\x06matrix\x18\x03\x20\x01(\tR\x06matrix\x12\x1a\n\x08mnemonic\x18\x04\
    \x20\x01(\tR\x08mnemonic\x12\x1f\n\x04node\x18\x05\x20\x01(\x0b2\x0b.HDN\
    odeTypeR\x04node\x123\n\x15passphrase_protection\x18\x06\x20\x01(\x08R\
    \x14passphraseProtection\x12\x1d\n\nreset_word\x18\x07\x20\x01(\tR\trese\
    tWord\x12#\n\rreset_entropy\x18\x08\x20\x01(\x0cR\x0cresetEntropy\x12,\n\
    \x12recovery_fake_word\x18\t\x20\x01(\tR\x10recoveryFakeWord\x12*\n\x11r\
    ecovery_word_pos\x18\n\x20\x01(\rR\x0frecoveryWordPos\"\x0f\n\rDebugLink\
    Stop\"P\n\x0cDebugLinkLog\x12\x14\n\x05level\x18\x01\x20\x01(\rR\x05leve\
    l\x12\x16\n\x06bucket\x18\x02\x20\x01(\tR\x06bucket\x12\x12\n\x04text\
    \x18\x03\x20\x01(\tR\x04text\"G\n\x13DebugLinkMemoryRead\x12\x18\n\x07ad\
    dress\x18\x01\x20\x01(\rR\x07address\x12\x16\n\x06length\x18\x02\x20\x01\
    (\rR\x06length\")\n\x0fDebugLinkMemory\x12\x16\n\x06memory\x18\x01\x20\
    \x01(\x0cR\x06memory\"^\n\x14DebugLinkMemoryWrite\x12\x18\n\x07address\
    \x18\x01\x20\x01(\rR\x07address\x12\x16\n\x06memory\x18\x02\x20\x01(\x0c\
    R\x06memory\x12\x14\n\x05flash\x18\x03\x20\x01(\x08R\x05flash\"-\n\x13De\
    bugLinkFlashErase\x12\x16\n\x06sector\x18\x01\x20\x01(\rR\x06sector*\xab\
    \x15\n\x0bMessageType\x12\x20\n\x16MessageType_Initialize\x10\0\x1a\x04\
    \x90\xb5\x18\x01\x12\x1a\n\x10MessageType_Ping\x10\x01\x1a\x04\x90\xb5\
    \x18\x01\x12\x1d\n\x13MessageType_Success\x10\x02\x1a\x04\x98\xb5\x18\
    \x01\x12\x1d\n\x13MessageType_Failure\x10\x03\x1a\x04\x98\xb5\x18\x01\
    \x12\x1f\n\x15MessageType_ChangePin\x10\x04\x1a\x04\x90\xb5\x18\x01\x12\
    \x20\n\x16MessageType_WipeDevice\x10\x05\x1a\x04\x90\xb5\x18\x01\x12'\n\
    \x19MessageType_FirmwareErase\x10\x06\x1a\x08\x90\xb5\x18\x01\xb8\xb5\
    \x18\x01\x12(\n\x1aMessageType_FirmwareUpload\x10\x07\x1a\x08\x90\xb5\
    \x18\x01\xb8\xb5\x18\x01\x12)\n\x1bMessageType_FirmwareRequest\x10\x08\
    \x1a\x08\x98\xb5\x18\x01\xb8\xb5\x18\x01\x12\x20\n\x16MessageType_GetEnt\
    ropy\x10\t\x1a\x04\x90\xb5\x18\x01\x12\x1d\n\x13MessageType_Entropy\x10\
    \n\x1a\x04\x98\xb5\x18\x01\x12\"\n\x18MessageType_GetPublicKey\x10\x0b\
    \x1a\x04\x90\xb5\x18\x01\x12\x1f\n\x15MessageType_PublicKey\x10\x0c\x1a\
    \x04\x98\xb5\x18\x01\x12\x20\n\x16MessageType_LoadDevice\x10\r\x1a\x04\
    \x90\xb5\x18\x01\x12!\n\x17MessageType_ResetDevice\x10\x0e\x1a\x04\x90\
    \xb5\x18\x01\x12\x1c\n\x12MessageType_SignTx\x10\x0f\x1a\x04\x90\xb5\x18\
    \x01\x12$\n\x18MessageType_SimpleSignTx\x10\x10\x1a\x06\x08\x01\x90\xb5\
    \x18\x01\x12\x1e\n\x14MessageType_Features\x10\x11\x1a\x04\x98\xb5\x18\
    \x01\x12&\n\x1cMessageType_PinMatrixRequest\x10\x12\x1a\x04\x98\xb5\x18\
    \x01\x12&\n\x18MessageType_PinMatrixAck\x10\x13\x1a\x08\x90\xb5\x18\x01\
    \xb0\xb5\x18\x01\x12\x1c\n\x12MessageType_Cancel\x10\x14\x1a\x04\x90\xb5\
    \x18\x01\x12\x1f\n\x15MessageType_TxRequest\x10\x15\x1a\x04\x98\xb5\x18\
    \x01\x12\x1b\n\x11MessageType_TxAck\x10\x16\x1a\x04\x90\xb5\x18\x01\x12$\
    \n\x1aMessageType_CipherKeyValue\x10\x17\x1a\x04\x90\xb5\x18\x01\x12\"\n\
    \x18MessageType_ClearSession\x10\x18\x1a\x04\x90\xb5\x18\x01\x12#\n\x19M\
    essageType_ApplySettings\x10\x19\x1a\x04\x90\xb5\x18\x01\x12#\n\x19Messa\
    geType_ButtonRequest\x10\x1a\x1a\x04\x98\xb5\x18\x01\x12#\n\x15MessageTy\
    pe_ButtonAck\x10\x1b\x1a\x08\xb0\xb5\x18\x01\x90\xb5\x18\x01\x12\x20\n\
    \x16MessageType_ApplyFlags\x10\x1c\x1a\x04\x90\xb5\x18\x01\x12\x20\n\x16\
    MessageType_GetAddress\x10\x1d\x1a\x04\x90\xb5\x18\x01\x12\x1d\n\x13Mess\
    ageType_Address\x10\x1e\x1a\x04\x98\xb5\x18\x01\x12\"\n\x14MessageType_S\
    elfTest\x10\x20\x1a\x08\x90\xb5\x18\x01\xb8\xb5\x18\x01\x12\"\n\x18Messa\
    geType_BackupDevice\x10\"\x1a\x04\x90\xb5\x18\x01\x12$\n\x1aMessageType_\
    EntropyRequest\x10#\x1a\x04\x98\xb5\x18\x01\x12\x20\n\x16MessageType_Ent\
    ropyAck\x10$\x1a\x04\x90\xb5\x18\x01\x12!\n\x17MessageType_SignMessage\
    \x10&\x1a\x04\x90\xb5\x18\x01\x12#\n\x19MessageType_VerifyMessage\x10'\
    \x1a\x04\x90\xb5\x18\x01\x12&\n\x1cMessageType_MessageSignature\x10(\x1a\
    \x04\x98\xb5\x18\x01\x12'\n\x1dMessageType_PassphraseRequest\x10)\x1a\
    \x04\x98\xb5\x18\x01\x12'\n\x19MessageType_PassphraseAck\x10*\x1a\x08\
    \xb0\xb5\x18\x01\x90\xb5\x18\x01\x12&\n\x1aMessageType_EstimateTxSize\
    \x10+\x1a\x06\x08\x01\x90\xb5\x18\x01\x12\x1e\n\x12MessageType_TxSize\
    \x10,\x1a\x06\x08\x01\x98\xb5\x18\x01\x12$\n\x1aMessageType_RecoveryDevi\
    ce\x10-\x1a\x04\x90\xb5\x18\x01\x12!\n\x17MessageType_WordRequest\x10.\
    \x1a\x04\x98\xb5\x18\x01\x12\x1d\n\x13MessageType_WordAck\x10/\x1a\x04\
    \x90\xb5\x18\x01\x12&\n\x1cMessageType_CipheredKeyValue\x100\x1a\x04\x98\
    \xb5\x18\x01\x12&\n\x1aMessageType_EncryptMessage\x101\x1a\x06\x08\x01\
    \x90\xb5\x18\x01\x12(\n\x1cMessageType_EncryptedMessage\x102\x1a\x06\x08\
    \x01\x98\xb5\x18\x01\x12&\n\x1aMessageType_DecryptMessage\x103\x1a\x06\
    \x08\x01\x90\xb5\x18\x01\x12(\n\x1cMessageType_DecryptedMessage\x104\x1a\
    \x06\x08\x01\x98\xb5\x18\x01\x12\"\n\x18MessageType_SignIdentity\x105\
    \x1a\x04\x90\xb5\x18\x01\x12$\n\x1aMessageType_SignedIdentity\x106\x1a\
    \x04\x98\xb5\x18\x01\x12!\n\x17MessageType_GetFeatures\x107\x1a\x04\x90\
    \xb5\x18\x01\x12(\n\x1eMessageType_EthereumGetAddress\x108\x1a\x04\x90\
    \xb5\x18\x01\x12%\n\x1bMessageType_EthereumAddress\x109\x1a\x04\x98\xb5\
    \x18\x01\x12$\n\x1aMessageType_EthereumSignTx\x10:\x1a\x04\x90\xb5\x18\
    \x01\x12'\n\x1dMessageType_EthereumTxRequest\x10;\x1a\x04\x98\xb5\x18\
    \x01\x12#\n\x19MessageType_EthereumTxAck\x10<\x1a\x04\x90\xb5\x18\x01\
    \x12'\n\x1dMessageType_GetECDHSessionKey\x10=\x1a\x04\x90\xb5\x18\x01\
    \x12$\n\x1aMessageType_ECDHSessionKey\x10>\x1a\x04\x98\xb5\x18\x01\x12#\
    \n\x19MessageType_SetU2FCounter\x10?\x1a\x04\x90\xb5\x18\x01\x12)\n\x1fM\
    essageType_EthereumSignMessage\x10@\x1a\x04\x90\xb5\x18\x01\x12+\n!Messa\
    geType_EthereumVerifyMessage\x10A\x1a\x04\x90\xb5\x18\x01\x12.\n$Message\
    Type_EthereumMessageSignature\x10B\x1a\x04\x98\xb5\x18\x01\x12+\n\x1dMes\
    sageType_DebugLinkDecision\x10d\x1a\x08\xb0\xb5\x18\x01\xa0\xb5\x18\x01\
    \x12'\n\x1dMessageType_DebugLinkGetState\x10e\x1a\x04\xa0\xb5\x18\x01\
    \x12$\n\x1aMessageType_DebugLinkState\x10f\x1a\x04\xa8\xb5\x18\x01\x12#\
    \n\x19MessageType_DebugLinkStop\x10g\x1a\x04\xa0\xb5\x18\x01\x12\"\n\x18\
    MessageType_DebugLinkLog\x10h\x1a\x04\xa8\xb5\x18\x01\x12)\n\x1fMessageT\
    ype_DebugLinkMemoryRead\x10n\x1a\x04\xa0\xb5\x18\x01\x12%\n\x1bMessageTy\
    pe_DebugLinkMemory\x10o\x1a\x04\xa8\xb5\x18\x01\x12*\n\x20MessageType_De\
    bugLinkMemoryWrite\x10p\x1a\x04\xa0\xb5\x18\x01\x12)\n\x1fMessageType_De\
    bugLinkFlashErase\x10q\x1a\x04\xa0\xb5\x18\x01B4\n#com.satoshilabs.trezo\
    r.lib.protobufB\rTrezorMessageJ\x81\xd7\x02\n\x07\x12\x05\x05\0\x82\x07\
    \x01\n\x08\n\x01\x08\x12\x03\x05\0<\nW\n\x04\x08\xe7\x07\0\x12\x03\x05\0\
    <\x1a#\x20Sugar\x20for\x20easier\x20handling\x20in\x20Java\n2%*\n\x20Mes\
    sages\x20for\x20TREZOR\x20communication\n\n\x0c\n\x05\x08\xe7\x07\0\x02\
    \x12\x03\x05\x07\x13\n\r\n\x06\x08\xe7\x07\0\x02\0\x12\x03\x05\x07\x13\n\
    \x0e\n\x07\x08\xe7\x07\0\x02\0\x01\x12\x03\x05\x07\x13\n\x0c\n\x05\x08\
    \xe7\x07\0\x07\x12\x03\x05\x16;\n\x08\n\x01\x08\x12\x03\x06\0.\n\x0b\n\
    \x04\x08\xe7\x07\x01\x12\x03\x06\0.\n\x0c\n\x05\x08\xe7\x07\x01\x02\x12\
    \x03\x06\x07\x1b\n\r\n\x06\x08\xe7\x07\x01\x02\0\x12\x03\x06\x07\x1b\n\
    \x0e\n\x07\x08\xe7\x07\x01\x02\0\x01\x12\x03\x06\x07\x1b\n\x0c\n\x05\x08\
    \xe7\x07\x01\x07\x12\x03\x06\x1e-\n\t\n\x02\x03\0\x12\x03\x08\x07\x14\nT\
    \n\x02\x05\0\x12\x04\r\0W\x01\x1aH*\n\x20Mapping\x20between\x20Trezor\
    \x20wire\x20identifier\x20(uint)\x20and\x20a\x20protobuf\x20message\n\n\
    \n\n\x03\x05\0\x01\x12\x03\r\x05\x10\n\x0b\n\x04\x05\0\x02\0\x12\x03\x0e\
    \x086\n\x0c\n\x05\x05\0\x02\0\x01\x12\x03\x0e\x08\x1e\n\x0c\n\x05\x05\0\
    \x02\0\x02\x12\x03\x0e!\"\n\x0c\n\x05\x05\0\x02\0\x03\x12\x03\x0e#5\n\
    \x0f\n\x08\x05\0\x02\0\x03\xe7\x07\0\x12\x03\x0e$4\n\x10\n\t\x05\0\x02\0\
    \x03\xe7\x07\0\x02\x12\x03\x0e$-\n\x11\n\n\x05\0\x02\0\x03\xe7\x07\0\x02\
    \0\x12\x03\x0e$-\n\x12\n\x0b\x05\0\x02\0\x03\xe7\x07\0\x02\0\x01\x12\x03\
    \x0e%,\n\x10\n\t\x05\0\x02\0\x03\xe7\x07\0\x03\x12\x03\x0e04\n\x0b\n\x04\
    \x05\0\x02\x01\x12\x03\x0f\x080\n\x0c\n\x05\x05\0\x02\x01\x01\x12\x03\
    \x0f\x08\x18\n\x0c\n\x05\x05\0\x02\x01\x02\x12\x03\x0f\x1b\x1c\n\x0c\n\
    \x05\x05\0\x02\x01\x03\x12\x03\x0f\x1d/\n\x0f\n\x08\x05\0\x02\x01\x03\
    \xe7\x07\0\x12\x03\x0f\x1e.\n\x10\n\t\x05\0\x02\x01\x03\xe7\x07\0\x02\
    \x12\x03\x0f\x1e'\n\x11\n\n\x05\0\x02\x01\x03\xe7\x07\0\x02\0\x12\x03\
    \x0f\x1e'\n\x12\n\x0b\x05\0\x02\x01\x03\xe7\x07\0\x02\0\x01\x12\x03\x0f\
    \x1f&\n\x10\n\t\x05\0\x02\x01\x03\xe7\x07\0\x03\x12\x03\x0f*.\n\x0b\n\
    \x04\x05\0\x02\x02\x12\x03\x10\x084\n\x0c\n\x05\x05\0\x02\x02\x01\x12\
    \x03\x10\x08\x1b\n\x0c\n\x05\x05\0\x02\x02\x02\x12\x03\x10\x1e\x1f\n\x0c\
    \n\x05\x05\0\x02\x02\x03\x12\x03\x10\x203\n\x0f\n\x08\x05\0\x02\x02\x03\
    \xe7\x07\0\x12\x03\x10!2\n\x10\n\t\x05\0\x02\x02\x03\xe7\x07\0\x02\x12\
    \x03\x10!+\n\x11\n\n\x05\0\x02\x02\x03\xe7\x07\0\x02\0\x12\x03\x10!+\n\
    \x12\n\x0b\x05\0\x02\x02\x03\xe7\x07\0\x02\0\x01\x12\x03\x10\"*\n\x10\n\
    \t\x05\0\x02\x02\x03\xe7\x07\0\x03\x12\x03\x10.2\n\x0b\n\x04\x05\0\x02\
    \x03\x12\x03\x11\x084\n\x0c\n\x05\x05\0\x02\x03\x01\x12\x03\x11\x08\x1b\
    \n\x0c\n\x05\x05\0\x02\x03\x02\x12\x03\x11\x1e\x1f\n\x0c\n\x05\x05\0\x02\
    \x03\x03\x12\x03\x11\x203\n\x0f\n\x08\x05\0\x02\x03\x03\xe7\x07\0\x12\
    \x03\x11!2\n\x10\n\t\x05\0\x02\x03\x03\xe7\x07\0\x02\x12\x03\x11!+\n\x11\
    \n\n\x05\0\x02\x03\x03\xe7\x07\0\x02\0\x12\x03\x11!+\n\x12\n\x0b\x05\0\
    \x02\x03\x03\xe7\x07\0\x02\0\x01\x12\x03\x11\"*\n\x10\n\t\x05\0\x02\x03\
    \x03\xe7\x07\0\x03\x12\x03\x11.2\n\x0b\n\x04\x05\0\x02\x04\x12\x03\x12\
    \x085\n\x0c\n\x05\x05\0\x02\x04\x01\x12\x03\x12\x08\x1d\n\x0c\n\x05\x05\
    \0\x02\x04\x02\x12\x03\x12\x20!\n\x0c\n\x05\x05\0\x02\x04\x03\x12\x03\
    \x12\"4\n\x0f\n\x08\x05\0\x02\x04\x03\xe7\x07\0\x12\x03\x12#3\n\x10\n\t\
    \x05\0\x02\x04\x03\xe7\x07\0\x02\x12\x03\x12#,\n\x11\n\n\x05\0\x02\x04\
    \x03\xe7\x07\0\x02\0\x12\x03\x12#,\n\x12\n\x0b\x05\0\x02\x04\x03\xe7\x07\
    \0\x02\0\x01\x12\x03\x12$+\n\x10\n\t\x05\0\x02\x04\x03\xe7\x07\0\x03\x12\
    \x03\x12/3\n\x0b\n\x04\x05\0\x02\x05\x12\x03\x13\x086\n\x0c\n\x05\x05\0\
    \x02\x05\x01\x12\x03\x13\x08\x1e\n\x0c\n\x05\x05\0\x02\x05\x02\x12\x03\
    \x13!\"\n\x0c\n\x05\x05\0\x02\x05\x03\x12\x03\x13#5\n\x0f\n\x08\x05\0\
    \x02\x05\x03\xe7\x07\0\x12\x03\x13$4\n\x10\n\t\x05\0\x02\x05\x03\xe7\x07\
    \0\x02\x12\x03\x13$-\n\x11\n\n\x05\0\x02\x05\x03\xe7\x07\0\x02\0\x12\x03\
    \x13$-\n\x12\n\x0b\x05\0\x02\x05\x03\xe7\x07\0\x02\0\x01\x12\x03\x13%,\n\
    \x10\n\t\x05\0\x02\x05\x03\xe7\x07\0\x03\x12\x03\x1304\n\x0b\n\x04\x05\0\
    \x02\x06\x12\x03\x14\x08S\n\x0c\n\x05\x05\0\x02\x06\x01\x12\x03\x14\x08!\
    \n\x0c\n\x05\x05\0\x02\x06\x02\x12\x03\x14$%\n\x0c\n\x05\x05\0\x02\x06\
    \x03\x12\x03\x14&R\n\x0f\n\x08\x05\0\x02\x06\x03\xe7\x07\0\x12\x03\x14'7\
    \n\x10\n\t\x05\0\x02\x06\x03\xe7\x07\0\x02\x12\x03\x14'0\n\x11\n\n\x05\0\
    \x02\x06\x03\xe7\x07\0\x02\0\x12\x03\x14'0\n\x12\n\x0b\x05\0\x02\x06\x03\
    \xe7\x07\0\x02\0\x01\x12\x03\x14(/\n\x10\n\t\x05\0\x02\x06\x03\xe7\x07\0\
    \x03\x12\x03\x1437\n\x0f\n\x08\x05\0\x02\x06\x03\xe7\x07\x01\x12\x03\x14\
    9Q\n\x10\n\t\x05\0\x02\x06\x03\xe7\x07\x01\x02\x12\x03\x149J\n\x11\n\n\
    \x05\0\x02\x06\x03\xe7\x07\x01\x02\0\x12\x03\x149J\n\x12\n\x0b\x05\0\x02\
    \x06\x03\xe7\x07\x01\x02\0\x01\x12\x03\x14:I\n\x10\n\t\x05\0\x02\x06\x03\
    \xe7\x07\x01\x03\x12\x03\x14MQ\n\x0b\n\x04\x05\0\x02\x07\x12\x03\x15\x08\
    T\n\x0c\n\x05\x05\0\x02\x07\x01\x12\x03\x15\x08\"\n\x0c\n\x05\x05\0\x02\
    \x07\x02\x12\x03\x15%&\n\x0c\n\x05\x05\0\x02\x07\x03\x12\x03\x15'S\n\x0f\
    \n\x08\x05\0\x02\x07\x03\xe7\x07\0\x12\x03\x15(8\n\x10\n\t\x05\0\x02\x07\
    \x03\xe7\x07\0\x02\x12\x03\x15(1\n\x11\n\n\x05\0\x02\x07\x03\xe7\x07\0\
    \x02\0\x12\x03\x15(1\n\x12\n\x0b\x05\0\x02\x07\x03\xe7\x07\0\x02\0\x01\
    \x12\x03\x15)0\n\x10\n\t\x05\0\x02\x07\x03\xe7\x07\0\x03\x12\x03\x1548\n\
    \x0f\n\x08\x05\0\x02\x07\x03\xe7\x07\x01\x12\x03\x15:R\n\x10\n\t\x05\0\
    \x02\x07\x03\xe7\x07\x01\x02\x12\x03\x15:K\n\x11\n\n\x05\0\x02\x07\x03\
    \xe7\x07\x01\x02\0\x12\x03\x15:K\n\x12\n\x0b\x05\0\x02\x07\x03\xe7\x07\
    \x01\x02\0\x01\x12\x03\x15;J\n\x10\n\t\x05\0\x02\x07\x03\xe7\x07\x01\x03\
    \x12\x03\x15NR\n\x0b\n\x04\x05\0\x02\x08\x12\x03\x16\x08V\n\x0c\n\x05\
    \x05\0\x02\x08\x01\x12\x03\x16\x08#\n\x0c\n\x05\x05\0\x02\x08\x02\x12\
    \x03\x16&'\n\x0c\n\x05\x05\0\x02\x08\x03\x12\x03\x16(U\n\x0f\n\x08\x05\0\
    \x02\x08\x03\xe7\x07\0\x12\x03\x16):\n\x10\n\t\x05\0\x02\x08\x03\xe7\x07\
    \0\x02\x12\x03\x16)3\n\x11\n\n\x05\0\x02\x08\x03\xe7\x07\0\x02\0\x12\x03\
    \x16)3\n\x12\n\x0b\x05\0\x02\x08\x03\xe7\x07\0\x02\0\x01\x12\x03\x16*2\n\
    \x10\n\t\x05\0\x02\x08\x03\xe7\x07\0\x03\x12\x03\x166:\n\x0f\n\x08\x05\0\
    \x02\x08\x03\xe7\x07\x01\x12\x03\x16<T\n\x10\n\t\x05\0\x02\x08\x03\xe7\
    \x07\x01\x02\x12\x03\x16<M\n\x11\n\n\x05\0\x02\x08\x03\xe7\x07\x01\x02\0\
    \x12\x03\x16<M\n\x12\n\x0b\x05\0\x02\x08\x03\xe7\x07\x01\x02\0\x01\x12\
    \x03\x16=L\n\x10\n\t\x05\0\x02\x08\x03\xe7\x07\x01\x03\x12\x03\x16PT\n\
    \x0b\n\x04\x05\0\x02\t\x12\x03\x17\x086\n\x0c\n\x05\x05\0\x02\t\x01\x12\
    \x03\x17\x08\x1e\n\x0c\n\x05\x05\0\x02\t\x02\x12\x03\x17!\"\n\x0c\n\x05\
    \x05\0\x02\t\x03\x12\x03\x17#5\n\x0f\n\x08\x05\0\x02\t\x03\xe7\x07\0\x12\
    \x03\x17$4\n\x10\n\t\x05\0\x02\t\x03\xe7\x07\0\x02\x12\x03\x17$-\n\x11\n\
    \n\x05\0\x02\t\x03\xe7\x07\0\x02\0\x12\x03\x17$-\n\x12\n\x0b\x05\0\x02\t\
    \x03\xe7\x07\0\x02\0\x01\x12\x03\x17%,\n\x10\n\t\x05\0\x02\t\x03\xe7\x07\
    \0\x03\x12\x03\x1704\n\x0b\n\x04\x05\0\x02\n\x12\x03\x18\x085\n\x0c\n\
    \x05\x05\0\x02\n\x01\x12\x03\x18\x08\x1b\n\x0c\n\x05\x05\0\x02\n\x02\x12\
    \x03\x18\x1e\x20\n\x0c\n\x05\x05\0\x02\n\x03\x12\x03\x18!4\n\x0f\n\x08\
    \x05\0\x02\n\x03\xe7\x07\0\x12\x03\x18\"3\n\x10\n\t\x05\0\x02\n\x03\xe7\
    \x07\0\x02\x12\x03\x18\",\n\x11\n\n\x05\0\x02\n\x03\xe7\x07\0\x02\0\x12\
    \x03\x18\",\n\x12\n\x0b\x05\0\x02\n\x03\xe7\x07\0\x02\0\x01\x12\x03\x18#\
    +\n\x10\n\t\x05\0\x02\n\x03\xe7\x07\0\x03\x12\x03\x18/3\n\x0b\n\x04\x05\
    \0\x02\x0b\x12\x03\x19\x089\n\x0c\n\x05\x05\0\x02\x0b\x01\x12\x03\x19\
    \x08\x20\n\x0c\n\x05\x05\0\x02\x0b\x02\x12\x03\x19#%\n\x0c\n\x05\x05\0\
    \x02\x0b\x03\x12\x03\x19&8\n\x0f\n\x08\x05\0\x02\x0b\x03\xe7\x07\0\x12\
    \x03\x19'7\n\x10\n\t\x05\0\x02\x0b\x03\xe7\x07\0\x02\x12\x03\x19'0\n\x11\
    \n\n\x05\0\x02\x0b\x03\xe7\x07\0\x02\0\x12\x03\x19'0\n\x12\n\x0b\x05\0\
    \x02\x0b\x03\xe7\x07\0\x02\0\x01\x12\x03\x19(/\n\x10\n\t\x05\0\x02\x0b\
    \x03\xe7\x07\0\x03\x12\x03\x1937\n\x0b\n\x04\x05\0\x02\x0c\x12\x03\x1a\
    \x087\n\x0c\n\x05\x05\0\x02\x0c\x01\x12\x03\x1a\x08\x1d\n\x0c\n\x05\x05\
    \0\x02\x0c\x02\x12\x03\x1a\x20\"\n\x0c\n\x05\x05\0\x02\x0c\x03\x12\x03\
    \x1a#6\n\x0f\n\x08\x05\0\x02\x0c\x03\xe7\x07\0\x12\x03\x1a$5\n\x10\n\t\
    \x05\0\x02\x0c\x03\xe7\x07\0\x02\x12\x03\x1a$.\n\x11\n\n\x05\0\x02\x0c\
    \x03\xe7\x07\0\x02\0\x12\x03\x1a$.\n\x12\n\x0b\x05\0\x02\x0c\x03\xe7\x07\
    \0\x02\0\x01\x12\x03\x1a%-\n\x10\n\t\x05\0\x02\x0c\x03\xe7\x07\0\x03\x12\
    \x03\x1a15\n\x0b\n\x04\x05\0\x02\r\x12\x03\x1b\x087\n\x0c\n\x05\x05\0\
    \x02\r\x01\x12\x03\x1b\x08\x1e\n\x0c\n\x05\x05\0\x02\r\x02\x12\x03\x1b!#\
    \n\x0c\n\x05\x05\0\x02\r\x03\x12\x03\x1b$6\n\x0f\n\x08\x05\0\x02\r\x03\
    \xe7\x07\0\x12\x03\x1b%5\n\x10\n\t\x05\0\x02\r\x03\xe7\x07\0\x02\x12\x03\
    \x1b%.\n\x11\n\n\x05\0\x02\r\x03\xe7\x07\0\x02\0\x12\x03\x1b%.\n\x12\n\
    \x0b\x05\0\x02\r\x03\xe7\x07\0\x02\0\x01\x12\x03\x1b&-\n\x10\n\t\x05\0\
    \x02\r\x03\xe7\x07\0\x03\x12\x03\x1b15\n\x0b\n\x04\x05\0\x02\x0e\x12\x03\
    \x1c\x088\n\x0c\n\x05\x05\0\x02\x0e\x01\x12\x03\x1c\x08\x1f\n\x0c\n\x05\
    \x05\0\x02\x0e\x02\x12\x03\x1c\"$\n\x0c\n\x05\x05\0\x02\x0e\x03\x12\x03\
    \x1c%7\n\x0f\n\x08\x05\0\x02\x0e\x03\xe7\x07\0\x12\x03\x1c&6\n\x10\n\t\
    \x05\0\x02\x0e\x03\xe7\x07\0\x02\x12\x03\x1c&/\n\x11\n\n\x05\0\x02\x0e\
    \x03\xe7\x07\0\x02\0\x12\x03\x1c&/\n\x12\n\x0b\x05\0\x02\x0e\x03\xe7\x07\
    \0\x02\0\x01\x12\x03\x1c'.\n\x10\n\t\x05\0\x02\x0e\x03\xe7\x07\0\x03\x12\
    \x03\x1c26\n\x0b\n\x04\x05\0\x02\x0f\x12\x03\x1d\x083\n\x0c\n\x05\x05\0\
    \x02\x0f\x01\x12\x03\x1d\x08\x1a\n\x0c\n\x05\x05\0\x02\x0f\x02\x12\x03\
    \x1d\x1d\x1f\n\x0c\n\x05\x05\0\x02\x0f\x03\x12\x03\x1d\x202\n\x0f\n\x08\
    \x05\0\x02\x0f\x03\xe7\x07\0\x12\x03\x1d!1\n\x10\n\t\x05\0\x02\x0f\x03\
    \xe7\x07\0\x02\x12\x03\x1d!*\n\x11\n\n\x05\0\x02\x0f\x03\xe7\x07\0\x02\0\
    \x12\x03\x1d!*\n\x12\n\x0b\x05\0\x02\x0f\x03\xe7\x07\0\x02\0\x01\x12\x03\
    \x1d\")\n\x10\n\t\x05\0\x02\x0f\x03\xe7\x07\0\x03\x12\x03\x1d-1\n\x0b\n\
    \x04\x05\0\x02\x10\x12\x03\x1e\x08L\n\x0c\n\x05\x05\0\x02\x10\x01\x12\
    \x03\x1e\x08\x20\n\x0c\n\x05\x05\0\x02\x10\x02\x12\x03\x1e#%\n\x0c\n\x05\
    \x05\0\x02\x10\x03\x12\x03\x1e&K\n\x0f\n\x08\x05\0\x02\x10\x03\xe7\x07\0\
    \x12\x03\x1e'7\n\x10\n\t\x05\0\x02\x10\x03\xe7\x07\0\x02\x12\x03\x1e'0\n\
    \x11\n\n\x05\0\x02\x10\x03\xe7\x07\0\x02\0\x12\x03\x1e'0\n\x12\n\x0b\x05\
    \0\x02\x10\x03\xe7\x07\0\x02\0\x01\x12\x03\x1e(/\n\x10\n\t\x05\0\x02\x10\
    \x03\xe7\x07\0\x03\x12\x03\x1e37\n\x0f\n\x08\x05\0\x02\x10\x03\xe7\x07\
    \x01\x12\x03\x1e9J\n\x10\n\t\x05\0\x02\x10\x03\xe7\x07\x01\x02\x12\x03\
    \x1e9C\n\x11\n\n\x05\0\x02\x10\x03\xe7\x07\x01\x02\0\x12\x03\x1e9C\n\x12\
    \n\x0b\x05\0\x02\x10\x03\xe7\x07\x01\x02\0\x01\x12\x03\x1e9C\n\x10\n\t\
    \x05\0\x02\x10\x03\xe7\x07\x01\x03\x12\x03\x1eFJ\n\x0b\n\x04\x05\0\x02\
    \x11\x12\x03\x1f\x086\n\x0c\n\x05\x05\0\x02\x11\x01\x12\x03\x1f\x08\x1c\
    \n\x0c\n\x05\x05\0\x02\x11\x02\x12\x03\x1f\x1f!\n\x0c\n\x05\x05\0\x02\
    \x11\x03\x12\x03\x1f\"5\n\x0f\n\x08\x05\0\x02\x11\x03\xe7\x07\0\x12\x03\
    \x1f#4\n\x10\n\t\x05\0\x02\x11\x03\xe7\x07\0\x02\x12\x03\x1f#-\n\x11\n\n\
    \x05\0\x02\x11\x03\xe7\x07\0\x02\0\x12\x03\x1f#-\n\x12\n\x0b\x05\0\x02\
    \x11\x03\xe7\x07\0\x02\0\x01\x12\x03\x1f$,\n\x10\n\t\x05\0\x02\x11\x03\
    \xe7\x07\0\x03\x12\x03\x1f04\n\x0b\n\x04\x05\0\x02\x12\x12\x03\x20\x08>\
    \n\x0c\n\x05\x05\0\x02\x12\x01\x12\x03\x20\x08$\n\x0c\n\x05\x05\0\x02\
    \x12\x02\x12\x03\x20')\n\x0c\n\x05\x05\0\x02\x12\x03\x12\x03\x20*=\n\x0f\
    \n\x08\x05\0\x02\x12\x03\xe7\x07\0\x12\x03\x20+<\n\x10\n\t\x05\0\x02\x12\
    \x03\xe7\x07\0\x02\x12\x03\x20+5\n\x11\n\n\x05\0\x02\x12\x03\xe7\x07\0\
    \x02\0\x12\x03\x20+5\n\x12\n\x0b\x05\0\x02\x12\x03\xe7\x07\0\x02\0\x01\
    \x12\x03\x20,4\n\x10\n\t\x05\0\x02\x12\x03\xe7\x07\0\x03\x12\x03\x208<\n\
    \x0b\n\x04\x05\0\x02\x13\x12\x03!\x08M\n\x0c\n\x05\x05\0\x02\x13\x01\x12\
    \x03!\x08\x20\n\x0c\n\x05\x05\0\x02\x13\x02\x12\x03!#%\n\x0c\n\x05\x05\0\
    \x02\x13\x03\x12\x03!&L\n\x0f\n\x08\x05\0\x02\x13\x03\xe7\x07\0\x12\x03!\
    '7\n\x10\n\t\x05\0\x02\x13\x03\xe7\x07\0\x02\x12\x03!'0\n\x11\n\n\x05\0\
    \x02\x13\x03\xe7\x07\0\x02\0\x12\x03!'0\n\x12\n\x0b\x05\0\x02\x13\x03\
    \xe7\x07\0\x02\0\x01\x12\x03!(/\n\x10\n\t\x05\0\x02\x13\x03\xe7\x07\0\
    \x03\x12\x03!37\n\x0f\n\x08\x05\0\x02\x13\x03\xe7\x07\x01\x12\x03!9K\n\
    \x10\n\t\x05\0\x02\x13\x03\xe7\x07\x01\x02\x12\x03!9D\n\x11\n\n\x05\0\
    \x02\x13\x03\xe7\x07\x01\x02\0\x12\x03!9D\n\x12\n\x0b\x05\0\x02\x13\x03\
    \xe7\x07\x01\x02\0\x01\x12\x03!:C\n\x10\n\t\x05\0\x02\x13\x03\xe7\x07\
    \x01\x03\x12\x03!GK\n\x0b\n\x04\x05\0\x02\x14\x12\x03\"\x083\n\x0c\n\x05\
    \x05\0\x02\x14\x01\x12\x03\"\x08\x1a\n\x0c\n\x05\x05\0\x02\x14\x02\x12\
    \x03\"\x1d\x1f\n\x0c\n\x05\x05\0\x02\x14\x03\x12\x03\"\x202\n\x0f\n\x08\
    \x05\0\x02\x14\x03\xe7\x07\0\x12\x03\"!1\n\x10\n\t\x05\0\x02\x14\x03\xe7\
    \x07\0\x02\x12\x03\"!*\n\x11\n\n\x05\0\x02\x14\x03\xe7\x07\0\x02\0\x12\
    \x03\"!*\n\x12\n\x0b\x05\0\x02\x14\x03\xe7\x07\0\x02\0\x01\x12\x03\"\")\
    \n\x10\n\t\x05\0\x02\x14\x03\xe7\x07\0\x03\x12\x03\"-1\n\x0b\n\x04\x05\0\
    \x02\x15\x12\x03#\x087\n\x0c\n\x05\x05\0\x02\x15\x01\x12\x03#\x08\x1d\n\
    \x0c\n\x05\x05\0\x02\x15\x02\x12\x03#\x20\"\n\x0c\n\x05\x05\0\x02\x15\
    \x03\x12\x03##6\n\x0f\n\x08\x05\0\x02\x15\x03\xe7\x07\0\x12\x03#$5\n\x10\
    \n\t\x05\0\x02\x15\x03\xe7\x07\0\x02\x12\x03#$.\n\x11\n\n\x05\0\x02\x15\
    \x03\xe7\x07\0\x02\0\x12\x03#$.\n\x12\n\x0b\x05\0\x02\x15\x03\xe7\x07\0\
    \x02\0\x01\x12\x03#%-\n\x10\n\t\x05\0\x02\x15\x03\xe7\x07\0\x03\x12\x03#\
    15\n\x0b\n\x04\x05\0\x02\x16\x12\x03$\x082\n\x0c\n\x05\x05\0\x02\x16\x01\
    \x12\x03$\x08\x19\n\x0c\n\x05\x05\0\x02\x16\x02\x12\x03$\x1c\x1e\n\x0c\n\
    \x05\x05\0\x02\x16\x03\x12\x03$\x1f1\n\x0f\n\x08\x05\0\x02\x16\x03\xe7\
    \x07\0\x12\x03$\x200\n\x10\n\t\x05\0\x02\x16\x03\xe7\x07\0\x02\x12\x03$\
    \x20)\n\x11\n\n\x05\0\x02\x16\x03\xe7\x07\0\x02\0\x12\x03$\x20)\n\x12\n\
    \x0b\x05\0\x02\x16\x03\xe7\x07\0\x02\0\x01\x12\x03$!(\n\x10\n\t\x05\0\
    \x02\x16\x03\xe7\x07\0\x03\x12\x03$,0\n\x0b\n\x04\x05\0\x02\x17\x12\x03%\
    \x08;\n\x0c\n\x05\x05\0\x02\x17\x01\x12\x03%\x08\"\n\x0c\n\x05\x05\0\x02\
    \x17\x02\x12\x03%%'\n\x0c\n\x05\x05\0\x02\x17\x03\x12\x03%(:\n\x0f\n\x08\
    \x05\0\x02\x17\x03\xe7\x07\0\x12\x03%)9\n\x10\n\t\x05\0\x02\x17\x03\xe7\
    \x07\0\x02\x12\x03%)2\n\x11\n\n\x05\0\x02\x17\x03\xe7\x07\0\x02\0\x12\
    \x03%)2\n\x12\n\x0b\x05\0\x02\x17\x03\xe7\x07\0\x02\0\x01\x12\x03%*1\n\
    \x10\n\t\x05\0\x02\x17\x03\xe7\x07\0\x03\x12\x03%59\n\x0b\n\x04\x05\0\
    \x02\x18\x12\x03&\x089\n\x0c\n\x05\x05\0\x02\x18\x01\x12\x03&\x08\x20\n\
    \x0c\n\x05\x05\0\x02\x18\x02\x12\x03&#%\n\x0c\n\x05\x05\0\x02\x18\x03\
    \x12\x03&&8\n\x0f\n\x08\x05\0\x02\x18\x03\xe7\x07\0\x12\x03&'7\n\x10\n\t\
    \x05\0\x02\x18\x03\xe7\x07\0\x02\x12\x03&'0\n\x11\n\n\x05\0\x02\x18\x03\
    \xe7\x07\0\x02\0\x12\x03&'0\n\x12\n\x0b\x05\0\x02\x18\x03\xe7\x07\0\x02\
    \0\x01\x12\x03&(/\n\x10\n\t\x05\0\x02\x18\x03\xe7\x07\0\x03\x12\x03&37\n\
    \x0b\n\x04\x05\0\x02\x19\x12\x03'\x08:\n\x0c\n\x05\x05\0\x02\x19\x01\x12\
    \x03'\x08!\n\x0c\n\x05\x05\0\x02\x19\x02\x12\x03'$&\n\x0c\n\x05\x05\0\
    \x02\x19\x03\x12\x03''9\n\x0f\n\x08\x05\0\x02\x19\x03\xe7\x07\0\x12\x03'\
    (8\n\x10\n\t\x05\0\x02\x19\x03\xe7\x07\0\x02\x12\x03'(1\n\x11\n\n\x05\0\
    \x02\x19\x03\xe7\x07\0\x02\0\x12\x03'(1\n\x12\n\x0b\x05\0\x02\x19\x03\
    \xe7\x07\0\x02\0\x01\x12\x03')0\n\x10\n\t\x05\0\x02\x19\x03\xe7\x07\0\
    \x03\x12\x03'48\n\x0b\n\x04\x05\0\x02\x1a\x12\x03(\x08;\n\x0c\n\x05\x05\
    \0\x02\x1a\x01\x12\x03(\x08!\n\x0c\n\x05\x05\0\x02\x1a\x02\x12\x03($&\n\
    \x0c\n\x05\x05\0\x02\x1a\x03\x12\x03(':\n\x0f\n\x08\x05\0\x02\x1a\x03\
    \xe7\x07\0\x12\x03((9\n\x10\n\t\x05\0\x02\x1a\x03\xe7\x07\0\x02\x12\x03(\
    (2\n\x11\n\n\x05\0\x02\x1a\x03\xe7\x07\0\x02\0\x12\x03((2\n\x12\n\x0b\
    \x05\0\x02\x1a\x03\xe7\x07\0\x02\0\x01\x12\x03()1\n\x10\n\t\x05\0\x02\
    \x1a\x03\xe7\x07\0\x03\x12\x03(59\n\x0b\n\x04\x05\0\x02\x1b\x12\x03)\x08\
    J\n\x0c\n\x05\x05\0\x02\x1b\x01\x12\x03)\x08\x1d\n\x0c\n\x05\x05\0\x02\
    \x1b\x02\x12\x03)\x20\"\n\x0c\n\x05\x05\0\x02\x1b\x03\x12\x03)#I\n\x0f\n\
    \x08\x05\0\x02\x1b\x03\xe7\x07\0\x12\x03)$4\n\x10\n\t\x05\0\x02\x1b\x03\
    \xe7\x07\0\x02\x12\x03)$-\n\x11\n\n\x05\0\x02\x1b\x03\xe7\x07\0\x02\0\
    \x12\x03)$-\n\x12\n\x0b\x05\0\x02\x1b\x03\xe7\x07\0\x02\0\x01\x12\x03)%,\
    \n\x10\n\t\x05\0\x02\x1b\x03\xe7\x07\0\x03\x12\x03)04\n\x0f\n\x08\x05\0\
    \x02\x1b\x03\xe7\x07\x01\x12\x03)6H\n\x10\n\t\x05\0\x02\x1b\x03\xe7\x07\
    \x01\x02\x12\x03)6A\n\x11\n\n\x05\0\x02\x1b\x03\xe7\x07\x01\x02\0\x12\
    \x03)6A\n\x12\n\x0b\x05\0\x02\x1b\x03\xe7\x07\x01\x02\0\x01\x12\x03)7@\n\
    \x10\n\t\x05\0\x02\x1b\x03\xe7\x07\x01\x03\x12\x03)DH\n\x0b\n\x04\x05\0\
    \x02\x1c\x12\x03*\x087\n\x0c\n\x05\x05\0\x02\x1c\x01\x12\x03*\x08\x1e\n\
    \x0c\n\x05\x05\0\x02\x1c\x02\x12\x03*!#\n\x0c\n\x05\x05\0\x02\x1c\x03\
    \x12\x03*$6\n\x0f\n\x08\x05\0\x02\x1c\x03\xe7\x07\0\x12\x03*%5\n\x10\n\t\
    \x05\0\x02\x1c\x03\xe7\x07\0\x02\x12\x03*%.\n\x11\n\n\x05\0\x02\x1c\x03\
    \xe7\x07\0\x02\0\x12\x03*%.\n\x12\n\x0b\x05\0\x02\x1c\x03\xe7\x07\0\x02\
    \0\x01\x12\x03*&-\n\x10\n\t\x05\0\x02\x1c\x03\xe7\x07\0\x03\x12\x03*15\n\
    \x0b\n\x04\x05\0\x02\x1d\x12\x03+\x087\n\x0c\n\x05\x05\0\x02\x1d\x01\x12\
    \x03+\x08\x1e\n\x0c\n\x05\x05\0\x02\x1d\x02\x12\x03+!#\n\x0c\n\x05\x05\0\
    \x02\x1d\x03\x12\x03+$6\n\x0f\n\x08\x05\0\x02\x1d\x03\xe7\x07\0\x12\x03+\
    %5\n\x10\n\t\x05\0\x02\x1d\x03\xe7\x07\0\x02\x12\x03+%.\n\x11\n\n\x05\0\
    \x02\x1d\x03\xe7\x07\0\x02\0\x12\x03+%.\n\x12\n\x0b\x05\0\x02\x1d\x03\
    \xe7\x07\0\x02\0\x01\x12\x03+&-\n\x10\n\t\x05\0\x02\x1d\x03\xe7\x07\0\
    \x03\x12\x03+15\n\x0b\n\x04\x05\0\x02\x1e\x12\x03,\x085\n\x0c\n\x05\x05\
    \0\x02\x1e\x01\x12\x03,\x08\x1b\n\x0c\n\x05\x05\0\x02\x1e\x02\x12\x03,\
    \x1e\x20\n\x0c\n\x05\x05\0\x02\x1e\x03\x12\x03,!4\n\x0f\n\x08\x05\0\x02\
    \x1e\x03\xe7\x07\0\x12\x03,\"3\n\x10\n\t\x05\0\x02\x1e\x03\xe7\x07\0\x02\
    \x12\x03,\",\n\x11\n\n\x05\0\x02\x1e\x03\xe7\x07\0\x02\0\x12\x03,\",\n\
    \x12\n\x0b\x05\0\x02\x1e\x03\xe7\x07\0\x02\0\x01\x12\x03,#+\n\x10\n\t\
    \x05\0\x02\x1e\x03\xe7\x07\0\x03\x12\x03,/3\n\x0b\n\x04\x05\0\x02\x1f\
    \x12\x03-\x08O\n\x0c\n\x05\x05\0\x02\x1f\x01\x12\x03-\x08\x1c\n\x0c\n\
    \x05\x05\0\x02\x1f\x02\x12\x03-\x1f!\n\x0c\n\x05\x05\0\x02\x1f\x03\x12\
    \x03-\"N\n\x0f\n\x08\x05\0\x02\x1f\x03\xe7\x07\0\x12\x03-#3\n\x10\n\t\
    \x05\0\x02\x1f\x03\xe7\x07\0\x02\x12\x03-#,\n\x11\n\n\x05\0\x02\x1f\x03\
    \xe7\x07\0\x02\0\x12\x03-#,\n\x12\n\x0b\x05\0\x02\x1f\x03\xe7\x07\0\x02\
    \0\x01\x12\x03-$+\n\x10\n\t\x05\0\x02\x1f\x03\xe7\x07\0\x03\x12\x03-/3\n\
    \x0f\n\x08\x05\0\x02\x1f\x03\xe7\x07\x01\x12\x03-5M\n\x10\n\t\x05\0\x02\
    \x1f\x03\xe7\x07\x01\x02\x12\x03-5F\n\x11\n\n\x05\0\x02\x1f\x03\xe7\x07\
    \x01\x02\0\x12\x03-5F\n\x12\n\x0b\x05\0\x02\x1f\x03\xe7\x07\x01\x02\0\
    \x01\x12\x03-6E\n\x10\n\t\x05\0\x02\x1f\x03\xe7\x07\x01\x03\x12\x03-IM\n\
    \x0b\n\x04\x05\0\x02\x20\x12\x03.\x089\n\x0c\n\x05\x05\0\x02\x20\x01\x12\
    \x03.\x08\x20\n\x0c\n\x05\x05\0\x02\x20\x02\x12\x03.#%\n\x0c\n\x05\x05\0\
    \x02\x20\x03\x12\x03.&8\n\x0f\n\x08\x05\0\x02\x20\x03\xe7\x07\0\x12\x03.\
    '7\n\x10\n\t\x05\0\x02\x20\x03\xe7\x07\0\x02\x12\x03.'0\n\x11\n\n\x05\0\
    \x02\x20\x03\xe7\x07\0\x02\0\x12\x03.'0\n\x12\n\x0b\x05\0\x02\x20\x03\
    \xe7\x07\0\x02\0\x01\x12\x03.(/\n\x10\n\t\x05\0\x02\x20\x03\xe7\x07\0\
    \x03\x12\x03.37\n\x0b\n\x04\x05\0\x02!\x12\x03/\x08<\n\x0c\n\x05\x05\0\
    \x02!\x01\x12\x03/\x08\"\n\x0c\n\x05\x05\0\x02!\x02\x12\x03/%'\n\x0c\n\
    \x05\x05\0\x02!\x03\x12\x03/(;\n\x0f\n\x08\x05\0\x02!\x03\xe7\x07\0\x12\
    \x03/):\n\x10\n\t\x05\0\x02!\x03\xe7\x07\0\x02\x12\x03/)3\n\x11\n\n\x05\
    \0\x02!\x03\xe7\x07\0\x02\0\x12\x03/)3\n\x12\n\x0b\x05\0\x02!\x03\xe7\
    \x07\0\x02\0\x01\x12\x03/*2\n\x10\n\t\x05\0\x02!\x03\xe7\x07\0\x03\x12\
    \x03/6:\n\x0b\n\x04\x05\0\x02\"\x12\x030\x087\n\x0c\n\x05\x05\0\x02\"\
    \x01\x12\x030\x08\x1e\n\x0c\n\x05\x05\0\x02\"\x02\x12\x030!#\n\x0c\n\x05\
    \x05\0\x02\"\x03\x12\x030$6\n\x0f\n\x08\x05\0\x02\"\x03\xe7\x07\0\x12\
    \x030%5\n\x10\n\t\x05\0\x02\"\x03\xe7\x07\0\x02\x12\x030%.\n\x11\n\n\x05\
    \0\x02\"\x03\xe7\x07\0\x02\0\x12\x030%.\n\x12\n\x0b\x05\0\x02\"\x03\xe7\
    \x07\0\x02\0\x01\x12\x030&-\n\x10\n\t\x05\0\x02\"\x03\xe7\x07\0\x03\x12\
    \x03015\n\x0b\n\x04\x05\0\x02#\x12\x031\x088\n\x0c\n\x05\x05\0\x02#\x01\
    \x12\x031\x08\x1f\n\x0c\n\x05\x05\0\x02#\x02\x12\x031\"$\n\x0c\n\x05\x05\
    \0\x02#\x03\x12\x031%7\n\x0f\n\x08\x05\0\x02#\x03\xe7\x07\0\x12\x031&6\n\
    \x10\n\t\x05\0\x02#\x03\xe7\x07\0\x02\x12\x031&/\n\x11\n\n\x05\0\x02#\
    \x03\xe7\x07\0\x02\0\x12\x031&/\n\x12\n\x0b\x05\0\x02#\x03\xe7\x07\0\x02\
    \0\x01\x12\x031'.\n\x10\n\t\x05\0\x02#\x03\xe7\x07\0\x03\x12\x03126\n\
    \x0b\n\x04\x05\0\x02$\x12\x032\x08:\n\x0c\n\x05\x05\0\x02$\x01\x12\x032\
    \x08!\n\x0c\n\x05\x05\0\x02$\x02\x12\x032$&\n\x0c\n\x05\x05\0\x02$\x03\
    \x12\x032'9\n\x0f\n\x08\x05\0\x02$\x03\xe7\x07\0\x12\x032(8\n\x10\n\t\
    \x05\0\x02$\x03\xe7\x07\0\x02\x12\x032(1\n\x11\n\n\x05\0\x02$\x03\xe7\
    \x07\0\x02\0\x12\x032(1\n\x12\n\x0b\x05\0\x02$\x03\xe7\x07\0\x02\0\x01\
    \x12\x032)0\n\x10\n\t\x05\0\x02$\x03\xe7\x07\0\x03\x12\x03248\n\x0b\n\
    \x04\x05\0\x02%\x12\x033\x08>\n\x0c\n\x05\x05\0\x02%\x01\x12\x033\x08$\n\
    \x0c\n\x05\x05\0\x02%\x02\x12\x033')\n\x0c\n\x05\x05\0\x02%\x03\x12\x033\
    *=\n\x0f\n\x08\x05\0\x02%\x03\xe7\x07\0\x12\x033+<\n\x10\n\t\x05\0\x02%\
    \x03\xe7\x07\0\x02\x12\x033+5\n\x11\n\n\x05\0\x02%\x03\xe7\x07\0\x02\0\
    \x12\x033+5\n\x12\n\x0b\x05\0\x02%\x03\xe7\x07\0\x02\0\x01\x12\x033,4\n\
    \x10\n\t\x05\0\x02%\x03\xe7\x07\0\x03\x12\x0338<\n\x0b\n\x04\x05\0\x02&\
    \x12\x034\x08?\n\x0c\n\x05\x05\0\x02&\x01\x12\x034\x08%\n\x0c\n\x05\x05\
    \0\x02&\x02\x12\x034(*\n\x0c\n\x05\x05\0\x02&\x03\x12\x034+>\n\x0f\n\x08\
    \x05\0\x02&\x03\xe7\x07\0\x12\x034,=\n\x10\n\t\x05\0\x02&\x03\xe7\x07\0\
    \x02\x12\x034,6\n\x11\n\n\x05\0\x02&\x03\xe7\x07\0\x02\0\x12\x034,6\n\
    \x12\n\x0b\x05\0\x02&\x03\xe7\x07\0\x02\0\x01\x12\x034-5\n\x10\n\t\x05\0\
    \x02&\x03\xe7\x07\0\x03\x12\x0349=\n\x0b\n\x04\x05\0\x02'\x12\x035\x08N\
    \n\x0c\n\x05\x05\0\x02'\x01\x12\x035\x08!\n\x0c\n\x05\x05\0\x02'\x02\x12\
    \x035$&\n\x0c\n\x05\x05\0\x02'\x03\x12\x035'M\n\x0f\n\x08\x05\0\x02'\x03\
    \xe7\x07\0\x12\x035(8\n\x10\n\t\x05\0\x02'\x03\xe7\x07\0\x02\x12\x035(1\
    \n\x11\n\n\x05\0\x02'\x03\xe7\x07\0\x02\0\x12\x035(1\n\x12\n\x0b\x05\0\
    \x02'\x03\xe7\x07\0\x02\0\x01\x12\x035)0\n\x10\n\t\x05\0\x02'\x03\xe7\
    \x07\0\x03\x12\x03548\n\x0f\n\x08\x05\0\x02'\x03\xe7\x07\x01\x12\x035:L\
    \n\x10\n\t\x05\0\x02'\x03\xe7\x07\x01\x02\x12\x035:E\n\x11\n\n\x05\0\x02\
    '\x03\xe7\x07\x01\x02\0\x12\x035:E\n\x12\n\x0b\x05\0\x02'\x03\xe7\x07\
    \x01\x02\0\x01\x12\x035;D\n\x10\n\t\x05\0\x02'\x03\xe7\x07\x01\x03\x12\
    \x035HL\n\x0b\n\x04\x05\0\x02(\x12\x036\x08N\n\x0c\n\x05\x05\0\x02(\x01\
    \x12\x036\x08\"\n\x0c\n\x05\x05\0\x02(\x02\x12\x036%'\n\x0c\n\x05\x05\0\
    \x02(\x03\x12\x036(M\n\x0f\n\x08\x05\0\x02(\x03\xe7\x07\0\x12\x036)9\n\
    \x10\n\t\x05\0\x02(\x03\xe7\x07\0\x02\x12\x036)2\n\x11\n\n\x05\0\x02(\
    \x03\xe7\x07\0\x02\0\x12\x036)2\n\x12\n\x0b\x05\0\x02(\x03\xe7\x07\0\x02\
    \0\x01\x12\x036*1\n\x10\n\t\x05\0\x02(\x03\xe7\x07\0\x03\x12\x03659\n\
    \x0f\n\x08\x05\0\x02(\x03\xe7\x07\x01\x12\x036;L\n\x10\n\t\x05\0\x02(\
    \x03\xe7\x07\x01\x02\x12\x036;E\n\x11\n\n\x05\0\x02(\x03\xe7\x07\x01\x02\
    \0\x12\x036;E\n\x12\n\x0b\x05\0\x02(\x03\xe7\x07\x01\x02\0\x01\x12\x036;\
    E\n\x10\n\t\x05\0\x02(\x03\xe7\x07\x01\x03\x12\x036HL\n\x0b\n\x04\x05\0\
    \x02)\x12\x037\x08G\n\x0c\n\x05\x05\0\x02)\x01\x12\x037\x08\x1a\n\x0c\n\
    \x05\x05\0\x02)\x02\x12\x037\x1d\x1f\n\x0c\n\x05\x05\0\x02)\x03\x12\x037\
    \x20F\n\x0f\n\x08\x05\0\x02)\x03\xe7\x07\0\x12\x037!2\n\x10\n\t\x05\0\
    \x02)\x03\xe7\x07\0\x02\x12\x037!+\n\x11\n\n\x05\0\x02)\x03\xe7\x07\0\
    \x02\0\x12\x037!+\n\x12\n\x0b\x05\0\x02)\x03\xe7\x07\0\x02\0\x01\x12\x03\
    7\"*\n\x10\n\t\x05\0\x02)\x03\xe7\x07\0\x03\x12\x037.2\n\x0f\n\x08\x05\0\
    \x02)\x03\xe7\x07\x01\x12\x0374E\n\x10\n\t\x05\0\x02)\x03\xe7\x07\x01\
    \x02\x12\x0374>\n\x11\n\n\x05\0\x02)\x03\xe7\x07\x01\x02\0\x12\x0374>\n\
    \x12\n\x0b\x05\0\x02)\x03\xe7\x07\x01\x02\0\x01\x12\x0374>\n\x10\n\t\x05\
    \0\x02)\x03\xe7\x07\x01\x03\x12\x037AE\n\x0b\n\x04\x05\0\x02*\x12\x038\
    \x08;\n\x0c\n\x05\x05\0\x02*\x01\x12\x038\x08\"\n\x0c\n\x05\x05\0\x02*\
    \x02\x12\x038%'\n\x0c\n\x05\x05\0\x02*\x03\x12\x038(:\n\x0f\n\x08\x05\0\
    \x02*\x03\xe7\x07\0\x12\x038)9\n\x10\n\t\x05\0\x02*\x03\xe7\x07\0\x02\
    \x12\x038)2\n\x11\n\n\x05\0\x02*\x03\xe7\x07\0\x02\0\x12\x038)2\n\x12\n\
    \x0b\x05\0\x02*\x03\xe7\x07\0\x02\0\x01\x12\x038*1\n\x10\n\t\x05\0\x02*\
    \x03\xe7\x07\0\x03\x12\x03859\n\x0b\n\x04\x05\0\x02+\x12\x039\x089\n\x0c\
    \n\x05\x05\0\x02+\x01\x12\x039\x08\x1f\n\x0c\n\x05\x05\0\x02+\x02\x12\
    \x039\"$\n\x0c\n\x05\x05\0\x02+\x03\x12\x039%8\n\x0f\n\x08\x05\0\x02+\
    \x03\xe7\x07\0\x12\x039&7\n\x10\n\t\x05\0\x02+\x03\xe7\x07\0\x02\x12\x03\
    9&0\n\x11\n\n\x05\0\x02+\x03\xe7\x07\0\x02\0\x12\x039&0\n\x12\n\x0b\x05\
    \0\x02+\x03\xe7\x07\0\x02\0\x01\x12\x039'/\n\x10\n\t\x05\0\x02+\x03\xe7\
    \x07\0\x03\x12\x03937\n\x0b\n\x04\x05\0\x02,\x12\x03:\x084\n\x0c\n\x05\
    \x05\0\x02,\x01\x12\x03:\x08\x1b\n\x0c\n\x05\x05\0\x02,\x02\x12\x03:\x1e\
    \x20\n\x0c\n\x05\x05\0\x02,\x03\x12\x03:!3\n\x0f\n\x08\x05\0\x02,\x03\
    \xe7\x07\0\x12\x03:\"2\n\x10\n\t\x05\0\x02,\x03\xe7\x07\0\x02\x12\x03:\"\
    +\n\x11\n\n\x05\0\x02,\x03\xe7\x07\0\x02\0\x12\x03:\"+\n\x12\n\x0b\x05\0\
    \x02,\x03\xe7\x07\0\x02\0\x01\x12\x03:#*\n\x10\n\t\x05\0\x02,\x03\xe7\
    \x07\0\x03\x12\x03:.2\n\x0b\n\x04\x05\0\x02-\x12\x03;\x08>\n\x0c\n\x05\
    \x05\0\x02-\x01\x12\x03;\x08$\n\x0c\n\x05\x05\0\x02-\x02\x12\x03;')\n\
    \x0c\n\x05\x05\0\x02-\x03\x12\x03;*=\n\x0f\n\x08\x05\0\x02-\x03\xe7\x07\
    \0\x12\x03;+<\n\x10\n\t\x05\0\x02-\x03\xe7\x07\0\x02\x12\x03;+5\n\x11\n\
    \n\x05\0\x02-\x03\xe7\x07\0\x02\0\x12\x03;+5\n\x12\n\x0b\x05\0\x02-\x03\
    \xe7\x07\0\x02\0\x01\x12\x03;,4\n\x10\n\t\x05\0\x02-\x03\xe7\x07\0\x03\
    \x12\x03;8<\n\x0b\n\x04\x05\0\x02.\x12\x03<\x08N\n\x0c\n\x05\x05\0\x02.\
    \x01\x12\x03<\x08\"\n\x0c\n\x05\x05\0\x02.\x02\x12\x03<%'\n\x0c\n\x05\
    \x05\0\x02.\x03\x12\x03<(M\n\x0f\n\x08\x05\0\x02.\x03\xe7\x07\0\x12\x03<\
    )9\n\x10\n\t\x05\0\x02.\x03\xe7\x07\0\x02\x12\x03<)2\n\x11\n\n\x05\0\x02\
    .\x03\xe7\x07\0\x02\0\x12\x03<)2\n\x12\n\x0b\x05\0\x02.\x03\xe7\x07\0\
    \x02\0\x01\x12\x03<*1\n\x10\n\t\x05\0\x02.\x03\xe7\x07\0\x03\x12\x03<59\
    \n\x0f\n\x08\x05\0\x02.\x03\xe7\x07\x01\x12\x03<;L\n\x10\n\t\x05\0\x02.\
    \x03\xe7\x07\x01\x02\x12\x03<;E\n\x11\n\n\x05\0\x02.\x03\xe7\x07\x01\x02\
    \0\x12\x03<;E\n\x12\n\x0b\x05\0\x02.\x03\xe7\x07\x01\x02\0\x01\x12\x03<;\
    E\n\x10\n\t\x05\0\x02.\x03\xe7\x07\x01\x03\x12\x03<HL\n\x0b\n\x04\x05\0\
    \x02/\x12\x03=\x08Q\n\x0c\n\x05\x05\0\x02/\x01\x12\x03=\x08$\n\x0c\n\x05\
    \x05\0\x02/\x02\x12\x03=')\n\x0c\n\x05\x05\0\x02/\x03\x12\x03=*P\n\x0f\n\
    \x08\x05\0\x02/\x03\xe7\x07\0\x12\x03=+<\n\x10\n\t\x05\0\x02/\x03\xe7\
    \x07\0\x02\x12\x03=+5\n\x11\n\n\x05\0\x02/\x03\xe7\x07\0\x02\0\x12\x03=+\
    5\n\x12\n\x0b\x05\0\x02/\x03\xe7\x07\0\x02\0\x01\x12\x03=,4\n\x10\n\t\
    \x05\0\x02/\x03\xe7\x07\0\x03\x12\x03=8<\n\x0f\n\x08\x05\0\x02/\x03\xe7\
    \x07\x01\x12\x03=>O\n\x10\n\t\x05\0\x02/\x03\xe7\x07\x01\x02\x12\x03=>H\
    \n\x11\n\n\x05\0\x02/\x03\xe7\x07\x01\x02\0\x12\x03=>H\n\x12\n\x0b\x05\0\
    \x02/\x03\xe7\x07\x01\x02\0\x01\x12\x03=>H\n\x10\n\t\x05\0\x02/\x03\xe7\
    \x07\x01\x03\x12\x03=KO\n\x0b\n\x04\x05\0\x020\x12\x03>\x08N\n\x0c\n\x05\
    \x05\0\x020\x01\x12\x03>\x08\"\n\x0c\n\x05\x05\0\x020\x02\x12\x03>%'\n\
    \x0c\n\x05\x05\0\x020\x03\x12\x03>(M\n\x0f\n\x08\x05\0\x020\x03\xe7\x07\
    \0\x12\x03>)9\n\x10\n\t\x05\0\x020\x03\xe7\x07\0\x02\x12\x03>)2\n\x11\n\
    \n\x05\0\x020\x03\xe7\x07\0\x02\0\x12\x03>)2\n\x12\n\x0b\x05\0\x020\x03\
    \xe7\x07\0\x02\0\x01\x12\x03>*1\n\x10\n\t\x05\0\x020\x03\xe7\x07\0\x03\
    \x12\x03>59\n\x0f\n\x08\x05\0\x020\x03\xe7\x07\x01\x12\x03>;L\n\x10\n\t\
    \x05\0\x020\x03\xe7\x07\x01\x02\x12\x03>;E\n\x11\n\n\x05\0\x020\x03\xe7\
    \x07\x01\x02\0\x12\x03>;E\n\x12\n\x0b\x05\0\x020\x03\xe7\x07\x01\x02\0\
    \x01\x12\x03>;E\n\x10\n\t\x05\0\x020\x03\xe7\x07\x01\x03\x12\x03>HL\n\
    \x0b\n\x04\x05\0\x021\x12\x03?\x08Q\n\x0c\n\x05\x05\0\x021\x01\x12\x03?\
    \x08$\n\x0c\n\x05\x05\0\x021\x02\x12\x03?')\n\x0c\n\x05\x05\0\x021\x03\
    \x12\x03?*P\n\x0f\n\x08\x05\0\x021\x03\xe7\x07\0\x12\x03?+<\n\x10\n\t\
    \x05\0\x021\x03\xe7\x07\0\x02\x12\x03?+5\n\x11\n\n\x05\0\x021\x03\xe7\
    \x07\0\x02\0\x12\x03?+5\n\x12\n\x0b\x05\0\x021\x03\xe7\x07\0\x02\0\x01\
    \x12\x03?,4\n\x10\n\t\x05\0\x021\x03\xe7\x07\0\x03\x12\x03?8<\n\x0f\n\
    \x08\x05\0\x021\x03\xe7\x07\x01\x12\x03?>O\n\x10\n\t\x05\0\x021\x03\xe7\
    \x07\x01\x02\x12\x03?>H\n\x11\n\n\x05\0\x021\x03\xe7\x07\x01\x02\0\x12\
    \x03?>H\n\x12\n\x0b\x05\0\x021\x03\xe7\x07\x01\x02\0\x01\x12\x03?>H\n\
    \x10\n\t\x05\0\x021\x03\xe7\x07\x01\x03\x12\x03?KO\n\x0b\n\x04\x05\0\x02\
    2\x12\x03@\x089\n\x0c\n\x05\x05\0\x022\x01\x12\x03@\x08\x20\n\x0c\n\x05\
    \x05\0\x022\x02\x12\x03@#%\n\x0c\n\x05\x05\0\x022\x03\x12\x03@&8\n\x0f\n\
    \x08\x05\0\x022\x03\xe7\x07\0\x12\x03@'7\n\x10\n\t\x05\0\x022\x03\xe7\
    \x07\0\x02\x12\x03@'0\n\x11\n\n\x05\0\x022\x03\xe7\x07\0\x02\0\x12\x03@'\
    0\n\x12\n\x0b\x05\0\x022\x03\xe7\x07\0\x02\0\x01\x12\x03@(/\n\x10\n\t\
    \x05\0\x022\x03\xe7\x07\0\x03\x12\x03@37\n\x0b\n\x04\x05\0\x023\x12\x03A\
    \x08<\n\x0c\n\x05\x05\0\x023\x01\x12\x03A\x08\"\n\x0c\n\x05\x05\0\x023\
    \x02\x12\x03A%'\n\x0c\n\x05\x05\0\x023\x03\x12\x03A(;\n\x0f\n\x08\x05\0\
    \x023\x03\xe7\x07\0\x12\x03A):\n\x10\n\t\x05\0\x023\x03\xe7\x07\0\x02\
    \x12\x03A)3\n\x11\n\n\x05\0\x023\x03\xe7\x07\0\x02\0\x12\x03A)3\n\x12\n\
    \x0b\x05\0\x023\x03\xe7\x07\0\x02\0\x01\x12\x03A*2\n\x10\n\t\x05\0\x023\
    \x03\xe7\x07\0\x03\x12\x03A6:\n\x0b\n\x04\x05\0\x024\x12\x03B\x088\n\x0c\
    \n\x05\x05\0\x024\x01\x12\x03B\x08\x1f\n\x0c\n\x05\x05\0\x024\x02\x12\
    \x03B\"$\n\x0c\n\x05\x05\0\x024\x03\x12\x03B%7\n\x0f\n\x08\x05\0\x024\
    \x03\xe7\x07\0\x12\x03B&6\n\x10\n\t\x05\0\x024\x03\xe7\x07\0\x02\x12\x03\
    B&/\n\x11\n\n\x05\0\x024\x03\xe7\x07\0\x02\0\x12\x03B&/\n\x12\n\x0b\x05\
    \0\x024\x03\xe7\x07\0\x02\0\x01\x12\x03B'.\n\x10\n\t\x05\0\x024\x03\xe7\
    \x07\0\x03\x12\x03B26\n\x0b\n\x04\x05\0\x025\x12\x03C\x08?\n\x0c\n\x05\
    \x05\0\x025\x01\x12\x03C\x08&\n\x0c\n\x05\x05\0\x025\x02\x12\x03C)+\n\
    \x0c\n\x05\x05\0\x025\x03\x12\x03C,>\n\x0f\n\x08\x05\0\x025\x03\xe7\x07\
    \0\x12\x03C-=\n\x10\n\t\x05\0\x025\x03\xe7\x07\0\x02\x12\x03C-6\n\x11\n\
    \n\x05\0\x025\x03\xe7\x07\0\x02\0\x12\x03C-6\n\x12\n\x0b\x05\0\x025\x03\
    \xe7\x07\0\x02\0\x01\x12\x03C.5\n\x10\n\t\x05\0\x025\x03\xe7\x07\0\x03\
    \x12\x03C9=\n\x0b\n\x04\x05\0\x026\x12\x03D\x08=\n\x0c\n\x05\x05\0\x026\
    \x01\x12\x03D\x08#\n\x0c\n\x05\x05\0\x026\x02\x12\x03D&(\n\x0c\n\x05\x05\
    \0\x026\x03\x12\x03D)<\n\x0f\n\x08\x05\0\x026\x03\xe7\x07\0\x12\x03D*;\n\
    \x10\n\t\x05\0\x026\x03\xe7\x07\0\x02\x12\x03D*4\n\x11\n\n\x05\0\x026\
    \x03\xe7\x07\0\x02\0\x12\x03D*4\n\x12\n\x0b\x05\0\x026\x03\xe7\x07\0\x02\
    \0\x01\x12\x03D+3\n\x10\n\t\x05\0\x026\x03\xe7\x07\0\x03\x12\x03D7;\n\
    \x0b\n\x04\x05\0\x027\x12\x03E\x08;\n\x0c\n\x05\x05\0\x027\x01\x12\x03E\
    \x08\"\n\x0c\n\x05\x05\0\x027\x02\x12\x03E%'\n\x0c\n\x05\x05\0\x027\x03\
    \x12\x03E(:\n\x0f\n\x08\x05\0\x027\x03\xe7\x07\0\x12\x03E)9\n\x10\n\t\
    \x05\0\x027\x03\xe7\x07\0\x02\x12\x03E)2\n\x11\n\n\x05\0\x027\x03\xe7\
    \x07\0\x02\0\x12\x03E)2\n\x12\n\x0b\x05\0\x027\x03\xe7\x07\0\x02\0\x01\
    \x12\x03E*1\n\x10\n\t\x05\0\x027\x03\xe7\x07\0\x03\x12\x03E59\n\x0b\n\
    \x04\x05\0\x028\x12\x03F\x08?\n\x0c\n\x05\x05\0\x028\x01\x12\x03F\x08%\n\
    \x0c\n\x05\x05\0\x028\x02\x12\x03F(*\n\x0c\n\x05\x05\0\x028\x03\x12\x03F\
    +>\n\x0f\n\x08\x05\0\x028\x03\xe7\x07\0\x12\x03F,=\n\x10\n\t\x05\0\x028\
    \x03\xe7\x07\0\x02\x12\x03F,6\n\x11\n\n\x05\0\x028\x03\xe7\x07\0\x02\0\
    \x12\x03F,6\n\x12\n\x0b\x05\0\x028\x03\xe7\x07\0\x02\0\x01\x12\x03F-5\n\
    \x10\n\t\x05\0\x028\x03\xe7\x07\0\x03\x12\x03F9=\n\x0b\n\x04\x05\0\x029\
    \x12\x03G\x08:\n\x0c\n\x05\x05\0\x029\x01\x12\x03G\x08!\n\x0c\n\x05\x05\
    \0\x029\x02\x12\x03G$&\n\x0c\n\x05\x05\0\x029\x03\x12\x03G'9\n\x0f\n\x08\
    \x05\0\x029\x03\xe7\x07\0\x12\x03G(8\n\x10\n\t\x05\0\x029\x03\xe7\x07\0\
    \x02\x12\x03G(1\n\x11\n\n\x05\0\x029\x03\xe7\x07\0\x02\0\x12\x03G(1\n\
    \x12\n\x0b\x05\0\x029\x03\xe7\x07\0\x02\0\x01\x12\x03G)0\n\x10\n\t\x05\0\
    \x029\x03\xe7\x07\0\x03\x12\x03G48\n\x0b\n\x04\x05\0\x02:\x12\x03H\x08>\
    \n\x0c\n\x05\x05\0\x02:\x01\x12\x03H\x08%\n\x0c\n\x05\x05\0\x02:\x02\x12\
    \x03H(*\n\x0c\n\x05\x05\0\x02:\x03\x12\x03H+=\n\x0f\n\x08\x05\0\x02:\x03\
    \xe7\x07\0\x12\x03H,<\n\x10\n\t\x05\0\x02:\x03\xe7\x07\0\x02\x12\x03H,5\
    \n\x11\n\n\x05\0\x02:\x03\xe7\x07\0\x02\0\x12\x03H,5\n\x12\n\x0b\x05\0\
    \x02:\x03\xe7\x07\0\x02\0\x01\x12\x03H-4\n\x10\n\t\x05\0\x02:\x03\xe7\
    \x07\0\x03\x12\x03H8<\n\x0b\n\x04\x05\0\x02;\x12\x03I\x08<\n\x0c\n\x05\
    \x05\0\x02;\x01\x12\x03I\x08\"\n\x0c\n\x05\x05\0\x02;\x02\x12\x03I%'\n\
    \x0c\n\x05\x05\0\x02;\x03\x12\x03I(;\n\x0f\n\x08\x05\0\x02;\x03\xe7\x07\
    \0\x12\x03I):\n\x10\n\t\x05\0\x02;\x03\xe7\x07\0\x02\x12\x03I)3\n\x11\n\
    \n\x05\0\x02;\x03\xe7\x07\0\x02\0\x12\x03I)3\n\x12\n\x0b\x05\0\x02;\x03\
    \xe7\x07\0\x02\0\x01\x12\x03I*2\n\x10\n\t\x05\0\x02;\x03\xe7\x07\0\x03\
    \x12\x03I6:\n\x0b\n\x04\x05\0\x02<\x12\x03J\x08:\n\x0c\n\x05\x05\0\x02<\
    \x01\x12\x03J\x08!\n\x0c\n\x05\x05\0\x02<\x02\x12\x03J$&\n\x0c\n\x05\x05\
    \0\x02<\x03\x12\x03J'9\n\x0f\n\x08\x05\0\x02<\x03\xe7\x07\0\x12\x03J(8\n\
    \x10\n\t\x05\0\x02<\x03\xe7\x07\0\x02\x12\x03J(1\n\x11\n\n\x05\0\x02<\
    \x03\xe7\x07\0\x02\0\x12\x03J(1\n\x12\n\x0b\x05\0\x02<\x03\xe7\x07\0\x02\
    \0\x01\x12\x03J)0\n\x10\n\t\x05\0\x02<\x03\xe7\x07\0\x03\x12\x03J48\n\
    \x0b\n\x04\x05\0\x02=\x12\x03K\x08@\n\x0c\n\x05\x05\0\x02=\x01\x12\x03K\
    \x08'\n\x0c\n\x05\x05\0\x02=\x02\x12\x03K*,\n\x0c\n\x05\x05\0\x02=\x03\
    \x12\x03K-?\n\x0f\n\x08\x05\0\x02=\x03\xe7\x07\0\x12\x03K.>\n\x10\n\t\
    \x05\0\x02=\x03\xe7\x07\0\x02\x12\x03K.7\n\x11\n\n\x05\0\x02=\x03\xe7\
    \x07\0\x02\0\x12\x03K.7\n\x12\n\x0b\x05\0\x02=\x03\xe7\x07\0\x02\0\x01\
    \x12\x03K/6\n\x10\n\t\x05\0\x02=\x03\xe7\x07\0\x03\x12\x03K:>\n\x0b\n\
    \x04\x05\0\x02>\x12\x03L\x08B\n\x0c\n\x05\x05\0\x02>\x01\x12\x03L\x08)\n\
    \x0c\n\x05\x05\0\x02>\x02\x12\x03L,.\n\x0c\n\x05\x05\0\x02>\x03\x12\x03L\
    /A\n\x0f\n\x08\x05\0\x02>\x03\xe7\x07\0\x12\x03L0@\n\x10\n\t\x05\0\x02>\
    \x03\xe7\x07\0\x02\x12\x03L09\n\x11\n\n\x05\0\x02>\x03\xe7\x07\0\x02\0\
    \x12\x03L09\n\x12\n\x0b\x05\0\x02>\x03\xe7\x07\0\x02\0\x01\x12\x03L18\n\
    \x10\n\t\x05\0\x02>\x03\xe7\x07\0\x03\x12\x03L<@\n\x0b\n\x04\x05\0\x02?\
    \x12\x03M\x08F\n\x0c\n\x05\x05\0\x02?\x01\x12\x03M\x08,\n\x0c\n\x05\x05\
    \0\x02?\x02\x12\x03M/1\n\x0c\n\x05\x05\0\x02?\x03\x12\x03M2E\n\x0f\n\x08\
    \x05\0\x02?\x03\xe7\x07\0\x12\x03M3D\n\x10\n\t\x05\0\x02?\x03\xe7\x07\0\
    \x02\x12\x03M3=\n\x11\n\n\x05\0\x02?\x03\xe7\x07\0\x02\0\x12\x03M3=\n\
    \x12\n\x0b\x05\0\x02?\x03\xe7\x07\0\x02\0\x01\x12\x03M4<\n\x10\n\t\x05\0\
    \x02?\x03\xe7\x07\0\x03\x12\x03M@D\n\x0b\n\x04\x05\0\x02@\x12\x03N\x08Y\
    \n\x0c\n\x05\x05\0\x02@\x01\x12\x03N\x08%\n\x0c\n\x05\x05\0\x02@\x02\x12\
    \x03N(+\n\x0c\n\x05\x05\0\x02@\x03\x12\x03N,X\n\x0f\n\x08\x05\0\x02@\x03\
    \xe7\x07\0\x12\x03N-C\n\x10\n\t\x05\0\x02@\x03\xe7\x07\0\x02\x12\x03N-<\
    \n\x11\n\n\x05\0\x02@\x03\xe7\x07\0\x02\0\x12\x03N-<\n\x12\n\x0b\x05\0\
    \x02@\x03\xe7\x07\0\x02\0\x01\x12\x03N.;\n\x10\n\t\x05\0\x02@\x03\xe7\
    \x07\0\x03\x12\x03N?C\n\x0f\n\x08\x05\0\x02@\x03\xe7\x07\x01\x12\x03NEW\
    \n\x10\n\t\x05\0\x02@\x03\xe7\x07\x01\x02\x12\x03NEP\n\x11\n\n\x05\0\x02\
    @\x03\xe7\x07\x01\x02\0\x12\x03NEP\n\x12\n\x0b\x05\0\x02@\x03\xe7\x07\
    \x01\x02\0\x01\x12\x03NFO\n\x10\n\t\x05\0\x02@\x03\xe7\x07\x01\x03\x12\
    \x03NSW\n\x0b\n\x04\x05\0\x02A\x12\x03O\x08E\n\x0c\n\x05\x05\0\x02A\x01\
    \x12\x03O\x08%\n\x0c\n\x05\x05\0\x02A\x02\x12\x03O(+\n\x0c\n\x05\x05\0\
    \x02A\x03\x12\x03O,D\n\x0f\n\x08\x05\0\x02A\x03\xe7\x07\0\x12\x03O-C\n\
    \x10\n\t\x05\0\x02A\x03\xe7\x07\0\x02\x12\x03O-<\n\x11\n\n\x05\0\x02A\
    \x03\xe7\x07\0\x02\0\x12\x03O-<\n\x12\n\x0b\x05\0\x02A\x03\xe7\x07\0\x02\
    \0\x01\x12\x03O.;\n\x10\n\t\x05\0\x02A\x03\xe7\x07\0\x03\x12\x03O?C\n\
    \x0b\n\x04\x05\0\x02B\x12\x03P\x08C\n\x0c\n\x05\x05\0\x02B\x01\x12\x03P\
    \x08\"\n\x0c\n\x05\x05\0\x02B\x02\x12\x03P%(\n\x0c\n\x05\x05\0\x02B\x03\
    \x12\x03P)B\n\x0f\n\x08\x05\0\x02B\x03\xe7\x07\0\x12\x03P*A\n\x10\n\t\
    \x05\0\x02B\x03\xe7\x07\0\x02\x12\x03P*:\n\x11\n\n\x05\0\x02B\x03\xe7\
    \x07\0\x02\0\x12\x03P*:\n\x12\n\x0b\x05\0\x02B\x03\xe7\x07\0\x02\0\x01\
    \x12\x03P+9\n\x10\n\t\x05\0\x02B\x03\xe7\x07\0\x03\x12\x03P=A\n\x0b\n\
    \x04\x05\0\x02C\x12\x03Q\x08A\n\x0c\n\x05\x05\0\x02C\x01\x12\x03Q\x08!\n\
    \x0c\n\x05\x05\0\x02C\x02\x12\x03Q$'\n\x0c\n\x05\x05\0\x02C\x03\x12\x03Q\
    (@\n\x0f\n\x08\x05\0\x02C\x03\xe7\x07\0\x12\x03Q)?\n\x10\n\t\x05\0\x02C\
    \x03\xe7\x07\0\x02\x12\x03Q)8\n\x11\n\n\x05\0\x02C\x03\xe7\x07\0\x02\0\
    \x12\x03Q)8\n\x12\n\x0b\x05\0\x02C\x03\xe7\x07\0\x02\0\x01\x12\x03Q*7\n\
    \x10\n\t\x05\0\x02C\x03\xe7\x07\0\x03\x12\x03Q;?\n\x0b\n\x04\x05\0\x02D\
    \x12\x03R\x08A\n\x0c\n\x05\x05\0\x02D\x01\x12\x03R\x08\x20\n\x0c\n\x05\
    \x05\0\x02D\x02\x12\x03R#&\n\x0c\n\x05\x05\0\x02D\x03\x12\x03R'@\n\x0f\n\
    \x08\x05\0\x02D\x03\xe7\x07\0\x12\x03R(?\n\x10\n\t\x05\0\x02D\x03\xe7\
    \x07\0\x02\x12\x03R(8\n\x11\n\n\x05\0\x02D\x03\xe7\x07\0\x02\0\x12\x03R(\
    8\n\x12\n\x0b\x05\0\x02D\x03\xe7\x07\0\x02\0\x01\x12\x03R)7\n\x10\n\t\
    \x05\0\x02D\x03\xe7\x07\0\x03\x12\x03R;?\n\x0b\n\x04\x05\0\x02E\x12\x03S\
    \x08G\n\x0c\n\x05\x05\0\x02E\x01\x12\x03S\x08'\n\x0c\n\x05\x05\0\x02E\
    \x02\x12\x03S*-\n\x0c\n\x05\x05\0\x02E\x03\x12\x03S.F\n\x0f\n\x08\x05\0\
    \x02E\x03\xe7\x07\0\x12\x03S/E\n\x10\n\t\x05\0\x02E\x03\xe7\x07\0\x02\
    \x12\x03S/>\n\x11\n\n\x05\0\x02E\x03\xe7\x07\0\x02\0\x12\x03S/>\n\x12\n\
    \x0b\x05\0\x02E\x03\xe7\x07\0\x02\0\x01\x12\x03S0=\n\x10\n\t\x05\0\x02E\
    \x03\xe7\x07\0\x03\x12\x03SAE\n\x0b\n\x04\x05\0\x02F\x12\x03T\x08D\n\x0c\
    \n\x05\x05\0\x02F\x01\x12\x03T\x08#\n\x0c\n\x05\x05\0\x02F\x02\x12\x03T&\
    )\n\x0c\n\x05\x05\0\x02F\x03\x12\x03T*C\n\x0f\n\x08\x05\0\x02F\x03\xe7\
    \x07\0\x12\x03T+B\n\x10\n\t\x05\0\x02F\x03\xe7\x07\0\x02\x12\x03T+;\n\
    \x11\n\n\x05\0\x02F\x03\xe7\x07\0\x02\0\x12\x03T+;\n\x12\n\x0b\x05\0\x02\
    F\x03\xe7\x07\0\x02\0\x01\x12\x03T,:\n\x10\n\t\x05\0\x02F\x03\xe7\x07\0\
    \x03\x12\x03T>B\n\x0b\n\x04\x05\0\x02G\x12\x03U\x08H\n\x0c\n\x05\x05\0\
    \x02G\x01\x12\x03U\x08(\n\x0c\n\x05\x05\0\x02G\x02\x12\x03U+.\n\x0c\n\
    \x05\x05\0\x02G\x03\x12\x03U/G\n\x0f\n\x08\x05\0\x02G\x03\xe7\x07\0\x12\
    \x03U0F\n\x10\n\t\x05\0\x02G\x03\xe7\x07\0\x02\x12\x03U0?\n\x11\n\n\x05\
    \0\x02G\x03\xe7\x07\0\x02\0\x12\x03U0?\n\x12\n\x0b\x05\0\x02G\x03\xe7\
    \x07\0\x02\0\x01\x12\x03U1>\n\x10\n\t\x05\0\x02G\x03\xe7\x07\0\x03\x12\
    \x03UBF\n\x0b\n\x04\x05\0\x02H\x12\x03V\x08G\n\x0c\n\x05\x05\0\x02H\x01\
    \x12\x03V\x08'\n\x0c\n\x05\x05\0\x02H\x02\x12\x03V*-\n\x0c\n\x05\x05\0\
    \x02H\x03\x12\x03V.F\n\x0f\n\x08\x05\0\x02H\x03\xe7\x07\0\x12\x03V/E\n\
    \x10\n\t\x05\0\x02H\x03\xe7\x07\0\x02\x12\x03V/>\n\x11\n\n\x05\0\x02H\
    \x03\xe7\x07\0\x02\0\x12\x03V/>\n\x12\n\x0b\x05\0\x02H\x03\xe7\x07\0\x02\
    \0\x01\x12\x03V0=\n\x10\n\t\x05\0\x02H\x03\xe7\x07\0\x03\x12\x03VAE\n\
    \x9c\x01\n\x02\x04\0\x12\x04a\0b\x01\x1aU*\n\x20Request:\x20Reset\x20dev\
    ice\x20to\x20default\x20state\x20and\x20ask\x20for\x20device\x20details\
    \n\x20@next\x20Features\n29//////////////////\n\x20Basic\x20messages\x20\
    //\n//////////////////\n\n\n\n\x03\x04\0\x01\x12\x03a\x08\x12\nQ\n\x02\
    \x04\x01\x12\x04h\0i\x01\x1aE*\n\x20Request:\x20Ask\x20for\x20device\x20\
    details\x20(no\x20device\x20reset)\n\x20@next\x20Features\n\n\n\n\x03\
    \x04\x01\x01\x12\x03h\x08\x13\nl\n\x02\x04\x02\x12\x05p\0\x85\x01\x01\
    \x1a_*\n\x20Response:\x20Reports\x20various\x20information\x20about\x20t\
    he\x20device\n\x20@prev\x20Initialize\n\x20@prev\x20GetFeatures\n\n\n\n\
    \x03\x04\x02\x01\x12\x03p\x08\x10\nA\n\x04\x04\x02\x02\0\x12\x03q\x08#\"\
    4\x20name\x20of\x20the\x20manufacturer,\x20e.g.\x20\"bitcointrezor.com\"\
    \n\n\x0c\n\x05\x04\x02\x02\0\x04\x12\x03q\x08\x10\n\x0c\n\x05\x04\x02\
    \x02\0\x05\x12\x03q\x11\x17\n\x0c\n\x05\x04\x02\x02\0\x01\x12\x03q\x18\
    \x1e\n\x0c\n\x05\x04\x02\x02\0\x03\x12\x03q!\"\n2\n\x04\x04\x02\x02\x01\
    \x12\x03r\x08*\"%\x20major\x20version\x20of\x20the\x20device,\x20e.g.\
    \x201\n\n\x0c\n\x05\x04\x02\x02\x01\x04\x12\x03r\x08\x10\n\x0c\n\x05\x04\
    \x02\x02\x01\x05\x12\x03r\x11\x17\n\x0c\n\x05\x04\x02\x02\x01\x01\x12\
    \x03r\x18%\n\x0c\n\x05\x04\x02\x02\x01\x03\x12\x03r()\n2\n\x04\x04\x02\
    \x02\x02\x12\x03s\x08*\"%\x20minor\x20version\x20of\x20the\x20device,\
    \x20e.g.\x200\n\n\x0c\n\x05\x04\x02\x02\x02\x04\x12\x03s\x08\x10\n\x0c\n\
    \x05\x04\x02\x02\x02\x05\x12\x03s\x11\x17\n\x0c\n\x05\x04\x02\x02\x02\
    \x01\x12\x03s\x18%\n\x0c\n\x05\x04\x02\x02\x02\x03\x12\x03s()\n2\n\x04\
    \x04\x02\x02\x03\x12\x03t\x08*\"%\x20patch\x20version\x20of\x20the\x20de\
    vice,\x20e.g.\x200\n\n\x0c\n\x05\x04\x02\x02\x03\x04\x12\x03t\x08\x10\n\
    \x0c\n\x05\x04\x02\x02\x03\x05\x12\x03t\x11\x17\n\x0c\n\x05\x04\x02\x02\
    \x03\x01\x12\x03t\x18%\n\x0c\n\x05\x04\x02\x02\x03\x03\x12\x03t()\n,\n\
    \x04\x04\x02\x02\x04\x12\x03u\x08*\"\x1f\x20is\x20device\x20in\x20bootlo\
    ader\x20mode?\n\n\x0c\n\x05\x04\x02\x02\x04\x04\x12\x03u\x08\x10\n\x0c\n\
    \x05\x04\x02\x02\x04\x05\x12\x03u\x11\x15\n\x0c\n\x05\x04\x02\x02\x04\
    \x01\x12\x03u\x16%\n\x0c\n\x05\x04\x02\x02\x04\x03\x12\x03u()\n)\n\x04\
    \x04\x02\x02\x05\x12\x03v\x08&\"\x1c\x20device's\x20unique\x20identifier\
    \n\n\x0c\n\x05\x04\x02\x02\x05\x04\x12\x03v\x08\x10\n\x0c\n\x05\x04\x02\
    \x02\x05\x05\x12\x03v\x11\x17\n\x0c\n\x05\x04\x02\x02\x05\x01\x12\x03v\
    \x18!\n\x0c\n\x05\x04\x02\x02\x05\x03\x12\x03v$%\n*\n\x04\x04\x02\x02\
    \x06\x12\x03w\x08)\"\x1d\x20is\x20device\x20protected\x20by\x20PIN?\n\n\
    \x0c\n\x05\x04\x02\x02\x06\x04\x12\x03w\x08\x10\n\x0c\n\x05\x04\x02\x02\
    \x06\x05\x12\x03w\x11\x15\n\x0c\n\x05\x04\x02\x02\x06\x01\x12\x03w\x16$\
    \n\x0c\n\x05\x04\x02\x02\x06\x03\x12\x03w'(\n;\n\x04\x04\x02\x02\x07\x12\
    \x03x\x080\".\x20is\x20node/mnemonic\x20encrypted\x20using\x20passphrase\
    ?\n\n\x0c\n\x05\x04\x02\x02\x07\x04\x12\x03x\x08\x10\n\x0c\n\x05\x04\x02\
    \x02\x07\x05\x12\x03x\x11\x15\n\x0c\n\x05\x04\x02\x02\x07\x01\x12\x03x\
    \x16+\n\x0c\n\x05\x04\x02\x02\x07\x03\x12\x03x./\n\x1e\n\x04\x04\x02\x02\
    \x08\x12\x03y\x08%\"\x11\x20device\x20language\n\n\x0c\n\x05\x04\x02\x02\
    \x08\x04\x12\x03y\x08\x10\n\x0c\n\x05\x04\x02\x02\x08\x05\x12\x03y\x11\
    \x17\n\x0c\n\x05\x04\x02\x02\x08\x01\x12\x03y\x18\x20\n\x0c\n\x05\x04\
    \x02\x02\x08\x03\x12\x03y#$\n'\n\x04\x04\x02\x02\t\x12\x03z\x08#\"\x1a\
    \x20device\x20description\x20label\n\n\x0c\n\x05\x04\x02\x02\t\x04\x12\
    \x03z\x08\x10\n\x0c\n\x05\x04\x02\x02\t\x05\x12\x03z\x11\x17\n\x0c\n\x05\
    \x04\x02\x02\t\x01\x12\x03z\x18\x1d\n\x0c\n\x05\x04\x02\x02\t\x03\x12\
    \x03z\x20\"\n\x1e\n\x04\x04\x02\x02\n\x12\x03{\x08%\"\x11\x20supported\
    \x20coins\n\n\x0c\n\x05\x04\x02\x02\n\x04\x12\x03{\x08\x10\n\x0c\n\x05\
    \x04\x02\x02\n\x06\x12\x03{\x11\x19\n\x0c\n\x05\x04\x02\x02\n\x01\x12\
    \x03{\x1a\x1f\n\x0c\n\x05\x04\x02\x02\n\x03\x12\x03{\"$\n(\n\x04\x04\x02\
    \x02\x0b\x12\x03|\x08'\"\x1b\x20does\x20device\x20contain\x20seed?\n\n\
    \x0c\n\x05\x04\x02\x02\x0b\x04\x12\x03|\x08\x10\n\x0c\n\x05\x04\x02\x02\
    \x0b\x05\x12\x03|\x11\x15\n\x0c\n\x05\x04\x02\x02\x0b\x01\x12\x03|\x16!\
    \n\x0c\n\x05\x04\x02\x02\x0b\x03\x12\x03|$&\n'\n\x04\x04\x02\x02\x0c\x12\
    \x03}\x08%\"\x1a\x20SCM\x20revision\x20of\x20firmware\n\n\x0c\n\x05\x04\
    \x02\x02\x0c\x04\x12\x03}\x08\x10\n\x0c\n\x05\x04\x02\x02\x0c\x05\x12\
    \x03}\x11\x16\n\x0c\n\x05\x04\x02\x02\x0c\x01\x12\x03}\x17\x1f\n\x0c\n\
    \x05\x04\x02\x02\x0c\x03\x12\x03}\"$\n%\n\x04\x04\x02\x02\r\x12\x03~\x08\
    ,\"\x18\x20hash\x20of\x20the\x20bootloader\n\n\x0c\n\x05\x04\x02\x02\r\
    \x04\x12\x03~\x08\x10\n\x0c\n\x05\x04\x02\x02\r\x05\x12\x03~\x11\x16\n\
    \x0c\n\x05\x04\x02\x02\r\x01\x12\x03~\x17&\n\x0c\n\x05\x04\x02\x02\r\x03\
    \x12\x03~)+\n<\n\x04\x04\x02\x02\x0e\x12\x03\x7f\x08$\"/\x20was\x20stora\
    ge\x20imported\x20from\x20an\x20external\x20source?\n\n\x0c\n\x05\x04\
    \x02\x02\x0e\x04\x12\x03\x7f\x08\x10\n\x0c\n\x05\x04\x02\x02\x0e\x05\x12\
    \x03\x7f\x11\x15\n\x0c\n\x05\x04\x02\x02\x0e\x01\x12\x03\x7f\x16\x1e\n\
    \x0c\n\x05\x04\x02\x02\x0e\x03\x12\x03\x7f!#\n1\n\x04\x04\x02\x02\x0f\
    \x12\x04\x80\x01\x08&\"#\x20is\x20PIN\x20already\x20cached\x20in\x20sess\
    ion?\n\n\r\n\x05\x04\x02\x02\x0f\x04\x12\x04\x80\x01\x08\x10\n\r\n\x05\
    \x04\x02\x02\x0f\x05\x12\x04\x80\x01\x11\x15\n\r\n\x05\x04\x02\x02\x0f\
    \x01\x12\x04\x80\x01\x16\x20\n\r\n\x05\x04\x02\x02\x0f\x03\x12\x04\x80\
    \x01#%\n8\n\x04\x04\x02\x02\x10\x12\x04\x81\x01\x08-\"*\x20is\x20passphr\
    ase\x20already\x20cached\x20in\x20session?\n\n\r\n\x05\x04\x02\x02\x10\
    \x04\x12\x04\x81\x01\x08\x10\n\r\n\x05\x04\x02\x02\x10\x05\x12\x04\x81\
    \x01\x11\x15\n\r\n\x05\x04\x02\x02\x10\x01\x12\x04\x81\x01\x16'\n\r\n\
    \x05\x04\x02\x02\x10\x03\x12\x04\x81\x01*,\n)\n\x04\x04\x02\x02\x11\x12\
    \x04\x82\x01\x08,\"\x1b\x20is\x20valid\x20firmware\x20loaded?\n\n\r\n\
    \x05\x04\x02\x02\x11\x04\x12\x04\x82\x01\x08\x10\n\r\n\x05\x04\x02\x02\
    \x11\x05\x12\x04\x82\x01\x11\x15\n\r\n\x05\x04\x02\x02\x11\x01\x12\x04\
    \x82\x01\x16&\n\r\n\x05\x04\x02\x02\x11\x03\x12\x04\x82\x01)+\nJ\n\x04\
    \x04\x02\x02\x12\x12\x04\x83\x01\x08(\"<\x20does\x20storage\x20need\x20b\
    ackup?\x20(equals\x20to\x20Storage.needs_backup)\n\n\r\n\x05\x04\x02\x02\
    \x12\x04\x12\x04\x83\x01\x08\x10\n\r\n\x05\x04\x02\x02\x12\x05\x12\x04\
    \x83\x01\x11\x15\n\r\n\x05\x04\x02\x02\x12\x01\x12\x04\x83\x01\x16\"\n\r\
    \n\x05\x04\x02\x02\x12\x03\x12\x04\x83\x01%'\n6\n\x04\x04\x02\x02\x13\
    \x12\x04\x84\x01\x08#\"(\x20device\x20flags\x20(equals\x20to\x20Storage.\
    flags)\n\n\r\n\x05\x04\x02\x02\x13\x04\x12\x04\x84\x01\x08\x10\n\r\n\x05\
    \x04\x02\x02\x13\x05\x12\x04\x84\x01\x11\x17\n\r\n\x05\x04\x02\x02\x13\
    \x01\x12\x04\x84\x01\x18\x1d\n\r\n\x05\x04\x02\x02\x13\x03\x12\x04\x84\
    \x01\x20\"\n^\n\x02\x04\x03\x12\x06\x8b\x01\0\x8c\x01\x01\x1aP*\n\x20Req\
    uest:\x20clear\x20session\x20(removes\x20cached\x20PIN,\x20passphrase,\
    \x20etc).\n\x20@next\x20Success\n\n\x0b\n\x03\x04\x03\x01\x12\x04\x8b\
    \x01\x08\x14\n\x91\x01\n\x02\x04\x04\x12\x06\x95\x01\0\x9a\x01\x01\x1a\
    \x82\x01*\n\x20Request:\x20change\x20language\x20and/or\x20label\x20of\
    \x20the\x20device\n\x20@next\x20Success\n\x20@next\x20Failure\n\x20@next\
    \x20ButtonRequest\n\x20@next\x20PinMatrixRequest\n\n\x0b\n\x03\x04\x04\
    \x01\x12\x04\x95\x01\x08\x15\n\x0c\n\x04\x04\x04\x02\0\x12\x04\x96\x01\
    \x08%\n\r\n\x05\x04\x04\x02\0\x04\x12\x04\x96\x01\x08\x10\n\r\n\x05\x04\
    \x04\x02\0\x05\x12\x04\x96\x01\x11\x17\n\r\n\x05\x04\x04\x02\0\x01\x12\
    \x04\x96\x01\x18\x20\n\r\n\x05\x04\x04\x02\0\x03\x12\x04\x96\x01#$\n\x0c\
    \n\x04\x04\x04\x02\x01\x12\x04\x97\x01\x08\"\n\r\n\x05\x04\x04\x02\x01\
    \x04\x12\x04\x97\x01\x08\x10\n\r\n\x05\x04\x04\x02\x01\x05\x12\x04\x97\
    \x01\x11\x17\n\r\n\x05\x04\x04\x02\x01\x01\x12\x04\x97\x01\x18\x1d\n\r\n\
    \x05\x04\x04\x02\x01\x03\x12\x04\x97\x01\x20!\n\x0c\n\x04\x04\x04\x02\
    \x02\x12\x04\x98\x01\x08)\n\r\n\x05\x04\x04\x02\x02\x04\x12\x04\x98\x01\
    \x08\x10\n\r\n\x05\x04\x04\x02\x02\x05\x12\x04\x98\x01\x11\x15\n\r\n\x05\
    \x04\x04\x02\x02\x01\x12\x04\x98\x01\x16$\n\r\n\x05\x04\x04\x02\x02\x03\
    \x12\x04\x98\x01'(\n\x0c\n\x04\x04\x04\x02\x03\x12\x04\x99\x01\x08&\n\r\
    \n\x05\x04\x04\x02\x03\x04\x12\x04\x99\x01\x08\x10\n\r\n\x05\x04\x04\x02\
    \x03\x05\x12\x04\x99\x01\x11\x16\n\r\n\x05\x04\x04\x02\x03\x01\x12\x04\
    \x99\x01\x17!\n\r\n\x05\x04\x04\x02\x03\x03\x12\x04\x99\x01$%\nP\n\x02\
    \x04\x05\x12\x06\xa1\x01\0\xa3\x01\x01\x1aB*\n\x20Request:\x20set\x20fla\
    gs\x20of\x20the\x20device\n\x20@next\x20Success\n\x20@next\x20Failure\n\
    \n\x0b\n\x03\x04\x05\x01\x12\x04\xa1\x01\x08\x12\n5\n\x04\x04\x05\x02\0\
    \x12\x04\xa2\x01\x08\"\"'\x20bitmask,\x20can\x20only\x20set\x20bits,\x20\
    not\x20unset\n\n\r\n\x05\x04\x05\x02\0\x04\x12\x04\xa2\x01\x08\x10\n\r\n\
    \x05\x04\x05\x02\0\x05\x12\x04\xa2\x01\x11\x17\n\r\n\x05\x04\x05\x02\0\
    \x01\x12\x04\xa2\x01\x18\x1d\n\r\n\x05\x04\x05\x02\0\x03\x12\x04\xa2\x01\
    \x20!\n}\n\x02\x04\x06\x12\x06\xaa\x01\0\xac\x01\x01\x1ao*\n\x20Request:\
    \x20Starts\x20workflow\x20for\x20setting/changing/removing\x20the\x20PIN\
    \n\x20@next\x20ButtonRequest\n\x20@next\x20PinMatrixRequest\n\n\x0b\n\
    \x03\x04\x06\x01\x12\x04\xaa\x01\x08\x11\n)\n\x04\x04\x06\x02\0\x12\x04\
    \xab\x01\x08!\"\x1b\x20is\x20PIN\x20removal\x20requested?\n\n\r\n\x05\
    \x04\x06\x02\0\x04\x12\x04\xab\x01\x08\x10\n\r\n\x05\x04\x06\x02\0\x05\
    \x12\x04\xab\x01\x11\x15\n\r\n\x05\x04\x06\x02\0\x01\x12\x04\xab\x01\x16\
    \x1c\n\r\n\x05\x04\x06\x02\0\x03\x12\x04\xab\x01\x1f\x20\nx\n\x02\x04\
    \x07\x12\x06\xb2\x01\0\xb7\x01\x01\x1aj*\n\x20Request:\x20Test\x20if\x20\
    the\x20device\x20is\x20alive,\x20device\x20sends\x20back\x20the\x20messa\
    ge\x20in\x20Success\x20response\n\x20@next\x20Success\n\n\x0b\n\x03\x04\
    \x07\x01\x12\x04\xb2\x01\x08\x0c\n7\n\x04\x04\x07\x02\0\x12\x04\xb3\x01\
    \x08$\")\x20message\x20to\x20send\x20back\x20in\x20Success\x20message\n\
    \n\r\n\x05\x04\x07\x02\0\x04\x12\x04\xb3\x01\x08\x10\n\r\n\x05\x04\x07\
    \x02\0\x05\x12\x04\xb3\x01\x11\x17\n\r\n\x05\x04\x07\x02\0\x01\x12\x04\
    \xb3\x01\x18\x1f\n\r\n\x05\x04\x07\x02\0\x03\x12\x04\xb3\x01\"#\n$\n\x04\
    \x04\x07\x02\x01\x12\x04\xb4\x01\x08,\"\x16\x20ask\x20for\x20button\x20p\
    ress\n\n\r\n\x05\x04\x07\x02\x01\x04\x12\x04\xb4\x01\x08\x10\n\r\n\x05\
    \x04\x07\x02\x01\x05\x12\x04\xb4\x01\x11\x15\n\r\n\x05\x04\x07\x02\x01\
    \x01\x12\x04\xb4\x01\x16'\n\r\n\x05\x04\x07\x02\x01\x03\x12\x04\xb4\x01*\
    +\n,\n\x04\x04\x07\x02\x02\x12\x04\xb5\x01\x08)\"\x1e\x20ask\x20for\x20P\
    IN\x20if\x20set\x20in\x20device\n\n\r\n\x05\x04\x07\x02\x02\x04\x12\x04\
    \xb5\x01\x08\x10\n\r\n\x05\x04\x07\x02\x02\x05\x12\x04\xb5\x01\x11\x15\n\
    \r\n\x05\x04\x07\x02\x02\x01\x12\x04\xb5\x01\x16$\n\r\n\x05\x04\x07\x02\
    \x02\x03\x12\x04\xb5\x01'(\n3\n\x04\x04\x07\x02\x03\x12\x04\xb6\x01\x080\
    \"%\x20ask\x20for\x20passphrase\x20if\x20set\x20in\x20device\n\n\r\n\x05\
    \x04\x07\x02\x03\x04\x12\x04\xb6\x01\x08\x10\n\r\n\x05\x04\x07\x02\x03\
    \x05\x12\x04\xb6\x01\x11\x15\n\r\n\x05\x04\x07\x02\x03\x01\x12\x04\xb6\
    \x01\x16+\n\r\n\x05\x04\x07\x02\x03\x03\x12\x04\xb6\x01./\n;\n\x02\x04\
    \x08\x12\x06\xbc\x01\0\xbe\x01\x01\x1a-*\n\x20Response:\x20Success\x20of\
    \x20the\x20previous\x20request\n\n\x0b\n\x03\x04\x08\x01\x12\x04\xbc\x01\
    \x08\x0f\nP\n\x04\x04\x08\x02\0\x12\x04\xbd\x01\x08$\"B\x20human\x20read\
    able\x20description\x20of\x20action\x20or\x20request-specific\x20payload\
    \n\n\r\n\x05\x04\x08\x02\0\x04\x12\x04\xbd\x01\x08\x10\n\r\n\x05\x04\x08\
    \x02\0\x05\x12\x04\xbd\x01\x11\x17\n\r\n\x05\x04\x08\x02\0\x01\x12\x04\
    \xbd\x01\x18\x1f\n\r\n\x05\x04\x08\x02\0\x03\x12\x04\xbd\x01\"#\n;\n\x02\
    \x04\t\x12\x06\xc3\x01\0\xc6\x01\x01\x1a-*\n\x20Response:\x20Failure\x20\
    of\x20the\x20previous\x20request\n\n\x0b\n\x03\x04\t\x01\x12\x04\xc3\x01\
    \x08\x0f\n?\n\x04\x04\t\x02\0\x12\x04\xc4\x01\x08&\"1\x20computer-readab\
    le\x20definition\x20of\x20the\x20error\x20state\n\n\r\n\x05\x04\t\x02\0\
    \x04\x12\x04\xc4\x01\x08\x10\n\r\n\x05\x04\t\x02\0\x06\x12\x04\xc4\x01\
    \x11\x1c\n\r\n\x05\x04\t\x02\0\x01\x12\x04\xc4\x01\x1d!\n\r\n\x05\x04\t\
    \x02\0\x03\x12\x04\xc4\x01$%\n9\n\x04\x04\t\x02\x01\x12\x04\xc5\x01\x08$\
    \"+\x20human-readable\x20message\x20of\x20the\x20error\x20state\n\n\r\n\
    \x05\x04\t\x02\x01\x04\x12\x04\xc5\x01\x08\x10\n\r\n\x05\x04\t\x02\x01\
    \x05\x12\x04\xc5\x01\x11\x17\n\r\n\x05\x04\t\x02\x01\x01\x12\x04\xc5\x01\
    \x18\x1f\n\r\n\x05\x04\t\x02\x01\x03\x12\x04\xc5\x01\"#\na\n\x02\x04\n\
    \x12\x06\xcd\x01\0\xd0\x01\x01\x1aS*\n\x20Response:\x20Device\x20is\x20w\
    aiting\x20for\x20HW\x20button\x20press.\n\x20@next\x20ButtonAck\n\x20@ne\
    xt\x20Cancel\n\n\x0b\n\x03\x04\n\x01\x12\x04\xcd\x01\x08\x15\n\x0c\n\x04\
    \x04\n\x02\0\x12\x04\xce\x01\x08,\n\r\n\x05\x04\n\x02\0\x04\x12\x04\xce\
    \x01\x08\x10\n\r\n\x05\x04\n\x02\0\x06\x12\x04\xce\x01\x11\"\n\r\n\x05\
    \x04\n\x02\0\x01\x12\x04\xce\x01#'\n\r\n\x05\x04\n\x02\0\x03\x12\x04\xce\
    \x01*+\n\x0c\n\x04\x04\n\x02\x01\x12\x04\xcf\x01\x08!\n\r\n\x05\x04\n\
    \x02\x01\x04\x12\x04\xcf\x01\x08\x10\n\r\n\x05\x04\n\x02\x01\x05\x12\x04\
    \xcf\x01\x11\x17\n\r\n\x05\x04\n\x02\x01\x01\x12\x04\xcf\x01\x18\x1c\n\r\
    \n\x05\x04\n\x02\x01\x03\x12\x04\xcf\x01\x1f\x20\n[\n\x02\x04\x0b\x12\
    \x06\xd6\x01\0\xd7\x01\x01\x1aM*\n\x20Request:\x20Computer\x20agrees\x20\
    to\x20wait\x20for\x20HW\x20button\x20press\n\x20@prev\x20ButtonRequest\n\
    \n\x0b\n\x03\x04\x0b\x01\x12\x04\xd6\x01\x08\x11\n\x9b\x01\n\x02\x04\x0c\
    \x12\x06\xde\x01\0\xe0\x01\x01\x1a\x8c\x01*\n\x20Response:\x20Device\x20\
    is\x20asking\x20computer\x20to\x20show\x20PIN\x20matrix\x20and\x20awaits\
    \x20PIN\x20encoded\x20using\x20this\x20matrix\x20scheme\n\x20@next\x20Pi\
    nMatrixAck\n\x20@next\x20Cancel\n\n\x0b\n\x03\x04\x0c\x01\x12\x04\xde\
    \x01\x08\x18\n\x0c\n\x04\x04\x0c\x02\0\x12\x04\xdf\x01\x08/\n\r\n\x05\
    \x04\x0c\x02\0\x04\x12\x04\xdf\x01\x08\x10\n\r\n\x05\x04\x0c\x02\0\x06\
    \x12\x04\xdf\x01\x11%\n\r\n\x05\x04\x0c\x02\0\x01\x12\x04\xdf\x01&*\n\r\
    \n\x05\x04\x0c\x02\0\x03\x12\x04\xdf\x01-.\nU\n\x02\x04\r\x12\x06\xe6\
    \x01\0\xe8\x01\x01\x1aG*\n\x20Request:\x20Computer\x20responds\x20with\
    \x20encoded\x20PIN\n\x20@prev\x20PinMatrixRequest\n\n\x0b\n\x03\x04\r\
    \x01\x12\x04\xe6\x01\x08\x14\n2\n\x04\x04\r\x02\0\x12\x04\xe7\x01\x08\
    \x20\"$\x20matrix\x20encoded\x20PIN\x20entered\x20by\x20user\n\n\r\n\x05\
    \x04\r\x02\0\x04\x12\x04\xe7\x01\x08\x10\n\r\n\x05\x04\r\x02\0\x05\x12\
    \x04\xe7\x01\x11\x17\n\r\n\x05\x04\r\x02\0\x01\x12\x04\xe7\x01\x18\x1b\n\
    \r\n\x05\x04\r\x02\0\x03\x12\x04\xe7\x01\x1e\x1f\n\x95\x01\n\x02\x04\x0e\
    \x12\x06\xf0\x01\0\xf1\x01\x01\x1a\x86\x01*\n\x20Request:\x20Abort\x20la\
    st\x20operation\x20that\x20required\x20user\x20interaction\n\x20@prev\
    \x20ButtonRequest\n\x20@prev\x20PinMatrixRequest\n\x20@prev\x20Passphras\
    eRequest\n\n\x0b\n\x03\x04\x0e\x01\x12\x04\xf0\x01\x08\x0e\nb\n\x02\x04\
    \x0f\x12\x06\xf8\x01\0\xf9\x01\x01\x1aT*\n\x20Response:\x20Device\x20awa\
    its\x20encryption\x20passphrase\n\x20@next\x20PassphraseAck\n\x20@next\
    \x20Cancel\n\n\x0b\n\x03\x04\x0f\x01\x12\x04\xf8\x01\x08\x19\nH\n\x02\
    \x04\x10\x12\x06\xff\x01\0\x81\x02\x01\x1a:*\n\x20Request:\x20Send\x20pa\
    ssphrase\x20back\n\x20@prev\x20PassphraseRequest\n\n\x0b\n\x03\x04\x10\
    \x01\x12\x04\xff\x01\x08\x15\n\x0c\n\x04\x04\x10\x02\0\x12\x04\x80\x02\
    \x08'\n\r\n\x05\x04\x10\x02\0\x04\x12\x04\x80\x02\x08\x10\n\r\n\x05\x04\
    \x10\x02\0\x05\x12\x04\x80\x02\x11\x17\n\r\n\x05\x04\x10\x02\0\x01\x12\
    \x04\x80\x02\x18\"\n\r\n\x05\x04\x10\x02\0\x03\x12\x04\x80\x02%&\n\xa2\
    \x01\n\x02\x04\x11\x12\x06\x89\x02\0\x8b\x02\x01\x1a\x93\x01*\n\x20Reque\
    st:\x20Request\x20a\x20sample\x20of\x20random\x20data\x20generated\x20by\
    \x20hardware\x20RNG.\x20May\x20be\x20used\x20for\x20testing.\n\x20@next\
    \x20ButtonRequest\n\x20@next\x20Entropy\n\x20@next\x20Failure\n\n\x0b\n\
    \x03\x04\x11\x01\x12\x04\x89\x02\x08\x12\n)\n\x04\x04\x11\x02\0\x12\x04\
    \x8a\x02\x08!\"\x1b\x20size\x20of\x20requested\x20entropy\n\n\r\n\x05\
    \x04\x11\x02\0\x04\x12\x04\x8a\x02\x08\x10\n\r\n\x05\x04\x11\x02\0\x05\
    \x12\x04\x8a\x02\x11\x17\n\r\n\x05\x04\x11\x02\0\x01\x12\x04\x8a\x02\x18\
    \x1c\n\r\n\x05\x04\x11\x02\0\x03\x12\x04\x8a\x02\x1f\x20\n^\n\x02\x04\
    \x12\x12\x06\x91\x02\0\x93\x02\x01\x1aP*\n\x20Response:\x20Reply\x20with\
    \x20random\x20data\x20generated\x20by\x20internal\x20RNG\n\x20@prev\x20G\
    etEntropy\n\n\x0b\n\x03\x04\x12\x01\x12\x04\x91\x02\x08\x0f\n0\n\x04\x04\
    \x12\x02\0\x12\x04\x92\x02\x08#\"\"\x20stream\x20of\x20random\x20generat\
    ed\x20bytes\n\n\r\n\x05\x04\x12\x02\0\x04\x12\x04\x92\x02\x08\x10\n\r\n\
    \x05\x04\x12\x02\0\x05\x12\x04\x92\x02\x11\x16\n\r\n\x05\x04\x12\x02\0\
    \x01\x12\x04\x92\x02\x17\x1e\n\r\n\x05\x04\x12\x02\0\x03\x12\x04\x92\x02\
    !\"\n\x8d\x01\n\x02\x04\x13\x12\x06\x9b\x02\0\xa0\x02\x01\x1a\x7f*\n\x20\
    Request:\x20Ask\x20device\x20for\x20public\x20key\x20corresponding\x20to\
    \x20address_n\x20path\n\x20@next\x20PassphraseRequest\n\x20@next\x20Publ\
    icKey\n\x20@next\x20Failure\n\n\x0b\n\x03\x04\x13\x01\x12\x04\x9b\x02\
    \x08\x14\n>\n\x04\x04\x13\x02\0\x12\x04\x9c\x02\x08&\"0\x20BIP-32\x20pat\
    h\x20to\x20derive\x20the\x20key\x20from\x20master\x20node\n\n\r\n\x05\
    \x04\x13\x02\0\x04\x12\x04\x9c\x02\x08\x10\n\r\n\x05\x04\x13\x02\0\x05\
    \x12\x04\x9c\x02\x11\x17\n\r\n\x05\x04\x13\x02\0\x01\x12\x04\x9c\x02\x18\
    !\n\r\n\x05\x04\x13\x02\0\x03\x12\x04\x9c\x02$%\n'\n\x04\x04\x13\x02\x01\
    \x12\x04\x9d\x02\x08-\"\x19\x20ECDSA\x20curve\x20name\x20to\x20use\n\n\r\
    \n\x05\x04\x13\x02\x01\x04\x12\x04\x9d\x02\x08\x10\n\r\n\x05\x04\x13\x02\
    \x01\x05\x12\x04\x9d\x02\x11\x17\n\r\n\x05\x04\x13\x02\x01\x01\x12\x04\
    \x9d\x02\x18(\n\r\n\x05\x04\x13\x02\x01\x03\x12\x04\x9d\x02+,\nD\n\x04\
    \x04\x13\x02\x02\x12\x04\x9e\x02\x08'\"6\x20optionally\x20show\x20on\x20\
    display\x20before\x20sending\x20the\x20result\n\n\r\n\x05\x04\x13\x02\
    \x02\x04\x12\x04\x9e\x02\x08\x10\n\r\n\x05\x04\x13\x02\x02\x05\x12\x04\
    \x9e\x02\x11\x15\n\r\n\x05\x04\x13\x02\x02\x01\x12\x04\x9e\x02\x16\"\n\r\
    \n\x05\x04\x13\x02\x02\x03\x12\x04\x9e\x02%&\n\x0c\n\x04\x04\x13\x02\x03\
    \x12\x04\x9f\x02\x08:\n\r\n\x05\x04\x13\x02\x03\x04\x12\x04\x9f\x02\x08\
    \x10\n\r\n\x05\x04\x13\x02\x03\x05\x12\x04\x9f\x02\x11\x17\n\r\n\x05\x04\
    \x13\x02\x03\x01\x12\x04\x9f\x02\x18!\n\r\n\x05\x04\x13\x02\x03\x03\x12\
    \x04\x9f\x02$%\n\r\n\x05\x04\x13\x02\x03\x08\x12\x04\x9f\x02&9\n\r\n\x05\
    \x04\x13\x02\x03\x07\x12\x04\x9f\x02/8\nd\n\x02\x04\x14\x12\x06\xa6\x02\
    \0\xa9\x02\x01\x1aV*\n\x20Response:\x20Contains\x20public\x20key\x20deri\
    ved\x20from\x20device\x20private\x20seed\n\x20@prev\x20GetPublicKey\n\n\
    \x0b\n\x03\x04\x14\x01\x12\x04\xa6\x02\x08\x11\n!\n\x04\x04\x14\x02\0\
    \x12\x04\xa7\x02\x08%\"\x13\x20BIP32\x20public\x20node\n\n\r\n\x05\x04\
    \x14\x02\0\x04\x12\x04\xa7\x02\x08\x10\n\r\n\x05\x04\x14\x02\0\x06\x12\
    \x04\xa7\x02\x11\x1b\n\r\n\x05\x04\x14\x02\0\x01\x12\x04\xa7\x02\x1c\x20\
    \n\r\n\x05\x04\x14\x02\0\x03\x12\x04\xa7\x02#$\n.\n\x04\x04\x14\x02\x01\
    \x12\x04\xa8\x02\x08!\"\x20\x20serialized\x20form\x20of\x20public\x20nod\
    e\n\n\r\n\x05\x04\x14\x02\x01\x04\x12\x04\xa8\x02\x08\x10\n\r\n\x05\x04\
    \x14\x02\x01\x05\x12\x04\xa8\x02\x11\x17\n\r\n\x05\x04\x14\x02\x01\x01\
    \x12\x04\xa8\x02\x18\x1c\n\r\n\x05\x04\x14\x02\x01\x03\x12\x04\xa8\x02\
    \x1f\x20\n\x88\x01\n\x02\x04\x15\x12\x06\xb1\x02\0\xb7\x02\x01\x1az*\n\
    \x20Request:\x20Ask\x20device\x20for\x20address\x20corresponding\x20to\
    \x20address_n\x20path\n\x20@next\x20PassphraseRequest\n\x20@next\x20Addr\
    ess\n\x20@next\x20Failure\n\n\x0b\n\x03\x04\x15\x01\x12\x04\xb1\x02\x08\
    \x12\n>\n\x04\x04\x15\x02\0\x12\x04\xb2\x02\x08&\"0\x20BIP-32\x20path\
    \x20to\x20derive\x20the\x20key\x20from\x20master\x20node\n\n\r\n\x05\x04\
    \x15\x02\0\x04\x12\x04\xb2\x02\x08\x10\n\r\n\x05\x04\x15\x02\0\x05\x12\
    \x04\xb2\x02\x11\x17\n\r\n\x05\x04\x15\x02\0\x01\x12\x04\xb2\x02\x18!\n\
    \r\n\x05\x04\x15\x02\0\x03\x12\x04\xb2\x02$%\n\x0c\n\x04\x04\x15\x02\x01\
    \x12\x04\xb3\x02\x08:\n\r\n\x05\x04\x15\x02\x01\x04\x12\x04\xb3\x02\x08\
    \x10\n\r\n\x05\x04\x15\x02\x01\x05\x12\x04\xb3\x02\x11\x17\n\r\n\x05\x04\
    \x15\x02\x01\x01\x12\x04\xb3\x02\x18!\n\r\n\x05\x04\x15\x02\x01\x03\x12\
    \x04\xb3\x02$%\n\r\n\x05\x04\x15\x02\x01\x08\x12\x04\xb3\x02&9\n\r\n\x05\
    \x04\x15\x02\x01\x07\x12\x04\xb3\x02/8\nD\n\x04\x04\x15\x02\x02\x12\x04\
    \xb4\x02\x089\"6\x20optionally\x20show\x20on\x20display\x20before\x20sen\
    ding\x20the\x20result\n\n\r\n\x05\x04\x15\x02\x02\x04\x12\x04\xb4\x02\
    \x08\x10\n\r\n\x05\x04\x15\x02\x02\x05\x12\x04\xb4\x02\x11\x15\n\r\n\x05\
    \x04\x15\x02\x02\x01\x12\x04\xb4\x02\x16\"\n\r\n\x05\x04\x15\x02\x02\x03\
    \x12\x04\xb4\x02%&\n;\n\x04\x04\x15\x02\x03\x12\x04\xb5\x02\x087\"-\x20f\
    illed\x20if\x20we\x20are\x20showing\x20a\x20multisig\x20address\n\n\r\n\
    \x05\x04\x15\x02\x03\x04\x12\x04\xb5\x02\x08\x10\n\r\n\x05\x04\x15\x02\
    \x03\x06\x12\x04\xb5\x02\x11)\n\r\n\x05\x04\x15\x02\x03\x01\x12\x04\xb5\
    \x02*2\n\r\n\x05\x04\x15\x02\x03\x03\x12\x04\xb5\x0256\n^\n\x04\x04\x15\
    \x02\x04\x12\x04\xb6\x02\x08H\"P\x20used\x20to\x20distinguish\x20between\
    \x20various\x20address\x20formats\x20(non-segwit,\x20segwit,\x20etc.)\n\
    \n\r\n\x05\x04\x15\x02\x04\x04\x12\x04\xb6\x02\x08\x10\n\r\n\x05\x04\x15\
    \x02\x04\x06\x12\x04\xb6\x02\x11\x20\n\r\n\x05\x04\x15\x02\x04\x01\x12\
    \x04\xb6\x02!,\n\r\n\x05\x04\x15\x02\x04\x03\x12\x04\xb6\x02/0\n\r\n\x05\
    \x04\x15\x02\x04\x08\x12\x04\xb6\x021G\n\r\n\x05\x04\x15\x02\x04\x07\x12\
    \x04\xb6\x02:F\n\x9a\x01\n\x02\x04\x16\x12\x06\xbf\x02\0\xc2\x02\x01\x1a\
    \x8b\x01*\n\x20Request:\x20Ask\x20device\x20for\x20Ethereum\x20address\
    \x20corresponding\x20to\x20address_n\x20path\n\x20@next\x20PassphraseReq\
    uest\n\x20@next\x20EthereumAddress\n\x20@next\x20Failure\n\n\x0b\n\x03\
    \x04\x16\x01\x12\x04\xbf\x02\x08\x1a\n>\n\x04\x04\x16\x02\0\x12\x04\xc0\
    \x02\x08&\"0\x20BIP-32\x20path\x20to\x20derive\x20the\x20key\x20from\x20\
    master\x20node\n\n\r\n\x05\x04\x16\x02\0\x04\x12\x04\xc0\x02\x08\x10\n\r\
    \n\x05\x04\x16\x02\0\x05\x12\x04\xc0\x02\x11\x17\n\r\n\x05\x04\x16\x02\0\
    \x01\x12\x04\xc0\x02\x18!\n\r\n\x05\x04\x16\x02\0\x03\x12\x04\xc0\x02$%\
    \nD\n\x04\x04\x16\x02\x01\x12\x04\xc1\x02\x08'\"6\x20optionally\x20show\
    \x20on\x20display\x20before\x20sending\x20the\x20result\n\n\r\n\x05\x04\
    \x16\x02\x01\x04\x12\x04\xc1\x02\x08\x10\n\r\n\x05\x04\x16\x02\x01\x05\
    \x12\x04\xc1\x02\x11\x15\n\r\n\x05\x04\x16\x02\x01\x01\x12\x04\xc1\x02\
    \x16\"\n\r\n\x05\x04\x16\x02\x01\x03\x12\x04\xc1\x02%&\n_\n\x02\x04\x17\
    \x12\x06\xc8\x02\0\xca\x02\x01\x1aQ*\n\x20Response:\x20Contains\x20addre\
    ss\x20derived\x20from\x20device\x20private\x20seed\n\x20@prev\x20GetAddr\
    ess\n\n\x0b\n\x03\x04\x17\x01\x12\x04\xc8\x02\x08\x0f\n/\n\x04\x04\x17\
    \x02\0\x12\x04\xc9\x02\x08$\"!\x20Coin\x20address\x20in\x20Base58\x20enc\
    oding\n\n\r\n\x05\x04\x17\x02\0\x04\x12\x04\xc9\x02\x08\x10\n\r\n\x05\
    \x04\x17\x02\0\x05\x12\x04\xc9\x02\x11\x17\n\r\n\x05\x04\x17\x02\0\x01\
    \x12\x04\xc9\x02\x18\x1f\n\r\n\x05\x04\x17\x02\0\x03\x12\x04\xc9\x02\"#\
    \ns\n\x02\x04\x18\x12\x06\xd0\x02\0\xd2\x02\x01\x1ae*\n\x20Response:\x20\
    Contains\x20an\x20Ethereum\x20address\x20derived\x20from\x20device\x20pr\
    ivate\x20seed\n\x20@prev\x20EthereumGetAddress\n\n\x0b\n\x03\x04\x18\x01\
    \x12\x04\xd0\x02\x08\x17\n8\n\x04\x04\x18\x02\0\x12\x04\xd1\x02\x08#\"*\
    \x20Coin\x20address\x20as\x20an\x20Ethereum\x20160\x20bit\x20hash\n\n\r\
    \n\x05\x04\x18\x02\0\x04\x12\x04\xd1\x02\x08\x10\n\r\n\x05\x04\x18\x02\0\
    \x05\x12\x04\xd1\x02\x11\x16\n\r\n\x05\x04\x18\x02\0\x01\x12\x04\xd1\x02\
    \x17\x1e\n\r\n\x05\x04\x18\x02\0\x03\x12\x04\xd1\x02!\"\nf\n\x02\x04\x19\
    \x12\x06\xd8\x02\0\xd9\x02\x01\x1aX*\n\x20Request:\x20Request\x20device\
    \x20to\x20wipe\x20all\x20sensitive\x20data\x20and\x20settings\n\x20@next\
    \x20ButtonRequest\n\n\x0b\n\x03\x04\x19\x01\x12\x04\xd8\x02\x08\x12\n\
    \x87\x01\n\x02\x04\x1a\x12\x06\xe1\x02\0\xea\x02\x01\x1ay*\n\x20Request:\
    \x20Load\x20seed\x20and\x20related\x20internal\x20settings\x20from\x20th\
    e\x20computer\n\x20@next\x20ButtonRequest\n\x20@next\x20Success\n\x20@ne\
    xt\x20Failure\n\n\x0b\n\x03\x04\x1a\x01\x12\x04\xe1\x02\x08\x12\nD\n\x04\
    \x04\x1a\x02\0\x12\x04\xe2\x02\x08%\"6\x20seed\x20encoded\x20as\x20BIP-3\
    9\x20mnemonic\x20(12,\x2018\x20or\x2024\x20words)\n\n\r\n\x05\x04\x1a\
    \x02\0\x04\x12\x04\xe2\x02\x08\x10\n\r\n\x05\x04\x1a\x02\0\x05\x12\x04\
    \xe2\x02\x11\x17\n\r\n\x05\x04\x1a\x02\0\x01\x12\x04\xe2\x02\x18\x20\n\r\
    \n\x05\x04\x1a\x02\0\x03\x12\x04\xe2\x02#$\n\x1b\n\x04\x04\x1a\x02\x01\
    \x12\x04\xe3\x02\x08%\"\r\x20BIP-32\x20node\n\n\r\n\x05\x04\x1a\x02\x01\
    \x04\x12\x04\xe3\x02\x08\x10\n\r\n\x05\x04\x1a\x02\x01\x06\x12\x04\xe3\
    \x02\x11\x1b\n\r\n\x05\x04\x1a\x02\x01\x01\x12\x04\xe3\x02\x1c\x20\n\r\n\
    \x05\x04\x1a\x02\x01\x03\x12\x04\xe3\x02#$\n\"\n\x04\x04\x1a\x02\x02\x12\
    \x04\xe4\x02\x08\x20\"\x14\x20set\x20PIN\x20protection\n\n\r\n\x05\x04\
    \x1a\x02\x02\x04\x12\x04\xe4\x02\x08\x10\n\r\n\x05\x04\x1a\x02\x02\x05\
    \x12\x04\xe4\x02\x11\x17\n\r\n\x05\x04\x1a\x02\x02\x01\x12\x04\xe4\x02\
    \x18\x1b\n\r\n\x05\x04\x1a\x02\x02\x03\x12\x04\xe4\x02\x1e\x1f\n>\n\x04\
    \x04\x1a\x02\x03\x12\x04\xe5\x02\x080\"0\x20enable\x20master\x20node\x20\
    encryption\x20using\x20passphrase\n\n\r\n\x05\x04\x1a\x02\x03\x04\x12\
    \x04\xe5\x02\x08\x10\n\r\n\x05\x04\x1a\x02\x03\x05\x12\x04\xe5\x02\x11\
    \x15\n\r\n\x05\x04\x1a\x02\x03\x01\x12\x04\xe5\x02\x16+\n\r\n\x05\x04\
    \x1a\x02\x03\x03\x12\x04\xe5\x02./\n\x1f\n\x04\x04\x1a\x02\x04\x12\x04\
    \xe6\x02\x089\"\x11\x20device\x20language\n\n\r\n\x05\x04\x1a\x02\x04\
    \x04\x12\x04\xe6\x02\x08\x10\n\r\n\x05\x04\x1a\x02\x04\x05\x12\x04\xe6\
    \x02\x11\x17\n\r\n\x05\x04\x1a\x02\x04\x01\x12\x04\xe6\x02\x18\x20\n\r\n\
    \x05\x04\x1a\x02\x04\x03\x12\x04\xe6\x02#$\n\r\n\x05\x04\x1a\x02\x04\x08\
    \x12\x04\xe6\x02%8\n\r\n\x05\x04\x1a\x02\x04\x07\x12\x04\xe6\x02.7\n\x1c\
    \n\x04\x04\x1a\x02\x05\x12\x04\xe7\x02\x08\"\"\x0e\x20device\x20label\n\
    \n\r\n\x05\x04\x1a\x02\x05\x04\x12\x04\xe7\x02\x08\x10\n\r\n\x05\x04\x1a\
    \x02\x05\x05\x12\x04\xe7\x02\x11\x17\n\r\n\x05\x04\x1a\x02\x05\x01\x12\
    \x04\xe7\x02\x18\x1d\n\r\n\x05\x04\x1a\x02\x05\x03\x12\x04\xe7\x02\x20!\
    \n>\n\x04\x04\x1a\x02\x06\x12\x04\xe8\x02\x08(\"0\x20do\x20not\x20test\
    \x20mnemonic\x20for\x20valid\x20BIP-39\x20checksum\n\n\r\n\x05\x04\x1a\
    \x02\x06\x04\x12\x04\xe8\x02\x08\x10\n\r\n\x05\x04\x1a\x02\x06\x05\x12\
    \x04\xe8\x02\x11\x15\n\r\n\x05\x04\x1a\x02\x06\x01\x12\x04\xe8\x02\x16#\
    \n\r\n\x05\x04\x1a\x02\x06\x03\x12\x04\xe8\x02&'\n\x1b\n\x04\x04\x1a\x02\
    \x07\x12\x04\xe9\x02\x08(\"\r\x20U2F\x20counter\n\n\r\n\x05\x04\x1a\x02\
    \x07\x04\x12\x04\xe9\x02\x08\x10\n\r\n\x05\x04\x1a\x02\x07\x05\x12\x04\
    \xe9\x02\x11\x17\n\r\n\x05\x04\x1a\x02\x07\x01\x12\x04\xe9\x02\x18#\n\r\
    \n\x05\x04\x1a\x02\x07\x03\x12\x04\xe9\x02&'\nz\n\x02\x04\x1b\x12\x06\
    \xf1\x02\0\xfa\x02\x01\x1al*\n\x20Request:\x20Ask\x20device\x20to\x20do\
    \x20initialization\x20involving\x20user\x20interaction\n\x20@next\x20Ent\
    ropyRequest\n\x20@next\x20Failure\n\n\x0b\n\x03\x04\x1b\x01\x12\x04\xf1\
    \x02\x08\x13\n\\\n\x04\x04\x1b\x02\0\x12\x04\xf2\x02\x08)\"N\x20display\
    \x20entropy\x20generated\x20by\x20the\x20device\x20before\x20asking\x20f\
    or\x20additional\x20entropy\n\n\r\n\x05\x04\x1b\x02\0\x04\x12\x04\xf2\
    \x02\x08\x10\n\r\n\x05\x04\x1b\x02\0\x05\x12\x04\xf2\x02\x11\x15\n\r\n\
    \x05\x04\x1b\x02\0\x01\x12\x04\xf2\x02\x16$\n\r\n\x05\x04\x1b\x02\0\x03\
    \x12\x04\xf2\x02'(\n(\n\x04\x04\x1b\x02\x01\x12\x04\xf3\x02\x083\"\x1a\
    \x20strength\x20of\x20seed\x20in\x20bits\n\n\r\n\x05\x04\x1b\x02\x01\x04\
    \x12\x04\xf3\x02\x08\x10\n\r\n\x05\x04\x1b\x02\x01\x05\x12\x04\xf3\x02\
    \x11\x17\n\r\n\x05\x04\x1b\x02\x01\x01\x12\x04\xf3\x02\x18\x20\n\r\n\x05\
    \x04\x1b\x02\x01\x03\x12\x04\xf3\x02#$\n\r\n\x05\x04\x1b\x02\x01\x08\x12\
    \x04\xf3\x02%2\n\r\n\x05\x04\x1b\x02\x01\x07\x12\x04\xf3\x02.1\n>\n\x04\
    \x04\x1b\x02\x02\x12\x04\xf4\x02\x080\"0\x20enable\x20master\x20node\x20\
    encryption\x20using\x20passphrase\n\n\r\n\x05\x04\x1b\x02\x02\x04\x12\
    \x04\xf4\x02\x08\x10\n\r\n\x05\x04\x1b\x02\x02\x05\x12\x04\xf4\x02\x11\
    \x15\n\r\n\x05\x04\x1b\x02\x02\x01\x12\x04\xf4\x02\x16+\n\r\n\x05\x04\
    \x1b\x02\x02\x03\x12\x04\xf4\x02./\n%\n\x04\x04\x1b\x02\x03\x12\x04\xf5\
    \x02\x08)\"\x17\x20enable\x20PIN\x20protection\n\n\r\n\x05\x04\x1b\x02\
    \x03\x04\x12\x04\xf5\x02\x08\x10\n\r\n\x05\x04\x1b\x02\x03\x05\x12\x04\
    \xf5\x02\x11\x15\n\r\n\x05\x04\x1b\x02\x03\x01\x12\x04\xf5\x02\x16$\n\r\
    \n\x05\x04\x1b\x02\x03\x03\x12\x04\xf5\x02'(\n\x1f\n\x04\x04\x1b\x02\x04\
    \x12\x04\xf6\x02\x089\"\x11\x20device\x20language\n\n\r\n\x05\x04\x1b\
    \x02\x04\x04\x12\x04\xf6\x02\x08\x10\n\r\n\x05\x04\x1b\x02\x04\x05\x12\
    \x04\xf6\x02\x11\x17\n\r\n\x05\x04\x1b\x02\x04\x01\x12\x04\xf6\x02\x18\
    \x20\n\r\n\x05\x04\x1b\x02\x04\x03\x12\x04\xf6\x02#$\n\r\n\x05\x04\x1b\
    \x02\x04\x08\x12\x04\xf6\x02%8\n\r\n\x05\x04\x1b\x02\x04\x07\x12\x04\xf6\
    \x02.7\n\x1c\n\x04\x04\x1b\x02\x05\x12\x04\xf7\x02\x08\"\"\x0e\x20device\
    \x20label\n\n\r\n\x05\x04\x1b\x02\x05\x04\x12\x04\xf7\x02\x08\x10\n\r\n\
    \x05\x04\x1b\x02\x05\x05\x12\x04\xf7\x02\x11\x17\n\r\n\x05\x04\x1b\x02\
    \x05\x01\x12\x04\xf7\x02\x18\x1d\n\r\n\x05\x04\x1b\x02\x05\x03\x12\x04\
    \xf7\x02\x20!\n\x1b\n\x04\x04\x1b\x02\x06\x12\x04\xf8\x02\x08(\"\r\x20U2\
    F\x20counter\n\n\r\n\x05\x04\x1b\x02\x06\x04\x12\x04\xf8\x02\x08\x10\n\r\
    \n\x05\x04\x1b\x02\x06\x05\x12\x04\xf8\x02\x11\x17\n\r\n\x05\x04\x1b\x02\
    \x06\x01\x12\x04\xf8\x02\x18#\n\r\n\x05\x04\x1b\x02\x06\x03\x12\x04\xf8\
    \x02&'\n=\n\x04\x04\x1b\x02\x07\x12\x04\xf9\x02\x08&\"/\x20postpone\x20s\
    eed\x20backup\x20to\x20BackupDevice\x20workflow\n\n\r\n\x05\x04\x1b\x02\
    \x07\x04\x12\x04\xf9\x02\x08\x10\n\r\n\x05\x04\x1b\x02\x07\x05\x12\x04\
    \xf9\x02\x11\x15\n\r\n\x05\x04\x1b\x02\x07\x01\x12\x04\xf9\x02\x16!\n\r\
    \n\x05\x04\x1b\x02\x07\x03\x12\x04\xf9\x02$%\nt\n\x02\x04\x1c\x12\x06\
    \x80\x03\0\x81\x03\x01\x1af*\n\x20Request:\x20Perform\x20backup\x20of\
    \x20the\x20device\x20seed\x20if\x20not\x20backed\x20up\x20using\x20Reset\
    Device\n\x20@next\x20ButtonRequest\n\n\x0b\n\x03\x04\x1c\x01\x12\x04\x80\
    \x03\x08\x14\nn\n\x02\x04\x1d\x12\x06\x88\x03\0\x89\x03\x01\x1a`*\n\x20R\
    esponse:\x20Ask\x20for\x20additional\x20entropy\x20from\x20host\x20compu\
    ter\n\x20@prev\x20ResetDevice\n\x20@next\x20EntropyAck\n\n\x0b\n\x03\x04\
    \x1d\x01\x12\x04\x88\x03\x08\x16\n}\n\x02\x04\x1e\x12\x06\x90\x03\0\x92\
    \x03\x01\x1ao*\n\x20Request:\x20Provide\x20additional\x20entropy\x20for\
    \x20seed\x20generation\x20function\n\x20@prev\x20EntropyRequest\n\x20@ne\
    xt\x20ButtonRequest\n\n\x0b\n\x03\x04\x1e\x01\x12\x04\x90\x03\x08\x12\n2\
    \n\x04\x04\x1e\x02\0\x12\x04\x91\x03\x08#\"$\x20256\x20bits\x20(32\x20by\
    tes)\x20of\x20random\x20data\n\n\r\n\x05\x04\x1e\x02\0\x04\x12\x04\x91\
    \x03\x08\x10\n\r\n\x05\x04\x1e\x02\0\x05\x12\x04\x91\x03\x11\x16\n\r\n\
    \x05\x04\x1e\x02\0\x01\x12\x04\x91\x03\x17\x1e\n\r\n\x05\x04\x1e\x02\0\
    \x03\x12\x04\x91\x03!\"\n\xad\x01\n\x02\x04\x1f\x12\x06\x99\x03\0\xa4\
    \x03\x01\x1a\x9e\x01*\n\x20Request:\x20Start\x20recovery\x20workflow\x20\
    asking\x20user\x20for\x20specific\x20words\x20of\x20mnemonic\n\x20Used\
    \x20to\x20recovery\x20device\x20safely\x20even\x20on\x20untrusted\x20com\
    puter.\n\x20@next\x20WordRequest\n\n\x0b\n\x03\x04\x1f\x01\x12\x04\x99\
    \x03\x08\x16\n2\n\x04\x04\x1f\x02\0\x12\x04\x9a\x03\x08'\"$\x20number\
    \x20of\x20words\x20in\x20BIP-39\x20mnemonic\n\n\r\n\x05\x04\x1f\x02\0\
    \x04\x12\x04\x9a\x03\x08\x10\n\r\n\x05\x04\x1f\x02\0\x05\x12\x04\x9a\x03\
    \x11\x17\n\r\n\x05\x04\x1f\x02\0\x01\x12\x04\x9a\x03\x18\"\n\r\n\x05\x04\
    \x1f\x02\0\x03\x12\x04\x9a\x03%&\n>\n\x04\x04\x1f\x02\x01\x12\x04\x9b\
    \x03\x080\"0\x20enable\x20master\x20node\x20encryption\x20using\x20passp\
    hrase\n\n\r\n\x05\x04\x1f\x02\x01\x04\x12\x04\x9b\x03\x08\x10\n\r\n\x05\
    \x04\x1f\x02\x01\x05\x12\x04\x9b\x03\x11\x15\n\r\n\x05\x04\x1f\x02\x01\
    \x01\x12\x04\x9b\x03\x16+\n\r\n\x05\x04\x1f\x02\x01\x03\x12\x04\x9b\x03.\
    /\n%\n\x04\x04\x1f\x02\x02\x12\x04\x9c\x03\x08)\"\x17\x20enable\x20PIN\
    \x20protection\n\n\r\n\x05\x04\x1f\x02\x02\x04\x12\x04\x9c\x03\x08\x10\n\
    \r\n\x05\x04\x1f\x02\x02\x05\x12\x04\x9c\x03\x11\x15\n\r\n\x05\x04\x1f\
    \x02\x02\x01\x12\x04\x9c\x03\x16$\n\r\n\x05\x04\x1f\x02\x02\x03\x12\x04\
    \x9c\x03'(\n\x1f\n\x04\x04\x1f\x02\x03\x12\x04\x9d\x03\x089\"\x11\x20dev\
    ice\x20language\n\n\r\n\x05\x04\x1f\x02\x03\x04\x12\x04\x9d\x03\x08\x10\
    \n\r\n\x05\x04\x1f\x02\x03\x05\x12\x04\x9d\x03\x11\x17\n\r\n\x05\x04\x1f\
    \x02\x03\x01\x12\x04\x9d\x03\x18\x20\n\r\n\x05\x04\x1f\x02\x03\x03\x12\
    \x04\x9d\x03#$\n\r\n\x05\x04\x1f\x02\x03\x08\x12\x04\x9d\x03%8\n\r\n\x05\
    \x04\x1f\x02\x03\x07\x12\x04\x9d\x03.7\n\x1c\n\x04\x04\x1f\x02\x04\x12\
    \x04\x9e\x03\x08\"\"\x0e\x20device\x20label\n\n\r\n\x05\x04\x1f\x02\x04\
    \x04\x12\x04\x9e\x03\x08\x10\n\r\n\x05\x04\x1f\x02\x04\x05\x12\x04\x9e\
    \x03\x11\x17\n\r\n\x05\x04\x1f\x02\x04\x01\x12\x04\x9e\x03\x18\x1d\n\r\n\
    \x05\x04\x1f\x02\x04\x03\x12\x04\x9e\x03\x20!\n:\n\x04\x04\x1f\x02\x05\
    \x12\x04\x9f\x03\x08+\",\x20enforce\x20BIP-39\x20wordlist\x20during\x20t\
    he\x20process\n\n\r\n\x05\x04\x1f\x02\x05\x04\x12\x04\x9f\x03\x08\x10\n\
    \r\n\x05\x04\x1f\x02\x05\x05\x12\x04\x9f\x03\x11\x15\n\r\n\x05\x04\x1f\
    \x02\x05\x01\x12\x04\x9f\x03\x16&\n\r\n\x05\x04\x1f\x02\x05\x03\x12\x04\
    \x9f\x03)*\nc\n\x04\x04\x1f\x02\x06\x12\x04\xa1\x03\x08!\x1a'\x207\x20re\
    served\x20for\x20unused\x20recovery\x20method\n\",\x20supported\x20recov\
    ery\x20type\x20(see\x20RecoveryType)\n\n\r\n\x05\x04\x1f\x02\x06\x04\x12\
    \x04\xa1\x03\x08\x10\n\r\n\x05\x04\x1f\x02\x06\x05\x12\x04\xa1\x03\x11\
    \x17\n\r\n\x05\x04\x1f\x02\x06\x01\x12\x04\xa1\x03\x18\x1c\n\r\n\x05\x04\
    \x1f\x02\x06\x03\x12\x04\xa1\x03\x1f\x20\n\x1b\n\x04\x04\x1f\x02\x07\x12\
    \x04\xa2\x03\x08(\"\r\x20U2F\x20counter\n\n\r\n\x05\x04\x1f\x02\x07\x04\
    \x12\x04\xa2\x03\x08\x10\n\r\n\x05\x04\x1f\x02\x07\x05\x12\x04\xa2\x03\
    \x11\x17\n\r\n\x05\x04\x1f\x02\x07\x01\x12\x04\xa2\x03\x18#\n\r\n\x05\
    \x04\x1f\x02\x07\x03\x12\x04\xa2\x03&'\nP\n\x04\x04\x1f\x02\x08\x12\x04\
    \xa3\x03\x08#\"B\x20perform\x20dry-run\x20recovery\x20workflow\x20(for\
    \x20safe\x20mnemonic\x20validation)\n\n\r\n\x05\x04\x1f\x02\x08\x04\x12\
    \x04\xa3\x03\x08\x10\n\r\n\x05\x04\x1f\x02\x08\x05\x12\x04\xa3\x03\x11\
    \x15\n\r\n\x05\x04\x1f\x02\x08\x01\x12\x04\xa3\x03\x16\x1d\n\r\n\x05\x04\
    \x1f\x02\x08\x03\x12\x04\xa3\x03\x20\"\n\xb4\x01\n\x02\x04\x20\x12\x06\
    \xac\x03\0\xae\x03\x01\x1a\xa5\x01*\n\x20Response:\x20Device\x20is\x20wa\
    iting\x20for\x20user\x20to\x20enter\x20word\x20of\x20the\x20mnemonic\n\
    \x20Its\x20position\x20is\x20shown\x20only\x20on\x20device's\x20internal\
    \x20display.\n\x20@prev\x20RecoveryDevice\n\x20@prev\x20WordAck\n\n\x0b\
    \n\x03\x04\x20\x01\x12\x04\xac\x03\x08\x13\n\x0c\n\x04\x04\x20\x02\0\x12\
    \x04\xad\x03\x08*\n\r\n\x05\x04\x20\x02\0\x04\x12\x04\xad\x03\x08\x10\n\
    \r\n\x05\x04\x20\x02\0\x06\x12\x04\xad\x03\x11\x20\n\r\n\x05\x04\x20\x02\
    \0\x01\x12\x04\xad\x03!%\n\r\n\x05\x04\x20\x02\0\x03\x12\x04\xad\x03()\n\
    \x8b\x01\n\x02\x04!\x12\x06\xb7\x03\0\xb9\x03\x01\x1a}*\n\x20Request:\
    \x20Computer\x20replies\x20with\x20word\x20from\x20the\x20mnemonic\n\x20\
    @prev\x20WordRequest\n\x20@next\x20WordRequest\n\x20@next\x20Success\n\
    \x20@next\x20Failure\n\n\x0b\n\x03\x04!\x01\x12\x04\xb7\x03\x08\x0f\n6\n\
    \x04\x04!\x02\0\x12\x04\xb8\x03\x08!\"(\x20one\x20word\x20of\x20mnemonic\
    \x20on\x20asked\x20position\n\n\r\n\x05\x04!\x02\0\x04\x12\x04\xb8\x03\
    \x08\x10\n\r\n\x05\x04!\x02\0\x05\x12\x04\xb8\x03\x11\x17\n\r\n\x05\x04!\
    \x02\0\x01\x12\x04\xb8\x03\x18\x1c\n\r\n\x05\x04!\x02\0\x03\x12\x04\xb8\
    \x03\x1f\x20\n\xb5\x01\n\x02\x04\"\x12\x06\xc4\x03\0\xc9\x03\x01\x1aN*\n\
    \x20Request:\x20Ask\x20device\x20to\x20sign\x20message\n\x20@next\x20Mes\
    sageSignature\n\x20@next\x20Failure\n2W////////////////////////////\n\
    \x20Message\x20signing\x20messages\x20//\n////////////////////////////\n\
    \n\x0b\n\x03\x04\"\x01\x12\x04\xc4\x03\x08\x13\n>\n\x04\x04\"\x02\0\x12\
    \x04\xc5\x03\x08&\"0\x20BIP-32\x20path\x20to\x20derive\x20the\x20key\x20\
    from\x20master\x20node\n\n\r\n\x05\x04\"\x02\0\x04\x12\x04\xc5\x03\x08\
    \x10\n\r\n\x05\x04\"\x02\0\x05\x12\x04\xc5\x03\x11\x17\n\r\n\x05\x04\"\
    \x02\0\x01\x12\x04\xc5\x03\x18!\n\r\n\x05\x04\"\x02\0\x03\x12\x04\xc5\
    \x03$%\n$\n\x04\x04\"\x02\x01\x12\x04\xc6\x03\x08#\"\x16\x20message\x20t\
    o\x20be\x20signed\n\n\r\n\x05\x04\"\x02\x01\x04\x12\x04\xc6\x03\x08\x10\
    \n\r\n\x05\x04\"\x02\x01\x05\x12\x04\xc6\x03\x11\x16\n\r\n\x05\x04\"\x02\
    \x01\x01\x12\x04\xc6\x03\x17\x1e\n\r\n\x05\x04\"\x02\x01\x03\x12\x04\xc6\
    \x03!\"\n'\n\x04\x04\"\x02\x02\x12\x04\xc7\x03\x08:\"\x19\x20coin\x20to\
    \x20use\x20for\x20signing\n\n\r\n\x05\x04\"\x02\x02\x04\x12\x04\xc7\x03\
    \x08\x10\n\r\n\x05\x04\"\x02\x02\x05\x12\x04\xc7\x03\x11\x17\n\r\n\x05\
    \x04\"\x02\x02\x01\x12\x04\xc7\x03\x18!\n\r\n\x05\x04\"\x02\x02\x03\x12\
    \x04\xc7\x03$%\n\r\n\x05\x04\"\x02\x02\x08\x12\x04\xc7\x03&9\n\r\n\x05\
    \x04\"\x02\x02\x07\x12\x04\xc7\x03/8\n^\n\x04\x04\"\x02\x03\x12\x04\xc8\
    \x03\x08H\"P\x20used\x20to\x20distinguish\x20between\x20various\x20addre\
    ss\x20formats\x20(non-segwit,\x20segwit,\x20etc.)\n\n\r\n\x05\x04\"\x02\
    \x03\x04\x12\x04\xc8\x03\x08\x10\n\r\n\x05\x04\"\x02\x03\x06\x12\x04\xc8\
    \x03\x11\x20\n\r\n\x05\x04\"\x02\x03\x01\x12\x04\xc8\x03!,\n\r\n\x05\x04\
    \"\x02\x03\x03\x12\x04\xc8\x03/0\n\r\n\x05\x04\"\x02\x03\x08\x12\x04\xc8\
    \x031G\n\r\n\x05\x04\"\x02\x03\x07\x12\x04\xc8\x03:F\nU\n\x02\x04#\x12\
    \x06\xd0\x03\0\xd5\x03\x01\x1aG*\n\x20Request:\x20Ask\x20device\x20to\
    \x20verify\x20message\n\x20@next\x20Success\n\x20@next\x20Failure\n\n\
    \x0b\n\x03\x04#\x01\x12\x04\xd0\x03\x08\x15\n!\n\x04\x04#\x02\0\x12\x04\
    \xd1\x03\x08$\"\x13\x20address\x20to\x20verify\n\n\r\n\x05\x04#\x02\0\
    \x04\x12\x04\xd1\x03\x08\x10\n\r\n\x05\x04#\x02\0\x05\x12\x04\xd1\x03\
    \x11\x17\n\r\n\x05\x04#\x02\0\x01\x12\x04\xd1\x03\x18\x1f\n\r\n\x05\x04#\
    \x02\0\x03\x12\x04\xd1\x03\"#\n#\n\x04\x04#\x02\x01\x12\x04\xd2\x03\x08%\
    \"\x15\x20signature\x20to\x20verify\n\n\r\n\x05\x04#\x02\x01\x04\x12\x04\
    \xd2\x03\x08\x10\n\r\n\x05\x04#\x02\x01\x05\x12\x04\xd2\x03\x11\x16\n\r\
    \n\x05\x04#\x02\x01\x01\x12\x04\xd2\x03\x17\x20\n\r\n\x05\x04#\x02\x01\
    \x03\x12\x04\xd2\x03#$\n!\n\x04\x04#\x02\x02\x12\x04\xd3\x03\x08#\"\x13\
    \x20message\x20to\x20verify\n\n\r\n\x05\x04#\x02\x02\x04\x12\x04\xd3\x03\
    \x08\x10\n\r\n\x05\x04#\x02\x02\x05\x12\x04\xd3\x03\x11\x16\n\r\n\x05\
    \x04#\x02\x02\x01\x12\x04\xd3\x03\x17\x1e\n\r\n\x05\x04#\x02\x02\x03\x12\
    \x04\xd3\x03!\"\n)\n\x04\x04#\x02\x03\x12\x04\xd4\x03\x08:\"\x1b\x20coin\
    \x20to\x20use\x20for\x20verifying\n\n\r\n\x05\x04#\x02\x03\x04\x12\x04\
    \xd4\x03\x08\x10\n\r\n\x05\x04#\x02\x03\x05\x12\x04\xd4\x03\x11\x17\n\r\
    \n\x05\x04#\x02\x03\x01\x12\x04\xd4\x03\x18!\n\r\n\x05\x04#\x02\x03\x03\
    \x12\x04\xd4\x03$%\n\r\n\x05\x04#\x02\x03\x08\x12\x04\xd4\x03&9\n\r\n\
    \x05\x04#\x02\x03\x07\x12\x04\xd4\x03/8\n=\n\x02\x04$\x12\x06\xdb\x03\0\
    \xde\x03\x01\x1a/*\n\x20Response:\x20Signed\x20message\n\x20@prev\x20Sig\
    nMessage\n\n\x0b\n\x03\x04$\x01\x12\x04\xdb\x03\x08\x18\n0\n\x04\x04$\
    \x02\0\x12\x04\xdc\x03\x08$\"\"\x20address\x20used\x20to\x20sign\x20the\
    \x20message\n\n\r\n\x05\x04$\x02\0\x04\x12\x04\xdc\x03\x08\x10\n\r\n\x05\
    \x04$\x02\0\x05\x12\x04\xdc\x03\x11\x17\n\r\n\x05\x04$\x02\0\x01\x12\x04\
    \xdc\x03\x18\x1f\n\r\n\x05\x04$\x02\0\x03\x12\x04\xdc\x03\"#\n(\n\x04\
    \x04$\x02\x01\x12\x04\xdd\x03\x08%\"\x1a\x20signature\x20of\x20the\x20me\
    ssage\n\n\r\n\x05\x04$\x02\x01\x04\x12\x04\xdd\x03\x08\x10\n\r\n\x05\x04\
    $\x02\x01\x05\x12\x04\xdd\x03\x11\x16\n\r\n\x05\x04$\x02\x01\x01\x12\x04\
    \xdd\x03\x17\x20\n\r\n\x05\x04$\x02\x01\x03\x12\x04\xdd\x03#$\n\xaf\x01\
    \n\x02\x04%\x12\x06\xe9\x03\0\xef\x03\x01\x1aQ*\n\x20Request:\x20Ask\x20\
    device\x20to\x20encrypt\x20message\n\x20@next\x20EncryptedMessage\n\x20@\
    next\x20Failure\n2N/////////////////////////\n\x20Encryption/decryption\
    \x20//\n/////////////////////////\n\n\x0b\n\x03\x04%\x01\x12\x04\xe9\x03\
    \x08\x16\n\x1a\n\x04\x04%\x02\0\x12\x04\xea\x03\x08\"\"\x0c\x20public\
    \x20key\n\n\r\n\x05\x04%\x02\0\x04\x12\x04\xea\x03\x08\x10\n\r\n\x05\x04\
    %\x02\0\x05\x12\x04\xea\x03\x11\x16\n\r\n\x05\x04%\x02\0\x01\x12\x04\xea\
    \x03\x17\x1d\n\r\n\x05\x04%\x02\0\x03\x12\x04\xea\x03\x20!\n\"\n\x04\x04\
    %\x02\x01\x12\x04\xeb\x03\x08#\"\x14\x20message\x20to\x20encrypt\n\n\r\n\
    \x05\x04%\x02\x01\x04\x12\x04\xeb\x03\x08\x10\n\r\n\x05\x04%\x02\x01\x05\
    \x12\x04\xeb\x03\x11\x16\n\r\n\x05\x04%\x02\x01\x01\x12\x04\xeb\x03\x17\
    \x1e\n\r\n\x05\x04%\x02\x01\x03\x12\x04\xeb\x03!\"\n@\n\x04\x04%\x02\x02\
    \x12\x04\xec\x03\x08'\"2\x20show\x20just\x20on\x20display?\x20(don't\x20\
    send\x20back\x20via\x20wire)\n\n\r\n\x05\x04%\x02\x02\x04\x12\x04\xec\
    \x03\x08\x10\n\r\n\x05\x04%\x02\x02\x05\x12\x04\xec\x03\x11\x15\n\r\n\
    \x05\x04%\x02\x02\x01\x12\x04\xec\x03\x16\"\n\r\n\x05\x04%\x02\x02\x03\
    \x12\x04\xec\x03%&\nF\n\x04\x04%\x02\x03\x12\x04\xed\x03\x08&\"8\x20BIP-\
    32\x20path\x20to\x20derive\x20the\x20signing\x20key\x20from\x20master\
    \x20node\n\n\r\n\x05\x04%\x02\x03\x04\x12\x04\xed\x03\x08\x10\n\r\n\x05\
    \x04%\x02\x03\x05\x12\x04\xed\x03\x11\x17\n\r\n\x05\x04%\x02\x03\x01\x12\
    \x04\xed\x03\x18!\n\r\n\x05\x04%\x02\x03\x03\x12\x04\xed\x03$%\n'\n\x04\
    \x04%\x02\x04\x12\x04\xee\x03\x08:\"\x19\x20coin\x20to\x20use\x20for\x20\
    signing\n\n\r\n\x05\x04%\x02\x04\x04\x12\x04\xee\x03\x08\x10\n\r\n\x05\
    \x04%\x02\x04\x05\x12\x04\xee\x03\x11\x17\n\r\n\x05\x04%\x02\x04\x01\x12\
    \x04\xee\x03\x18!\n\r\n\x05\x04%\x02\x04\x03\x12\x04\xee\x03$%\n\r\n\x05\
    \x04%\x02\x04\x08\x12\x04\xee\x03&9\n\r\n\x05\x04%\x02\x04\x07\x12\x04\
    \xee\x03/8\nC\n\x02\x04&\x12\x06\xf5\x03\0\xf9\x03\x01\x1a5*\n\x20Respon\
    se:\x20Encrypted\x20message\n\x20@prev\x20EncryptMessage\n\n\x0b\n\x03\
    \x04&\x01\x12\x04\xf5\x03\x08\x18\n,\n\x04\x04&\x02\0\x12\x04\xf6\x03\
    \x08!\"\x1e\x20nonce\x20used\x20during\x20encryption\n\n\r\n\x05\x04&\
    \x02\0\x04\x12\x04\xf6\x03\x08\x10\n\r\n\x05\x04&\x02\0\x05\x12\x04\xf6\
    \x03\x11\x16\n\r\n\x05\x04&\x02\0\x01\x12\x04\xf6\x03\x17\x1c\n\r\n\x05\
    \x04&\x02\0\x03\x12\x04\xf6\x03\x1f\x20\n!\n\x04\x04&\x02\x01\x12\x04\
    \xf7\x03\x08#\"\x13\x20encrypted\x20message\n\n\r\n\x05\x04&\x02\x01\x04\
    \x12\x04\xf7\x03\x08\x10\n\r\n\x05\x04&\x02\x01\x05\x12\x04\xf7\x03\x11\
    \x16\n\r\n\x05\x04&\x02\x01\x01\x12\x04\xf7\x03\x17\x1e\n\r\n\x05\x04&\
    \x02\x01\x03\x12\x04\xf7\x03!\"\n\x1c\n\x04\x04&\x02\x02\x12\x04\xf8\x03\
    \x08\x20\"\x0e\x20message\x20hmac\n\n\r\n\x05\x04&\x02\x02\x04\x12\x04\
    \xf8\x03\x08\x10\n\r\n\x05\x04&\x02\x02\x05\x12\x04\xf8\x03\x11\x16\n\r\
    \n\x05\x04&\x02\x02\x01\x12\x04\xf8\x03\x17\x1b\n\r\n\x05\x04&\x02\x02\
    \x03\x12\x04\xf8\x03\x1e\x1f\nV\n\x02\x04'\x12\x06\x80\x04\0\x85\x04\x01\
    \x1aH*\n\x20Request:\x20Ask\x20device\x20to\x20decrypt\x20message\n\x20@\
    next\x20Success\n\x20@next\x20Failure\n\n\x0b\n\x03\x04'\x01\x12\x04\x80\
    \x04\x08\x16\nI\n\x04\x04'\x02\0\x12\x04\x81\x04\x08&\";\x20BIP-32\x20pa\
    th\x20to\x20derive\x20the\x20decryption\x20key\x20from\x20master\x20node\
    \n\n\r\n\x05\x04'\x02\0\x04\x12\x04\x81\x04\x08\x10\n\r\n\x05\x04'\x02\0\
    \x05\x12\x04\x81\x04\x11\x17\n\r\n\x05\x04'\x02\0\x01\x12\x04\x81\x04\
    \x18!\n\r\n\x05\x04'\x02\0\x03\x12\x04\x81\x04$%\n,\n\x04\x04'\x02\x01\
    \x12\x04\x82\x04\x08!\"\x1e\x20nonce\x20used\x20during\x20encryption\n\n\
    \r\n\x05\x04'\x02\x01\x04\x12\x04\x82\x04\x08\x10\n\r\n\x05\x04'\x02\x01\
    \x05\x12\x04\x82\x04\x11\x16\n\r\n\x05\x04'\x02\x01\x01\x12\x04\x82\x04\
    \x17\x1c\n\r\n\x05\x04'\x02\x01\x03\x12\x04\x82\x04\x1f\x20\n\"\n\x04\
    \x04'\x02\x02\x12\x04\x83\x04\x08#\"\x14\x20message\x20to\x20decrypt\n\n\
    \r\n\x05\x04'\x02\x02\x04\x12\x04\x83\x04\x08\x10\n\r\n\x05\x04'\x02\x02\
    \x05\x12\x04\x83\x04\x11\x16\n\r\n\x05\x04'\x02\x02\x01\x12\x04\x83\x04\
    \x17\x1e\n\r\n\x05\x04'\x02\x02\x03\x12\x04\x83\x04!\"\n\x1c\n\x04\x04'\
    \x02\x03\x12\x04\x84\x04\x08\x20\"\x0e\x20message\x20hmac\n\n\r\n\x05\
    \x04'\x02\x03\x04\x12\x04\x84\x04\x08\x10\n\r\n\x05\x04'\x02\x03\x05\x12\
    \x04\x84\x04\x11\x16\n\r\n\x05\x04'\x02\x03\x01\x12\x04\x84\x04\x17\x1b\
    \n\r\n\x05\x04'\x02\x03\x03\x12\x04\x84\x04\x1e\x1f\nE\n\x02\x04(\x12\
    \x06\x8b\x04\0\x8e\x04\x01\x1a7*\n\x20Response:\x20Decrypted\x20message\
    \n\x20@prev\x20DecryptedMessage\n\n\x0b\n\x03\x04(\x01\x12\x04\x8b\x04\
    \x08\x18\n!\n\x04\x04(\x02\0\x12\x04\x8c\x04\x08#\"\x13\x20decrypted\x20\
    message\n\n\r\n\x05\x04(\x02\0\x04\x12\x04\x8c\x04\x08\x10\n\r\n\x05\x04\
    (\x02\0\x05\x12\x04\x8c\x04\x11\x16\n\r\n\x05\x04(\x02\0\x01\x12\x04\x8c\
    \x04\x17\x1e\n\r\n\x05\x04(\x02\0\x03\x12\x04\x8c\x04!\"\n:\n\x04\x04(\
    \x02\x01\x12\x04\x8d\x04\x08$\",\x20address\x20used\x20to\x20sign\x20the\
    \x20message\x20(if\x20used)\n\n\r\n\x05\x04(\x02\x01\x04\x12\x04\x8d\x04\
    \x08\x10\n\r\n\x05\x04(\x02\x01\x05\x12\x04\x8d\x04\x11\x17\n\r\n\x05\
    \x04(\x02\x01\x01\x12\x04\x8d\x04\x18\x1f\n\r\n\x05\x04(\x02\x01\x03\x12\
    \x04\x8d\x04\"#\nu\n\x02\x04)\x12\x06\x95\x04\0\x9d\x04\x01\x1ag*\n\x20R\
    equest:\x20Ask\x20device\x20to\x20encrypt\x20or\x20decrypt\x20value\x20o\
    f\x20given\x20key\n\x20@next\x20CipheredKeyValue\n\x20@next\x20Failure\n\
    \n\x0b\n\x03\x04)\x01\x12\x04\x95\x04\x08\x16\n>\n\x04\x04)\x02\0\x12\
    \x04\x96\x04\x08&\"0\x20BIP-32\x20path\x20to\x20derive\x20the\x20key\x20\
    from\x20master\x20node\n\n\r\n\x05\x04)\x02\0\x04\x12\x04\x96\x04\x08\
    \x10\n\r\n\x05\x04)\x02\0\x05\x12\x04\x96\x04\x11\x17\n\r\n\x05\x04)\x02\
    \0\x01\x12\x04\x96\x04\x18!\n\r\n\x05\x04)\x02\0\x03\x12\x04\x96\x04$%\n\
    *\n\x04\x04)\x02\x01\x12\x04\x97\x04\x08\x20\"\x1c\x20key\x20component\
    \x20of\x20key:value\n\n\r\n\x05\x04)\x02\x01\x04\x12\x04\x97\x04\x08\x10\
    \n\r\n\x05\x04)\x02\x01\x05\x12\x04\x97\x04\x11\x17\n\r\n\x05\x04)\x02\
    \x01\x01\x12\x04\x97\x04\x18\x1b\n\r\n\x05\x04)\x02\x01\x03\x12\x04\x97\
    \x04\x1e\x1f\n,\n\x04\x04)\x02\x02\x12\x04\x98\x04\x08!\"\x1e\x20value\
    \x20component\x20of\x20key:value\n\n\r\n\x05\x04)\x02\x02\x04\x12\x04\
    \x98\x04\x08\x10\n\r\n\x05\x04)\x02\x02\x05\x12\x04\x98\x04\x11\x16\n\r\
    \n\x05\x04)\x02\x02\x01\x12\x04\x98\x04\x17\x1c\n\r\n\x05\x04)\x02\x02\
    \x03\x12\x04\x98\x04\x1f\x20\n?\n\x04\x04)\x02\x03\x12\x04\x99\x04\x08\"\
    \"1\x20are\x20we\x20encrypting\x20(True)\x20or\x20decrypting\x20(False)?\
    \n\n\r\n\x05\x04)\x02\x03\x04\x12\x04\x99\x04\x08\x10\n\r\n\x05\x04)\x02\
    \x03\x05\x12\x04\x99\x04\x11\x15\n\r\n\x05\x04)\x02\x03\x01\x12\x04\x99\
    \x04\x16\x1d\n\r\n\x05\x04)\x02\x03\x03\x12\x04\x99\x04\x20!\n3\n\x04\
    \x04)\x02\x04\x12\x04\x9a\x04\x08)\"%\x20should\x20we\x20ask\x20on\x20en\
    crypt\x20operation?\n\n\r\n\x05\x04)\x02\x04\x04\x12\x04\x9a\x04\x08\x10\
    \n\r\n\x05\x04)\x02\x04\x05\x12\x04\x9a\x04\x11\x15\n\r\n\x05\x04)\x02\
    \x04\x01\x12\x04\x9a\x04\x16$\n\r\n\x05\x04)\x02\x04\x03\x12\x04\x9a\x04\
    '(\n3\n\x04\x04)\x02\x05\x12\x04\x9b\x04\x08)\"%\x20should\x20we\x20ask\
    \x20on\x20decrypt\x20operation?\n\n\r\n\x05\x04)\x02\x05\x04\x12\x04\x9b\
    \x04\x08\x10\n\r\n\x05\x04)\x02\x05\x05\x12\x04\x9b\x04\x11\x15\n\r\n\
    \x05\x04)\x02\x05\x01\x12\x04\x9b\x04\x16$\n\r\n\x05\x04)\x02\x05\x03\
    \x12\x04\x9b\x04'(\nC\n\x04\x04)\x02\x06\x12\x04\x9c\x04\x08\x1e\"5\x20i\
    nitialization\x20vector\x20(will\x20be\x20computed\x20if\x20not\x20set)\
    \n\n\r\n\x05\x04)\x02\x06\x04\x12\x04\x9c\x04\x08\x10\n\r\n\x05\x04)\x02\
    \x06\x05\x12\x04\x9c\x04\x11\x16\n\r\n\x05\x04)\x02\x06\x01\x12\x04\x9c\
    \x04\x17\x19\n\r\n\x05\x04)\x02\x06\x03\x12\x04\x9c\x04\x1c\x1d\nR\n\x02\
    \x04*\x12\x06\xa3\x04\0\xa5\x04\x01\x1aD*\n\x20Response:\x20Return\x20ci\
    phered/deciphered\x20value\n\x20@prev\x20CipherKeyValue\n\n\x0b\n\x03\
    \x04*\x01\x12\x04\xa3\x04\x08\x18\n)\n\x04\x04*\x02\0\x12\x04\xa4\x04\
    \x08!\"\x1b\x20ciphered/deciphered\x20value\n\n\r\n\x05\x04*\x02\0\x04\
    \x12\x04\xa4\x04\x08\x10\n\r\n\x05\x04*\x02\0\x05\x12\x04\xa4\x04\x11\
    \x16\n\r\n\x05\x04*\x02\0\x01\x12\x04\xa4\x04\x17\x1c\n\r\n\x05\x04*\x02\
    \0\x03\x12\x04\xa4\x04\x1f\x20\n\xe0\x02\n\x02\x04+\x12\x06\xb2\x04\0\
    \xb6\x04\x01\x1a\xec\x01*\n\x20Request:\x20Estimated\x20size\x20of\x20th\
    e\x20transaction\n\x20This\x20behaves\x20exactly\x20like\x20SignTx,\x20w\
    hich\x20means\x20that\x20it\x20can\x20ask\x20using\x20TxRequest\n\x20Thi\
    s\x20call\x20is\x20non-blocking\x20(except\x20possible\x20PassphraseRequ\
    est\x20to\x20unlock\x20the\x20seed)\n\x20@next\x20TxSize\n\x20@next\x20F\
    ailure\n2c////////////////////////////////\n\x20Transaction\x20signing\
    \x20messages\x20//\n////////////////////////////////\n\n\x0b\n\x03\x04+\
    \x01\x12\x04\xb2\x04\x08\x16\n-\n\x04\x04+\x02\0\x12\x04\xb3\x04\x08*\"\
    \x1f\x20number\x20of\x20transaction\x20outputs\n\n\r\n\x05\x04+\x02\0\
    \x04\x12\x04\xb3\x04\x08\x10\n\r\n\x05\x04+\x02\0\x05\x12\x04\xb3\x04\
    \x11\x17\n\r\n\x05\x04+\x02\0\x01\x12\x04\xb3\x04\x18%\n\r\n\x05\x04+\
    \x02\0\x03\x12\x04\xb3\x04()\n,\n\x04\x04+\x02\x01\x12\x04\xb4\x04\x08)\
    \"\x1e\x20number\x20of\x20transaction\x20inputs\n\n\r\n\x05\x04+\x02\x01\
    \x04\x12\x04\xb4\x04\x08\x10\n\r\n\x05\x04+\x02\x01\x05\x12\x04\xb4\x04\
    \x11\x17\n\r\n\x05\x04+\x02\x01\x01\x12\x04\xb4\x04\x18$\n\r\n\x05\x04+\
    \x02\x01\x03\x12\x04\xb4\x04'(\n\x1b\n\x04\x04+\x02\x02\x12\x04\xb5\x04\
    \x08:\"\r\x20coin\x20to\x20use\n\n\r\n\x05\x04+\x02\x02\x04\x12\x04\xb5\
    \x04\x08\x10\n\r\n\x05\x04+\x02\x02\x05\x12\x04\xb5\x04\x11\x17\n\r\n\
    \x05\x04+\x02\x02\x01\x12\x04\xb5\x04\x18!\n\r\n\x05\x04+\x02\x02\x03\
    \x12\x04\xb5\x04$%\n\r\n\x05\x04+\x02\x02\x08\x12\x04\xb5\x04&9\n\r\n\
    \x05\x04+\x02\x02\x07\x12\x04\xb5\x04/8\nS\n\x02\x04,\x12\x06\xbc\x04\0\
    \xbe\x04\x01\x1aE*\n\x20Response:\x20Estimated\x20size\x20of\x20the\x20t\
    ransaction\n\x20@prev\x20EstimateTxSize\n\n\x0b\n\x03\x04,\x01\x12\x04\
    \xbc\x04\x08\x0e\n6\n\x04\x04,\x02\0\x12\x04\xbd\x04\x08$\"(\x20estimate\
    d\x20size\x20of\x20transaction\x20in\x20bytes\n\n\r\n\x05\x04,\x02\0\x04\
    \x12\x04\xbd\x04\x08\x10\n\r\n\x05\x04,\x02\0\x05\x12\x04\xbd\x04\x11\
    \x17\n\r\n\x05\x04,\x02\0\x01\x12\x04\xbd\x04\x18\x1f\n\r\n\x05\x04,\x02\
    \0\x03\x12\x04\xbd\x04\"#\n\x8a\x01\n\x02\x04-\x12\x06\xc7\x04\0\xcd\x04\
    \x01\x1a|*\n\x20Request:\x20Ask\x20device\x20to\x20sign\x20transaction\n\
    \x20@next\x20PassphraseRequest\n\x20@next\x20PinMatrixRequest\n\x20@next\
    \x20TxRequest\n\x20@next\x20Failure\n\n\x0b\n\x03\x04-\x01\x12\x04\xc7\
    \x04\x08\x0e\n-\n\x04\x04-\x02\0\x12\x04\xc8\x04\x08*\"\x1f\x20number\
    \x20of\x20transaction\x20outputs\n\n\r\n\x05\x04-\x02\0\x04\x12\x04\xc8\
    \x04\x08\x10\n\r\n\x05\x04-\x02\0\x05\x12\x04\xc8\x04\x11\x17\n\r\n\x05\
    \x04-\x02\0\x01\x12\x04\xc8\x04\x18%\n\r\n\x05\x04-\x02\0\x03\x12\x04\
    \xc8\x04()\n,\n\x04\x04-\x02\x01\x12\x04\xc9\x04\x08)\"\x1e\x20number\
    \x20of\x20transaction\x20inputs\n\n\r\n\x05\x04-\x02\x01\x04\x12\x04\xc9\
    \x04\x08\x10\n\r\n\x05\x04-\x02\x01\x05\x12\x04\xc9\x04\x11\x17\n\r\n\
    \x05\x04-\x02\x01\x01\x12\x04\xc9\x04\x18$\n\r\n\x05\x04-\x02\x01\x03\
    \x12\x04\xc9\x04'(\n\x1b\n\x04\x04-\x02\x02\x12\x04\xca\x04\x08:\"\r\x20\
    coin\x20to\x20use\n\n\r\n\x05\x04-\x02\x02\x04\x12\x04\xca\x04\x08\x10\n\
    \r\n\x05\x04-\x02\x02\x05\x12\x04\xca\x04\x11\x17\n\r\n\x05\x04-\x02\x02\
    \x01\x12\x04\xca\x04\x18!\n\r\n\x05\x04-\x02\x02\x03\x12\x04\xca\x04$%\n\
    \r\n\x05\x04-\x02\x02\x08\x12\x04\xca\x04&9\n\r\n\x05\x04-\x02\x02\x07\
    \x12\x04\xca\x04/8\n#\n\x04\x04-\x02\x03\x12\x04\xcb\x04\x080\"\x15\x20t\
    ransaction\x20version\n\n\r\n\x05\x04-\x02\x03\x04\x12\x04\xcb\x04\x08\
    \x10\n\r\n\x05\x04-\x02\x03\x05\x12\x04\xcb\x04\x11\x17\n\r\n\x05\x04-\
    \x02\x03\x01\x12\x04\xcb\x04\x18\x1f\n\r\n\x05\x04-\x02\x03\x03\x12\x04\
    \xcb\x04\"#\n\r\n\x05\x04-\x02\x03\x08\x12\x04\xcb\x04$/\n\r\n\x05\x04-\
    \x02\x03\x07\x12\x04\xcb\x04-.\n%\n\x04\x04-\x02\x04\x12\x04\xcc\x04\x08\
    2\"\x17\x20transaction\x20lock_time\n\n\r\n\x05\x04-\x02\x04\x04\x12\x04\
    \xcc\x04\x08\x10\n\r\n\x05\x04-\x02\x04\x05\x12\x04\xcc\x04\x11\x17\n\r\
    \n\x05\x04-\x02\x04\x01\x12\x04\xcc\x04\x18!\n\r\n\x05\x04-\x02\x04\x03\
    \x12\x04\xcc\x04$%\n\r\n\x05\x04-\x02\x04\x08\x12\x04\xcc\x04&1\n\r\n\
    \x05\x04-\x02\x04\x07\x12\x04\xcc\x04/0\n\xb6\x02\n\x02\x04.\x12\x06\xd8\
    \x04\0\xdf\x04\x01\x1a\xa7\x02*\n\x20Request:\x20Simplified\x20transacti\
    on\x20signing\n\x20This\x20method\x20doesn't\x20support\x20streaming,\
    \x20so\x20there\x20are\x20hardware\x20limits\x20in\x20number\x20of\x20in\
    puts\x20and\x20outputs.\n\x20In\x20case\x20of\x20success,\x20the\x20resu\
    lt\x20is\x20returned\x20using\x20TxRequest\x20message.\n\x20@next\x20Pas\
    sphraseRequest\n\x20@next\x20PinMatrixRequest\n\x20@next\x20TxRequest\n\
    \x20@next\x20Failure\n\n\x0b\n\x03\x04.\x01\x12\x04\xd8\x04\x08\x14\n\"\
    \n\x04\x04.\x02\0\x12\x04\xd9\x04\x08(\"\x14\x20transaction\x20inputs\n\
    \n\r\n\x05\x04.\x02\0\x04\x12\x04\xd9\x04\x08\x10\n\r\n\x05\x04.\x02\0\
    \x06\x12\x04\xd9\x04\x11\x1c\n\r\n\x05\x04.\x02\0\x01\x12\x04\xd9\x04\
    \x1d#\n\r\n\x05\x04.\x02\0\x03\x12\x04\xd9\x04&'\n#\n\x04\x04.\x02\x01\
    \x12\x04\xda\x04\x08*\"\x15\x20transaction\x20outputs\n\n\r\n\x05\x04.\
    \x02\x01\x04\x12\x04\xda\x04\x08\x10\n\r\n\x05\x04.\x02\x01\x06\x12\x04\
    \xda\x04\x11\x1d\n\r\n\x05\x04.\x02\x01\x01\x12\x04\xda\x04\x1e%\n\r\n\
    \x05\x04.\x02\x01\x03\x12\x04\xda\x04()\nK\n\x04\x04.\x02\x02\x12\x04\
    \xdb\x04\x082\"=\x20transactions\x20whose\x20outputs\x20are\x20used\x20t\
    o\x20build\x20current\x20inputs\n\n\r\n\x05\x04.\x02\x02\x04\x12\x04\xdb\
    \x04\x08\x10\n\r\n\x05\x04.\x02\x02\x06\x12\x04\xdb\x04\x11\x20\n\r\n\
    \x05\x04.\x02\x02\x01\x12\x04\xdb\x04!-\n\r\n\x05\x04.\x02\x02\x03\x12\
    \x04\xdb\x0401\n\x1b\n\x04\x04.\x02\x03\x12\x04\xdc\x04\x08:\"\r\x20coin\
    \x20to\x20use\n\n\r\n\x05\x04.\x02\x03\x04\x12\x04\xdc\x04\x08\x10\n\r\n\
    \x05\x04.\x02\x03\x05\x12\x04\xdc\x04\x11\x17\n\r\n\x05\x04.\x02\x03\x01\
    \x12\x04\xdc\x04\x18!\n\r\n\x05\x04.\x02\x03\x03\x12\x04\xdc\x04$%\n\r\n\
    \x05\x04.\x02\x03\x08\x12\x04\xdc\x04&9\n\r\n\x05\x04.\x02\x03\x07\x12\
    \x04\xdc\x04/8\n#\n\x04\x04.\x02\x04\x12\x04\xdd\x04\x080\"\x15\x20trans\
    action\x20version\n\n\r\n\x05\x04.\x02\x04\x04\x12\x04\xdd\x04\x08\x10\n\
    \r\n\x05\x04.\x02\x04\x05\x12\x04\xdd\x04\x11\x17\n\r\n\x05\x04.\x02\x04\
    \x01\x12\x04\xdd\x04\x18\x1f\n\r\n\x05\x04.\x02\x04\x03\x12\x04\xdd\x04\
    \"#\n\r\n\x05\x04.\x02\x04\x08\x12\x04\xdd\x04$/\n\r\n\x05\x04.\x02\x04\
    \x07\x12\x04\xdd\x04-.\n%\n\x04\x04.\x02\x05\x12\x04\xde\x04\x082\"\x17\
    \x20transaction\x20lock_time\n\n\r\n\x05\x04.\x02\x05\x04\x12\x04\xde\
    \x04\x08\x10\n\r\n\x05\x04.\x02\x05\x05\x12\x04\xde\x04\x11\x17\n\r\n\
    \x05\x04.\x02\x05\x01\x12\x04\xde\x04\x18!\n\r\n\x05\x04.\x02\x05\x03\
    \x12\x04\xde\x04$%\n\r\n\x05\x04.\x02\x05\x08\x12\x04\xde\x04&1\n\r\n\
    \x05\x04.\x02\x05\x07\x12\x04\xde\x04/0\n\xdb\x02\n\x02\x04/\x12\x06\xe9\
    \x04\0\xed\x04\x01\x1a\xcc\x02*\n\x20Response:\x20Device\x20asks\x20for\
    \x20information\x20for\x20signing\x20transaction\x20or\x20returns\x20the\
    \x20last\x20result\n\x20If\x20request_index\x20is\x20set,\x20device\x20a\
    waits\x20TxAck\x20message\x20(with\x20fields\x20filled\x20in\x20accordin\
    g\x20to\x20request_type)\n\x20If\x20signature_index\x20is\x20set,\x20'si\
    gnature'\x20contains\x20signed\x20input\x20of\x20signature_index's\x20in\
    put\n\x20@prev\x20SignTx\n\x20@prev\x20SimpleSignTx\n\x20@prev\x20TxAck\
    \n\n\x0b\n\x03\x04/\x01\x12\x04\xe9\x04\x08\x11\n7\n\x04\x04/\x02\0\x12\
    \x04\xea\x04\x08.\")\x20what\x20should\x20be\x20filled\x20in\x20TxAck\
    \x20message?\n\n\r\n\x05\x04/\x02\0\x04\x12\x04\xea\x04\x08\x10\n\r\n\
    \x05\x04/\x02\0\x06\x12\x04\xea\x04\x11\x1c\n\r\n\x05\x04/\x02\0\x01\x12\
    \x04\xea\x04\x1d)\n\r\n\x05\x04/\x02\0\x03\x12\x04\xea\x04,-\n&\n\x04\
    \x04/\x02\x01\x12\x04\xeb\x04\x082\"\x18\x20request\x20for\x20tx\x20deta\
    ils\n\n\r\n\x05\x04/\x02\x01\x04\x12\x04\xeb\x04\x08\x10\n\r\n\x05\x04/\
    \x02\x01\x06\x12\x04\xeb\x04\x11%\n\r\n\x05\x04/\x02\x01\x01\x12\x04\xeb\
    \x04&-\n\r\n\x05\x04/\x02\x01\x03\x12\x04\xeb\x0401\n4\n\x04\x04/\x02\
    \x02\x12\x04\xec\x04\x088\"&\x20serialized\x20data\x20and\x20request\x20\
    for\x20next\n\n\r\n\x05\x04/\x02\x02\x04\x12\x04\xec\x04\x08\x10\n\r\n\
    \x05\x04/\x02\x02\x06\x12\x04\xec\x04\x11(\n\r\n\x05\x04/\x02\x02\x01\
    \x12\x04\xec\x04)3\n\r\n\x05\x04/\x02\x02\x03\x12\x04\xec\x0467\nV\n\x02\
    \x040\x12\x06\xf4\x04\0\xf6\x04\x01\x1aH*\n\x20Request:\x20Reported\x20t\
    ransaction\x20data\n\x20@prev\x20TxRequest\n\x20@next\x20TxRequest\n\n\
    \x0b\n\x03\x040\x01\x12\x04\xf4\x04\x08\r\n\x0c\n\x04\x040\x02\0\x12\x04\
    \xf5\x04\x08(\n\r\n\x05\x040\x02\0\x04\x12\x04\xf5\x04\x08\x10\n\r\n\x05\
    \x040\x02\0\x06\x12\x04\xf5\x04\x11\x20\n\r\n\x05\x040\x02\0\x01\x12\x04\
    \xf5\x04!#\n\r\n\x05\x040\x02\0\x03\x12\x04\xf5\x04&'\n\xd5\x02\n\x02\
    \x041\x12\x06\x81\x05\0\x8b\x05\x01\x1a\xc6\x02*\n\x20Request:\x20Ask\
    \x20device\x20to\x20sign\x20transaction\n\x20All\x20fields\x20are\x20opt\
    ional\x20from\x20the\x20protocol's\x20point\x20of\x20view.\x20Each\x20fi\
    eld\x20defaults\x20to\x20value\x20`0`\x20if\x20missing.\n\x20Note:\x20th\
    e\x20first\x20at\x20most\x201024\x20bytes\x20of\x20data\x20MUST\x20be\
    \x20transmitted\x20as\x20part\x20of\x20this\x20message.\n\x20@next\x20Pa\
    ssphraseRequest\n\x20@next\x20PinMatrixRequest\n\x20@next\x20EthereumTxR\
    equest\n\x20@next\x20Failure\n\n\x0b\n\x03\x041\x01\x12\x04\x81\x05\x08\
    \x16\n>\n\x04\x041\x02\0\x12\x04\x82\x05\x08&\"0\x20BIP-32\x20path\x20to\
    \x20derive\x20the\x20key\x20from\x20master\x20node\n\n\r\n\x05\x041\x02\
    \0\x04\x12\x04\x82\x05\x08\x10\n\r\n\x05\x041\x02\0\x05\x12\x04\x82\x05\
    \x11\x17\n\r\n\x05\x041\x02\0\x01\x12\x04\x82\x05\x18!\n\r\n\x05\x041\
    \x02\0\x03\x12\x04\x82\x05$%\n-\n\x04\x041\x02\x01\x12\x04\x83\x05\x08!\
    \"\x1f\x20<=256\x20bit\x20unsigned\x20big\x20endian\n\n\r\n\x05\x041\x02\
    \x01\x04\x12\x04\x83\x05\x08\x10\n\r\n\x05\x041\x02\x01\x05\x12\x04\x83\
    \x05\x11\x16\n\r\n\x05\x041\x02\x01\x01\x12\x04\x83\x05\x17\x1c\n\r\n\
    \x05\x041\x02\x01\x03\x12\x04\x83\x05\x1f\x20\n6\n\x04\x041\x02\x02\x12\
    \x04\x84\x05\x08%\"(\x20<=256\x20bit\x20unsigned\x20big\x20endian\x20(in\
    \x20wei)\n\n\r\n\x05\x041\x02\x02\x04\x12\x04\x84\x05\x08\x10\n\r\n\x05\
    \x041\x02\x02\x05\x12\x04\x84\x05\x11\x16\n\r\n\x05\x041\x02\x02\x01\x12\
    \x04\x84\x05\x17\x20\n\r\n\x05\x041\x02\x02\x03\x12\x04\x84\x05#$\n-\n\
    \x04\x041\x02\x03\x12\x04\x85\x05\x08%\"\x1f\x20<=256\x20bit\x20unsigned\
    \x20big\x20endian\n\n\r\n\x05\x041\x02\x03\x04\x12\x04\x85\x05\x08\x10\n\
    \r\n\x05\x041\x02\x03\x05\x12\x04\x85\x05\x11\x16\n\r\n\x05\x041\x02\x03\
    \x01\x12\x04\x85\x05\x17\x20\n\r\n\x05\x041\x02\x03\x03\x12\x04\x85\x05#\
    $\n$\n\x04\x041\x02\x04\x12\x04\x86\x05\x08\x1e\"\x16\x20160\x20bit\x20a\
    ddress\x20hash\n\n\r\n\x05\x041\x02\x04\x04\x12\x04\x86\x05\x08\x10\n\r\
    \n\x05\x041\x02\x04\x05\x12\x04\x86\x05\x11\x16\n\r\n\x05\x041\x02\x04\
    \x01\x12\x04\x86\x05\x17\x19\n\r\n\x05\x041\x02\x04\x03\x12\x04\x86\x05\
    \x1c\x1d\n6\n\x04\x041\x02\x05\x12\x04\x87\x05\x08!\"(\x20<=256\x20bit\
    \x20unsigned\x20big\x20endian\x20(in\x20wei)\n\n\r\n\x05\x041\x02\x05\
    \x04\x12\x04\x87\x05\x08\x10\n\r\n\x05\x041\x02\x05\x05\x12\x04\x87\x05\
    \x11\x16\n\r\n\x05\x041\x02\x05\x01\x12\x04\x87\x05\x17\x1c\n\r\n\x05\
    \x041\x02\x05\x03\x12\x04\x87\x05\x1f\x20\n6\n\x04\x041\x02\x06\x12\x04\
    \x88\x05\x08.\"(\x20The\x20initial\x20data\x20chunk\x20(<=\x201024\x20by\
    tes)\n\n\r\n\x05\x041\x02\x06\x04\x12\x04\x88\x05\x08\x10\n\r\n\x05\x041\
    \x02\x06\x05\x12\x04\x88\x05\x11\x16\n\r\n\x05\x041\x02\x06\x01\x12\x04\
    \x88\x05\x17)\n\r\n\x05\x041\x02\x06\x03\x12\x04\x88\x05,-\n-\n\x04\x041\
    \x02\x07\x12\x04\x89\x05\x08(\"\x1f\x20Length\x20of\x20transaction\x20pa\
    yload\n\n\r\n\x05\x041\x02\x07\x04\x12\x04\x89\x05\x08\x10\n\r\n\x05\x04\
    1\x02\x07\x05\x12\x04\x89\x05\x11\x17\n\r\n\x05\x041\x02\x07\x01\x12\x04\
    \x89\x05\x18#\n\r\n\x05\x041\x02\x07\x03\x12\x04\x89\x05&'\n$\n\x04\x041\
    \x02\x08\x12\x04\x8a\x05\x08%\"\x16\x20Chain\x20Id\x20for\x20EIP\x20155\
    \n\n\r\n\x05\x041\x02\x08\x04\x12\x04\x8a\x05\x08\x10\n\r\n\x05\x041\x02\
    \x08\x05\x12\x04\x8a\x05\x11\x17\n\r\n\x05\x041\x02\x08\x01\x12\x04\x8a\
    \x05\x18\x20\n\r\n\x05\x041\x02\x08\x03\x12\x04\x8a\x05#$\n\xcd\x02\n\
    \x02\x042\x12\x06\x94\x05\0\x99\x05\x01\x1a\xbe\x02*\n\x20Response:\x20D\
    evice\x20asks\x20for\x20more\x20data\x20from\x20transaction\x20payload,\
    \x20or\x20returns\x20the\x20signature.\n\x20If\x20data_length\x20is\x20s\
    et,\x20device\x20awaits\x20that\x20many\x20more\x20bytes\x20of\x20payloa\
    d.\n\x20Otherwise,\x20the\x20signature_*\x20fields\x20contain\x20the\x20\
    computed\x20transaction\x20signature.\x20All\x20three\x20fields\x20will\
    \x20be\x20present.\n\x20@prev\x20EthereumSignTx\n\x20@next\x20EthereumTx\
    Ack\n\n\x0b\n\x03\x042\x01\x12\x04\x94\x05\x08\x19\n9\n\x04\x042\x02\0\
    \x12\x04\x95\x05\x08(\"+\x20Number\x20of\x20bytes\x20being\x20requested\
    \x20(<=\x201024)\n\n\r\n\x05\x042\x02\0\x04\x12\x04\x95\x05\x08\x10\n\r\
    \n\x05\x042\x02\0\x05\x12\x04\x95\x05\x11\x17\n\r\n\x05\x042\x02\0\x01\
    \x12\x04\x95\x05\x18#\n\r\n\x05\x042\x02\0\x03\x12\x04\x95\x05&'\nL\n\
    \x04\x042\x02\x01\x12\x04\x96\x05\x08(\">\x20Computed\x20signature\x20(r\
    ecovery\x20parameter,\x20limited\x20to\x2027\x20or\x2028)\n\n\r\n\x05\
    \x042\x02\x01\x04\x12\x04\x96\x05\x08\x10\n\r\n\x05\x042\x02\x01\x05\x12\
    \x04\x96\x05\x11\x17\n\r\n\x05\x042\x02\x01\x01\x12\x04\x96\x05\x18#\n\r\
    \n\x05\x042\x02\x01\x03\x12\x04\x96\x05&'\n8\n\x04\x042\x02\x02\x12\x04\
    \x97\x05\x08'\"*\x20Computed\x20signature\x20R\x20component\x20(256\x20b\
    it)\n\n\r\n\x05\x042\x02\x02\x04\x12\x04\x97\x05\x08\x10\n\r\n\x05\x042\
    \x02\x02\x05\x12\x04\x97\x05\x11\x16\n\r\n\x05\x042\x02\x02\x01\x12\x04\
    \x97\x05\x17\"\n\r\n\x05\x042\x02\x02\x03\x12\x04\x97\x05%&\n8\n\x04\x04\
    2\x02\x03\x12\x04\x98\x05\x08'\"*\x20Computed\x20signature\x20S\x20compo\
    nent\x20(256\x20bit)\n\n\r\n\x05\x042\x02\x03\x04\x12\x04\x98\x05\x08\
    \x10\n\r\n\x05\x042\x02\x03\x05\x12\x04\x98\x05\x11\x16\n\r\n\x05\x042\
    \x02\x03\x01\x12\x04\x98\x05\x17\"\n\r\n\x05\x042\x02\x03\x03\x12\x04\
    \x98\x05%&\nf\n\x02\x043\x12\x06\xa0\x05\0\xa2\x05\x01\x1aX*\n\x20Reques\
    t:\x20Transaction\x20payload\x20data.\n\x20@prev\x20EthereumTxRequest\n\
    \x20@next\x20EthereumTxRequest\n\n\x0b\n\x03\x043\x01\x12\x04\xa0\x05\
    \x08\x15\n>\n\x04\x043\x02\0\x12\x04\xa1\x05\x08&\"0\x20Bytes\x20from\
    \x20transaction\x20payload\x20(<=\x201024\x20bytes)\n\n\r\n\x05\x043\x02\
    \0\x04\x12\x04\xa1\x05\x08\x10\n\r\n\x05\x043\x02\0\x05\x12\x04\xa1\x05\
    \x11\x16\n\r\n\x05\x043\x02\0\x01\x12\x04\xa1\x05\x17!\n\r\n\x05\x043\
    \x02\0\x03\x12\x04\xa1\x05$%\n\xdb\x01\n\x02\x044\x12\x06\xad\x05\0\xb0\
    \x05\x01\x1aV*\n\x20Request:\x20Ask\x20device\x20to\x20sign\x20message\n\
    \x20@next\x20EthereumMessageSignature\n\x20@next\x20Failure\n2u/////////\
    /////////////////////////////\n\x20Ethereum:\x20Message\x20signing\x20me\
    ssages\x20//\n//////////////////////////////////////\n\n\x0b\n\x03\x044\
    \x01\x12\x04\xad\x05\x08\x1b\n>\n\x04\x044\x02\0\x12\x04\xae\x05\x08&\"0\
    \x20BIP-32\x20path\x20to\x20derive\x20the\x20key\x20from\x20master\x20no\
    de\n\n\r\n\x05\x044\x02\0\x04\x12\x04\xae\x05\x08\x10\n\r\n\x05\x044\x02\
    \0\x05\x12\x04\xae\x05\x11\x17\n\r\n\x05\x044\x02\0\x01\x12\x04\xae\x05\
    \x18!\n\r\n\x05\x044\x02\0\x03\x12\x04\xae\x05$%\n$\n\x04\x044\x02\x01\
    \x12\x04\xaf\x05\x08#\"\x16\x20message\x20to\x20be\x20signed\n\n\r\n\x05\
    \x044\x02\x01\x04\x12\x04\xaf\x05\x08\x10\n\r\n\x05\x044\x02\x01\x05\x12\
    \x04\xaf\x05\x11\x16\n\r\n\x05\x044\x02\x01\x01\x12\x04\xaf\x05\x17\x1e\
    \n\r\n\x05\x044\x02\x01\x03\x12\x04\xaf\x05!\"\nU\n\x02\x045\x12\x06\xb7\
    \x05\0\xbb\x05\x01\x1aG*\n\x20Request:\x20Ask\x20device\x20to\x20verify\
    \x20message\n\x20@next\x20Success\n\x20@next\x20Failure\n\n\x0b\n\x03\
    \x045\x01\x12\x04\xb7\x05\x08\x1d\n!\n\x04\x045\x02\0\x12\x04\xb8\x05\
    \x08#\"\x13\x20address\x20to\x20verify\n\n\r\n\x05\x045\x02\0\x04\x12\
    \x04\xb8\x05\x08\x10\n\r\n\x05\x045\x02\0\x05\x12\x04\xb8\x05\x11\x16\n\
    \r\n\x05\x045\x02\0\x01\x12\x04\xb8\x05\x17\x1e\n\r\n\x05\x045\x02\0\x03\
    \x12\x04\xb8\x05!\"\n#\n\x04\x045\x02\x01\x12\x04\xb9\x05\x08%\"\x15\x20\
    signature\x20to\x20verify\n\n\r\n\x05\x045\x02\x01\x04\x12\x04\xb9\x05\
    \x08\x10\n\r\n\x05\x045\x02\x01\x05\x12\x04\xb9\x05\x11\x16\n\r\n\x05\
    \x045\x02\x01\x01\x12\x04\xb9\x05\x17\x20\n\r\n\x05\x045\x02\x01\x03\x12\
    \x04\xb9\x05#$\n!\n\x04\x045\x02\x02\x12\x04\xba\x05\x08#\"\x13\x20messa\
    ge\x20to\x20verify\n\n\r\n\x05\x045\x02\x02\x04\x12\x04\xba\x05\x08\x10\
    \n\r\n\x05\x045\x02\x02\x05\x12\x04\xba\x05\x11\x16\n\r\n\x05\x045\x02\
    \x02\x01\x12\x04\xba\x05\x17\x1e\n\r\n\x05\x045\x02\x02\x03\x12\x04\xba\
    \x05!\"\nE\n\x02\x046\x12\x06\xc1\x05\0\xc4\x05\x01\x1a7*\n\x20Response:\
    \x20Signed\x20message\n\x20@prev\x20EthereumSignMessage\n\n\x0b\n\x03\
    \x046\x01\x12\x04\xc1\x05\x08\x20\n0\n\x04\x046\x02\0\x12\x04\xc2\x05\
    \x08#\"\"\x20address\x20used\x20to\x20sign\x20the\x20message\n\n\r\n\x05\
    \x046\x02\0\x04\x12\x04\xc2\x05\x08\x10\n\r\n\x05\x046\x02\0\x05\x12\x04\
    \xc2\x05\x11\x16\n\r\n\x05\x046\x02\0\x01\x12\x04\xc2\x05\x17\x1e\n\r\n\
    \x05\x046\x02\0\x03\x12\x04\xc2\x05!\"\n(\n\x04\x046\x02\x01\x12\x04\xc3\
    \x05\x08%\"\x1a\x20signature\x20of\x20the\x20message\n\n\r\n\x05\x046\
    \x02\x01\x04\x12\x04\xc3\x05\x08\x10\n\r\n\x05\x046\x02\x01\x05\x12\x04\
    \xc3\x05\x11\x16\n\r\n\x05\x046\x02\x01\x01\x12\x04\xc3\x05\x17\x20\n\r\
    \n\x05\x046\x02\x01\x03\x12\x04\xc3\x05#$\n\x9f\x01\n\x02\x047\x12\x06\
    \xcf\x05\0\xd4\x05\x01\x1aM*\n\x20Request:\x20Ask\x20device\x20to\x20sig\
    n\x20identity\n\x20@next\x20SignedIdentity\n\x20@next\x20Failure\n2B////\
    /////////////////\n\x20Identity\x20messages\x20//\n/////////////////////\
    \n\n\x0b\n\x03\x047\x01\x12\x04\xcf\x05\x08\x14\n\x18\n\x04\x047\x02\0\
    \x12\x04\xd0\x05\x08+\"\n\x20identity\n\n\r\n\x05\x047\x02\0\x04\x12\x04\
    \xd0\x05\x08\x10\n\r\n\x05\x047\x02\0\x06\x12\x04\xd0\x05\x11\x1d\n\r\n\
    \x05\x047\x02\0\x01\x12\x04\xd0\x05\x1e&\n\r\n\x05\x047\x02\0\x03\x12\
    \x04\xd0\x05)*\n%\n\x04\x047\x02\x01\x12\x04\xd1\x05\x08,\"\x17\x20non-v\
    isible\x20challenge\n\n\r\n\x05\x047\x02\x01\x04\x12\x04\xd1\x05\x08\x10\
    \n\r\n\x05\x047\x02\x01\x05\x12\x04\xd1\x05\x11\x16\n\r\n\x05\x047\x02\
    \x01\x01\x12\x04\xd1\x05\x17'\n\r\n\x05\x047\x02\x01\x03\x12\x04\xd1\x05\
    *+\n;\n\x04\x047\x02\x02\x12\x04\xd2\x05\x08-\"-\x20challenge\x20shown\
    \x20on\x20display\x20(e.g.\x20date+time)\n\n\r\n\x05\x047\x02\x02\x04\
    \x12\x04\xd2\x05\x08\x10\n\r\n\x05\x047\x02\x02\x05\x12\x04\xd2\x05\x11\
    \x17\n\r\n\x05\x047\x02\x02\x01\x12\x04\xd2\x05\x18(\n\r\n\x05\x047\x02\
    \x02\x03\x12\x04\xd2\x05+,\n'\n\x04\x047\x02\x03\x12\x04\xd3\x05\x08-\"\
    \x19\x20ECDSA\x20curve\x20name\x20to\x20use\n\n\r\n\x05\x047\x02\x03\x04\
    \x12\x04\xd3\x05\x08\x10\n\r\n\x05\x047\x02\x03\x05\x12\x04\xd3\x05\x11\
    \x17\n\r\n\x05\x047\x02\x03\x01\x12\x04\xd3\x05\x18(\n\r\n\x05\x047\x02\
    \x03\x03\x12\x04\xd3\x05+,\nO\n\x02\x048\x12\x06\xda\x05\0\xde\x05\x01\
    \x1aA*\n\x20Response:\x20Device\x20provides\x20signed\x20identity\n\x20@\
    prev\x20SignIdentity\n\n\x0b\n\x03\x048\x01\x12\x04\xda\x05\x08\x16\n\
    \x20\n\x04\x048\x02\0\x12\x04\xdb\x05\x08$\"\x12\x20identity\x20address\
    \n\n\r\n\x05\x048\x02\0\x04\x12\x04\xdb\x05\x08\x10\n\r\n\x05\x048\x02\0\
    \x05\x12\x04\xdb\x05\x11\x17\n\r\n\x05\x048\x02\0\x01\x12\x04\xdb\x05\
    \x18\x1f\n\r\n\x05\x048\x02\0\x03\x12\x04\xdb\x05\"#\n#\n\x04\x048\x02\
    \x01\x12\x04\xdc\x05\x08&\"\x15\x20identity\x20public\x20key\n\n\r\n\x05\
    \x048\x02\x01\x04\x12\x04\xdc\x05\x08\x10\n\r\n\x05\x048\x02\x01\x05\x12\
    \x04\xdc\x05\x11\x16\n\r\n\x05\x048\x02\x01\x01\x12\x04\xdc\x05\x17!\n\r\
    \n\x05\x048\x02\x01\x03\x12\x04\xdc\x05$%\n.\n\x04\x048\x02\x02\x12\x04\
    \xdd\x05\x08%\"\x20\x20signature\x20of\x20the\x20identity\x20data\n\n\r\
    \n\x05\x048\x02\x02\x04\x12\x04\xdd\x05\x08\x10\n\r\n\x05\x048\x02\x02\
    \x05\x12\x04\xdd\x05\x11\x16\n\r\n\x05\x048\x02\x02\x01\x12\x04\xdd\x05\
    \x17\x20\n\r\n\x05\x048\x02\x02\x03\x12\x04\xdd\x05#$\n\x9f\x01\n\x02\
    \x049\x12\x06\xe9\x05\0\xed\x05\x01\x1aY*\n\x20Request:\x20Ask\x20device\
    \x20to\x20generate\x20ECDH\x20session\x20key\n\x20@next\x20ECDHSessionKe\
    y\n\x20@next\x20Failure\n26/////////////////\n\x20ECDH\x20messages\x20//\
    \n/////////////////\n\n\x0b\n\x03\x049\x01\x12\x04\xe9\x05\x08\x19\n\x18\
    \n\x04\x049\x02\0\x12\x04\xea\x05\x08+\"\n\x20identity\n\n\r\n\x05\x049\
    \x02\0\x04\x12\x04\xea\x05\x08\x10\n\r\n\x05\x049\x02\0\x06\x12\x04\xea\
    \x05\x11\x1d\n\r\n\x05\x049\x02\0\x01\x12\x04\xea\x05\x1e&\n\r\n\x05\x04\
    9\x02\0\x03\x12\x04\xea\x05)*\n!\n\x04\x049\x02\x01\x12\x04\xeb\x05\x08+\
    \"\x13\x20peer's\x20public\x20key\n\n\r\n\x05\x049\x02\x01\x04\x12\x04\
    \xeb\x05\x08\x10\n\r\n\x05\x049\x02\x01\x05\x12\x04\xeb\x05\x11\x16\n\r\
    \n\x05\x049\x02\x01\x01\x12\x04\xeb\x05\x17&\n\r\n\x05\x049\x02\x01\x03\
    \x12\x04\xeb\x05)*\n'\n\x04\x049\x02\x02\x12\x04\xec\x05\x08-\"\x19\x20E\
    CDSA\x20curve\x20name\x20to\x20use\n\n\r\n\x05\x049\x02\x02\x04\x12\x04\
    \xec\x05\x08\x10\n\r\n\x05\x049\x02\x02\x05\x12\x04\xec\x05\x11\x17\n\r\
    \n\x05\x049\x02\x02\x01\x12\x04\xec\x05\x18(\n\r\n\x05\x049\x02\x02\x03\
    \x12\x04\xec\x05+,\nU\n\x02\x04:\x12\x06\xf3\x05\0\xf5\x05\x01\x1aG*\n\
    \x20Response:\x20Device\x20provides\x20ECDH\x20session\x20key\n\x20@prev\
    \x20GetECDHSessionKey\n\n\x0b\n\x03\x04:\x01\x12\x04\xf3\x05\x08\x16\n\
    \x20\n\x04\x04:\x02\0\x12\x04\xf4\x05\x08'\"\x12\x20ECDH\x20session\x20k\
    ey\n\n\r\n\x05\x04:\x02\0\x04\x12\x04\xf4\x05\x08\x10\n\r\n\x05\x04:\x02\
    \0\x05\x12\x04\xf4\x05\x11\x16\n\r\n\x05\x04:\x02\0\x01\x12\x04\xf4\x05\
    \x17\"\n\r\n\x05\x04:\x02\0\x03\x12\x04\xf4\x05%&\np\n\x02\x04;\x12\x06\
    \xff\x05\0\x81\x06\x01\x1a+*\n\x20Request:\x20Set\x20U2F\x20counter\n\
    \x20@next\x20Success\n25/////////////////\n\x20U2F\x20messages\x20//\n//\
    ///////////////\n\n\x0b\n\x03\x04;\x01\x12\x04\xff\x05\x08\x15\n\x17\n\
    \x04\x04;\x02\0\x12\x04\x80\x06\x08(\"\t\x20counter\n\n\r\n\x05\x04;\x02\
    \0\x04\x12\x04\x80\x06\x08\x10\n\r\n\x05\x04;\x02\0\x05\x12\x04\x80\x06\
    \x11\x17\n\r\n\x05\x04;\x02\0\x01\x12\x04\x80\x06\x18#\n\r\n\x05\x04;\
    \x02\0\x03\x12\x04\x80\x06&'\n\xe6\x01\n\x02\x04<\x12\x06\x8d\x06\0\x8f\
    \x06\x01\x1a\x8d\x01*\n\x20Request:\x20Ask\x20device\x20to\x20erase\x20i\
    ts\x20firmware\x20(so\x20it\x20can\x20be\x20replaced\x20via\x20FirmwareU\
    pload)\n\x20@next\x20Success\n\x20@next\x20FirmwareRequest\n\x20@next\
    \x20Failure\n2H///////////////////////\n\x20Bootloader\x20messages\x20//\
    \n///////////////////////\n\n\x0b\n\x03\x04<\x01\x12\x04\x8d\x06\x08\x15\
    \n&\n\x04\x04<\x02\0\x12\x04\x8e\x06\x08#\"\x18\x20length\x20of\x20new\
    \x20firmware\n\n\r\n\x05\x04<\x02\0\x04\x12\x04\x8e\x06\x08\x10\n\r\n\
    \x05\x04<\x02\0\x05\x12\x04\x8e\x06\x11\x17\n\r\n\x05\x04<\x02\0\x01\x12\
    \x04\x8e\x06\x18\x1e\n\r\n\x05\x04<\x02\0\x03\x12\x04\x8e\x06!\"\nH\n\
    \x02\x04=\x12\x06\x95\x06\0\x98\x06\x01\x1a:*\n\x20Response:\x20Ask\x20f\
    or\x20firmware\x20chunk\n\x20@next\x20FirmwareUpload\n\n\x0b\n\x03\x04=\
    \x01\x12\x04\x95\x06\x08\x17\n2\n\x04\x04=\x02\0\x12\x04\x96\x06\x08#\"$\
    \x20offset\x20of\x20requested\x20firmware\x20chunk\n\n\r\n\x05\x04=\x02\
    \0\x04\x12\x04\x96\x06\x08\x10\n\r\n\x05\x04=\x02\0\x05\x12\x04\x96\x06\
    \x11\x17\n\r\n\x05\x04=\x02\0\x01\x12\x04\x96\x06\x18\x1e\n\r\n\x05\x04=\
    \x02\0\x03\x12\x04\x96\x06!\"\n2\n\x04\x04=\x02\x01\x12\x04\x97\x06\x08#\
    \"$\x20length\x20of\x20requested\x20firmware\x20chunk\n\n\r\n\x05\x04=\
    \x02\x01\x04\x12\x04\x97\x06\x08\x10\n\r\n\x05\x04=\x02\x01\x05\x12\x04\
    \x97\x06\x11\x17\n\r\n\x05\x04=\x02\x01\x01\x12\x04\x97\x06\x18\x1e\n\r\
    \n\x05\x04=\x02\x01\x03\x12\x04\x97\x06!\"\nc\n\x02\x04>\x12\x06\x9f\x06\
    \0\xa2\x06\x01\x1aU*\n\x20Request:\x20Send\x20firmware\x20in\x20binary\
    \x20form\x20to\x20the\x20device\n\x20@next\x20Success\n\x20@next\x20Fail\
    ure\n\n\x0b\n\x03\x04>\x01\x12\x04\x9f\x06\x08\x16\n1\n\x04\x04>\x02\0\
    \x12\x04\xa0\x06\x08#\"#\x20firmware\x20to\x20be\x20loaded\x20into\x20de\
    vice\n\n\r\n\x05\x04>\x02\0\x04\x12\x04\xa0\x06\x08\x10\n\r\n\x05\x04>\
    \x02\0\x05\x12\x04\xa0\x06\x11\x16\n\r\n\x05\x04>\x02\0\x01\x12\x04\xa0\
    \x06\x17\x1e\n\r\n\x05\x04>\x02\0\x03\x12\x04\xa0\x06!\"\n#\n\x04\x04>\
    \x02\x01\x12\x04\xa1\x06\x08\x20\"\x15\x20hash\x20of\x20the\x20payload\n\
    \n\r\n\x05\x04>\x02\x01\x04\x12\x04\xa1\x06\x08\x10\n\r\n\x05\x04>\x02\
    \x01\x05\x12\x04\xa1\x06\x11\x16\n\r\n\x05\x04>\x02\x01\x01\x12\x04\xa1\
    \x06\x17\x1b\n\r\n\x05\x04>\x02\x01\x03\x12\x04\xa1\x06\x1e\x1f\nS\n\x02\
    \x04?\x12\x06\xaa\x06\0\xac\x06\x01\x1aE*\n\x20Request:\x20Perform\x20a\
    \x20device\x20self-test\n\x20@next\x20Success\n\x20@next\x20Failure\n\n\
    \x0b\n\x03\x04?\x01\x12\x04\xaa\x06\x08\x10\n/\n\x04\x04?\x02\0\x12\x04\
    \xab\x06\x08#\"!\x20payload\x20to\x20be\x20used\x20in\x20self-test\n\n\r\
    \n\x05\x04?\x02\0\x04\x12\x04\xab\x06\x08\x10\n\r\n\x05\x04?\x02\0\x05\
    \x12\x04\xab\x06\x11\x16\n\r\n\x05\x04?\x02\0\x01\x12\x04\xab\x06\x17\
    \x1e\n\r\n\x05\x04?\x02\0\x03\x12\x04\xab\x06!\"\n\x81\x02\n\x02\x04@\
    \x12\x06\xb6\x06\0\xb8\x06\x01\x1a<*\n\x20Request:\x20\"Press\"\x20the\
    \x20button\x20on\x20the\x20device\n\x20@next\x20Success\n2\xb4\x01//////\
    /////////////////////////////////////////////////////\n\x20Debug\x20mess\
    ages\x20(only\x20available\x20if\x20DebugLink\x20is\x20enabled)\x20//\n/\
    //////////////////////////////////////////////////////////\n\n\x0b\n\x03\
    \x04@\x01\x12\x04\xb6\x06\x08\x19\n6\n\x04\x04@\x02\0\x12\x04\xb7\x06\
    \x08!\"(\x20true\x20for\x20\"Confirm\",\x20false\x20for\x20\"Cancel\"\n\
    \n\r\n\x05\x04@\x02\0\x04\x12\x04\xb7\x06\x08\x10\n\r\n\x05\x04@\x02\0\
    \x05\x12\x04\xb7\x06\x11\x15\n\r\n\x05\x04@\x02\0\x01\x12\x04\xb7\x06\
    \x16\x1c\n\r\n\x05\x04@\x02\0\x03\x12\x04\xb7\x06\x1f\x20\nO\n\x02\x04A\
    \x12\x06\xbe\x06\0\xbf\x06\x01\x1aA*\n\x20Request:\x20Computer\x20asks\
    \x20for\x20device\x20state\n\x20@next\x20DebugLinkState\n\n\x0b\n\x03\
    \x04A\x01\x12\x04\xbe\x06\x08\x19\nI\n\x02\x04B\x12\x06\xc5\x06\0\xd0\
    \x06\x01\x1a;*\n\x20Response:\x20Device\x20current\x20state\n\x20@prev\
    \x20DebugLinkGetState\n\n\x0b\n\x03\x04B\x01\x12\x04\xc5\x06\x08\x16\n%\
    \n\x04\x04B\x02\0\x12\x04\xc6\x06\x08\"\"\x17\x20raw\x20buffer\x20of\x20\
    display\n\n\r\n\x05\x04B\x02\0\x04\x12\x04\xc6\x06\x08\x10\n\r\n\x05\x04\
    B\x02\0\x05\x12\x04\xc6\x06\x11\x16\n\r\n\x05\x04B\x02\0\x01\x12\x04\xc6\
    \x06\x17\x1d\n\r\n\x05\x04B\x02\0\x03\x12\x04\xc6\x06\x20!\n<\n\x04\x04B\
    \x02\x01\x12\x04\xc7\x06\x08\x20\".\x20current\x20PIN,\x20blank\x20if\
    \x20PIN\x20is\x20not\x20set/enabled\n\n\r\n\x05\x04B\x02\x01\x04\x12\x04\
    \xc7\x06\x08\x10\n\r\n\x05\x04B\x02\x01\x05\x12\x04\xc7\x06\x11\x17\n\r\
    \n\x05\x04B\x02\x01\x01\x12\x04\xc7\x06\x18\x1b\n\r\n\x05\x04B\x02\x01\
    \x03\x12\x04\xc7\x06\x1e\x1f\n\"\n\x04\x04B\x02\x02\x12\x04\xc8\x06\x08#\
    \"\x14\x20current\x20PIN\x20matrix\n\n\r\n\x05\x04B\x02\x02\x04\x12\x04\
    \xc8\x06\x08\x10\n\r\n\x05\x04B\x02\x02\x05\x12\x04\xc8\x06\x11\x17\n\r\
    \n\x05\x04B\x02\x02\x01\x12\x04\xc8\x06\x18\x1e\n\r\n\x05\x04B\x02\x02\
    \x03\x12\x04\xc8\x06!\"\n'\n\x04\x04B\x02\x03\x12\x04\xc9\x06\x08%\"\x19\
    \x20current\x20BIP-39\x20mnemonic\n\n\r\n\x05\x04B\x02\x03\x04\x12\x04\
    \xc9\x06\x08\x10\n\r\n\x05\x04B\x02\x03\x05\x12\x04\xc9\x06\x11\x17\n\r\
    \n\x05\x04B\x02\x03\x01\x12\x04\xc9\x06\x18\x20\n\r\n\x05\x04B\x02\x03\
    \x03\x12\x04\xc9\x06#$\n#\n\x04\x04B\x02\x04\x12\x04\xca\x06\x08%\"\x15\
    \x20current\x20BIP-32\x20node\n\n\r\n\x05\x04B\x02\x04\x04\x12\x04\xca\
    \x06\x08\x10\n\r\n\x05\x04B\x02\x04\x06\x12\x04\xca\x06\x11\x1b\n\r\n\
    \x05\x04B\x02\x04\x01\x12\x04\xca\x06\x1c\x20\n\r\n\x05\x04B\x02\x04\x03\
    \x12\x04\xca\x06#$\n<\n\x04\x04B\x02\x05\x12\x04\xcb\x06\x080\".\x20is\
    \x20node/mnemonic\x20encrypted\x20using\x20passphrase?\n\n\r\n\x05\x04B\
    \x02\x05\x04\x12\x04\xcb\x06\x08\x10\n\r\n\x05\x04B\x02\x05\x05\x12\x04\
    \xcb\x06\x11\x15\n\r\n\x05\x04B\x02\x05\x01\x12\x04\xcb\x06\x16+\n\r\n\
    \x05\x04B\x02\x05\x03\x12\x04\xcb\x06./\nB\n\x04\x04B\x02\x06\x12\x04\
    \xcc\x06\x08'\"4\x20word\x20on\x20device\x20display\x20during\x20ResetDe\
    vice\x20workflow\n\n\r\n\x05\x04B\x02\x06\x04\x12\x04\xcc\x06\x08\x10\n\
    \r\n\x05\x04B\x02\x06\x05\x12\x04\xcc\x06\x11\x17\n\r\n\x05\x04B\x02\x06\
    \x01\x12\x04\xcc\x06\x18\"\n\r\n\x05\x04B\x02\x06\x03\x12\x04\xcc\x06%&\
    \n;\n\x04\x04B\x02\x07\x12\x04\xcd\x06\x08)\"-\x20current\x20entropy\x20\
    during\x20ResetDevice\x20workflow\n\n\r\n\x05\x04B\x02\x07\x04\x12\x04\
    \xcd\x06\x08\x10\n\r\n\x05\x04B\x02\x07\x05\x12\x04\xcd\x06\x11\x16\n\r\
    \n\x05\x04B\x02\x07\x01\x12\x04\xcd\x06\x17$\n\r\n\x05\x04B\x02\x07\x03\
    \x12\x04\xcd\x06'(\nE\n\x04\x04B\x02\x08\x12\x04\xce\x06\x08/\"7\x20(fak\
    e)\x20word\x20on\x20display\x20during\x20RecoveryDevice\x20workflow\n\n\
    \r\n\x05\x04B\x02\x08\x04\x12\x04\xce\x06\x08\x10\n\r\n\x05\x04B\x02\x08\
    \x05\x12\x04\xce\x06\x11\x17\n\r\n\x05\x04B\x02\x08\x01\x12\x04\xce\x06\
    \x18*\n\r\n\x05\x04B\x02\x08\x03\x12\x04\xce\x06-.\n]\n\x04\x04B\x02\t\
    \x12\x04\xcf\x06\x08/\"O\x20index\x20of\x20mnemonic\x20word\x20the\x20de\
    vice\x20is\x20expecting\x20during\x20RecoveryDevice\x20workflow\n\n\r\n\
    \x05\x04B\x02\t\x04\x12\x04\xcf\x06\x08\x10\n\r\n\x05\x04B\x02\t\x05\x12\
    \x04\xcf\x06\x11\x17\n\r\n\x05\x04B\x02\t\x01\x12\x04\xcf\x06\x18)\n\r\n\
    \x05\x04B\x02\t\x03\x12\x04\xcf\x06,.\n0\n\x02\x04C\x12\x06\xd5\x06\0\
    \xd6\x06\x01\x1a\"*\n\x20Request:\x20Ask\x20device\x20to\x20restart\n\n\
    \x0b\n\x03\x04C\x01\x12\x04\xd5\x06\x08\x15\n:\n\x02\x04D\x12\x06\xdb\
    \x06\0\xdf\x06\x01\x1a,*\n\x20Response:\x20Device\x20wants\x20host\x20to\
    \x20log\x20event\n\n\x0b\n\x03\x04D\x01\x12\x04\xdb\x06\x08\x14\n\x0c\n\
    \x04\x04D\x02\0\x12\x04\xdc\x06\x08\"\n\r\n\x05\x04D\x02\0\x04\x12\x04\
    \xdc\x06\x08\x10\n\r\n\x05\x04D\x02\0\x05\x12\x04\xdc\x06\x11\x17\n\r\n\
    \x05\x04D\x02\0\x01\x12\x04\xdc\x06\x18\x1d\n\r\n\x05\x04D\x02\0\x03\x12\
    \x04\xdc\x06\x20!\n\x0c\n\x04\x04D\x02\x01\x12\x04\xdd\x06\x08#\n\r\n\
    \x05\x04D\x02\x01\x04\x12\x04\xdd\x06\x08\x10\n\r\n\x05\x04D\x02\x01\x05\
    \x12\x04\xdd\x06\x11\x17\n\r\n\x05\x04D\x02\x01\x01\x12\x04\xdd\x06\x18\
    \x1e\n\r\n\x05\x04D\x02\x01\x03\x12\x04\xdd\x06!\"\n\x0c\n\x04\x04D\x02\
    \x02\x12\x04\xde\x06\x08!\n\r\n\x05\x04D\x02\x02\x04\x12\x04\xde\x06\x08\
    \x10\n\r\n\x05\x04D\x02\x02\x05\x12\x04\xde\x06\x11\x17\n\r\n\x05\x04D\
    \x02\x02\x01\x12\x04\xde\x06\x18\x1c\n\r\n\x05\x04D\x02\x02\x03\x12\x04\
    \xde\x06\x1f\x20\nI\n\x02\x04E\x12\x06\xe5\x06\0\xe8\x06\x01\x1a;*\n\x20\
    Request:\x20Read\x20memory\x20from\x20device\n\x20@next\x20DebugLinkMemo\
    ry\n\n\x0b\n\x03\x04E\x01\x12\x04\xe5\x06\x08\x1b\n\x0c\n\x04\x04E\x02\0\
    \x12\x04\xe6\x06\x08$\n\r\n\x05\x04E\x02\0\x04\x12\x04\xe6\x06\x08\x10\n\
    \r\n\x05\x04E\x02\0\x05\x12\x04\xe6\x06\x11\x17\n\r\n\x05\x04E\x02\0\x01\
    \x12\x04\xe6\x06\x18\x1f\n\r\n\x05\x04E\x02\0\x03\x12\x04\xe6\x06\"#\n\
    \x0c\n\x04\x04E\x02\x01\x12\x04\xe7\x06\x08#\n\r\n\x05\x04E\x02\x01\x04\
    \x12\x04\xe7\x06\x08\x10\n\r\n\x05\x04E\x02\x01\x05\x12\x04\xe7\x06\x11\
    \x17\n\r\n\x05\x04E\x02\x01\x01\x12\x04\xe7\x06\x18\x1e\n\r\n\x05\x04E\
    \x02\x01\x03\x12\x04\xe7\x06!\"\nO\n\x02\x04F\x12\x06\xee\x06\0\xf0\x06\
    \x01\x1aA*\n\x20Response:\x20Device\x20sends\x20memory\x20back\n\x20@pre\
    v\x20DebugLinkMemoryRead\n\n\x0b\n\x03\x04F\x01\x12\x04\xee\x06\x08\x17\
    \n\x0c\n\x04\x04F\x02\0\x12\x04\xef\x06\x08\"\n\r\n\x05\x04F\x02\0\x04\
    \x12\x04\xef\x06\x08\x10\n\r\n\x05\x04F\x02\0\x05\x12\x04\xef\x06\x11\
    \x16\n\r\n\x05\x04F\x02\0\x01\x12\x04\xef\x06\x17\x1d\n\r\n\x05\x04F\x02\
    \0\x03\x12\x04\xef\x06\x20!\n|\n\x02\x04G\x12\x06\xf6\x06\0\xfa\x06\x01\
    \x1an*\n\x20Request:\x20Write\x20memory\x20to\x20device.\n\x20WARNING:\
    \x20Writing\x20to\x20the\x20wrong\x20location\x20can\x20irreparably\x20b\
    reak\x20the\x20device.\n\n\x0b\n\x03\x04G\x01\x12\x04\xf6\x06\x08\x1c\n\
    \x0c\n\x04\x04G\x02\0\x12\x04\xf7\x06\x08$\n\r\n\x05\x04G\x02\0\x04\x12\
    \x04\xf7\x06\x08\x10\n\r\n\x05\x04G\x02\0\x05\x12\x04\xf7\x06\x11\x17\n\
    \r\n\x05\x04G\x02\0\x01\x12\x04\xf7\x06\x18\x1f\n\r\n\x05\x04G\x02\0\x03\
    \x12\x04\xf7\x06\"#\n\x0c\n\x04\x04G\x02\x01\x12\x04\xf8\x06\x08\"\n\r\n\
    \x05\x04G\x02\x01\x04\x12\x04\xf8\x06\x08\x10\n\r\n\x05\x04G\x02\x01\x05\
    \x12\x04\xf8\x06\x11\x16\n\r\n\x05\x04G\x02\x01\x01\x12\x04\xf8\x06\x17\
    \x1d\n\r\n\x05\x04G\x02\x01\x03\x12\x04\xf8\x06\x20!\n\x0c\n\x04\x04G\
    \x02\x02\x12\x04\xf9\x06\x08\x20\n\r\n\x05\x04G\x02\x02\x04\x12\x04\xf9\
    \x06\x08\x10\n\r\n\x05\x04G\x02\x02\x05\x12\x04\xf9\x06\x11\x15\n\r\n\
    \x05\x04G\x02\x02\x01\x12\x04\xf9\x06\x16\x1b\n\r\n\x05\x04G\x02\x02\x03\
    \x12\x04\xf9\x06\x1e\x1f\n\x83\x01\n\x02\x04H\x12\x06\x80\x07\0\x82\x07\
    \x01\x1au*\n\x20Request:\x20Erase\x20block\x20of\x20flash\x20on\x20devic\
    e\n\x20WARNING:\x20Writing\x20to\x20the\x20wrong\x20location\x20can\x20i\
    rreparably\x20break\x20the\x20device.\n\n\x0b\n\x03\x04H\x01\x12\x04\x80\
    \x07\x08\x1b\n\x0c\n\x04\x04H\x02\0\x12\x04\x81\x07\x08#\n\r\n\x05\x04H\
    \x02\0\x04\x12\x04\x81\x07\x08\x10\n\r\n\x05\x04H\x02\0\x05\x12\x04\x81\
    \x07\x11\x17\n\r\n\x05\x04H\x02\0\x01\x12\x04\x81\x07\x18\x1e\n\r\n\x05\
    \x04H\x02\0\x03\x12\x04\x81\x07!\"\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
