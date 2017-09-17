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

//! Ethash params deserialization.

use uint::Uint;
use hash::Address;

/// Deserializable doppelganger of EthashParams.
#[derive(Debug, PartialEq, Deserialize)]
pub struct EthashParams {
	/// See main EthashParams docs.
	#[serde(rename="minimumDifficulty")]
	pub minimum_difficulty: Uint,
	/// See main EthashParams docs.
	#[serde(rename="difficultyBoundDivisor")]
	pub difficulty_bound_divisor: Uint,
	/// See main EthashParams docs.
	#[serde(rename="difficultyIncrementDivisor")]
	pub difficulty_increment_divisor: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="metropolisDifficultyIncrementDivisor")]
	pub metropolis_difficulty_increment_divisor: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="durationLimit")]
	pub duration_limit: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="homesteadTransition")]
	pub homestead_transition: Option<Uint>,

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
	pub difficulty_hardfork_bound_divisor: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="bombDefuseTransition")]
	pub bomb_defuse_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="eip100bTransition")]
	pub eip100b_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="eip150Transition")]
	pub eip150_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="eip160Transition")]
	pub eip160_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="eip161abcTransition")]
	pub eip161abc_transition: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="eip161dTransition")]
	pub eip161d_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="ecip1010PauseTransition")]
	pub ecip1010_pause_transition: Option<Uint>,
	/// See main EthashParams docs.
	#[serde(rename="ecip1010ContinueTransition")]
	pub ecip1010_continue_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="ecip1017EraRounds")]
	pub ecip1017_era_rounds: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="maxCodeSize")]
	pub max_code_size: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="maxGasLimitTransition")]
	pub max_gas_limit_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="maxGasLimit")]
	pub max_gas_limit: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="minGasPriceTransition")]
	pub min_gas_price_transition: Option<Uint>,

	/// See main EthashParams docs.
	#[serde(rename="minGasPrice")]
	pub min_gas_price: Option<Uint>,

	/// EIP-649 transition block.
	#[serde(rename="eip649Transition")]
	pub eip649_transition: Option<Uint>,

	/// EIP-649 bomb delay.
	#[serde(rename="eip649Delay")]
	pub eip649_delay: Option<Uint>,

	/// EIP-649 base reward.
	#[serde(rename="eip649Reward")]
	pub eip649_reward: Option<Uint>,
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
	use bigint::prelude::{H160, U256};
	use hash::Address;
	use spec::ethash::{Ethash, EthashParams};

	#[test]
	fn ethash_deserialization() {
		let s = r#"{
			"params": {
				"minimumDifficulty": "0x020000",
				"difficultyBoundDivisor": "0x0800",
				"durationLimit": "0x0d",
				"homesteadTransition": "0x42",
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
				"eip100bTransition": "0x42",
				"eip150Transition": "0x43",
				"eip160Transition": "0x45",
				"eip161abcTransition": "0x46",
				"eip161dTransition": "0x47"
			}
		}"#;

		let deserialized: Ethash = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Ethash {
			params: EthashParams{
				minimum_difficulty: Uint(U256::from(0x020000)),
				difficulty_bound_divisor: Uint(U256::from(0x0800)),
				difficulty_increment_divisor: None,
				metropolis_difficulty_increment_divisor: None,
				duration_limit: Some(Uint(U256::from(0x0d))),
				homestead_transition: Some(Uint(U256::from(0x42))),
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
				eip150_transition: Some(Uint(U256::from(0x43))),
				eip160_transition: Some(Uint(U256::from(0x45))),
				eip161abc_transition: Some(Uint(U256::from(0x46))),
				eip161d_transition: Some(Uint(U256::from(0x47))),
				ecip1010_pause_transition: None,
				ecip1010_continue_transition: None,
				ecip1017_era_rounds: None,
				max_code_size: None,
				max_gas_limit_transition: None,
				max_gas_limit: None,
				min_gas_price_transition: None,
				min_gas_price: None,
				eip649_transition: None,
				eip649_delay: None,
				eip649_reward: None,
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
				dao_hardfork_transition: None,
				dao_hardfork_beneficiary: None,
				dao_hardfork_accounts: None,
				difficulty_hardfork_transition: None,
				difficulty_hardfork_bound_divisor: None,
				bomb_defuse_transition: None,
				eip100b_transition: None,
				eip150_transition: None,
				eip160_transition: None,
				eip161abc_transition: None,
				eip161d_transition: None,
				ecip1010_pause_transition: None,
				ecip1010_continue_transition: None,
				ecip1017_era_rounds: None,
				max_code_size: None,
				max_gas_limit_transition: None,
				max_gas_limit: None,
				min_gas_price_transition: None,
				min_gas_price: None,
				eip649_transition: None,
				eip649_delay: None,
				eip649_reward: None,
			}
		});
	}
}
