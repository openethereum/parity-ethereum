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

//! Authority params deserialization.

use uint::Uint;
use hash::Address;
use super::ValidatorSet;

/// Authority params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct AuthorityRoundParams {
	/// Gas limit divisor.
	#[serde(rename="gasLimitBoundDivisor")]
	pub gas_limit_bound_divisor: Uint,
	/// Block duration.
	#[serde(rename="stepDuration")]
	pub step_duration: Uint,
	/// Valid authorities
	pub validators: ValidatorSet,
	/// Block reward.
	#[serde(rename="blockReward")]
	pub block_reward: Option<Uint>,
	/// Address of the registrar contract.
	pub registrar: Option<Address>,
	/// Starting step. Determined automatically if not specified.
	/// To be used for testing only.
	#[serde(rename="startStep")]
	pub start_step: Option<Uint>,
	/// Block at which score validation should start.
	#[serde(rename="validateScoreTransition")]
	pub validate_score_transition: Option<Uint>,
	/// See main AuthorityRoundParams docs.
	#[serde(rename="eip155Transition")]
	pub eip155_transition: Option<Uint>,
	/// Block from which monotonic steps start.
	#[serde(rename="validateStepTransition")]
	pub validate_step_transition: Option<Uint>,
	/// Whether transitions should be immediate.
	#[serde(rename="immediateTransitions")]
	pub immediate_transitions: Option<bool>,
}

/// Authority engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct AuthorityRound {
	/// Ethash params.
	pub params: AuthorityRoundParams,
}

#[cfg(test)]
mod tests {
	use util::{H160, U256};
	use uint::Uint;
	use hash::Address;
	use serde_json;
	use spec::validator_set::ValidatorSet;
	use spec::authority_round::AuthorityRound;

	#[test]
	fn authority_round_deserialization() {
		let s = r#"{
			"params": {
				"gasLimitBoundDivisor": "0x0400",
				"stepDuration": "0x02",
				"validators": {
					"list" : ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"]
				},
				"blockReward": "0x50",
				"startStep" : 24,
				"eip155Transition": "0x42",
				"validateStepTransition": 150
			}
		}"#;

		let deserialized: AuthorityRound = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.params.gas_limit_bound_divisor, Uint(U256::from(0x0400)));
		assert_eq!(deserialized.params.step_duration, Uint(U256::from(0x02)));
		assert_eq!(deserialized.params.validators, ValidatorSet::List(vec![Address(H160::from("0xc6d9d2cd449a754c494264e1809c50e34d64562b"))]));
		assert_eq!(deserialized.params.block_reward, Some(Uint(U256::from(0x50))));
		assert!(deserialized.params.registrar.is_none());
		assert_eq!(deserialized.params.start_step, Some(Uint(U256::from(24))));
		assert_eq!(deserialized.params.eip155_transition, Some(Uint(U256::from(0x42))));
		assert_eq!(deserialized.params.immediate_transitions, None);
	}
}
