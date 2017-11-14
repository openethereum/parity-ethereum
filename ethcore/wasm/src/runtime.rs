// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Wasm evm program runtime intstance

use std::sync::Arc;

use byteorder::{LittleEndian, ByteOrder};

use vm;
use parity_wasm::interpreter;
use wasm_utils::rules;
use bigint::prelude::U256;
use bigint::hash::H256;
use util::Address;

use vm::CallType;
use super::ptr::{WasmPtr, Error as PtrError};

/// User trap in native code
#[derive(Debug, Clone, PartialEq)]
pub enum UserTrap {
	/// Storage read error
	StorageReadError,
	/// Storage update error
	StorageUpdateError,
	/// Memory access violation
	MemoryAccessViolation,
	/// Native code resulted in suicide
	Suicide,
	/// Suicide was requested but coudn't complete
	SuicideAbort,
	/// Invalid gas state inside interpreter
	InvalidGasState,
	/// Query of the balance resulted in an error
	BalanceQueryError,
	/// Failed allocation
	AllocationFailed,
	/// Gas limit reached
	GasLimit,
	/// Unknown runtime function
	Unknown,
	/// Passed string had invalid utf-8 encoding
	BadUtf8,
	/// Log event error
	Log,
	/// Other error in native code
	Other,
	/// Panic with message
	Panic(String),
}

impl ::std::fmt::Display for UserTrap {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		match *self {
			UserTrap::StorageReadError => write!(f, "Storage read error"),
			UserTrap::StorageUpdateError => write!(f, "Storage update error"),
			UserTrap::MemoryAccessViolation => write!(f, "Memory access violation"),
			UserTrap::SuicideAbort => write!(f, "Attempt to suicide resulted in an error"),
			UserTrap::InvalidGasState => write!(f, "Invalid gas state"),
			UserTrap::BalanceQueryError => write!(f, "Balance query resulted in an error"),
			UserTrap::Suicide => write!(f, "Suicide result"),
			UserTrap::Unknown => write!(f, "Unknown runtime function invoked"),
			UserTrap::AllocationFailed => write!(f, "Memory allocation failed (OOM)"),
			UserTrap::BadUtf8 => write!(f, "String encoding is bad utf-8 sequence"),
			UserTrap::GasLimit => write!(f, "Invocation resulted in gas limit violated"),
			UserTrap::Log => write!(f, "Error occured while logging an event"),
			UserTrap::Other => write!(f, "Other unspecified error"),
			UserTrap::Panic(ref msg) => write!(f, "Panic: {}", msg),
		}
	}
}

impl interpreter::UserError for UserTrap { }

pub type InterpreterError = interpreter::Error<UserTrap>;
pub type InterpreterMemoryInstance = interpreter::MemoryInstance<UserTrap>;
pub type InterpreterProgramInstance = interpreter::ProgramInstance<UserTrap>;
pub type InterpreterCallerContext<'a> = interpreter::CallerContext<'a, UserTrap>;

impl From<PtrError> for UserTrap {
	fn from(err: PtrError) -> Self {
		match err {
			PtrError::AccessViolation => UserTrap::MemoryAccessViolation,
		}
	}
}

pub struct RuntimeContext {
	pub address: Address,
	pub sender: Address,
	pub origin: Address,
	pub value: U256,
}

/// Runtime enviroment data for wasm contract execution
pub struct Runtime<'a, 'b> {
	gas_counter: u64,
	gas_limit: u64,
	dynamic_top: u32,
	ext: &'a mut vm::Ext,
	memory: Arc<InterpreterMemoryInstance>,
	context: RuntimeContext,
	instance: &'b InterpreterProgramInstance,
	gas_rules: rules::Set,
}

