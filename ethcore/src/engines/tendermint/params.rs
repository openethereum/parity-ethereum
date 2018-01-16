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

//! Tendermint specific parameters.

use ethjson;
use time::Duration;
use ethereum_types::U256;
use super::super::validator_set::{ValidatorSet, new_validator_set};
use super::super::transition::Timeouts;
use super::Step;

/// `Tendermint` params.
pub struct TendermintParams {
	/// List of validators.
	pub validators: Box<ValidatorSet>,
	/// Timeout durations for different steps.
	pub timeouts: TendermintTimeouts,
	/// Reward per block in base units.
	pub block_reward: U256,
}

/// Base timeout of each step in ms.
#[derive(Debug, Clone)]
pub struct TendermintTimeouts {
	pub propose: Duration,
	pub prevote: Duration,
	pub precommit: Duration,
	pub commit: Duration,
}

impl Default for TendermintTimeouts {
	fn default() -> Self {
		TendermintTimeouts {
			propose: Duration::milliseconds(1000),
			prevote: Duration::milliseconds(1000),
			precommit: Duration::milliseconds(1000),
			commit: Duration::milliseconds(1000),
		}
	}
}

impl Timeouts<Step> for TendermintTimeouts {
	fn initial(&self) -> Duration {
		self.propose
	}

	fn timeout(&self, step: &Step) -> Duration {
		match *step {
			Step::Propose => self.propose,
			Step::Prevote => self.prevote,
			Step::Precommit => self.precommit,
			Step::Commit => self.commit,
		}
	}
}

fn to_duration(ms: ethjson::uint::Uint) -> Duration {
	let ms: usize = ms.into();
	Duration::milliseconds(ms as i64)
}

impl From<ethjson::spec::TendermintParams> for TendermintParams {
	fn from(p: ethjson::spec::TendermintParams) -> Self {
		let dt = TendermintTimeouts::default();
		TendermintParams {
			validators: new_validator_set(p.validators),
			timeouts: TendermintTimeouts {
				propose: p.timeout_propose.map_or(dt.propose, to_duration),
				prevote: p.timeout_prevote.map_or(dt.prevote, to_duration),
				precommit: p.timeout_precommit.map_or(dt.precommit, to_duration),
				commit: p.timeout_commit.map_or(dt.commit, to_duration),
			},
			block_reward: p.block_reward.map_or(U256::default(), Into::into),
		}
	}
}
