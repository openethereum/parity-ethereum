//! Bare rust wrapper around evmjit
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

extern crate libc;
use self::ffi::*;

pub use self::ffi::JitReturnCode as ReturnCode;

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

/// Safe handle for jit context.
pub struct ContextHandle {
	context: *mut JitContext,
	_data_handle: RuntimeDataHandle
}

impl ContextHandle {
	/// Creates new context handle.
	pub fn new(mut data_handle: RuntimeDataHandle, mut env: EnvHandle) -> Self {
		let context = unsafe { evmjit_create_context(data_handle.mut_runtime_data(), &mut env) };
		ContextHandle {
			context: context,
			_data_handle: data_handle
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

impl Env for EnvHandle {
	fn sload(&self, index: *const JitI256, out_value: *mut JitI256) { 
		match self.env_impl {
			Some(ref env) => env.sload(index, out_value),
			None => { panic!(); }
		}
	}

	fn sstore(&mut self, index: *const JitI256, value: *const JitI256) { 
		match self.env_impl {
			Some(ref mut env) => env.sstore(index, value),
			None => { panic!(); }
		}
	}

}

/// ffi functions
pub mod ffi {
	use super::*;
	use libc;

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
	/// Signed 256 bit integer.
	pub struct JitI256 {
		pub words: [u64; 4]
	}

	#[repr(C)]
	/// Jit runtime data.
	pub struct JitRuntimeData {
		pub gas: i64,
		pub gas_price: i64,
		pub call_data: *const libc::c_char,
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
		pub code: *const libc::c_char,
		pub code_size: u64,
		pub code_hash: JitI256
	}

	#[no_mangle]
	pub unsafe extern fn env_sload(env: *mut EnvHandle, index: *const JitI256, out_value: *mut JitI256) {
		let env = &*env;
		env.sload(index, out_value);
	}

	#[no_mangle]
	pub unsafe extern fn env_sstore(env: *mut EnvHandle, index: *const JitI256, value: *const JitI256) {
		let env = &mut *env;
		env.sstore(index, value);
	}

	#[no_mangle]
	pub extern fn env_balance(_env: *mut EnvHandle, _address: *const JitI256, _value: *const JitI256) {
		unimplemented!()
	}

	#[no_mangle]
	pub extern fn env_blockhash(_env: *mut EnvHandle, _number: *const JitI256, _hash: *const JitI256) {
		unimplemented!()
	}

	#[no_mangle]
	pub extern fn env_create(_env: *mut EnvHandle, 
							 _io_gas: *const u64, 
							 _endowment: *const JitI256, 
							 _init_beg: *const u8, 
							 _init_size: *const u64, 
							 _address: *const JitI256) {
		unimplemented!()
	}

	#[no_mangle]
	pub extern fn env_call(_env: *mut EnvHandle, 
						   _io_gas: *const u64,
						   _call_gas: *const u64,
						   _receive_address: *const JitI256,
						   _value: *const JitI256,
						   _in_beg: *const u8,
						   _in_size: *const u64,
						   _out_beg: *const u8,
						   _out_size: *const u64,
						   _code_address: JitI256) {
		unimplemented!()
	}

	#[no_mangle]
	pub extern fn env_sha3(_begin: *const u8, _size: *const u64, _hash: *const JitI256) {
		unimplemented!()
	}

	#[no_mangle]
	pub extern fn env_extcode(_env: *mut EnvHandle, _address: *const JitI256, _size: *const u64) {
		unimplemented!()
	}

	#[no_mangle]
	pub extern fn env_log(_env: *mut EnvHandle,
						  _beg: *const u8,
						  _size: *const u64,
						  _topic1: *const JitI256,
						  _topic2: *const JitI256,
						  _topic3: *const JitI256,
						  _topic4: *const JitI256) {
		unimplemented!()
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
