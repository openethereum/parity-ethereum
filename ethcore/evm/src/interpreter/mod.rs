// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Rust VM implementation

#[macro_use]
mod informant;
mod gasometer;
mod stack;
mod memory;
mod shared_cache;

use std::marker::PhantomData;
use std::{cmp, mem};
use std::sync::Arc;
use std::convert::TryFrom;
use hash::keccak;
use bytes::Bytes;
use ethereum_types::{U256, U512, H256, Address, BigEndianHash};


use vm::{
	self, ActionParams, ParamsType, ActionValue, CallType, MessageCallResult,
	ContractCreateResult, CreateContractAddress, ReturnData, GasLeft, Schedule,
	TrapKind, TrapError
};

use evm::CostType;
use instructions::{self, Instruction, InstructionInfo};

use self::gasometer::Gasometer;
use self::stack::{Stack, VecStack};
use self::memory::Memory;
pub use self::shared_cache::SharedCache;

use bit_set::BitSet;

const GASOMETER_PROOF: &str = "If gasometer is None, Err is immediately returned in step; this function is only called by step; qed";

type ProgramCounter = usize;

const ONE: U256 = U256([1, 0, 0, 0]);
const TWO: U256 = U256([2, 0, 0, 0]);
const TWO_POW_5: U256 = U256([0x20, 0, 0, 0]);
const TWO_POW_8: U256 = U256([0x100, 0, 0, 0]);
const TWO_POW_16: U256 = U256([0x10000, 0, 0, 0]);
const TWO_POW_24: U256 = U256([0x1000000, 0, 0, 0]);
const TWO_POW_64: U256 = U256([0, 0x1, 0, 0]); // 0x1 00000000 00000000
const TWO_POW_96: U256 = U256([0, 0x100000000, 0, 0]); //0x1 00000000 00000000 00000000
const TWO_POW_224: U256 = U256([0, 0, 0, 0x100000000]); //0x1 00000000 00000000 00000000 00000000 00000000 00000000 00000000
const TWO_POW_248: U256 = U256([0, 0, 0, 0x100000000000000]); //0x1 00000000 00000000 00000000 00000000 00000000 00000000 00000000 000000

/// Abstraction over raw vector of Bytes. Easier state management of PC.
struct CodeReader {
	position: ProgramCounter,
	code: Arc<Bytes>,
}

impl CodeReader {
	/// Create new code reader - starting at position 0.
	fn new(code: Arc<Bytes>) -> Self {
		CodeReader {
			code,
			position: 0,
		}
	}

	/// Get `no_of_bytes` from code and convert to U256. Move PC
	fn read(&mut self, no_of_bytes: usize) -> U256 {
		let pos = self.position;
		self.position += no_of_bytes;
		let max = cmp::min(pos + no_of_bytes, self.code.len());
		U256::from(&self.code[pos..max])
	}

	fn len(&self) -> usize {
		self.code.len()
	}
}

enum InstructionResult<Gas> {
	Ok,
	UnusedGas(Gas),
	JumpToPosition(U256),
	StopExecutionNeedsReturn {
		/// Gas left.
		gas: Gas,
		/// Return data offset.
		init_off: U256,
		/// Return data size.
		init_size: U256,
		/// Apply or revert state changes.
		apply: bool,
	},
	StopExecution,
	Trap(TrapKind),
}

/// ActionParams without code, so that it can be feed into CodeReader.
#[derive(Debug)]
struct InterpreterParams {
	/// Address of currently executed code.
	pub code_address: Address,
	/// Hash of currently executed code.
	pub code_hash: Option<H256>,
	/// Code version.
	pub code_version: U256,
	/// Receive address. Usually equal to code_address,
	/// except when called using CALLCODE.
	pub address: Address,
	/// Sender of current part of the transaction.
	pub sender: Address,
	/// Transaction initiator.
	pub origin: Address,
	/// Gas paid up front for transaction execution
	pub gas: U256,
	/// Gas price.
	pub gas_price: U256,
	/// Transaction value.
	pub value: ActionValue,
	/// Input data.
	pub data: Option<Bytes>,
	/// Type of call
	pub call_type: CallType,
	/// Param types encoding
	pub params_type: ParamsType,
}

impl From<ActionParams> for InterpreterParams {
	fn from(params: ActionParams) -> Self {
		InterpreterParams {
			code_address: params.code_address,
			code_hash: params.code_hash,
			code_version: params.code_version,
			address: params.address,
			sender: params.sender,
			origin: params.origin,
			gas: params.gas,
			gas_price: params.gas_price,
			value: params.value,
			data: params.data,
			call_type: params.call_type,
			params_type: params.params_type,
		}
	}
}

/// Stepping result returned by interpreter.
pub enum InterpreterResult {
	/// The VM has already stopped.
	Stopped,
	/// The VM has just finished execution in the current step.
	Done(vm::Result<GasLeft>),
	/// The VM can continue to run.
	Continue,
	Trap(TrapKind),
}

/// Intepreter EVM implementation
pub struct Interpreter<Cost: CostType> {
	mem: Vec<u8>,
	cache: Arc<SharedCache>,
	params: InterpreterParams,
	reader: CodeReader,
	return_data: ReturnData,
	informant: informant::EvmInformant,
	do_trace: bool,
	done: bool,
	valid_jump_destinations: Option<Arc<BitSet>>,
	gasometer: Option<Gasometer<Cost>>,
	stack: VecStack<U256>,
	resume_output_range: Option<(U256, U256)>,
	resume_result: Option<InstructionResult<Cost>>,
	last_stack_ret_len: usize,
	_type: PhantomData<Cost>,
}

