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
pub struct HDNodeType {
    // message fields
    depth: ::std::option::Option<u32>,
    fingerprint: ::std::option::Option<u32>,
    child_num: ::std::option::Option<u32>,
    chain_code: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    private_key: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    public_key: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for HDNodeType {}

impl HDNodeType {
    pub fn new() -> HDNodeType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static HDNodeType {
        static mut instance: ::protobuf::lazy::Lazy<HDNodeType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HDNodeType,
        };
        unsafe {
            instance.get(HDNodeType::new)
        }
    }

    // required uint32 depth = 1;

    pub fn clear_depth(&mut self) {
        self.depth = ::std::option::Option::None;
    }

    pub fn has_depth(&self) -> bool {
        self.depth.is_some()
    }

    // Param is passed by value, moved
    pub fn set_depth(&mut self, v: u32) {
        self.depth = ::std::option::Option::Some(v);
    }

    pub fn get_depth(&self) -> u32 {
        self.depth.unwrap_or(0)
    }

    fn get_depth_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.depth
    }

    fn mut_depth_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.depth
    }

    // required uint32 fingerprint = 2;

    pub fn clear_fingerprint(&mut self) {
        self.fingerprint = ::std::option::Option::None;
    }

    pub fn has_fingerprint(&self) -> bool {
        self.fingerprint.is_some()
    }

    // Param is passed by value, moved
    pub fn set_fingerprint(&mut self, v: u32) {
        self.fingerprint = ::std::option::Option::Some(v);
    }

    pub fn get_fingerprint(&self) -> u32 {
        self.fingerprint.unwrap_or(0)
    }

    fn get_fingerprint_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.fingerprint
    }

    fn mut_fingerprint_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.fingerprint
    }

    // required uint32 child_num = 3;

    pub fn clear_child_num(&mut self) {
        self.child_num = ::std::option::Option::None;
    }

    pub fn has_child_num(&self) -> bool {
        self.child_num.is_some()
    }

    // Param is passed by value, moved
    pub fn set_child_num(&mut self, v: u32) {
        self.child_num = ::std::option::Option::Some(v);
    }

    pub fn get_child_num(&self) -> u32 {
        self.child_num.unwrap_or(0)
    }

    fn get_child_num_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.child_num
    }

    fn mut_child_num_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.child_num
    }

    // required bytes chain_code = 4;

    pub fn clear_chain_code(&mut self) {
        self.chain_code.clear();
    }

    pub fn has_chain_code(&self) -> bool {
        self.chain_code.is_some()
    }

    // Param is passed by value, moved
    pub fn set_chain_code(&mut self, v: ::std::vec::Vec<u8>) {
        self.chain_code = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_chain_code(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.chain_code.is_none() {
            self.chain_code.set_default();
        }
        self.chain_code.as_mut().unwrap()
    }

    // Take field
    pub fn take_chain_code(&mut self) -> ::std::vec::Vec<u8> {
        self.chain_code.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_chain_code(&self) -> &[u8] {
        match self.chain_code.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_chain_code_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.chain_code
    }

    fn mut_chain_code_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.chain_code
    }

    // optional bytes private_key = 5;

    pub fn clear_private_key(&mut self) {
        self.private_key.clear();
    }

    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    // Param is passed by value, moved
    pub fn set_private_key(&mut self, v: ::std::vec::Vec<u8>) {
        self.private_key = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_private_key(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.private_key.is_none() {
            self.private_key.set_default();
        }
        self.private_key.as_mut().unwrap()
    }

    // Take field
    pub fn take_private_key(&mut self) -> ::std::vec::Vec<u8> {
        self.private_key.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_private_key(&self) -> &[u8] {
        match self.private_key.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_private_key_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.private_key
    }

    fn mut_private_key_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.private_key
    }

    // optional bytes public_key = 6;

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
}

impl ::protobuf::Message for HDNodeType {
    fn is_initialized(&self) -> bool {
        if self.depth.is_none() {
            return false;
        }
        if self.fingerprint.is_none() {
            return false;
        }
        if self.child_num.is_none() {
            return false;
        }
        if self.chain_code.is_none() {
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
                    self.depth = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.fingerprint = ::std::option::Option::Some(tmp);
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.child_num = ::std::option::Option::Some(tmp);
                },
                4 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.chain_code)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.private_key)?;
                },
                6 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.public_key)?;
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
        if let Some(v) = self.depth {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.fingerprint {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.child_num {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.chain_code.as_ref() {
            my_size += ::protobuf::rt::bytes_size(4, &v);
        }
        if let Some(ref v) = self.private_key.as_ref() {
            my_size += ::protobuf::rt::bytes_size(5, &v);
        }
        if let Some(ref v) = self.public_key.as_ref() {
            my_size += ::protobuf::rt::bytes_size(6, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.depth {
            os.write_uint32(1, v)?;
        }
        if let Some(v) = self.fingerprint {
            os.write_uint32(2, v)?;
        }
        if let Some(v) = self.child_num {
            os.write_uint32(3, v)?;
        }
        if let Some(ref v) = self.chain_code.as_ref() {
            os.write_bytes(4, &v)?;
        }
        if let Some(ref v) = self.private_key.as_ref() {
            os.write_bytes(5, &v)?;
        }
        if let Some(ref v) = self.public_key.as_ref() {
            os.write_bytes(6, &v)?;
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

impl ::protobuf::MessageStatic for HDNodeType {
    fn new() -> HDNodeType {
        HDNodeType::new()
    }

    fn descriptor_static(_: ::std::option::Option<HDNodeType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "depth",
                    HDNodeType::get_depth_for_reflect,
                    HDNodeType::mut_depth_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "fingerprint",
                    HDNodeType::get_fingerprint_for_reflect,
                    HDNodeType::mut_fingerprint_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "child_num",
                    HDNodeType::get_child_num_for_reflect,
                    HDNodeType::mut_child_num_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "chain_code",
                    HDNodeType::get_chain_code_for_reflect,
                    HDNodeType::mut_chain_code_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "private_key",
                    HDNodeType::get_private_key_for_reflect,
                    HDNodeType::mut_private_key_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "public_key",
                    HDNodeType::get_public_key_for_reflect,
                    HDNodeType::mut_public_key_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HDNodeType>(
                    "HDNodeType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for HDNodeType {
    fn clear(&mut self) {
        self.clear_depth();
        self.clear_fingerprint();
        self.clear_child_num();
        self.clear_chain_code();
        self.clear_private_key();
        self.clear_public_key();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for HDNodeType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HDNodeType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct HDNodePathType {
    // message fields
    node: ::protobuf::SingularPtrField<HDNodeType>,
    address_n: ::std::vec::Vec<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for HDNodePathType {}

impl HDNodePathType {
    pub fn new() -> HDNodePathType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static HDNodePathType {
        static mut instance: ::protobuf::lazy::Lazy<HDNodePathType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HDNodePathType,
        };
        unsafe {
            instance.get(HDNodePathType::new)
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
    pub fn set_node(&mut self, v: HDNodeType) {
        self.node = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_node(&mut self) -> &mut HDNodeType {
        if self.node.is_none() {
            self.node.set_default();
        }
        self.node.as_mut().unwrap()
    }

    // Take field
    pub fn take_node(&mut self) -> HDNodeType {
        self.node.take().unwrap_or_else(|| HDNodeType::new())
    }

    pub fn get_node(&self) -> &HDNodeType {
        self.node.as_ref().unwrap_or_else(|| HDNodeType::default_instance())
    }

    fn get_node_for_reflect(&self) -> &::protobuf::SingularPtrField<HDNodeType> {
        &self.node
    }

    fn mut_node_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<HDNodeType> {
        &mut self.node
    }

    // repeated uint32 address_n = 2;

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
}

impl ::protobuf::Message for HDNodePathType {
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
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
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
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(2, *value, ::protobuf::wire_format::WireTypeVarint);
        };
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
        for v in &self.address_n {
            os.write_uint32(2, *v)?;
        };
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

impl ::protobuf::MessageStatic for HDNodePathType {
    fn new() -> HDNodePathType {
        HDNodePathType::new()
    }

    fn descriptor_static(_: ::std::option::Option<HDNodePathType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<HDNodeType>>(
                    "node",
                    HDNodePathType::get_node_for_reflect,
                    HDNodePathType::mut_node_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    HDNodePathType::get_address_n_for_reflect,
                    HDNodePathType::mut_address_n_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HDNodePathType>(
                    "HDNodePathType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for HDNodePathType {
    fn clear(&mut self) {
        self.clear_node();
        self.clear_address_n();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for HDNodePathType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HDNodePathType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CoinType {
    // message fields
    coin_name: ::protobuf::SingularField<::std::string::String>,
    coin_shortcut: ::protobuf::SingularField<::std::string::String>,
    address_type: ::std::option::Option<u32>,
    maxfee_kb: ::std::option::Option<u64>,
    address_type_p2sh: ::std::option::Option<u32>,
    signed_message_header: ::protobuf::SingularField<::std::string::String>,
    xpub_magic: ::std::option::Option<u32>,
    xprv_magic: ::std::option::Option<u32>,
    segwit: ::std::option::Option<bool>,
    forkid: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CoinType {}

impl CoinType {
    pub fn new() -> CoinType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CoinType {
        static mut instance: ::protobuf::lazy::Lazy<CoinType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CoinType,
        };
        unsafe {
            instance.get(CoinType::new)
        }
    }

    // optional string coin_name = 1;

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
            None => "",
        }
    }

    fn get_coin_name_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_name
    }

    fn mut_coin_name_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_name
    }

    // optional string coin_shortcut = 2;

    pub fn clear_coin_shortcut(&mut self) {
        self.coin_shortcut.clear();
    }

    pub fn has_coin_shortcut(&self) -> bool {
        self.coin_shortcut.is_some()
    }

    // Param is passed by value, moved
    pub fn set_coin_shortcut(&mut self, v: ::std::string::String) {
        self.coin_shortcut = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_coin_shortcut(&mut self) -> &mut ::std::string::String {
        if self.coin_shortcut.is_none() {
            self.coin_shortcut.set_default();
        }
        self.coin_shortcut.as_mut().unwrap()
    }

    // Take field
    pub fn take_coin_shortcut(&mut self) -> ::std::string::String {
        self.coin_shortcut.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_coin_shortcut(&self) -> &str {
        match self.coin_shortcut.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_coin_shortcut_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.coin_shortcut
    }

    fn mut_coin_shortcut_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.coin_shortcut
    }

    // optional uint32 address_type = 3;

    pub fn clear_address_type(&mut self) {
        self.address_type = ::std::option::Option::None;
    }

    pub fn has_address_type(&self) -> bool {
        self.address_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address_type(&mut self, v: u32) {
        self.address_type = ::std::option::Option::Some(v);
    }

    pub fn get_address_type(&self) -> u32 {
        self.address_type.unwrap_or(0u32)
    }

    fn get_address_type_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.address_type
    }

    fn mut_address_type_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.address_type
    }

    // optional uint64 maxfee_kb = 4;

    pub fn clear_maxfee_kb(&mut self) {
        self.maxfee_kb = ::std::option::Option::None;
    }

    pub fn has_maxfee_kb(&self) -> bool {
        self.maxfee_kb.is_some()
    }

    // Param is passed by value, moved
    pub fn set_maxfee_kb(&mut self, v: u64) {
        self.maxfee_kb = ::std::option::Option::Some(v);
    }

    pub fn get_maxfee_kb(&self) -> u64 {
        self.maxfee_kb.unwrap_or(0)
    }

    fn get_maxfee_kb_for_reflect(&self) -> &::std::option::Option<u64> {
        &self.maxfee_kb
    }

    fn mut_maxfee_kb_for_reflect(&mut self) -> &mut ::std::option::Option<u64> {
        &mut self.maxfee_kb
    }

    // optional uint32 address_type_p2sh = 5;

    pub fn clear_address_type_p2sh(&mut self) {
        self.address_type_p2sh = ::std::option::Option::None;
    }

    pub fn has_address_type_p2sh(&self) -> bool {
        self.address_type_p2sh.is_some()
    }

    // Param is passed by value, moved
    pub fn set_address_type_p2sh(&mut self, v: u32) {
        self.address_type_p2sh = ::std::option::Option::Some(v);
    }

    pub fn get_address_type_p2sh(&self) -> u32 {
        self.address_type_p2sh.unwrap_or(5u32)
    }

    fn get_address_type_p2sh_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.address_type_p2sh
    }

    fn mut_address_type_p2sh_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.address_type_p2sh
    }

    // optional string signed_message_header = 8;

    pub fn clear_signed_message_header(&mut self) {
        self.signed_message_header.clear();
    }

    pub fn has_signed_message_header(&self) -> bool {
        self.signed_message_header.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signed_message_header(&mut self, v: ::std::string::String) {
        self.signed_message_header = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_signed_message_header(&mut self) -> &mut ::std::string::String {
        if self.signed_message_header.is_none() {
            self.signed_message_header.set_default();
        }
        self.signed_message_header.as_mut().unwrap()
    }

    // Take field
    pub fn take_signed_message_header(&mut self) -> ::std::string::String {
        self.signed_message_header.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_signed_message_header(&self) -> &str {
        match self.signed_message_header.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_signed_message_header_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.signed_message_header
    }

    fn mut_signed_message_header_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.signed_message_header
    }

    // optional uint32 xpub_magic = 9;

    pub fn clear_xpub_magic(&mut self) {
        self.xpub_magic = ::std::option::Option::None;
    }

    pub fn has_xpub_magic(&self) -> bool {
        self.xpub_magic.is_some()
    }

    // Param is passed by value, moved
    pub fn set_xpub_magic(&mut self, v: u32) {
        self.xpub_magic = ::std::option::Option::Some(v);
    }

    pub fn get_xpub_magic(&self) -> u32 {
        self.xpub_magic.unwrap_or(76067358u32)
    }

    fn get_xpub_magic_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.xpub_magic
    }

    fn mut_xpub_magic_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.xpub_magic
    }

    // optional uint32 xprv_magic = 10;

    pub fn clear_xprv_magic(&mut self) {
        self.xprv_magic = ::std::option::Option::None;
    }

    pub fn has_xprv_magic(&self) -> bool {
        self.xprv_magic.is_some()
    }

    // Param is passed by value, moved
    pub fn set_xprv_magic(&mut self, v: u32) {
        self.xprv_magic = ::std::option::Option::Some(v);
    }

    pub fn get_xprv_magic(&self) -> u32 {
        self.xprv_magic.unwrap_or(76066276u32)
    }

    fn get_xprv_magic_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.xprv_magic
    }

    fn mut_xprv_magic_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.xprv_magic
    }

    // optional bool segwit = 11;

    pub fn clear_segwit(&mut self) {
        self.segwit = ::std::option::Option::None;
    }

    pub fn has_segwit(&self) -> bool {
        self.segwit.is_some()
    }

    // Param is passed by value, moved
    pub fn set_segwit(&mut self, v: bool) {
        self.segwit = ::std::option::Option::Some(v);
    }

    pub fn get_segwit(&self) -> bool {
        self.segwit.unwrap_or(false)
    }

    fn get_segwit_for_reflect(&self) -> &::std::option::Option<bool> {
        &self.segwit
    }

    fn mut_segwit_for_reflect(&mut self) -> &mut ::std::option::Option<bool> {
        &mut self.segwit
    }

    // optional uint32 forkid = 12;

    pub fn clear_forkid(&mut self) {
        self.forkid = ::std::option::Option::None;
    }

    pub fn has_forkid(&self) -> bool {
        self.forkid.is_some()
    }

    // Param is passed by value, moved
    pub fn set_forkid(&mut self, v: u32) {
        self.forkid = ::std::option::Option::Some(v);
    }

    pub fn get_forkid(&self) -> u32 {
        self.forkid.unwrap_or(0)
    }

    fn get_forkid_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.forkid
    }

    fn mut_forkid_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.forkid
    }
}

impl ::protobuf::Message for CoinType {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.coin_shortcut)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.address_type = ::std::option::Option::Some(tmp);
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.maxfee_kb = ::std::option::Option::Some(tmp);
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.address_type_p2sh = ::std::option::Option::Some(tmp);
                },
                8 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.signed_message_header)?;
                },
                9 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.xpub_magic = ::std::option::Option::Some(tmp);
                },
                10 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.xprv_magic = ::std::option::Option::Some(tmp);
                },
                11 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.segwit = ::std::option::Option::Some(tmp);
                },
                12 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.forkid = ::std::option::Option::Some(tmp);
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
        if let Some(ref v) = self.coin_name.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(ref v) = self.coin_shortcut.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        if let Some(v) = self.address_type {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.maxfee_kb {
            my_size += ::protobuf::rt::value_size(4, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.address_type_p2sh {
            my_size += ::protobuf::rt::value_size(5, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.signed_message_header.as_ref() {
            my_size += ::protobuf::rt::string_size(8, &v);
        }
        if let Some(v) = self.xpub_magic {
            my_size += ::protobuf::rt::value_size(9, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.xprv_magic {
            my_size += ::protobuf::rt::value_size(10, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.segwit {
            my_size += 2;
        }
        if let Some(v) = self.forkid {
            my_size += ::protobuf::rt::value_size(12, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.coin_name.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(ref v) = self.coin_shortcut.as_ref() {
            os.write_string(2, &v)?;
        }
        if let Some(v) = self.address_type {
            os.write_uint32(3, v)?;
        }
        if let Some(v) = self.maxfee_kb {
            os.write_uint64(4, v)?;
        }
        if let Some(v) = self.address_type_p2sh {
            os.write_uint32(5, v)?;
        }
        if let Some(ref v) = self.signed_message_header.as_ref() {
            os.write_string(8, &v)?;
        }
        if let Some(v) = self.xpub_magic {
            os.write_uint32(9, v)?;
        }
        if let Some(v) = self.xprv_magic {
            os.write_uint32(10, v)?;
        }
        if let Some(v) = self.segwit {
            os.write_bool(11, v)?;
        }
        if let Some(v) = self.forkid {
            os.write_uint32(12, v)?;
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

impl ::protobuf::MessageStatic for CoinType {
    fn new() -> CoinType {
        CoinType::new()
    }

    fn descriptor_static(_: ::std::option::Option<CoinType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_name",
                    CoinType::get_coin_name_for_reflect,
                    CoinType::mut_coin_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "coin_shortcut",
                    CoinType::get_coin_shortcut_for_reflect,
                    CoinType::mut_coin_shortcut_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_type",
                    CoinType::get_address_type_for_reflect,
                    CoinType::mut_address_type_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "maxfee_kb",
                    CoinType::get_maxfee_kb_for_reflect,
                    CoinType::mut_maxfee_kb_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_type_p2sh",
                    CoinType::get_address_type_p2sh_for_reflect,
                    CoinType::mut_address_type_p2sh_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "signed_message_header",
                    CoinType::get_signed_message_header_for_reflect,
                    CoinType::mut_signed_message_header_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "xpub_magic",
                    CoinType::get_xpub_magic_for_reflect,
                    CoinType::mut_xpub_magic_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "xprv_magic",
                    CoinType::get_xprv_magic_for_reflect,
                    CoinType::mut_xprv_magic_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "segwit",
                    CoinType::get_segwit_for_reflect,
                    CoinType::mut_segwit_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "forkid",
                    CoinType::get_forkid_for_reflect,
                    CoinType::mut_forkid_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<CoinType>(
                    "CoinType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CoinType {
    fn clear(&mut self) {
        self.clear_coin_name();
        self.clear_coin_shortcut();
        self.clear_address_type();
        self.clear_maxfee_kb();
        self.clear_address_type_p2sh();
        self.clear_signed_message_header();
        self.clear_xpub_magic();
        self.clear_xprv_magic();
        self.clear_segwit();
        self.clear_forkid();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CoinType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CoinType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct MultisigRedeemScriptType {
    // message fields
    pubkeys: ::protobuf::RepeatedField<HDNodePathType>,
    signatures: ::protobuf::RepeatedField<::std::vec::Vec<u8>>,
    m: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for MultisigRedeemScriptType {}

impl MultisigRedeemScriptType {
    pub fn new() -> MultisigRedeemScriptType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static MultisigRedeemScriptType {
        static mut instance: ::protobuf::lazy::Lazy<MultisigRedeemScriptType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const MultisigRedeemScriptType,
        };
        unsafe {
            instance.get(MultisigRedeemScriptType::new)
        }
    }

    // repeated .HDNodePathType pubkeys = 1;

    pub fn clear_pubkeys(&mut self) {
        self.pubkeys.clear();
    }

    // Param is passed by value, moved
    pub fn set_pubkeys(&mut self, v: ::protobuf::RepeatedField<HDNodePathType>) {
        self.pubkeys = v;
    }

    // Mutable pointer to the field.
    pub fn mut_pubkeys(&mut self) -> &mut ::protobuf::RepeatedField<HDNodePathType> {
        &mut self.pubkeys
    }

    // Take field
    pub fn take_pubkeys(&mut self) -> ::protobuf::RepeatedField<HDNodePathType> {
        ::std::mem::replace(&mut self.pubkeys, ::protobuf::RepeatedField::new())
    }

    pub fn get_pubkeys(&self) -> &[HDNodePathType] {
        &self.pubkeys
    }

    fn get_pubkeys_for_reflect(&self) -> &::protobuf::RepeatedField<HDNodePathType> {
        &self.pubkeys
    }

    fn mut_pubkeys_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<HDNodePathType> {
        &mut self.pubkeys
    }

    // repeated bytes signatures = 2;

    pub fn clear_signatures(&mut self) {
        self.signatures.clear();
    }

    // Param is passed by value, moved
    pub fn set_signatures(&mut self, v: ::protobuf::RepeatedField<::std::vec::Vec<u8>>) {
        self.signatures = v;
    }

    // Mutable pointer to the field.
    pub fn mut_signatures(&mut self) -> &mut ::protobuf::RepeatedField<::std::vec::Vec<u8>> {
        &mut self.signatures
    }

    // Take field
    pub fn take_signatures(&mut self) -> ::protobuf::RepeatedField<::std::vec::Vec<u8>> {
        ::std::mem::replace(&mut self.signatures, ::protobuf::RepeatedField::new())
    }

    pub fn get_signatures(&self) -> &[::std::vec::Vec<u8>] {
        &self.signatures
    }

    fn get_signatures_for_reflect(&self) -> &::protobuf::RepeatedField<::std::vec::Vec<u8>> {
        &self.signatures
    }

    fn mut_signatures_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<::std::vec::Vec<u8>> {
        &mut self.signatures
    }

    // optional uint32 m = 3;

    pub fn clear_m(&mut self) {
        self.m = ::std::option::Option::None;
    }

    pub fn has_m(&self) -> bool {
        self.m.is_some()
    }

    // Param is passed by value, moved
    pub fn set_m(&mut self, v: u32) {
        self.m = ::std::option::Option::Some(v);
    }

    pub fn get_m(&self) -> u32 {
        self.m.unwrap_or(0)
    }

    fn get_m_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.m
    }

    fn mut_m_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.m
    }
}

impl ::protobuf::Message for MultisigRedeemScriptType {
    fn is_initialized(&self) -> bool {
        for v in &self.pubkeys {
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
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.pubkeys)?;
                },
                2 => {
                    ::protobuf::rt::read_repeated_bytes_into(wire_type, is, &mut self.signatures)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.m = ::std::option::Option::Some(tmp);
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
        for value in &self.pubkeys {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        for value in &self.signatures {
            my_size += ::protobuf::rt::bytes_size(2, &value);
        };
        if let Some(v) = self.m {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.pubkeys {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        for v in &self.signatures {
            os.write_bytes(2, &v)?;
        };
        if let Some(v) = self.m {
            os.write_uint32(3, v)?;
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

impl ::protobuf::MessageStatic for MultisigRedeemScriptType {
    fn new() -> MultisigRedeemScriptType {
        MultisigRedeemScriptType::new()
    }

    fn descriptor_static(_: ::std::option::Option<MultisigRedeemScriptType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<HDNodePathType>>(
                    "pubkeys",
                    MultisigRedeemScriptType::get_pubkeys_for_reflect,
                    MultisigRedeemScriptType::mut_pubkeys_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signatures",
                    MultisigRedeemScriptType::get_signatures_for_reflect,
                    MultisigRedeemScriptType::mut_signatures_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "m",
                    MultisigRedeemScriptType::get_m_for_reflect,
                    MultisigRedeemScriptType::mut_m_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<MultisigRedeemScriptType>(
                    "MultisigRedeemScriptType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for MultisigRedeemScriptType {
    fn clear(&mut self) {
        self.clear_pubkeys();
        self.clear_signatures();
        self.clear_m();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for MultisigRedeemScriptType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for MultisigRedeemScriptType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TxInputType {
    // message fields
    address_n: ::std::vec::Vec<u32>,
    prev_hash: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    prev_index: ::std::option::Option<u32>,
    script_sig: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    sequence: ::std::option::Option<u32>,
    script_type: ::std::option::Option<InputScriptType>,
    multisig: ::protobuf::SingularPtrField<MultisigRedeemScriptType>,
    amount: ::std::option::Option<u64>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TxInputType {}

impl TxInputType {
    pub fn new() -> TxInputType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TxInputType {
        static mut instance: ::protobuf::lazy::Lazy<TxInputType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TxInputType,
        };
        unsafe {
            instance.get(TxInputType::new)
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

    // required bytes prev_hash = 2;

    pub fn clear_prev_hash(&mut self) {
        self.prev_hash.clear();
    }

    pub fn has_prev_hash(&self) -> bool {
        self.prev_hash.is_some()
    }

    // Param is passed by value, moved
    pub fn set_prev_hash(&mut self, v: ::std::vec::Vec<u8>) {
        self.prev_hash = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_prev_hash(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.prev_hash.is_none() {
            self.prev_hash.set_default();
        }
        self.prev_hash.as_mut().unwrap()
    }

    // Take field
    pub fn take_prev_hash(&mut self) -> ::std::vec::Vec<u8> {
        self.prev_hash.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_prev_hash(&self) -> &[u8] {
        match self.prev_hash.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_prev_hash_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.prev_hash
    }

    fn mut_prev_hash_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.prev_hash
    }

    // required uint32 prev_index = 3;

    pub fn clear_prev_index(&mut self) {
        self.prev_index = ::std::option::Option::None;
    }

    pub fn has_prev_index(&self) -> bool {
        self.prev_index.is_some()
    }

    // Param is passed by value, moved
    pub fn set_prev_index(&mut self, v: u32) {
        self.prev_index = ::std::option::Option::Some(v);
    }

    pub fn get_prev_index(&self) -> u32 {
        self.prev_index.unwrap_or(0)
    }

    fn get_prev_index_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.prev_index
    }

    fn mut_prev_index_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.prev_index
    }

    // optional bytes script_sig = 4;

    pub fn clear_script_sig(&mut self) {
        self.script_sig.clear();
    }

    pub fn has_script_sig(&self) -> bool {
        self.script_sig.is_some()
    }

    // Param is passed by value, moved
    pub fn set_script_sig(&mut self, v: ::std::vec::Vec<u8>) {
        self.script_sig = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_script_sig(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.script_sig.is_none() {
            self.script_sig.set_default();
        }
        self.script_sig.as_mut().unwrap()
    }

    // Take field
    pub fn take_script_sig(&mut self) -> ::std::vec::Vec<u8> {
        self.script_sig.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_script_sig(&self) -> &[u8] {
        match self.script_sig.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_script_sig_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.script_sig
    }

    fn mut_script_sig_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.script_sig
    }

    // optional uint32 sequence = 5;

    pub fn clear_sequence(&mut self) {
        self.sequence = ::std::option::Option::None;
    }

    pub fn has_sequence(&self) -> bool {
        self.sequence.is_some()
    }

    // Param is passed by value, moved
    pub fn set_sequence(&mut self, v: u32) {
        self.sequence = ::std::option::Option::Some(v);
    }

    pub fn get_sequence(&self) -> u32 {
        self.sequence.unwrap_or(4294967295u32)
    }

    fn get_sequence_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.sequence
    }

    fn mut_sequence_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.sequence
    }

    // optional .InputScriptType script_type = 6;

    pub fn clear_script_type(&mut self) {
        self.script_type = ::std::option::Option::None;
    }

    pub fn has_script_type(&self) -> bool {
        self.script_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_script_type(&mut self, v: InputScriptType) {
        self.script_type = ::std::option::Option::Some(v);
    }

    pub fn get_script_type(&self) -> InputScriptType {
        self.script_type.unwrap_or(InputScriptType::SPENDADDRESS)
    }

    fn get_script_type_for_reflect(&self) -> &::std::option::Option<InputScriptType> {
        &self.script_type
    }

    fn mut_script_type_for_reflect(&mut self) -> &mut ::std::option::Option<InputScriptType> {
        &mut self.script_type
    }

    // optional .MultisigRedeemScriptType multisig = 7;

    pub fn clear_multisig(&mut self) {
        self.multisig.clear();
    }

    pub fn has_multisig(&self) -> bool {
        self.multisig.is_some()
    }

    // Param is passed by value, moved
    pub fn set_multisig(&mut self, v: MultisigRedeemScriptType) {
        self.multisig = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_multisig(&mut self) -> &mut MultisigRedeemScriptType {
        if self.multisig.is_none() {
            self.multisig.set_default();
        }
        self.multisig.as_mut().unwrap()
    }

    // Take field
    pub fn take_multisig(&mut self) -> MultisigRedeemScriptType {
        self.multisig.take().unwrap_or_else(|| MultisigRedeemScriptType::new())
    }

    pub fn get_multisig(&self) -> &MultisigRedeemScriptType {
        self.multisig.as_ref().unwrap_or_else(|| MultisigRedeemScriptType::default_instance())
    }

    fn get_multisig_for_reflect(&self) -> &::protobuf::SingularPtrField<MultisigRedeemScriptType> {
        &self.multisig
    }

    fn mut_multisig_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<MultisigRedeemScriptType> {
        &mut self.multisig
    }

    // optional uint64 amount = 8;

    pub fn clear_amount(&mut self) {
        self.amount = ::std::option::Option::None;
    }

    pub fn has_amount(&self) -> bool {
        self.amount.is_some()
    }

    // Param is passed by value, moved
    pub fn set_amount(&mut self, v: u64) {
        self.amount = ::std::option::Option::Some(v);
    }

    pub fn get_amount(&self) -> u64 {
        self.amount.unwrap_or(0)
    }

    fn get_amount_for_reflect(&self) -> &::std::option::Option<u64> {
        &self.amount
    }

    fn mut_amount_for_reflect(&mut self) -> &mut ::std::option::Option<u64> {
        &mut self.amount
    }
}

impl ::protobuf::Message for TxInputType {
    fn is_initialized(&self) -> bool {
        if self.prev_hash.is_none() {
            return false;
        }
        if self.prev_index.is_none() {
            return false;
        }
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
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.prev_hash)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.prev_index = ::std::option::Option::Some(tmp);
                },
                4 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.script_sig)?;
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.sequence = ::std::option::Option::Some(tmp);
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.script_type = ::std::option::Option::Some(tmp);
                },
                7 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.multisig)?;
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.amount = ::std::option::Option::Some(tmp);
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
        if let Some(ref v) = self.prev_hash.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(v) = self.prev_index {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.script_sig.as_ref() {
            my_size += ::protobuf::rt::bytes_size(4, &v);
        }
        if let Some(v) = self.sequence {
            my_size += ::protobuf::rt::value_size(5, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.script_type {
            my_size += ::protobuf::rt::enum_size(6, v);
        }
        if let Some(ref v) = self.multisig.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(v) = self.amount {
            my_size += ::protobuf::rt::value_size(8, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.address_n {
            os.write_uint32(1, *v)?;
        };
        if let Some(ref v) = self.prev_hash.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(v) = self.prev_index {
            os.write_uint32(3, v)?;
        }
        if let Some(ref v) = self.script_sig.as_ref() {
            os.write_bytes(4, &v)?;
        }
        if let Some(v) = self.sequence {
            os.write_uint32(5, v)?;
        }
        if let Some(v) = self.script_type {
            os.write_enum(6, v.value())?;
        }
        if let Some(ref v) = self.multisig.as_ref() {
            os.write_tag(7, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(v) = self.amount {
            os.write_uint64(8, v)?;
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

impl ::protobuf::MessageStatic for TxInputType {
    fn new() -> TxInputType {
        TxInputType::new()
    }

    fn descriptor_static(_: ::std::option::Option<TxInputType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    TxInputType::get_address_n_for_reflect,
                    TxInputType::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "prev_hash",
                    TxInputType::get_prev_hash_for_reflect,
                    TxInputType::mut_prev_hash_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "prev_index",
                    TxInputType::get_prev_index_for_reflect,
                    TxInputType::mut_prev_index_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "script_sig",
                    TxInputType::get_script_sig_for_reflect,
                    TxInputType::mut_script_sig_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "sequence",
                    TxInputType::get_sequence_for_reflect,
                    TxInputType::mut_sequence_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<InputScriptType>>(
                    "script_type",
                    TxInputType::get_script_type_for_reflect,
                    TxInputType::mut_script_type_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<MultisigRedeemScriptType>>(
                    "multisig",
                    TxInputType::get_multisig_for_reflect,
                    TxInputType::mut_multisig_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "amount",
                    TxInputType::get_amount_for_reflect,
                    TxInputType::mut_amount_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TxInputType>(
                    "TxInputType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TxInputType {
    fn clear(&mut self) {
        self.clear_address_n();
        self.clear_prev_hash();
        self.clear_prev_index();
        self.clear_script_sig();
        self.clear_sequence();
        self.clear_script_type();
        self.clear_multisig();
        self.clear_amount();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TxInputType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TxInputType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TxOutputType {
    // message fields
    address: ::protobuf::SingularField<::std::string::String>,
    address_n: ::std::vec::Vec<u32>,
    amount: ::std::option::Option<u64>,
    script_type: ::std::option::Option<OutputScriptType>,
    multisig: ::protobuf::SingularPtrField<MultisigRedeemScriptType>,
    op_return_data: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TxOutputType {}

impl TxOutputType {
    pub fn new() -> TxOutputType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TxOutputType {
        static mut instance: ::protobuf::lazy::Lazy<TxOutputType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TxOutputType,
        };
        unsafe {
            instance.get(TxOutputType::new)
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

    // repeated uint32 address_n = 2;

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

    // required uint64 amount = 3;

    pub fn clear_amount(&mut self) {
        self.amount = ::std::option::Option::None;
    }

    pub fn has_amount(&self) -> bool {
        self.amount.is_some()
    }

    // Param is passed by value, moved
    pub fn set_amount(&mut self, v: u64) {
        self.amount = ::std::option::Option::Some(v);
    }

    pub fn get_amount(&self) -> u64 {
        self.amount.unwrap_or(0)
    }

    fn get_amount_for_reflect(&self) -> &::std::option::Option<u64> {
        &self.amount
    }

    fn mut_amount_for_reflect(&mut self) -> &mut ::std::option::Option<u64> {
        &mut self.amount
    }

    // required .OutputScriptType script_type = 4;

    pub fn clear_script_type(&mut self) {
        self.script_type = ::std::option::Option::None;
    }

    pub fn has_script_type(&self) -> bool {
        self.script_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_script_type(&mut self, v: OutputScriptType) {
        self.script_type = ::std::option::Option::Some(v);
    }

    pub fn get_script_type(&self) -> OutputScriptType {
        self.script_type.unwrap_or(OutputScriptType::PAYTOADDRESS)
    }

    fn get_script_type_for_reflect(&self) -> &::std::option::Option<OutputScriptType> {
        &self.script_type
    }

    fn mut_script_type_for_reflect(&mut self) -> &mut ::std::option::Option<OutputScriptType> {
        &mut self.script_type
    }

    // optional .MultisigRedeemScriptType multisig = 5;

    pub fn clear_multisig(&mut self) {
        self.multisig.clear();
    }

    pub fn has_multisig(&self) -> bool {
        self.multisig.is_some()
    }

    // Param is passed by value, moved
    pub fn set_multisig(&mut self, v: MultisigRedeemScriptType) {
        self.multisig = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_multisig(&mut self) -> &mut MultisigRedeemScriptType {
        if self.multisig.is_none() {
            self.multisig.set_default();
        }
        self.multisig.as_mut().unwrap()
    }

    // Take field
    pub fn take_multisig(&mut self) -> MultisigRedeemScriptType {
        self.multisig.take().unwrap_or_else(|| MultisigRedeemScriptType::new())
    }

    pub fn get_multisig(&self) -> &MultisigRedeemScriptType {
        self.multisig.as_ref().unwrap_or_else(|| MultisigRedeemScriptType::default_instance())
    }

    fn get_multisig_for_reflect(&self) -> &::protobuf::SingularPtrField<MultisigRedeemScriptType> {
        &self.multisig
    }

    fn mut_multisig_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<MultisigRedeemScriptType> {
        &mut self.multisig
    }

    // optional bytes op_return_data = 6;

    pub fn clear_op_return_data(&mut self) {
        self.op_return_data.clear();
    }

    pub fn has_op_return_data(&self) -> bool {
        self.op_return_data.is_some()
    }

    // Param is passed by value, moved
    pub fn set_op_return_data(&mut self, v: ::std::vec::Vec<u8>) {
        self.op_return_data = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_op_return_data(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.op_return_data.is_none() {
            self.op_return_data.set_default();
        }
        self.op_return_data.as_mut().unwrap()
    }

    // Take field
    pub fn take_op_return_data(&mut self) -> ::std::vec::Vec<u8> {
        self.op_return_data.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_op_return_data(&self) -> &[u8] {
        match self.op_return_data.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_op_return_data_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.op_return_data
    }

    fn mut_op_return_data_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.op_return_data
    }
}

impl ::protobuf::Message for TxOutputType {
    fn is_initialized(&self) -> bool {
        if self.amount.is_none() {
            return false;
        }
        if self.script_type.is_none() {
            return false;
        }
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
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.address)?;
                },
                2 => {
                    ::protobuf::rt::read_repeated_uint32_into(wire_type, is, &mut self.address_n)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.amount = ::std::option::Option::Some(tmp);
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.script_type = ::std::option::Option::Some(tmp);
                },
                5 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.multisig)?;
                },
                6 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.op_return_data)?;
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
        for value in &self.address_n {
            my_size += ::protobuf::rt::value_size(2, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        if let Some(v) = self.amount {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.script_type {
            my_size += ::protobuf::rt::enum_size(4, v);
        }
        if let Some(ref v) = self.multisig.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.op_return_data.as_ref() {
            my_size += ::protobuf::rt::bytes_size(6, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.address.as_ref() {
            os.write_string(1, &v)?;
        }
        for v in &self.address_n {
            os.write_uint32(2, *v)?;
        };
        if let Some(v) = self.amount {
            os.write_uint64(3, v)?;
        }
        if let Some(v) = self.script_type {
            os.write_enum(4, v.value())?;
        }
        if let Some(ref v) = self.multisig.as_ref() {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.op_return_data.as_ref() {
            os.write_bytes(6, &v)?;
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

impl ::protobuf::MessageStatic for TxOutputType {
    fn new() -> TxOutputType {
        TxOutputType::new()
    }

    fn descriptor_static(_: ::std::option::Option<TxOutputType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "address",
                    TxOutputType::get_address_for_reflect,
                    TxOutputType::mut_address_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "address_n",
                    TxOutputType::get_address_n_for_reflect,
                    TxOutputType::mut_address_n_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "amount",
                    TxOutputType::get_amount_for_reflect,
                    TxOutputType::mut_amount_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeEnum<OutputScriptType>>(
                    "script_type",
                    TxOutputType::get_script_type_for_reflect,
                    TxOutputType::mut_script_type_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<MultisigRedeemScriptType>>(
                    "multisig",
                    TxOutputType::get_multisig_for_reflect,
                    TxOutputType::mut_multisig_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "op_return_data",
                    TxOutputType::get_op_return_data_for_reflect,
                    TxOutputType::mut_op_return_data_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TxOutputType>(
                    "TxOutputType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TxOutputType {
    fn clear(&mut self) {
        self.clear_address();
        self.clear_address_n();
        self.clear_amount();
        self.clear_script_type();
        self.clear_multisig();
        self.clear_op_return_data();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TxOutputType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TxOutputType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TxOutputBinType {
    // message fields
    amount: ::std::option::Option<u64>,
    script_pubkey: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TxOutputBinType {}

impl TxOutputBinType {
    pub fn new() -> TxOutputBinType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TxOutputBinType {
        static mut instance: ::protobuf::lazy::Lazy<TxOutputBinType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TxOutputBinType,
        };
        unsafe {
            instance.get(TxOutputBinType::new)
        }
    }

    // required uint64 amount = 1;

    pub fn clear_amount(&mut self) {
        self.amount = ::std::option::Option::None;
    }

    pub fn has_amount(&self) -> bool {
        self.amount.is_some()
    }

    // Param is passed by value, moved
    pub fn set_amount(&mut self, v: u64) {
        self.amount = ::std::option::Option::Some(v);
    }

    pub fn get_amount(&self) -> u64 {
        self.amount.unwrap_or(0)
    }

    fn get_amount_for_reflect(&self) -> &::std::option::Option<u64> {
        &self.amount
    }

    fn mut_amount_for_reflect(&mut self) -> &mut ::std::option::Option<u64> {
        &mut self.amount
    }

    // required bytes script_pubkey = 2;

    pub fn clear_script_pubkey(&mut self) {
        self.script_pubkey.clear();
    }

    pub fn has_script_pubkey(&self) -> bool {
        self.script_pubkey.is_some()
    }

    // Param is passed by value, moved
    pub fn set_script_pubkey(&mut self, v: ::std::vec::Vec<u8>) {
        self.script_pubkey = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_script_pubkey(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.script_pubkey.is_none() {
            self.script_pubkey.set_default();
        }
        self.script_pubkey.as_mut().unwrap()
    }

    // Take field
    pub fn take_script_pubkey(&mut self) -> ::std::vec::Vec<u8> {
        self.script_pubkey.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_script_pubkey(&self) -> &[u8] {
        match self.script_pubkey.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_script_pubkey_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.script_pubkey
    }

    fn mut_script_pubkey_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.script_pubkey
    }
}

impl ::protobuf::Message for TxOutputBinType {
    fn is_initialized(&self) -> bool {
        if self.amount.is_none() {
            return false;
        }
        if self.script_pubkey.is_none() {
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
                    let tmp = is.read_uint64()?;
                    self.amount = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.script_pubkey)?;
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
        if let Some(v) = self.amount {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.script_pubkey.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.amount {
            os.write_uint64(1, v)?;
        }
        if let Some(ref v) = self.script_pubkey.as_ref() {
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

impl ::protobuf::MessageStatic for TxOutputBinType {
    fn new() -> TxOutputBinType {
        TxOutputBinType::new()
    }

    fn descriptor_static(_: ::std::option::Option<TxOutputBinType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "amount",
                    TxOutputBinType::get_amount_for_reflect,
                    TxOutputBinType::mut_amount_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "script_pubkey",
                    TxOutputBinType::get_script_pubkey_for_reflect,
                    TxOutputBinType::mut_script_pubkey_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TxOutputBinType>(
                    "TxOutputBinType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TxOutputBinType {
    fn clear(&mut self) {
        self.clear_amount();
        self.clear_script_pubkey();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TxOutputBinType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TxOutputBinType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TransactionType {
    // message fields
    version: ::std::option::Option<u32>,
    inputs: ::protobuf::RepeatedField<TxInputType>,
    bin_outputs: ::protobuf::RepeatedField<TxOutputBinType>,
    outputs: ::protobuf::RepeatedField<TxOutputType>,
    lock_time: ::std::option::Option<u32>,
    inputs_cnt: ::std::option::Option<u32>,
    outputs_cnt: ::std::option::Option<u32>,
    extra_data: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    extra_data_len: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TransactionType {}

impl TransactionType {
    pub fn new() -> TransactionType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TransactionType {
        static mut instance: ::protobuf::lazy::Lazy<TransactionType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TransactionType,
        };
        unsafe {
            instance.get(TransactionType::new)
        }
    }

    // optional uint32 version = 1;

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
        self.version.unwrap_or(0)
    }

    fn get_version_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.version
    }

    fn mut_version_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.version
    }

    // repeated .TxInputType inputs = 2;

    pub fn clear_inputs(&mut self) {
        self.inputs.clear();
    }

    // Param is passed by value, moved
    pub fn set_inputs(&mut self, v: ::protobuf::RepeatedField<TxInputType>) {
        self.inputs = v;
    }

    // Mutable pointer to the field.
    pub fn mut_inputs(&mut self) -> &mut ::protobuf::RepeatedField<TxInputType> {
        &mut self.inputs
    }

    // Take field
    pub fn take_inputs(&mut self) -> ::protobuf::RepeatedField<TxInputType> {
        ::std::mem::replace(&mut self.inputs, ::protobuf::RepeatedField::new())
    }

    pub fn get_inputs(&self) -> &[TxInputType] {
        &self.inputs
    }

    fn get_inputs_for_reflect(&self) -> &::protobuf::RepeatedField<TxInputType> {
        &self.inputs
    }

    fn mut_inputs_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<TxInputType> {
        &mut self.inputs
    }

    // repeated .TxOutputBinType bin_outputs = 3;

    pub fn clear_bin_outputs(&mut self) {
        self.bin_outputs.clear();
    }

    // Param is passed by value, moved
    pub fn set_bin_outputs(&mut self, v: ::protobuf::RepeatedField<TxOutputBinType>) {
        self.bin_outputs = v;
    }

    // Mutable pointer to the field.
    pub fn mut_bin_outputs(&mut self) -> &mut ::protobuf::RepeatedField<TxOutputBinType> {
        &mut self.bin_outputs
    }

    // Take field
    pub fn take_bin_outputs(&mut self) -> ::protobuf::RepeatedField<TxOutputBinType> {
        ::std::mem::replace(&mut self.bin_outputs, ::protobuf::RepeatedField::new())
    }

    pub fn get_bin_outputs(&self) -> &[TxOutputBinType] {
        &self.bin_outputs
    }

    fn get_bin_outputs_for_reflect(&self) -> &::protobuf::RepeatedField<TxOutputBinType> {
        &self.bin_outputs
    }

    fn mut_bin_outputs_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<TxOutputBinType> {
        &mut self.bin_outputs
    }

    // repeated .TxOutputType outputs = 5;

    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }

    // Param is passed by value, moved
    pub fn set_outputs(&mut self, v: ::protobuf::RepeatedField<TxOutputType>) {
        self.outputs = v;
    }

    // Mutable pointer to the field.
    pub fn mut_outputs(&mut self) -> &mut ::protobuf::RepeatedField<TxOutputType> {
        &mut self.outputs
    }

    // Take field
    pub fn take_outputs(&mut self) -> ::protobuf::RepeatedField<TxOutputType> {
        ::std::mem::replace(&mut self.outputs, ::protobuf::RepeatedField::new())
    }

    pub fn get_outputs(&self) -> &[TxOutputType] {
        &self.outputs
    }

    fn get_outputs_for_reflect(&self) -> &::protobuf::RepeatedField<TxOutputType> {
        &self.outputs
    }

    fn mut_outputs_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<TxOutputType> {
        &mut self.outputs
    }

    // optional uint32 lock_time = 4;

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
        self.lock_time.unwrap_or(0)
    }

    fn get_lock_time_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.lock_time
    }

    fn mut_lock_time_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.lock_time
    }

    // optional uint32 inputs_cnt = 6;

    pub fn clear_inputs_cnt(&mut self) {
        self.inputs_cnt = ::std::option::Option::None;
    }

    pub fn has_inputs_cnt(&self) -> bool {
        self.inputs_cnt.is_some()
    }

    // Param is passed by value, moved
    pub fn set_inputs_cnt(&mut self, v: u32) {
        self.inputs_cnt = ::std::option::Option::Some(v);
    }

    pub fn get_inputs_cnt(&self) -> u32 {
        self.inputs_cnt.unwrap_or(0)
    }

    fn get_inputs_cnt_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.inputs_cnt
    }

    fn mut_inputs_cnt_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.inputs_cnt
    }

    // optional uint32 outputs_cnt = 7;

    pub fn clear_outputs_cnt(&mut self) {
        self.outputs_cnt = ::std::option::Option::None;
    }

    pub fn has_outputs_cnt(&self) -> bool {
        self.outputs_cnt.is_some()
    }

    // Param is passed by value, moved
    pub fn set_outputs_cnt(&mut self, v: u32) {
        self.outputs_cnt = ::std::option::Option::Some(v);
    }

    pub fn get_outputs_cnt(&self) -> u32 {
        self.outputs_cnt.unwrap_or(0)
    }

    fn get_outputs_cnt_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.outputs_cnt
    }

    fn mut_outputs_cnt_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.outputs_cnt
    }

    // optional bytes extra_data = 8;

    pub fn clear_extra_data(&mut self) {
        self.extra_data.clear();
    }

    pub fn has_extra_data(&self) -> bool {
        self.extra_data.is_some()
    }

    // Param is passed by value, moved
    pub fn set_extra_data(&mut self, v: ::std::vec::Vec<u8>) {
        self.extra_data = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_extra_data(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.extra_data.is_none() {
            self.extra_data.set_default();
        }
        self.extra_data.as_mut().unwrap()
    }

    // Take field
    pub fn take_extra_data(&mut self) -> ::std::vec::Vec<u8> {
        self.extra_data.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_extra_data(&self) -> &[u8] {
        match self.extra_data.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_extra_data_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.extra_data
    }

    fn mut_extra_data_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.extra_data
    }

    // optional uint32 extra_data_len = 9;

    pub fn clear_extra_data_len(&mut self) {
        self.extra_data_len = ::std::option::Option::None;
    }

    pub fn has_extra_data_len(&self) -> bool {
        self.extra_data_len.is_some()
    }

    // Param is passed by value, moved
    pub fn set_extra_data_len(&mut self, v: u32) {
        self.extra_data_len = ::std::option::Option::Some(v);
    }

    pub fn get_extra_data_len(&self) -> u32 {
        self.extra_data_len.unwrap_or(0)
    }

    fn get_extra_data_len_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.extra_data_len
    }

    fn mut_extra_data_len_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.extra_data_len
    }
}

impl ::protobuf::Message for TransactionType {
    fn is_initialized(&self) -> bool {
        for v in &self.inputs {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.bin_outputs {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.outputs {
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
                    let tmp = is.read_uint32()?;
                    self.version = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.inputs)?;
                },
                3 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.bin_outputs)?;
                },
                5 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.outputs)?;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.lock_time = ::std::option::Option::Some(tmp);
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.inputs_cnt = ::std::option::Option::Some(tmp);
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.outputs_cnt = ::std::option::Option::Some(tmp);
                },
                8 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.extra_data)?;
                },
                9 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.extra_data_len = ::std::option::Option::Some(tmp);
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
        if let Some(v) = self.version {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        for value in &self.inputs {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        for value in &self.bin_outputs {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        for value in &self.outputs {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        if let Some(v) = self.lock_time {
            my_size += ::protobuf::rt::value_size(4, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.inputs_cnt {
            my_size += ::protobuf::rt::value_size(6, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.outputs_cnt {
            my_size += ::protobuf::rt::value_size(7, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.extra_data.as_ref() {
            my_size += ::protobuf::rt::bytes_size(8, &v);
        }
        if let Some(v) = self.extra_data_len {
            my_size += ::protobuf::rt::value_size(9, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.version {
            os.write_uint32(1, v)?;
        }
        for v in &self.inputs {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        for v in &self.bin_outputs {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        for v in &self.outputs {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        if let Some(v) = self.lock_time {
            os.write_uint32(4, v)?;
        }
        if let Some(v) = self.inputs_cnt {
            os.write_uint32(6, v)?;
        }
        if let Some(v) = self.outputs_cnt {
            os.write_uint32(7, v)?;
        }
        if let Some(ref v) = self.extra_data.as_ref() {
            os.write_bytes(8, &v)?;
        }
        if let Some(v) = self.extra_data_len {
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

impl ::protobuf::MessageStatic for TransactionType {
    fn new() -> TransactionType {
        TransactionType::new()
    }

    fn descriptor_static(_: ::std::option::Option<TransactionType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "version",
                    TransactionType::get_version_for_reflect,
                    TransactionType::mut_version_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<TxInputType>>(
                    "inputs",
                    TransactionType::get_inputs_for_reflect,
                    TransactionType::mut_inputs_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<TxOutputBinType>>(
                    "bin_outputs",
                    TransactionType::get_bin_outputs_for_reflect,
                    TransactionType::mut_bin_outputs_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<TxOutputType>>(
                    "outputs",
                    TransactionType::get_outputs_for_reflect,
                    TransactionType::mut_outputs_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "lock_time",
                    TransactionType::get_lock_time_for_reflect,
                    TransactionType::mut_lock_time_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "inputs_cnt",
                    TransactionType::get_inputs_cnt_for_reflect,
                    TransactionType::mut_inputs_cnt_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "outputs_cnt",
                    TransactionType::get_outputs_cnt_for_reflect,
                    TransactionType::mut_outputs_cnt_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "extra_data",
                    TransactionType::get_extra_data_for_reflect,
                    TransactionType::mut_extra_data_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "extra_data_len",
                    TransactionType::get_extra_data_len_for_reflect,
                    TransactionType::mut_extra_data_len_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TransactionType>(
                    "TransactionType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TransactionType {
    fn clear(&mut self) {
        self.clear_version();
        self.clear_inputs();
        self.clear_bin_outputs();
        self.clear_outputs();
        self.clear_lock_time();
        self.clear_inputs_cnt();
        self.clear_outputs_cnt();
        self.clear_extra_data();
        self.clear_extra_data_len();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TransactionType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TransactionType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TxRequestDetailsType {
    // message fields
    request_index: ::std::option::Option<u32>,
    tx_hash: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    extra_data_len: ::std::option::Option<u32>,
    extra_data_offset: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TxRequestDetailsType {}

impl TxRequestDetailsType {
    pub fn new() -> TxRequestDetailsType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TxRequestDetailsType {
        static mut instance: ::protobuf::lazy::Lazy<TxRequestDetailsType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TxRequestDetailsType,
        };
        unsafe {
            instance.get(TxRequestDetailsType::new)
        }
    }

    // optional uint32 request_index = 1;

    pub fn clear_request_index(&mut self) {
        self.request_index = ::std::option::Option::None;
    }

    pub fn has_request_index(&self) -> bool {
        self.request_index.is_some()
    }

    // Param is passed by value, moved
    pub fn set_request_index(&mut self, v: u32) {
        self.request_index = ::std::option::Option::Some(v);
    }

    pub fn get_request_index(&self) -> u32 {
        self.request_index.unwrap_or(0)
    }

    fn get_request_index_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.request_index
    }

    fn mut_request_index_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.request_index
    }

    // optional bytes tx_hash = 2;

    pub fn clear_tx_hash(&mut self) {
        self.tx_hash.clear();
    }

    pub fn has_tx_hash(&self) -> bool {
        self.tx_hash.is_some()
    }

    // Param is passed by value, moved
    pub fn set_tx_hash(&mut self, v: ::std::vec::Vec<u8>) {
        self.tx_hash = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_tx_hash(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.tx_hash.is_none() {
            self.tx_hash.set_default();
        }
        self.tx_hash.as_mut().unwrap()
    }

    // Take field
    pub fn take_tx_hash(&mut self) -> ::std::vec::Vec<u8> {
        self.tx_hash.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_tx_hash(&self) -> &[u8] {
        match self.tx_hash.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_tx_hash_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.tx_hash
    }

    fn mut_tx_hash_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.tx_hash
    }

    // optional uint32 extra_data_len = 3;

    pub fn clear_extra_data_len(&mut self) {
        self.extra_data_len = ::std::option::Option::None;
    }

    pub fn has_extra_data_len(&self) -> bool {
        self.extra_data_len.is_some()
    }

    // Param is passed by value, moved
    pub fn set_extra_data_len(&mut self, v: u32) {
        self.extra_data_len = ::std::option::Option::Some(v);
    }

    pub fn get_extra_data_len(&self) -> u32 {
        self.extra_data_len.unwrap_or(0)
    }

    fn get_extra_data_len_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.extra_data_len
    }

    fn mut_extra_data_len_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.extra_data_len
    }

    // optional uint32 extra_data_offset = 4;

    pub fn clear_extra_data_offset(&mut self) {
        self.extra_data_offset = ::std::option::Option::None;
    }

    pub fn has_extra_data_offset(&self) -> bool {
        self.extra_data_offset.is_some()
    }

    // Param is passed by value, moved
    pub fn set_extra_data_offset(&mut self, v: u32) {
        self.extra_data_offset = ::std::option::Option::Some(v);
    }

    pub fn get_extra_data_offset(&self) -> u32 {
        self.extra_data_offset.unwrap_or(0)
    }

    fn get_extra_data_offset_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.extra_data_offset
    }

    fn mut_extra_data_offset_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.extra_data_offset
    }
}

impl ::protobuf::Message for TxRequestDetailsType {
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
                    self.request_index = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.tx_hash)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.extra_data_len = ::std::option::Option::Some(tmp);
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.extra_data_offset = ::std::option::Option::Some(tmp);
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
        if let Some(v) = self.request_index {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.tx_hash.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(v) = self.extra_data_len {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.extra_data_offset {
            my_size += ::protobuf::rt::value_size(4, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.request_index {
            os.write_uint32(1, v)?;
        }
        if let Some(ref v) = self.tx_hash.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(v) = self.extra_data_len {
            os.write_uint32(3, v)?;
        }
        if let Some(v) = self.extra_data_offset {
            os.write_uint32(4, v)?;
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

impl ::protobuf::MessageStatic for TxRequestDetailsType {
    fn new() -> TxRequestDetailsType {
        TxRequestDetailsType::new()
    }

    fn descriptor_static(_: ::std::option::Option<TxRequestDetailsType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "request_index",
                    TxRequestDetailsType::get_request_index_for_reflect,
                    TxRequestDetailsType::mut_request_index_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "tx_hash",
                    TxRequestDetailsType::get_tx_hash_for_reflect,
                    TxRequestDetailsType::mut_tx_hash_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "extra_data_len",
                    TxRequestDetailsType::get_extra_data_len_for_reflect,
                    TxRequestDetailsType::mut_extra_data_len_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "extra_data_offset",
                    TxRequestDetailsType::get_extra_data_offset_for_reflect,
                    TxRequestDetailsType::mut_extra_data_offset_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TxRequestDetailsType>(
                    "TxRequestDetailsType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TxRequestDetailsType {
    fn clear(&mut self) {
        self.clear_request_index();
        self.clear_tx_hash();
        self.clear_extra_data_len();
        self.clear_extra_data_offset();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TxRequestDetailsType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TxRequestDetailsType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TxRequestSerializedType {
    // message fields
    signature_index: ::std::option::Option<u32>,
    signature: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    serialized_tx: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TxRequestSerializedType {}

impl TxRequestSerializedType {
    pub fn new() -> TxRequestSerializedType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TxRequestSerializedType {
        static mut instance: ::protobuf::lazy::Lazy<TxRequestSerializedType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TxRequestSerializedType,
        };
        unsafe {
            instance.get(TxRequestSerializedType::new)
        }
    }

    // optional uint32 signature_index = 1;

    pub fn clear_signature_index(&mut self) {
        self.signature_index = ::std::option::Option::None;
    }

    pub fn has_signature_index(&self) -> bool {
        self.signature_index.is_some()
    }

    // Param is passed by value, moved
    pub fn set_signature_index(&mut self, v: u32) {
        self.signature_index = ::std::option::Option::Some(v);
    }

    pub fn get_signature_index(&self) -> u32 {
        self.signature_index.unwrap_or(0)
    }

    fn get_signature_index_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.signature_index
    }

    fn mut_signature_index_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.signature_index
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

    // optional bytes serialized_tx = 3;

    pub fn clear_serialized_tx(&mut self) {
        self.serialized_tx.clear();
    }

    pub fn has_serialized_tx(&self) -> bool {
        self.serialized_tx.is_some()
    }

    // Param is passed by value, moved
    pub fn set_serialized_tx(&mut self, v: ::std::vec::Vec<u8>) {
        self.serialized_tx = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_serialized_tx(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.serialized_tx.is_none() {
            self.serialized_tx.set_default();
        }
        self.serialized_tx.as_mut().unwrap()
    }

    // Take field
    pub fn take_serialized_tx(&mut self) -> ::std::vec::Vec<u8> {
        self.serialized_tx.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_serialized_tx(&self) -> &[u8] {
        match self.serialized_tx.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }

    fn get_serialized_tx_for_reflect(&self) -> &::protobuf::SingularField<::std::vec::Vec<u8>> {
        &self.serialized_tx
    }

    fn mut_serialized_tx_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::vec::Vec<u8>> {
        &mut self.serialized_tx
    }
}

impl ::protobuf::Message for TxRequestSerializedType {
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
                    self.signature_index = ::std::option::Option::Some(tmp);
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.signature)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.serialized_tx)?;
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
        if let Some(v) = self.signature_index {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(ref v) = self.signature.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        if let Some(ref v) = self.serialized_tx.as_ref() {
            my_size += ::protobuf::rt::bytes_size(3, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.signature_index {
            os.write_uint32(1, v)?;
        }
        if let Some(ref v) = self.signature.as_ref() {
            os.write_bytes(2, &v)?;
        }
        if let Some(ref v) = self.serialized_tx.as_ref() {
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

impl ::protobuf::MessageStatic for TxRequestSerializedType {
    fn new() -> TxRequestSerializedType {
        TxRequestSerializedType::new()
    }

    fn descriptor_static(_: ::std::option::Option<TxRequestSerializedType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "signature_index",
                    TxRequestSerializedType::get_signature_index_for_reflect,
                    TxRequestSerializedType::mut_signature_index_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "signature",
                    TxRequestSerializedType::get_signature_for_reflect,
                    TxRequestSerializedType::mut_signature_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                    "serialized_tx",
                    TxRequestSerializedType::get_serialized_tx_for_reflect,
                    TxRequestSerializedType::mut_serialized_tx_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TxRequestSerializedType>(
                    "TxRequestSerializedType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TxRequestSerializedType {
    fn clear(&mut self) {
        self.clear_signature_index();
        self.clear_signature();
        self.clear_serialized_tx();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TxRequestSerializedType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TxRequestSerializedType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct IdentityType {
    // message fields
    proto: ::protobuf::SingularField<::std::string::String>,
    user: ::protobuf::SingularField<::std::string::String>,
    host: ::protobuf::SingularField<::std::string::String>,
    port: ::protobuf::SingularField<::std::string::String>,
    path: ::protobuf::SingularField<::std::string::String>,
    index: ::std::option::Option<u32>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for IdentityType {}

impl IdentityType {
    pub fn new() -> IdentityType {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static IdentityType {
        static mut instance: ::protobuf::lazy::Lazy<IdentityType> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const IdentityType,
        };
        unsafe {
            instance.get(IdentityType::new)
        }
    }

    // optional string proto = 1;

    pub fn clear_proto(&mut self) {
        self.proto.clear();
    }

    pub fn has_proto(&self) -> bool {
        self.proto.is_some()
    }

    // Param is passed by value, moved
    pub fn set_proto(&mut self, v: ::std::string::String) {
        self.proto = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_proto(&mut self) -> &mut ::std::string::String {
        if self.proto.is_none() {
            self.proto.set_default();
        }
        self.proto.as_mut().unwrap()
    }

    // Take field
    pub fn take_proto(&mut self) -> ::std::string::String {
        self.proto.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_proto(&self) -> &str {
        match self.proto.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_proto_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.proto
    }

    fn mut_proto_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.proto
    }

    // optional string user = 2;

    pub fn clear_user(&mut self) {
        self.user.clear();
    }

    pub fn has_user(&self) -> bool {
        self.user.is_some()
    }

    // Param is passed by value, moved
    pub fn set_user(&mut self, v: ::std::string::String) {
        self.user = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_user(&mut self) -> &mut ::std::string::String {
        if self.user.is_none() {
            self.user.set_default();
        }
        self.user.as_mut().unwrap()
    }

    // Take field
    pub fn take_user(&mut self) -> ::std::string::String {
        self.user.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_user(&self) -> &str {
        match self.user.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_user_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.user
    }

    fn mut_user_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.user
    }

    // optional string host = 3;

    pub fn clear_host(&mut self) {
        self.host.clear();
    }

    pub fn has_host(&self) -> bool {
        self.host.is_some()
    }

    // Param is passed by value, moved
    pub fn set_host(&mut self, v: ::std::string::String) {
        self.host = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_host(&mut self) -> &mut ::std::string::String {
        if self.host.is_none() {
            self.host.set_default();
        }
        self.host.as_mut().unwrap()
    }

    // Take field
    pub fn take_host(&mut self) -> ::std::string::String {
        self.host.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_host(&self) -> &str {
        match self.host.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_host_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.host
    }

    fn mut_host_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.host
    }

    // optional string port = 4;

    pub fn clear_port(&mut self) {
        self.port.clear();
    }

    pub fn has_port(&self) -> bool {
        self.port.is_some()
    }

    // Param is passed by value, moved
    pub fn set_port(&mut self, v: ::std::string::String) {
        self.port = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_port(&mut self) -> &mut ::std::string::String {
        if self.port.is_none() {
            self.port.set_default();
        }
        self.port.as_mut().unwrap()
    }

    // Take field
    pub fn take_port(&mut self) -> ::std::string::String {
        self.port.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_port(&self) -> &str {
        match self.port.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_port_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.port
    }

    fn mut_port_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.port
    }

    // optional string path = 5;

    pub fn clear_path(&mut self) {
        self.path.clear();
    }

    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }

    // Param is passed by value, moved
    pub fn set_path(&mut self, v: ::std::string::String) {
        self.path = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_path(&mut self) -> &mut ::std::string::String {
        if self.path.is_none() {
            self.path.set_default();
        }
        self.path.as_mut().unwrap()
    }

    // Take field
    pub fn take_path(&mut self) -> ::std::string::String {
        self.path.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_path(&self) -> &str {
        match self.path.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    fn get_path_for_reflect(&self) -> &::protobuf::SingularField<::std::string::String> {
        &self.path
    }

    fn mut_path_for_reflect(&mut self) -> &mut ::protobuf::SingularField<::std::string::String> {
        &mut self.path
    }

    // optional uint32 index = 6;

    pub fn clear_index(&mut self) {
        self.index = ::std::option::Option::None;
    }

    pub fn has_index(&self) -> bool {
        self.index.is_some()
    }

    // Param is passed by value, moved
    pub fn set_index(&mut self, v: u32) {
        self.index = ::std::option::Option::Some(v);
    }

    pub fn get_index(&self) -> u32 {
        self.index.unwrap_or(0u32)
    }

    fn get_index_for_reflect(&self) -> &::std::option::Option<u32> {
        &self.index
    }

    fn mut_index_for_reflect(&mut self) -> &mut ::std::option::Option<u32> {
        &mut self.index
    }
}

impl ::protobuf::Message for IdentityType {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.proto)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.user)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.host)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.port)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.path)?;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.index = ::std::option::Option::Some(tmp);
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
        if let Some(ref v) = self.proto.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(ref v) = self.user.as_ref() {
            my_size += ::protobuf::rt::string_size(2, &v);
        }
        if let Some(ref v) = self.host.as_ref() {
            my_size += ::protobuf::rt::string_size(3, &v);
        }
        if let Some(ref v) = self.port.as_ref() {
            my_size += ::protobuf::rt::string_size(4, &v);
        }
        if let Some(ref v) = self.path.as_ref() {
            my_size += ::protobuf::rt::string_size(5, &v);
        }
        if let Some(v) = self.index {
            my_size += ::protobuf::rt::value_size(6, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.proto.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(ref v) = self.user.as_ref() {
            os.write_string(2, &v)?;
        }
        if let Some(ref v) = self.host.as_ref() {
            os.write_string(3, &v)?;
        }
        if let Some(ref v) = self.port.as_ref() {
            os.write_string(4, &v)?;
        }
        if let Some(ref v) = self.path.as_ref() {
            os.write_string(5, &v)?;
        }
        if let Some(v) = self.index {
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

impl ::protobuf::MessageStatic for IdentityType {
    fn new() -> IdentityType {
        IdentityType::new()
    }

    fn descriptor_static(_: ::std::option::Option<IdentityType>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "proto",
                    IdentityType::get_proto_for_reflect,
                    IdentityType::mut_proto_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "user",
                    IdentityType::get_user_for_reflect,
                    IdentityType::mut_user_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "host",
                    IdentityType::get_host_for_reflect,
                    IdentityType::mut_host_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "port",
                    IdentityType::get_port_for_reflect,
                    IdentityType::mut_port_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "path",
                    IdentityType::get_path_for_reflect,
                    IdentityType::mut_path_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "index",
                    IdentityType::get_index_for_reflect,
                    IdentityType::mut_index_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<IdentityType>(
                    "IdentityType",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for IdentityType {
    fn clear(&mut self) {
        self.clear_proto();
        self.clear_user();
        self.clear_host();
        self.clear_port();
        self.clear_path();
        self.clear_index();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for IdentityType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for IdentityType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum FailureType {
    Failure_UnexpectedMessage = 1,
    Failure_ButtonExpected = 2,
    Failure_DataError = 3,
    Failure_ActionCancelled = 4,
    Failure_PinExpected = 5,
    Failure_PinCancelled = 6,
    Failure_PinInvalid = 7,
    Failure_InvalidSignature = 8,
    Failure_ProcessError = 9,
    Failure_NotEnoughFunds = 10,
    Failure_NotInitialized = 11,
    Failure_FirmwareError = 99,
}

impl ::protobuf::ProtobufEnum for FailureType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<FailureType> {
        match value {
            1 => ::std::option::Option::Some(FailureType::Failure_UnexpectedMessage),
            2 => ::std::option::Option::Some(FailureType::Failure_ButtonExpected),
            3 => ::std::option::Option::Some(FailureType::Failure_DataError),
            4 => ::std::option::Option::Some(FailureType::Failure_ActionCancelled),
            5 => ::std::option::Option::Some(FailureType::Failure_PinExpected),
            6 => ::std::option::Option::Some(FailureType::Failure_PinCancelled),
            7 => ::std::option::Option::Some(FailureType::Failure_PinInvalid),
            8 => ::std::option::Option::Some(FailureType::Failure_InvalidSignature),
            9 => ::std::option::Option::Some(FailureType::Failure_ProcessError),
            10 => ::std::option::Option::Some(FailureType::Failure_NotEnoughFunds),
            11 => ::std::option::Option::Some(FailureType::Failure_NotInitialized),
            99 => ::std::option::Option::Some(FailureType::Failure_FirmwareError),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [FailureType] = &[
            FailureType::Failure_UnexpectedMessage,
            FailureType::Failure_ButtonExpected,
            FailureType::Failure_DataError,
            FailureType::Failure_ActionCancelled,
            FailureType::Failure_PinExpected,
            FailureType::Failure_PinCancelled,
            FailureType::Failure_PinInvalid,
            FailureType::Failure_InvalidSignature,
            FailureType::Failure_ProcessError,
            FailureType::Failure_NotEnoughFunds,
            FailureType::Failure_NotInitialized,
            FailureType::Failure_FirmwareError,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<FailureType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("FailureType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for FailureType {
}

impl ::protobuf::reflect::ProtobufValue for FailureType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum OutputScriptType {
    PAYTOADDRESS = 0,
    PAYTOSCRIPTHASH = 1,
    PAYTOMULTISIG = 2,
    PAYTOOPRETURN = 3,
    PAYTOWITNESS = 4,
    PAYTOP2SHWITNESS = 5,
}

impl ::protobuf::ProtobufEnum for OutputScriptType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<OutputScriptType> {
        match value {
            0 => ::std::option::Option::Some(OutputScriptType::PAYTOADDRESS),
            1 => ::std::option::Option::Some(OutputScriptType::PAYTOSCRIPTHASH),
            2 => ::std::option::Option::Some(OutputScriptType::PAYTOMULTISIG),
            3 => ::std::option::Option::Some(OutputScriptType::PAYTOOPRETURN),
            4 => ::std::option::Option::Some(OutputScriptType::PAYTOWITNESS),
            5 => ::std::option::Option::Some(OutputScriptType::PAYTOP2SHWITNESS),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [OutputScriptType] = &[
            OutputScriptType::PAYTOADDRESS,
            OutputScriptType::PAYTOSCRIPTHASH,
            OutputScriptType::PAYTOMULTISIG,
            OutputScriptType::PAYTOOPRETURN,
            OutputScriptType::PAYTOWITNESS,
            OutputScriptType::PAYTOP2SHWITNESS,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<OutputScriptType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("OutputScriptType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for OutputScriptType {
}

impl ::protobuf::reflect::ProtobufValue for OutputScriptType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum InputScriptType {
    SPENDADDRESS = 0,
    SPENDMULTISIG = 1,
    EXTERNAL = 2,
    SPENDWITNESS = 3,
    SPENDP2SHWITNESS = 4,
}

impl ::protobuf::ProtobufEnum for InputScriptType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<InputScriptType> {
        match value {
            0 => ::std::option::Option::Some(InputScriptType::SPENDADDRESS),
            1 => ::std::option::Option::Some(InputScriptType::SPENDMULTISIG),
            2 => ::std::option::Option::Some(InputScriptType::EXTERNAL),
            3 => ::std::option::Option::Some(InputScriptType::SPENDWITNESS),
            4 => ::std::option::Option::Some(InputScriptType::SPENDP2SHWITNESS),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [InputScriptType] = &[
            InputScriptType::SPENDADDRESS,
            InputScriptType::SPENDMULTISIG,
            InputScriptType::EXTERNAL,
            InputScriptType::SPENDWITNESS,
            InputScriptType::SPENDP2SHWITNESS,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<InputScriptType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("InputScriptType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for InputScriptType {
}

impl ::protobuf::reflect::ProtobufValue for InputScriptType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum RequestType {
    TXINPUT = 0,
    TXOUTPUT = 1,
    TXMETA = 2,
    TXFINISHED = 3,
    TXEXTRADATA = 4,
}

impl ::protobuf::ProtobufEnum for RequestType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<RequestType> {
        match value {
            0 => ::std::option::Option::Some(RequestType::TXINPUT),
            1 => ::std::option::Option::Some(RequestType::TXOUTPUT),
            2 => ::std::option::Option::Some(RequestType::TXMETA),
            3 => ::std::option::Option::Some(RequestType::TXFINISHED),
            4 => ::std::option::Option::Some(RequestType::TXEXTRADATA),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [RequestType] = &[
            RequestType::TXINPUT,
            RequestType::TXOUTPUT,
            RequestType::TXMETA,
            RequestType::TXFINISHED,
            RequestType::TXEXTRADATA,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<RequestType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("RequestType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for RequestType {
}

impl ::protobuf::reflect::ProtobufValue for RequestType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
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
    ButtonRequest_PublicKey = 11,
}

impl ::protobuf::ProtobufEnum for ButtonRequestType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<ButtonRequestType> {
        match value {
            1 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_Other),
            2 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_FeeOverThreshold),
            3 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_ConfirmOutput),
            4 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_ResetDevice),
            5 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_ConfirmWord),
            6 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_WipeDevice),
            7 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_ProtectCall),
            8 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_SignTx),
            9 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_FirmwareCheck),
            10 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_Address),
            11 => ::std::option::Option::Some(ButtonRequestType::ButtonRequest_PublicKey),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [ButtonRequestType] = &[
            ButtonRequestType::ButtonRequest_Other,
            ButtonRequestType::ButtonRequest_FeeOverThreshold,
            ButtonRequestType::ButtonRequest_ConfirmOutput,
            ButtonRequestType::ButtonRequest_ResetDevice,
            ButtonRequestType::ButtonRequest_ConfirmWord,
            ButtonRequestType::ButtonRequest_WipeDevice,
            ButtonRequestType::ButtonRequest_ProtectCall,
            ButtonRequestType::ButtonRequest_SignTx,
            ButtonRequestType::ButtonRequest_FirmwareCheck,
            ButtonRequestType::ButtonRequest_Address,
            ButtonRequestType::ButtonRequest_PublicKey,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<ButtonRequestType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("ButtonRequestType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for ButtonRequestType {
}

impl ::protobuf::reflect::ProtobufValue for ButtonRequestType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum PinMatrixRequestType {
    PinMatrixRequestType_Current = 1,
    PinMatrixRequestType_NewFirst = 2,
    PinMatrixRequestType_NewSecond = 3,
}

impl ::protobuf::ProtobufEnum for PinMatrixRequestType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<PinMatrixRequestType> {
        match value {
            1 => ::std::option::Option::Some(PinMatrixRequestType::PinMatrixRequestType_Current),
            2 => ::std::option::Option::Some(PinMatrixRequestType::PinMatrixRequestType_NewFirst),
            3 => ::std::option::Option::Some(PinMatrixRequestType::PinMatrixRequestType_NewSecond),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [PinMatrixRequestType] = &[
            PinMatrixRequestType::PinMatrixRequestType_Current,
            PinMatrixRequestType::PinMatrixRequestType_NewFirst,
            PinMatrixRequestType::PinMatrixRequestType_NewSecond,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<PinMatrixRequestType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("PinMatrixRequestType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for PinMatrixRequestType {
}

impl ::protobuf::reflect::ProtobufValue for PinMatrixRequestType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum RecoveryDeviceType {
    RecoveryDeviceType_ScrambledWords = 0,
    RecoveryDeviceType_Matrix = 1,
}

impl ::protobuf::ProtobufEnum for RecoveryDeviceType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<RecoveryDeviceType> {
        match value {
            0 => ::std::option::Option::Some(RecoveryDeviceType::RecoveryDeviceType_ScrambledWords),
            1 => ::std::option::Option::Some(RecoveryDeviceType::RecoveryDeviceType_Matrix),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [RecoveryDeviceType] = &[
            RecoveryDeviceType::RecoveryDeviceType_ScrambledWords,
            RecoveryDeviceType::RecoveryDeviceType_Matrix,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<RecoveryDeviceType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("RecoveryDeviceType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for RecoveryDeviceType {
}

impl ::protobuf::reflect::ProtobufValue for RecoveryDeviceType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum WordRequestType {
    WordRequestType_Plain = 0,
    WordRequestType_Matrix9 = 1,
    WordRequestType_Matrix6 = 2,
}

impl ::protobuf::ProtobufEnum for WordRequestType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<WordRequestType> {
        match value {
            0 => ::std::option::Option::Some(WordRequestType::WordRequestType_Plain),
            1 => ::std::option::Option::Some(WordRequestType::WordRequestType_Matrix9),
            2 => ::std::option::Option::Some(WordRequestType::WordRequestType_Matrix6),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [WordRequestType] = &[
            WordRequestType::WordRequestType_Plain,
            WordRequestType::WordRequestType_Matrix9,
            WordRequestType::WordRequestType_Matrix6,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<WordRequestType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("WordRequestType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for WordRequestType {
}

impl ::protobuf::reflect::ProtobufValue for WordRequestType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

pub mod exts {
    use protobuf::Message as Message_imported_for_functions;

    pub const wire_in: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::EnumValueOptions, ::protobuf::types::ProtobufTypeBool> = ::protobuf::ext::ExtFieldOptional { field_number: 50002, phantom: ::std::marker::PhantomData };

    pub const wire_out: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::EnumValueOptions, ::protobuf::types::ProtobufTypeBool> = ::protobuf::ext::ExtFieldOptional { field_number: 50003, phantom: ::std::marker::PhantomData };

    pub const wire_debug_in: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::EnumValueOptions, ::protobuf::types::ProtobufTypeBool> = ::protobuf::ext::ExtFieldOptional { field_number: 50004, phantom: ::std::marker::PhantomData };

    pub const wire_debug_out: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::EnumValueOptions, ::protobuf::types::ProtobufTypeBool> = ::protobuf::ext::ExtFieldOptional { field_number: 50005, phantom: ::std::marker::PhantomData };

    pub const wire_tiny: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::EnumValueOptions, ::protobuf::types::ProtobufTypeBool> = ::protobuf::ext::ExtFieldOptional { field_number: 50006, phantom: ::std::marker::PhantomData };

    pub const wire_bootloader: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::EnumValueOptions, ::protobuf::types::ProtobufTypeBool> = ::protobuf::ext::ExtFieldOptional { field_number: 50007, phantom: ::std::marker::PhantomData };
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0btypes.proto\x1a\x20google/protobuf/descriptor.proto\"\xc0\x01\n\nH\
    DNodeType\x12\x14\n\x05depth\x18\x01\x20\x02(\rR\x05depth\x12\x20\n\x0bf\
    ingerprint\x18\x02\x20\x02(\rR\x0bfingerprint\x12\x1b\n\tchild_num\x18\
    \x03\x20\x02(\rR\x08childNum\x12\x1d\n\nchain_code\x18\x04\x20\x02(\x0cR\
    \tchainCode\x12\x1f\n\x0bprivate_key\x18\x05\x20\x01(\x0cR\nprivateKey\
    \x12\x1d\n\npublic_key\x18\x06\x20\x01(\x0cR\tpublicKey\"N\n\x0eHDNodePa\
    thType\x12\x1f\n\x04node\x18\x01\x20\x02(\x0b2\x0b.HDNodeTypeR\x04node\
    \x12\x1b\n\taddress_n\x18\x02\x20\x03(\rR\x08addressN\"\xf4\x02\n\x08Coi\
    nType\x12\x1b\n\tcoin_name\x18\x01\x20\x01(\tR\x08coinName\x12#\n\rcoin_\
    shortcut\x18\x02\x20\x01(\tR\x0ccoinShortcut\x12$\n\x0caddress_type\x18\
    \x03\x20\x01(\r:\x010R\x0baddressType\x12\x1b\n\tmaxfee_kb\x18\x04\x20\
    \x01(\x04R\x08maxfeeKb\x12-\n\x11address_type_p2sh\x18\x05\x20\x01(\r:\
    \x015R\x0faddressTypeP2sh\x122\n\x15signed_message_header\x18\x08\x20\
    \x01(\tR\x13signedMessageHeader\x12'\n\nxpub_magic\x18\t\x20\x01(\r:\x08\
    76067358R\txpubMagic\x12'\n\nxprv_magic\x18\n\x20\x01(\r:\x0876066276R\t\
    xprvMagic\x12\x16\n\x06segwit\x18\x0b\x20\x01(\x08R\x06segwit\x12\x16\n\
    \x06forkid\x18\x0c\x20\x01(\rR\x06forkid\"s\n\x18MultisigRedeemScriptTyp\
    e\x12)\n\x07pubkeys\x18\x01\x20\x03(\x0b2\x0f.HDNodePathTypeR\x07pubkeys\
    \x12\x1e\n\nsignatures\x18\x02\x20\x03(\x0cR\nsignatures\x12\x0c\n\x01m\
    \x18\x03\x20\x01(\rR\x01m\"\xbd\x02\n\x0bTxInputType\x12\x1b\n\taddress_\
    n\x18\x01\x20\x03(\rR\x08addressN\x12\x1b\n\tprev_hash\x18\x02\x20\x02(\
    \x0cR\x08prevHash\x12\x1d\n\nprev_index\x18\x03\x20\x02(\rR\tprevIndex\
    \x12\x1d\n\nscript_sig\x18\x04\x20\x01(\x0cR\tscriptSig\x12&\n\x08sequen\
    ce\x18\x05\x20\x01(\r:\n4294967295R\x08sequence\x12?\n\x0bscript_type\
    \x18\x06\x20\x01(\x0e2\x10.InputScriptType:\x0cSPENDADDRESSR\nscriptType\
    \x125\n\x08multisig\x18\x07\x20\x01(\x0b2\x19.MultisigRedeemScriptTypeR\
    \x08multisig\x12\x16\n\x06amount\x18\x08\x20\x01(\x04R\x06amount\"\xee\
    \x01\n\x0cTxOutputType\x12\x18\n\x07address\x18\x01\x20\x01(\tR\x07addre\
    ss\x12\x1b\n\taddress_n\x18\x02\x20\x03(\rR\x08addressN\x12\x16\n\x06amo\
    unt\x18\x03\x20\x02(\x04R\x06amount\x122\n\x0bscript_type\x18\x04\x20\
    \x02(\x0e2\x11.OutputScriptTypeR\nscriptType\x125\n\x08multisig\x18\x05\
    \x20\x01(\x0b2\x19.MultisigRedeemScriptTypeR\x08multisig\x12$\n\x0eop_re\
    turn_data\x18\x06\x20\x01(\x0cR\x0copReturnData\"N\n\x0fTxOutputBinType\
    \x12\x16\n\x06amount\x18\x01\x20\x02(\x04R\x06amount\x12#\n\rscript_pubk\
    ey\x18\x02\x20\x02(\x0cR\x0cscriptPubkey\"\xcf\x02\n\x0fTransactionType\
    \x12\x18\n\x07version\x18\x01\x20\x01(\rR\x07version\x12$\n\x06inputs\
    \x18\x02\x20\x03(\x0b2\x0c.TxInputTypeR\x06inputs\x121\n\x0bbin_outputs\
    \x18\x03\x20\x03(\x0b2\x10.TxOutputBinTypeR\nbinOutputs\x12'\n\x07output\
    s\x18\x05\x20\x03(\x0b2\r.TxOutputTypeR\x07outputs\x12\x1b\n\tlock_time\
    \x18\x04\x20\x01(\rR\x08lockTime\x12\x1d\n\ninputs_cnt\x18\x06\x20\x01(\
    \rR\tinputsCnt\x12\x1f\n\x0boutputs_cnt\x18\x07\x20\x01(\rR\noutputsCnt\
    \x12\x1d\n\nextra_data\x18\x08\x20\x01(\x0cR\textraData\x12$\n\x0eextra_\
    data_len\x18\t\x20\x01(\rR\x0cextraDataLen\"\xa6\x01\n\x14TxRequestDetai\
    lsType\x12#\n\rrequest_index\x18\x01\x20\x01(\rR\x0crequestIndex\x12\x17\
    \n\x07tx_hash\x18\x02\x20\x01(\x0cR\x06txHash\x12$\n\x0eextra_data_len\
    \x18\x03\x20\x01(\rR\x0cextraDataLen\x12*\n\x11extra_data_offset\x18\x04\
    \x20\x01(\rR\x0fextraDataOffset\"\x85\x01\n\x17TxRequestSerializedType\
    \x12'\n\x0fsignature_index\x18\x01\x20\x01(\rR\x0esignatureIndex\x12\x1c\
    \n\tsignature\x18\x02\x20\x01(\x0cR\tsignature\x12#\n\rserialized_tx\x18\
    \x03\x20\x01(\x0cR\x0cserializedTx\"\x8d\x01\n\x0cIdentityType\x12\x14\n\
    \x05proto\x18\x01\x20\x01(\tR\x05proto\x12\x12\n\x04user\x18\x02\x20\x01\
    (\tR\x04user\x12\x12\n\x04host\x18\x03\x20\x01(\tR\x04host\x12\x12\n\x04\
    port\x18\x04\x20\x01(\tR\x04port\x12\x12\n\x04path\x18\x05\x20\x01(\tR\
    \x04path\x12\x17\n\x05index\x18\x06\x20\x01(\r:\x010R\x05index*\xd2\x02\
    \n\x0bFailureType\x12\x1d\n\x19Failure_UnexpectedMessage\x10\x01\x12\x1a\
    \n\x16Failure_ButtonExpected\x10\x02\x12\x15\n\x11Failure_DataError\x10\
    \x03\x12\x1b\n\x17Failure_ActionCancelled\x10\x04\x12\x17\n\x13Failure_P\
    inExpected\x10\x05\x12\x18\n\x14Failure_PinCancelled\x10\x06\x12\x16\n\
    \x12Failure_PinInvalid\x10\x07\x12\x1c\n\x18Failure_InvalidSignature\x10\
    \x08\x12\x18\n\x14Failure_ProcessError\x10\t\x12\x1a\n\x16Failure_NotEno\
    ughFunds\x10\n\x12\x1a\n\x16Failure_NotInitialized\x10\x0b\x12\x19\n\x15\
    Failure_FirmwareError\x10c*\x87\x01\n\x10OutputScriptType\x12\x10\n\x0cP\
    AYTOADDRESS\x10\0\x12\x13\n\x0fPAYTOSCRIPTHASH\x10\x01\x12\x11\n\rPAYTOM\
    ULTISIG\x10\x02\x12\x11\n\rPAYTOOPRETURN\x10\x03\x12\x10\n\x0cPAYTOWITNE\
    SS\x10\x04\x12\x14\n\x10PAYTOP2SHWITNESS\x10\x05*l\n\x0fInputScriptType\
    \x12\x10\n\x0cSPENDADDRESS\x10\0\x12\x11\n\rSPENDMULTISIG\x10\x01\x12\
    \x0c\n\x08EXTERNAL\x10\x02\x12\x10\n\x0cSPENDWITNESS\x10\x03\x12\x14\n\
    \x10SPENDP2SHWITNESS\x10\x04*U\n\x0bRequestType\x12\x0b\n\x07TXINPUT\x10\
    \0\x12\x0c\n\x08TXOUTPUT\x10\x01\x12\n\n\x06TXMETA\x10\x02\x12\x0e\n\nTX\
    FINISHED\x10\x03\x12\x0f\n\x0bTXEXTRADATA\x10\x04*\xdf\x02\n\x11ButtonRe\
    questType\x12\x17\n\x13ButtonRequest_Other\x10\x01\x12\"\n\x1eButtonRequ\
    est_FeeOverThreshold\x10\x02\x12\x1f\n\x1bButtonRequest_ConfirmOutput\
    \x10\x03\x12\x1d\n\x19ButtonRequest_ResetDevice\x10\x04\x12\x1d\n\x19But\
    tonRequest_ConfirmWord\x10\x05\x12\x1c\n\x18ButtonRequest_WipeDevice\x10\
    \x06\x12\x1d\n\x19ButtonRequest_ProtectCall\x10\x07\x12\x18\n\x14ButtonR\
    equest_SignTx\x10\x08\x12\x1f\n\x1bButtonRequest_FirmwareCheck\x10\t\x12\
    \x19\n\x15ButtonRequest_Address\x10\n\x12\x1b\n\x17ButtonRequest_PublicK\
    ey\x10\x0b*\x7f\n\x14PinMatrixRequestType\x12\x20\n\x1cPinMatrixRequestT\
    ype_Current\x10\x01\x12!\n\x1dPinMatrixRequestType_NewFirst\x10\x02\x12\
    \"\n\x1ePinMatrixRequestType_NewSecond\x10\x03*Z\n\x12RecoveryDeviceType\
    \x12%\n!RecoveryDeviceType_ScrambledWords\x10\0\x12\x1d\n\x19RecoveryDev\
    iceType_Matrix\x10\x01*f\n\x0fWordRequestType\x12\x19\n\x15WordRequestTy\
    pe_Plain\x10\0\x12\x1b\n\x17WordRequestType_Matrix9\x10\x01\x12\x1b\n\
    \x17WordRequestType_Matrix6\x10\x02:<\n\x07wire_in\x18\xd2\x86\x03\x20\
    \x01(\x08\x12!.google.protobuf.EnumValueOptionsR\x06wireIn:>\n\x08wire_o\
    ut\x18\xd3\x86\x03\x20\x01(\x08\x12!.google.protobuf.EnumValueOptionsR\
    \x07wireOut:G\n\rwire_debug_in\x18\xd4\x86\x03\x20\x01(\x08\x12!.google.\
    protobuf.EnumValueOptionsR\x0bwireDebugIn:I\n\x0ewire_debug_out\x18\xd5\
    \x86\x03\x20\x01(\x08\x12!.google.protobuf.EnumValueOptionsR\x0cwireDebu\
    gOut:@\n\twire_tiny\x18\xd6\x86\x03\x20\x01(\x08\x12!.google.protobuf.En\
    umValueOptionsR\x08wireTiny:L\n\x0fwire_bootloader\x18\xd7\x86\x03\x20\
    \x01(\x08\x12!.google.protobuf.EnumValueOptionsR\x0ewireBootloaderB1\n#c\
    om.satoshilabs.trezor.lib.protobufB\nTrezorTypeJ\xf6[\n\x07\x12\x05\x08\
    \0\x8f\x02\x01\n\x08\n\x01\x08\x12\x03\x08\0<\n\x94\x01\n\x04\x08\xe7\
    \x07\0\x12\x03\x08\0<\x1a#\x20Sugar\x20for\x20easier\x20handling\x20in\
    \x20Java\n2b*\n\x20Types\x20for\x20TREZOR\x20communication\n\n\x20@autho\
    r\tMarek\x20Palatinus\x20<slush@satoshilabs.com>\n\x20@version\t1.2\n\n\
    \x0c\n\x05\x08\xe7\x07\0\x02\x12\x03\x08\x07\x13\n\r\n\x06\x08\xe7\x07\0\
    \x02\0\x12\x03\x08\x07\x13\n\x0e\n\x07\x08\xe7\x07\0\x02\0\x01\x12\x03\
    \x08\x07\x13\n\x0c\n\x05\x08\xe7\x07\0\x07\x12\x03\x08\x16;\n\x08\n\x01\
    \x08\x12\x03\t\0+\n\x0b\n\x04\x08\xe7\x07\x01\x12\x03\t\0+\n\x0c\n\x05\
    \x08\xe7\x07\x01\x02\x12\x03\t\x07\x1b\n\r\n\x06\x08\xe7\x07\x01\x02\0\
    \x12\x03\t\x07\x1b\n\x0e\n\x07\x08\xe7\x07\x01\x02\0\x01\x12\x03\t\x07\
    \x1b\n\x0c\n\x05\x08\xe7\x07\x01\x07\x12\x03\t\x1e*\n\t\n\x02\x03\0\x12\
    \x03\x0b\x07)\nW\n\x01\x07\x12\x04\x10\0\x17\x01\x1aL*\n\x20Options\x20f\
    or\x20specifying\x20message\x20direction\x20and\x20type\x20of\x20wire\
    \x20(normal/debug)\n\nB\n\x02\x07\0\x12\x03\x11\x08&\"7\x20message\x20ca\
    n\x20be\x20transmitted\x20via\x20wire\x20from\x20PC\x20to\x20TREZOR\n\n\
    \n\n\x03\x07\0\x02\x12\x03\x10\x07'\n\n\n\x03\x07\0\x04\x12\x03\x11\x08\
    \x10\n\n\n\x03\x07\0\x05\x12\x03\x11\x11\x15\n\n\n\x03\x07\0\x01\x12\x03\
    \x11\x16\x1d\n\n\n\x03\x07\0\x03\x12\x03\x11\x20%\nB\n\x02\x07\x01\x12\
    \x03\x12\x08'\"7\x20message\x20can\x20be\x20transmitted\x20via\x20wire\
    \x20from\x20TREZOR\x20to\x20PC\n\n\n\n\x03\x07\x01\x02\x12\x03\x10\x07'\
    \n\n\n\x03\x07\x01\x04\x12\x03\x12\x08\x10\n\n\n\x03\x07\x01\x05\x12\x03\
    \x12\x11\x15\n\n\n\x03\x07\x01\x01\x12\x03\x12\x16\x1e\n\n\n\x03\x07\x01\
    \x03\x12\x03\x12!&\nH\n\x02\x07\x02\x12\x03\x13\x08,\"=\x20message\x20ca\
    n\x20be\x20transmitted\x20via\x20debug\x20wire\x20from\x20PC\x20to\x20TR\
    EZOR\n\n\n\n\x03\x07\x02\x02\x12\x03\x10\x07'\n\n\n\x03\x07\x02\x04\x12\
    \x03\x13\x08\x10\n\n\n\x03\x07\x02\x05\x12\x03\x13\x11\x15\n\n\n\x03\x07\
    \x02\x01\x12\x03\x13\x16#\n\n\n\x03\x07\x02\x03\x12\x03\x13&+\nH\n\x02\
    \x07\x03\x12\x03\x14\x08-\"=\x20message\x20can\x20be\x20transmitted\x20v\
    ia\x20debug\x20wire\x20from\x20TREZOR\x20to\x20PC\n\n\n\n\x03\x07\x03\
    \x02\x12\x03\x10\x07'\n\n\n\x03\x07\x03\x04\x12\x03\x14\x08\x10\n\n\n\
    \x03\x07\x03\x05\x12\x03\x14\x11\x15\n\n\n\x03\x07\x03\x01\x12\x03\x14\
    \x16$\n\n\n\x03\x07\x03\x03\x12\x03\x14',\nL\n\x02\x07\x04\x12\x03\x15\
    \x08(\"A\x20message\x20is\x20handled\x20by\x20TREZOR\x20when\x20the\x20U\
    SB\x20stack\x20is\x20in\x20tiny\x20mode\n\n\n\n\x03\x07\x04\x02\x12\x03\
    \x10\x07'\n\n\n\x03\x07\x04\x04\x12\x03\x15\x08\x10\n\n\n\x03\x07\x04\
    \x05\x12\x03\x15\x11\x15\n\n\n\x03\x07\x04\x01\x12\x03\x15\x16\x1f\n\n\n\
    \x03\x07\x04\x03\x12\x03\x15\"'\n9\n\x02\x07\x05\x12\x03\x16\x08.\".\x20\
    message\x20is\x20only\x20handled\x20by\x20TREZOR\x20Bootloader\n\n\n\n\
    \x03\x07\x05\x02\x12\x03\x10\x07'\n\n\n\x03\x07\x05\x04\x12\x03\x16\x08\
    \x10\n\n\n\x03\x07\x05\x05\x12\x03\x16\x11\x15\n\n\n\x03\x07\x05\x01\x12\
    \x03\x16\x16%\n\n\n\x03\x07\x05\x03\x12\x03\x16(-\nN\n\x02\x05\0\x12\x04\
    \x1d\0*\x01\x1aB*\n\x20Type\x20of\x20failures\x20returned\x20by\x20Failu\
    re\x20message\n\x20@used_in\x20Failure\n\n\n\n\x03\x05\0\x01\x12\x03\x1d\
    \x05\x10\n\x0b\n\x04\x05\0\x02\0\x12\x03\x1e\x08&\n\x0c\n\x05\x05\0\x02\
    \0\x01\x12\x03\x1e\x08!\n\x0c\n\x05\x05\0\x02\0\x02\x12\x03\x1e$%\n\x0b\
    \n\x04\x05\0\x02\x01\x12\x03\x1f\x08#\n\x0c\n\x05\x05\0\x02\x01\x01\x12\
    \x03\x1f\x08\x1e\n\x0c\n\x05\x05\0\x02\x01\x02\x12\x03\x1f!\"\n\x0b\n\
    \x04\x05\0\x02\x02\x12\x03\x20\x08\x1e\n\x0c\n\x05\x05\0\x02\x02\x01\x12\
    \x03\x20\x08\x19\n\x0c\n\x05\x05\0\x02\x02\x02\x12\x03\x20\x1c\x1d\n\x0b\
    \n\x04\x05\0\x02\x03\x12\x03!\x08$\n\x0c\n\x05\x05\0\x02\x03\x01\x12\x03\
    !\x08\x1f\n\x0c\n\x05\x05\0\x02\x03\x02\x12\x03!\"#\n\x0b\n\x04\x05\0\
    \x02\x04\x12\x03\"\x08\x20\n\x0c\n\x05\x05\0\x02\x04\x01\x12\x03\"\x08\
    \x1b\n\x0c\n\x05\x05\0\x02\x04\x02\x12\x03\"\x1e\x1f\n\x0b\n\x04\x05\0\
    \x02\x05\x12\x03#\x08!\n\x0c\n\x05\x05\0\x02\x05\x01\x12\x03#\x08\x1c\n\
    \x0c\n\x05\x05\0\x02\x05\x02\x12\x03#\x1f\x20\n\x0b\n\x04\x05\0\x02\x06\
    \x12\x03$\x08\x1f\n\x0c\n\x05\x05\0\x02\x06\x01\x12\x03$\x08\x1a\n\x0c\n\
    \x05\x05\0\x02\x06\x02\x12\x03$\x1d\x1e\n\x0b\n\x04\x05\0\x02\x07\x12\
    \x03%\x08%\n\x0c\n\x05\x05\0\x02\x07\x01\x12\x03%\x08\x20\n\x0c\n\x05\
    \x05\0\x02\x07\x02\x12\x03%#$\n\x0b\n\x04\x05\0\x02\x08\x12\x03&\x08!\n\
    \x0c\n\x05\x05\0\x02\x08\x01\x12\x03&\x08\x1c\n\x0c\n\x05\x05\0\x02\x08\
    \x02\x12\x03&\x1f\x20\n\x0b\n\x04\x05\0\x02\t\x12\x03'\x08$\n\x0c\n\x05\
    \x05\0\x02\t\x01\x12\x03'\x08\x1e\n\x0c\n\x05\x05\0\x02\t\x02\x12\x03'!#\
    \n\x0b\n\x04\x05\0\x02\n\x12\x03(\x08$\n\x0c\n\x05\x05\0\x02\n\x01\x12\
    \x03(\x08\x1e\n\x0c\n\x05\x05\0\x02\n\x02\x12\x03(!#\n\x0b\n\x04\x05\0\
    \x02\x0b\x12\x03)\x08#\n\x0c\n\x05\x05\0\x02\x0b\x01\x12\x03)\x08\x1d\n\
    \x0c\n\x05\x05\0\x02\x0b\x02\x12\x03)\x20\"\n_\n\x02\x05\x01\x12\x040\07\
    \x01\x1aS*\n\x20Type\x20of\x20script\x20which\x20will\x20be\x20used\x20f\
    or\x20transaction\x20output\n\x20@used_in\x20TxOutputType\n\n\n\n\x03\
    \x05\x01\x01\x12\x030\x05\x15\n>\n\x04\x05\x01\x02\0\x12\x031\x08\x19\"1\
    \x20used\x20for\x20all\x20addresses\x20(bitcoin,\x20p2sh,\x20witness)\n\
    \n\x0c\n\x05\x05\x01\x02\0\x01\x12\x031\x08\x14\n\x0c\n\x05\x05\x01\x02\
    \0\x02\x12\x031\x17\x18\n:\n\x04\x05\x01\x02\x01\x12\x032\x08\x1c\"-\x20\
    p2sh\x20address\x20(deprecated;\x20use\x20PAYTOADDRESS)\n\n\x0c\n\x05\
    \x05\x01\x02\x01\x01\x12\x032\x08\x17\n\x0c\n\x05\x05\x01\x02\x01\x02\
    \x12\x032\x1a\x1b\n%\n\x04\x05\x01\x02\x02\x12\x033\x08\x1a\"\x18\x20onl\
    y\x20for\x20change\x20output\n\n\x0c\n\x05\x05\x01\x02\x02\x01\x12\x033\
    \x08\x15\n\x0c\n\x05\x05\x01\x02\x02\x02\x12\x033\x18\x19\n\x18\n\x04\
    \x05\x01\x02\x03\x12\x034\x08\x1a\"\x0b\x20op_return\n\n\x0c\n\x05\x05\
    \x01\x02\x03\x01\x12\x034\x08\x15\n\x0c\n\x05\x05\x01\x02\x03\x02\x12\
    \x034\x18\x19\n%\n\x04\x05\x01\x02\x04\x12\x035\x08\x19\"\x18\x20only\
    \x20for\x20change\x20output\n\n\x0c\n\x05\x05\x01\x02\x04\x01\x12\x035\
    \x08\x14\n\x0c\n\x05\x05\x01\x02\x04\x02\x12\x035\x17\x18\n%\n\x04\x05\
    \x01\x02\x05\x12\x036\x08\x1d\"\x18\x20only\x20for\x20change\x20output\n\
    \n\x0c\n\x05\x05\x01\x02\x05\x01\x12\x036\x08\x18\n\x0c\n\x05\x05\x01\
    \x02\x05\x02\x12\x036\x1b\x1c\n^\n\x02\x05\x02\x12\x04=\0C\x01\x1aR*\n\
    \x20Type\x20of\x20script\x20which\x20will\x20be\x20used\x20for\x20transa\
    ction\x20output\n\x20@used_in\x20TxInputType\n\n\n\n\x03\x05\x02\x01\x12\
    \x03=\x05\x14\n%\n\x04\x05\x02\x02\0\x12\x03>\x08\x19\"\x18\x20standard\
    \x20p2pkh\x20address\n\n\x0c\n\x05\x05\x02\x02\0\x01\x12\x03>\x08\x14\n\
    \x0c\n\x05\x05\x02\x02\0\x02\x12\x03>\x17\x18\n$\n\x04\x05\x02\x02\x01\
    \x12\x03?\x08\x1a\"\x17\x20p2sh\x20multisig\x20address\n\n\x0c\n\x05\x05\
    \x02\x02\x01\x01\x12\x03?\x08\x15\n\x0c\n\x05\x05\x02\x02\x01\x02\x12\
    \x03?\x18\x19\n6\n\x04\x05\x02\x02\x02\x12\x03@\x08\x15\")\x20reserved\
    \x20for\x20external\x20inputs\x20(coinjoin)\n\n\x0c\n\x05\x05\x02\x02\
    \x02\x01\x12\x03@\x08\x10\n\x0c\n\x05\x05\x02\x02\x02\x02\x12\x03@\x13\
    \x14\n\x1c\n\x04\x05\x02\x02\x03\x12\x03A\x08\x19\"\x0f\x20native\x20seg\
    wit\n\n\x0c\n\x05\x05\x02\x02\x03\x01\x12\x03A\x08\x14\n\x0c\n\x05\x05\
    \x02\x02\x03\x02\x12\x03A\x17\x18\n5\n\x04\x05\x02\x02\x04\x12\x03B\x08\
    \x1d\"(\x20segwit\x20over\x20p2sh\x20(backward\x20compatible)\n\n\x0c\n\
    \x05\x05\x02\x02\x04\x01\x12\x03B\x08\x18\n\x0c\n\x05\x05\x02\x02\x04\
    \x02\x12\x03B\x1b\x1c\n_\n\x02\x05\x03\x12\x04I\0O\x01\x1aS*\n\x20Type\
    \x20of\x20information\x20required\x20by\x20transaction\x20signing\x20pro\
    cess\n\x20@used_in\x20TxRequest\n\n\n\n\x03\x05\x03\x01\x12\x03I\x05\x10\
    \n\x0b\n\x04\x05\x03\x02\0\x12\x03J\x08\x14\n\x0c\n\x05\x05\x03\x02\0\
    \x01\x12\x03J\x08\x0f\n\x0c\n\x05\x05\x03\x02\0\x02\x12\x03J\x12\x13\n\
    \x0b\n\x04\x05\x03\x02\x01\x12\x03K\x08\x15\n\x0c\n\x05\x05\x03\x02\x01\
    \x01\x12\x03K\x08\x10\n\x0c\n\x05\x05\x03\x02\x01\x02\x12\x03K\x13\x14\n\
    \x0b\n\x04\x05\x03\x02\x02\x12\x03L\x08\x13\n\x0c\n\x05\x05\x03\x02\x02\
    \x01\x12\x03L\x08\x0e\n\x0c\n\x05\x05\x03\x02\x02\x02\x12\x03L\x11\x12\n\
    \x0b\n\x04\x05\x03\x02\x03\x12\x03M\x08\x17\n\x0c\n\x05\x05\x03\x02\x03\
    \x01\x12\x03M\x08\x12\n\x0c\n\x05\x05\x03\x02\x03\x02\x12\x03M\x15\x16\n\
    \x0b\n\x04\x05\x03\x02\x04\x12\x03N\x08\x18\n\x0c\n\x05\x05\x03\x02\x04\
    \x01\x12\x03N\x08\x13\n\x0c\n\x05\x05\x03\x02\x04\x02\x12\x03N\x16\x17\n\
    >\n\x02\x05\x04\x12\x04U\0a\x01\x1a2*\n\x20Type\x20of\x20button\x20reque\
    st\n\x20@used_in\x20ButtonRequest\n\n\n\n\x03\x05\x04\x01\x12\x03U\x05\
    \x16\n\x0b\n\x04\x05\x04\x02\0\x12\x03V\x08\x20\n\x0c\n\x05\x05\x04\x02\
    \0\x01\x12\x03V\x08\x1b\n\x0c\n\x05\x05\x04\x02\0\x02\x12\x03V\x1e\x1f\n\
    \x0b\n\x04\x05\x04\x02\x01\x12\x03W\x08+\n\x0c\n\x05\x05\x04\x02\x01\x01\
    \x12\x03W\x08&\n\x0c\n\x05\x05\x04\x02\x01\x02\x12\x03W)*\n\x0b\n\x04\
    \x05\x04\x02\x02\x12\x03X\x08(\n\x0c\n\x05\x05\x04\x02\x02\x01\x12\x03X\
    \x08#\n\x0c\n\x05\x05\x04\x02\x02\x02\x12\x03X&'\n\x0b\n\x04\x05\x04\x02\
    \x03\x12\x03Y\x08&\n\x0c\n\x05\x05\x04\x02\x03\x01\x12\x03Y\x08!\n\x0c\n\
    \x05\x05\x04\x02\x03\x02\x12\x03Y$%\n\x0b\n\x04\x05\x04\x02\x04\x12\x03Z\
    \x08&\n\x0c\n\x05\x05\x04\x02\x04\x01\x12\x03Z\x08!\n\x0c\n\x05\x05\x04\
    \x02\x04\x02\x12\x03Z$%\n\x0b\n\x04\x05\x04\x02\x05\x12\x03[\x08%\n\x0c\
    \n\x05\x05\x04\x02\x05\x01\x12\x03[\x08\x20\n\x0c\n\x05\x05\x04\x02\x05\
    \x02\x12\x03[#$\n\x0b\n\x04\x05\x04\x02\x06\x12\x03\\\x08&\n\x0c\n\x05\
    \x05\x04\x02\x06\x01\x12\x03\\\x08!\n\x0c\n\x05\x05\x04\x02\x06\x02\x12\
    \x03\\$%\n\x0b\n\x04\x05\x04\x02\x07\x12\x03]\x08!\n\x0c\n\x05\x05\x04\
    \x02\x07\x01\x12\x03]\x08\x1c\n\x0c\n\x05\x05\x04\x02\x07\x02\x12\x03]\
    \x1f\x20\n\x0b\n\x04\x05\x04\x02\x08\x12\x03^\x08(\n\x0c\n\x05\x05\x04\
    \x02\x08\x01\x12\x03^\x08#\n\x0c\n\x05\x05\x04\x02\x08\x02\x12\x03^&'\n\
    \x0b\n\x04\x05\x04\x02\t\x12\x03_\x08#\n\x0c\n\x05\x05\x04\x02\t\x01\x12\
    \x03_\x08\x1d\n\x0c\n\x05\x05\x04\x02\t\x02\x12\x03_\x20\"\n\x0b\n\x04\
    \x05\x04\x02\n\x12\x03`\x08%\n\x0c\n\x05\x05\x04\x02\n\x01\x12\x03`\x08\
    \x1f\n\x0c\n\x05\x05\x04\x02\n\x02\x12\x03`\"$\n>\n\x02\x05\x05\x12\x04g\
    \0k\x01\x1a2*\n\x20Type\x20of\x20PIN\x20request\n\x20@used_in\x20PinMatr\
    ixRequest\n\n\n\n\x03\x05\x05\x01\x12\x03g\x05\x19\n\x0b\n\x04\x05\x05\
    \x02\0\x12\x03h\x08)\n\x0c\n\x05\x05\x05\x02\0\x01\x12\x03h\x08$\n\x0c\n\
    \x05\x05\x05\x02\0\x02\x12\x03h'(\n\x0b\n\x04\x05\x05\x02\x01\x12\x03i\
    \x08*\n\x0c\n\x05\x05\x05\x02\x01\x01\x12\x03i\x08%\n\x0c\n\x05\x05\x05\
    \x02\x01\x02\x12\x03i()\n\x0b\n\x04\x05\x05\x02\x02\x12\x03j\x08+\n\x0c\
    \n\x05\x05\x05\x02\x02\x01\x12\x03j\x08&\n\x0c\n\x05\x05\x05\x02\x02\x02\
    \x12\x03j)*\n\xe9\x02\n\x02\x05\x06\x12\x04w\0{\x01\x1a\xdc\x02*\n\x20Ty\
    pe\x20of\x20recovery\x20procedure.\x20These\x20should\x20be\x20used\x20a\
    s\x20bitmask,\x20e.g.,\n\x20`RecoveryDeviceType_ScrambledWords\x20|\x20R\
    ecoveryDeviceType_Matrix`\n\x20listing\x20every\x20method\x20supported\
    \x20by\x20the\x20host\x20computer.\n\n\x20Note\x20that\x20ScrambledWords\
    \x20must\x20be\x20supported\x20by\x20every\x20implementation\n\x20for\
    \x20backward\x20compatibility;\x20there\x20is\x20no\x20way\x20to\x20not\
    \x20support\x20it.\n\n\x20@used_in\x20RecoveryDevice\n\n\n\n\x03\x05\x06\
    \x01\x12\x03w\x05\x17\nV\n\x04\x05\x06\x02\0\x12\x03y\x08.\x1a-\x20use\
    \x20powers\x20of\x20two\x20when\x20extending\x20this\x20field\n\"\x1a\
    \x20words\x20in\x20scrambled\x20order\n\n\x0c\n\x05\x05\x06\x02\0\x01\
    \x12\x03y\x08)\n\x0c\n\x05\x05\x06\x02\0\x02\x12\x03y,-\n#\n\x04\x05\x06\
    \x02\x01\x12\x03z\x08&\"\x16\x20matrix\x20recovery\x20type\n\n\x0c\n\x05\
    \x05\x06\x02\x01\x01\x12\x03z\x08!\n\x0c\n\x05\x05\x06\x02\x01\x02\x12\
    \x03z$%\nE\n\x02\x05\x07\x12\x06\x81\x01\0\x85\x01\x01\x1a7*\n\x20Type\
    \x20of\x20Recovery\x20Word\x20request\n\x20@used_in\x20WordRequest\n\n\
    \x0b\n\x03\x05\x07\x01\x12\x04\x81\x01\x05\x14\n\x0c\n\x04\x05\x07\x02\0\
    \x12\x04\x82\x01\x08\"\n\r\n\x05\x05\x07\x02\0\x01\x12\x04\x82\x01\x08\
    \x1d\n\r\n\x05\x05\x07\x02\0\x02\x12\x04\x82\x01\x20!\n\x0c\n\x04\x05\
    \x07\x02\x01\x12\x04\x83\x01\x08$\n\r\n\x05\x05\x07\x02\x01\x01\x12\x04\
    \x83\x01\x08\x1f\n\r\n\x05\x05\x07\x02\x01\x02\x12\x04\x83\x01\"#\n\x0c\
    \n\x04\x05\x07\x02\x02\x12\x04\x84\x01\x08$\n\r\n\x05\x05\x07\x02\x02\
    \x01\x12\x04\x84\x01\x08\x1f\n\r\n\x05\x05\x07\x02\x02\x02\x12\x04\x84\
    \x01\"#\n\xfd\x01\n\x02\x04\0\x12\x06\x8f\x01\0\x96\x01\x01\x1a\xee\x01*\
    \n\x20Structure\x20representing\x20BIP32\x20(hierarchical\x20determinist\
    ic)\x20node\n\x20Used\x20for\x20imports\x20of\x20private\x20key\x20into\
    \x20the\x20device\x20and\x20exporting\x20public\x20key\x20out\x20of\x20d\
    evice\n\x20@used_in\x20PublicKey\n\x20@used_in\x20LoadDevice\n\x20@used_\
    in\x20DebugLinkState\n\x20@used_in\x20Storage\n\n\x0b\n\x03\x04\0\x01\
    \x12\x04\x8f\x01\x08\x12\n\x0c\n\x04\x04\0\x02\0\x12\x04\x90\x01\x08\"\n\
    \r\n\x05\x04\0\x02\0\x04\x12\x04\x90\x01\x08\x10\n\r\n\x05\x04\0\x02\0\
    \x05\x12\x04\x90\x01\x11\x17\n\r\n\x05\x04\0\x02\0\x01\x12\x04\x90\x01\
    \x18\x1d\n\r\n\x05\x04\0\x02\0\x03\x12\x04\x90\x01\x20!\n\x0c\n\x04\x04\
    \0\x02\x01\x12\x04\x91\x01\x08(\n\r\n\x05\x04\0\x02\x01\x04\x12\x04\x91\
    \x01\x08\x10\n\r\n\x05\x04\0\x02\x01\x05\x12\x04\x91\x01\x11\x17\n\r\n\
    \x05\x04\0\x02\x01\x01\x12\x04\x91\x01\x18#\n\r\n\x05\x04\0\x02\x01\x03\
    \x12\x04\x91\x01&'\n\x0c\n\x04\x04\0\x02\x02\x12\x04\x92\x01\x08&\n\r\n\
    \x05\x04\0\x02\x02\x04\x12\x04\x92\x01\x08\x10\n\r\n\x05\x04\0\x02\x02\
    \x05\x12\x04\x92\x01\x11\x17\n\r\n\x05\x04\0\x02\x02\x01\x12\x04\x92\x01\
    \x18!\n\r\n\x05\x04\0\x02\x02\x03\x12\x04\x92\x01$%\n\x0c\n\x04\x04\0\
    \x02\x03\x12\x04\x93\x01\x08&\n\r\n\x05\x04\0\x02\x03\x04\x12\x04\x93\
    \x01\x08\x10\n\r\n\x05\x04\0\x02\x03\x05\x12\x04\x93\x01\x11\x16\n\r\n\
    \x05\x04\0\x02\x03\x01\x12\x04\x93\x01\x17!\n\r\n\x05\x04\0\x02\x03\x03\
    \x12\x04\x93\x01$%\n\x0c\n\x04\x04\0\x02\x04\x12\x04\x94\x01\x08'\n\r\n\
    \x05\x04\0\x02\x04\x04\x12\x04\x94\x01\x08\x10\n\r\n\x05\x04\0\x02\x04\
    \x05\x12\x04\x94\x01\x11\x16\n\r\n\x05\x04\0\x02\x04\x01\x12\x04\x94\x01\
    \x17\"\n\r\n\x05\x04\0\x02\x04\x03\x12\x04\x94\x01%&\n\x0c\n\x04\x04\0\
    \x02\x05\x12\x04\x95\x01\x08&\n\r\n\x05\x04\0\x02\x05\x04\x12\x04\x95\
    \x01\x08\x10\n\r\n\x05\x04\0\x02\x05\x05\x12\x04\x95\x01\x11\x16\n\r\n\
    \x05\x04\0\x02\x05\x01\x12\x04\x95\x01\x17!\n\r\n\x05\x04\0\x02\x05\x03\
    \x12\x04\x95\x01$%\n\x0c\n\x02\x04\x01\x12\x06\x98\x01\0\x9b\x01\x01\n\
    \x0b\n\x03\x04\x01\x01\x12\x04\x98\x01\x08\x16\n0\n\x04\x04\x01\x02\0\
    \x12\x04\x99\x01\x08%\"\"\x20BIP-32\x20node\x20in\x20deserialized\x20for\
    m\n\n\r\n\x05\x04\x01\x02\0\x04\x12\x04\x99\x01\x08\x10\n\r\n\x05\x04\
    \x01\x02\0\x06\x12\x04\x99\x01\x11\x1b\n\r\n\x05\x04\x01\x02\0\x01\x12\
    \x04\x99\x01\x1c\x20\n\r\n\x05\x04\x01\x02\0\x03\x12\x04\x99\x01#$\n7\n\
    \x04\x04\x01\x02\x01\x12\x04\x9a\x01\x08&\")\x20BIP-32\x20path\x20to\x20\
    derive\x20the\x20key\x20from\x20node\n\n\r\n\x05\x04\x01\x02\x01\x04\x12\
    \x04\x9a\x01\x08\x10\n\r\n\x05\x04\x01\x02\x01\x05\x12\x04\x9a\x01\x11\
    \x17\n\r\n\x05\x04\x01\x02\x01\x01\x12\x04\x9a\x01\x18!\n\r\n\x05\x04\
    \x01\x02\x01\x03\x12\x04\x9a\x01$%\n@\n\x02\x04\x02\x12\x06\xa1\x01\0\
    \xac\x01\x01\x1a2*\n\x20Structure\x20representing\x20Coin\n\x20@used_in\
    \x20Features\n\n\x0b\n\x03\x04\x02\x01\x12\x04\xa1\x01\x08\x10\n\x0c\n\
    \x04\x04\x02\x02\0\x12\x04\xa2\x01\x08&\n\r\n\x05\x04\x02\x02\0\x04\x12\
    \x04\xa2\x01\x08\x10\n\r\n\x05\x04\x02\x02\0\x05\x12\x04\xa2\x01\x11\x17\
    \n\r\n\x05\x04\x02\x02\0\x01\x12\x04\xa2\x01\x18!\n\r\n\x05\x04\x02\x02\
    \0\x03\x12\x04\xa2\x01$%\n\x0c\n\x04\x04\x02\x02\x01\x12\x04\xa3\x01\x08\
    *\n\r\n\x05\x04\x02\x02\x01\x04\x12\x04\xa3\x01\x08\x10\n\r\n\x05\x04\
    \x02\x02\x01\x05\x12\x04\xa3\x01\x11\x17\n\r\n\x05\x04\x02\x02\x01\x01\
    \x12\x04\xa3\x01\x18%\n\r\n\x05\x04\x02\x02\x01\x03\x12\x04\xa3\x01()\n\
    \x0c\n\x04\x04\x02\x02\x02\x12\x04\xa4\x01\x085\n\r\n\x05\x04\x02\x02\
    \x02\x04\x12\x04\xa4\x01\x08\x10\n\r\n\x05\x04\x02\x02\x02\x05\x12\x04\
    \xa4\x01\x11\x17\n\r\n\x05\x04\x02\x02\x02\x01\x12\x04\xa4\x01\x18$\n\r\
    \n\x05\x04\x02\x02\x02\x03\x12\x04\xa4\x01'(\n\r\n\x05\x04\x02\x02\x02\
    \x08\x12\x04\xa4\x01)4\n\r\n\x05\x04\x02\x02\x02\x07\x12\x04\xa4\x0123\n\
    \x0c\n\x04\x04\x02\x02\x03\x12\x04\xa5\x01\x08&\n\r\n\x05\x04\x02\x02\
    \x03\x04\x12\x04\xa5\x01\x08\x10\n\r\n\x05\x04\x02\x02\x03\x05\x12\x04\
    \xa5\x01\x11\x17\n\r\n\x05\x04\x02\x02\x03\x01\x12\x04\xa5\x01\x18!\n\r\
    \n\x05\x04\x02\x02\x03\x03\x12\x04\xa5\x01$%\n\x0c\n\x04\x04\x02\x02\x04\
    \x12\x04\xa6\x01\x08:\n\r\n\x05\x04\x02\x02\x04\x04\x12\x04\xa6\x01\x08\
    \x10\n\r\n\x05\x04\x02\x02\x04\x05\x12\x04\xa6\x01\x11\x17\n\r\n\x05\x04\
    \x02\x02\x04\x01\x12\x04\xa6\x01\x18)\n\r\n\x05\x04\x02\x02\x04\x03\x12\
    \x04\xa6\x01,-\n\r\n\x05\x04\x02\x02\x04\x08\x12\x04\xa6\x01.9\n\r\n\x05\
    \x04\x02\x02\x04\x07\x12\x04\xa6\x0178\n\x0c\n\x04\x04\x02\x02\x05\x12\
    \x04\xa7\x01\x082\n\r\n\x05\x04\x02\x02\x05\x04\x12\x04\xa7\x01\x08\x10\
    \n\r\n\x05\x04\x02\x02\x05\x05\x12\x04\xa7\x01\x11\x17\n\r\n\x05\x04\x02\
    \x02\x05\x01\x12\x04\xa7\x01\x18-\n\r\n\x05\x04\x02\x02\x05\x03\x12\x04\
    \xa7\x0101\n\"\n\x04\x04\x02\x02\x06\x12\x04\xa8\x01\x08:\"\x14\x20defau\
    lt=0x0488b21e\n\n\r\n\x05\x04\x02\x02\x06\x04\x12\x04\xa8\x01\x08\x10\n\
    \r\n\x05\x04\x02\x02\x06\x05\x12\x04\xa8\x01\x11\x17\n\r\n\x05\x04\x02\
    \x02\x06\x01\x12\x04\xa8\x01\x18\"\n\r\n\x05\x04\x02\x02\x06\x03\x12\x04\
    \xa8\x01%&\n\r\n\x05\x04\x02\x02\x06\x08\x12\x04\xa8\x01'9\n\r\n\x05\x04\
    \x02\x02\x06\x07\x12\x04\xa8\x0108\n\"\n\x04\x04\x02\x02\x07\x12\x04\xa9\
    \x01\x08;\"\x14\x20default=0x0488ade4\n\n\r\n\x05\x04\x02\x02\x07\x04\
    \x12\x04\xa9\x01\x08\x10\n\r\n\x05\x04\x02\x02\x07\x05\x12\x04\xa9\x01\
    \x11\x17\n\r\n\x05\x04\x02\x02\x07\x01\x12\x04\xa9\x01\x18\"\n\r\n\x05\
    \x04\x02\x02\x07\x03\x12\x04\xa9\x01%'\n\r\n\x05\x04\x02\x02\x07\x08\x12\
    \x04\xa9\x01(:\n\r\n\x05\x04\x02\x02\x07\x07\x12\x04\xa9\x0119\n\x0c\n\
    \x04\x04\x02\x02\x08\x12\x04\xaa\x01\x08\"\n\r\n\x05\x04\x02\x02\x08\x04\
    \x12\x04\xaa\x01\x08\x10\n\r\n\x05\x04\x02\x02\x08\x05\x12\x04\xaa\x01\
    \x11\x15\n\r\n\x05\x04\x02\x02\x08\x01\x12\x04\xaa\x01\x16\x1c\n\r\n\x05\
    \x04\x02\x02\x08\x03\x12\x04\xaa\x01\x1f!\n\x0c\n\x04\x04\x02\x02\t\x12\
    \x04\xab\x01\x08$\n\r\n\x05\x04\x02\x02\t\x04\x12\x04\xab\x01\x08\x10\n\
    \r\n\x05\x04\x02\x02\t\x05\x12\x04\xab\x01\x11\x17\n\r\n\x05\x04\x02\x02\
    \t\x01\x12\x04\xab\x01\x18\x1e\n\r\n\x05\x04\x02\x02\t\x03\x12\x04\xab\
    \x01!#\nK\n\x02\x04\x03\x12\x06\xb2\x01\0\xb6\x01\x01\x1a=*\n\x20Type\
    \x20of\x20redeem\x20script\x20used\x20in\x20input\n\x20@used_in\x20TxInp\
    utType\n\n\x0b\n\x03\x04\x03\x01\x12\x04\xb2\x01\x08\x20\nH\n\x04\x04\
    \x03\x02\0\x12\x04\xb3\x01\x08,\":\x20pubkeys\x20from\x20multisig\x20add\
    ress\x20(sorted\x20lexicographically)\n\n\r\n\x05\x04\x03\x02\0\x04\x12\
    \x04\xb3\x01\x08\x10\n\r\n\x05\x04\x03\x02\0\x06\x12\x04\xb3\x01\x11\x1f\
    \n\r\n\x05\x04\x03\x02\0\x01\x12\x04\xb3\x01\x20'\n\r\n\x05\x04\x03\x02\
    \0\x03\x12\x04\xb3\x01*+\n>\n\x04\x04\x03\x02\x01\x12\x04\xb4\x01\x08&\"\
    0\x20existing\x20signatures\x20for\x20partially\x20signed\x20input\n\n\r\
    \n\x05\x04\x03\x02\x01\x04\x12\x04\xb4\x01\x08\x10\n\r\n\x05\x04\x03\x02\
    \x01\x05\x12\x04\xb4\x01\x11\x16\n\r\n\x05\x04\x03\x02\x01\x01\x12\x04\
    \xb4\x01\x17!\n\r\n\x05\x04\x03\x02\x01\x03\x12\x04\xb4\x01$%\nO\n\x04\
    \x04\x03\x02\x02\x12\x04\xb5\x01\x08\x1e\"A\x20\"m\"\x20from\x20n,\x20ho\
    w\x20many\x20valid\x20signatures\x20is\x20necessary\x20for\x20spending\n\
    \n\r\n\x05\x04\x03\x02\x02\x04\x12\x04\xb5\x01\x08\x10\n\r\n\x05\x04\x03\
    \x02\x02\x05\x12\x04\xb5\x01\x11\x17\n\r\n\x05\x04\x03\x02\x02\x01\x12\
    \x04\xb5\x01\x18\x19\n\r\n\x05\x04\x03\x02\x02\x03\x12\x04\xb5\x01\x1c\
    \x1d\nk\n\x02\x04\x04\x12\x06\xbd\x01\0\xc6\x01\x01\x1a]*\n\x20Structure\
    \x20representing\x20transaction\x20input\n\x20@used_in\x20SimpleSignTx\n\
    \x20@used_in\x20TransactionType\n\n\x0b\n\x03\x04\x04\x01\x12\x04\xbd\
    \x01\x08\x13\n>\n\x04\x04\x04\x02\0\x12\x04\xbe\x01\x08&\"0\x20BIP-32\
    \x20path\x20to\x20derive\x20the\x20key\x20from\x20master\x20node\n\n\r\n\
    \x05\x04\x04\x02\0\x04\x12\x04\xbe\x01\x08\x10\n\r\n\x05\x04\x04\x02\0\
    \x05\x12\x04\xbe\x01\x11\x17\n\r\n\x05\x04\x04\x02\0\x01\x12\x04\xbe\x01\
    \x18!\n\r\n\x05\x04\x04\x02\0\x03\x12\x04\xbe\x01$%\nJ\n\x04\x04\x04\x02\
    \x01\x12\x04\xbf\x01\x08%\"<\x20hash\x20of\x20previous\x20transaction\
    \x20output\x20to\x20spend\x20by\x20this\x20input\n\n\r\n\x05\x04\x04\x02\
    \x01\x04\x12\x04\xbf\x01\x08\x10\n\r\n\x05\x04\x04\x02\x01\x05\x12\x04\
    \xbf\x01\x11\x16\n\r\n\x05\x04\x04\x02\x01\x01\x12\x04\xbf\x01\x17\x20\n\
    \r\n\x05\x04\x04\x02\x01\x03\x12\x04\xbf\x01#$\n1\n\x04\x04\x04\x02\x02\
    \x12\x04\xc0\x01\x08'\"#\x20index\x20of\x20previous\x20output\x20to\x20s\
    pend\n\n\r\n\x05\x04\x04\x02\x02\x04\x12\x04\xc0\x01\x08\x10\n\r\n\x05\
    \x04\x04\x02\x02\x05\x12\x04\xc0\x01\x11\x17\n\r\n\x05\x04\x04\x02\x02\
    \x01\x12\x04\xc0\x01\x18\"\n\r\n\x05\x04\x04\x02\x02\x03\x12\x04\xc0\x01\
    %&\n6\n\x04\x04\x04\x02\x03\x12\x04\xc1\x01\x08&\"(\x20script\x20signatu\
    re,\x20unset\x20for\x20tx\x20to\x20sign\n\n\r\n\x05\x04\x04\x02\x03\x04\
    \x12\x04\xc1\x01\x08\x10\n\r\n\x05\x04\x04\x02\x03\x05\x12\x04\xc1\x01\
    \x11\x16\n\r\n\x05\x04\x04\x02\x03\x01\x12\x04\xc1\x01\x17!\n\r\n\x05\
    \x04\x04\x02\x03\x03\x12\x04\xc1\x01$%\n-\n\x04\x04\x04\x02\x04\x12\x04\
    \xc2\x01\x08:\"\x1f\x20sequence\x20(default=0xffffffff)\n\n\r\n\x05\x04\
    \x04\x02\x04\x04\x12\x04\xc2\x01\x08\x10\n\r\n\x05\x04\x04\x02\x04\x05\
    \x12\x04\xc2\x01\x11\x17\n\r\n\x05\x04\x04\x02\x04\x01\x12\x04\xc2\x01\
    \x18\x20\n\r\n\x05\x04\x04\x02\x04\x03\x12\x04\xc2\x01#$\n\r\n\x05\x04\
    \x04\x02\x04\x08\x12\x04\xc2\x01%9\n\r\n\x05\x04\x04\x02\x04\x07\x12\x04\
    \xc2\x01.8\n0\n\x04\x04\x04\x02\x05\x12\x04\xc3\x01\x08H\"\"\x20defines\
    \x20template\x20of\x20input\x20script\n\n\r\n\x05\x04\x04\x02\x05\x04\
    \x12\x04\xc3\x01\x08\x10\n\r\n\x05\x04\x04\x02\x05\x06\x12\x04\xc3\x01\
    \x11\x20\n\r\n\x05\x04\x04\x02\x05\x01\x12\x04\xc3\x01!,\n\r\n\x05\x04\
    \x04\x02\x05\x03\x12\x04\xc3\x01/0\n\r\n\x05\x04\x04\x02\x05\x08\x12\x04\
    \xc3\x011G\n\r\n\x05\x04\x04\x02\x05\x07\x12\x04\xc3\x01:F\n=\n\x04\x04\
    \x04\x02\x06\x12\x04\xc4\x01\x087\"/\x20Filled\x20if\x20input\x20is\x20g\
    oing\x20to\x20spend\x20multisig\x20tx\n\n\r\n\x05\x04\x04\x02\x06\x04\
    \x12\x04\xc4\x01\x08\x10\n\r\n\x05\x04\x04\x02\x06\x06\x12\x04\xc4\x01\
    \x11)\n\r\n\x05\x04\x04\x02\x06\x01\x12\x04\xc4\x01*2\n\r\n\x05\x04\x04\
    \x02\x06\x03\x12\x04\xc4\x0156\nG\n\x04\x04\x04\x02\x07\x12\x04\xc5\x01\
    \x08#\"9\x20amount\x20of\x20previous\x20transaction\x20output\x20(for\
    \x20segwit\x20only)\n\n\r\n\x05\x04\x04\x02\x07\x04\x12\x04\xc5\x01\x08\
    \x10\n\r\n\x05\x04\x04\x02\x07\x05\x12\x04\xc5\x01\x11\x17\n\r\n\x05\x04\
    \x04\x02\x07\x01\x12\x04\xc5\x01\x18\x1e\n\r\n\x05\x04\x04\x02\x07\x03\
    \x12\x04\xc5\x01!\"\nl\n\x02\x04\x05\x12\x06\xcd\x01\0\xd4\x01\x01\x1a^*\
    \n\x20Structure\x20representing\x20transaction\x20output\n\x20@used_in\
    \x20SimpleSignTx\n\x20@used_in\x20TransactionType\n\n\x0b\n\x03\x04\x05\
    \x01\x12\x04\xcd\x01\x08\x14\n6\n\x04\x04\x05\x02\0\x12\x04\xce\x01\x08$\
    \"(\x20target\x20coin\x20address\x20in\x20Base58\x20encoding\n\n\r\n\x05\
    \x04\x05\x02\0\x04\x12\x04\xce\x01\x08\x10\n\r\n\x05\x04\x05\x02\0\x05\
    \x12\x04\xce\x01\x11\x17\n\r\n\x05\x04\x05\x02\0\x01\x12\x04\xce\x01\x18\
    \x1f\n\r\n\x05\x04\x05\x02\0\x03\x12\x04\xce\x01\"#\nb\n\x04\x04\x05\x02\
    \x01\x12\x04\xcf\x01\x08&\"T\x20BIP-32\x20path\x20to\x20derive\x20the\
    \x20key\x20from\x20master\x20node;\x20has\x20higher\x20priority\x20than\
    \x20\"address\"\n\n\r\n\x05\x04\x05\x02\x01\x04\x12\x04\xcf\x01\x08\x10\
    \n\r\n\x05\x04\x05\x02\x01\x05\x12\x04\xcf\x01\x11\x17\n\r\n\x05\x04\x05\
    \x02\x01\x01\x12\x04\xcf\x01\x18!\n\r\n\x05\x04\x05\x02\x01\x03\x12\x04\
    \xcf\x01$%\n+\n\x04\x04\x05\x02\x02\x12\x04\xd0\x01\x08#\"\x1d\x20amount\
    \x20to\x20spend\x20in\x20satoshis\n\n\r\n\x05\x04\x05\x02\x02\x04\x12\
    \x04\xd0\x01\x08\x10\n\r\n\x05\x04\x05\x02\x02\x05\x12\x04\xd0\x01\x11\
    \x17\n\r\n\x05\x04\x05\x02\x02\x01\x12\x04\xd0\x01\x18\x1e\n\r\n\x05\x04\
    \x05\x02\x02\x03\x12\x04\xd0\x01!\"\n\"\n\x04\x04\x05\x02\x03\x12\x04\
    \xd1\x01\x082\"\x14\x20output\x20script\x20type\n\n\r\n\x05\x04\x05\x02\
    \x03\x04\x12\x04\xd1\x01\x08\x10\n\r\n\x05\x04\x05\x02\x03\x06\x12\x04\
    \xd1\x01\x11!\n\r\n\x05\x04\x05\x02\x03\x01\x12\x04\xd1\x01\"-\n\r\n\x05\
    \x04\x05\x02\x03\x03\x12\x04\xd1\x0101\nK\n\x04\x04\x05\x02\x04\x12\x04\
    \xd2\x01\x087\"=\x20defines\x20multisig\x20address;\x20script_type\x20mu\
    st\x20be\x20PAYTOMULTISIG\n\n\r\n\x05\x04\x05\x02\x04\x04\x12\x04\xd2\
    \x01\x08\x10\n\r\n\x05\x04\x05\x02\x04\x06\x12\x04\xd2\x01\x11)\n\r\n\
    \x05\x04\x05\x02\x04\x01\x12\x04\xd2\x01*2\n\r\n\x05\x04\x05\x02\x04\x03\
    \x12\x04\xd2\x0156\n[\n\x04\x04\x05\x02\x05\x12\x04\xd3\x01\x08*\"M\x20d\
    efines\x20op_return\x20data;\x20script_type\x20must\x20be\x20PAYTOOPRETU\
    RN,\x20amount\x20must\x20be\x200\n\n\r\n\x05\x04\x05\x02\x05\x04\x12\x04\
    \xd3\x01\x08\x10\n\r\n\x05\x04\x05\x02\x05\x05\x12\x04\xd3\x01\x11\x16\n\
    \r\n\x05\x04\x05\x02\x05\x01\x12\x04\xd3\x01\x17%\n\r\n\x05\x04\x05\x02\
    \x05\x03\x12\x04\xd3\x01()\n^\n\x02\x04\x06\x12\x06\xda\x01\0\xdd\x01\
    \x01\x1aP*\n\x20Structure\x20representing\x20compiled\x20transaction\x20\
    output\n\x20@used_in\x20TransactionType\n\n\x0b\n\x03\x04\x06\x01\x12\
    \x04\xda\x01\x08\x17\n\x0c\n\x04\x04\x06\x02\0\x12\x04\xdb\x01\x08#\n\r\
    \n\x05\x04\x06\x02\0\x04\x12\x04\xdb\x01\x08\x10\n\r\n\x05\x04\x06\x02\0\
    \x05\x12\x04\xdb\x01\x11\x17\n\r\n\x05\x04\x06\x02\0\x01\x12\x04\xdb\x01\
    \x18\x1e\n\r\n\x05\x04\x06\x02\0\x03\x12\x04\xdb\x01!\"\n\x0c\n\x04\x04\
    \x06\x02\x01\x12\x04\xdc\x01\x08)\n\r\n\x05\x04\x06\x02\x01\x04\x12\x04\
    \xdc\x01\x08\x10\n\r\n\x05\x04\x06\x02\x01\x05\x12\x04\xdc\x01\x11\x16\n\
    \r\n\x05\x04\x06\x02\x01\x01\x12\x04\xdc\x01\x17$\n\r\n\x05\x04\x06\x02\
    \x01\x03\x12\x04\xdc\x01'(\nK\n\x02\x04\x07\x12\x06\xe3\x01\0\xed\x01\
    \x01\x1a=*\n\x20Structure\x20representing\x20transaction\n\x20@used_in\
    \x20SimpleSignTx\n\n\x0b\n\x03\x04\x07\x01\x12\x04\xe3\x01\x08\x17\n\x0c\
    \n\x04\x04\x07\x02\0\x12\x04\xe4\x01\x08$\n\r\n\x05\x04\x07\x02\0\x04\
    \x12\x04\xe4\x01\x08\x10\n\r\n\x05\x04\x07\x02\0\x05\x12\x04\xe4\x01\x11\
    \x17\n\r\n\x05\x04\x07\x02\0\x01\x12\x04\xe4\x01\x18\x1f\n\r\n\x05\x04\
    \x07\x02\0\x03\x12\x04\xe4\x01\"#\n\x0c\n\x04\x04\x07\x02\x01\x12\x04\
    \xe5\x01\x08(\n\r\n\x05\x04\x07\x02\x01\x04\x12\x04\xe5\x01\x08\x10\n\r\
    \n\x05\x04\x07\x02\x01\x06\x12\x04\xe5\x01\x11\x1c\n\r\n\x05\x04\x07\x02\
    \x01\x01\x12\x04\xe5\x01\x1d#\n\r\n\x05\x04\x07\x02\x01\x03\x12\x04\xe5\
    \x01&'\n\x0c\n\x04\x04\x07\x02\x02\x12\x04\xe6\x01\x081\n\r\n\x05\x04\
    \x07\x02\x02\x04\x12\x04\xe6\x01\x08\x10\n\r\n\x05\x04\x07\x02\x02\x06\
    \x12\x04\xe6\x01\x11\x20\n\r\n\x05\x04\x07\x02\x02\x01\x12\x04\xe6\x01!,\
    \n\r\n\x05\x04\x07\x02\x02\x03\x12\x04\xe6\x01/0\n\x0c\n\x04\x04\x07\x02\
    \x03\x12\x04\xe7\x01\x08*\n\r\n\x05\x04\x07\x02\x03\x04\x12\x04\xe7\x01\
    \x08\x10\n\r\n\x05\x04\x07\x02\x03\x06\x12\x04\xe7\x01\x11\x1d\n\r\n\x05\
    \x04\x07\x02\x03\x01\x12\x04\xe7\x01\x1e%\n\r\n\x05\x04\x07\x02\x03\x03\
    \x12\x04\xe7\x01()\n\x0c\n\x04\x04\x07\x02\x04\x12\x04\xe8\x01\x08&\n\r\
    \n\x05\x04\x07\x02\x04\x04\x12\x04\xe8\x01\x08\x10\n\r\n\x05\x04\x07\x02\
    \x04\x05\x12\x04\xe8\x01\x11\x17\n\r\n\x05\x04\x07\x02\x04\x01\x12\x04\
    \xe8\x01\x18!\n\r\n\x05\x04\x07\x02\x04\x03\x12\x04\xe8\x01$%\n\x0c\n\
    \x04\x04\x07\x02\x05\x12\x04\xe9\x01\x08'\n\r\n\x05\x04\x07\x02\x05\x04\
    \x12\x04\xe9\x01\x08\x10\n\r\n\x05\x04\x07\x02\x05\x05\x12\x04\xe9\x01\
    \x11\x17\n\r\n\x05\x04\x07\x02\x05\x01\x12\x04\xe9\x01\x18\"\n\r\n\x05\
    \x04\x07\x02\x05\x03\x12\x04\xe9\x01%&\n\x0c\n\x04\x04\x07\x02\x06\x12\
    \x04\xea\x01\x08(\n\r\n\x05\x04\x07\x02\x06\x04\x12\x04\xea\x01\x08\x10\
    \n\r\n\x05\x04\x07\x02\x06\x05\x12\x04\xea\x01\x11\x17\n\r\n\x05\x04\x07\
    \x02\x06\x01\x12\x04\xea\x01\x18#\n\r\n\x05\x04\x07\x02\x06\x03\x12\x04\
    \xea\x01&'\n\x0c\n\x04\x04\x07\x02\x07\x12\x04\xeb\x01\x08&\n\r\n\x05\
    \x04\x07\x02\x07\x04\x12\x04\xeb\x01\x08\x10\n\r\n\x05\x04\x07\x02\x07\
    \x05\x12\x04\xeb\x01\x11\x16\n\r\n\x05\x04\x07\x02\x07\x01\x12\x04\xeb\
    \x01\x17!\n\r\n\x05\x04\x07\x02\x07\x03\x12\x04\xeb\x01$%\n\x0c\n\x04\
    \x04\x07\x02\x08\x12\x04\xec\x01\x08+\n\r\n\x05\x04\x07\x02\x08\x04\x12\
    \x04\xec\x01\x08\x10\n\r\n\x05\x04\x07\x02\x08\x05\x12\x04\xec\x01\x11\
    \x17\n\r\n\x05\x04\x07\x02\x08\x01\x12\x04\xec\x01\x18&\n\r\n\x05\x04\
    \x07\x02\x08\x03\x12\x04\xec\x01)*\nL\n\x02\x04\x08\x12\x06\xf3\x01\0\
    \xf8\x01\x01\x1a>*\n\x20Structure\x20representing\x20request\x20details\
    \n\x20@used_in\x20TxRequest\n\n\x0b\n\x03\x04\x08\x01\x12\x04\xf3\x01\
    \x08\x1c\n>\n\x04\x04\x08\x02\0\x12\x04\xf4\x01\x08*\"0\x20device\x20exp\
    ects\x20TxAck\x20message\x20from\x20the\x20computer\n\n\r\n\x05\x04\x08\
    \x02\0\x04\x12\x04\xf4\x01\x08\x10\n\r\n\x05\x04\x08\x02\0\x05\x12\x04\
    \xf4\x01\x11\x17\n\r\n\x05\x04\x08\x02\0\x01\x12\x04\xf4\x01\x18%\n\r\n\
    \x05\x04\x08\x02\0\x03\x12\x04\xf4\x01()\n0\n\x04\x04\x08\x02\x01\x12\
    \x04\xf5\x01\x08#\"\"\x20tx_hash\x20of\x20requested\x20transaction\n\n\r\
    \n\x05\x04\x08\x02\x01\x04\x12\x04\xf5\x01\x08\x10\n\r\n\x05\x04\x08\x02\
    \x01\x05\x12\x04\xf5\x01\x11\x16\n\r\n\x05\x04\x08\x02\x01\x01\x12\x04\
    \xf5\x01\x17\x1e\n\r\n\x05\x04\x08\x02\x01\x03\x12\x04\xf5\x01!\"\n.\n\
    \x04\x04\x08\x02\x02\x12\x04\xf6\x01\x08+\"\x20\x20length\x20of\x20reque\
    sted\x20extra\x20data\n\n\r\n\x05\x04\x08\x02\x02\x04\x12\x04\xf6\x01\
    \x08\x10\n\r\n\x05\x04\x08\x02\x02\x05\x12\x04\xf6\x01\x11\x17\n\r\n\x05\
    \x04\x08\x02\x02\x01\x12\x04\xf6\x01\x18&\n\r\n\x05\x04\x08\x02\x02\x03\
    \x12\x04\xf6\x01)*\n.\n\x04\x04\x08\x02\x03\x12\x04\xf7\x01\x08.\"\x20\
    \x20offset\x20of\x20requested\x20extra\x20data\n\n\r\n\x05\x04\x08\x02\
    \x03\x04\x12\x04\xf7\x01\x08\x10\n\r\n\x05\x04\x08\x02\x03\x05\x12\x04\
    \xf7\x01\x11\x17\n\r\n\x05\x04\x08\x02\x03\x01\x12\x04\xf7\x01\x18)\n\r\
    \n\x05\x04\x08\x02\x03\x03\x12\x04\xf7\x01,-\nL\n\x02\x04\t\x12\x06\xfe\
    \x01\0\x82\x02\x01\x1a>*\n\x20Structure\x20representing\x20serialized\
    \x20data\n\x20@used_in\x20TxRequest\n\n\x0b\n\x03\x04\t\x01\x12\x04\xfe\
    \x01\x08\x1f\nE\n\x04\x04\t\x02\0\x12\x04\xff\x01\x08,\"7\x20'signature'\
    \x20field\x20contains\x20signed\x20input\x20of\x20this\x20index\n\n\r\n\
    \x05\x04\t\x02\0\x04\x12\x04\xff\x01\x08\x10\n\r\n\x05\x04\t\x02\0\x05\
    \x12\x04\xff\x01\x11\x17\n\r\n\x05\x04\t\x02\0\x01\x12\x04\xff\x01\x18'\
    \n\r\n\x05\x04\t\x02\0\x03\x12\x04\xff\x01*+\n6\n\x04\x04\t\x02\x01\x12\
    \x04\x80\x02\x08%\"(\x20signature\x20of\x20the\x20signature_index\x20inp\
    ut\n\n\r\n\x05\x04\t\x02\x01\x04\x12\x04\x80\x02\x08\x10\n\r\n\x05\x04\t\
    \x02\x01\x05\x12\x04\x80\x02\x11\x16\n\r\n\x05\x04\t\x02\x01\x01\x12\x04\
    \x80\x02\x17\x20\n\r\n\x05\x04\t\x02\x01\x03\x12\x04\x80\x02#$\n9\n\x04\
    \x04\t\x02\x02\x12\x04\x81\x02\x08)\"+\x20part\x20of\x20serialized\x20an\
    d\x20signed\x20transaction\n\n\r\n\x05\x04\t\x02\x02\x04\x12\x04\x81\x02\
    \x08\x10\n\r\n\x05\x04\t\x02\x02\x05\x12\x04\x81\x02\x11\x16\n\r\n\x05\
    \x04\t\x02\x02\x01\x12\x04\x81\x02\x17$\n\r\n\x05\x04\t\x02\x02\x03\x12\
    \x04\x81\x02'(\nM\n\x02\x04\n\x12\x06\x88\x02\0\x8f\x02\x01\x1a?*\n\x20S\
    tructure\x20representing\x20identity\x20data\n\x20@used_in\x20IdentityTy\
    pe\n\n\x0b\n\x03\x04\n\x01\x12\x04\x88\x02\x08\x14\n!\n\x04\x04\n\x02\0\
    \x12\x04\x89\x02\x08\"\"\x13\x20proto\x20part\x20of\x20URI\n\n\r\n\x05\
    \x04\n\x02\0\x04\x12\x04\x89\x02\x08\x10\n\r\n\x05\x04\n\x02\0\x05\x12\
    \x04\x89\x02\x11\x17\n\r\n\x05\x04\n\x02\0\x01\x12\x04\x89\x02\x18\x1d\n\
    \r\n\x05\x04\n\x02\0\x03\x12\x04\x89\x02\x20!\n\x20\n\x04\x04\n\x02\x01\
    \x12\x04\x8a\x02\x08!\"\x12\x20user\x20part\x20of\x20URI\n\n\r\n\x05\x04\
    \n\x02\x01\x04\x12\x04\x8a\x02\x08\x10\n\r\n\x05\x04\n\x02\x01\x05\x12\
    \x04\x8a\x02\x11\x17\n\r\n\x05\x04\n\x02\x01\x01\x12\x04\x8a\x02\x18\x1c\
    \n\r\n\x05\x04\n\x02\x01\x03\x12\x04\x8a\x02\x1f\x20\n\x20\n\x04\x04\n\
    \x02\x02\x12\x04\x8b\x02\x08!\"\x12\x20host\x20part\x20of\x20URI\n\n\r\n\
    \x05\x04\n\x02\x02\x04\x12\x04\x8b\x02\x08\x10\n\r\n\x05\x04\n\x02\x02\
    \x05\x12\x04\x8b\x02\x11\x17\n\r\n\x05\x04\n\x02\x02\x01\x12\x04\x8b\x02\
    \x18\x1c\n\r\n\x05\x04\n\x02\x02\x03\x12\x04\x8b\x02\x1f\x20\n\x20\n\x04\
    \x04\n\x02\x03\x12\x04\x8c\x02\x08!\"\x12\x20port\x20part\x20of\x20URI\n\
    \n\r\n\x05\x04\n\x02\x03\x04\x12\x04\x8c\x02\x08\x10\n\r\n\x05\x04\n\x02\
    \x03\x05\x12\x04\x8c\x02\x11\x17\n\r\n\x05\x04\n\x02\x03\x01\x12\x04\x8c\
    \x02\x18\x1c\n\r\n\x05\x04\n\x02\x03\x03\x12\x04\x8c\x02\x1f\x20\n\x20\n\
    \x04\x04\n\x02\x04\x12\x04\x8d\x02\x08!\"\x12\x20path\x20part\x20of\x20U\
    RI\n\n\r\n\x05\x04\n\x02\x04\x04\x12\x04\x8d\x02\x08\x10\n\r\n\x05\x04\n\
    \x02\x04\x05\x12\x04\x8d\x02\x11\x17\n\r\n\x05\x04\n\x02\x04\x01\x12\x04\
    \x8d\x02\x18\x1c\n\r\n\x05\x04\n\x02\x04\x03\x12\x04\x8d\x02\x1f\x20\n\
    \x1e\n\x04\x04\n\x02\x05\x12\x04\x8e\x02\x08.\"\x10\x20identity\x20index\
    \n\n\r\n\x05\x04\n\x02\x05\x04\x12\x04\x8e\x02\x08\x10\n\r\n\x05\x04\n\
    \x02\x05\x05\x12\x04\x8e\x02\x11\x17\n\r\n\x05\x04\n\x02\x05\x01\x12\x04\
    \x8e\x02\x18\x1d\n\r\n\x05\x04\n\x02\x05\x03\x12\x04\x8e\x02\x20!\n\r\n\
    \x05\x04\n\x02\x05\x08\x12\x04\x8e\x02\"-\n\r\n\x05\x04\n\x02\x05\x07\
    \x12\x04\x8e\x02+,\
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
