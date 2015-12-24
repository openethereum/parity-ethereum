//! Bare rust wrapper around evmjit.
//! 
//! Requires latest version of Ethereum EVM JIT. https://github.com/debris/evmjit
//! 
//! ```
//! extern crate evmjit;
//! use evmjit::*;
//! 
//! fn main() {
//! 	let mut context = ContextHandle::new(RuntimeDataHandle::new(), EnvHandle::empty());
//! 	assert_eq!(context.exec(), ReturnCode::Stop);
//! }
//!
//! ```

extern crate tiny_keccak;

use std::ops::{Deref, DerefMut};
use self::ffi::*;

pub use self::ffi::JitReturnCode as ReturnCode;
pub use self::ffi::JitI256 as I256;

/// Component oriented safe handle to `JitRuntimeData`.
pub struct RuntimeDataHandle {
	runtime_data: *mut JitRuntimeData
}

impl RuntimeDataHandle {
	/// Creates new handle.
	pub fn new() -> Self {
		RuntimeDataHandle {
			runtime_data: unsafe { evmjit_create_runtime_data() }
		}
	}

	/// Returns immutable reference to runtime data.
	pub fn runtime_data(&self) -> &JitRuntimeData {
		unsafe { &*self.runtime_data }
	}

	/// Returns mutable reference to runtime data.
	pub fn mut_runtime_data(&mut self) -> &mut JitRuntimeData {
		unsafe { &mut *self.runtime_data }
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
		self.runtime_data()
	}
}

impl DerefMut for RuntimeDataHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.mut_runtime_data()
	}
}

/// Safe handle for jit context.
pub struct ContextHandle {
	context: *mut JitContext,
	_data_handle: RuntimeDataHandle,
	_env: EnvHandle
}

impl ContextHandle {
	/// Creates new context handle.
	pub fn new(mut data_handle: RuntimeDataHandle, mut env: EnvHandle) -> Self {
		import_evmjit_abi();
		let context = unsafe { evmjit_create_context(data_handle.mut_runtime_data(), &mut env) };
		ContextHandle {
			context: context,
			_data_handle: data_handle,
			_env: env
		}
	}

	/// Executes context.
	pub fn exec(&mut self) -> JitReturnCode {
		unsafe { evmjit_exec(self.context) }
	}
}

impl Drop for ContextHandle {
	fn drop(&mut self) {
		unsafe { evmjit_destroy_context(self.context); }
	}
}

/// Component oriented wrapper around jit env c interface.
pub trait Env {
	fn sload(&self, index: *const JitI256, out_value: *mut JitI256);
	fn sstore(&mut self, index: *const JitI256, value: *const JitI256);
	fn balance(&self, address: *const JitI256, out_value: *mut JitI256);
	fn blockhash(&self, number: *const JitI256, out_hash: *mut JitI256);

	fn create(&mut self,
			  io_gas: *mut u64,
			  endowment: *const JitI256,
			  init_beg: *const u8,
			  init_size: *const u64,
			  address: *mut JitI256);

	fn call(&mut self,
				io_gas: *mut u64,
				call_gas: *const u64,
				receive_address: *const JitI256,
				value: *const JitI256,
				in_beg: *const u8,
				in_size: *const u64,
				out_beg: *mut u8,
				out_size: *mut u64,
				code_address: JitI256) -> bool;

	fn log(&mut self,
		   beg: *const u8,
		   size: *const u64,
		   topic1: *const JitI256,
		   topic2: *const JitI256,
		   topic3: *const JitI256,
		   topic4: *const JitI256);

	fn extcode(&self, address: *const JitI256, size: *mut u64) -> *const u8;
}

/// C abi compatible wrapper for jit env implementers.
pub struct EnvHandle {
	env_impl: Option<Box<Env>>
}

impl EnvHandle {
	/// Creates new environment wrapper for given implementation
	pub fn new<T>(env_impl: T) -> Self where T: Env + 'static {
		EnvHandle { env_impl: Some(Box::new(env_impl)) }
	}

	/// Creates empty environment.
	/// It can be used to for any operations.
	pub fn empty() -> Self {
		EnvHandle { env_impl: None }
	}
}

impl Deref for EnvHandle {
	type Target = Box<Env>;

	fn deref(&self) -> &Self::Target {
		match self.env_impl {
			Some(ref env) => env,
			None => { panic!(); }
		}
	}
}

impl DerefMut for EnvHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self.env_impl {
			Some(ref mut env) => env,
			None => { panic!(); }
		}
	}
}

/// ffi functions
pub mod ffi {
	use std::slice;
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
	#[derive(Debug)]
	/// Signed 256 bit integer.
	pub struct JitI256 {
		pub words: [u64; 4]
	}

	#[repr(C)]
	/// Jit runtime data.
	pub struct JitRuntimeData {
		pub gas: i64,
		pub gas_price: i64,
		pub call_data: *const u8,
		pub call_data_size: u64,
		pub address: JitI256,
		pub caller: JitI256,
		pub origin: JitI256,
		pub call_value: JitI256,
		pub coinbase: JitI256,
		pub difficulty: JitI256,
		pub gas_limit: JitI256,
		pub number: u64,
		pub timestamp: i64,
		pub code: *const u8,
		pub code_size: u64,
		pub code_hash: JitI256
	}