impl<Cost: 'static + CostType> vm::Exec for Interpreter<Cost> {
	fn exec(mut self: Box<Self>, ext: &mut dyn vm::Ext) -> vm::ExecTrapResult<GasLeft> {
		loop {
			let result = self.step(ext);
			match result {
				InterpreterResult::Continue => {},
				InterpreterResult::Done(value) => return Ok(value),
				InterpreterResult::Trap(trap) => match trap {
					TrapKind::Call(params) => {
						return Err(TrapError::Call(params, self));
					},
					TrapKind::Create(params, address) => {
						return Err(TrapError::Create(params, address, self));
					},
				},
				InterpreterResult::Stopped => panic!("Attempted to execute an already stopped VM.")
			}
		}
	}
}

impl<Cost: 'static + CostType> vm::ResumeCall for Interpreter<Cost> {
	fn resume_call(mut self: Box<Self>, result: MessageCallResult) -> Box<dyn vm::Exec> {
		{
			let this = &mut *self;
			let (out_off, out_size) = this.resume_output_range.take().expect("Box<ResumeCall> is obtained from a call opcode; resume_output_range is always set after those opcodes are executed; qed");

			match result {
				MessageCallResult::Success(gas_left, data) => {
					let output = this.mem.writeable_slice(out_off, out_size);
					let len = cmp::min(output.len(), data.len());
					(&mut output[..len]).copy_from_slice(&data[..len]);

					this.return_data = data;
					this.stack.push(U256::one());
					this.resume_result = Some(InstructionResult::UnusedGas(Cost::from_u256(gas_left).expect("Gas left cannot be greater than current one")));
				},
				MessageCallResult::Reverted(gas_left, data) => {
					let output = this.mem.writeable_slice(out_off, out_size);
					let len = cmp::min(output.len(), data.len());
					(&mut output[..len]).copy_from_slice(&data[..len]);

					this.return_data = data;
					this.stack.push(U256::zero());
					this.resume_result = Some(InstructionResult::UnusedGas(Cost::from_u256(gas_left).expect("Gas left cannot be greater than current one")));
				},
				MessageCallResult::Failed => {
					this.stack.push(U256::zero());
					this.resume_result = Some(InstructionResult::Ok);
				},
			}
		}
		self
	}
}

impl<Cost: 'static + CostType> vm::ResumeCreate for Interpreter<Cost> {
	fn resume_create(mut self: Box<Self>, result: ContractCreateResult) -> Box<dyn vm::Exec> {
		match result {
			ContractCreateResult::Created(address, gas_left) => {
				self.stack.push(address_to_u256(address));
				self.resume_result = Some(InstructionResult::UnusedGas(Cost::from_u256(gas_left).expect("Gas left cannot be greater.")));
			},
			ContractCreateResult::Reverted(gas_left, return_data) => {
				self.stack.push(U256::zero());
				self.return_data = return_data;
				self.resume_result = Some(InstructionResult::UnusedGas(Cost::from_u256(gas_left).expect("Gas left cannot be greater.")));
			},
			ContractCreateResult::Failed => {
				self.stack.push(U256::zero());
				self.resume_result = Some(InstructionResult::Ok);
			},
		}
		self
	}
}

impl<Cost: CostType> Interpreter<Cost> {
	/// Create a new `Interpreter` instance with shared cache.
	pub fn new(mut params: ActionParams, cache: Arc<SharedCache>, schedule: &Schedule, depth: usize) -> Interpreter<Cost> {
		let reader = CodeReader::new(params.code.take().expect("VM always called with code; qed"));
		let params = InterpreterParams::from(params);
		let informant = informant::EvmInformant::new(depth);
		let valid_jump_destinations = None;
		let gasometer = Cost::from_u256(params.gas).ok().map(|gas| Gasometer::<Cost>::new(gas));
		let stack = VecStack::with_capacity(schedule.stack_limit, U256::zero());

		Interpreter {
			cache, params, reader, informant,
			valid_jump_destinations, gasometer, stack,
			done: false,
			// Overridden in `step_inner` based on
			// the result of `ext.trace_next_instruction`.
			do_trace: true,
			mem: Vec::new(),
			return_data: ReturnData::empty(),
			last_stack_ret_len: 0,
			resume_output_range: None,
			resume_result: None,
			_type: PhantomData,
		}
	}

	/// Execute a single step on the VM.
	#[inline(always)]
	pub fn step(&mut self, ext: &mut dyn vm::Ext) -> InterpreterResult {
		if self.done {
			return InterpreterResult::Stopped;
		}

		let result = if self.gasometer.is_none() {
			InterpreterResult::Done(Err(vm::Error::OutOfGas))
		} else if self.reader.len() == 0 {
			let current_gas = self.gasometer
				.as_ref()
				.expect("Gasometer None case is checked above; qed")
				.current_gas
				.as_u256();
			InterpreterResult::Done(Ok(GasLeft::Known(current_gas)))
		} else {
			self.step_inner(ext)
		};

		if let &InterpreterResult::Done(_) = &result {
			self.done = true;
			self.informant.done();
		}
		result
	}

