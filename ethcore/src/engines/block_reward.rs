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

use ethabi;
use ethabi::ParamType;
use ethereum_types::{H160, Address, U256};

use block::ExecutedBlock;
use error::Error;
use machine::EthereumMachine;

pub type SystemCall<'a> = FnMut(Address, Vec<u8>) -> Result<Vec<u8>, String> + 'a;

use_contract!(block_reward_contract, "BlockReward", "res/contracts/block_reward.json");

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RewardKind {
	Author = 0,
	Uncle = 1,
	EmptyStep = 2,
}

impl From<RewardKind> for u16 {
	fn from(reward_type: RewardKind) -> Self {
		reward_type as u16
	}
}

struct BlockRewardContract {
	address: Address,
	block_reward_contract: block_reward_contract::BlockReward,
}

impl BlockRewardContract {
	fn new(address: Address) -> BlockRewardContract {
		BlockRewardContract {
			address,
			block_reward_contract: block_reward_contract::BlockReward::default(),
		}
	}

	fn reward(
		&self,
		benefactors: &[(Address, RewardKind)],
		caller: &mut SystemCall,
	) -> Result<Vec<(Address, U256)>, Error> {
		let reward = self.block_reward_contract.functions().reward();

		let input = reward.input(
			benefactors.iter().map(|&(address, _)| H160::from(address)),
			benefactors.iter().map(|&(_, ref reward_type)| u16::from(*reward_type)),
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

fn apply_block_rewards(rewards: &[(Address, U256)], block: &mut ExecutedBlock, machine: &EthereumMachine) -> Result<(), Error> {
	use parity_machine::WithBalances;

	for &(ref author, ref block_reward) in rewards {
		machine.add_balance(block, author, block_reward)?;
	}

	machine.note_rewards(block, &rewards, &[])
}
