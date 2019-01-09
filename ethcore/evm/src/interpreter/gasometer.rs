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

use std::cmp;
use ethereum_types::{U256, H256};
use super::u256_to_address;

use {evm, vm};
use instructions::{self, Instruction, InstructionInfo};
use interpreter::stack::Stack;
use vm::Schedule;

macro_rules! overflowing {
	($x: expr) => {{
		let (v, overflow) = $x;
		if overflow { return Err(vm::Error::OutOfGas); }
		v
	}}
}

enum Request<Cost: ::evm::CostType> {
	Gas(Cost),
	GasMem(Cost, Cost),
	GasMemProvide(Cost, Cost, Option<U256>),
	GasMemCopy(Cost, Cost, Cost)
}

pub struct InstructionRequirements<Cost> {
	pub gas_cost: Cost,
	pub provide_gas: Option<Cost>,
	pub memory_total_gas: Cost,
	pub memory_required_size: usize,
}

pub struct Gasometer<Gas> {
	pub current_gas: Gas,
	pub current_mem_gas: Gas,
}

impl<Gas: evm::CostType> Gasometer<Gas> {

	pub fn new(current_gas: Gas) -> Self {
		Gasometer {
			current_gas: current_gas,
			current_mem_gas: Gas::from(0),
		}
	}

	pub fn verify_gas(&self, gas_cost: &Gas) -> vm::Result<()> {
		match &self.current_gas < gas_cost {
			true => Err(vm::Error::OutOfGas),
			false => Ok(())
		}
	}

