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

//! Bare rust wrapper around evmjit.
//! 
//! Requires latest version of Ethereum EVM JIT. https://github.com/debris/evmjit
//! 
//! ```
//! extern crate evmjit;
//! use evmjit::*;
//! 
//! fn main() {
//! 	let mut context = ContextHandle::new(RuntimeDataHandle::new(), ExtHandle::empty());
//! 	assert_eq!(context.exec(), ReturnCode::Stop);
//! }
//! ```
//! 
//!
//! To verify that c abi is "imported" correctly, run:
//! 
//! ```bash
//!	nm your_executable -g | grep ext 
//! ```
//! 
//! It should give the following output:
//!
//! ```bash
//! 00000001000779e0 T _ext_balance
//! 0000000100077a10 T _ext_blockhash
//! 0000000100077a90 T _ext_call
//! 0000000100077a40 T _ext_create
//! 0000000100077b50 T _ext_extcode
//! 0000000100077b80 T _ext_log
//! 0000000100077b20 T _ext_sha3
//! 0000000100077980 T _ext_sload
//! 00000001000779b0 T _ext_sstore
//! ```

extern crate tiny_keccak;

use std::ops::{Deref, DerefMut};
use self::ffi::*;

pub use self::ffi::JitReturnCode as ReturnCode;
pub use self::ffi::JitI256 as I256;
pub use self::ffi::JitH256 as H256;

/// Takes care of proper initialization and destruction of `RuntimeData`.
///
/// This handle must be used to create runtime data,
/// cause underneath it's a `C++` structure. Incompatible with rust
/// structs.
pub struct RuntimeDataHandle {
	runtime_data: *mut JitRuntimeData
}

impl RuntimeDataHandle {
	/// Creates new `RuntimeData` handle.
	pub fn new() -> Self {
		RuntimeDataHandle {
			runtime_data: unsafe { evmjit_create_runtime_data() }
		}
	}
}

impl Drop for RuntimeDataHandle {
	fn drop(&mut self) {
		unsafe { evmjit_destroy_runtime_data(self.runtime_data) }
	}
}

impl Deref for RuntimeDataHandle {
	type Target = JitRuntimeData;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.runtime_data }
	}
}

impl DerefMut for RuntimeDataHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.runtime_data }
	}
}

/// Takes care of proper initilization and destruction of `JitSchedule`.
/// 
/// This handle must be used to jit schedule,
/// cause underneath it's a `C++` structure. Incompatible with rust
/// structs.
pub struct ScheduleHandle {
	schedule: *mut JitSchedule
}

impl ScheduleHandle {
	/// Creates new `Schedule` handle.
	pub fn new() -> Self {
		ScheduleHandle {
			schedule: unsafe { evmjit_create_schedule() }
		}
	}
}

impl Drop for ScheduleHandle {
	fn drop(&mut self) {
		unsafe { evmjit_destroy_schedule(self.schedule) }
	}
}

impl Deref for ScheduleHandle {
	type Target = JitSchedule;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.schedule }
	}
}

impl DerefMut for ScheduleHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.schedule }
	}
}

/// Takes care of  proper initialization and destruction of jit context.
///
/// This handle must be used to create context,
/// cause underneath it's a `C++` structure. Incombatible with rust
/// structs.
pub struct ContextHandle {
	context: *mut JitContext,
	data_handle: RuntimeDataHandle,
	schedule_handle: ScheduleHandle
}

impl ContextHandle {
	/// Creates new context handle.
	/// 
	/// This function is unsafe cause ext lifetime is not considered
	/// We also can't make ExtHandle a member of `ContextHandle` structure,
	/// cause this would be a move operation or it would require a template 
	/// lifetime to a reference. Both solutions are not possible.
	pub unsafe fn new(data_handle: RuntimeDataHandle, schedule_handle: ScheduleHandle, ext: &mut ExtHandle) -> Self {
		let mut handle = ContextHandle {
			context: std::mem::uninitialized(),
			schedule_handle: schedule_handle,
			data_handle: data_handle,
		};

		handle.context = evmjit_create_context(handle.data_handle.deref_mut(), ext);
		handle
	}

	/// Executes context.
	pub fn exec(&mut self) -> JitReturnCode {
		unsafe { evmjit_exec(self.context, self.schedule_handle.deref_mut()) }
	}

	/// Returns output data.
	pub fn output_data(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.data_handle.call_data, self.data_handle.call_data_size as usize) }
	}

	/// Returns address to which funds should be transfered after suicide.
	pub fn suicide_refund_address(&self) -> JitI256 {
		// evmjit reuses data_handle address field to store suicide address
		self.data_handle.address
	}

	/// Returns gas left.
	pub fn gas_left(&self) -> u64 {
		self.data_handle.gas as u64
	}
}

