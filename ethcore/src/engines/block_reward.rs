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

//! A module with types for declaring block rewards and a client interface for interacting with a
//! block reward contract.

use ethabi;
use ethabi::ParamType;
use ethereum_types::{H160, Address, U256};

use error::Error;
use machine::WithRewards;
use parity_machine::{Machine, WithBalances};
use trace;
use super::SystemCall;

use_contract!(block_reward_contract, "BlockReward", "res/contracts/block_reward.json");

/// The kind of block reward.
/// Depending on the consensus engine the allocated block reward might have
/// different semantics which could lead e.g. to different reward values.
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RewardKind {
	/// Reward attributed to the block author.
	Author = 0,
	/// Reward attributed to the block uncle(s).
	Uncle = 1,
	/// Reward attributed to the author(s) of empty step(s) included in the block (AuthorityRound engine).
	EmptyStep = 2,
	/// Reward attributed by an external protocol (e.g. block reward contract).
	External = 3,
}

impl From<RewardKind> for u16 {
	fn from(reward_kind: RewardKind) -> Self {
		reward_kind as u16
	}
}

impl Into<trace::RewardType> for RewardKind {
	fn into(self) -> trace::RewardType {
		match self {
			RewardKind::Author => trace::RewardType::Block,
			RewardKind::Uncle => trace::RewardType::Uncle,
			RewardKind::EmptyStep => trace::RewardType::EmptyStep,
			RewardKind::External => trace::RewardType::External,
		}
	}
}

/// A client for the block reward contract.
pub struct BlockRewardContract {
	/// Address of the contract.
	address: Address,
	block_reward_contract: block_reward_contract::BlockReward,
}

impl BlockRewardContract {
	/// Create a new block reward contract client targeting the given address.
	pub fn new(address: Address) -> BlockRewardContract {
		BlockRewardContract {
			address,
			block_reward_contract: block_reward_contract::BlockReward::default(),
		}
	}

	/// Calls the block reward contract with the given benefactors list (and associated reward kind)
	/// and returns the reward allocation (address - value). The block reward contract *must* be
	/// called by the system address so the `caller` must ensure that (e.g. using
	/// `machine.execute_as_system`).
	pub fn reward(
		&self,
		benefactors: &[(Address, RewardKind)],
		caller: &mut SystemCall,
	) -> Result<Vec<(Address, U256)>, Error> {
		let reward = self.block_reward_contract.functions().reward();

		let input = reward.input(
			benefactors.iter().map(|&(address, _)| H160::from(address)),
			benefactors.iter().map(|&(_, ref reward_kind)| u16::from(*reward_kind)),
		);

		let output = caller(self.address, input)
			.map_err(Into::into)
			.map_err(::engines::EngineError::FailedSystemCall)?;

		// since this is a non-constant call we can't use ethabi's function output
		// deserialization, sadness ensues.
		let types = &[
			ParamType::Array(Box::new(ParamType::Address)),
			ParamType::Array(Box::new(ParamType::Uint(256))),
		];

		let tokens = ethabi::decode(types, &output)
			.map_err(|err| err.to_string())
			.map_err(::engines::EngineError::FailedSystemCall)?;

		assert!(tokens.len() == 2);

		let addresses = tokens[0].clone().to_array().expect("type checked by ethabi::decode; qed");
		let rewards = tokens[1].clone().to_array().expect("type checked by ethabi::decode; qed");

		if addresses.len() != rewards.len() {
			return Err(::engines::EngineError::FailedSystemCall(
				"invalid data returned by reward contract: both arrays must have the same size".into()
			).into());
		}

		let addresses = addresses.into_iter().map(|t| t.to_address().expect("type checked by ethabi::decode; qed"));
		let rewards = rewards.into_iter().map(|t| t.to_uint().expect("type checked by ethabi::decode; qed"));

		Ok(addresses.zip(rewards).collect())
	}
}

/// Applies the given block rewards, i.e. adds the given balance to each benefactors' address.
/// If tracing is enabled the operations are recorded.
pub fn apply_block_rewards<M: Machine + WithBalances + WithRewards>(
	rewards: &[(Address, RewardKind, U256)],
	block: &mut M::LiveBlock,
	machine: &M,
) -> Result<(), M::Error> {
	for &(ref author, _, ref block_reward) in rewards {
		machine.add_balance(block, author, block_reward)?;
	}

	let rewards: Vec<_> = rewards.into_iter().map(|&(a, k, r)| (a, k.into(), r)).collect();
	machine.note_rewards(block,  &rewards)
}

#[cfg(test)]
mod test {
	use client::PrepareOpenBlock;
	use ethereum_types::U256;
	use spec::Spec;
	use test_helpers::generate_dummy_client_with_spec_and_accounts;

	use super::{BlockRewardContract, RewardKind};

	#[test]
	fn block_reward_contract() {
		let client = generate_dummy_client_with_spec_and_accounts(
			Spec::new_test_round_block_reward_contract,
			None,
		);

		let machine = Spec::new_test_machine();

		// the spec has a block reward contract defined at the given address
		let block_reward_contract = BlockRewardContract::new(
			"0000000000000000000000000000000000000042".into(),
		);

		let mut call = |to, data| {
			let mut block = client.prepare_open_block(
				"0000000000000000000000000000000000000001".into(),
				(3141562.into(), 31415620.into()),
				vec![],
			);

			let result = machine.execute_as_system(
				block.block_mut(),
				to,
				U256::max_value(),
				Some(data),
			);

			result.map_err(|e| format!("{}", e))
		};

		// if no benefactors are given no rewards are attributed
		assert!(block_reward_contract.reward(&vec![], &mut call).unwrap().is_empty());

		// the contract rewards (1000 + kind) for each benefactor
		let benefactors = vec![
			("0000000000000000000000000000000000000033".into(), RewardKind::Author),
			("0000000000000000000000000000000000000034".into(), RewardKind::Uncle),
			("0000000000000000000000000000000000000035".into(), RewardKind::EmptyStep),
		];

		let rewards = block_reward_contract.reward(&benefactors, &mut call).unwrap();
		let expected = vec![
			("0000000000000000000000000000000000000033".into(), U256::from(1000)),
			("0000000000000000000000000000000000000034".into(), U256::from(1000 + 1)),
			("0000000000000000000000000000000000000035".into(), U256::from(1000 + 2)),
		];

		assert_eq!(expected, rewards);
	}
}
