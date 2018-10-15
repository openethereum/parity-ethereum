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

//! Clique specific parameters.

use ethjson;
use super::super::validator_set::{ValidatorSet, new_validator_set};

/// `Clique` params.
pub struct CliqueParams {
	/// List of validators.
	pub validators: Box<ValidatorSet>
}

fn to_duration(ms: ethjson::uint::Uint) -> Duration {
	let ms: usize = ms.into();
	Duration::from_millis(ms as u64)
}

impl From<ethjson::spec::CliqueParams> for CliqueParams {
	fn from(p: ethjson::spec::CliqueParams) -> Self {
		CliqueParams {
			validators: new_validator_set(p.validators)
		}
	}
}
