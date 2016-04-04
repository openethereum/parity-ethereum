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

///! Rust VM implementation

use common::*;
use super::instructions as instructions;
use super::instructions::Instruction;
use std::marker::Copy;
use evm::{self, MessageCallResult, ContractCreateResult};

#[cfg(not(feature = "evm-debug"))]
macro_rules! evm_debug {
	($x: expr) => {}
}

#[cfg(feature = "evm-debug")]
macro_rules! evm_debug {
	($x: expr) => {
		$x
	}
}

#[cfg(feature = "evm-debug")]
fn color(instruction: Instruction, name: &'static str) -> String {
	let c = instruction as usize % 6;
	let colors = [31, 34, 33, 32, 35, 36];
	format!("\x1B[1;{}m{}\x1B[0m", colors[c], name)
}

macro_rules! overflowing {
	($x: expr) => {{
		let (v, overflow) = $x;
		if overflow { return Err(evm::Error::OutOfGas); }
		v
	}}
}

type CodePosition = usize;
type Gas = U256;
type ProgramCounter = usize;

/// Stack trait with VM-friendly API
trait Stack<T> {
	/// Returns `Stack[len(Stack) - no_from_top]`
	fn peek(&self, no_from_top: usize) -> &T;
	/// Swaps Stack[len(Stack)] and Stack[len(Stack) - no_from_top]
	fn swap_with_top(&mut self, no_from_top: usize);
	/// Returns true if Stack has at least `no_of_elems` elements
	fn has(&self, no_of_elems: usize) -> bool;
	/// Get element from top and remove it from Stack. Panics if stack is empty.
	fn pop_back(&mut self) -> T;
	/// Get (up to `instructions::MAX_NO_OF_TOPICS`) elements from top and remove them from Stack. Panics if stack is empty.
	fn pop_n(&mut self, no_of_elems: usize) -> &[T];
	/// Add element on top of the Stack
	fn push(&mut self, elem: T);
	/// Get number of elements on Stack
	fn size(&self) -> usize;
}

struct VecStack<S> {
	stack: Vec<S>,
	logs: [S; instructions::MAX_NO_OF_TOPICS]
}

impl<S : Copy> VecStack<S> {
	fn with_capacity(capacity: usize, zero: S) -> Self {
		VecStack {
			stack: Vec::with_capacity(capacity),
			logs: [zero; instructions::MAX_NO_OF_TOPICS]
		}
	}
}

impl<S : fmt::Display> Stack<S> for VecStack<S> {
	fn peek(&self, no_from_top: usize) -> &S {
		&self.stack[self.stack.len() - no_from_top - 1]
	}

	fn swap_with_top(&mut self, no_from_top: usize) {
		let len = self.stack.len();
		self.stack.swap(len - no_from_top - 1, len - 1);
	}

	fn has(&self, no_of_elems: usize) -> bool {
		self.stack.len() >= no_of_elems
	}

	fn pop_back(&mut self) -> S {
		let val = self.stack.pop();
		match val {
			Some(x) => {
				evm_debug!({
					println!("   POP: {}", x)
				});
				x
			},
			None => panic!("Tried to pop from empty stack.")
		}
	}

	fn pop_n(&mut self, no_of_elems: usize) -> &[S] {
		assert!(no_of_elems <= instructions::MAX_NO_OF_TOPICS);

		for i in 0..no_of_elems {
			self.logs[i] = self.pop_back();
		}
		&self.logs[0..no_of_elems]
	}

	fn push(&mut self, elem: S) {
		evm_debug!({
			println!("  PUSH: {}", elem)
		});
		self.stack.push(elem);
	}

	fn size(&self) -> usize {
		self.stack.len()
	}
}

trait Memory {
	/// Retrieve current size of the memory
	fn size(&self) -> usize;
	/// Resize (shrink or expand) the memory to specified size (fills 0)
	fn resize(&mut self, new_size: usize);
	/// Resize the memory only if its smaller
	fn expand(&mut self, new_size: usize);
	/// Write single byte to memory
	fn write_byte(&mut self, offset: U256, value: U256);
	/// Write a word to memory. Does not resize memory!
	fn write(&mut self, offset: U256, value: U256);
	/// Read a word from memory
	fn read(&self, offset: U256) -> U256;
	/// Write slice of bytes to memory. Does not resize memory!
	fn write_slice(&mut self, offset: U256, &[u8]);
	/// Retrieve part of the memory between offset and offset + size
	fn read_slice(&self, offset: U256, size: U256) -> &[u8];
	/// Retrieve writeable part of memory
	fn writeable_slice(&mut self, offset: U256, size: U256) -> &mut[u8];
	fn dump(&self);
}