	/// Dumb function to "import" c abi in libraries 
	/// which inherit from this library.
	/// 
	/// It needs compiled as a part of main executable.
	/// 
	/// To verify that c abi is "imported" correctly, run:
	/// 
	/// ```bash
	///	nm your_executable -g | grep env 
	/// ```
	/// 
	/// It Should give the following output:
	///
	/// ```bash
	/// 0000000100078500 T _env_sha3
	/// 0000000100078470 T _env_sstore
	/// ```
	pub fn import_evmjit_abi() {
		let _env_sload = env_sload;
		let _env_sstore = env_sstore;
		let _env_balance = env_balance;
		let _env_blockhash = env_blockhash;
		let _env_create = env_create;
		let _env_call = env_call;
		let _env_sha3 = env_sha3;
		let _env_extcode = env_extcode;
		let _env_log = env_log;
	}

	#[no_mangle]
	pub unsafe extern fn env_sload(env: *const EnvHandle, index: *const JitI256, out_value: *mut JitI256) {
		let env = &*env;
		env.sload(index, out_value);
	}

	#[no_mangle]
	pub unsafe extern fn env_sstore(env: *mut EnvHandle, index: *const JitI256, value: *const JitI256) {
		let env = &mut *env;
		env.sstore(index, value);
	}

	#[no_mangle]
	pub unsafe extern fn env_balance(env: *const EnvHandle, address: *const JitI256, out_value: *mut JitI256) {
		let env = &*env;
		env.balance(address, out_value);
	}

	#[no_mangle]
	pub unsafe extern fn env_blockhash(env: *const EnvHandle, number: *const JitI256, out_hash: *mut JitI256) {
		let env = &*env;
		env.blockhash(number, out_hash);
	}

	#[no_mangle]
	pub unsafe extern fn env_create(env: *mut EnvHandle, 
							 io_gas: *mut u64, 
							 endowment: *const JitI256, 
							 init_beg: *const u8, 
							 init_size: *const u64, 
							 address: *mut JitI256) {
		let env = &mut *env;
		env.create(io_gas, endowment, init_beg, init_size, address);
	}

	#[no_mangle]
	pub unsafe extern fn env_call(env: *mut EnvHandle, 
						   io_gas: *mut u64,
						   call_gas: *const u64,
						   receive_address: *const JitI256,
						   value: *const JitI256,
						   in_beg: *const u8,
						   in_size: *const u64,
						   out_beg: *mut u8,
						   out_size: *mut u64,
						   code_address: JitI256) -> bool {
		let env = &mut *env;
		env.call(io_gas, call_gas, receive_address, value, in_beg, in_size, out_beg, out_size, code_address)
	}

	#[no_mangle]
	pub unsafe extern fn env_sha3(begin: *const u8, size: u64, out_hash: *mut JitI256) {
		// TODO: write tests
		// it may be incorrect due to endianess
		// if it is, don't use `from_raw_parts`
		let out_hash = &mut *out_hash;
		let input = slice::from_raw_parts(begin, size as usize);
		let outlen = out_hash.words.len() * 8;
		let output = slice::from_raw_parts_mut(out_hash.words.as_mut_ptr() as *mut u8, outlen);
		let mut sha3 = Keccak::new_sha3_256();
		sha3.update(input);	
		sha3.finalize(output);
	}

	#[no_mangle]
	pub unsafe extern fn env_extcode(env: *const EnvHandle, address: *const JitI256, size: *mut u64) -> *const u8 {
		let env = &*env;
		env.extcode(address, size)
	}

	#[no_mangle]
	pub unsafe extern fn env_log(env: *mut EnvHandle,
						  beg: *const u8,
						  size: *const u64,
						  topic1: *const JitI256,
						  topic2: *const JitI256,
						  topic3: *const JitI256,
						  topic4: *const JitI256) {
		let env = &mut *env;
		env.log(beg, size, topic1, topic2, topic3, topic4);
	}


	#[link(name="evmjit")]
	extern "C" {
		pub fn evmjit_create_runtime_data() -> *mut JitRuntimeData;
		pub fn evmjit_destroy_runtime_data(data: *mut JitRuntimeData);
		pub fn evmjit_destroy_context(context: *mut JitContext);
		pub fn evmjit_exec(context: *mut JitContext) -> JitReturnCode;
	}

	#[link(name="evmjit")]
	// EnvHandle does not have to by a C type
	#[allow(improper_ctypes)]
	extern "C" {
		pub fn evmjit_create_context(data: *mut JitRuntimeData, env: *mut EnvHandle) -> *mut JitContext;
	}
}

#[test]
fn ffi_test() {
	unsafe {
		let data = evmjit_create_runtime_data();
		let context = evmjit_create_context(data, &mut EnvHandle::empty());

		let code = evmjit_exec(context);
		assert_eq!(code, JitReturnCode::Stop);

		evmjit_destroy_runtime_data(data);
		evmjit_destroy_context(context);
	}
}

#[test]
fn handle_test() {
	let mut context = ContextHandle::new(RuntimeDataHandle::new(), EnvHandle::empty());
	assert_eq!(context.exec(), ReturnCode::Stop);
}
