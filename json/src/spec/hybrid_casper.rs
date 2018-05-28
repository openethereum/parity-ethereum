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

//! Hybrid Casper spec deserialization.

use uint::Uint;
use hash::Address;
use bytes::Bytes;

/// Hybrid Casper params deserialization.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HybridCasperParams {
	/// Main contract code.
	pub contract_code: Option<Bytes>,
	/// Address to deploy the main contract.
	pub contract_address: Option<Address>,
	/// Balance to force set in the beginning.
	pub contract_balance: Option<Uint>,
	/// Purity checker contract code.
	pub purity_checker_contract_code: Option<Bytes>,
	/// Address to deploy the purity checker.
	pub purity_checker_contract_address: Option<Address>,
	/// Msg hasher contract code.
	pub msg_hasher_contract_code: Option<Bytes>,
	/// Address to deploy the msg hasher.
	pub msg_hasher_contract_address: Option<Address>,
	/// RLP decoder contract code.
	pub rlp_decoder_contract_code: Option<Bytes>,
	/// Address to deploy the RLP decoder.
	pub rlp_decoder_contract_address: Option<Address>,
	/// Whether force-deploying the RLP decoder or not.
	pub deploy_rlp_decoder: Option<bool>,

	/// Casper epoch length.
	pub epoch_length: Option<Uint>,
	/// Casper withdrawal delay.
	pub withdrawal_delay: Option<Uint>,
	/// Casper dynasty logout delay.
	pub dynasty_logout_delay: Option<Uint>,
	/// Base interest factor passed to the Casper init function.
	pub base_interest_factor: Option<Uint>,
	/// Base penalty factor passed to the Casper init function.
	pub base_penalty_factor: Option<Uint>,
	/// Min deposit size accepted by Casper.
	pub min_deposit_size: Option<Uint>,
	/// Warm up period before vote begins.
	pub warm_up_period: Option<Uint>,
	/// Min deposit to consider a block to be justified.
	pub non_revert_min_deposits: Option<Uint>,
}