/// Checks whether offset and size is valid memory range
fn is_valid_range(off: usize, size: usize)  -> bool {
	// When size is zero we haven't actually expanded the memory
	let overflow = off.overflowing_add(size).1;
	size > 0 && !overflow
}

impl Memory for Vec<u8> {
	fn dump(&self) {
		println!("MemoryDump:");
		for i in self.iter() {
			println!("{:02x} ", i);
		}
		println!("");
	}

	fn size(&self) -> usize {
		self.len()
	}

	fn read_slice(&self, init_off_u: U256, init_size_u: U256) -> &[u8] {
		let off = init_off_u.low_u64() as usize;
		let size = init_size_u.low_u64() as usize;
		if !is_valid_range(off, size) {
			&self[0..0]
		} else {
			&self[off..off+size]
		}
	}

	fn read(&self, offset: U256) -> U256 {
		let off = offset.low_u64() as usize;
		U256::from(&self[off..off+32])
	}

	fn writeable_slice(&mut self, offset: U256, size: U256) -> &mut [u8] {
		let off = offset.low_u64() as usize;
		let s = size.low_u64() as usize;
		if !is_valid_range(off, s) {
			&mut self[0..0]
		} else {
			&mut self[off..off+s]
		}
	}

	fn write_slice(&mut self, offset: U256, slice: &[u8]) {
		let off = offset.low_u64() as usize;

		// TODO [todr] Optimize?
		for pos in off..off+slice.len() {
			self[pos] = slice[pos - off];
		}
	}

	fn write(&mut self, offset: U256, value: U256) {
		let off = offset.low_u64() as usize;
		let mut val = value;

		let end = off + 32;
		for pos in 0..32 {
			self[end - pos - 1] = val.low_u64() as u8;
			val = val >> 8;
		}
	}

	fn write_byte(&mut self, offset: U256, value: U256) {
		let off = offset.low_u64() as usize;
		let val = value.low_u64() as u64;
		self[off] = val as u8;
	}

	fn resize(&mut self, new_size: usize) {
		self.resize(new_size, 0);
	}

	fn expand(&mut self, size: usize) {
		if size > self.len() {
			Memory::resize(self, size)
		}
	}
}

/// Abstraction over raw vector of Bytes. Easier state management of PC.
struct CodeReader<'a> {
	position: ProgramCounter,
	code: &'a Bytes
}

#[cfg_attr(feature="dev", allow(len_without_is_empty))]
impl<'a> CodeReader<'a> {
	/// Get `no_of_bytes` from code and convert to U256. Move PC
	fn read(&mut self, no_of_bytes: usize) -> U256 {
		let pos = self.position;
		self.position += no_of_bytes;
		let max = cmp::min(pos + no_of_bytes, self.code.len());
		U256::from(&self.code[pos..max])
	}

	fn len (&self) -> usize {
		self.code.len()
	}
}

#[cfg_attr(feature="dev", allow(enum_variant_names))]
enum InstructionCost {
	Gas(U256),
	GasMem(U256, U256),
	GasMemCopy(U256, U256, U256)
}

enum InstructionResult {
	Ok,
	UseAllGas,
	GasLeft(U256),
	UnusedGas(U256),
	JumpToPosition(U256),
	StopExecutionWithGasLeft(U256),
	StopExecution
}

/// Intepreter EVM implementation
pub struct Interpreter;

impl evm::Evm for Interpreter {
	fn exec(&self, params: ActionParams, ext: &mut evm::Ext) -> evm::Result {
		let code = &params.code.as_ref().unwrap();
		let valid_jump_destinations = self.find_jump_destinations(&code);

		let mut current_gas = params.gas;
		let mut stack = VecStack::with_capacity(ext.schedule().stack_limit, U256::zero());
		let mut mem = vec![];
		let mut reader = CodeReader {
			position: 0,
			code: &code
		};

		while reader.position < code.len() {
			let instruction = code[reader.position];
			reader.position += 1;

			// Calculate gas cost
			let (gas_cost, mem_size) = try!(self.get_gas_cost_mem(ext, instruction, &mut mem, &stack));
			try!(self.verify_gas(&current_gas, &gas_cost));
			mem.expand(mem_size);
			current_gas = current_gas - gas_cost; //TODO: use operator -=

			evm_debug!({
				println!("[0x{:x}][{}(0x{:x}) Gas: {:x}\n  Gas Before: {:x}",
					reader.position,
					color(instruction, instructions::get_info(instruction).name),
					instruction,
					gas_cost,
					current_gas + gas_cost
				);
			});

			// Execute instruction
			let result = try!(self.exec_instruction(
					current_gas, &params, ext, instruction, &mut reader, &mut mem, &mut stack
					));

			// Advance
			match result {
				InstructionResult::Ok => {},
				InstructionResult::UnusedGas(gas) => {
					current_gas = current_gas + gas; //TODO: use operator +=
				},
				InstructionResult::UseAllGas => {
					current_gas = U256::zero();
				},
				InstructionResult::GasLeft(gas_left) => {
					current_gas = gas_left;
				},
				InstructionResult::JumpToPosition(position) => {
					let pos = try!(self.verify_jump(position, &valid_jump_destinations));
					reader.position = pos;
				},
				InstructionResult::StopExecutionWithGasLeft(gas_left) => {
					current_gas = gas_left;
					reader.position = code.len();
				},
				InstructionResult::StopExecution => {
					reader.position = code.len();
				}
			}
		}

		Ok(current_gas)
	}
}

