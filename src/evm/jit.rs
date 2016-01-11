//! Just in time compiler execution environment.
use std::mem;
use std::ptr;
use std::slice;
use evmjit;
use util::hash::*;
use util::uint::*;
use util::bytes::*;
use util::sha3::*;
use evm;

/// Ethcore representation of evmjit runtime data.
pub struct RuntimeData {
	pub gas: U256,
	pub gas_price: U256,
	pub call_data: Vec<u8>,
	pub address: Address,
	pub caller: Address,
	pub origin: Address,
	pub call_value: U256,
	pub coinbase: Address,
	pub difficulty: U256,
	pub gas_limit: U256,
	pub number: u64,
	pub timestamp: u64,
	pub code: Vec<u8>
}

impl RuntimeData {
	pub fn new() -> RuntimeData {
		RuntimeData {
			gas: U256::zero(),
			gas_price: U256::zero(),
			call_data: vec![],
			address: Address::new(),
			caller: Address::new(),
			origin: Address::new(),
			call_value: U256::zero(),
			coinbase: Address::new(),
			difficulty: U256::zero(),
			gas_limit: U256::zero(),
			number: 0,
			timestamp: 0,
			code: vec![]
		}
	}
}

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

impl IntoJit<evmjit::H256> for Address {
	fn into_jit(self) -> evmjit::H256 {
		H256::from(self).into_jit()
	}
}

impl IntoJit<evmjit::RuntimeDataHandle> for RuntimeData {
	fn into_jit(self) -> evmjit::RuntimeDataHandle {
		let mut data = evmjit::RuntimeDataHandle::new();
		assert!(self.gas <= U256::from(u64::max_value()), "evmjit gas must be lower than 2 ^ 64");
		assert!(self.gas_price <= U256::from(u64::max_value()), "evmjit gas_price must be lower than 2 ^ 64");
		data.gas = self.gas.low_u64() as i64;
		data.gas_price = self.gas_price.low_u64() as i64;
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

/// Externalities adapter. Maps callbacks from evmjit to externalities trait.
/// 
/// Evmjit doesn't have to know about children execution failures. 
/// This adapter 'catches' them and moves upstream.
struct ExtAdapter<'a> {
	ext: &'a mut evm::Ext,
	err: &'a mut Option<evm::EvmError>
}

impl<'a> ExtAdapter<'a> {
	fn new(ext: &'a mut evm::Ext, err: &'a mut Option<evm::EvmError>) -> Self {
		ExtAdapter {
			ext: ext,
			err: err
		}
	}
}

impl<'a> evmjit::Ext for ExtAdapter<'a> {
	fn sload(&self, index: *const evmjit::I256, out_value: *mut evmjit::I256) {
		unsafe {
			let i = H256::from_jit(&*index);
			let o = self.ext.sload(&i);
			*out_value = o.into_jit();
		}
	}

	fn sstore(&mut self, index: *const evmjit::I256, value: *const evmjit::I256) {
		unsafe {
			self.ext.sstore(H256::from_jit(&*index), H256::from_jit(&*value));
		}
	}

	fn balance(&self, address: *const evmjit::H256, out_value: *mut evmjit::I256) {
		unsafe {
			let a = Address::from_jit(&*address);
			let o = self.ext.balance(&a);
			*out_value = o.into_jit();
		}
	}

	fn blockhash(&self, number: *const evmjit::I256, out_hash: *mut evmjit::H256) {
		unsafe {
			let n = U256::from_jit(&*number);
			let o = self.ext.blockhash(&n);
			*out_hash = o.into_jit();
		}
	}

	fn create(&mut self,
			  io_gas: *mut u64,
			  endowment: *const evmjit::I256,
			  init_beg: *const u8,
			  init_size: u64,
			  address: *mut evmjit::H256) {
		unsafe {
			match self.ext.create(*io_gas, &U256::from_jit(&*endowment), slice::from_raw_parts(init_beg, init_size as usize)) {
				Ok((gas_left, opt)) => {
					*io_gas = gas_left;
					if let Some(addr) = opt {
						*address = addr.into_jit();
					}
				},
				Err(err @ evm::EvmError::OutOfGas) => {
					*self.err = Some(err);
					// hack to propagate `OutOfGas` to evmjit and stop
					// the execution immediately.
					// Works, cause evmjit uses i64, not u64
					*io_gas = -1i64 as u64
				},
				Err(err) => *self.err = Some(err)
			}
		}
	}

