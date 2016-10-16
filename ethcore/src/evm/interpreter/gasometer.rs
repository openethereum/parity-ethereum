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
use evm::schedule::Schedule;

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
	GasMem(Cost, Cost, Option<Cost>),
	GasMemCopy(Cost, Cost, Cost)
}

pub struct Gasometer<Gas: CostType> {
	pub current_gas: Gas,
	pub current_mem_gas: Gas,
}

impl<Gas: CostType> Gasometer<Gas> {

	pub fn new(current_gas: Gas) -> Self {
		Gasometer {
			current_gas: current_gas,
			current_mem_gas: Gas::from(0),
		}
	}

	pub fn verify_gas(&self, gas_cost: &Gas) -> evm::Result<()> {
		match &self.current_gas < gas_cost {
			true => Err(evm::Error::OutOfGas),
			false => Ok(())
		}
	}

	/// How much gas is provided to a CALL/CREATE, given that we need to deduct `needed` for this operation
	/// and that we `requested` some.
	pub fn gas_provided(&self, schedule: &Schedule, needed: Gas, requested: Option<evm::Result<Gas>>) -> evm::Result<Gas> {
		match schedule.sub_gas_cap_divisor {
			Some(cap_divisor) if self.current_gas >= needed => {
				let gas_remaining = self.current_gas - needed;
				let max_gas_provided = gas_remaining - gas_remaining / Gas::from(cap_divisor);
				if let Some(Ok(r)) = requested { 
					Ok(min(r, max_gas_provided))
				} else {
					Ok(max_gas_provided)
				}
			},
			_ => {
				if let Some(r) = requested { 
					r
				} else if self.current_gas >= needed {
					Ok(self.current_gas - needed)
				} else {
					Ok(0.into())
				}
			}
		}
	}

	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	/// Determine how much gas is used by the given instruction, given the machine's state.
	/// 
	/// We guarantee that the final element of the returned tuple (`provided`) will be `Some`
	/// iff the `instruction` is one of `CREATE`, or any of the `CALL` variants. In this case,
	/// it will be the amount of gas that the current context provides to the child context. 
	pub fn get_gas_cost_mem(
		&mut self,
		ext: &evm::Ext,
		instruction: Instruction,
		info: &InstructionInfo,
		stack: &Stack<U256>,
		current_mem_size: usize,
	) -> evm::Result<(Gas, Gas, usize, Option<Gas>)> {
		let schedule = ext.schedule();
		let tier = instructions::get_tier_idx(info.tier);
		let default_gas = Gas::from(schedule.tier_step_gas[tier]);

		let cost = match instruction {
			instructions::JUMPDEST => {
				InstructionCost::Gas(Gas::from(1))
			},
			instructions::SSTORE => {
				let address = H256::from(stack.peek(0));
				let newval = stack.peek(1);
				let val = U256::from(&*ext.storage_at(&address));

				let gas = if val.is_zero() && !newval.is_zero() {
					schedule.sstore_set_gas
				} else {
					// Refund for below case is added when actually executing sstore
					// !is_zero(&val) && is_zero(newval)
					schedule.sstore_reset_gas
				};
				InstructionCost::Gas(Gas::from(gas))
			},
			instructions::SLOAD => {
				InstructionCost::Gas(Gas::from(schedule.sload_gas))
			},
			instructions::BALANCE => {
				InstructionCost::Gas(Gas::from(schedule.balance_gas))
			},
			instructions::EXTCODESIZE => {
				InstructionCost::Gas(Gas::from(schedule.extcodesize_gas))
			},
			instructions::SUICIDE => {
				let mut gas = Gas::from(schedule.suicide_gas);

				let address = u256_to_address(stack.peek(0));
				if !ext.exists(&address) {
					gas = overflowing!(gas.overflow_add(schedule.suicide_to_new_account_cost.into()));
				}

				InstructionCost::Gas(gas)
			},
			instructions::MSTORE | instructions::MLOAD => {
				InstructionCost::GasMem(default_gas, try!(mem_needed_const(stack.peek(0), 32)), None)
			},
			instructions::MSTORE8 => {
				InstructionCost::GasMem(default_gas, try!(mem_needed_const(stack.peek(0), 1)), None)
			},
			instructions::RETURN => {
				InstructionCost::GasMem(default_gas, try!(mem_needed(stack.peek(0), stack.peek(1))), None)
			},
			instructions::SHA3 => {
				let w = overflowing!(add_gas_usize(try!(Gas::from_u256(*stack.peek(1))), 31));
				let words = w >> 5;
				let gas = Gas::from(schedule.sha3_gas) + (Gas::from(schedule.sha3_word_gas) * words);
				InstructionCost::GasMem(gas, try!(mem_needed(stack.peek(0), stack.peek(1))), None)
			},
			instructions::CALLDATACOPY | instructions::CODECOPY => {
				InstructionCost::GasMemCopy(default_gas, try!(mem_needed(stack.peek(0), stack.peek(2))), try!(Gas::from_u256(*stack.peek(2))))
			},
			instructions::EXTCODECOPY => {
				InstructionCost::GasMemCopy(schedule.extcodecopy_base_gas.into(), try!(mem_needed(stack.peek(1), stack.peek(3))), try!(Gas::from_u256(*stack.peek(3))))
			},
			instructions::LOG0...instructions::LOG4 => {
				let no_of_topics = instructions::get_log_topics(instruction);
				let log_gas = schedule.log_gas + schedule.log_topic_gas * no_of_topics;

				let data_gas = overflowing!(try!(Gas::from_u256(*stack.peek(1))).overflow_mul(Gas::from(schedule.log_data_gas)));
				let gas = overflowing!(data_gas.overflow_add(Gas::from(log_gas)));
				InstructionCost::GasMem(gas, try!(mem_needed(stack.peek(0), stack.peek(1))), None)
			},
			instructions::CALL | instructions::CALLCODE => {
				let mut gas = Gas::from(schedule.call_gas);
				let mem = cmp::max(
					try!(mem_needed(stack.peek(5), stack.peek(6))),
					try!(mem_needed(stack.peek(3), stack.peek(4)))
				);

				let address = u256_to_address(stack.peek(1));

				if instruction == instructions::CALL && !ext.exists(&address) {
					gas = overflowing!(gas.overflow_add(schedule.call_new_account_gas.into()));
				};

				if !stack.peek(2).is_zero() {
					gas = overflowing!(gas.overflow_add(schedule.call_value_transfer_gas.into()));
				};

				// TODO: refactor to avoid duplicate calculation here and later on. 
				let (mem_gas_cost, _, _) = try!(self.mem_gas_cost(schedule, current_mem_size, &mem));
				let cost_so_far = overflowing!(gas.overflow_add(mem_gas_cost.into()));
				let requested = Gas::from_u256(*stack.peek(0));
				let provided = try!(self.gas_provided(schedule, cost_so_far, Some(requested)));
				gas = overflowing!(gas.overflow_add(provided));

				InstructionCost::GasMem(gas, mem, Some(provided))
			},
			instructions::DELEGATECALL => {
				let mut gas = Gas::from(schedule.call_gas);
				let mem = cmp::max(
					try!(mem_needed(stack.peek(4), stack.peek(5))),
					try!(mem_needed(stack.peek(2), stack.peek(3)))
				);

				// TODO: refactor to avoid duplicate calculation here and later on. 
				let (mem_gas_cost, _, _) = try!(self.mem_gas_cost(schedule, current_mem_size, &mem));
				let cost_so_far = overflowing!(gas.overflow_add(mem_gas_cost.into()));
				let requested = Gas::from_u256(*stack.peek(0));
				let provided = try!(self.gas_provided(schedule, cost_so_far, Some(requested)));
				gas = overflowing!(gas.overflow_add(provided));

				InstructionCost::GasMem(gas, mem, Some(provided))
			},
			instructions::CREATE => {
				let mut gas = Gas::from(schedule.create_gas);
				let mem = try!(mem_needed(stack.peek(1), stack.peek(2)));

				// TODO: refactor to avoid duplicate calculation here and later on. 
				let (mem_gas_cost, _, _) = try!(self.mem_gas_cost(schedule, current_mem_size, &mem));
				let cost_so_far = overflowing!(gas.overflow_add(mem_gas_cost.into())); 
				let provided = try!(self.gas_provided(schedule, cost_so_far, None));
				gas = overflowing!(gas.overflow_add(provided));

				InstructionCost::GasMem(gas, mem, Some(provided))
			},
			instructions::EXP => {
				let expon = stack.peek(1);
				let bytes = ((expon.bits() + 7) / 8) as usize;
				let gas = Gas::from(schedule.exp_gas + schedule.exp_byte_gas * bytes);
				InstructionCost::Gas(gas)
			},
			_ => InstructionCost::Gas(default_gas),
		};

		match cost {
			InstructionCost::Gas(gas) => {
				Ok((gas, self.current_mem_gas, 0, None))
			},
			InstructionCost::GasMem(gas, mem_size, provided) => {
				let (mem_gas_cost, new_mem_gas, new_mem_size) = try!(self.mem_gas_cost(schedule, current_mem_size, &mem_size));
				let gas = overflowing!(gas.overflow_add(mem_gas_cost));
				Ok((gas, new_mem_gas, new_mem_size, provided))
			},
			InstructionCost::GasMemCopy(gas, mem_size, copy) => {
				let (mem_gas_cost, new_mem_gas, new_mem_size) = try!(self.mem_gas_cost(schedule, current_mem_size, &mem_size));
				let copy = overflowing!(add_gas_usize(copy, 31)) >> 5;
				let copy_gas = Gas::from(schedule.copy_gas) * copy;
				let gas = overflowing!(gas.overflow_add(copy_gas));
				let gas = overflowing!(gas.overflow_add(mem_gas_cost));
				Ok((gas, new_mem_gas, new_mem_size, None))
			}
		}
	}