impl Interpreter {
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn get_gas_cost_mem(
		&self,
		ext: &evm::Ext,
		instruction: Instruction,
		mem: &mut Memory,
		stack: &Stack<U256>
	) -> Result<(U256, usize), evm::Error> {
		let schedule = ext.schedule();
		let info = instructions::get_info(instruction);

		if !schedule.have_delegate_call && instruction == instructions::DELEGATECALL {
			return Err(evm::Error::BadInstruction {
				instruction: instruction
			});
		}
		if info.tier == instructions::GasPriceTier::Invalid {
			return Err(evm::Error::BadInstruction {
				instruction: instruction
			});
		}

		try!(self.verify_instructions_requirements(&info, schedule.stack_limit, stack));

		let tier = instructions::get_tier_idx(info.tier);
		let default_gas = U256::from(schedule.tier_step_gas[tier]);

		let cost = match instruction {
			instructions::SSTORE => {
				let address = H256::from(stack.peek(0));
				let newval = stack.peek(1);
				let val = U256::from(ext.storage_at(&address).as_slice());

				let gas = if self.is_zero(&val) && !self.is_zero(newval) {
					schedule.sstore_set_gas
				} else {
					// Refund for below case is added when actually executing sstore
					// !self.is_zero(&val) && self.is_zero(newval)
					schedule.sstore_reset_gas
				};
				InstructionCost::Gas(U256::from(gas))
			},
			instructions::SLOAD => {
				InstructionCost::Gas(U256::from(schedule.sload_gas))
			},
			instructions::MSTORE | instructions::MLOAD => {
				InstructionCost::GasMem(default_gas, try!(self.mem_needed_const(stack.peek(0), 32)))
			},
			instructions::MSTORE8 => {
				InstructionCost::GasMem(default_gas, try!(self.mem_needed_const(stack.peek(0), 1)))
			},
			instructions::RETURN => {
				InstructionCost::GasMem(default_gas, try!(self.mem_needed(stack.peek(0), stack.peek(1))))
			},
			instructions::SHA3 => {
				let w = overflowing!(add_u256_usize(stack.peek(1), 31));
				let words = w >> 5;
				let gas = U256::from(schedule.sha3_gas) + (U256::from(schedule.sha3_word_gas) * words);
				InstructionCost::GasMem(gas, try!(self.mem_needed(stack.peek(0), stack.peek(1))))
			},
			instructions::CALLDATACOPY | instructions::CODECOPY => {
				InstructionCost::GasMemCopy(default_gas, try!(self.mem_needed(stack.peek(0), stack.peek(2))), stack.peek(2).clone())
			},
			instructions::EXTCODECOPY => {
				InstructionCost::GasMemCopy(default_gas, try!(self.mem_needed(stack.peek(1), stack.peek(3))), stack.peek(3).clone())
			},
			instructions::JUMPDEST => {
				InstructionCost::Gas(U256::one())
			},
			instructions::LOG0...instructions::LOG4 => {
				let no_of_topics = instructions::get_log_topics(instruction);
				let log_gas = schedule.log_gas + schedule.log_topic_gas * no_of_topics;

				let data_gas = overflowing!(stack.peek(1).overflowing_mul(U256::from(schedule.log_data_gas)));
				let gas = overflowing!(data_gas.overflowing_add(U256::from(log_gas)));
				InstructionCost::GasMem(gas, try!(self.mem_needed(stack.peek(0), stack.peek(1))))
			},
			instructions::CALL | instructions::CALLCODE => {
				let mut gas  = overflowing!(add_u256_usize(stack.peek(0), schedule.call_gas));
				let mem = cmp::max(
					try!(self.mem_needed(stack.peek(5), stack.peek(6))),
					try!(self.mem_needed(stack.peek(3), stack.peek(4)))
				);

				let address = u256_to_address(stack.peek(1));

				if instruction == instructions::CALL && !ext.exists(&address) {
					gas = overflowing!(gas.overflowing_add(U256::from(schedule.call_new_account_gas)));
				};

				if stack.peek(2).clone() > U256::zero() {
					gas = overflowing!(gas.overflowing_add(U256::from(schedule.call_value_transfer_gas)));
				};

				InstructionCost::GasMem(gas,mem)
			},
			instructions::DELEGATECALL => {
				let gas = overflowing!(add_u256_usize(stack.peek(0), schedule.call_gas));
				let mem = cmp::max(
					try!(self.mem_needed(stack.peek(4), stack.peek(5))),
					try!(self.mem_needed(stack.peek(2), stack.peek(3)))
				);
				InstructionCost::GasMem(gas, mem)
			},
			instructions::CREATE => {
				let gas = U256::from(schedule.create_gas);
				let mem = try!(self.mem_needed(stack.peek(1), stack.peek(2)));
				InstructionCost::GasMem(gas, mem)
			},
			instructions::EXP => {
				let expon = stack.peek(1);
				let bytes = ((expon.bits() + 7) / 8) as usize;
				let gas = U256::from(schedule.exp_gas + schedule.exp_byte_gas * bytes);
				InstructionCost::Gas(gas)
			},
			_ => InstructionCost::Gas(default_gas)
		};

		match cost {
			InstructionCost::Gas(gas) => {
				Ok((gas, 0))
			},
			InstructionCost::GasMem(gas, mem_size) => {
				let (mem_gas, new_mem_size) = try!(self.mem_gas_cost(schedule, mem.size(), &mem_size));
				let gas = overflowing!(gas.overflowing_add(mem_gas));
				Ok((gas, new_mem_size))
			},
			InstructionCost::GasMemCopy(gas, mem_size, copy) => {
				let (mem_gas, new_mem_size) = try!(self.mem_gas_cost(schedule, mem.size(), &mem_size));
				let copy = overflowing!(add_u256_usize(&copy, 31));
				let copy_gas = U256::from(schedule.copy_gas) * (copy / U256::from(32));
				let gas = overflowing!(gas.overflowing_add(copy_gas));
				let gas = overflowing!(gas.overflowing_add(mem_gas));
				Ok((gas, new_mem_size))
			}
		}
	}

