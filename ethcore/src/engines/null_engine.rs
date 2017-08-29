// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::collections::BTreeMap;
use util::Address;
use builtin::Builtin;
use block::{ExecutedBlock, IsBlock};
use util::U256;
use engines::Engine;
use spec::CommonParams;
use evm::Schedule;
use header::BlockNumber;
use error::Error;
use state::CleanupMode;
use trace::{Tracer, ExecutiveTracer, RewardType};

/// An engine which does not provide any consensus mechanism and does not seal blocks.
pub struct NullEngine {
	params: CommonParams,
	builtins: BTreeMap<Address, Builtin>,
}

impl NullEngine {
	/// Returns new instance of NullEngine with default VM Factory
	pub fn new(params: CommonParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		NullEngine{
			params: params,
			builtins: builtins,
		}
	}
}

impl Default for NullEngine {
	fn default() -> Self {
		Self::new(Default::default(), Default::default())
	}
}

impl Engine for NullEngine {
	fn name(&self) -> &str {
		"NullEngine"
	}

	fn params(&self) -> &CommonParams {
		&self.params
	}

	fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		&self.builtins
	}

	fn schedule(&self, _block_number: BlockNumber) -> Schedule {
		Schedule::new_homestead()
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		Some(Box::new(::snapshot::PowSnapshot::new(10000, 10000)))
	}

	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		if self.params.block_reward == U256::zero() {
			// we don't have to apply reward in this case
			return Ok(())
		}

		/// Block reward
		let tracing_enabled = block.tracing_enabled();
		let fields = block.fields_mut();		
		let mut tracer = ExecutiveTracer::default();

		let result_block_reward = U256::from(1000000000);
		fields.state.add_balance(
			fields.header.author(),
			&result_block_reward,
			CleanupMode::NoEmpty
		)?;

		if tracing_enabled {
			let block_author = fields.header.author().clone();
			tracer.trace_reward(block_author, result_block_reward, RewardType::Block);
		}

		/// Uncle rewards
		let result_uncle_reward = U256::from(10000000);
		for u in fields.uncles.iter() {
			let uncle_author = u.author().clone();
			fields.state.add_balance(
				u.author(),
				&(result_uncle_reward),
				CleanupMode::NoEmpty
			)?;
			if tracing_enabled {
				tracer.trace_reward(uncle_author, result_uncle_reward, RewardType::Uncle);
			}			
		}

		fields.state.commit()?;
		if tracing_enabled {
			fields.traces.as_mut().map(|mut traces| traces.push(tracer.traces()));
		}
		Ok(())
	}
}
