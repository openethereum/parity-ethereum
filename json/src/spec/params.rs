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

//! Spec params deserialization.

use uint::Uint;
use hash::Address;

/// Spec params.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Params {
	#[serde(rename="accountStartNonce")]
	account_start_nonce: Uint,
	#[serde(rename="frontierCompatibilityModeLimit")]
	frontier_compatibility_mode_limit: Uint,
	#[serde(rename="maximumExtraDataSize")]
	maximum_extra_data_size: Uint,
	#[serde(rename="tieBreakingGas")]
	tie_breaking_gas: bool,
	#[serde(rename="minGasLimit")]
	min_gas_limit: Uint,
	#[serde(rename="gasLimitBoundDivisor")]
	gas_limit_bound_divisor: Uint,
	#[serde(rename="minimumDifficulty")]
	minimum_difficulty: Uint,
	#[serde(rename="difficultyBoundDivisor")]
	difficulty_bound_divisor: Uint,
	#[serde(rename="durationLimit")]
	duration_limit: Uint,
	#[serde(rename="blockReward")]
	block_reward: Uint,
	registrar: Address,
	#[serde(rename="networkID")]
	network_id: Uint,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::params::Params;

	#[test]
	fn params_deserialization() {
		let s = r#"{
			"accountStartNonce": "0x00",
			"frontierCompatibilityModeLimit": "0x118c30",
			"maximumExtraDataSize": "0x20",
			"tieBreakingGas": false,
			"minGasLimit": "0x1388",
			"gasLimitBoundDivisor": "0x0400",
			"minimumDifficulty": "0x020000",
			"difficultyBoundDivisor": "0x0800",
			"durationLimit": "0x0d",
			"blockReward": "0x4563918244F40000",
			"registrar" : "0xc6d9d2cd449a754c494264e1809c50e34d64562b",
			"networkID" : "0x1"
		}"#;
		let _deserialized: Params = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
