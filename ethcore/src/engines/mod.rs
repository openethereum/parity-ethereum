// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Consensus engine specification and basic implementations.

mod authority_round;
mod basic_authority;
mod clique;
mod ethash;
mod instant_seal;
mod null_engine;
mod validator_set;

pub mod block_reward;

pub use self::authority_round::AuthorityRound;
pub use self::basic_authority::BasicAuthority;
pub use self::instant_seal::{InstantSeal, InstantSealParams};
pub use self::null_engine::NullEngine;
pub use self::clique::Clique;
pub use self::ethash::{Ethash, Seal as EthashSeal};

// TODO [ToDr] Remove re-export (#10130)
pub use types::engines::ForkChoice;
pub use types::engines::epoch::{self, Transition as EpochTransition};

use std::sync::Arc;

use vm::{CallType, ActionValue};
use types::{
	engines::{
		Headers, PendingTransitionStore,
	},
};

use machine::{
	Machine,
	executed_block::ExecutedBlock,
};
use ethereum_types::{H256, U256, Address};

/// A system-calling closure. Enacts calls on a block's state from the system address.
pub type SystemCall<'a> = dyn FnMut(Address, Vec<u8>) -> Result<Vec<u8>, String> + 'a;

/// A system-calling closure. Enacts calls on a block's state with code either from an on-chain contract, or hard-coded EVM or WASM (if enabled on-chain) codes.
pub type SystemOrCodeCall<'a> = dyn FnMut(SystemOrCodeCallKind, Vec<u8>) -> Result<Vec<u8>, String> + 'a;

/// Kind of SystemOrCodeCall, this is either an on-chain address, or code.
#[derive(PartialEq, Debug, Clone)]
pub enum SystemOrCodeCallKind {
	/// On-chain address.
	Address(Address),
	/// Hard-coded code.
	Code(Arc<Vec<u8>>, H256),
}

/// Default SystemOrCodeCall implementation.
pub fn default_system_or_code_call<'a>(machine: &'a Machine, block: &'a mut ExecutedBlock) -> impl FnMut(SystemOrCodeCallKind, Vec<u8>) -> Result<Vec<u8>, String> + 'a {
	move |to, data| {
		let result = match to {
			SystemOrCodeCallKind::Address(address) => {
				machine.execute_as_system(
					block,
					address,
					U256::max_value(),
					Some(data),
				)
			},
			SystemOrCodeCallKind::Code(code, code_hash) => {
				machine.execute_code_as_system(
					block,
					None,
					Some(code),
					Some(code_hash),
					Some(ActionValue::Apparent(U256::zero())),
					U256::max_value(),
					Some(data),
					Some(CallType::StaticCall),
				)
			},
		};

		result.map_err(|e| format!("{}", e))
	}
}
