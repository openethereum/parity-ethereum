use std::mem;
use std::ptr;
use std::slice;
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
		unsafe {
			let mut res: U256 = mem::uninitialized();
			ptr::copy(input.words.as_ptr(), res.0.as_mut_ptr(), 4);
			res
		}
	}
}

impl<'a> FromJit<&'a evmjit::I256> for H256 {
	fn from_jit(input: &'a evmjit::I256) -> Self {
		let u = U256::from_jit(input);
		H256::from(&u)
	}
}

impl<'a> FromJit<&'a evmjit::I256> for Address {
	fn from_jit(input: &'a evmjit::I256) -> Self {
		Address::from(H256::from_jit(input))
	}
}

impl<'a> FromJit<&'a evmjit::H256> for H256 {
	fn from_jit(input: &'a evmjit::H256) -> Self {
		H256::from_jit(&evmjit::I256::from(input.clone()))
	}
}

impl<'a> FromJit<&'a evmjit::H256> for Address {
	fn from_jit(input: &'a evmjit::H256) -> Self {
		Address::from(H256::from_jit(input))
	}
}

impl IntoJit<evmjit::I256> for U256 {
	fn into_jit(self) -> evmjit::I256 {
		unsafe {
			let mut res: evmjit::I256 = mem::uninitialized();
			ptr::copy(self.0.as_ptr(), res.words.as_mut_ptr(), 4);
			res
		}
	}
}

impl IntoJit<evmjit::I256> for H256 {
	fn into_jit(self) -> evmjit::I256 {
		let mut ret = [0; 4];
		for i in 0..self.bytes().len() {
			let rev = self.bytes().len() - 1 - i;
			let pos = rev / 8;
			ret[pos] += (self.bytes()[i] as u64) << (rev % 8) * 8;
		}
		evmjit::I256 { words: ret }
	}
}

impl IntoJit<evmjit::H256> for H256 {
	fn into_jit(self) -> evmjit::H256 {
		let i: evmjit::I256 = self.into_jit();
		From::from(i)
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
			self.env.sstore(H256::from_jit(&*index), H256::from_jit(&*value));
		}
	}

	fn balance(&self, address: *const evmjit::H256, out_value: *mut evmjit::I256) {
		unsafe {
			let a = Address::from_jit(&*address);
			let o = self.env.balance(&a);
			*out_value = o.into_jit();
		}
	}

	fn blockhash(&self, number: *const evmjit::I256, out_hash: *mut evmjit::H256) {
		unsafe {
			let n = U256::from_jit(&*number);
			let o = self.env.blockhash(&n);
			*out_hash = o.into_jit();
		}
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
		   beg: *const u8,
		   size: u64,
		   topic1: *const evmjit::H256,
		   topic2: *const evmjit::H256,
		   topic3: *const evmjit::H256,
		   topic4: *const evmjit::H256) {

		unsafe {
			let mut topics = vec![];
			if !topic1.is_null() {
				topics.push(H256::from_jit(&*topic1));
			}

			if !topic2.is_null() {
				topics.push(H256::from_jit(&*topic2));
			}

			if !topic3.is_null() {
				topics.push(H256::from_jit(&*topic3));
			}

			if !topic4.is_null() {
				topics.push(H256::from_jit(&*topic4));
			}
		
			let bytes_ref: &[u8] = slice::from_raw_parts(beg, size as usize);
			self.env.log(topics, bytes_ref.to_vec());
		}
	}

	fn extcode(&self, address: *const evmjit::H256, size: *mut u64) -> *const u8 {
		unsafe {
			let code = self.env.extcode(&Address::from_jit(&*address));
			*size = code.len() as u64;
			let ptr = code.as_ptr();
			mem::forget(code);
			ptr
		}
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
	fn exec(&self, data: evm::RuntimeData, env: &mut evm::Env) -> evm::ReturnCode {
		// Dirty hack. This is unsafe, but we interact with ffi, so it's justified.
		let env_adapter: EnvAdapter<'static> = unsafe { ::std::mem::transmute(EnvAdapter::new(env)) };
		let mut env_handle = evmjit::EnvHandle::new(env_adapter);
		let mut context = unsafe { evmjit::ContextHandle::new(data.into_jit(), &mut env_handle) };
		From::from(context.exec())
	}
}

