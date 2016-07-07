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
use evm::{self, CostType};
use evm::instructions::{self, Instruction, InstructionInfo};
use evm::interpreter::stack::Stack;

macro_rules! overflowing {
	($x: expr) => {{
		let (v, overflow) = $x;
		if overflow { return Err(evm::Error::OutOfGas); }
		v
	}}
}

#[cfg_attr(feature="dev", allow(enum_variant_names))]
enum InstructionCost<Cost: CostType> {
	Gas(Cost),
	GasMem(Cost, Cost),
	GasMemCopy(Cost, Cost, Cost)
}

pub struct Gasometer<Gas: CostType> {
	pub current_gas: Gas,
}

impl<Gas: CostType> Gasometer<Gas> {

	pub fn new(current_gas: Gas) -> Self {
		Gasometer {
			current_gas: current_gas,
		}
	}

	pub fn verify_gas(&self, gas_cost: &Gas) -> evm::Result<()> {
		match &self.current_gas < gas_cost {
			true => Err(evm::Error::OutOfGas),
			false => Ok(())
		}
	}

	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	pub fn get_gas_cost_mem(
		&mut self,
		ext: &evm::Ext,
		instruction: Instruction,
		info: &InstructionInfo,
		stack: &Stack<U256>,
		current_mem_size: usize,
	) -> evm::Result<(Gas, usize)> {
		let schedule = ext.schedule();
		let tier = instructions::get_tier_idx(info.tier);
		let default_gas = Gas::from(schedule.tier_step_gas[tier]);

		let cost = match instruction {
			instructions::SSTORE => {
				let address = H256::from(stack.peek(0));
				let newval = stack.peek(1);
				let val = U256::from(ext.storage_at(&address).as_slice());

				let gas = if U256::zero() == val && &U256::zero() != newval {
					schedule.sstore_set_gas
				} else {
					// Refund for below case is added when actually executing sstore
					// !self.is_zero(&val) && self.is_zero(newval)
					schedule.sstore_reset_gas
				};
				InstructionCost::Gas(Gas::from(gas))
			},
			instructions::SLOAD => {
				InstructionCost::Gas(Gas::from(schedule.sload_gas))
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
				let w = overflowing!(add_gas_usize(try!(Gas::from_u256(*stack.peek(1))), 31));
				let words = w >> 5;
				let gas = Gas::from(schedule.sha3_gas) + (Gas::from(schedule.sha3_word_gas) * words);
				InstructionCost::GasMem(gas, try!(self.mem_needed(stack.peek(0), stack.peek(1))))
			},
			instructions::CALLDATACOPY | instructions::CODECOPY => {
				InstructionCost::GasMemCopy(default_gas, try!(self.mem_needed(stack.peek(0), stack.peek(2))), try!(Gas::from_u256(*stack.peek(2))))
			},
			instructions::EXTCODECOPY => {
				InstructionCost::GasMemCopy(default_gas, try!(self.mem_needed(stack.peek(1), stack.peek(3))), try!(Gas::from_u256(*stack.peek(3))))
			},
			instructions::JUMPDEST => {
				InstructionCost::Gas(Gas::from(1))
			},
			instructions::LOG0...instructions::LOG4 => {
				let no_of_topics = instructions::get_log_topics(instruction);
				let log_gas = schedule.log_gas + schedule.log_topic_gas * no_of_topics;

				let data_gas = overflowing!(try!(Gas::from_u256(*stack.peek(1))).overflow_mul(Gas::from(schedule.log_data_gas)));
				let gas = overflowing!(data_gas.overflow_add(Gas::from(log_gas)));
				InstructionCost::GasMem(gas, try!(self.mem_needed(stack.peek(0), stack.peek(1))))
			},
			instructions::CALL | instructions::CALLCODE => {
				let mut gas  = overflowing!(add_gas_usize(try!(Gas::from_u256(*stack.peek(0))), schedule.call_gas));
				let mem = cmp::max(
					try!(self.mem_needed(stack.peek(5), stack.peek(6))),
					try!(self.mem_needed(stack.peek(3), stack.peek(4)))
				);

				let address = u256_to_address(stack.peek(1));

				if instruction == instructions::CALL && !ext.exists(&address) {
					gas = overflowing!(gas.overflow_add(Gas::from(schedule.call_new_account_gas)));
				};

				if stack.peek(2) > &U256::zero() {
					gas = overflowing!(gas.overflow_add(Gas::from(schedule.call_value_transfer_gas)));
				};

				InstructionCost::GasMem(gas,mem)
			},
			instructions::DELEGATECALL => {
				let gas = overflowing!(add_gas_usize(try!(Gas::from_u256(*stack.peek(0))), schedule.call_gas));
				let mem = cmp::max(
					try!(self.mem_needed(stack.peek(4), stack.peek(5))),
					try!(self.mem_needed(stack.peek(2), stack.peek(3)))
				);
				InstructionCost::GasMem(gas, mem)
			},
			instructions::CREATE => {
				let gas = Gas::from(schedule.create_gas);
				let mem = try!(self.mem_needed(stack.peek(1), stack.peek(2)));
				InstructionCost::GasMem(gas, mem)
			},
			instructions::EXP => {
				let expon = stack.peek(1);
				let bytes = ((expon.bits() + 7) / 8) as usize;
				let gas = Gas::from(schedule.exp_gas + schedule.exp_byte_gas * bytes);
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
				let gas = overflowing!(gas.overflow_add(mem_gas));
				Ok((gas, new_mem_size))
			},
			InstructionCost::GasMemCopy(gas, mem_size, copy) => {
				let (mem_gas, new_mem_size) = try!(self.mem_gas_cost(schedule, current_mem_size, &mem_size));
				let copy = overflowing!(add_gas_usize(copy, 31));
				let copy_gas = Gas::from(schedule.copy_gas) * (copy / Gas::from(32 as usize));
				let gas = overflowing!(gas.overflow_add(copy_gas));
				let gas = overflowing!(gas.overflow_add(mem_gas));
				Ok((gas, new_mem_size))
			}
		}
	}