impl<'a, 'b> Runtime<'a, 'b> {
	/// New runtime for wasm contract with specified params
	pub fn with_params<'c, 'd>(
		ext: &'c mut vm::Ext,
		memory: Arc<InterpreterMemoryInstance>,
		stack_space: u32,
		gas_limit: u64,
		context: RuntimeContext,
		program_instance: &'d InterpreterProgramInstance,
	) -> Runtime<'c, 'd> {

		let rules = {
			let schedule = ext.schedule();

			rules::Set::new({
				let mut vals = ::std::collections::HashMap::with_capacity(4);
				vals.insert(rules::InstructionType::Load, schedule.wasm.mem as u32);
				vals.insert(rules::InstructionType::Store, schedule.wasm.mem as u32);
				vals.insert(rules::InstructionType::Div, schedule.wasm.div as u32);
				vals.insert(rules::InstructionType::Mul, schedule.wasm.mul as u32);
				vals
			})
		};

		Runtime {
			gas_counter: 0,
			gas_limit: gas_limit,
			dynamic_top: stack_space,
			memory: memory,
			ext: ext,
			context: context,
			instance: program_instance,
			gas_rules: rules,
		}
	}

	/// Write to the storage from wasm memory
	pub fn storage_write(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let mut context = context;
		let val = self.pop_h256(&mut context)?;
		let key = self.pop_h256(&mut context)?;
		trace!(target: "wasm", "storage_write: value {} at @{}", &val, &key);

		self.charge(|schedule| schedule.sstore_set_gas as u64)?;

		self.ext.set_storage(key, val).map_err(|_| UserTrap::StorageUpdateError)?;

		Ok(Some(0i32.into()))
	}

	/// Read from the storage to wasm memory
	pub fn storage_read(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let mut context = context;
		let val_ptr = context.value_stack.pop_as::<i32>()?;
		let key = self.pop_h256(&mut context)?;
		let val = self.ext.storage_at(&key).map_err(|_| UserTrap::StorageReadError)?;

		self.charge(|schedule| schedule.sload_gas as u64)?;

		self.memory.set(val_ptr as u32, &*val)?;

		Ok(Some(0.into()))
	}

	/// Fetches balance for address
	pub fn balance(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let mut context = context;
		let return_ptr = context.value_stack.pop_as::<i32>()? as u32;
		let address = self.pop_address(&mut context)?;

		self.charge(|schedule| schedule.balance_gas as u64)?;

		let balance = self.ext.balance(&address).map_err(|_| UserTrap::BalanceQueryError)?;
		let value: H256 = balance.into();
		self.memory.set(return_ptr, &*value)?;
		Ok(None)
	}

	/// Pass suicide to state runtime
	pub fn suicide(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let mut context = context;
		let refund_address = self.pop_address(&mut context)?;

		if self.ext.exists(&refund_address).map_err(|_| UserTrap::SuicideAbort)? {
			trace!(target: "wasm", "Suicide: refund to existing address {}", refund_address);
			self.charge(|schedule| schedule.suicide_gas as u64)?;
		} else {
			trace!(target: "wasm", "Suicide: refund to new address {}", refund_address);
			self.charge(|schedule| schedule.suicide_to_new_account_cost as u64)?;
		}

		self.ext.suicide(&refund_address).map_err(|_| UserTrap::SuicideAbort)?;

		// We send trap to interpreter so it should abort further execution
		Err(UserTrap::Suicide.into())
	}

	/// Charge gas according to closure
	pub fn charge<F>(&mut self, f: F) -> Result<(), InterpreterError>
		where F: FnOnce(&vm::Schedule) -> u64
	{
		let amount = f(self.ext.schedule());
		if !self.charge_gas(amount as u64) {
			Err(UserTrap::GasLimit.into())
		} else {
			Ok(())
		}
	}

	pub fn overflow_charge<F>(&mut self, f: F) -> Result<(), InterpreterError>
		where F: FnOnce(&vm::Schedule) -> Option<u64>
	{
		let amount = match f(self.ext.schedule()) {
			Some(amount) => amount,
			None => { return Err(UserTrap::GasLimit.into()); }
		};

		if !self.charge_gas(amount as u64) {
			Err(UserTrap::GasLimit.into())
		} else {
			Ok(())
		}
	}

