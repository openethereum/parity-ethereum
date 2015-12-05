//! Bare rust wrapper around evmjit
//! 
//! Requires latest version of Ethereum EVM JIT. https://github.com/debris/evmjit

extern crate libc;
use std::ptr;

#[repr(C)]
pub struct JitI256 {
	pub words: [u64; 4]
}

impl JitI256 {
	pub fn new() -> JitI256 {
		JitI256 {
			words: [0; 4]
		}
	}
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

impl JitRuntimeData {
	pub fn new() -> JitRuntimeData {
		JitRuntimeData { 
			gas: 0,
			gas_price: 0,
			call_data: ptr::null(),
			call_data_size: 0,
			address: JitI256::new(),
			caller: JitI256::new(),
			origin: JitI256::new(),
			call_value: JitI256::new(),
			coinbase: JitI256::new(),
			difficulty: JitI256::new(),
			gas_limit: JitI256::new(),
			number: 0,
			timestamp: 0,
			code: ptr::null(),
			code_size: 0,
			code_hash: JitI256::new()
		}
	}
}

#[repr(C)]
#[derive(Debug)]
pub enum JitReturnCode {
	Stop = 0,
	Return = 1,
	Suicide = 2,

	OutOfGas = -1,

	LLVMError = -101,
	UnexpectedError = -111
}

#[derive(Copy, Clone)]
pub enum JitContext {}

#[link(name="evmjit")]
extern "C" {
	pub fn evmjit_create(data: JitRuntimeData, env: u8) -> JitContext;
	pub fn evmjit_exec(context: JitContext) -> JitReturnCode;
	pub fn evmjit_destroy(context: JitContext);
}

#[test]
fn it_works() {
	unsafe {
		let context = evmjit_create(JitRuntimeData::new(), 0);
		let _result = evmjit_exec(context);
		evmjit_destroy(context);
	}
}

