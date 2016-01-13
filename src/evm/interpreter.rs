///! Rust VM implementation

use common::*;
use evm;
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
		println!("Pushing to stack: {}", elem);
		self.push(elem);
	}

	fn size(&self) -> usize {
		self.len()
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
		let mut value = U256::from(self.code[self.position]);

		for off in 1..no_of_bytes {
			let pos = self.position + off;
			value = value << 8;
			// TODO [todr] Directly bitor with u8?
			value = value | U256::from(self.code[pos]);
		}

		// Move PC
		self.position += no_of_bytes;
		value
	}

	fn len (&self) -> usize {
		self.code.len()
	}

	// TODO [todr] All get_slice should operate on memory not code!!!
	/// Retrieve part of the code described by offset and size
	fn get_slice(&self, init_off_u: U256, init_size_u: U256) -> &[u8] {
		let init_off = init_off_u.low_u64() as usize;
		let init_size = init_size_u.low_u64() as usize;
		&self.code[self.position + init_off..self.position + init_off + init_size]
	}
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
    let code = &params.code;
    let valid_jump_destinations = self.find_jump_destinations(&code);

    // TODO reserve stack
		let mut current_gas = params.gas.clone();
    let mut stack = vec![];
		let mut reader = CodeReader {
			position: 0,
			code: &code
		};

    while reader.position < code.len() {
      let instruction = code[reader.position];
			reader.position += 1;

			// Calculate gas cost
			let gas_cost = try!(self.get_gas_cost(current_gas, params, ext, instruction, &stack));
			try!(self.verify_gas(&current_gas, &gas_cost));
			current_gas = current_gas - gas_cost;

			// Execute instruction
			let result = try!(self.exec_instruction(
					current_gas, params, ext, instruction, &mut reader, &mut stack
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

	fn get_gas_cost(&self,
									gas: Gas,
									params: &ActionParams,
									ext: &evm::Ext,
									instruction: Instruction,
									stack: &Stack<U256>
									) -> evm::Result {

			let schedule = ext.schedule();
			let info = instructions::get_info(instruction);

			if !schedule.have_delegate_call && instruction == instructions::DELEGATECALL {
				return Err(evm::Error::BadInstruction);
			}
			if info.tier == instructions::GasPriceTier::InvalidTier {
				return Err(evm::Error::BadInstruction);
			}

			try!(self.verify_instructions_requirements(&info, schedule.stack_limit, stack));

			let tier = instructions::get_tier_idx(info.tier);
			let run_gas = schedule.tier_step_gas[tier];


			Ok(Gas::from(run_gas))
	}

	fn verify_instructions_requirements(&self, 
																			info: &instructions::InstructionInfo, 
																			stack_limit: usize, 
																			stack: &Stack<U256>) -> Result<(), evm::Error> {
		if !stack.has(info.args) {
			Err(evm::Error::StackUnderflow(info.args, stack.size()))
		} else if stack.size() - info.args + info.ret > stack_limit {
			Err(evm::Error::OutOfStack(info.ret - info.args, stack_limit))
		} else {
			Ok(())
		}
	}

	fn exec_instruction(&self,
											gas: Gas,
											params: &ActionParams,
											ext: &mut evm::Ext,
											instruction: Instruction,
											code: &mut CodeReader, 
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
				let condition = stack.pop_back();
				let jump = stack.pop_back();
				if !self.is_zero(condition) {
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

				// TODO [todr] Fix u64 for gas
				let contract_code = code.get_slice(init_off, init_size);
				// TODO [todr] Fix u64 for gasLeft
				let (gas_left, maybe_address) = try!(
					ext.create(gas.low_u64(), &endowment, &contract_code)
				);
				match maybe_address {
					Some(address) => stack.push(address_to_u256(address)),
					None => stack.push(U256::zero())
				}
				return Ok(InstructionResult::AdditionalGasCost(
					gas - Gas::from(gas_left)
				));
			},
			// CALL, CALLCODE, DELEGATECALL
			instructions::RETURN => {
				let init_off = stack.pop_back();
				let init_size = stack.pop_back();
				let return_code = code.get_slice(init_off, init_size);
				// TODO [todr] Fix u64 for gas
				let gas_left = try!(ext.ret(gas.low_u64(), &return_code));
				// TODO [todr] Fix u64 for gasLeft
				return Ok(InstructionResult::StopExecutionWithGasCost(
					gas - Gas::from(gas_left)
				));
			},
			instructions::STOP => {
				return Ok(InstructionResult::StopExecution);
			},
			instructions::SUICIDE => {
				// TODO [todr] Suicide should have argument with address of contract that funds should be transfered to
				let address = stack.pop_back();
				// ext.suicide(Address::from(address));
				ext.suicide();
				return Ok(InstructionResult::StopExecution);
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
			},
			instructions::PUSH1...instructions::PUSH32 => {
				// Load to stack
				let bytes = instructions::get_push_bytes(instruction);
				// TODO [todr] move positions management outside of CodeReader
				let val = code.read(bytes);
				stack.push(val);
			},
			instructions::MLOAD => {
				// TODO [ToDr] load word from mem?
			},
			instructions::MSTORE => {
				// TODO [ToDr] save word to mem?
			},
			instructions::MSTORE8 => {
				// TODO [ToDr] save byte to mem?
			},
			instructions::MSIZE => {
				// Size of memry to stack
			},
			instructions::SHA3 => {
				let offset = stack.pop_back();
				let size = stack.pop_back();
				let sha3 = code.get_slice(offset, size).sha3();
				stack.push(U256::from(sha3.as_slice()));
			},
			instructions::SLOAD => {
				let key = H256::from(&stack.pop_back());
				let word = U256::from(ext.sload(&key).as_slice());
				stack.push(word);
			},
			instructions::SSTORE => {
				let key = H256::from(&stack.pop_back());
				let word = H256::from(&stack.pop_back());
				ext.sstore(key, word);
			},
			instructions::PC => {
				stack.push(U256::from(code.position));
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
			// instructions::CALLDATALOAD
			instructions::CALLDATASIZE => {
				stack.push(U256::from(params.data.len()));
			},
			instructions::CODESIZE => {
				stack.push(U256::from(code.len()));
			},
			instructions::EXTCODESIZE => {
				let address = u256_to_address(&stack.pop_back());
				let len = ext.extcode(&address).len();
				stack.push(U256::from(len));
			},
			// instructions::CALLDATACOPY => {},
			// instructions::CODECOPY => {},
			// instructions::EXTCODECOPY => {
			// 	let address = u256_to_addres(&stack.pop_back());
			// 	let code = ext.extcode(address);
			// },
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
				self.exec_stack_instruction(instruction, stack);
			}
		};
		Ok(InstructionResult::AdditionalGasCost(U256::zero()))
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

		if valid_jump_destinations.contains(&jump) {
			Ok(jump)
		} else {
			Err(evm::Error::BadJumpDestination)
		}
	}

	fn is_zero(&self, val: U256) -> bool {
		U256::zero() == val
	}

	fn bool_to_u256(&self, val: bool) -> U256 {
		if val {
			U256::one()
		} else {
			U256::zero()
		}
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
			instructions::POP => {
				stack.pop_back();
			},
			instructions::ADD => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a + b);
			},
			instructions::MUL => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a * b);
			},
			instructions::SUB => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(a - b);
			},
			instructions::DIV => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(if self.is_zero(b) {
					a / b 
				} else {
					U256::zero()
				});
			},
			instructions::MOD => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(if self.is_zero(b) {
					a % b 
				} else {
					U256::zero()
				});
			},
			// instructions::SDIV => {},
			// instructions::SMOD => {},
			// instructions::EXP => {},
			instructions::NOT => {
				let a = stack.pop_back();
				stack.push(!a);
			},
			instructions::LT => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(self.bool_to_u256(a < b));
			},
			// instructions::SLT => {},
			instructions::GT => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(self.bool_to_u256(a > b));
			},
			// instructions::SGT => {},
			instructions::EQ => {
				let a = stack.pop_back();
				let b = stack.pop_back();
				stack.push(self.bool_to_u256(a == b));
			},
			instructions::ISZERO => {
				let a = stack.pop_back();
				stack.push(self.bool_to_u256(self.is_zero(a)));
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
			// instructions::BYTE => {},
			// instructions::ADDMOD => {},
			// instructions::MULMOD => {},
			// instructions::SIGNEXTEND => {},
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
