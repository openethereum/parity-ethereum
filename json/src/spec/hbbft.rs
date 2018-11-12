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

//! Honey Badger BFT engine params deserialization.

use hash::Address;
use spec::ValidatorSet;
use uint::Uint;

/// `Hbbft` engine configuration parameters, this structure is used to deserialize the values found
/// in the `Hbbft` engine's JSON spec.
#[derive(Debug, PartialEq, Deserialize)]
pub struct HbbftParams {
	/// Whether to enable millisecond timestamp.
	#[serde(rename="millisecondTimestamp")]
	#[serde(default)]
	pub millisecond_timestamp: bool,

	/// The block range for which each validator-set source should be used. A "validator-set
	/// source" is either a hardcoded list of addresses or a smart contract address to query the
	/// validator set from. The `validators` configuration option conveys information of the form:
	/// "from block #0 to #50 use a constant list of validator addresses, for every block sealed
	/// after block #50, query the set of validator addresses from a smart contract which is
	/// deployed at some address".
	pub validators: ValidatorSet,

	/// The address at which the block reward contract is deployed. The block reward contract is
	/// used to calculate block rewards, to create new coin after each new block is sealed, and to
	/// distribute the created coin as block rewards to a set of recipient addresses.
	pub block_reward_contract_address: Option<Address>,

	/// If no `block_reward_contract_address` is provided, the `block_reward` configuration option
	/// can be used to set a constant block reward ammount to be transfered to the address found in
	/// each newly sealed block header's `author` field. If neither the
	/// `block_reward_contract_address` nor `block_reward` options are supplied in the Hbbft
	/// engine's JSON spec, no block rewards will be distributed. If values are provided for both
	/// `block_reward_contract_address` and `block_reward`, the value for `block_reward` will be
	/// ignored and only the contract will be queried to determine reward ammounts.
	pub block_reward: Option<Uint>,
}

/// Honey Badger BFT engine descriptor.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Hbbft {
	pub params: HbbftParams,
}