	/// How much gas is provided to a CALL/CREATE, given that we need to deduct `needed` for this operation
	/// and that we `requested` some.
	pub fn gas_provided(&self, schedule: &Schedule, needed: Gas, requested: Option<U256>) -> vm::Result<Gas> {
		// Try converting requested gas to `Gas` (`U256/u64`)
		// but in EIP150 even if we request more we should never fail from OOG
		let requested = requested.map(Gas::from_u256);

		match schedule.sub_gas_cap_divisor {
			Some(cap_divisor) if self.current_gas >= needed => {
				let gas_remaining = self.current_gas - needed;
				let max_gas_provided = match cap_divisor {
					64 => gas_remaining - (gas_remaining >> 6),
					cap_divisor => gas_remaining - gas_remaining / Gas::from(cap_divisor),
				};

				if let Some(Ok(r)) = requested {
					Ok(cmp::min(r, max_gas_provided))
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
			},
		}
	}

	/// Determine how much gas is used by the given instruction, given the machine's state.
	///
	/// We guarantee that the final element of the returned tuple (`provided`) will be `Some`
	/// iff the `instruction` is one of `CREATE`, or any of the `CALL` variants. In this case,
	/// it will be the amount of gas that the current context provides to the child context.
	pub fn requirements(
		&mut self,
		ext: &vm::Ext,
		instruction: Instruction,
		info: &InstructionInfo,
		stack: &Stack<U256>,
		current_mem_size: usize,
	) -> vm::Result<InstructionRequirements<Gas>> {
		let schedule = ext.schedule();
		let tier = info.tier.idx();
		let default_gas = Gas::from(schedule.tier_step_gas[tier]);

		let cost = match instruction {
			instructions::JUMPDEST => {
				Request::Gas(Gas::from(1))
			},
			instructions::SSTORE => {
				let address = H256::from(stack.peek(0));
				let newval = stack.peek(1);
				let val = U256::from(&*ext.storage_at(&address)?);

				let gas = if schedule.eip1283 {
					let orig = U256::from(&*ext.initial_storage_at(&address)?);
					calculate_eip1283_sstore_gas(schedule, &orig, &val, &newval)
				} else {
					if val.is_zero() && !newval.is_zero() {
						schedule.sstore_set_gas
					} else {
						// Refund for below case is added when actually executing sstore
						// !is_zero(&val) && is_zero(newval)
						schedule.sstore_reset_gas
					}
				};
				Request::Gas(Gas::from(gas))
			},
			instructions::SLOAD => {
				Request::Gas(Gas::from(schedule.sload_gas))
			},
			instructions::BALANCE => {
				Request::Gas(Gas::from(schedule.balance_gas))
			},
			instructions::EXTCODESIZE => {
				Request::Gas(Gas::from(schedule.extcodesize_gas))
			},
			instructions::EXTCODEHASH => {
				Request::Gas(Gas::from(schedule.extcodehash_gas))
			},
			instructions::SUICIDE => {
				let mut gas = Gas::from(schedule.suicide_gas);

				let is_value_transfer = !ext.origin_balance()?.is_zero();
				let address = u256_to_address(stack.peek(0));
				if (
					!schedule.no_empty && !ext.exists(&address)?
				) || (
					schedule.no_empty && is_value_transfer && !ext.exists_and_not_null(&address)?
				) {
					gas = overflowing!(gas.overflow_add(schedule.suicide_to_new_account_cost.into()));
				}

				Request::Gas(gas)
			},
			instructions::MSTORE | instructions::MLOAD => {
				Request::GasMem(default_gas, mem_needed_const(stack.peek(0), 32)?)
			},
			instructions::MSTORE8 => {
				Request::GasMem(default_gas, mem_needed_const(stack.peek(0), 1)?)
			},
			instructions::RETURN | instructions::REVERT => {
				Request::GasMem(default_gas, mem_needed(stack.peek(0), stack.peek(1))?)
			},
			instructions::SHA3 => {
				let words = overflowing!(to_word_size(Gas::from_u256(*stack.peek(1))?));
				let gas = overflowing!(Gas::from(schedule.sha3_gas).overflow_add(overflowing!(Gas::from(schedule.sha3_word_gas).overflow_mul(words))));
				Request::GasMem(gas, mem_needed(stack.peek(0), stack.peek(1))?)
			},
			instructions::CALLDATACOPY | instructions::CODECOPY | instructions::RETURNDATACOPY => {
				Request::GasMemCopy(default_gas, mem_needed(stack.peek(0), stack.peek(2))?, Gas::from_u256(*stack.peek(2))?)
			},
			instructions::EXTCODECOPY => {
				Request::GasMemCopy(schedule.extcodecopy_base_gas.into(), mem_needed(stack.peek(1), stack.peek(3))?, Gas::from_u256(*stack.peek(3))?)
			},
			instructions::LOG0 | instructions::LOG1 | instructions::LOG2 | instructions::LOG3 | instructions::LOG4 => {
				let no_of_topics = instruction.log_topics().expect("log_topics always return some for LOG* instructions; qed");
				let log_gas = schedule.log_gas + schedule.log_topic_gas * no_of_topics;

				let data_gas = overflowing!(Gas::from_u256(*stack.peek(1))?.overflow_mul(Gas::from(schedule.log_data_gas)));
				let gas = overflowing!(data_gas.overflow_add(Gas::from(log_gas)));
				Request::GasMem(gas, mem_needed(stack.peek(0), stack.peek(1))?)
			},
			instructions::CALL | instructions::CALLCODE => {
				let mut gas = Gas::from(schedule.call_gas);
				let mem = cmp::max(
					mem_needed(stack.peek(5), stack.peek(6))?,
					mem_needed(stack.peek(3), stack.peek(4))?
				);

				let address = u256_to_address(stack.peek(1));
				let is_value_transfer = !stack.peek(2).is_zero();

				if instruction == instructions::CALL && (
					(!schedule.no_empty && !ext.exists(&address)?)
					||
					(schedule.no_empty && is_value_transfer && !ext.exists_and_not_null(&address)?)
				) {
					gas = overflowing!(gas.overflow_add(schedule.call_new_account_gas.into()));
				}

				if is_value_transfer {
					gas = overflowing!(gas.overflow_add(schedule.call_value_transfer_gas.into()));
				}

				let requested = *stack.peek(0);

				Request::GasMemProvide(gas, mem, Some(requested))
			},
			instructions::DELEGATECALL | instructions::STATICCALL => {
				let gas = Gas::from(schedule.call_gas);
				let mem = cmp::max(
					mem_needed(stack.peek(4), stack.peek(5))?,
					mem_needed(stack.peek(2), stack.peek(3))?
				);
				let requested = *stack.peek(0);

				Request::GasMemProvide(gas, mem, Some(requested))
			},
			instructions::CREATE => {
				let start = stack.peek(1);
				let len = stack.peek(2);

				let gas = Gas::from(schedule.create_gas);
				let mem = mem_needed(start, len)?;

				Request::GasMemProvide(gas, mem, None)
			},
			instructions::CREATE2 => {
				let start = stack.peek(1);
				let len = stack.peek(2);

				let base = Gas::from(schedule.create_gas);
				let word = overflowing!(to_word_size(Gas::from_u256(*len)?));
				let word_gas = overflowing!(Gas::from(schedule.sha3_word_gas).overflow_mul(word));
				let gas = overflowing!(base.overflow_add(word_gas));
				let mem = mem_needed(start, len)?;

				Request::GasMemProvide(gas, mem, None)
			},
			instructions::EXP => {
				let expon = stack.peek(1);
				let bytes = ((expon.bits() + 7) / 8) as usize;
				let gas = Gas::from(schedule.exp_gas + schedule.exp_byte_gas * bytes);
				Request::Gas(gas)
			},
			instructions::BLOCKHASH => {
				Request::Gas(Gas::from(schedule.blockhash_gas))
			},
			_ => Request::Gas(default_gas),
		};

		Ok(match cost {
			Request::Gas(gas) => {
				InstructionRequirements {
					gas_cost: gas,
					provide_gas: None,
					memory_required_size: 0,
					memory_total_gas: self.current_mem_gas,
				}
			},
			Request::GasMem(gas, mem_size) => {
				let (mem_gas_cost, new_mem_gas, new_mem_size) = self.mem_gas_cost(schedule, current_mem_size, &mem_size)?;
				let gas = overflowing!(gas.overflow_add(mem_gas_cost));
				InstructionRequirements {
					gas_cost: gas,
					provide_gas: None,
					memory_required_size: new_mem_size,
					memory_total_gas: new_mem_gas,
				}
			},
			Request::GasMemProvide(gas, mem_size, requested) => {
				let (mem_gas_cost, new_mem_gas, new_mem_size) = self.mem_gas_cost(schedule, current_mem_size, &mem_size)?;
				let gas = overflowing!(gas.overflow_add(mem_gas_cost));
				let provided = self.gas_provided(schedule, gas, requested)?;
				let total_gas = overflowing!(gas.overflow_add(provided));

				InstructionRequirements {
					gas_cost: total_gas,
					provide_gas: Some(provided),
					memory_required_size: new_mem_size,
					memory_total_gas: new_mem_gas,
				}
			},
			Request::GasMemCopy(gas, mem_size, copy) => {
				let (mem_gas_cost, new_mem_gas, new_mem_size) = self.mem_gas_cost(schedule, current_mem_size, &mem_size)?;
				let copy = overflowing!(to_word_size(copy));
				let copy_gas = overflowing!(Gas::from(schedule.copy_gas).overflow_mul(copy));
				let gas = overflowing!(gas.overflow_add(copy_gas));
				let gas = overflowing!(gas.overflow_add(mem_gas_cost));

				InstructionRequirements {
					gas_cost: gas,
					provide_gas: None,
					memory_required_size: new_mem_size,
					memory_total_gas: new_mem_gas,
				}
			},
		})
	}

	fn mem_gas_cost(&self, schedule: &Schedule, current_mem_size: usize, mem_size: &Gas) -> vm::Result<(Gas, Gas, usize)> {
		let gas_for_mem = |mem_size: Gas| {
			let s = mem_size >> 5;
			// s * memory_gas + s * s / quad_coeff_div
			let a = overflowing!(s.overflow_mul(Gas::from(schedule.memory_gas)));

			// Calculate s*s/quad_coeff_div
			assert_eq!(schedule.quad_coeff_div, 512);
			let b = overflowing!(s.overflow_mul_shr(s, 9));
			Ok(overflowing!(a.overflow_add(b)))
		};

		let current_mem_size = Gas::from(current_mem_size);
		let req_mem_size_rounded = overflowing!(to_word_size(*mem_size)) << 5;

		let (mem_gas_cost, new_mem_gas) = if req_mem_size_rounded > current_mem_size {
			let new_mem_gas = gas_for_mem(req_mem_size_rounded)?;
			(new_mem_gas - self.current_mem_gas, new_mem_gas)
		} else {
			(Gas::from(0), self.current_mem_gas)
		};

		Ok((mem_gas_cost, new_mem_gas, req_mem_size_rounded.as_usize()))
	}
}

#[inline]
fn mem_needed_const<Gas: evm::CostType>(mem: &U256, add: usize) -> vm::Result<Gas> {
	Gas::from_u256(overflowing!(mem.overflowing_add(U256::from(add))))
}

#[inline]
fn mem_needed<Gas: evm::CostType>(offset: &U256, size: &U256) -> vm::Result<Gas> {
	if size.is_zero() {
		return Ok(Gas::from(0));
	}

	Gas::from_u256(overflowing!(offset.overflowing_add(*size)))
}

#[inline]
fn add_gas_usize<Gas: evm::CostType>(value: Gas, num: usize) -> (Gas, bool) {
	value.overflow_add(Gas::from(num))
}

#[inline]
fn to_word_size<Gas: evm::CostType>(value: Gas) -> (Gas, bool) {
	let (gas, overflow) = add_gas_usize(value, 31);
	if overflow {
		return (gas, overflow);
	}

	(gas >> 5, false)
}

#[inline]
fn calculate_eip1283_sstore_gas<Gas: evm::CostType>(schedule: &Schedule, original: &U256, current: &U256, new: &U256) -> Gas {
	Gas::from(
		if current == new {
			// 1. If current value equals new value (this is a no-op), 200 gas is deducted.
			schedule.sload_gas
		} else {
			// 2. If current value does not equal new value
			if original == current {
				// 2.1. If original value equals current value (this storage slot has not been changed by the current execution context)
				if original.is_zero() {
					// 2.1.1. If original value is 0, 20000 gas is deducted.
					schedule.sstore_set_gas
				} else {
					// 2.1.2. Otherwise, 5000 gas is deducted.
					schedule.sstore_reset_gas

					// 2.1.2.1. If new value is 0, add 15000 gas to refund counter.
				}
			} else {
				// 2.2. If original value does not equal current value (this storage slot is dirty), 200 gas is deducted. Apply both of the following clauses.
				schedule.sload_gas

				// 2.2.1. If original value is not 0
				// 2.2.1.1. If current value is 0 (also means that new value is not 0), remove 15000 gas from refund counter. We can prove that refund counter will never go below 0.
				// 2.2.1.2. If new value is 0 (also means that current value is not 0), add 15000 gas to refund counter.

				// 2.2.2. If original value equals new value (this storage slot is reset)
				// 2.2.2.1. If original value is 0, add 19800 gas to refund counter.
				// 2.2.2.2. Otherwise, add 4800 gas to refund counter.
			}
		}
	)
}

pub fn handle_eip1283_sstore_clears_refund(ext: &mut vm::Ext, original: &U256, current: &U256, new: &U256) {
	let sstore_clears_schedule = ext.schedule().sstore_refund_gas;

	if current == new {
		// 1. If current value equals new value (this is a no-op), 200 gas is deducted.
	} else {
		// 2. If current value does not equal new value
		if original == current {
			// 2.1. If original value equals current value (this storage slot has not been changed by the current execution context)
			if original.is_zero() {
				// 2.1.1. If original value is 0, 20000 gas is deducted.
			} else {
				// 2.1.2. Otherwise, 5000 gas is deducted.
				if new.is_zero() {
					// 2.1.2.1. If new value is 0, add 15000 gas to refund counter.
					ext.add_sstore_refund(sstore_clears_schedule);
				}
			}
		} else {
			// 2.2. If original value does not equal current value (this storage slot is dirty), 200 gas is deducted. Apply both of the following clauses.

			if !original.is_zero() {
				// 2.2.1. If original value is not 0
				if current.is_zero() {
					// 2.2.1.1. If current value is 0 (also means that new value is not 0), remove 15000 gas from refund counter. We can prove that refund counter will never go below 0.
					ext.sub_sstore_refund(sstore_clears_schedule);
				} else if new.is_zero() {
					// 2.2.1.2. If new value is 0 (also means that current value is not 0), add 15000 gas to refund counter.
					ext.add_sstore_refund(sstore_clears_schedule);
				}
			}

			if original == new {
				// 2.2.2. If original value equals new value (this storage slot is reset)
				if original.is_zero() {
					// 2.2.2.1. If original value is 0, add 19800 gas to refund counter.
					let refund = ext.schedule().sstore_set_gas - ext.schedule().sload_gas;
					ext.add_sstore_refund(refund);
				} else {
					// 2.2.2.2. Otherwise, add 4800 gas to refund counter.
					let refund = ext.schedule().sstore_reset_gas - ext.schedule().sload_gas;
					ext.add_sstore_refund(refund);
				}
			}
		}
	}
}

#[test]
fn test_mem_gas_cost() {
	// given
	let gasometer = Gasometer::<U256>::new(U256::zero());
	let schedule = Schedule::default();
	let current_mem_size = 5;
	let mem_size = !U256::zero();

	// when
	let result = gasometer.mem_gas_cost(&schedule, current_mem_size, &mem_size);

	// then
	if result.is_ok() {
		assert!(false, "Should fail with OutOfGas");
	}
}

#[test]
fn test_calculate_mem_cost() {
	// given
	let gasometer = Gasometer::<usize>::new(0);
	let schedule = Schedule::default();
	let current_mem_size = 0;
	let mem_size = 5;

	// when
	let (mem_cost, new_mem_gas, mem_size) = gasometer.mem_gas_cost(&schedule, current_mem_size, &mem_size).unwrap();

	// then
	assert_eq!(mem_cost, 3);
	assert_eq!(new_mem_gas, 3);
	assert_eq!(mem_size, 32);
}