	/// Inner helper function for step.
	#[inline(always)]
	fn step_inner(&mut self, ext: &mut dyn vm::Ext) -> InterpreterResult {
		let result = match self.resume_result.take() {
			Some(result) => result,
			None => {
				let opcode = self.reader.code[self.reader.position];
				let instruction = Instruction::from_u8(opcode);
				self.reader.position += 1;

				// TODO: make compile-time removable if too much of a performance hit.
				self.do_trace = self.do_trace && ext.trace_next_instruction(
					self.reader.position - 1, opcode, self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas.as_u256(),
				);

				let instruction = match instruction {
					Some(i) => i,
					None => return InterpreterResult::Done(Err(vm::Error::BadInstruction {
						instruction: opcode
					})),
				};

				let info = instruction.info();
				self.last_stack_ret_len = info.ret;
				if let Err(e) = self.verify_instruction(ext, instruction, info) {
					return InterpreterResult::Done(Err(e));
				};

				// Calculate gas cost
				let requirements = match self.gasometer.as_mut().expect(GASOMETER_PROOF).requirements(ext, instruction, info, &self.stack, self.mem.size()) {
					Ok(t) => t,
					Err(e) => return InterpreterResult::Done(Err(e)),
				};
				if self.do_trace {
					ext.trace_prepare_execute(self.reader.position - 1, opcode, requirements.gas_cost.as_u256(), Self::mem_written(instruction, &self.stack), Self::store_written(instruction, &self.stack));
				}
				if let Err(e) = self.gasometer.as_mut().expect(GASOMETER_PROOF).verify_gas(&requirements.gas_cost) {
					if self.do_trace {
						ext.trace_failed();
					}
					return InterpreterResult::Done(Err(e));
				}
				self.mem.expand(requirements.memory_required_size);
				self.gasometer.as_mut().expect(GASOMETER_PROOF).current_mem_gas = requirements.memory_total_gas;
				self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas = self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas - requirements.gas_cost;

				evm_debug!({ self.informant.before_instruction(self.reader.position, instruction, info, &self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas, &self.stack) });

				// Execute instruction
				let current_gas = self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas;
				let result = match self.exec_instruction(
					current_gas, ext, instruction, requirements.provide_gas
				) {
					Err(x) => {
						if self.do_trace {
							ext.trace_failed();
						}
						return InterpreterResult::Done(Err(x));
					},
					Ok(x) => x,
				};
				evm_debug!({ self.informant.after_instruction(instruction) });
				result
			},
		};

		if let InstructionResult::Trap(trap) = result {
			return InterpreterResult::Trap(trap);
		}

		if let InstructionResult::UnusedGas(ref gas) = result {
			self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas = self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas + *gas;
		}

		if self.do_trace {
			ext.trace_executed(
				self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas.as_u256(),
				self.stack.peek_top(self.last_stack_ret_len),
				&self.mem,
			);
		}

		// Advance
		match result {
			InstructionResult::JumpToPosition(position) => {
				if self.valid_jump_destinations.is_none() {
					self.valid_jump_destinations = Some(self.cache.jump_destinations(&self.params.code_hash, &self.reader.code));
				}
				let jump_destinations = self.valid_jump_destinations.as_ref().expect("jump_destinations are initialized on first jump; qed");
				let pos = match self.verify_jump(position, jump_destinations) {
					Ok(x) => x,
					Err(e) => return InterpreterResult::Done(Err(e))
				};
				self.reader.position = pos;
			},
			InstructionResult::StopExecutionNeedsReturn {gas, init_off, init_size, apply} => {
				let mem = mem::replace(&mut self.mem, Vec::new());
				return InterpreterResult::Done(Ok(GasLeft::NeedsReturn {
					gas_left: gas.as_u256(),
					data: mem.into_return_data(init_off, init_size),
					apply_state: apply
				}));
			},
			InstructionResult::StopExecution => {
				return InterpreterResult::Done(Ok(GasLeft::Known(self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas.as_u256())));
			},
			_ => {},
		}

		if self.reader.position >= self.reader.len() {
			return InterpreterResult::Done(Ok(GasLeft::Known(self.gasometer.as_mut().expect(GASOMETER_PROOF).current_gas.as_u256())));
		}

		InterpreterResult::Continue
	}

	fn verify_instruction(&self, ext: &dyn vm::Ext, instruction: Instruction, info: &InstructionInfo) -> vm::Result<()> {
		let schedule = ext.schedule();

		if (instruction == instructions::DELEGATECALL && !schedule.have_delegate_call) ||
			(instruction == instructions::CREATE2 && !schedule.have_create2) ||
			(instruction == instructions::STATICCALL && !schedule.have_static_call) ||
			((instruction == instructions::RETURNDATACOPY || instruction == instructions::RETURNDATASIZE) && !schedule.have_return_data) ||
			(instruction == instructions::REVERT && !schedule.have_revert) ||
			((instruction == instructions::SHL || instruction == instructions::SHR || instruction == instructions::SAR) && !schedule.have_bitwise_shifting) ||
			(instruction == instructions::EXTCODEHASH && !schedule.have_extcodehash) ||
			(instruction == instructions::CHAINID && !schedule.have_chain_id) ||
			(instruction == instructions::SELFBALANCE && !schedule.have_selfbalance)
		{
			return Err(vm::Error::BadInstruction {
				instruction: instruction as u8
			});
		}

		if !self.stack.has(info.args) {
			Err(vm::Error::StackUnderflow {
				instruction: info.name,
				wanted: info.args,
				on_stack: self.stack.size()
			})
		} else if self.stack.size() - info.args + info.ret > schedule.stack_limit {
			Err(vm::Error::OutOfStack {
				instruction: info.name,
				wanted: info.ret - info.args,
				limit: schedule.stack_limit
			})
		} else {
			Ok(())
		}
	}

	fn mem_written(
		instruction: Instruction,
		stack: &dyn Stack<U256>
	) -> Option<(usize, usize)> {
		let read = |pos| stack.peek(pos).low_u64() as usize;
		let written = match instruction {
			instructions::MSTORE | instructions::MLOAD => Some((read(0), 32)),
			instructions::MSTORE8 => Some((read(0), 1)),
			instructions::CALLDATACOPY | instructions::CODECOPY | instructions::RETURNDATACOPY => Some((read(0), read(2))),
			instructions::EXTCODECOPY => Some((read(1), read(3))),
			instructions::CALL | instructions::CALLCODE => Some((read(5), read(6))),
			instructions::DELEGATECALL | instructions::STATICCALL => Some((read(4), read(5))),
			_ => None,
		};

		match written {
			Some((offset, size)) if !memory::is_valid_range(offset, size) => None,
			written => written,
		}
	}

