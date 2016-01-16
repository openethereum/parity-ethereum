///! Rust VM implementation

use common::*;
use evm;
use super::instructions as instructions;
use super::instructions::Instruction;
use std::num::wrapping::OverflowingOps;

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
	/// Get elements from top and remove them from Stack. Panics if stack is empty.
	fn pop_n(&mut self, no_of_elems: usize) -> Vec<T>;
	/// Add element on top of the Stack
	fn push(&mut self, elem: T);
	/// Get number of elements on Stack
	fn size(&self) -> usize;
}
impl<S : fmt::Display> Stack<S> for Vec<S> {
	fn peek(&self, no_from_top: usize) -> &S {
		return &self[self.len() - no_from_top - 1];
	}

	fn swap_with_top(&mut self, no_from_top: usize) {
		let len = self.len();
		self.swap(len - no_from_top - 1, len - 1);
	}

	fn has(&self, no_of_elems: usize) -> bool {
		self.len() >= no_of_elems
	}

	fn pop_back(&mut self) -> S {
		let val = self.pop();
		match val {
			Some(x) => {
				// println!("Poping from stack: {}", x);
				x
			},
			None => panic!("Tried to pop from empty stack.")
		}
	}

	fn pop_n(&mut self, no_of_elems: usize) -> Vec<S> {
		let mut vec = Vec::new();

		for _i in 1..no_of_elems+1 {
			vec.push(self.pop_back());
		}
		vec
	}

	fn push(&mut self, elem: S) {
		// println!("Pushing to stack: {}", elem);
		self.push(elem);
	}