	/// Invoke create in the state runtime
	pub fn create(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		//
		// method signature:
		//   fn create(endowment: *const u8, code_ptr: *const u8, code_len: u32, result_ptr: *mut u8) -> i32;
		//

		trace!(target: "wasm", "runtime: create contract");
		let mut context = context;
		let result_ptr = context.value_stack.pop_as::<i32>()? as u32;
		trace!(target: "wasm", "result_ptr: {:?}", result_ptr);
		let code_len = context.value_stack.pop_as::<i32>()? as u32;
		trace!(target: "wasm", "  code_len: {:?}", code_len);
		let code_ptr = context.value_stack.pop_as::<i32>()? as u32;
		trace!(target: "wasm", "  code_ptr: {:?}", code_ptr);
		let endowment = self.pop_u256(&mut context)?;
		trace!(target: "wasm", "       val: {:?}", endowment);

		let code = self.memory.get(code_ptr, code_len as usize)?;

		self.charge(|schedule| schedule.create_gas as u64)?;
		self.charge(|schedule| schedule.create_data_gas as u64 * code.len() as u64)?;

		let gas_left = self.gas_left()
			.map_err(|_| UserTrap::InvalidGasState)?
			.into();

		match self.ext.create(&gas_left, &endowment, &code, vm::CreateContractAddress::FromSenderAndCodeHash) {
			vm::ContractCreateResult::Created(address, gas_left) => {
				self.memory.set(result_ptr, &*address)?;
				self.gas_counter = self.gas_limit - gas_left.low_u64();
				trace!(target: "wasm", "runtime: create contract success (@{:?})", address);
				Ok(Some(0i32.into()))
			},
			vm::ContractCreateResult::Failed => {
				trace!(target: "wasm", "runtime: create contract fail");
				Ok(Some((-1i32).into()))
			},
			vm::ContractCreateResult::Reverted(gas_left, _) => {
				trace!(target: "wasm", "runtime: create contract reverted");
				self.gas_counter = self.gas_limit - gas_left.low_u64();
				Ok(Some((-1i32).into()))
			},
		}
	}

	pub fn call(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		//
		// method signature:
		// fn (
		// 	address: *const u8,
		// 	val_ptr: *const u8,
		// 	input_ptr: *const u8,
		// 	input_len: u32,
		// 	result_ptr: *mut u8,
		// 	result_len: u32,
		// ) -> i32

		self.do_call(true, CallType::Call, context)
	}


	fn call_code(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		//
		// signature (same as static call):
		// fn (
		// 	address: *const u8,
		// 	input_ptr: *const u8,
		// 	input_len: u32,
		// 	result_ptr: *mut u8,
		// 	result_len: u32,
		// ) -> i32

		self.do_call(false, CallType::CallCode, context)
	}