	fn mem_gas_cost(&self, schedule: &evm::Schedule, current_mem_size: usize, mem_size: &Gas) -> evm::Result<(Gas, Gas, usize)> {
		let gas_for_mem = |mem_size: Gas| {
			let s = mem_size >> 5;
			// s * memory_gas + s * s / quad_coeff_div
			let a = overflowing!(s.overflow_mul(Gas::from(schedule.memory_gas)));

			// Calculate s*s/quad_coeff_div
			debug_assert_eq!(schedule.quad_coeff_div, 512);
			let b = overflowing!(s.overflow_mul_shr(s, 9));
			Ok(overflowing!(a.overflow_add(b)))
		};

		let current_mem_size = Gas::from(current_mem_size);
		let req_mem_size_rounded = (overflowing!(mem_size.overflow_add(Gas::from(31 as usize))) >> 5) << 5;

		let (mem_gas_cost, new_mem_gas) = if req_mem_size_rounded > current_mem_size {
			let new_mem_gas = try!(gas_for_mem(req_mem_size_rounded));
			(new_mem_gas - self.current_mem_gas, new_mem_gas)
		} else {
			(Gas::from(0), self.current_mem_gas)
		};

		Ok((mem_gas_cost, new_mem_gas, req_mem_size_rounded.as_usize()))
	}
}


#[inline]
fn mem_needed_const<Gas: CostType>(mem: &U256, add: usize) -> evm::Result<Gas> {
	Gas::from_u256(overflowing!(mem.overflowing_add(U256::from(add))))
}

#[inline]
fn mem_needed<Gas: CostType>(offset: &U256, size: &U256) -> evm::Result<Gas> {
	if size.is_zero() {
		return Ok(Gas::from(0));
	}

	Gas::from_u256(overflowing!(offset.overflowing_add(*size)))
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
	let (mem_cost, new_mem_gas, mem_size) = gasometer.mem_gas_cost(&schedule, current_mem_size, &mem_size).unwrap();

	// then
	assert_eq!(mem_cost, 3);
	assert_eq!(new_mem_gas, 3);
	assert_eq!(mem_size, 32);
}
