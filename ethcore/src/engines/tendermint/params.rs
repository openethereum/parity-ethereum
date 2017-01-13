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

//! Tendermint specific parameters.

use ethjson;
use super::transition::TendermintTimeouts;
use util::{U256, Uint};
use time::Duration;

/// `Tendermint` params.
#[derive(Debug)]
pub struct TendermintParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// List of validators.
	pub validators: ethjson::spec::ValidatorSet,
	/// Timeout durations for different steps.
	pub timeouts: TendermintTimeouts,
	/// Block reward.
	pub block_reward: U256,
}

fn to_duration(ms: ethjson::uint::Uint) -> Duration {
	let ms: usize = ms.into();
	Duration::milliseconds(ms as i64)
}

impl From<ethjson::spec::TendermintParams> for TendermintParams {
	fn from(p: ethjson::spec::TendermintParams) -> Self {
		let dt = TendermintTimeouts::default();
		TendermintParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			validators: p.validators,
			timeouts: TendermintTimeouts {
				propose: p.timeout_propose.map_or(dt.propose, to_duration),
				prevote: p.timeout_prevote.map_or(dt.prevote, to_duration),
				precommit: p.timeout_precommit.map_or(dt.precommit, to_duration),
				commit: p.timeout_commit.map_or(dt.commit, to_duration),
			},
			block_reward: p.block_reward.map_or_else(U256::zero, Into::into),
		}
	}
}