	fn mem_gas_cost(&self, schedule: &evm::Schedule, current_mem_size: usize, mem_size: &U256) -> Result<(U256, usize), evm::Error> {
		let gas_for_mem = |mem_size: U256| {
			let s = mem_size >> 5;
			// s * memory_gas + s * s / quad_coeff_div
			let a = overflowing!(s.overflowing_mul(U256::from(schedule.memory_gas)));
			// We need to go to U512 to calculate s*s/quad_coeff_div
			let b = U512::from(s) * U512::from(s) / U512::from(schedule.quad_coeff_div);
			if b > U512::from(!U256::zero()) {
				Err(evm::Error::OutOfGas)
			} else {
				Ok(overflowing!(a.overflowing_add(U256::from(b))))
			}
		};
		let current_mem_size = U256::from(current_mem_size);
		let req_mem_size_rounded = (overflowing!(mem_size.overflowing_add(U256::from(31))) >> 5) << 5;
		let new_mem_gas = try!(gas_for_mem(U256::from(req_mem_size_rounded)));
		let current_mem_gas = try!(gas_for_mem(current_mem_size));

		Ok((if req_mem_size_rounded > current_mem_size {
			new_mem_gas - current_mem_gas
		} else {
			U256::zero()
		}, req_mem_size_rounded.low_u64() as usize))
	}

	fn mem_needed_const(&self, mem: &U256, add: usize) -> Result<U256, evm::Error> {
		Ok(overflowing!(mem.overflowing_add(U256::from(add))))
	}

	fn mem_needed(&self, offset: &U256, size: &U256) -> Result<U256, ::evm::Error> {
		if self.is_zero(size) {
			return Ok(U256::zero());
		}

		Ok(overflowing!(offset.overflowing_add(size.clone())))
	}

