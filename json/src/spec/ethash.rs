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

//! Ethash params deserialization.

use std::collections::BTreeMap;
use uint::{self, Uint};
use bytes::Bytes;
use hash::Address;

/// Deserializable doppelganger of block rewards for EthashParams
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BlockReward {
	Single(Uint),
	Multi(BTreeMap<Uint, Uint>),
}

/// Deserializable doppelganger of EthashParams.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct EthashParams {
	/// See main EthashParams docs.
	#[serde(rename="minimumDifficulty")]
	#[serde(deserialize_with="uint::validate_non_zero")]
	pub minimum_difficulty: Uint,
	/// See main EthashParams docs.
	#[serde(rename="difficultyBoundDivisor")]
	#[serde(deserialize_with="uint::validate_non_zero")]
	pub difficulty_bound_divisor: Uint,
	/// See main EthashParams docs.
	#[serde(rename="difficultyIncrementDivisor")]
	#[serde(default, deserialize_with="uint::validate_optional_non_zero")]
	pub difficulty_increment_divisor: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="metropolisDifficultyIncrementDivisor")]
	#[serde(default, deserialize_with="uint::validate_optional_non_zero")]
	pub metropolis_difficulty_increment_divisor: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="durationLimit")]
	pub duration_limit: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="homesteadTransition")]
	pub homestead_transition: Option<Uint>,
	/// Reward per block in wei.
	#[serde(rename="blockReward")]
	pub block_reward: Option<BlockReward>,
	/// Block at which the block reward contract should start being used.
	#[serde(rename="blockRewardContractTransition")]
	pub block_reward_contract_transition: Option<Uint>,
	/// Block reward contract address (setting the block reward contract
	/// overrides all other block reward parameters).
	#[serde(rename="blockRewardContractAddress")]
	pub block_reward_contract_address: Option<Address>,
	/// Block reward code. This overrides the block reward contract address.
	#[serde(rename="blockRewardContractCode")]
	pub block_reward_contract_code: Option<Bytes>,

	/// See main EthashParams docs.
	#[serde(rename="daoHardforkTransition")]
	pub dao_hardfork_transition: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="daoHardforkBeneficiary")]
	pub dao_hardfork_beneficiary: Option<Address>,
	/// See main EthashParams docs.
	#[serde(rename="daoHardforkAccounts")]
	pub dao_hardfork_accounts: Option<Vec<Address>>,

	/// See main EthashParams docs.
	#[serde(rename="difficultyHardforkTransition")]
	pub difficulty_hardfork_transition: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="difficultyHardforkBoundDivisor")]
	#[serde(default, deserialize_with="uint::validate_optional_non_zero")]
	pub difficulty_hardfork_bound_divisor: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="bombDefuseTransition")]
	pub bomb_defuse_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="eip100bTransition")]
	pub eip100b_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="ecip1010PauseTransition")]
	pub ecip1010_pause_transition: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="ecip1010ContinueTransition")]
	pub ecip1010_continue_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="ecip1017EraRounds")]
	pub ecip1017_era_rounds: Option<Uint>,

	/// Delays of difficulty bombs.
	#[serde(rename="difficultyBombDelays")]
	pub difficulty_bomb_delays: Option<BTreeMap<Uint, Uint>>,

	/// EXPIP-2 block height
	#[serde(rename="expip2Transition")]
	pub expip2_transition: Option<Uint>,
	/// EXPIP-2 duration limit
	#[serde(rename="expip2DurationLimit")]
	pub expip2_duration_limit: Option<Uint>,
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
	use uint::Uint;
	use ethereum_types::{H160, U256};
	use hash::Address;
	use spec::ethash::{Ethash, EthashParams, BlockReward};

	#[test]
	fn ethash_deserialization() {
		let s = r#"{
			"params": {
				"minimumDifficulty": "0x020000",
				"difficultyBoundDivisor": "0x0800",
				"durationLimit": "0x0d",
				"homesteadTransition": "0x42",
				"blockReward": "0x100",
				"daoHardforkTransition": "0x08",
				"daoHardforkBeneficiary": "0xabcabcabcabcabcabcabcabcabcabcabcabcabca",
				"daoHardforkAccounts": [
					"0x304a554a310c7e546dfe434669c62820b7d83490",
					"0x914d1b8b43e92723e64fd0a06f5bdb8dd9b10c79",
					"0xfe24cdd8648121a43a7c86d289be4dd2951ed49f",
					"0x17802f43a0137c506ba92291391a8a8f207f487d",
					"0xb136707642a4ea12fb4bae820f03d2562ebff487",
					"0xdbe9b615a3ae8709af8b93336ce9b477e4ac0940",
					"0xf14c14075d6c4ed84b86798af0956deef67365b5",
					"0xca544e5c4687d109611d0f8f928b53a25af72448",
					"0xaeeb8ff27288bdabc0fa5ebb731b6f409507516c",
					"0xcbb9d3703e651b0d496cdefb8b92c25aeb2171f7",
					"0xaccc230e8a6e5be9160b8cdf2864dd2a001c28b6",
					"0x2b3455ec7fedf16e646268bf88846bd7a2319bb2",
					"0x4613f3bca5c44ea06337a9e439fbc6d42e501d0a",
					"0xd343b217de44030afaa275f54d31a9317c7f441e",
					"0x84ef4b2357079cd7a7c69fd7a37cd0609a679106",
					"0xda2fef9e4a3230988ff17df2165440f37e8b1708",
					"0xf4c64518ea10f995918a454158c6b61407ea345c",
					"0x7602b46df5390e432ef1c307d4f2c9ff6d65cc97",
					"0xbb9bc244d798123fde783fcc1c72d3bb8c189413",
					"0x807640a13483f8ac783c557fcdf27be11ea4ac7a"
				],
				"difficultyHardforkTransition": "0x59d9",
				"difficultyHardforkBoundDivisor": "0x0200",
				"bombDefuseTransition": "0x41",
				"eip100bTransition": "0x42"
			}
		}"#;

		let deserialized: Ethash = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Ethash {
			params: EthashParams {
				minimum_difficulty: Uint(U256::from(0x020000)),
				difficulty_bound_divisor: Uint(U256::from(0x0800)),
				difficulty_increment_divisor: None,
				metropolis_difficulty_increment_divisor: None,
				duration_limit: Some(Uint(U256::from(0x0d))),
				homestead_transition: Some(Uint(U256::from(0x42))),
				block_reward: Some(BlockReward::Single(Uint(U256::from(0x100)))),
				block_reward_contract_address: None,
				block_reward_contract_code: None,
				block_reward_contract_transition: None,
				dao_hardfork_transition: Some(Uint(U256::from(0x08))),
				dao_hardfork_beneficiary: Some(Address(H160::from("0xabcabcabcabcabcabcabcabcabcabcabcabcabca"))),
				dao_hardfork_accounts: Some(vec![
					Address(H160::from("0x304a554a310c7e546dfe434669c62820b7d83490")),
					Address(H160::from("0x914d1b8b43e92723e64fd0a06f5bdb8dd9b10c79")),
					Address(H160::from("0xfe24cdd8648121a43a7c86d289be4dd2951ed49f")),
					Address(H160::from("0x17802f43a0137c506ba92291391a8a8f207f487d")),
					Address(H160::from("0xb136707642a4ea12fb4bae820f03d2562ebff487")),
					Address(H160::from("0xdbe9b615a3ae8709af8b93336ce9b477e4ac0940")),
					Address(H160::from("0xf14c14075d6c4ed84b86798af0956deef67365b5")),
					Address(H160::from("0xca544e5c4687d109611d0f8f928b53a25af72448")),
					Address(H160::from("0xaeeb8ff27288bdabc0fa5ebb731b6f409507516c")),
					Address(H160::from("0xcbb9d3703e651b0d496cdefb8b92c25aeb2171f7")),
					Address(H160::from("0xaccc230e8a6e5be9160b8cdf2864dd2a001c28b6")),
					Address(H160::from("0x2b3455ec7fedf16e646268bf88846bd7a2319bb2")),
					Address(H160::from("0x4613f3bca5c44ea06337a9e439fbc6d42e501d0a")),
					Address(H160::from("0xd343b217de44030afaa275f54d31a9317c7f441e")),
					Address(H160::from("0x84ef4b2357079cd7a7c69fd7a37cd0609a679106")),
					Address(H160::from("0xda2fef9e4a3230988ff17df2165440f37e8b1708")),
					Address(H160::from("0xf4c64518ea10f995918a454158c6b61407ea345c")),
					Address(H160::from("0x7602b46df5390e432ef1c307d4f2c9ff6d65cc97")),
					Address(H160::from("0xbb9bc244d798123fde783fcc1c72d3bb8c189413")),
					Address(H160::from("0x807640a13483f8ac783c557fcdf27be11ea4ac7a")),
				]),
				difficulty_hardfork_transition: Some(Uint(U256::from(0x59d9))),
				difficulty_hardfork_bound_divisor: Some(Uint(U256::from(0x0200))),
				bomb_defuse_transition: Some(Uint(U256::from(0x41))),
				eip100b_transition: Some(Uint(U256::from(0x42))),
				ecip1010_pause_transition: None,
				ecip1010_continue_transition: None,
				ecip1017_era_rounds: None,
				expip2_transition: None,
				expip2_duration_limit: None,
				difficulty_bomb_delays: None,
			}
		});
	}

	#[test]
	fn ethash_deserialization_missing_optionals() {
		let s = r#"{
			"params": {
				"difficultyBoundDivisor": "0x0800",
				"minimumDifficulty": "0x020000"
			}
		}"#;

		let deserialized: Ethash = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, Ethash {
			params: EthashParams {
				minimum_difficulty: Uint(U256::from(0x020000)),
				difficulty_bound_divisor: Uint(U256::from(0x0800)),
				difficulty_increment_divisor: None,
				metropolis_difficulty_increment_divisor: None,
				duration_limit: None,
				homestead_transition: None,
				block_reward: None,
				block_reward_contract_address: None,
				block_reward_contract_code: None,
				block_reward_contract_transition: None,
				dao_hardfork_transition: None,
				dao_hardfork_beneficiary: None,
				dao_hardfork_accounts: None,
				difficulty_hardfork_transition: None,
				difficulty_hardfork_bound_divisor: None,
				bomb_defuse_transition: None,
				eip100b_transition: None,
				ecip1010_pause_transition: None,
				ecip1010_continue_transition: None,
				ecip1017_era_rounds: None,
				expip2_transition: None,
				expip2_duration_limit: None,
				difficulty_bomb_delays: None,
			}
		});
	}

	#[test]
	#[should_panic(expected = "a non-zero value")]
	fn test_zero_value_divisor() {
		let s = r#"{
			"params": {
				"difficultyBoundDivisor": "0x0",
				"minimumDifficulty": "0x020000"
			}
		}"#;

		let _deserialized: Ethash = serde_json::from_str(s).unwrap();
	}
}
