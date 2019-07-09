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
