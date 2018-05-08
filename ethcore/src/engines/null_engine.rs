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

use ethereum_types::U256;
use engines::Engine;
use engines::block_reward::{self, RewardKind};
use header::BlockNumber;
use machine::WithRewards;
use parity_machine::{Header, LiveBlock, WithBalances, TotalScoredHeader};

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
pub struct NullEngine<M> {
	params: NullEngineParams,
	machine: M,
}

impl<M> NullEngine<M> {
	/// Returns new instance of NullEngine with default VM Factory
	pub fn new(params: NullEngineParams, machine: M) -> Self {
		NullEngine {
			params: params,
			machine: machine,
		}
	}
}

impl<M: Default> Default for NullEngine<M> {
	fn default() -> Self {
		Self::new(Default::default(), Default::default())
	}
}

impl<M: WithBalances + WithRewards> Engine<M> for NullEngine<M>
  where M::ExtendedHeader: TotalScoredHeader,
        <M::ExtendedHeader as TotalScoredHeader>::Value: Ord
{
	fn name(&self) -> &str {
		"NullEngine"
	}

	fn machine(&self) -> &M { &self.machine }

	fn on_close_block(&self, block: &mut M::LiveBlock) -> Result<(), M::Error> {
		use std::ops::Shr;

		let author = *LiveBlock::header(&*block).author();
		let number = LiveBlock::header(&*block).number();

		let reward = self.params.block_reward;
		if reward == U256::zero() { return Ok(()) }

		let n_uncles = LiveBlock::uncles(&*block).len();

		let mut rewards = Vec::new();

		// Bestow block reward
		let result_block_reward = reward + reward.shr(5) * U256::from(n_uncles);
		rewards.push((author, RewardKind::Author, result_block_reward));

		// bestow uncle rewards.
		for u in LiveBlock::uncles(&*block) {
			let uncle_author = u.author();
			let result_uncle_reward = (reward * U256::from(8 + u.number() - number)).shr(3);
			rewards.push((*uncle_author, RewardKind::Uncle, result_uncle_reward));
		}

		block_reward::apply_block_rewards(&rewards, block, &self.machine)
	}

	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 2 }

	fn verify_local_seal(&self, _header: &M::Header) -> Result<(), M::Error> {
		Ok(())
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		Some(Box::new(::snapshot::PowSnapshot::new(10000, 10000)))
	}

	fn fork_choice(&self, new: &M::ExtendedHeader, current: &M::ExtendedHeader) -> super::ForkChoice {
		super::total_difficulty_fork_choice(new, current)
	}
}