	fn call(&mut self,
				io_gas: *mut u64,
				call_gas: u64,
				receive_address: *const evmjit::H256,
				value: *const evmjit::I256,
				in_beg: *const u8,
				in_size: u64,
				out_beg: *mut u8,
				out_size: u64,
				code_address: *const evmjit::H256) -> bool {
		unsafe {
			let res = self.ext.call(*io_gas, 
									call_gas, 
									&Address::from_jit(&*receive_address),
									&U256::from_jit(&*value),
									slice::from_raw_parts(in_beg, in_size as usize),
									&Address::from_jit(&*code_address),
									slice::from_raw_parts_mut(out_beg, out_size as usize));

			match res {
				Ok(gas_left) => {
					*io_gas = gas_left;
					true
				},
				Err(err @ evm::EvmError::OutOfGas) => {
					*self.err = Some(err);
					// hack to propagate `OutOfGas` to evmjit and stop
					// the execution immediately.
					// Works, cause evmjit uses i64, not u64
					*io_gas = -1i64 as u64;
					false
				},
				Err(err) => {
					*self.err = Some(err);
					false
				}
			}
		}
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
			self.ext.log(topics, bytes_ref.to_vec());
		}
	}

	fn extcode(&self, address: *const evmjit::H256, size: *mut u64) -> *const u8 {
		unsafe {
			let code = self.ext.extcode(&Address::from_jit(&*address));
			*size = code.len() as u64;
			let ptr = code.as_ptr();
			mem::forget(code);
			ptr
		}
	}
}

pub struct JitEvm;

impl evm::Evm for JitEvm {
	fn exec(&self, params: &evm::ActionParams, ext: &mut evm::Ext) -> evm::EvmResult {
		let mut optional_err = None;
		// Dirty hack. This is unsafe, but we interact with ffi, so it's justified.
		let ext_adapter: ExtAdapter<'static> = unsafe { ::std::mem::transmute(ExtAdapter::new(ext, &mut optional_err)) };
		let mut ext_handle = evmjit::ExtHandle::new(ext_adapter);
		let mut data = RuntimeData::new();
		data.gas = params.gas;
		data.gas_price = params.gas_price;
		data.call_data = params.data.clone();
		data.address = params.address.clone();
		data.caller = params.sender.clone();
		data.origin = params.origin.clone();
		data.call_value = params.value;
		data.code = params.code.clone();

		// TODO:
		data.coinbase = Address::new();
		data.difficulty = U256::zero();
		data.gas_limit = U256::zero();
		data.number = 0;
		data.timestamp = 0;
		
		let mut context = unsafe { evmjit::ContextHandle::new(data.into_jit(), &mut ext_handle) };
		let res = context.exec();
		
		// check in adapter if execution of children contracts failed.
		if let Some(err) = optional_err {
			return Err(err);
		}
		
		match res {
			evmjit::ReturnCode::Stop => Ok(U256::from(context.gas_left())),
			evmjit::ReturnCode::Return => ext.ret(context.gas_left(), context.output_data()).map(|gas_left| U256::from(gas_left)),
			evmjit::ReturnCode::Suicide => { 
				// what if there is a suicide and we run out of gas just after?
				ext.suicide();
				Ok(U256::from(context.gas_left()))
			},
			evmjit::ReturnCode::OutOfGas => Err(evm::EvmError::OutOfGas),
			_err => Err(evm::EvmError::Internal)
		}
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
	use engine::*;
	use schedule::*;
	use spec::*;

	struct TestEngine;

	impl TestEngine {
		fn new() -> Self { TestEngine }
	}

	impl Engine for TestEngine {
		fn name(&self) -> &str { "TestEngine" }
		fn spec(&self) -> &Spec { unimplemented!() }
		fn schedule(&self, _env_info: &EnvInfo) -> Schedule { Schedule::new_frontier() }
	}

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
		let j: ::evmjit::I256 = a.clone().into_jit();
		let a2 = Address::from_jit(&j);
		assert_eq!(a, a2);
	}

	#[test]
	fn test_ext_add() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone(); 
		params.gas = U256::from(0x174876e800u64);
		params.code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