	#[cfg_attr(feature="dev", allow(too_many_arguments))]
	fn exec_instruction(
		&self,
		gas: Gas,
		params: &ActionParams,
		ext: &mut evm::Ext,
		instruction: Instruction,
		code: &mut CodeReader,
		mem: &mut Memory,
		stack: &mut Stack<U256>
	) -> Result<InstructionResult, evm::Error> {
		match instruction {
			instructions::JUMP => {
				let jump = stack.pop_back();
				return Ok(InstructionResult::JumpToPosition(
					jump
				));
			},
			instructions::JUMPI => {
				let jump = stack.pop_back();
				let condition = stack.pop_back();
				if !self.is_zero(&condition) {
					return Ok(InstructionResult::JumpToPosition(
						jump
					));
				}
			},
			instructions::JUMPDEST => {
				// ignore
			},
			instructions::CREATE => {
				let endowment = stack.pop_back();
				let init_off = stack.pop_back();
				let init_size = stack.pop_back();

				let contract_code = mem.read_slice(init_off, init_size);
				let can_create = ext.balance(&params.address) >= endowment && ext.depth() < ext.schedule().max_depth;

				if !can_create {
					stack.push(U256::zero());
					return Ok(InstructionResult::Ok);
				}

				let create_result = ext.create(&gas, &endowment, &contract_code);
				return match create_result {
					ContractCreateResult::Created(address, gas_left) => {
						stack.push(address_to_u256(address));
						Ok(InstructionResult::GasLeft(gas_left))
					},
					ContractCreateResult::Failed => {
						stack.push(U256::zero());
						// TODO [todr] Should we just StopExecution here?
						Ok(InstructionResult::UseAllGas)
					}
				};
			},
			instructions::CALL | instructions::CALLCODE | instructions::DELEGATECALL => {
				assert!(ext.schedule().call_value_transfer_gas > ext.schedule().call_stipend, "overflow possible");
				let call_gas = stack.pop_back();
				let code_address = stack.pop_back();
				let code_address = u256_to_address(&code_address);

				let value = if instruction == instructions::DELEGATECALL {
					None
				} else {
					Some(stack.pop_back())
				};

				let in_off = stack.pop_back();
				let in_size = stack.pop_back();
				let out_off = stack.pop_back();
				let out_size = stack.pop_back();

				// Add stipend (only CALL|CALLCODE when value > 0)
				let call_gas = call_gas + value.map_or_else(U256::zero, |val| match val > U256::zero() {
					true => U256::from(ext.schedule().call_stipend),
					false => U256::zero()
				});

				// Get sender & receive addresses, check if we have balance
				let (sender_address, receive_address, has_balance) = match instruction {
					instructions::CALL => {
						let has_balance = ext.balance(&params.address) >= value.unwrap();
						(&params.address, &code_address, has_balance)
					},
					instructions::CALLCODE => {
						let has_balance = ext.balance(&params.address) >= value.unwrap();
						(&params.address, &params.address, has_balance)
					},
					instructions::DELEGATECALL => (&params.sender, &params.address, true),
					_ => panic!(format!("Unexpected instruction {} in CALL branch.", instruction))
				};

				let can_call = has_balance && ext.depth() < ext.schedule().max_depth;
				if !can_call {
					stack.push(U256::zero());
					return Ok(InstructionResult::UnusedGas(call_gas));
				}

				let call_result = {
					// we need to write and read from memory in the same time
					// and we don't want to copy
					let input = unsafe { ::std::mem::transmute(mem.read_slice(in_off, in_size)) };
					let output = mem.writeable_slice(out_off, out_size);
					ext.call(&call_gas, sender_address, receive_address, value, input, &code_address, output)
				};

				return match call_result {
					MessageCallResult::Success(gas_left) => {
						stack.push(U256::one());
						Ok(InstructionResult::UnusedGas(gas_left))
					},
					MessageCallResult::Failed  => {
						stack.push(U256::zero());
						Ok(InstructionResult::Ok)
					}
				};
			},
			instructions::RETURN => {
				let init_off = stack.pop_back();
				let init_size = stack.pop_back();
				let return_code = mem.read_slice(init_off, init_size);
				let gas_left = try!(ext.ret(&gas, &return_code));
				return Ok(InstructionResult::StopExecutionWithGasLeft(
					gas_left
				));
			},
			instructions::STOP => {
				return Ok(InstructionResult::StopExecution);
			},
			instructions::SUICIDE => {
				let address = stack.pop_back();
				ext.suicide(&u256_to_address(&address));
				return Ok(InstructionResult::StopExecution);
			},
			instructions::LOG0...instructions::LOG4 => {
				let no_of_topics = instructions::get_log_topics(instruction);

				let offset = stack.pop_back();
				let size = stack.pop_back();
				let topics = stack.pop_n(no_of_topics)
					.iter()
					.map(H256::from)
					.collect();
				ext.log(topics, mem.read_slice(offset, size));
			},
			instructions::PUSH1...instructions::PUSH32 => {
				let bytes = instructions::get_push_bytes(instruction);
				let val = code.read(bytes);
				stack.push(val);
			},
			instructions::MLOAD => {
				let word = mem.read(stack.pop_back());
				stack.push(U256::from(word));
			},
			instructions::MSTORE => {
				let offset = stack.pop_back();
				let word = stack.pop_back();
				mem.write(offset, word);
			},
			instructions::MSTORE8 => {
				let offset = stack.pop_back();
				let byte = stack.pop_back();
				mem.write_byte(offset, byte);
			},
			instructions::MSIZE => {
				stack.push(U256::from(mem.size()));
			},
			instructions::SHA3 => {
				let offset = stack.pop_back();
				let size = stack.pop_back();
				let sha3 = mem.read_slice(offset, size).sha3();
				stack.push(U256::from(sha3.as_slice()));
			},
			instructions::SLOAD => {
				let key = H256::from(&stack.pop_back());
				let word = U256::from(ext.storage_at(&key).as_slice());
				stack.push(word);
			},
			instructions::SSTORE => {
				let address = H256::from(&stack.pop_back());
				let val = stack.pop_back();

				let current_val = U256::from(ext.storage_at(&address).as_slice());
				// Increase refund for clear
				if !self.is_zero(&current_val) && self.is_zero(&val) {
					ext.inc_sstore_clears();
				}
				ext.set_storage(address, H256::from(&val));
			},
			instructions::PC => {
				stack.push(U256::from(code.position - 1));
			},
			instructions::GAS => {
				stack.push(gas.clone());
			},
			instructions::ADDRESS => {
				stack.push(address_to_u256(params.address.clone()));
			},
			instructions::ORIGIN => {
				stack.push(address_to_u256(params.origin.clone()));
			},
			instructions::BALANCE => {
				let address = u256_to_address(&stack.pop_back());
				let balance = ext.balance(&address);
				stack.push(balance);
			},
			instructions::CALLER => {
				stack.push(address_to_u256(params.sender.clone()));
			},
			instructions::CALLVALUE => {
				stack.push(match params.value {
					ActionValue::Transfer(val) | ActionValue::Apparent(val) => val
				});
			},
			instructions::CALLDATALOAD => {
				let big_id = stack.pop_back();
				let id = big_id.low_u64() as usize;
				let max = id.wrapping_add(32);
				if let Some(data) = params.data.as_ref() {
					let bound = cmp::min(data.len(), max);
					if id < bound && big_id < U256::from(data.len()) {
						let mut v = [0u8; 32];
						v[0..bound-id].clone_from_slice(&data[id..bound]);
						stack.push(U256::from(&v[..]))
					} else {
						stack.push(U256::zero())
					}
				} else {
					stack.push(U256::zero())
				}
			},
			instructions::CALLDATASIZE => {
				stack.push(U256::from(params.data.clone().map_or(0, |l| l.len())));
			},
			instructions::CODESIZE => {
				stack.push(U256::from(code.len()));
			},
			instructions::EXTCODESIZE => {
				let address = u256_to_address(&stack.pop_back());
				let len = ext.extcode(&address).len();
				stack.push(U256::from(len));
			},
			instructions::CALLDATACOPY => {
				self.copy_data_to_memory(mem, stack, &params.data.clone().unwrap_or_else(|| vec![]));
			},
			instructions::CODECOPY => {
				self.copy_data_to_memory(mem, stack, &params.code.clone().unwrap_or_else(|| vec![]));
			},
			instructions::EXTCODECOPY => {
				let address = u256_to_address(&stack.pop_back());
				let code = ext.extcode(&address);
				self.copy_data_to_memory(mem, stack, &code);
			},
			instructions::GASPRICE => {
				stack.push(params.gas_price.clone());
			},
			instructions::BLOCKHASH => {
				let block_number = stack.pop_back();
				let block_hash = ext.blockhash(&block_number);
				stack.push(U256::from(block_hash.as_slice()));
			},
			instructions::COINBASE => {
				stack.push(address_to_u256(ext.env_info().author.clone()));
			},
			instructions::TIMESTAMP => {
				stack.push(U256::from(ext.env_info().timestamp));
			},
			instructions::NUMBER => {
				stack.push(U256::from(ext.env_info().number));
			},
			instructions::DIFFICULTY => {
				stack.push(ext.env_info().difficulty.clone());
			},
			instructions::GASLIMIT => {
				stack.push(ext.env_info().gas_limit.clone());
			},
			_ => {
				try!(self.exec_stack_instruction(instruction, stack));
			}
		};
		Ok(InstructionResult::Ok)
	}