	fn is_zero(&self, val: &Gas) -> bool {
		&Gas::from(0) == val
	}

	fn mem_needed_const(&self, mem: &U256, add: usize) -> evm::Result<Gas> {
		Gas::from_u256(overflowing!(mem.overflowing_add(U256::from(add))))
	}

	fn mem_needed(&self, offset: &U256, size: &U256) -> evm::Result<Gas> {
		if self.is_zero(&try!(Gas::from_u256(*size))) {
			return Ok(Gas::from(0));
		}

		Gas::from_u256(overflowing!(offset.overflowing_add(*size)))
	}

	fn mem_gas_cost(&self, schedule: &evm::Schedule, current_mem_size: usize, mem_size: &Gas) -> evm::Result<(Gas, usize)> {
		let gas_for_mem = |mem_size: Gas| {
			let s = mem_size >> 5;
			// s * memory_gas + s * s / quad_coeff_div
			let a = overflowing!(s.overflow_mul(Gas::from(schedule.memory_gas)));
			// We need to go to U512 to calculate s*s/quad_coeff_div
			let b = U512::from(s.as_u256()) * U512::from(s.as_u256()) / U512::from(schedule.quad_coeff_div);
			if b > U512::from(!U256::zero()) {
				Err(evm::Error::OutOfGas)
			} else {
				Ok(overflowing!(a.overflow_add(try!(Gas::from_u256(U256::from(b))))))
			}
		};
		let current_mem_size = Gas::from(current_mem_size);
		let req_mem_size_rounded = (overflowing!(mem_size.overflow_add(Gas::from(31 as usize))) >> 5) << 5;

		let mem_gas_cost = if req_mem_size_rounded > current_mem_size {
			let new_mem_gas = try!(gas_for_mem(req_mem_size_rounded));
			let current_mem_gas = try!(gas_for_mem(current_mem_size));
			new_mem_gas - current_mem_gas
		} else {
			Gas::from(0)
		};

		Ok((mem_gas_cost, req_mem_size_rounded.as_usize()))
	}
}

#[inline]
fn add_gas_usize<Gas: CostType>(value: Gas, num: usize) -> (Gas, bool) {
	value.overflow_add(Gas::from(num))
}

#[test]
fn test_mem_gas_cost() {
	// given
	let gasometer = Gasometer::<U256>::new(U256::zero());
	let schedule = evm::Schedule::default();
	let current_mem_size = 5;
	let mem_size = !U256::zero();

	// when
	let result = gasometer.mem_gas_cost(&schedule, current_mem_size, &mem_size);

	// then
	if let Ok(_) = result {
		assert!(false, "Should fail with OutOfGas");
	}
}

#[test]
fn test_calculate_mem_cost() {
	// given
	let gasometer = Gasometer::<usize>::new(0);
	let schedule = evm::Schedule::default();
	let current_mem_size = 0;
	let mem_size = 5;

	// when
	let (mem_cost, mem_size) = gasometer.mem_gas_cost(&schedule, current_mem_size, &mem_size).unwrap();

	// then
	assert_eq!(mem_cost, 3);
	assert_eq!(mem_size, 32);
}