impl Drop for ContextHandle {
	fn drop(&mut self) {
		unsafe { evmjit_destroy_context(self.context); }
	}
}

/// Component oriented wrapper around jit ext c interface.
pub trait Ext {
	fn sload(&self, index: *const JitI256, out_value: *mut JitI256);
	fn sstore(&mut self, index: *const JitI256, value: *const JitI256);
	fn balance(&self, address: *const JitH256, out_value: *mut JitI256);
	fn blockhash(&self, number: *const JitI256, out_hash: *mut JitH256);

	fn create(&mut self,
			  io_gas: *mut u64,
			  endowment: *const JitI256,
			  init_beg: *const u8,
			  init_size: u64,
			  address: *mut JitH256);

	fn call(&mut self,
				io_gas: *mut u64,
				call_gas: u64,
				sender_address: *const JitH256,
				receive_address: *const JitH256,
				code_address: *const JitH256,
				transfer_value: *const JitI256,
				apparent_value: *const JitI256,
				in_beg: *const u8,
				in_size: u64,
				out_beg: *mut u8,
				out_size: u64) -> bool;

	fn log(&mut self,
		   beg: *const u8,
		   size: u64,
		   topic1: *const JitH256,
		   topic2: *const JitH256,
		   topic3: *const JitH256,
		   topic4: *const JitH256);

	fn extcode(&self, address: *const JitH256, size: *mut u64) -> *const u8;
}

/// C abi compatible wrapper for jit ext implementers.
pub struct ExtHandle {
	ext_impl: Option<Box<Ext>>
}

impl ExtHandle {
	/// Creates new extironment wrapper for given implementation
	pub fn new<T>(ext_impl: T) -> Self where T: Ext + 'static {
		ExtHandle { ext_impl: Some(Box::new(ext_impl)) }
	}

	/// Creates empty extironment.
	/// It can be used to for any operations.
	pub fn empty() -> Self {
		ExtHandle { ext_impl: None }
	}
}

impl Deref for ExtHandle {
	type Target = Box<Ext>;

	fn deref(&self) -> &Self::Target {
		match self.ext_impl {
			Some(ref ext) => ext,
			None => { panic!("Handle is empty!"); }
		}
	}
}

impl DerefMut for ExtHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self.ext_impl {
			Some(ref mut ext) => ext,
			None => { panic!("Handle is empty!"); }
		}
	}
}

/// ffi functions
pub mod ffi {
	use std::slice;
	use std::mem;
	use tiny_keccak::Keccak;
	use super::*;

	/// Jit context struct declaration.
	pub enum JitContext {}

	#[repr(C)]
	#[derive(Debug, Eq, PartialEq)]
	/// Jit context execution return code.
	pub enum JitReturnCode {
		Stop = 0,
		Return = 1,
		Suicide = 2,

		OutOfGas = -1,

		LLVMError = -101,
		UnexpectedError = -111
	}

	#[repr(C)]
	#[derive(Debug, Copy, Clone)]
	/// Signed 256 bit integer.
	pub struct JitI256 {
		pub words: [u64; 4]
	}

	#[repr(C)]
	#[derive(Debug, Copy, Clone)]
	/// Jit Hash
	pub struct JitH256 {
		pub words: [u64; 4]
	}

	impl From<JitH256> for JitI256 {
		fn from(mut hash: JitH256) -> JitI256 {
			unsafe {
				{
					let bytes: &mut [u8] = slice::from_raw_parts_mut(hash.words.as_mut_ptr() as *mut u8, 32);
					bytes.reverse();
				}
				mem::transmute(hash)
			}
		}
	}

	impl From<JitI256> for JitH256 {
		fn from(mut i: JitI256) -> JitH256 {
			unsafe {
				{
					let bytes: &mut [u8] = slice::from_raw_parts_mut(i.words.as_mut_ptr() as *mut u8, 32);
					bytes.reverse();
				}
				mem::transmute(i)
			}
		}
	}

	#[repr(C)]
	#[derive(Debug)]
	/// Jit runtime data.
	pub struct JitRuntimeData {
		pub gas: i64,
		pub gas_price: i64,
		pub call_data: *const u8,
		pub call_data_size: u64,
		pub address: JitI256,
		pub caller: JitI256,
		pub origin: JitI256,
		pub transfer_value: JitI256,
		pub apparent_value: JitI256,
		pub author: JitI256,
		pub difficulty: JitI256,
		pub gas_limit: JitI256,
		pub number: u64,
		pub timestamp: i64,
		pub code: *const u8,
		pub code_size: u64,
		pub code_hash: JitI256
	}

