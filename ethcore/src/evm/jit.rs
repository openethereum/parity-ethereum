// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Just in time compiler execution environment.
use common::*;
use evmjit;
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
			ret[pos] += (self.bytes()[i] as u64) << ((rev % 8) * 8);
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

/// Externalities adapter. Maps callbacks from evmjit to externalities trait.
/// 
/// Evmjit doesn't have to know about children execution failures. 
/// This adapter 'catches' them and moves upstream.
struct ExtAdapter<'a> {
	ext: &'a mut evm::Ext,
	address: Address
}

impl<'a> ExtAdapter<'a> {
	fn new(ext: &'a mut evm::Ext, address: Address) -> Self {
		ExtAdapter {
			ext: ext,
			address: address
		}
	}
}

impl<'a> evmjit::Ext for ExtAdapter<'a> {
	fn sload(&self, key: *const evmjit::I256, out_value: *mut evmjit::I256) {
		unsafe {
			let i = H256::from_jit(&*key);
			let o = self.ext.storage_at(&i);
			*out_value = o.into_jit();
		}
	}

	fn sstore(&mut self, key: *const evmjit::I256, value: *const evmjit::I256) {
		let key = unsafe { H256::from_jit(&*key) };
		let value = unsafe { H256::from_jit(&*value) };
		let old_value = self.ext.storage_at(&key);
		// if SSTORE nonzero -> zero, increment refund count
		if !old_value.is_zero() && value.is_zero() {
			self.ext.inc_sstore_clears();
		}
		self.ext.set_storage(key, value);
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
			  value: *const evmjit::I256,
			  init_beg: *const u8,
			  init_size: u64,
			  address: *mut evmjit::H256) {
			
		let gas = unsafe { U256::from(*io_gas) };
		let value = unsafe { U256::from_jit(&*value) };
		let code = unsafe { slice::from_raw_parts(init_beg, init_size as usize) };

		// check if balance is sufficient and we are not too deep
		if self.ext.balance(&self.address) >= value && self.ext.depth() < self.ext.schedule().max_depth {
			match self.ext.create(&gas, &value, code) {
				evm::ContractCreateResult::Created(new_address, gas_left) => unsafe {
					*address = new_address.into_jit();
					*io_gas = gas_left.low_u64();
				},
				evm::ContractCreateResult::Failed => unsafe {
					*address = Address::new().into_jit();
					*io_gas = 0;
				}
			}
		} else {
			unsafe { *address = Address::new().into_jit(); }
		}
	}