	fn size(&self) -> usize {
		self.len()
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

fn is_valid_range(off: usize, size: usize)  -> bool {
	// When size is zero we haven't actually expanded the memory
	let (_a, overflow) = off.overflowing_add(size);
	size > 0 && !overflow
}

impl Memory for Vec<u8> {
	fn dump(&self) {
		println!("MemoryDump:");
		for i in self.iter() {
			print!("{:02x} ", i);
		}
		println!("");
	}

	fn size(&self) -> usize {
		return self.len()
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

enum RequiredMem {
	Mem(U256),
	OutOfMemory
}

enum InstructionCost {
	Gas(U256),
	GasMem(U256, RequiredMem),
	GasMemCopy(U256, RequiredMem, U256)
}

enum InstructionResult {
	AdditionalGasCost(U256),
	JumpToPosition(U256),
	StopExecutionWithGasCost(U256),
	StopExecution
}

pub struct Interpreter;

impl evm::Evm for Interpreter {
	fn exec(&self, params: &ActionParams, ext: &mut evm::Ext) -> evm::Result {
		let code = &params.code.clone().unwrap();
		let valid_jump_destinations = self.find_jump_destinations(&code);

		let mut current_gas = params.gas.clone();
		let mut stack = Vec::with_capacity(ext.schedule().stack_limit);
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
			current_gas = current_gas - gas_cost;

			// println!("Executing: {} (0x{:x}) [Gas Cost: {} (Left: {})]", 
			// 				   instructions::get_info(instruction).name, instruction,
			// 				   gas_cost,
			// 				   current_gas
			// );

			// Execute instruction
			let result = try!(self.exec_instruction(
					current_gas, params, ext, instruction, &mut reader, &mut mem, &mut stack
					));

			// Advance
			match result {
				InstructionResult::JumpToPosition(position) => {
					let pos = try!(self.verify_jump(position, &valid_jump_destinations));
					reader.position = pos;
				},
				InstructionResult::AdditionalGasCost(gas_cost) => {
					current_gas = current_gas - gas_cost;
				},
				InstructionResult::StopExecutionWithGasCost(gas_cost) => { 
					current_gas = current_gas - gas_cost;
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

	fn get_gas_cost_mem(&self,
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
		if info.tier == instructions::GasPriceTier::InvalidTier {
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
				} else if !self.is_zero(&val) && self.is_zero(newval) {
					schedule.sstore_set_gas
				} else {
					schedule.sstore_reset_gas
				};
				InstructionCost::Gas(U256::from(gas))
			},
			instructions::SLOAD => {
				InstructionCost::Gas(U256::from(schedule.sload_gas))
			},
			instructions::MSTORE => {
				InstructionCost::GasMem(default_gas, self.mem_needed_const(stack.peek(0), 32))
			},
			instructions::MLOAD => {
				InstructionCost::GasMem(default_gas, self.mem_needed_const(stack.peek(0), 32))
			},
			instructions::MSTORE8 => {
				InstructionCost::GasMem(default_gas, self.mem_needed_const(stack.peek(0), 1))
			},
			instructions::RETURN => {
				InstructionCost::GasMem(default_gas, self.mem_needed(stack.peek(0), stack.peek(1)))
			},
			instructions::SHA3 => {
				match add_u256_usize(stack.peek(1), 31) {
					(_w, true) => InstructionCost::GasMem(U256::zero(), RequiredMem::OutOfMemory),
					(w, false) => {
						let words = w >> 5;
						let gas = U256::from(schedule.sha3_gas) + (U256::from(schedule.sha3_word_gas) * words);
						InstructionCost::GasMem(gas, self.mem_needed(stack.peek(0), stack.peek(1)))
					}
				}
			},
			instructions::CALLDATACOPY => {
				InstructionCost::GasMemCopy(default_gas, self.mem_needed(stack.peek(0), stack.peek(2)), stack.peek(2).clone())
			},
			instructions::CODECOPY => {
				InstructionCost::GasMemCopy(default_gas, self.mem_needed(stack.peek(0), stack.peek(2)), stack.peek(2).clone())
			},
			instructions::EXTCODECOPY => {
				InstructionCost::GasMemCopy(default_gas, self.mem_needed(stack.peek(1), stack.peek(3)), stack.peek(3).clone())
			},
			instructions::JUMPDEST => {
				InstructionCost::Gas(U256::one())
			},
			instructions::LOG0...instructions::LOG4 => {
				let no_of_topics = instructions::get_log_topics(instruction);
				let log_gas = schedule.log_gas + schedule.log_topic_gas * no_of_topics;
				let data_gas = stack.peek(1).clone() * U256::from(schedule.log_data_gas);
				let gas = try!(self.gas_add(data_gas, U256::from(log_gas)));
				InstructionCost::GasMem(gas, self.mem_needed(stack.peek(0), stack.peek(1)))
			},
			instructions::CALL | instructions::CALLCODE => {
				// [todr] we actuall call gas_cost is calculated in ext
				let gas = U256::from(schedule.call_gas);
				let mem = self.mem_max(
					self.mem_needed(stack.peek(5), stack.peek(6)),
					self.mem_needed(stack.peek(3), stack.peek(4))
				);
				InstructionCost::GasMem(gas, mem)
			},
			instructions::DELEGATECALL => {
				match add_u256_usize(stack.peek(0), schedule.call_gas) {
					(_gas, true) => InstructionCost::GasMem(U256::zero(), RequiredMem::OutOfMemory),
					(gas, false) => {
						let mem = self.mem_max(
							self.mem_needed(stack.peek(4), stack.peek(5)),
							self.mem_needed(stack.peek(2), stack.peek(3))
						);
						InstructionCost::GasMem(gas, mem)
					}
				}
			},
			instructions::CREATE => {
				let gas = U256::from(schedule.create_gas);
				let mem = self.mem_needed(stack.peek(1), stack.peek(2));
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
			InstructionCost::GasMem(gas, mem_size) => match mem_size {
				RequiredMem::Mem(mem_size) => {
					let (mem_gas, new_mem_size) = self.mem_gas_cost(schedule, mem.size(), &mem_size);
					let gas = try!(self.gas_add(gas, mem_gas));
					Ok((gas, new_mem_size))
				},
				RequiredMem::OutOfMemory => Err(evm::Error::OutOfGas)
			},
			InstructionCost::GasMemCopy(gas, mem_size, copy) => match mem_size {
				RequiredMem::Mem(mem_size) => {
					let (mem_gas, new_mem_size) = self.mem_gas_cost(schedule, mem.size(), &mem_size);
					match add_u256_usize(&copy, 31) {
						(_c, true) => Err(evm::Error::OutOfGas),
						(copy, false) => {
							let copy_gas = U256::from(schedule.copy_gas) * (copy / U256::from(32));
							let gas = try!(self.gas_add(try!(self.gas_add(gas, copy_gas)), mem_gas));
							Ok((gas, new_mem_size))
						}
					}
				},
				RequiredMem::OutOfMemory => Err(evm::Error::OutOfGas)
			}
		}
	}

	fn gas_add(&self, a: U256, b: U256) -> Result<U256, evm::Error> {
		match a.overflowing_add(b) {
			(_val, true) => Err(evm::Error::OutOfGas),
			(val, false) => Ok(val)
		}
	}
	
	fn mem_gas_cost(&self, schedule: &evm::Schedule, current_mem_size: usize, mem_size: &U256) -> (U256, usize) {
		let gas_for_mem = |mem_size: U256| {
			let s = mem_size >> 5;
			s * U256::from(schedule.memory_gas) + s * s / U256::from(schedule.quad_coeff_div)
		};

		let current_mem_size = U256::from(current_mem_size);
		let req_mem_size_rounded = ((mem_size.clone() + U256::from(31)) >> 5) << 5;
		let new_mem_gas = gas_for_mem(U256::from(req_mem_size_rounded));
		let current_mem_gas = gas_for_mem(current_mem_size);

		(if req_mem_size_rounded > current_mem_size {
			new_mem_gas - current_mem_gas
		} else {
			U256::zero()
		}, req_mem_size_rounded.low_u64() as usize)
	}


	fn mem_max(&self, m_a: RequiredMem, m_b: RequiredMem) -> RequiredMem {
		match (m_a, m_b) {
			(RequiredMem::Mem(a), RequiredMem::Mem(b)) => {
				RequiredMem::Mem(cmp::max(a, b))
			},
			(RequiredMem::OutOfMemory, _) | (_, RequiredMem::OutOfMemory) => {
				RequiredMem::OutOfMemory
			}
		}
	}

	fn mem_needed_const(&self, mem: &U256, add: usize) -> RequiredMem {
		match mem.overflowing_add(U256::from(add)) {
			(_, true) => RequiredMem::OutOfMemory,
			(mem, false) => RequiredMem::Mem(mem)
		}
	}

	fn mem_needed(&self, offset: &U256, size: &U256) -> RequiredMem {
		if self.is_zero(size) {
			RequiredMem::Mem(U256::zero())
		} else {
			match offset.clone().overflowing_add(size.clone()) {
				(_result, true) => RequiredMem::OutOfMemory,
				(result, false) => {
						RequiredMem::Mem(result)
				}
			}
		}
	}

	fn exec_instruction(&self,
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
				let (gas_left, maybe_address) = ext.create(&gas, &endowment, &contract_code);
				match maybe_address {
					Some(address) => stack.push(address_to_u256(address)),
					None => stack.push(U256::zero())
				}
				return Ok(InstructionResult::AdditionalGasCost(
					gas - gas_left
				));
			},
			instructions::CALL | instructions::CALLCODE | instructions::DELEGATECALL => {
				let call_gas = stack.pop_back();
				let code_address = stack.pop_back();
				let code_address = u256_to_address(&code_address);

				let value = if instruction == instructions::DELEGATECALL {
					params.value
				} else {
					stack.pop_back()
				};

				let address = if instruction == instructions::CALL {
					&code_address
				} else {
					&params.address
				};

				let in_off = stack.pop_back();
				let in_size = stack.pop_back();
				let out_off = stack.pop_back();
				let out_size = stack.pop_back();

				let (gas_left, call_successful) = {
					// we need to write and read from memory in the same time
					// and we don't want to copy
					let input = unsafe { ::std::mem::transmute(mem.read_slice(in_off, in_size)) };
					let output = mem.writeable_slice(out_off, out_size);
					try!(
						ext.call(&gas, &call_gas, address, &value, input, &code_address, output)
					)
				};
				if call_successful {
					stack.push(U256::one());
				} else {
					stack.push(U256::zero());
				}
				return Ok(InstructionResult::AdditionalGasCost(
					gas - gas_left
				));
			}, 
			instructions::RETURN => {
				let init_off = stack.pop_back();
				let init_size = stack.pop_back();
				let return_code = mem.read_slice(init_off, init_size);
				let gas_left = try!(ext.ret(&gas, &return_code));
				return Ok(InstructionResult::StopExecutionWithGasCost(
					gas - gas_left
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
				ext.log(topics, mem.read_slice(offset, size).to_vec());
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
				let key = H256::from(&stack.pop_back());
				let word = H256::from(&stack.pop_back());
				ext.set_storage_at(key, word);
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
				stack.push(params.value.clone());
			},
			instructions::CALLDATALOAD => {
				let big_id = stack.pop_back();
				let id = big_id.low_u64() as usize;
				let max = id.wrapping_add(32);
				let data = params.data.clone().unwrap();
				let bound = cmp::min(data.len(), max);
				if id < bound && big_id < U256::from(data.len()) {
					let mut v = data[id..bound].to_vec();
					v.resize(32, 0);
					stack.push(U256::from(&v[..]))
				} else {
					stack.push(U256::zero())
				}
			},
			instructions::CALLDATASIZE => {
				stack.push(U256::from(params.data.clone().unwrap().len()));
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
				self.copy_data_to_memory(mem, stack, &params.data.clone().unwrap());
			},
			instructions::CODECOPY => {
				self.copy_data_to_memory(mem, stack, &params.code.clone().unwrap());
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
		Ok(InstructionResult::AdditionalGasCost(U256::zero()))
	}

	fn copy_data_to_memory(&self,
						   mem: &mut Memory,
						   stack: &mut Stack<U256>,
						   data: &Bytes) {
		let offset = stack.pop_back();
		let index = stack.pop_back();
		let size = stack.pop_back();
		let data_size = data.len();

		if index < U256::from(data_size) {
			let u_index = index.low_u64() as usize;
			let bound_size = if size + index > U256::from(data_size) {
				data_size
			} else {
				size.low_u64() as usize + u_index
			};

			mem.write_slice(offset, &data[u_index..bound_size]);
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
		if current_gas < gas_cost {
			Err(evm::Error::OutOfGas)
		} else {
			Ok(())
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

	fn exec_stack_instruction(&self, instruction: Instruction, stack : &mut Stack<U256>) -> Result<(), evm::Error> {
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
				let (c, _overflow) = a.overflowing_add(b);
				stack.push(c);
			},
			instructions::MUL => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				let (c, _overflow) = a.overflowing_mul(b);
				stack.push(c);
			},
			instructions::SUB => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				let (c, _overflow) = a.overflowing_sub(b);
				stack.push(c);
			},
			instructions::DIV => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(if !self.is_zero(&b) {
					let (c, _overflow) = a.overflowing_div(b);
					c
				} else {
					U256::zero()
				});
			},
			instructions::MOD => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(if !self.is_zero(&b) {
					let (c, _overflow) = a.overflowing_rem(b);
					c
				} else {
					U256::zero()
				});
			},
			instructions::SDIV => {
				let (a, sign_a) = get_and_reset_sign(stack.pop_back());
				let (b, sign_b) = get_and_reset_sign(stack.pop_back());

				// -2^255
				let min = U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
				stack.push(if self.is_zero(&b) {
					U256::zero()
				} else if a == min && b == !U256::zero() {
					min
				} else {
					let (c, _overflow) = a.overflowing_div(b);
					set_sign(c, sign_a ^ sign_b)
				});
			},
			instructions::SMOD => {
				let ua = stack.pop_back();
				let ub = stack.pop_back();
				let (a, sign_a) = get_and_reset_sign(ua);
				let (b, _sign_b) = get_and_reset_sign(ub);

				stack.push(if !self.is_zero(&b) {
					let (c, _overflow) = a.overflowing_rem(b);
					set_sign(c, sign_a)
				} else {
					U256::zero()
				});
			},
			instructions::EXP => {
				let base = stack.pop_back();
				let expon = stack.pop_back();
				let (res, _overflow) = base.overflowing_pow(expon);
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

				let is_positive_lt = a < b && (neg_a | neg_b) == false;
				let is_negative_lt = a > b && (neg_a & neg_b) == true;
				let has_different_signs = neg_a == true && neg_b == false;

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

				let is_positive_gt = a > b && (neg_a | neg_b) == false;
				let is_negative_gt = a < b && (neg_a & neg_b) == true;
				let has_different_signs = neg_a == false && neg_b == true;

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
				let byte = if word < U256::from(32) {
					(val >> (8 * (31 - word.low_u64() as usize))) & U256::from(0xff)
				} else {
					U256::zero()
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
					let (res, _overflow) = a5.overflowing_add(U512::from(b));
					let (x, _overflow) = res.overflowing_rem(U512::from(c));
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
					let (res, _overflow) = a5.overflowing_mul(U512::from(b));
					let (x, _overflow) = res.overflowing_rem(U512::from(c));
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

	fn find_jump_destinations(&self, code : &Bytes) -> HashSet<CodePosition> {
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

		return jump_dests;
	}

}

fn get_and_reset_sign(value: U256) -> (U256, bool) {
	let sign = (value >> 255).low_u64() == 1;
	(set_sign(value, sign), sign)
}

pub fn set_sign(value: U256, sign: bool) -> U256 {
	if sign {
		let (val, _overflow) = (!U256::zero() ^ value).overflowing_add(U256::one());
		val
	} else {
		value
	}
}

fn add_u256_usize(value: &U256, num: usize) -> (U256, bool) {
	value.clone().overflowing_add(U256::from(num))
}

fn u256_to_address(value: &U256) -> Address {
	Address::from(H256::from(value))
}

fn address_to_u256(value: Address) -> U256 {
	U256::from(H256::from(value).as_slice())
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
		let (mem_cost, mem_size) = interpreter.mem_gas_cost(&schedule, current_mem_size, &mem_size);

		// then
		assert_eq!(mem_cost, U256::from(3));
		assert_eq!(mem_size, 32);
	}

	#[test]
	fn test_memory_read_and_write() {
		// given
		let mem : &mut super::Memory = &mut vec![];
		mem.resize(0x80 + 32);

		// when
		mem.write(U256::from(0x80), U256::from(0xabcdef));

		// then
		assert_eq!(mem.read(U256::from(0x80)), U256::from(0xabcdef));
	}

	#[test]
	fn test_memory_read_and_write_byte() {
		// given
		let mem : &mut super::Memory = &mut vec![];
		mem.resize(32);

		// when
		mem.write_byte(U256::from(0x1d), U256::from(0xab));
		mem.write_byte(U256::from(0x1e), U256::from(0xcd));
		mem.write_byte(U256::from(0x1f), U256::from(0xef));

		// then
		assert_eq!(mem.read(U256::from(0x00)), U256::from(0xabcdef));
	}
}