	fn store_written(
		instruction: Instruction,
		stack: &dyn Stack<U256>
	) -> Option<(U256, U256)> {
		match instruction {
			instructions::SSTORE => Some((stack.peek(0).clone(), stack.peek(1).clone())),
			_ => None,
		}
	}

	fn exec_instruction(
		&mut self,
		gas: Cost,
		ext: &mut dyn vm::Ext,
		instruction: Instruction,
		provided: Option<Cost>
	) -> vm::Result<InstructionResult<Cost>> {
		match instruction {
			instructions::JUMP => {
				let jump = self.stack.pop_back();
				return Ok(InstructionResult::JumpToPosition(
					jump
				));
			},
			instructions::JUMPI => {
				let jump = self.stack.pop_back();
				let condition = self.stack.pop_back();
				if !condition.is_zero() {
					return Ok(InstructionResult::JumpToPosition(
						jump
					));
				}
			},
			instructions::JUMPDEST => {
				// ignore
			},
			instructions::CREATE | instructions::CREATE2 => {
				let endowment = self.stack.pop_back();
				let init_off = self.stack.pop_back();
				let init_size = self.stack.pop_back();
				let address_scheme = match instruction {
					instructions::CREATE => CreateContractAddress::FromSenderAndNonce,
					instructions::CREATE2 => CreateContractAddress::FromSenderSaltAndCodeHash(BigEndianHash::from_uint(&self.stack.pop_back())),
					_ => unreachable!("instruction can only be CREATE/CREATE2 checked above; qed"),
				};

				let create_gas = provided.expect("`provided` comes through Self::exec from `Gasometer::get_gas_cost_mem`; `gas_gas_mem_cost` guarantees `Some` when instruction is `CALL`/`CALLCODE`/`DELEGATECALL`/`CREATE`; this is `CREATE`; qed");

				if ext.is_static() {
					return Err(vm::Error::MutableCallInStaticContext);
				}

				// clear return data buffer before creating new call frame.
				self.return_data = ReturnData::empty();

				let can_create = ext.balance(&self.params.address)? >= endowment && ext.depth() < ext.schedule().max_depth;
				if !can_create {
					self.stack.push(U256::zero());
					return Ok(InstructionResult::UnusedGas(create_gas));
				}

				let contract_code = self.mem.read_slice(init_off, init_size);

				let create_result = ext.create(&create_gas.as_u256(), &endowment, contract_code, &self.params.code_version, address_scheme, true);
				return match create_result {
					Ok(ContractCreateResult::Created(address, gas_left)) => {
						self.stack.push(address_to_u256(address));
						Ok(InstructionResult::UnusedGas(Cost::from_u256(gas_left).expect("Gas left cannot be greater.")))
					},
					Ok(ContractCreateResult::Reverted(gas_left, return_data)) => {
						self.stack.push(U256::zero());
						self.return_data = return_data;
						Ok(InstructionResult::UnusedGas(Cost::from_u256(gas_left).expect("Gas left cannot be greater.")))
					},
					Ok(ContractCreateResult::Failed) => {
						self.stack.push(U256::zero());
						Ok(InstructionResult::Ok)
					},
					Err(trap) => {
						Ok(InstructionResult::Trap(trap))
					},
				};
			},
			instructions::CALL | instructions::CALLCODE | instructions::DELEGATECALL | instructions::STATICCALL => {
				assert!(ext.schedule().call_value_transfer_gas > ext.schedule().call_stipend, "overflow possible");

				self.stack.pop_back();
				let call_gas = provided.expect("`provided` comes through Self::exec from `Gasometer::get_gas_cost_mem`; `gas_gas_mem_cost` guarantees `Some` when instruction is `CALL`/`CALLCODE`/`DELEGATECALL`/`CREATE`; this is one of `CALL`/`CALLCODE`/`DELEGATECALL`; qed");
				let code_address = self.stack.pop_back();
				let code_address = u256_to_address(&code_address);

				let value = if instruction == instructions::DELEGATECALL {
					None
				} else if instruction == instructions::STATICCALL {
					Some(U256::zero())
				} else {
					Some(self.stack.pop_back())
				};

				let in_off = self.stack.pop_back();
				let in_size = self.stack.pop_back();
				let out_off = self.stack.pop_back();
				let out_size = self.stack.pop_back();

				// Add stipend (only CALL|CALLCODE when value > 0)
				let call_gas = call_gas.overflow_add(value.map_or_else(|| Cost::from(0), |val| match val.is_zero() {
					false => Cost::from(ext.schedule().call_stipend),
					true => Cost::from(0),
				})).0;

				// Get sender & receive addresses, check if we have balance
				let (sender_address, receive_address, has_balance, call_type) = match instruction {
					instructions::CALL => {
						if ext.is_static() && value.map_or(false, |v| !v.is_zero()) {
							return Err(vm::Error::MutableCallInStaticContext);
						}
						let has_balance = ext.balance(&self.params.address)? >= value.expect("value set for all but delegate call; qed");
						(&self.params.address, &code_address, has_balance, CallType::Call)
					},
					instructions::CALLCODE => {
						let has_balance = ext.balance(&self.params.address)? >= value.expect("value set for all but delegate call; qed");
						(&self.params.address, &self.params.address, has_balance, CallType::CallCode)
					},
					instructions::DELEGATECALL => (&self.params.sender, &self.params.address, true, CallType::DelegateCall),
					instructions::STATICCALL => (&self.params.address, &code_address, true, CallType::StaticCall),
					_ => panic!(format!("Unexpected instruction {:?} in CALL branch.", instruction))
				};

				// clear return data buffer before creating new call frame.
				self.return_data = ReturnData::empty();

				let can_call = has_balance && ext.depth() < ext.schedule().max_depth;
				if !can_call {
					self.stack.push(U256::zero());
					return Ok(InstructionResult::UnusedGas(call_gas));
				}

				let call_result = {
					let input = self.mem.read_slice(in_off, in_size);
					ext.call(&call_gas.as_u256(), sender_address, receive_address, value, input, &code_address, call_type, true)
				};

				self.resume_output_range = Some((out_off, out_size));

				return match call_result {
					Ok(MessageCallResult::Success(gas_left, data)) => {
						let output = self.mem.writeable_slice(out_off, out_size);
						let len = cmp::min(output.len(), data.len());
						(&mut output[..len]).copy_from_slice(&data[..len]);

						self.stack.push(U256::one());
						self.return_data = data;
						Ok(InstructionResult::UnusedGas(Cost::from_u256(gas_left).expect("Gas left cannot be greater than current one")))
					},
					Ok(MessageCallResult::Reverted(gas_left, data)) => {
						let output = self.mem.writeable_slice(out_off, out_size);
						let len = cmp::min(output.len(), data.len());
						(&mut output[..len]).copy_from_slice(&data[..len]);

						self.stack.push(U256::zero());
						self.return_data = data;
						Ok(InstructionResult::UnusedGas(Cost::from_u256(gas_left).expect("Gas left cannot be greater than current one")))
					},
					Ok(MessageCallResult::Failed) => {
						self.stack.push(U256::zero());
						Ok(InstructionResult::Ok)
					},
					Err(trap) => {
						Ok(InstructionResult::Trap(trap))
					},
				};
			},
			instructions::RETURN => {
				let init_off = self.stack.pop_back();
				let init_size = self.stack.pop_back();

				return Ok(InstructionResult::StopExecutionNeedsReturn {gas: gas, init_off: init_off, init_size: init_size, apply: true})
			},
			instructions::REVERT => {
				let init_off = self.stack.pop_back();
				let init_size = self.stack.pop_back();

				return Ok(InstructionResult::StopExecutionNeedsReturn {gas: gas, init_off: init_off, init_size: init_size, apply: false})
			},
			instructions::STOP => {
				return Ok(InstructionResult::StopExecution);
			},
			instructions::SUICIDE => {
				let address = self.stack.pop_back();
				ext.suicide(&u256_to_address(&address))?;
				return Ok(InstructionResult::StopExecution);
			},
			instructions::LOG0 | instructions::LOG1 | instructions::LOG2 | instructions::LOG3 | instructions::LOG4 => {
				let no_of_topics = instruction.log_topics().expect("log_topics always return some for LOG* instructions; qed");

				let offset = self.stack.pop_back();
				let size = self.stack.pop_back();
				let topics = self.stack.pop_n(no_of_topics)
					.iter()
					.map(BigEndianHash::from_uint)
					.collect();
				ext.log(topics, self.mem.read_slice(offset, size))?;
			},
			instructions::PUSH1 | instructions::PUSH2 | instructions::PUSH3 | instructions::PUSH4 |
			instructions::PUSH5 | instructions::PUSH6 | instructions::PUSH7 | instructions::PUSH8 |
			instructions::PUSH9 | instructions::PUSH10 | instructions::PUSH11 | instructions::PUSH12 |
			instructions::PUSH13 | instructions::PUSH14 | instructions::PUSH15 | instructions::PUSH16 |
			instructions::PUSH17 | instructions::PUSH18 | instructions::PUSH19 | instructions::PUSH20 |
			instructions::PUSH21 | instructions::PUSH22 | instructions::PUSH23 | instructions::PUSH24 |
			instructions::PUSH25 | instructions::PUSH26 | instructions::PUSH27 | instructions::PUSH28 |
			instructions::PUSH29 | instructions::PUSH30 | instructions::PUSH31 | instructions::PUSH32 => {
				let bytes = instruction.push_bytes().expect("push_bytes always return some for PUSH* instructions");
				let val = self.reader.read(bytes);
				self.stack.push(val);
			},
			instructions::MLOAD => {
				let word = self.mem.read(self.stack.pop_back());
				self.stack.push(U256::from(word));
			},
			instructions::MSTORE => {
				let offset = self.stack.pop_back();
				let word = self.stack.pop_back();
				Memory::write(&mut self.mem, offset, word);
			},
			instructions::MSTORE8 => {
				let offset = self.stack.pop_back();
				let byte = self.stack.pop_back();
				self.mem.write_byte(offset, byte);
			},
			instructions::MSIZE => {
				self.stack.push(U256::from(self.mem.size()));
			},
			instructions::SHA3 => {
				let offset = self.stack.pop_back();
				let size = self.stack.pop_back();
				let k = keccak(self.mem.read_slice(offset, size));
				self.stack.push(k.into_uint());
			},
			instructions::SLOAD => {
				let key = BigEndianHash::from_uint(&self.stack.pop_back());
				let word = ext.storage_at(&key)?.into_uint();
				self.stack.push(word);
			},
			instructions::SSTORE => {
				let address = BigEndianHash::from_uint(&self.stack.pop_back());
				let val = self.stack.pop_back();

				let current_val = ext.storage_at(&address)?.into_uint();
				// Increase refund for clear
				if ext.schedule().eip1283 {
					let original_val = ext.initial_storage_at(&address)?.into_uint();
					gasometer::handle_eip1283_sstore_clears_refund(ext, &original_val, &current_val, &val);
				} else {
					if !current_val.is_zero() && val.is_zero() {
						let sstore_clears_schedule = ext.schedule().sstore_refund_gas;
						ext.add_sstore_refund(sstore_clears_schedule);
					}
				}
				ext.set_storage(address, BigEndianHash::from_uint(&val))?;
			},
			instructions::PC => {
				self.stack.push(U256::from(self.reader.position - 1));
			},
			instructions::GAS => {
				self.stack.push(gas.as_u256());
			},
			instructions::ADDRESS => {
				self.stack.push(address_to_u256(self.params.address.clone()));
			},
			instructions::ORIGIN => {
				self.stack.push(address_to_u256(self.params.origin.clone()));
			},
			instructions::BALANCE => {
				let address = u256_to_address(&self.stack.pop_back());
				let balance = ext.balance(&address)?;
				self.stack.push(balance);
			},
			instructions::CALLER => {
				self.stack.push(address_to_u256(self.params.sender.clone()));
			},
			instructions::CALLVALUE => {
				self.stack.push(match self.params.value {
					ActionValue::Transfer(val) | ActionValue::Apparent(val) => val
				});
			},
			instructions::CALLDATALOAD => {
				let big_id = self.stack.pop_back();
				let id = big_id.low_u64() as usize;
				let max = id.wrapping_add(32);
				if let Some(data) = self.params.data.as_ref() {
					let bound = cmp::min(data.len(), max);
					if id < bound && big_id < U256::from(data.len()) {
						let mut v = [0u8; 32];
						v[0..bound-id].clone_from_slice(&data[id..bound]);
						self.stack.push(U256::from(&v[..]))
					} else {
						self.stack.push(U256::zero())
					}
				} else {
					self.stack.push(U256::zero())
				}
			},
			instructions::CALLDATASIZE => {
				self.stack.push(U256::from(self.params.data.as_ref().map_or(0, |l| l.len())));
			},
			instructions::CODESIZE => {
				self.stack.push(U256::from(self.reader.len()));
			},
			instructions::RETURNDATASIZE => {
				self.stack.push(U256::from(self.return_data.len()))
			},
			instructions::EXTCODESIZE => {
				let address = u256_to_address(&self.stack.pop_back());
				let len = ext.extcodesize(&address)?.unwrap_or(0);
				self.stack.push(U256::from(len));
			},
			instructions::EXTCODEHASH => {
				let address = u256_to_address(&self.stack.pop_back());
				let hash = ext.extcodehash(&address)?.unwrap_or_else(H256::zero);
				self.stack.push(hash.into_uint());
			},
			instructions::CALLDATACOPY => {
				Self::copy_data_to_memory(&mut self.mem, &mut self.stack, &self.params.data.as_ref().map_or_else(|| &[] as &[u8], |d| &*d as &[u8]));
			},
			instructions::RETURNDATACOPY => {
				{
					let source_offset = self.stack.peek(1);
					let size = self.stack.peek(2);
					let return_data_len = U256::from(self.return_data.len());
					if source_offset.saturating_add(*size) > return_data_len {
						return Err(vm::Error::OutOfBounds);
					}
				}
				Self::copy_data_to_memory(&mut self.mem, &mut self.stack, &*self.return_data);
			},
			instructions::CODECOPY => {
				Self::copy_data_to_memory(&mut self.mem, &mut self.stack, &self.reader.code);
			},
			instructions::EXTCODECOPY => {
				let address = u256_to_address(&self.stack.pop_back());
				let code = ext.extcode(&address)?;
				Self::copy_data_to_memory(
					&mut self.mem,
					&mut self.stack,
					code.as_ref().map(|c| &(*c)[..]).unwrap_or(&[])
				);
			},
			instructions::GASPRICE => {
				self.stack.push(self.params.gas_price.clone());
			},
			instructions::BLOCKHASH => {
				let block_number = self.stack.pop_back();
				let block_hash = ext.blockhash(&block_number);
				self.stack.push(block_hash.into_uint());
			},
			instructions::COINBASE => {
				self.stack.push(address_to_u256(ext.env_info().author.clone()));
			},
			instructions::TIMESTAMP => {
				self.stack.push(U256::from(ext.env_info().timestamp));
			},
			instructions::NUMBER => {
				self.stack.push(U256::from(ext.env_info().number));
			},
			instructions::DIFFICULTY => {
				self.stack.push(ext.env_info().difficulty.clone());
			},
			instructions::GASLIMIT => {
				self.stack.push(ext.env_info().gas_limit.clone());
			},
			instructions::CHAINID => {
				self.stack.push(ext.chain_id().into())
			},
			instructions::SELFBALANCE => {
				self.stack.push(ext.balance(&self.params.address)?);
			}

			// Stack instructions

			instructions::DUP1 | instructions::DUP2 | instructions::DUP3 | instructions::DUP4 |
			instructions::DUP5 | instructions::DUP6 | instructions::DUP7 | instructions::DUP8 |
			instructions::DUP9 | instructions::DUP10 | instructions::DUP11 | instructions::DUP12 |
			instructions::DUP13 | instructions::DUP14 | instructions::DUP15 | instructions::DUP16 => {
				let position = instruction.dup_position().expect("dup_position always return some for DUP* instructions");
				let val = self.stack.peek(position).clone();
				self.stack.push(val);
			},
			instructions::SWAP1 | instructions::SWAP2 | instructions::SWAP3 | instructions::SWAP4 |
			instructions::SWAP5 | instructions::SWAP6 | instructions::SWAP7 | instructions::SWAP8 |
			instructions::SWAP9 | instructions::SWAP10 | instructions::SWAP11 | instructions::SWAP12 |
			instructions::SWAP13 | instructions::SWAP14 | instructions::SWAP15 | instructions::SWAP16 => {
				let position = instruction.swap_position().expect("swap_position always return some for SWAP* instructions");
				self.stack.swap_with_top(position)
			},
			instructions::POP => {
				self.stack.pop_back();
			},
			instructions::ADD => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(a.overflowing_add(b).0);
			},
			instructions::MUL => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(a.overflowing_mul(b).0);
			},
			instructions::SUB => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(a.overflowing_sub(b).0);
			},
			instructions::DIV => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(if !b.is_zero() {
					match b {
						ONE => a,
						TWO => a >> 1,
						TWO_POW_5 => a >> 5,
						TWO_POW_8 => a >> 8,
						TWO_POW_16 => a >> 16,
						TWO_POW_24 => a >> 24,
						TWO_POW_64 => a >> 64,
						TWO_POW_96 => a >> 96,
						TWO_POW_224 => a >> 224,
						TWO_POW_248 => a >> 248,
						_ => a / b,
					}
				} else {
					U256::zero()
				});
			},
			instructions::MOD => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(if !b.is_zero() {
					a % b
				} else {
					U256::zero()
				});
			},
			instructions::SDIV => {
				let (a, sign_a) = get_and_reset_sign(self.stack.pop_back());
				let (b, sign_b) = get_and_reset_sign(self.stack.pop_back());

				// -2^255
				let min = (U256::one() << 255) - U256::one();
				self.stack.push(if b.is_zero() {
					U256::zero()
				} else if a == min && b == !U256::zero() {
					min
				} else {
					let c = a / b;
					set_sign(c, sign_a ^ sign_b)
				});
			},
			instructions::SMOD => {
				let ua = self.stack.pop_back();
				let ub = self.stack.pop_back();
				let (a, sign_a) = get_and_reset_sign(ua);
				let b = get_and_reset_sign(ub).0;

				self.stack.push(if !b.is_zero() {
					let c = a % b;
					set_sign(c, sign_a)
				} else {
					U256::zero()
				});
			},
			instructions::EXP => {
				let base = self.stack.pop_back();
				let expon = self.stack.pop_back();
				let res = base.overflowing_pow(expon).0;
				self.stack.push(res);
			},
			instructions::NOT => {
				let a = self.stack.pop_back();
				self.stack.push(!a);
			},
			instructions::LT => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(Self::bool_to_u256(a < b));
			},
			instructions::SLT => {
				let (a, neg_a) = get_and_reset_sign(self.stack.pop_back());
				let (b, neg_b) = get_and_reset_sign(self.stack.pop_back());

				let is_positive_lt = a < b && !(neg_a | neg_b);
				let is_negative_lt = a > b && (neg_a & neg_b);
				let has_different_signs = neg_a && !neg_b;

				self.stack.push(Self::bool_to_u256(is_positive_lt | is_negative_lt | has_different_signs));
			},
			instructions::GT => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(Self::bool_to_u256(a > b));
			},
			instructions::SGT => {
				let (a, neg_a) = get_and_reset_sign(self.stack.pop_back());
				let (b, neg_b) = get_and_reset_sign(self.stack.pop_back());

				let is_positive_gt = a > b && !(neg_a | neg_b);
				let is_negative_gt = a < b && (neg_a & neg_b);
				let has_different_signs = !neg_a && neg_b;

				self.stack.push(Self::bool_to_u256(is_positive_gt | is_negative_gt | has_different_signs));
			},
			instructions::EQ => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(Self::bool_to_u256(a == b));
			},
			instructions::ISZERO => {
				let a = self.stack.pop_back();
				self.stack.push(Self::bool_to_u256(a.is_zero()));
			},
			instructions::AND => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(a & b);
			},
			instructions::OR => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(a | b);
			},
			instructions::XOR => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				self.stack.push(a ^ b);
			},
			instructions::BYTE => {
				let word = self.stack.pop_back();
				let val = self.stack.pop_back();
				let byte = match word < U256::from(32) {
					true => (val >> (8 * (31 - word.low_u64() as usize))) & U256::from(0xff),
					false => U256::zero()
				};
				self.stack.push(byte);
			},
			instructions::ADDMOD => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				let c = self.stack.pop_back();

				self.stack.push(if !c.is_zero() {
					let a_512 = U512::from(a);
					let b_512 = U512::from(b);
					let c_512 = U512::from(c);
					let res = a_512 + b_512;
					let x = res % c_512;
					U256::try_from(x).expect("U512 % U256 fits U256; qed")
				} else {
					U256::zero()
				});
			},
			instructions::MULMOD => {
				let a = self.stack.pop_back();
				let b = self.stack.pop_back();
				let c = self.stack.pop_back();

				self.stack.push(if !c.is_zero() {
					let a_512 = U512::from(a);
					let b_512 = U512::from(b);
					let c_512 = U512::from(c);
					let res = a_512 * b_512;
					let x = res % c_512;
					U256::try_from(x).expect("U512 % U256 fits U256; qed")
				} else {
					U256::zero()
				});
			},
			instructions::SIGNEXTEND => {
				let bit = self.stack.pop_back();
				if bit < U256::from(32) {
					let number = self.stack.pop_back();
					let bit_position = (bit.low_u64() * 8 + 7) as usize;

					let bit = number.bit(bit_position);
					let mask = (U256::one() << bit_position) - U256::one();
					self.stack.push(if bit {
						number | !mask
					} else {
						number & mask
					});
				}
			},
			instructions::SHL => {
				const CONST_256: U256 = U256([256, 0, 0, 0]);

				let shift = self.stack.pop_back();
				let value = self.stack.pop_back();

				let result = if shift >= CONST_256 {
					U256::zero()
				} else {
					value << (shift.as_u32() as usize)
				};
				self.stack.push(result);
			},
			instructions::SHR => {
				const CONST_256: U256 = U256([256, 0, 0, 0]);

				let shift = self.stack.pop_back();
				let value = self.stack.pop_back();

				let result = if shift >= CONST_256 {
					U256::zero()
				} else {
					value >> (shift.as_u32() as usize)
				};
				self.stack.push(result);
			},
			instructions::SAR => {
				// We cannot use get_and_reset_sign/set_sign here, because the rounding looks different.

				const CONST_256: U256 = U256([256, 0, 0, 0]);
				const CONST_HIBIT: U256 = U256([0, 0, 0, 0x8000000000000000]);

				let shift = self.stack.pop_back();
				let value = self.stack.pop_back();
				let sign = value & CONST_HIBIT != U256::zero();

				let result = if shift >= CONST_256 {
					if sign {
						U256::max_value()
					} else {
						U256::zero()
					}
				} else {
					let shift = shift.as_u32() as usize;
					let mut shifted = value >> shift;
					if sign {
						shifted = shifted | (U256::max_value() << (256 - shift));
					}
					shifted
				};
				self.stack.push(result);
			},
		};
		Ok(InstructionResult::Ok)
	}

	fn copy_data_to_memory(mem: &mut Vec<u8>, stack: &mut dyn Stack<U256>, source: &[u8]) {
		let dest_offset = stack.pop_back();
		let source_offset = stack.pop_back();
		let size = stack.pop_back();
		let source_size = U256::from(source.len());

		let output_end = match source_offset > source_size || size > source_size || source_offset + size > source_size {
			true => {
				let zero_slice = if source_offset > source_size {
					mem.writeable_slice(dest_offset, size)
				} else {
					mem.writeable_slice(dest_offset + source_size - source_offset, source_offset + size - source_size)
				};
				for i in zero_slice.iter_mut() {
					*i = 0;
				}
				source.len()
			},
			false => (size.low_u64() + source_offset.low_u64()) as usize
		};

		if source_offset < source_size {
			let output_begin = source_offset.low_u64() as usize;
			mem.write_slice(dest_offset, &source[output_begin..output_end]);
		}
	}

	fn verify_jump(&self, jump_u: U256, valid_jump_destinations: &BitSet) -> vm::Result<usize> {
		let jump = jump_u.low_u64() as usize;

		if valid_jump_destinations.contains(jump) && U256::from(jump) == jump_u {
			Ok(jump)
		} else {
			Err(vm::Error::BadJumpDestination {
				destination: jump
			})
		}
	}

	fn bool_to_u256(val: bool) -> U256 {
		if val {
			U256::one()
		} else {
			U256::zero()
		}
	}
}

