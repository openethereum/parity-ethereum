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

pub enum JitContext {}

#[link(name="evmjit")]
extern "C" {
	pub fn evmjit_create_runtime_data() -> *mut JitRuntimeData;
	pub fn evmjit_destroy_runtime_data(data: *mut JitRuntimeData);

	pub fn evmjit_create_context(data: *mut JitRuntimeData, env: u8) -> *mut JitContext;
	pub fn evmjit_destroy_context(context: *mut JitContext);

	pub fn evmjit_exec(context: *mut JitContext) -> JitReturnCode;

}

#[test]
fn it_works() {
	unsafe {
		let data = evmjit_create_runtime_data();
		let context = evmjit_create_context(data, 0);

		let code = evmjit_exec(context);
		assert_eq!(code, JitReturnCode::Stop);

		evmjit_destroy_runtime_data(data);
		evmjit_destroy_context(context);
	}
}

