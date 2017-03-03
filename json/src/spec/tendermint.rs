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

//! Tendermint params deserialization.

use uint::Uint;
use hash::Address;
use super::ValidatorSet;

/// Tendermint params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct TendermintParams {
	/// Gas limit divisor.
	#[serde(rename="gasLimitBoundDivisor")]
	pub gas_limit_bound_divisor: Uint,
	/// Valid validators.
	pub validators: ValidatorSet,
	/// Propose step timeout in milliseconds.
	#[serde(rename="timeoutPropose")]
	pub timeout_propose: Option<Uint>,
	/// Prevote step timeout in milliseconds.
	#[serde(rename="timeoutPrevote")]
	pub timeout_prevote: Option<Uint>,
	/// Precommit step timeout in milliseconds.
	#[serde(rename="timeoutPrecommit")]
	pub timeout_precommit: Option<Uint>,
	/// Commit step timeout in milliseconds.
	#[serde(rename="timeoutCommit")]
	pub timeout_commit: Option<Uint>,
	/// Block reward.
	#[serde(rename="blockReward")]
	pub block_reward: Option<Uint>,
	/// Address of the registrar contract.
	pub registrar: Option<Address>,
}

/// Tendermint engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Tendermint {
	/// Ethash params.
	pub params: TendermintParams,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::tendermint::Tendermint;

	#[test]
	fn tendermint_deserialization() {
		let s = r#"{
			"params": {
				"gasLimitBoundDivisor": "0x0400",
				"validators": {
					"list": ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"]
				},
				"blockReward": "0x50"
			}
		}"#;

		let _deserialized: Tendermint = serde_json::from_str(s).unwrap();
	}
}