fn get_and_reset_sign(value: U256) -> (U256, bool) {
	let U256(arr) = value;
	let sign = arr[3].leading_zeros() == 0;
	(set_sign(value, sign), sign)
}

fn set_sign(value: U256, sign: bool) -> U256 {
	if sign {
		(!U256::zero() ^ value).overflowing_add(U256::one()).0
	} else {
		value
	}
}

#[inline]
fn u256_to_address(value: &U256) -> Address {
	let addr: H256 = BigEndianHash::from_uint(value);
	Address::from(addr)
}

#[inline]
fn address_to_u256(value: Address) -> U256 {
	H256::from(value).into_uint()
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use rustc_hex::FromHex;
	use factory::Factory;
	use vm::{self, Exec, ActionParams, ActionValue};
	use vm::tests::{FakeExt, test_finalize};
	use ethereum_types::Address;

	fn interpreter(params: ActionParams, ext: &dyn vm::Ext) -> Box<dyn Exec> {
		Factory::new(1).create(params, ext.schedule(), ext.depth())
	}

	#[test]
	fn should_not_fail_on_tracing_mem() {
		let code = "7feeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff006000527faaffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffaa6020526000620f120660406000601773945304eb96065b2a98b57a48a06ae28d285a71b56101f4f1600055".from_hex().unwrap();

		let mut params = ActionParams::default();
		params.address = Address::from_low_u64_be(5);
		params.gas = 300_000.into();
		params.gas_price = 1.into();
		params.value = ActionValue::Transfer(100_000.into());
		params.code = Some(Arc::new(code));
		let mut ext = FakeExt::new();
		ext.balances.insert(Address::from_low_u64_be(5), 1_000_000_000.into());
		ext.tracing = true;

		let gas_left = {
			let vm = interpreter(params, &ext);
			test_finalize(vm.exec(&mut ext).ok().unwrap()).unwrap()
		};

		assert_eq!(ext.calls.len(), 1);
		assert_eq!(gas_left, 248_212.into());
	}

	#[test]
	fn should_not_overflow_returndata() {
		let code = "6001600160000360003e00".from_hex().unwrap();

		let mut params = ActionParams::default();
		params.address = Address::from_low_u64_be(5);
		params.gas = 300_000.into();
		params.gas_price = 1.into();
		params.code = Some(Arc::new(code));
		let mut ext = FakeExt::new_byzantium();
		ext.balances.insert(Address::from_low_u64_be(5), 1_000_000_000.into());
		ext.tracing = true;

		let err = {
			let vm = interpreter(params, &ext);
			test_finalize(vm.exec(&mut ext).ok().unwrap()).err().unwrap()
		};

		assert_eq!(err, ::vm::Error::OutOfBounds);
	}
}