	fn do_call(
		&mut self,
		use_val: bool,
		call_type: CallType,
		context: InterpreterCallerContext,
	)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{

		trace!(target: "wasm", "runtime: call code");
		let mut context = context;
		let result_alloc_len = context.value_stack.pop_as::<i32>()? as u32;
		trace!(target: "wasm", "    result_len: {:?}", result_alloc_len);

		let result_ptr = context.value_stack.pop_as::<i32>()? as u32;
		trace!(target: "wasm", "    result_ptr: {:?}", result_ptr);

		let input_len = context.value_stack.pop_as::<i32>()? as u32;
		trace!(target: "wasm", "     input_len: {:?}", input_len);

		let input_ptr = context.value_stack.pop_as::<i32>()? as u32;
		trace!(target: "wasm", "     input_ptr: {:?}", input_ptr);

		let val = if use_val { Some(self.pop_u256(&mut context)?) }
		else { None };
		trace!(target: "wasm", "           val: {:?}", val);

		let address = self.pop_address(&mut context)?;
		trace!(target: "wasm", "       address: {:?}", address);

		if let Some(ref val) = val {
			let address_balance = self.ext.balance(&self.context.address)
				.map_err(|_| UserTrap::BalanceQueryError)?;

			if &address_balance < val {
				trace!(target: "wasm", "runtime: call failed due to balance check");
				return Ok(Some((-1i32).into()));
			}
		}

		self.charge(|schedule| schedule.call_gas as u64)?;

		let mut result = Vec::with_capacity(result_alloc_len as usize);
		result.resize(result_alloc_len as usize, 0);
		let gas = self.gas_left()
			.map_err(|_| UserTrap::InvalidGasState)?
			.into();
		// todo: optimize to use memory views once it's in
		let payload = self.memory.get(input_ptr, input_len as usize)?;

		let call_result = self.ext.call(
			&gas,
			&self.context.sender,
			&self.context.address,
			val,
			&payload,
			&address,
			&mut result[..],
			call_type,
		);

		match call_result {
			vm::MessageCallResult::Success(gas_left, _) => {
				self.gas_counter = self.gas_limit - gas_left.low_u64();
				self.memory.set(result_ptr, &result)?;
				Ok(Some(0i32.into()))
			},
			vm::MessageCallResult::Reverted(gas_left, _) => {
				self.gas_counter = self.gas_limit - gas_left.low_u64();
				self.memory.set(result_ptr, &result)?;
				Ok(Some((-1i32).into()))
			},
			vm::MessageCallResult::Failed  => {
				Ok(Some((-1i32).into()))
			}
		}
	}

	pub fn static_call(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		// signature (same as code call):
		// fn (
		// 	address: *const u8,
		// 	input_ptr: *const u8,
		// 	input_len: u32,
		// 	result_ptr: *mut u8,
		// 	result_len: u32,
		// ) -> i32

		self.do_call(false, CallType::StaticCall, context)
	}


	/// Allocate memory using the wasm stack params
	pub fn malloc(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let amount = context.value_stack.pop_as::<i32>()? as u32;

		self.charge(|schedule| schedule.wasm.alloc as u64 * amount as u64)?;

		let previous_top = self.dynamic_top;
		self.dynamic_top = previous_top + amount;
		Ok(Some((previous_top as i32).into()))
	}

	/// Allocate memory in wasm memory instance
	pub fn alloc(&mut self, amount: u32) -> Result<u32, UserTrap> {
		let previous_top = self.dynamic_top;
		self.dynamic_top = previous_top + amount;
		Ok(previous_top.into())
	}

	/// Report gas cost with the params passed in wasm stack
	fn gas(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let amount = context.value_stack.pop_as::<i32>()? as u64;
		if self.charge_gas(amount) {
			Ok(None)
		} else {
			Err(UserTrap::GasLimit.into())
		}
	}

	fn charge_gas(&mut self, amount: u64) -> bool {
		let prev = self.gas_counter;
		if prev + amount > self.gas_limit {
			// exceeds gas
			false
		} else {
			self.gas_counter = prev + amount;
			true
		}
	}

	fn h256_at(&self, ptr: WasmPtr) -> Result<H256, InterpreterError> {
		Ok(H256::from_slice(&ptr.slice(32, &*self.memory)
			.map_err(|_| UserTrap::MemoryAccessViolation)?
		))
	}

	fn pop_h256(&self, context: &mut InterpreterCallerContext) -> Result<H256, InterpreterError> {
		let ptr = WasmPtr::from_i32(context.value_stack.pop_as::<i32>()?)
			.map_err(|_| UserTrap::MemoryAccessViolation)?;
		self.h256_at(ptr)
	}

	fn pop_u256(&self, context: &mut InterpreterCallerContext) -> Result<U256, InterpreterError> {
		let ptr = WasmPtr::from_i32(context.value_stack.pop_as::<i32>()?)
			.map_err(|_| UserTrap::MemoryAccessViolation)?;
		self.h256_at(ptr).map(Into::into)
	}

	fn address_at(&self, ptr: WasmPtr) -> Result<Address, InterpreterError> {
		Ok(Address::from_slice(&ptr.slice(20, &*self.memory)
			.map_err(|_| UserTrap::MemoryAccessViolation)?
		))
	}