	#[repr(C)]
	#[derive(Debug)]
	/// Configurable properties of git schedule.
	pub struct JitSchedule {
		pub have_delegate_call: bool
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_sload(ext: *const ExtHandle, index: *const JitI256, out_value: *mut JitI256) {
		let ext = &*ext;
		ext.sload(index, out_value);
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_sstore(ext: *mut ExtHandle, index: *mut JitI256, value: *mut JitI256) {
		let ext = &mut *ext;
		ext.sstore(index, value);
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_balance(ext: *const ExtHandle, address: *const JitH256, out_value: *mut JitI256) {
		let ext = &*ext;
		ext.balance(address, out_value);
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_blockhash(ext: *const ExtHandle, number: *const JitI256, out_hash: *mut JitH256) {
		let ext = &*ext;
		ext.blockhash(number, out_hash);
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_create(ext: *mut ExtHandle, 
							 io_gas: *mut u64, 
							 endowment: *const JitI256, 
							 init_beg: *const u8, 
							 init_size: u64, 
							 address: *mut JitH256) {
		let ext = &mut *ext;
		ext.create(io_gas, endowment, init_beg, init_size, address);
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_call(ext: *mut ExtHandle, 
						   io_gas: *mut u64,
						   call_gas: u64,
						   sender_address: *const JitH256,
						   receive_address: *const JitH256,
						   code_address: *const JitH256,
						   transfer_value: *const JitI256,
						   apparent_value: *const JitI256,
						   in_beg: *const u8,
						   in_size: u64,
						   out_beg: *mut u8,
						   out_size: u64) -> bool {
		let ext = &mut *ext;
		ext.call(io_gas, call_gas, sender_address, receive_address, code_address, transfer_value, apparent_value, in_beg, in_size, out_beg, out_size)
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_sha3(begin: *const u8, size: u64, out_hash: *mut JitH256) {
		let out_hash = &mut *out_hash;
		let input = slice::from_raw_parts(begin, size as usize);
		let outlen = out_hash.words.len() * 8;
		let output = slice::from_raw_parts_mut(out_hash.words.as_mut_ptr() as *mut u8, outlen);
		let mut sha3 = Keccak::new_keccak256();
		sha3.update(input);	
		sha3.finalize(output);
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_extcode(ext: *const ExtHandle, address: *const JitH256, size: *mut u64) -> *const u8 {
		let ext = &*ext;
		ext.extcode(address, size)
	}

	#[no_mangle]
	pub unsafe extern "C" fn env_log(ext: *mut ExtHandle,
						  beg: *const u8,
						  size: u64,
						  topic1: *const JitH256,
						  topic2: *const JitH256,
						  topic3: *const JitH256,
						  topic4: *const JitH256) {
		let ext = &mut *ext;
		ext.log(beg, size, topic1, topic2, topic3, topic4);
	}


	#[link(name="evmjit")]
	extern "C" {
		pub fn evmjit_create_schedule() -> *mut JitSchedule;
		pub fn evmjit_destroy_schedule(schedule: *mut JitSchedule);
		pub fn evmjit_create_runtime_data() -> *mut JitRuntimeData;
		pub fn evmjit_destroy_runtime_data(data: *mut JitRuntimeData);
		pub fn evmjit_destroy_context(context: *mut JitContext);
		pub fn evmjit_exec(context: *mut JitContext, schedule: *mut JitSchedule) -> JitReturnCode;
	}

	// ExtHandle is not a C type, so we need to allow "improper_ctypes" 
	#[link(name="evmjit")]
	#[allow(improper_ctypes)]
	extern "C" {
		pub fn evmjit_create_context(data: *mut JitRuntimeData, ext: *mut ExtHandle) -> *mut JitContext;
	}
}

#[test]
fn ffi_test() {
	unsafe {
		let data = evmjit_create_runtime_data();
		let schedule = evmjit_create_schedule();
		let context = evmjit_create_context(data, &mut ExtHandle::empty());

		let code = evmjit_exec(context, schedule);
		assert_eq!(code, JitReturnCode::Stop);

		evmjit_destroy_schedule(schedule);
		evmjit_destroy_runtime_data(data);
		evmjit_destroy_context(context);
	}
}

#[test]
fn handle_test() {
	unsafe {
		let mut ext = ExtHandle::empty();
		let mut context = ContextHandle::new(RuntimeDataHandle::new(), ScheduleHandle::new(), &mut ext);
		assert_eq!(context.exec(), ReturnCode::Stop);
	}
}

#[test]
fn hash_to_int() {
	let h = H256 { words:[0x0123456789abcdef, 0, 0, 0] };
	let i = I256::from(h);
	assert_eq!([0u64, 0, 0, 0xefcdab8967452301], i.words);
	assert_eq!(H256::from(i).words, h.words);
}
