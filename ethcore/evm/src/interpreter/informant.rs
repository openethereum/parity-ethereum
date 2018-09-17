// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

pub use self::inner::*;

#[macro_use]
#[cfg(not(feature = "evm-debug"))]
mod inner {
	macro_rules! evm_debug {
		($x: expr) => {}
	}

	pub struct EvmInformant;
	impl EvmInformant {
		pub fn new(_depth: usize) -> Self {
			EvmInformant {}
		}
		pub fn done(&mut self) {}
	}
}

#[macro_use]
#[cfg(feature = "evm-debug")]
mod inner {
	use std::iter;
	use std::collections::HashMap;
	use std::time::{Instant, Duration};

	use ethereum_types::U256;

	use interpreter::stack::Stack;
	use instructions::{Instruction, InstructionInfo};
	use CostType;

	macro_rules! evm_debug {
		($x: expr) => {
			$x
		}
	}

	fn print(data: String) {
		if cfg!(feature = "evm-debug-tests") {
			println!("{}", data);
		} else {
			debug!(target: "evm", "{}", data);
		}
	}

	pub struct EvmInformant {
		spacing: String,
		last_instruction: Instant,
		stats: HashMap<Instruction, Stats>,
	}

	impl EvmInformant {

		fn color(instruction: Instruction, name: &str) -> String {
			let c = instruction as usize % 6;
			let colors = [31, 34, 33, 32, 35, 36];
			format!("\x1B[1;{}m{}\x1B[0m", colors[c], name)
		}

		fn as_micro(duration: &Duration) -> u64 {
			let mut sec = duration.as_secs();
			let subsec = duration.subsec_nanos() as u64;
			sec = sec.saturating_mul(1_000_000u64);
			sec += subsec / 1_000;
			sec
		}

		pub fn new(depth: usize) -> Self {
			EvmInformant {
				spacing: iter::repeat(".").take(depth).collect(),
				last_instruction: Instant::now(),
				stats: HashMap::new(),
			}
		}

		pub fn before_instruction<Cost: CostType>(&mut self, pc: usize, instruction: Instruction, info: &InstructionInfo, current_gas: &Cost, stack: &Stack<U256>) {
			let time = self.last_instruction.elapsed();
			self.last_instruction = Instant::now();

			print(format!("{}[0x{:<3x}][{:>19}(0x{:<2x}) Gas Left: {:6?} (Previous took: {:10}μs)",
				&self.spacing,
				pc,
				Self::color(instruction, info.name),
				instruction as u8,
				current_gas,
				Self::as_micro(&time),
			));

			if info.args > 0 {
				for (idx, item) in stack.peek_top(info.args).iter().enumerate() {
					print(format!("{}       |{:2}: {:?}", self.spacing, idx, item));
				}
			}
		}

		pub fn after_instruction(&mut self, instruction: Instruction) {
			let stats = self.stats.entry(instruction).or_insert_with(|| Stats::default());
			let took = self.last_instruction.elapsed();
			stats.note(took);
		}

		pub fn done(&mut self) {
			// Print out stats
			let mut stats: Vec<(_,_)> = self.stats.drain().collect();
			stats.sort_by(|ref a, ref b| b.1.avg().cmp(&a.1.avg()));

			print(format!("\n{}-------OPCODE STATS:", self.spacing));
			for (instruction, stats) in stats.into_iter() {
				let info = instruction.info();
				print(format!("{}-------{:>19}(0x{:<2x}) count: {:4}, avg: {:10}μs",
					self.spacing,
					Self::color(instruction, info.name),
					instruction as u8,
					stats.count,
					stats.avg(),
				));
			}
		}

	}

	struct Stats {
		count: u64,
		total_duration: Duration,
	}

	impl Default for Stats {
		fn default() -> Self {
			Stats {
				count: 0,
				total_duration: Duration::from_secs(0),
			}
		}
	}

	impl Stats {
		fn note(&mut self, took: Duration) {
			self.count += 1;
			self.total_duration += took;
		}

		fn avg(&self) -> u64 {
			EvmInformant::as_micro(&self.total_duration) / self.count
		}
	}
}
