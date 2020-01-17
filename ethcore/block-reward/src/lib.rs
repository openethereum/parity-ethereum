// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Types for declaring block rewards and a client interface for interacting with a
//! block reward contract.

use std::sync::Arc;

use ethabi::FunctionOutputDecoder;
use ethabi_contract::use_contract;
use ethereum_types::{Address, U256};
use common_types::{
	BlockNumber,
	errors::{EngineError, EthcoreError as Error},
};
use keccak_hash::keccak;
use machine::{Machine, ExecutedBlock};
use engine::{SystemOrCodeCall, SystemOrCodeCallKind};
use trace;
use trace::{Tracer, ExecutiveTracer, Tracing};

use_contract!(block_reward_contract, "res/block_reward.json");

/// The kind of block reward.
/// Depending on the consensus engine the allocated block reward might have
/// different semantics which could lead e.g. to different reward values.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RewardKind {
	/// Reward attributed to the block author.
	Author,
	/// Reward attributed to the author(s) of empty step(s) included in the block (AuthorityRound engine).
	EmptyStep,
	/// Reward attributed by an external protocol (e.g. block reward contract).
	External,
	/// Reward attributed to the block uncle(s) with given difference.
	Uncle(u8),
}

impl RewardKind {
	/// Create `RewardKind::Uncle` from given current block number and uncle block number.
	pub fn uncle(number: BlockNumber, uncle: BlockNumber) -> Self {
		RewardKind::Uncle(if number > uncle && number - uncle <= u8::max_value().into() { (number - uncle) as u8 } else { 0 })
	}
}

impl From<RewardKind> for u16 {
	fn from(reward_kind: RewardKind) -> Self {
		match reward_kind {
			RewardKind::Author => 0,
			RewardKind::EmptyStep => 2,
			RewardKind::External => 3,

			RewardKind::Uncle(depth) => 100 + depth as u16,
		}
	}
}

impl Into<trace::RewardType> for RewardKind {
	fn into(self) -> trace::RewardType {
		match self {
			RewardKind::Author => trace::RewardType::Block,
			RewardKind::Uncle(_) => trace::RewardType::Uncle,
			RewardKind::EmptyStep => trace::RewardType::EmptyStep,
			RewardKind::External => trace::RewardType::External,
		}
	}
}

/// A client for the block reward contract.
#[derive(PartialEq, Debug)]
pub struct BlockRewardContract {
	kind: SystemOrCodeCallKind,
}

impl BlockRewardContract {
	/// Create a new block reward contract client targeting the system call kind.
	pub fn new(kind: SystemOrCodeCallKind) -> BlockRewardContract {
		BlockRewardContract {
			kind,
		}
	}

	/// Create a new block reward contract client targeting the contract address.
	pub fn new_from_address(address: Address) -> BlockRewardContract {
		Self::new(SystemOrCodeCallKind::Address(address))
	}

	/// Create a new block reward contract client targeting the given code.
	pub fn new_from_code(code: Arc<Vec<u8>>) -> BlockRewardContract {
		let code_hash = keccak(&code[..]);

		Self::new(SystemOrCodeCallKind::Code(code, code_hash))
	}

	/// Calls the block reward contract with the given beneficiaries list (and associated reward kind)
	/// and returns the reward allocation (address - value). The block reward contract *must* be
	/// called by the system address so the `caller` must ensure that (e.g. using
	/// `machine.execute_as_system`).
	pub fn reward(
		&self,
		beneficiaries: Vec<(Address, RewardKind)>,
		caller: &mut SystemOrCodeCall,
	) -> Result<Vec<(Address, U256)>, Error> {
		let (addresses, rewards): (Vec<_>, Vec<_>) = beneficiaries.into_iter().unzip();
		let (input, decoder) = block_reward_contract::functions::reward::call(addresses, rewards.into_iter().map(u16::from));

		let output = caller(self.kind.clone(), input)
			.map_err(Into::into)
			.map_err(EngineError::FailedSystemCall)?;

		let (addresses, rewards) = decoder.decode(&output)
			.map_err(|err| err.to_string())
			.map_err(EngineError::FailedSystemCall)?;

		if addresses.len() != rewards.len() {
			return Err(EngineError::FailedSystemCall(
				"invalid data returned by reward contract: both arrays must have the same size".into()
			).into());
		}

		Ok(addresses.into_iter().zip(rewards.into_iter()).collect())
	}
}

/// Applies the given block rewards, i.e. adds the given balance to each beneficiary' address.
/// If tracing is enabled the operations are recorded.
pub fn apply_block_rewards(
	rewards: &[(Address, RewardKind, U256)],
	block: &mut ExecutedBlock,
	machine: &Machine,
) -> Result<(), Error> {
	for &(ref author, _, ref block_reward) in rewards {
		machine.add_balance(block, author, block_reward)?;
	}

	if let Tracing::Enabled(ref mut traces) = *block.traces_mut() {
		let mut tracer = ExecutiveTracer::default();

		for &(address, reward_kind, amount) in rewards {
			tracer.trace_reward(address, amount, reward_kind.into());
		}

		traces.push(tracer.drain().into());
	}

	Ok(())
}

#[cfg(test)]
mod test {
	use std::str::FromStr;
	use ethcore::{
		client::PrepareOpenBlock,
		test_helpers::generate_dummy_client_with_spec,
	};
	use ethereum_types::{U256, Address};
	use engine::SystemOrCodeCallKind;
	use spec;

	use crate::{BlockRewardContract, RewardKind};

	#[test]
	fn block_reward_contract() {
		let client = generate_dummy_client_with_spec(spec::new_test_round_block_reward_contract);

		let machine = spec::new_test_machine();

		// the spec has a block reward contract defined at the given address
		let block_reward_contract = BlockRewardContract::new_from_address(
			Address::from_str("0000000000000000000000000000000000000042").unwrap(),
		);

		let mut call = |to, data| {
			let mut block = client.prepare_open_block(
				Address::from_str("0000000000000000000000000000000000000001").unwrap(),
				(3141562.into(), 31415620.into()),
				vec![],
			).unwrap();

			let result = match to {
				SystemOrCodeCallKind::Address(to) => {
					machine.execute_as_system(
						block.block_mut(),
						to,
						U256::max_value(),
						Some(data),
					)
				},
				_ => panic!("Test reward contract is created by an address, we never reach this branch."),
			};

			result.map_err(|e| format!("{}", e))
		};

		// if no beneficiaries are given no rewards are attributed
		assert!(block_reward_contract.reward(vec![], &mut call).unwrap().is_empty());

		// the contract rewards (1000 + kind) for each benefactor
		let beneficiaries = vec![
			(Address::from_str("0000000000000000000000000000000000000033").unwrap(), RewardKind::Author),
			(Address::from_str("0000000000000000000000000000000000000034").unwrap(), RewardKind::Uncle(1)),
			(Address::from_str("0000000000000000000000000000000000000035").unwrap(), RewardKind::EmptyStep),
		];

		let rewards = block_reward_contract.reward(beneficiaries, &mut call).unwrap();
		let expected = vec![
			(Address::from_str("0000000000000000000000000000000000000033").unwrap(), U256::from(1000)),
			(Address::from_str("0000000000000000000000000000000000000034").unwrap(), U256::from(1000 + 101)),
			(Address::from_str("0000000000000000000000000000000000000035").unwrap(), U256::from(1000 + 2)),
		];

		assert_eq!(expected, rewards);
	}
}
