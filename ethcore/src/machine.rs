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

//! Ethereum-like state machine definition.

use std::collections::BTreeMap;

use builtin::Builtin;
use header::{BlockNumber, Header};
use spec::CommonParams;

use util::{Address, U256};
use vm::{EnvInfo, Schedule, CreateContractAddress};

/// An ethereum-like state machine.
pub struct EthereumMachine {
	params: CommonParams,
	builtins: BTreeMap<Address, Builtin>,
}

impl EthereumMachine {
	/// Get the general parameters of the chain.
	pub fn params(&self) -> &CommonParams {
		&self.params
	}

	/// Get the EVM schedule for the given block number.
	pub fn schedule(&self, block_number: BlockNumber) -> Schedule {
		self.params().schedule(block_number)
	}

	/// Builtin-contracts for the chain..
	pub fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		&self.builtins
	}

	/// Attempt to get a handle to a built-in contract.
	/// Only returns references to activated built-ins.
	// TODO: builtin contract routing - to do this properly, it will require removing the built-in configuration-reading logic
	// from Spec into here and removing the Spec::builtins field.
	pub fn builtin(&self, a: &Address, block_number: BlockNumber) -> Option<&Builtin> {
		self.builtins()
			.get(a)
			.and_then(|b| if b.is_active(block_number) { Some(b) } else { None })
	}

	/// Some intrinsic operation parameters; by default they take their value from the `spec()`'s `engine_params`.
	pub fn maximum_extra_data_size(&self) -> usize { self.params().maximum_extra_data_size }

	/// The nonce with which accounts begin at given block.
	pub fn account_start_nonce(&self, block: u64) -> U256 {
		if block >= self.params().dust_protection_transition {
			U256::from(self.params().nonce_cap_increment) * U256::from(block)
		} else {
			self.params().account_start_nonce
		}
	}

	/// The network ID that transactions should be signed with.
	pub fn signing_chain_id(&self, env_info: &EnvInfo) -> Option<u64> {
		if env_info.number >= self.params().eip155_transition {
			Some(self.params().chain_id)
		} else {
			None
		}
	}

	/// Returns new contract address generation scheme at given block number.
	pub fn create_address_scheme(&self, number: BlockNumber) -> CreateContractAddress {
		if number >= self.params().eip86_transition {
			CreateContractAddress::FromCodeHash
		} else {
			CreateContractAddress::FromSenderAndNonce
		}
	}
}