	fn copy_data_to_memory(&self, mem: &mut Memory, stack: &mut Stack<U256>, data: &[u8]) {
		let dest_offset = stack.pop_back();
		let source_offset = stack.pop_back();
		let size = stack.pop_back();
		let source_size = U256::from(data.len());

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
				data.len()
			},
			false => (size.low_u64() + source_offset.low_u64()) as usize
		};

		if source_offset < source_size {
			let output_begin = source_offset.low_u64() as usize;
			mem.write_slice(dest_offset, &data[output_begin..output_end]);
		}
	}

	fn verify_instructions_requirements(&self,
										info: &instructions::InstructionInfo,
										stack_limit: usize,
										stack: &Stack<U256>) -> Result<(), evm::Error> {
		if !stack.has(info.args) {
			Err(evm::Error::StackUnderflow {
				instruction: info.name,
				wanted: info.args,
				on_stack: stack.size()
			})
		} else if stack.size() - info.args + info.ret > stack_limit {
			Err(evm::Error::OutOfStack {
				instruction: info.name,
				wanted: info.ret - info.args,
				limit: stack_limit
			})
		} else {
			Ok(())
		}
	}

	fn verify_gas(&self, current_gas: &U256, gas_cost: &U256) -> Result<(), evm::Error> {
		match current_gas < gas_cost {
			true => Err(evm::Error::OutOfGas),
			false => Ok(())
		}
	}

	fn verify_jump(&self, jump_u: U256, valid_jump_destinations: &HashSet<usize>) -> Result<usize, evm::Error> {
		let jump = jump_u.low_u64() as usize;

		if valid_jump_destinations.contains(&jump) && jump_u < U256::from(!0 as usize) {
			Ok(jump)
		} else {
			Err(evm::Error::BadJumpDestination {
				destination: jump
			})
		}
	}

	fn is_zero(&self, val: &U256) -> bool {
		&U256::zero() == val
	}

	fn bool_to_u256(&self, val: bool) -> U256 {
		if val {
			U256::one()
		} else {
			U256::zero()
		}
	}

	fn exec_stack_instruction(&self, instruction: Instruction, stack: &mut Stack<U256>) -> Result<(), evm::Error> {
		match instruction {
			instructions::DUP1...instructions::DUP16 => {
				let position = instructions::get_dup_position(instruction);
				let val = stack.peek(position).clone();
				stack.push(val);
			},
			instructions::SWAP1...instructions::SWAP16 => {
				let position = instructions::get_swap_position(instruction);
				stack.swap_with_top(position)
			},
			instructions::POP => {
				stack.pop_back();
			},
			instructions::ADD => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a.overflowing_add(b).0);
			},
			instructions::MUL => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a.overflowing_mul(b).0);
			},
			instructions::SUB => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a.overflowing_sub(b).0);
			},
			instructions::DIV => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(if !self.is_zero(&b) {
					a.overflowing_div(b).0
				} else {
					U256::zero()
				});
			},
			instructions::MOD => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(if !self.is_zero(&b) {
					a.overflowing_rem(b).0
				} else {
					U256::zero()
				});
			},
			instructions::SDIV => {
				let (a, sign_a) = get_and_reset_sign(stack.pop_back());
				let (b, sign_b) = get_and_reset_sign(stack.pop_back());

				// -2^255
				let min = (U256::one() << 255) - U256::one();
				stack.push(if self.is_zero(&b) {
					U256::zero()
				} else if a == min && b == !U256::zero() {
					min
				} else {
					let c = a.overflowing_div(b).0;
					set_sign(c, sign_a ^ sign_b)
				});
			},
			instructions::SMOD => {
				let ua = stack.pop_back();
				let ub = stack.pop_back();
				let (a, sign_a) = get_and_reset_sign(ua);
				let b = get_and_reset_sign(ub).0;

				stack.push(if !self.is_zero(&b) {
					let c = a.overflowing_rem(b).0;
					set_sign(c, sign_a)
				} else {
					U256::zero()
				});
			},
			instructions::EXP => {
				let base = stack.pop_back();
				let expon = stack.pop_back();
				let res = base.overflowing_pow(expon).0;
				stack.push(res);
			},
			instructions::NOT => {
				let a = stack.pop_back();
				stack.push(!a);
			},
			instructions::LT => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(self.bool_to_u256(a < b));
			},
			instructions::SLT => {
				let (a, neg_a) = get_and_reset_sign(stack.pop_back());
				let (b, neg_b) = get_and_reset_sign(stack.pop_back());

				let is_positive_lt = a < b && !(neg_a | neg_b);
				let is_negative_lt = a > b && (neg_a & neg_b);
				let has_different_signs = neg_a && !neg_b;

				stack.push(self.bool_to_u256(is_positive_lt | is_negative_lt | has_different_signs));
			},
			instructions::GT => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(self.bool_to_u256(a > b));
			},
			instructions::SGT => {
				let (a, neg_a) = get_and_reset_sign(stack.pop_back());
				let (b, neg_b) = get_and_reset_sign(stack.pop_back());

				let is_positive_gt = a > b && !(neg_a | neg_b);
				let is_negative_gt = a < b && (neg_a & neg_b);
				let has_different_signs = !neg_a && neg_b;

				stack.push(self.bool_to_u256(is_positive_gt | is_negative_gt | has_different_signs));
			},
			instructions::EQ => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(self.bool_to_u256(a == b));
			},
			instructions::ISZERO => {
				let a = stack.pop_back();
				stack.push(self.bool_to_u256(self.is_zero(&a)));
			},
			instructions::AND => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a & b);
			},
			instructions::OR => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a | b);
			},
			instructions::XOR => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a ^ b);
			},
			instructions::BYTE => {
				let word = stack.pop_back();
				let val = stack.pop_back();
				let byte = match word < U256::from(32) {
					true => (val >> (8 * (31 - word.low_u64() as usize))) & U256::from(0xff),
					false => U256::zero()
				};
				stack.push(byte);
			},
			instructions::ADDMOD => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				let c = stack.pop_back();

				stack.push(if !self.is_zero(&c) {
					// upcast to 512
					let a5 = U512::from(a);
					let res = a5.overflowing_add(U512::from(b)).0;
					let x = res.overflowing_rem(U512::from(c)).0;
					U256::from(x)
				} else {
					U256::zero()
				});
			},
			instructions::MULMOD => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				let c = stack.pop_back();

				stack.push(if !self.is_zero(&c) {
					let a5 = U512::from(a);
					let res = a5.overflowing_mul(U512::from(b)).0;
					let x = res.overflowing_rem(U512::from(c)).0;
					U256::from(x)
				} else {
					U256::zero()
				});
			},
			instructions::SIGNEXTEND => {
				let bit = stack.pop_back();
				if bit < U256::from(32) {
					let number = stack.pop_back();
					let bit_position = (bit.low_u64() * 8 + 7) as usize;

					let bit = number.bit(bit_position);
					let mask = (U256::one() << bit_position) - U256::one();
					stack.push(if bit {
						number | !mask
					} else {
						number & mask
					});
				}
			},
			_ => {
				return Err(evm::Error::BadInstruction {
					instruction: instruction
				});
			}
		}
		Ok(())
	}

	fn find_jump_destinations(&self, code: &[u8]) -> HashSet<CodePosition> {
		let mut jump_dests = HashSet::new();
		let mut position = 0;

		while position < code.len() {
			let instruction = code[position];

			if instruction == instructions::JUMPDEST {
				jump_dests.insert(position);
			} else if instructions::is_push(instruction) {
				position += instructions::get_push_bytes(instruction);
			}
			position += 1;
		}

		jump_dests
	}
}

