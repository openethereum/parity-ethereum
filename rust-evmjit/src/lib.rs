//! Bare rust wrapper around evmjit
//! 
//! Requires latest version of Ethereum EVM JIT. https://github.com/debris/evmjit

extern crate libc;
use std::ptr;

#[repr(C)]
pub struct JitI256 {
	pub words: [u64; 4]
}

#[repr(C)]
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

/// Component oriented safe handle to `JitRuntimeData`.
pub struct RuntimeDataHandle {
	runtime_data: *mut JitRuntimeData
}

impl RuntimeDataHandle {
	/// Creates new handle.
	pub unsafe fn new() -> Self {
		RuntimeDataHandle {
			runtime_data: evmjit_create_runtime_data()
		}
	}

	/// Returns immutable reference to runtime data.
	pub unsafe fn runtime_data(&self) -> &JitRuntimeData {
		&*self.runtime_data
	}

	/// Returns mutable reference to runtime data.
	pub unsafe fn mut_runtime_data(&mut self) -> &mut JitRuntimeData {
		&mut *self.runtime_data
	}
}

impl Drop for RuntimeDataHandle {
	fn drop(&mut self) {
		unsafe { evmjit_destroy_runtime_data(self.runtime_data) }
	}
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub enum JitReturnCode {
	Stop = 0,
	Return = 1,
	Suicide = 2,

	OutOfGas = -1,

	LLVMError = -101,
	UnexpectedError = -111
}

/// JitContext struct declaration.
pub enum JitContext {}

pub struct ContextHandle {
	data_handle: RuntimeDataHandle,
	env: Env,
	context: *mut JitContext
}

impl ContextHandle {
	pub unsafe fn new(mut data_handle: RuntimeDataHandle, env: Env) -> Self {
		// TODO: context should take env 
		let context = evmjit_create_context(data_handle.mut_runtime_data(), &mut 0);
		ContextHandle {
			data_handle: data_handle,
			env: env,
			context: context
		}
	}
}

/// Component oriented wrapper around jit env c interface.
pub trait JitEnv {
	fn sload(&mut self);
}

/// C abi compatible wrapper for JitEnvTrait implementers.
pub struct Env {
	env_impl: Option<Box<JitEnv>>
}

impl Env {
	pub fn new<T>(env_impl: T) -> Self where T: JitEnv + 'static {
		Env { env_impl: Some(Box::new(env_impl)) }
	}
}

impl JitEnv for Env {
	fn sload(&mut self) { 
		if let Some(ref mut env) = self.env_impl {
			env.sload();
		}
	}
}

#[no_mangle]
pub unsafe extern fn env_sload(env: *mut Env, _index: *const JitI256, _value: *const JitI256) {
	let env = &mut *env;
	env.sload();
}

#[no_mangle]
pub extern fn env_sstore(_env: *mut Env, _index: *const JitI256, _value: *const JitI256) {
	unimplemented!()
}

#[no_mangle]
pub extern fn env_balance(_env: *mut Env, _address: *const JitI256, _value: *const JitI256) {
	unimplemented!()
}

#[no_mangle]
pub extern fn env_blockhash(_env: *mut Env, _number: *const JitI256, _hash: *const JitI256) {
	unimplemented!()
}

#[no_mangle]
pub extern fn env_create(_env: *mut Env, 
						 _io_gas: *const u64, 
						 _endowment: *const JitI256, 
						 _init_beg: *const u8, 
						 _init_size: *const u64, 
						 _address: *const JitI256) {
	unimplemented!()
}

#[no_mangle]
pub extern fn env_call(_env: *mut Env, 
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
pub extern fn env_extcode(_env: *mut Env, _address: *const JitI256, _size: *const u64) {
	unimplemented!()
}

#[no_mangle]
pub extern fn env_log(_env: *mut Env,
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

	pub fn evmjit_create_context(data: *mut JitRuntimeData, env: *mut u8) -> *mut JitContext;
	pub fn evmjit_destroy_context(context: *mut JitContext);

	pub fn evmjit_exec(context: *mut JitContext) -> JitReturnCode;

}

#[test]
fn it_works() {
	unsafe {
		let mut env = 0u8;

		let data = evmjit_create_runtime_data();
		let context = evmjit_create_context(data, &mut env);

		let code = evmjit_exec(context);
		assert_eq!(code, JitReturnCode::Stop);

		evmjit_destroy_runtime_data(data);
		evmjit_destroy_context(context);
	}
}