		let mut state = State::new_temp();
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop);
		}

		assert_eq!(state.storage_at(&address, &H256::new()), 
				   H256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap());
	}

	#[test]
	fn test_ext_sha3_0() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone(); 
		params.gas = U256::from(0x174876e800u64);
		params.code = "6000600020600055".from_hex().unwrap();

		let mut state = State::new_temp();
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}

		assert_eq!(state.storage_at(&address, &H256::new()), 
				   H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap());
	}

	#[test]
	fn test_ext_sha3_1() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone(); 
		params.gas = U256::from(0x174876e800u64);
		params.code = "6005600420600055".from_hex().unwrap();

		let mut state = State::new_temp();
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}

		assert_eq!(state.storage_at(&address, &H256::new()), 
				   H256::from_str("c41589e7559804ea4a2080dad19d876a024ccb05117835447d72ce08c1d020ec").unwrap());
	}

	#[test]
	fn test_origin() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone();
		params.origin = address.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "32600055".from_hex().unwrap();

		let mut state = State::new_temp();
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}

		assert_eq!(Address::from(state.storage_at(&address, &H256::new())), address.clone());
	}

	#[test]
	fn test_sender() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone();
		params.sender = address.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "32600055".from_hex().unwrap();
		params.code = "33600055".from_hex().unwrap();

		let mut state = State::new_temp();
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}

		assert_eq!(Address::from(state.storage_at(&address, &H256::new())), address.clone());
	}

	#[test]
	fn test_extcode_copy0() {
		// 33 - sender
		// 3b - extcodesize
		// 60 00 - push 0
		// 60 00 - push 0
		// 33 - sender
		// 3c - extcodecopy
		// 60 00 - push 0
		// 51 - load word from memory
		// 60 00 - push 0
		// 55 - sstore

		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let address_code = "333b60006000333c600051600055".from_hex().unwrap(); 
		let sender_code = "6005600055".from_hex().unwrap(); 
		let mut params = ActionParams::new();
		params.address = address.clone();
		params.sender = sender.clone();
		params.origin = sender.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = address_code.clone();

		let mut state = State::new_temp();
		state.init_code(&address, address_code);
		state.init_code(&sender, sender_code);
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}

		assert_eq!(state.storage_at(&address, &H256::new()), 
				   H256::from_str("6005600055000000000000000000000000000000000000000000000000000000").unwrap());
	}

	#[test]
	fn test_balance() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone();
		params.sender = address.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "3331600055".from_hex().unwrap();

		let mut state = State::new_temp();
		state.add_balance(&address, &U256::from(0x10));
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}

		assert_eq!(state.storage_at(&address, &H256::new()), H256::from(&U256::from(0x10)));
	}

	#[test]
	fn test_empty_log() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "60006000a0".from_hex().unwrap();

		let mut state = State::new_temp();
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();
		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}
		let logs = substate.logs();
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
		// 33 - sender
		// 60 20 - push 20
		// 60 00 - push 0
		// a1 - log with 1 topic

		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone();
		params.sender = address.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "60ff6000533360206000a1".from_hex().unwrap();

		let mut state = State::new_temp();
		let info = EnvInfo::new();
		let engine = TestEngine::new();
		let mut substate = Substate::new();
		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}
		let logs = substate.logs();
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
		let mut params = ActionParams::new();
		params.address = address.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "600040600055".from_hex().unwrap();

		let mut state = State::new_temp();
		let mut info = EnvInfo::new();
		info.number = 1;
		info.last_hashes.push(H256::from(address.clone()));
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}

		assert_eq!(state.storage_at(&address, &H256::new()), H256::from(address.clone()));
	}

	#[test]
	fn test_calldataload() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let mut params = ActionParams::new();
		params.address = address.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "600135600055".from_hex().unwrap();
		params.data = "0123ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff23".from_hex().unwrap();

		let mut state = State::new_temp();
		let mut info = EnvInfo::new();
		info.number = 1;
		info.last_hashes.push(H256::from(address.clone()));
		let engine = TestEngine::new();
		let mut substate = Substate::new();

		{
			let mut ext = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::InitContract);
			let evm = JitEvm;
			let _res = evm.exec(&params, &mut ext);
			//assert_eq!(evm.exec(&params, &mut ext), EvmResult::Stop {});
		}

		assert_eq!(state.storage_at(&address, &H256::new()), H256::from_str("23ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff23").unwrap());
	}
}