	fn pop_address(&self, context: &mut InterpreterCallerContext) -> Result<Address, InterpreterError> {
		let ptr = WasmPtr::from_i32(context.value_stack.pop_as::<i32>()?)
			.map_err(|_| UserTrap::MemoryAccessViolation)?;
		self.address_at(ptr)
	}

	fn unknown_trap(&mut self, _context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, UserTrap>
	{
		Err(UserTrap::Unknown)
	}

	fn user_noop(&mut self,
		_context: InterpreterCallerContext
	) -> Result<Option<interpreter::RuntimeValue>, InterpreterError> {
		Ok(None)
	}

	/// Write call descriptor to wasm memory
	pub fn write_descriptor(&mut self, input: &[u8]) -> Result<WasmPtr, InterpreterError> {
		let d_ptr = self.alloc(16)?;

		let args_len = input.len() as u32;
		let args_ptr = self.alloc(args_len)?;

		// write call descriptor
		// call descriptor is [args_ptr, args_len, return_ptr, return_len]
		//   all are 4 byte length, last 2 are zeroed
		let mut d_buf = [0u8; 16];
		LittleEndian::write_u32(&mut d_buf[0..4], args_ptr);
		LittleEndian::write_u32(&mut d_buf[4..8], args_len);
		self.memory.set(d_ptr, &d_buf)?;

		// write call args to memory
		self.memory.set(args_ptr, input)?;

		Ok(d_ptr.into())
	}

	fn debug_log(&mut self, context: InterpreterCallerContext)
			-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let msg_len = context.value_stack.pop_as::<i32>()? as u32;
		let msg_ptr = context.value_stack.pop_as::<i32>()? as u32;

		let msg = String::from_utf8(self.memory.get(msg_ptr, msg_len as usize)?)
			.map_err(|_| UserTrap::BadUtf8)?;

		trace!(target: "wasm", "Contract debug message: {}", msg);

		Ok(None)
	}

	/// Query current gas left for execution
	pub fn gas_left(&self) -> Result<u64, UserTrap> {
		if self.gas_counter > self.gas_limit { return Err(UserTrap::InvalidGasState); }
		Ok(self.gas_limit - self.gas_counter)
	}

	/// Shared memory reference
	pub fn memory(&self) -> &InterpreterMemoryInstance {
		&*self.memory
	}

	fn mem_copy(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		//
		// method signature:
		//   fn memcpy(dest: *const u8, src: *const u8, len: u32) -> *mut u8;
		//

		let len = context.value_stack.pop_as::<i32>()? as u32;
		let src = context.value_stack.pop_as::<i32>()? as u32;
		let dst = context.value_stack.pop_as::<i32>()? as u32;

		self.charge(|schedule| schedule.wasm.mem_copy as u64 * len as u64)?;

		self.memory().copy_nonoverlapping(src as usize, dst as usize, len as usize)?;

		Ok(Some(Into::into(dst as i32)))
	}

	fn mem_move(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		//
		// method signature:
		//   fn memmove(dest: *const u8, src: *const u8, len: u32) -> *mut u8;
		//

		let len = context.value_stack.pop_as::<i32>()? as u32;
		let src = context.value_stack.pop_as::<i32>()? as u32;
		let dst = context.value_stack.pop_as::<i32>()? as u32;

		self.charge(|schedule| schedule.wasm.mem_move as u64 * len as u64)?;

		self.memory().copy(src as usize, dst as usize, len as usize)?;

		Ok(Some(Into::into(dst as i32)))
	}

	fn mem_set(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		//
		// method signature:
		//   fn memset(dest: *const u8, c: u32, len: u32) -> *mut u8;
		//

		let len = context.value_stack.pop_as::<i32>()? as u32;
		let c = context.value_stack.pop_as::<i32>()? as u32;
		let dst = context.value_stack.pop_as::<i32>()? as u32;

		self.charge(|schedule| schedule.wasm.mem_set as u64 * len as u64)?;

		self.memory().clear(dst as usize, c as u8, len as usize)?;

		Ok(Some(Into::into(dst as i32)))
	}