#[cfg(test)]
mod tests {
	use rustc_serialize::hex::FromHex;
	use std::str::FromStr;
	use util::hash::*;
	use util::uint::*;
	use evm::*;
	use evm::jit::{FromJit, IntoJit};
	use super::*;
	use state::*;
	use env_info::*;

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
		let j: ::evmjit::I256 = h.clone().into_jit();
		let h2 = H256::from_jit(&j);
		assert_eq!(h, h2);
	}

	#[test]
	fn test_to_and_from_address() {
		use std::str::FromStr;

		let a = Address::from_str("2adc25665018aa1fe0e6bc666dac8fc2697ff9ba").unwrap();
		let j = a.clone().into_jit();
		let a2 = Address::from_jit(&j);
		assert_eq!(a, a2);
	}

	#[test]
	fn test_env_add() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone(); 
		data.gas = 0x174876e800;
		data.code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

		let mut env = Env::new(EnvInfo::new(), State::new_temp(), address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let state = env.state();
		assert_eq!(state.storage_at(&address, &H256::new()), 
				   H256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap());
	}

	#[test]
	fn test_env_sha3_0() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone(); 
		data.gas = 0x174876e800;
		data.code = "6000600020600055".from_hex().unwrap();

		let mut env = Env::new(EnvInfo::new(), State::new_temp(), address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let state = env.state();
		assert_eq!(state.storage_at(&address, &H256::new()), 
				   H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap());
	}

	#[test]
	fn test_env_sha3_1() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone(); 
		data.gas = 0x174876e800;
		data.code = "6005600420600055".from_hex().unwrap();

		let mut env = Env::new(EnvInfo::new(), State::new_temp(), address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let state = env.state();
		assert_eq!(state.storage_at(&address, &H256::new()), 
				   H256::from_str("c41589e7559804ea4a2080dad19d876a024ccb05117835447d72ce08c1d020ec").unwrap());
	}

	#[test]
	fn test_origin() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone();
		data.origin = address.clone();
		data.gas = 0x174876e800;
		data.code = "32600055".from_hex().unwrap();

		let mut env = Env::new(EnvInfo::new(), State::new_temp(), address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let state = env.state();
		assert_eq!(Address::from(state.storage_at(&address, &H256::new())), address.clone());
	}

	#[test]
	fn test_caller() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone();
		data.caller = address.clone();
		data.gas = 0x174876e800;
		data.code = "33600055".from_hex().unwrap();

		let mut env = Env::new(EnvInfo::new(), State::new_temp(), address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let state = env.state();
		assert_eq!(Address::from(state.storage_at(&address, &H256::new())), address.clone());
	}

	#[test]
	fn test_extcode_copy0() {
		// 33 - caller
		// 3b - extcodesize
		// 60 00 - push 0
		// 60 00 - push 0
		// 33 - caller
		// 3c - extcodecopy
		// 60 00 - push 0
		// 51 - load word from memory
		// 60 00 - push 0
		// 55 - sstore

		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let caller = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let address_code = "333b60006000333c600051600055".from_hex().unwrap(); 
		let caller_code = "6005600055".from_hex().unwrap(); 
		let mut data = RuntimeData::new();
		data.address = address.clone();
		data.caller = caller.clone();
		data.origin = caller.clone();
		data.gas = 0x174876e800;
		data.code = address_code.clone();

		let mut state = State::new_temp();
		state.set_code(&address, address_code);
		state.set_code(&caller, caller_code);
		let mut env = Env::new(EnvInfo::new(), state, caller.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let state = env.state();
		assert_eq!(state.storage_at(&caller, &H256::new()), 
				   H256::from_str("6005600055000000000000000000000000000000000000000000000000000000").unwrap());
	}

	#[test]
	fn test_balance() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone();
		data.caller = address.clone();
		data.gas = 0x174876e800;
		data.code = "3331600055".from_hex().unwrap();

		let mut state = State::new_temp();
		state.add_balance(&address, &U256::from(0x10));
		let mut env = Env::new(EnvInfo::new(), state, address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let state = env.state();
		assert_eq!(state.storage_at(&address, &H256::new()), H256::from(&U256::from(0x10)));
	}

	#[test]
	fn test_empty_log() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone();
		data.caller = address.clone();
		data.gas = 0x174876e800;
		data.code = "60006000a0".from_hex().unwrap();

		let mut env = Env::new(EnvInfo::new(), State::new_temp(), address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let logs = env.logs();
		assert_eq!(logs.len(), 1);
		let log = &logs[0];
		assert_eq!(log.address(), &address);
		assert_eq!(log.topics().len(), 0);
		assert_eq!(log.bloom(), H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap());
	}

	#[test]
	fn test_log_with_one_topic() {
		// 60 ff - push ff
		// 60 00 - push 00
		// 53 - mstore 
		// 33 - caller
		// 60 20 - push 20
		// 60 00 - push 0
		// a1 - log with 1 topic

		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone();
		data.caller = address.clone();
		data.gas = 0x174876e800;
		data.code = "60ff6000533360206000a1".from_hex().unwrap();

		let mut env = Env::new(EnvInfo::new(), State::new_temp(), address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let logs = env.logs();
		assert_eq!(logs.len(), 1);
		let log = &logs[0];
		assert_eq!(log.address(), &address);
		assert_eq!(log.topics().len(), 1);
		let topic = &log.topics()[0];
		assert_eq!(topic, &H256::from_str("0000000000000000000000000f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap());
		assert_eq!(topic, &H256::from(address.clone()));
		assert_eq!(log.data(), &"ff00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap());
	}

	#[test]
	fn test_blockhash() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut data = RuntimeData::new();
		data.address = address.clone();
		data.caller = address.clone();
		data.gas = 0x174876e800;
		data.code = "600040600055".from_hex().unwrap();

		let mut info = EnvInfo::new();
		info.number = U256::one();
		info.last_hashes.push(H256::from(address.clone()));
		let mut env = Env::new(info, State::new_temp(), address.clone());
		let evm = JitEvm;
		assert_eq!(evm.exec(data, &mut env), ReturnCode::Stop);
		let state = env.state();
		assert_eq!(state.storage_at(&address, &H256::new()), H256::from(address.clone()));
	}
}
