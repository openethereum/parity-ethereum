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

use common::*;
use super::u256_to_address;
use evm;
use evm::instructions::{self, Instruction};
use evm::interpreter::stack::Stack;

macro_rules! overflowing {
	($x: expr) => {{
		let (v, overflow) = $x;
		if overflow { return Err(evm::Error::OutOfGas); }
		v
	}}
}

#[cfg_attr(feature="dev", allow(enum_variant_names))]
enum InstructionCost {
	Gas(U256),
	GasMem(U256, U256),
	GasMemCopy(U256, U256, U256)
}

pub trait Gasometer {
	type Gas;

	fn new(current_gas: Self::Gas) -> Self;

	fn current_gas(&self) -> Self::Gas;

	fn subtract_gas(&mut self, gas_cost: Self::Gas);

	fn add_gas(&mut self, gas: Self::Gas);

	fn set_gas(&mut self, gas: Self::Gas);

	fn verify_gas(&self, gas_cost: &Self::Gas) -> evm::Result<()>;

	fn get_gas_cost_mem(
		&mut self,
		ext: &evm::Ext,
		instruction: Instruction,
		stack: &Stack<U256>,
		current_mem_size: usize,
	) -> evm::Result<(Self::Gas, usize)>;

}

pub struct GasometerU256 {
	pub current_gas: U256,
}

impl Gasometer for GasometerU256 {
	type Gas = U256;

	fn new(current_gas: Self::Gas) -> Self {
		GasometerU256 {
			current_gas: current_gas,
		}
	}

	fn current_gas(&self) -> Self::Gas {
		self.current_gas
	}

	fn subtract_gas(&mut self, gas_cost: Self::Gas) {
		self.current_gas = self.current_gas - gas_cost;
	}

	fn add_gas(&mut self, gas: Self::Gas) {
		self.current_gas = self.current_gas + gas;
	}

	fn set_gas(&mut self, gas: Self::Gas) {
		self.current_gas = gas;
	}

	fn verify_gas(&self, gas_cost: &U256) -> evm::Result<()> {
		match &self.current_gas < gas_cost {
			true => Err(evm::Error::OutOfGas),
			false => Ok(())
		}
	}

	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn get_gas_cost_mem(
		&mut self,
		ext: &evm::Ext,
		instruction: Instruction,
		stack: &Stack<U256>,
		current_mem_size: usize,
	) -> evm::Result<(Self::Gas, usize)> {
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
				let (mem_gas, new_mem_size) = try!(self.mem_gas_cost(schedule, current_mem_size, &mem_size));
				let gas = overflowing!(gas.overflowing_add(mem_gas));
				Ok((gas, new_mem_size))
			},
			InstructionCost::GasMemCopy(gas, mem_size, copy) => {
				let (mem_gas, new_mem_size) = try!(self.mem_gas_cost(schedule, current_mem_size, &mem_size));
				let copy = overflowing!(add_u256_usize(&copy, 31));
				let copy_gas = U256::from(schedule.copy_gas) * (copy / U256::from(32));
				let gas = overflowing!(gas.overflowing_add(copy_gas));
				let gas = overflowing!(gas.overflowing_add(mem_gas));
				Ok((gas, new_mem_size))
			}
		}
	}
}

impl GasometerU256 {

	fn verify_instructions_requirements(
		&self,
		info: &instructions::InstructionInfo,
		stack_limit: usize,
		stack: &Stack<U256>
	) -> evm::Result<()> {
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

	fn is_zero(&self, val: &U256) -> bool {
		&U256::zero() == val
	}

	fn mem_needed_const(&self, mem: &U256, add: usize) -> evm::Result<U256> {
		Ok(overflowing!(mem.overflowing_add(U256::from(add))))
	}

	fn mem_needed(&self, offset: &U256, size: &U256) -> evm::Result<U256> {
		if self.is_zero(size) {
			return Ok(U256::zero());
		}

		Ok(overflowing!(offset.overflowing_add(size.clone())))
	}

	fn mem_gas_cost(&self, schedule: &evm::Schedule, current_mem_size: usize, mem_size: &U256) -> evm::Result<(U256, usize)> {
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

}

#[inline]
fn add_u256_usize(value: &U256, num: usize) -> (U256, bool) {
	value.clone().overflowing_add(U256::from(num))
}

#[test]
fn test_mem_gas_cost() {
	// given
	let interpreter = Interpreter::default();
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

#[test]
fn test_calculate_mem_cost() {
	// given
	let gasometer = Gasometer::new(U256::zero());
	let schedule = evm::Schedule::default();
	let current_mem_size = 0;
	let mem_size = U256::from(5);

	// when
	let (mem_cost, mem_size) = gasometer.mem_gas_cost(&schedule, current_mem_size, &mem_size).unwrap();

	// then
	assert_eq!(mem_cost, U256::from(3));
	assert_eq!(mem_size, 32);
}