	fn bitswap_i64(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let x = context.value_stack.pop_as::<i64>()?;
		let result = x.swap_bytes();

		Ok(Some(result.into()))
	}

	fn user_panic(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let msg_len = context.value_stack.pop_as::<i32>()? as u32;
		let msg_ptr = context.value_stack.pop_as::<i32>()? as u32;

		let msg = String::from_utf8(self.memory.get(msg_ptr, msg_len as usize)?)
			.map_err(|_| UserTrap::BadUtf8)?;

		trace!(target: "wasm", "Contract custom panic message: {}", msg);

		Err(UserTrap::Panic(msg).into())
	}

	fn block_hash(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let return_ptr = context.value_stack.pop_as::<i32>()? as u32;
		let block_num = context.value_stack.pop_as::<i64>()? as u64;

		self.charge(|schedule| schedule.blockhash_gas as u64)?;

		trace!("Requesting block hash for block #{}", block_num);
		let hash = self.ext.blockhash(&U256::from(block_num));

		self.memory.set(return_ptr, &*hash)?;

		Ok(Some(0i32.into()))
	}

	fn return_address_ptr(&mut self, ptr: u32, val: Address) -> Result<(), InterpreterError>
	{
		self.charge(|schedule| schedule.wasm.static_address as u64)?;
		self.memory.set(ptr, &*val)?;
		Ok(())
	}

	fn return_u256_ptr(&mut self, ptr: u32, val: U256) -> Result<(), InterpreterError> {
		let value: H256 = val.into();
		self.charge(|schedule| schedule.wasm.static_u256 as u64)?;
		self.memory.set(ptr, &*value)?;
		Ok(())
	}

	fn coinbase(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let author = self.ext.env_info().author;
		self.return_address_ptr(
			context.value_stack.pop_as::<i32>()? as u32,
			author,
		)?;
		Ok(None)
	}

	fn sender(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let sender = self.context.sender;
		self.return_address_ptr(
			context.value_stack.pop_as::<i32>()? as u32,
			sender,
		)?;
		Ok(None)
	}

	fn address(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let addr = self.context.address;
		self.return_address_ptr(
			context.value_stack.pop_as::<i32>()? as u32,
			addr,
		)?;
		Ok(None)
	}

	fn origin(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let origin = self.context.origin;
		self.return_address_ptr(
			context.value_stack.pop_as::<i32>()? as u32,
			origin,
		)?;
		Ok(None)
	}

	fn value(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let value = self.context.value;
		self.return_u256_ptr(
			context.value_stack.pop_as::<i32>()? as u32,
			value,
		)?;
		Ok(None)
	}

	fn timestamp(&mut self, _context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let timestamp = self.ext.env_info().timestamp as i64;
		Ok(Some(timestamp.into()))
	}

	fn block_number(&mut self, _context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let block_number = self.ext.env_info().number as i64;
		Ok(Some(block_number.into()))
	}

	fn difficulty(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let difficulty = self.ext.env_info().difficulty;
		self.return_u256_ptr(
			context.value_stack.pop_as::<i32>()? as u32,
			difficulty,
		)?;
		Ok(None)
	}

	fn ext_gas_limit(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		let gas_limit = self.ext.env_info().gas_limit;
		self.return_u256_ptr(
			context.value_stack.pop_as::<i32>()? as u32,
			gas_limit,
		)?;
		Ok(None)
	}

	pub fn execution_params(&mut self) -> interpreter::ExecutionParams<UserTrap> {
		use super::env;

		let env_instance = self.instance.module("env")
			.expect("Env module always exists; qed");

		interpreter::ExecutionParams::with_external(
			"env".into(),
			Arc::new(
				interpreter::env_native_module(env_instance, env::native_bindings(self))
					.expect("Env module always exists; qed")
			)
		)
	}

	pub fn gas_rules(&self) -> &rules::Set {
		&self.gas_rules
	}

	pub fn ext(&mut self) -> &mut vm::Ext {
		self.ext
	}

