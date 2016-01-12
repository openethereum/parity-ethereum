///! Rust VM implementation

use common::*;
use evm;
use super::schedule::Schedule;
use super::instructions as instructions;
use super::instructions::Instruction;

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
}
impl<S> Stack<S> for Vec<S> {
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
			Some(x) => x,
			None => panic!("Tried to pop from empty stack.")
		}
	}

	fn pop_n(&mut self, no_of_elems: usize) -> Vec<S> {
		let mut vec = Vec::new();
		for i in 1..no_of_elems {
			vec.push(self.pop_back());
		}
		vec
	}


	fn push(&mut self, elem: S) {
		self.push(elem);
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
		self.position += no_of_bytes;
		// TODO [todr] READ and return something usefull
		U256::zero()
	}

	/// Retrieve part of the code described by offset and size
	fn get_slice(&self, init_off_u: U256, init_size_u: U256) -> &[u8] {
		let init_off = init_off_u.low_u64() as usize;
		let init_size = init_size_u.low_u64() as usize;
		&self.code[self.position + init_off..self.position + init_off + init_size]
	}

	/// Stop any further execution (move PC to the end)
	fn stop_execution(&mut self) {
		self.position = self.code.len();
	}
}

pub struct Interpreter;

impl evm::Evm for Interpreter {
	fn exec(&self, params: &ActionParams, ext: &mut evm::Ext) -> evm::Result {
    // TODO schedule?
    // TODO reserve stack

		// let schedule = ext.schedule();
    let code = &params.code;
    let valid_jump_destinations = self.find_jump_destinations(&code);

		let mut gas = params.gas.clone();
    let mut stack = vec![];
		let mut reader = CodeReader {
			position: 0,
			code: &code
		};

    while reader.position < code.len() {
      let instruction = code[reader.position];
			let gas_usage = self.check_and_get_gas_usage(instruction/*, schedule*/);
			// TODO check if we have enough
		
			// Handle jumps
			match instruction {
				instructions::JUMP => {
					let jump = stack.pop_back();
					reader.position = try!(self.verify_jump(jump, &valid_jump_destinations));
				},
				instructions::JUMPI => {
					let condition = stack.pop_back();
					let jump = stack.pop_back();
					if !self.is_zero(condition) {
						reader.position = try!(self.verify_jump(jump, &valid_jump_destinations));
					}
				},
				instructions::JUMPDEST => {
					// ignore
				},
				_ => {
					// Execute all other instructions
					self.exec_instruction(params, ext, gas, instruction, &mut reader, &mut stack);
				}
			}
			reader.position += 1;
		}
		Ok(gas)
  }
}

impl Interpreter {

	fn check_and_get_gas_usage(&self, instruction: Instruction/*, schedule: &Schedule*/) -> Gas {
		U256::zero()
	}

	fn exec_instruction(&self,
											params: &ActionParams,
											ext: &mut evm::Ext,
											gas: Gas,
											instruction: Instruction,
											code: &mut CodeReader, 
											stack: &mut Stack<U256>
										 ) -> evm::Result {
		match instruction {
			instructions::CREATE => {
				let endowment = stack.pop_back();
				let init_off = stack.pop_back();
				let init_size = stack.pop_back();

				// TODO [todr] Fix u64 for gas
				let contract_code = code.get_slice(init_off, init_size);
				// TODO [todr] Fix u64 for gasLeft
				let (gas_left, maybe_address) = try!(ext.create(gas.low_u64(), &endowment, &contract_code));
				match maybe_address {
					Some(address) => stack.push(U256::from(address)),
					None => stack.push(U256::zero())
				}
				Ok(U256::from(gas_left))
			},
			// CALL, CALLCODE, DELEGATECALL
			instructions::RETURN => {
				let init_off = stack.pop_back();
				let init_size = stack.pop_back();
				code.stop_execution();
				let return_code = code.get_slice(init_off, init_size);
				// TODO [todr] Fix u64 for gas
				let gas_left = try!(ext.ret(gas.low_u64(), &return_code));
				// TODO [todr] Fix u64 for gasLeft
				Ok(U256::from(gas_left))
			},
			instructions::STOP => {
				code.stop_execution();
				Ok(gas)
			},
			instructions::SUICIDE => {
				// TODO [todr] Suicide should have argument with address of contract that funds should be transfered to
				let address = stack.pop_back();
				// ext.suicide(Address::from(address));
				ext.suicide();
				code.stop_execution();
				Ok(gas)
			},
			instructions::LOG0...instructions::LOG4 => {
				let no_of_topics = instructions::get_log_topics(instruction);
				let topics_data = stack.pop_n(no_of_topics + 2);

				let offset = topics_data[0];
				let size = topics_data[1];
				let topics = topics_data
					.iter()
					.skip(2)
					.map(H256::from)
					.collect();
				ext.log(topics, code.get_slice(offset, size));
				Ok(gas)
			},
			instructions::PUSH1...instructions::PUSH32 => {
				// Load to stack
				let bytes = instructions::get_push_bytes(instruction);
				let val = code.read(bytes);
				stack.push(val);
				Ok(gas)
			},
			instructions::MLOAD => {
				// TODO [ToDr] load word from mem?
				Ok(gas)
			},
			instructions::MSTORE => {
				// TODO [ToDr] save word to mem?
				Ok(gas)
			},
			instructions::MSTORE8 => {
				// TODO [ToDr] save byte to mem?
				Ok(gas)
			},
			instructions::MSIZE => {
				// Size of memry to stack
				Ok(gas)
			},
			instructions::SLOAD => {
				let key = H256::from(&stack.pop_back());
				let word = U256::from(ext.sload(&key).as_slice());
				stack.push(word);
				Ok(gas)
			},
			instructions::SSTORE => {
				let key = H256::from(&stack.pop_back());
				let word = H256::from(&stack.pop_back());
				ext.sstore(key, word);
				Ok(gas)
			},
			instructions::PC => {
				stack.push(U256::from(code.position));
				Ok(gas)
			},
			instructions::GAS => {
				stack.push(U256::from(gas));
				Ok(gas)
			},
			_ => {
				self.exec_stack_instruction(instruction, stack);
				Ok(gas)
			}
		}
	}

	fn verify_jump(&self, jump_u: U256, valid_jump_destinations: &HashSet<usize>) -> Result<usize, evm::Error> {
		let jump = jump_u.low_u64() as usize;

		if valid_jump_destinations.contains(&jump) {
			Ok(jump)
		} else {
			Err(evm::Error::BadJumpDestination)
		}
	}

	fn is_zero(&self, val: U256) -> bool {
		U256::zero() == val
	}

	fn exec_stack_instruction(&self, instruction: Instruction, stack : &mut Stack<U256>) {
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
			instructions::ADD => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a + b);
			},
			_ => panic!(format!("Unknown stack instruction: {:x}", instruction))
		}
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

#[cfg(test)]
mod tests {
	use common::*;
	use super::*;

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
}
