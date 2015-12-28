use std::mem;
use evmjit;
use util::hash::*;
use util::uint::*;
use util::bytes::*;
use util::sha3::*;
use evm;

/// Should be used to convert jit types to ethcore
trait FromJit<T>: Sized {
	fn from_jit(input: T) -> Self;
}

/// Should be used to covert ethcore types to jit
trait IntoJit<T> {
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

impl IntoJit<evmjit::I256> for Address {
	fn into_jit(self) -> evmjit::I256 {
		H256::from(self).into_jit()
	}
}

impl IntoJit<evmjit::RuntimeDataHandle> for evm::RuntimeData {
	fn into_jit(self) -> evmjit::RuntimeDataHandle {
		let mut data = evmjit::RuntimeDataHandle::new();
		data.gas = self.gas as i64;
		data.gas_price = self.gas_price as i64;
		data.call_data = self.call_data.as_ptr();
		data.call_data_size = self.call_data.len() as u64;
		mem::forget(self.call_data);
		data.address = self.address.into_jit();
		data.caller = self.caller.into_jit();
		data.origin = self.origin.into_jit();
		data.call_value = self.call_value.into_jit();
		data.coinbase = self.coinbase.into_jit();
		data.difficulty = self.difficulty.into_jit();
		data.gas_limit = self.gas_limit.into_jit();
		data.number = self.number;
		data.timestamp = self.timestamp as i64;
		data.code = self.code.as_ptr();
		data.code_size = self.code.len() as u64;
		data.code_hash = self.code.sha3().into_jit();
		mem::forget(self.code);
		data
	}
}

struct EnvAdapter<'a> {
	env: &'a mut evm::Env
}

impl<'a> EnvAdapter<'a> {
	fn new(env: &'a mut evm::Env) -> Self {
		EnvAdapter {
			env: env
		}
	}
}

impl<'a> evmjit::Env for EnvAdapter<'a> {
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

impl From<evmjit::ReturnCode> for evm::ReturnCode {
	fn from(code: evmjit::ReturnCode) -> Self {
		match code {
			evmjit::ReturnCode::Stop => evm::ReturnCode::Stop,
			evmjit::ReturnCode::Return => evm::ReturnCode::Return,
			evmjit::ReturnCode::Suicide => evm::ReturnCode::Suicide,
			evmjit::ReturnCode::OutOfGas => evm::ReturnCode::OutOfGas,
			_ => evm::ReturnCode::InternalError
		}
	}
}

pub struct JitEvm;

impl evm::Evm for JitEvm {
	fn exec(data: evm::RuntimeData, env: &mut evm::Env) -> evm::ReturnCode {
		// Dirty hack. This is unsafe, but we interact with ffi, so it's justified.
		let env_adapter: EnvAdapter<'static> = unsafe { ::std::mem::transmute(EnvAdapter::new(env)) };
		let mut env_handle = evmjit::EnvHandle::new(env_adapter);
		let mut context = unsafe { evmjit::ContextHandle::new(data.into_jit(), &mut env_handle) };
		From::from(context.exec())
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::*;
	use util::uint::*;
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

		let mut data = RuntimeData::new();
		data.coinbase = Address::from_str("2adc25665018aa1fe0e6bc666dac8fc2697ff9ba").unwrap();
		data.difficulty = U256::from(0x0100);
		data.gas_limit = U256::from(0x0f4240);
		data.number = 0;
		data.timestamp = 1;
		
		data.address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		data.caller = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		data.code = vec![0x60, 0x00, 0x60, 0x00, 0x55];
		data.gas = 0x174876e800;
		data.gas_price = 0x3b9aca00;
		data.origin = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		data.call_value = U256::from_str("0de0b6b3a7640000").unwrap();
		let mut env = Env::new();
		assert_eq!(JitEvm::exec(data, &mut env), ReturnCode::Stop);
	}

}