	fn call(&mut self,
				io_gas: *mut u64,
				call_gas: u64,
				sender_address: *const evmjit::H256,
				receive_address: *const evmjit::H256,
				code_address: *const evmjit::H256,
				transfer_value: *const evmjit::I256,
				_apparent_value: *const evmjit::I256,
				in_beg: *const u8,
				in_size: u64,
				out_beg: *mut u8,
				out_size: u64) -> bool {

		let mut gas = unsafe { U256::from(*io_gas) };
		let mut call_gas = U256::from(call_gas);
		let mut gas_cost = call_gas;
		let sender_address = unsafe { Address::from_jit(&*sender_address) };
		let receive_address = unsafe { Address::from_jit(&*receive_address) };
		let code_address = unsafe { Address::from_jit(&*code_address) };
		let transfer_value = unsafe { U256::from_jit(&*transfer_value) };
		let value = Some(transfer_value);

		// receive address and code address are the same in normal calls
		let is_callcode = receive_address != code_address;

		if !is_callcode && !self.ext.exists(&code_address) {
			gas_cost = gas_cost + U256::from(self.ext.schedule().call_new_account_gas);
		}

		if transfer_value > U256::zero() {
			assert!(self.ext.schedule().call_value_transfer_gas > self.ext.schedule().call_stipend, "overflow possible");
			gas_cost = gas_cost + U256::from(self.ext.schedule().call_value_transfer_gas);
			call_gas = call_gas + U256::from(self.ext.schedule().call_stipend);
		}

		if gas_cost > gas {
			unsafe {
				*io_gas = -1i64 as u64;
				return false;
			}
		}

		gas = gas - gas_cost;

		// check if balance is sufficient and we are not too deep
		if self.ext.balance(&self.address) < transfer_value || self.ext.depth() >= self.ext.schedule().max_depth {
			unsafe {
				*io_gas = (gas + call_gas).low_u64();
				return false;
			}
		}

		match self.ext.call(
					  &call_gas, 
					  &sender_address,
					  &receive_address, 
					  value,
					  unsafe { slice::from_raw_parts(in_beg, in_size as usize) },
					  &code_address,
					  unsafe { slice::from_raw_parts_mut(out_beg, out_size as usize) }) {
			evm::MessageCallResult::Success(gas_left) => unsafe {
				*io_gas = (gas + gas_left).low_u64();
				true
			},
			evm::MessageCallResult::Failed => unsafe {
				*io_gas = gas.low_u64();
				false
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
			self.ext.log(topics, bytes_ref);
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
	fn exec(&self, params: ActionParams, ext: &mut evm::Ext) -> evm::Result {
		// Dirty hack. This is unsafe, but we interact with ffi, so it's justified.
		let ext_adapter: ExtAdapter<'static> = unsafe { ::std::mem::transmute(ExtAdapter::new(ext, params.address.clone())) };
		let mut ext_handle = evmjit::ExtHandle::new(ext_adapter);
		assert!(params.gas <= U256::from(i64::max_value() as u64), "evmjit max gas is 2 ^ 63");
		assert!(params.gas_price <= U256::from(i64::max_value() as u64), "evmjit max gas is 2 ^ 63");

		let call_data = params.data.unwrap_or_else(Vec::new);
		let code = params.code.unwrap_or_else(Vec::new);

		let mut data = evmjit::RuntimeDataHandle::new();
		data.gas = params.gas.low_u64() as i64;
		data.gas_price = params.gas_price.low_u64() as i64;
		data.call_data = call_data.as_ptr();
		data.call_data_size = call_data.len() as u64;
		mem::forget(call_data);
		data.code = code.as_ptr();
		data.code_size = code.len() as u64;
		data.code_hash = code.sha3().into_jit();
		mem::forget(code);
		data.address = params.address.into_jit();
		data.caller = params.sender.into_jit();
		data.origin = params.origin.into_jit();
		data.transfer_value = match params.value {
			ActionValue::Transfer(val) => val.into_jit(),
			ActionValue::Apparent(val) => val.into_jit()
		};
		data.apparent_value = data.transfer_value;

		let mut schedule = evmjit::ScheduleHandle::new();
		schedule.have_delegate_call = ext.schedule().have_delegate_call;

		data.author = ext.env_info().author.clone().into_jit();
		data.difficulty = ext.env_info().difficulty.into_jit();
		data.gas_limit = ext.env_info().gas_limit.into_jit();
		data.number = ext.env_info().number;
		// don't really know why jit timestamp is int..
		data.timestamp = ext.env_info().timestamp as i64;

		let mut context = unsafe { evmjit::ContextHandle::new(data, schedule, &mut ext_handle) };
		let res = context.exec();
		
		match res {
			evmjit::ReturnCode::Stop => Ok(U256::from(context.gas_left())),
			evmjit::ReturnCode::Return => ext.ret(&U256::from(context.gas_left()), context.output_data()),
			evmjit::ReturnCode::Suicide => { 
				ext.suicide(&Address::from_jit(&context.suicide_refund_address()));
				Ok(U256::from(context.gas_left()))
			},
			evmjit::ReturnCode::OutOfGas => Err(evm::Error::OutOfGas),
			_err => Err(evm::Error::Internal)
		}
	}
}

#[test]
fn test_to_and_from_u256() {
	let u = U256::from_str("d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap();
	let j = u.into_jit();
	let u2 = U256::from_jit(&j);
	assert_eq!(u, u2);
}

#[test]
fn test_to_and_from_h256() {
	let h = H256::from_str("d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap();
	let j: ::evmjit::I256 = h.clone().into_jit();
	let h2 = H256::from_jit(&j);
	
	assert_eq!(h, h2);

	let j: ::evmjit::H256 = h.clone().into_jit();
	let h2 = H256::from_jit(&j);
	assert_eq!(h, h2);
}

#[test]
fn test_to_and_from_address() {
	let a = Address::from_str("2adc25665018aa1fe0e6bc666dac8fc2697ff9ba").unwrap();
	let j: ::evmjit::I256 = a.clone().into_jit();
	let a2 = Address::from_jit(&j);

	assert_eq!(a, a2);

	let j: ::evmjit::H256 = a.clone().into_jit();
	let a2 = Address::from_jit(&j);
	assert_eq!(a, a2);
}
