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

use engines::Engine;
use engines::block_reward::{self, RewardKind};
use ethereum_types::U256;
use machine::Machine;
use types::BlockNumber;
use types::header::{Header, ExtendedHeader};
use block::ExecutedBlock;
use error::Error;

/// Params for a null engine.
#[derive(Clone, Default)]
pub struct NullEngineParams {
	/// base reward for a block.
	pub block_reward: U256,
}

impl From<::ethjson::spec::NullEngineParams> for NullEngineParams {
	fn from(p: ::ethjson::spec::NullEngineParams) -> Self {
		NullEngineParams {
			block_reward: p.block_reward.map_or_else(Default::default, Into::into),
		}
	}
}

/// An engine which does not provide any consensus mechanism and does not seal blocks.
pub struct NullEngine {
	params: NullEngineParams,
	machine: Machine,
}

impl NullEngine {
	/// Returns new instance of NullEngine with default VM Factory
	pub fn new(params: NullEngineParams, machine: Machine) -> Self {
		NullEngine {
			params,
			machine,
		}
	}
}

impl Engine for NullEngine {
	fn name(&self) -> &str {
		"NullEngine"
	}

	fn machine(&self) -> &Machine { &self.machine }

	fn on_close_block(
		&self,
		block: &mut ExecutedBlock,
		_parent_header: &Header
	) -> Result<(), Error> {
		use std::ops::Shr;

		let author = *block.header.author();
		let number = block.header.number();

		let reward = self.params.block_reward;
		if reward == U256::zero() { return Ok(()) }

		let n_uncles = block.uncles.len();

		let mut rewards = Vec::new();

		// Bestow block reward
		let result_block_reward = reward + reward.shr(5) * U256::from(n_uncles);
		rewards.push((author, RewardKind::Author, result_block_reward));

		// bestow uncle rewards.
		for u in &block.uncles {
			let uncle_author = u.author();
			let result_uncle_reward = (reward * U256::from(8 + u.number() - number)).shr(3);
			rewards.push((*uncle_author, RewardKind::uncle(number, u.number()), result_uncle_reward));
		}

		block_reward::apply_block_rewards(&rewards, block, &self.machine)
	}

	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 2 }

	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
		Ok(())
	}

	fn snapshot_components(&self) -> Option<Box<dyn (::snapshot::SnapshotComponents)>> {
		Some(Box::new(::snapshot::PowSnapshot::new(10000, 10000)))
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
		super::total_difficulty_fork_choice(new, current)
	}
}