	pub fn log(&mut self, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		// signature is:
		// pub fn elog(topic_ptr: *const u8, topic_count: u32, data_ptr: *const u8, data_len: u32);
		let data_len = context.value_stack.pop_as::<i32>()? as u32;
		let data_ptr = context.value_stack.pop_as::<i32>()? as u32;
		let topic_count = context.value_stack.pop_as::<i32>()? as u32;
		let topic_ptr = context.value_stack.pop_as::<i32>()? as u32;

		if topic_count > 4 {
			return Err(UserTrap::Log.into());
		}

		self.overflow_charge(|schedule|
			{
				let topics_gas = schedule.log_gas as u64 + schedule.log_topic_gas as u64 * topic_count as u64;
				(schedule.log_data_gas as u64)
					.checked_mul(schedule.log_data_gas as u64)
					.and_then(|data_gas| data_gas.checked_add(topics_gas))
			}
		)?;

		let mut topics: Vec<H256> = Vec::with_capacity(topic_count as usize);
		topics.resize(topic_count as usize, H256::zero());
		for i in 0..topic_count {
			let offset = i.checked_mul(32).ok_or(UserTrap::MemoryAccessViolation)?
				.checked_add(topic_ptr).ok_or(UserTrap::MemoryAccessViolation)?;

			*topics.get_mut(i as usize)
				.expect("topics is resized to `topic_count`, i is in 0..topic count iterator, get_mut uses i as an indexer, get_mut cannot fail; qed")
				= H256::from(&self.memory.get(offset, 32)?[..]);
		}
		self.ext.log(topics, &self.memory.get(data_ptr, data_len as usize)?).map_err(|_| UserTrap::Log)?;

		Ok(None)
	}
}

impl<'a, 'b> interpreter::UserFunctionExecutor<UserTrap> for Runtime<'a, 'b> {
	fn execute(&mut self, name: &str, context: InterpreterCallerContext)
		-> Result<Option<interpreter::RuntimeValue>, InterpreterError>
	{
		match name {
			"_ext_malloc" => {
				self.malloc(context)
			},
			"_ext_free" => {
				// Since it is arena allocator, free does nothing
				// todo: update if changed
				self.user_noop(context)
			},
			"_storage_read" => {
				self.storage_read(context)
			},
			"_storage_write" => {
				self.storage_write(context)
			},
			"_balance" => {
				self.balance(context)
			},
			"_suicide" => {
				self.suicide(context)
			},
			"_create" => {
				self.create(context)
			},
			"_ccall" => {
				self.call(context)
			},
			"_dcall" => {
				self.call_code(context)
			},
			"_scall" => {
			 	self.static_call(context)
			},
			"_debug" => {
				self.debug_log(context)
			},
			"gas" => {
				self.gas(context)
			},
			"_emscripten_memcpy_big" => {
				self.mem_copy(context)
			},
			"_ext_memcpy" => {
				self.mem_copy(context)
			},
			"_ext_memmove" => {
				self.mem_move(context)
			},
			"_ext_memset" => {
				self.mem_set(context)
			},
			"_llvm_bswap_i64" => {
				self.bitswap_i64(context)
			},
			"_panic" => {
				self.user_panic(context)
			},
			"_blockhash" => {
				self.block_hash(context)
			},
			"_coinbase" => {
				self.coinbase(context)
			},
			"_timestamp" => {
				self.timestamp(context)
			},
			"_blocknumber" => {
				self.block_number(context)
			},
			"_difficulty" => {
				self.difficulty(context)
			},
			"_gaslimit" => {
				self.ext_gas_limit(context)
			},
			"_sender" => {
				self.sender(context)
			},
			"_address" => {
				self.address(context)
			},
			"_origin" => {
				self.origin(context)
			},
			"_value" => {
				self.value(context)
			},
			"_elog" => {
				self.log(context)
			},
			_ => {
				trace!(target: "wasm", "Trapped due to unhandled function: '{}'", name);
				Ok(self.unknown_trap(context)?)
			},
		}
	}
}
