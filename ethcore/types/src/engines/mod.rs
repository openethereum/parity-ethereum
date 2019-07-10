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

//! Engine-specific types.

use ethereum_types::Address;
use ethjson;

use crate::BlockNumber;

pub mod epoch;
pub mod params;

/// The number of generations back that uncles can be.
// todo: hook up in ethcore
pub const MAX_UNCLE_AGE: usize = 6;

/// Default EIP-210 contract code.
/// As defined in https://github.com/ethereum/EIPs/pull/210
// todo: why can't I pre-hex this so we don't have to import rustc-hex?
pub const DEFAULT_BLOCKHASH_CONTRACT: &'static str = "73fffffffffffffffffffffffffffffffffffffffe33141561006a5760014303600035610100820755610100810715156100455760003561010061010083050761010001555b6201000081071515610064576000356101006201000083050761020001555b5061013e565b4360003512151561008457600060405260206040f361013d565b61010060003543031315156100a857610100600035075460605260206060f361013c565b6101006000350715156100c55762010000600035430313156100c8565b60005b156100ea576101006101006000350507610100015460805260206080f361013b565b620100006000350715156101095763010000006000354303131561010c565b60005b1561012f57610100620100006000350507610200015460a052602060a0f361013a565b600060c052602060c0f35b5b5b5b5b";

/// Fork choice.
#[derive(Debug, PartialEq, Eq)]
pub enum ForkChoice {
	/// Choose the new block.
	New,
	/// Choose the current best block.
	Old,
}

/// Ethash-specific extensions.
#[derive(Debug, Clone)]
pub struct EthashExtensions {
	/// Homestead transition block number.
	pub homestead_transition: BlockNumber,
	/// DAO hard-fork transition block (X).
	pub dao_hardfork_transition: u64,
	/// DAO hard-fork refund contract address (C).
	pub dao_hardfork_beneficiary: Address,
	/// DAO hard-fork DAO accounts list (L)
	pub dao_hardfork_accounts: Vec<Address>,
}

impl From<ethjson::spec::EthashParams> for EthashExtensions {
	fn from(p: ::ethjson::spec::EthashParams) -> Self {
		EthashExtensions {
			homestead_transition: p.homestead_transition.map_or(0, Into::into),
			dao_hardfork_transition: p.dao_hardfork_transition.map_or(u64::max_value(), Into::into),
			dao_hardfork_beneficiary: p.dao_hardfork_beneficiary.map_or_else(Address::zero, Into::into),
			dao_hardfork_accounts: p.dao_hardfork_accounts.unwrap_or_else(Vec::new).into_iter().map(Into::into).collect(),
		}
	}
}
