use std::mem;
use evmjit;
use util::hash::*;
use util::uint::*;
use util::bytes::*;
use evm;

/// Should be used to convert jit types to ethcore
pub trait FromJit<T>: Sized {
	fn from_jit(input: T) -> Self;
}

/// Should be used to covert ethcore types to jit
pub trait IntoJit<T> {
	fn into_jit(self) -> T;
}

impl<'a> FromJit<&'a evmjit::I256> for U256 {
	fn from_jit(input: &'a evmjit::I256) -> Self {
		let mut res: U256 = unsafe { mem::uninitialized() };
		res.0[0] = input.words[3];
		res.0[1] = input.words[2];
		res.0[2] = input.words[1];
		res.0[3] = input.words[0];
		res
	}
}

impl<'a> FromJit<&'a evmjit::I256> for H256 {
	fn from_jit(input: &'a evmjit::I256) -> Self {
		let u = U256::from_jit(input);
		H256::from(&u)
	}
}

impl IntoJit<evmjit::I256> for U256 {
	fn into_jit(self) -> evmjit::I256 {
		let mut res: evmjit::I256 = unsafe { mem::uninitialized() };
		res.words[0] = self.0[3];
		res.words[1] = self.0[2];
		res.words[2] = self.0[1];
		res.words[3] = self.0[0];
		res
	}
}

impl IntoJit<evmjit::I256> for H256 {
	fn into_jit(self) -> evmjit::I256 {
		let mut ret = [0; 4];
		for i in 0..self.bytes().len() {
			let rev = self.bytes().len() - 1 - i;
			let pos = i / 8;
			ret[pos] += (self.bytes()[i] as u64) << (rev % 8) * 8;
		}
		evmjit::I256 { words: ret }
	}
}

pub struct EnvAdapter {
	env: evm::Env
}

impl EnvAdapter {
	pub fn new() -> EnvAdapter {
		EnvAdapter {
			env: evm::Env::new()
		}
	}
}

impl evmjit::Env for EnvAdapter {
	fn sload(&self, index: *const evmjit::I256, out_value: *mut evmjit::I256) {
		unsafe {
			let i = H256::from_jit(&*index);
			let o = self.env.sload(&i);
			*out_value = o.into_jit();
		}
	}

	fn sstore(&mut self, index: *const evmjit::I256, value: *const evmjit::I256) {
		unsafe {
			self.env.sstore(&H256::from_jit(&*index), &H256::from_jit(&*value));
		}
	}

	fn balance(&self, _address: *const evmjit::I256, _out_value: *mut evmjit::I256) {
		unimplemented!();
	}

	fn blockhash(&self, _number: *const evmjit::I256, _out_hash: *mut evmjit::I256) {
		unimplemented!();
	}

	fn create(&mut self,
			  _io_gas: *mut u64,
			  _endowment: *const evmjit::I256,
			  _init_beg: *const u8,
			  _init_size: *const u64,
			  _address: *mut evmjit::I256) {
		unimplemented!();
	}

	fn call(&mut self,
				_io_gas: *mut u64,
				_call_gas: *const u64,
				_receive_address: *const evmjit::I256,
				_value: *const evmjit::I256,
				_in_beg: *const u8,
				_in_size: *const u64,
				_out_beg: *mut u8,
				_out_size: *mut u64,
				_code_address: evmjit::I256) -> bool {
		unimplemented!();
	}

	fn log(&mut self,
		   _beg: *const u8,
		   _size: *const u64,
		   _topic1: *const evmjit::I256,
		   _topic2: *const evmjit::I256,
		   _topic3: *const evmjit::I256,
		   _topic4: *const evmjit::I256) {
		unimplemented!();
	}

	fn extcode(&self, _address: *const evmjit::I256, _size: *mut u64) -> *const u8 {
		unimplemented!();
	}
}

#[cfg(test)]
mod tests {
	use util::hash::*;
	use util::uint::*;
	use evmjit::{ContextHandle, RuntimeDataHandle, EnvHandle, ReturnCode};
	use evm::*;
	use evm::jit::{FromJit, IntoJit};

	#[test]
	fn test_to_and_from_u256() {
		use std::str::FromStr;

		let u = U256::from_str("d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap();
		let j = u.into_jit();
		let u2 = U256::from_jit(&j);
		assert_eq!(u, u2);
	}

	#[test]
	fn test_to_and_from_h256() {
		use std::str::FromStr;

		let h = H256::from_str("d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap();
		let j = h.clone().into_jit();
		let h2 = H256::from_jit(&j);
		assert_eq!(h, h2);
	}

	#[test]
	fn test_env_adapter() {
		let data = RuntimeDataHandle::new();
		let env = EnvAdapter::new();
		let mut context = ContextHandle::new(data, EnvHandle::new(env));
		assert_eq!(context.exec(), ReturnCode::Stop);
	}

}