fn get_and_reset_sign(value: U256) -> (U256, bool) {
	let sign = (value >> 255).low_u64() == 1;
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
fn add_u256_usize(value: &U256, num: usize) -> (U256, bool) {
	value.clone().overflowing_add(U256::from(num))
}

#[inline]
fn u256_to_address(value: &U256) -> Address {
	Address::from(H256::from(value))
}

#[inline]
fn address_to_u256(value: Address) -> U256 {
	U256::from(H256::from(value).as_slice())
}

#[test]
fn test_mem_gas_cost() {
	// given
	let interpreter = Interpreter;
	let schedule = evm::Schedule::default();
	let current_mem_size = 5;
	let mem_size = !U256::zero();

	// when
	let result = interpreter.mem_gas_cost(&schedule, current_mem_size, &mem_size);

	// then
	if let Ok(_) = result {
		assert!(false, "Should fail with OutOfGas");
	}
}

#[cfg(test)]
mod tests {
	use common::*;
	use super::*;
	use evm;

	#[test]
	fn test_find_jump_destinations() {
		// given
		let interpreter = Interpreter;
		let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5b01600055".from_hex().unwrap();

		// when
		let valid_jump_destinations = interpreter.find_jump_destinations(&code);

		// then
		assert!(valid_jump_destinations.contains(&66));
	}

	#[test]
	fn test_calculate_mem_cost() {
		// given
		let interpreter = Interpreter;
		let schedule = evm::Schedule::default();
		let current_mem_size = 0;
		let mem_size = U256::from(5);

		// when
		let (mem_cost, mem_size) = interpreter.mem_gas_cost(&schedule, current_mem_size, &mem_size).unwrap();

		// then
		assert_eq!(mem_cost, U256::from(3));
		assert_eq!(mem_size, 32);
	}

	#[test]
	fn test_memory_read_and_write() {
		// given
		let mem: &mut super::Memory = &mut vec![];
		mem.resize(0x80 + 32);

		// when
		mem.write(U256::from(0x80), U256::from(0xabcdef));

		// then
		assert_eq!(mem.read(U256::from(0x80)), U256::from(0xabcdef));
	}

	#[test]
	fn test_memory_read_and_write_byte() {
		// given
		let mem: &mut super::Memory = &mut vec![];
		mem.resize(32);

		// when
		mem.write_byte(U256::from(0x1d), U256::from(0xab));
		mem.write_byte(U256::from(0x1e), U256::from(0xcd));
		mem.write_byte(U256::from(0x1f), U256::from(0xef));

		// then
		assert_eq!(mem.read(U256::from(0x00)), U256::from(0xabcdef));
	}
}
