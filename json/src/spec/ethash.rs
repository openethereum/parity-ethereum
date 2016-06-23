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

//! Ethash params deserialization.

use uint::Uint;
use hash::Address;

/// Ethash params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct EthashParams {
	/// Gas limit divisor.
	#[serde(rename="gasLimitBoundDivisor")]
	pub gas_limit_bound_divisor: Uint,
	/// Minimum difficulty.
	#[serde(rename="minimumDifficulty")]
	pub minimum_difficulty: Uint,
	/// Difficulty bound divisor.
	#[serde(rename="difficultyBoundDivisor")]
	pub difficulty_bound_divisor: Uint,
	/// Block duration.
	#[serde(rename="durationLimit")]
	pub duration_limit: Uint,
	/// Block reward.
	#[serde(rename="blockReward")]
	pub block_reward: Uint,
	/// Namereg contract address.
	pub registrar: Address,
	/// Homestead transition block number.
	#[serde(rename="frontierCompatibilityModeLimit")]
	pub frontier_compatibility_mode_limit: Uint,
	/// DAO rescue soft-fork?
	#[serde(rename="daoRescueSoftFork")]
	pub dao_rescue_soft_fork: bool,
}

/// Ethash engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Ethash {
	/// Ethash params.
	pub params: EthashParams,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::ethash::Ethash;

	#[test]
	fn ethash_deserialization() {
		let s = r#"{
			"params": {
				"gasLimitBoundDivisor": "0x0400",
				"minimumDifficulty": "0x020000",
				"difficultyBoundDivisor": "0x0800",
				"durationLimit": "0x0d",
				"blockReward": "0x4563918244F40000",
				"registrar": "0xc6d9d2cd449a754c494264e1809c50e34d64562b",
				"frontierCompatibilityModeLimit": "0x42",
				"daoRescueSoftFork": true
			}
		}"#;

		let _deserialized: Ethash = serde_json::from_str(s).unwrap();
	}
}
