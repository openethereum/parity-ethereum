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

//! Parity-specific rpc interface for operations altering the settings.

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_derive::rpc;

use v1::types::{Bytes, H160, H256, U256, ReleaseInfo, Transaction};

/// Parity-specific rpc interface for operations altering the settings.
#[rpc]
pub trait ParitySet {
	/// Sets new minimal gas price for mined blocks.
	#[rpc(name = "parity_setMinGasPrice")]
	fn set_min_gas_price(&self, U256) -> Result<bool>;

	/// Sets new gas floor target for mined blocks.
	#[rpc(name = "parity_setGasFloorTarget")]
	fn set_gas_floor_target(&self, U256) -> Result<bool>;

	/// Sets new gas ceiling target for mined blocks.
	#[rpc(name = "parity_setGasCeilTarget")]
	fn set_gas_ceil_target(&self, U256) -> Result<bool>;

	/// Sets new extra data for mined blocks.
	#[rpc(name = "parity_setExtraData")]
	fn set_extra_data(&self, Bytes) -> Result<bool>;

	/// Sets new author for mined block.
	#[rpc(name = "parity_setAuthor")]
	fn set_author(&self, H160) -> Result<bool>;

	/// Sets account for signing consensus messages.
	#[rpc(name = "parity_setEngineSigner")]
	fn set_engine_signer(&self, H160, String) -> Result<bool>;

	/// Sets the limits for transaction queue.
	#[rpc(name = "parity_setTransactionsLimit")]
	fn set_transactions_limit(&self, usize) -> Result<bool>;

	/// Sets the maximum amount of gas a single transaction may consume.
	#[rpc(name = "parity_setMaxTransactionGas")]
	fn set_tx_gas_limit(&self, U256) -> Result<bool>;

	/// Add a reserved peer.
	#[rpc(name = "parity_addReservedPeer")]
	fn add_reserved_peer(&self, String) -> Result<bool>;

	/// Remove a reserved peer.
	#[rpc(name = "parity_removeReservedPeer")]
	fn remove_reserved_peer(&self, String) -> Result<bool>;

	/// Drop all non-reserved peers.
	#[rpc(name = "parity_dropNonReservedPeers")]
	fn drop_non_reserved_peers(&self) -> Result<bool>;

	/// Accept non-reserved peers (default behavior)
	#[rpc(name = "parity_acceptNonReservedPeers")]
	fn accept_non_reserved_peers(&self) -> Result<bool>;

	/// Start the network.
	///
	/// @deprecated - Use `set_mode("active")` instead.
	#[rpc(name = "parity_startNetwork")]
	fn start_network(&self) -> Result<bool>;

	/// Stop the network.
	///
	/// @deprecated - Use `set_mode("offline")` instead.
	#[rpc(name = "parity_stopNetwork")]
	fn stop_network(&self) -> Result<bool>;

	/// Set the mode. Argument must be one of: "active", "passive", "dark", "offline".
	#[rpc(name = "parity_setMode")]
	fn set_mode(&self, String) -> Result<bool>;

	/// Set the network spec. Argument must be one of pre-configured chains or a filename.
	#[rpc(name = "parity_setChain")]
	fn set_spec_name(&self, String) -> Result<bool>;

	/// Hash a file content under given URL.
	#[rpc(name = "parity_hashContent")]
	fn hash_content(&self, String) -> BoxFuture<H256>;

	/// Is there a release ready for install?
	#[rpc(name = "parity_upgradeReady")]
	fn upgrade_ready(&self) -> Result<Option<ReleaseInfo>>;

	/// Execute a release which is ready according to upgrade_ready().
	#[rpc(name = "parity_executeUpgrade")]
	fn execute_upgrade(&self) -> Result<bool>;

	/// Removes transaction from transaction queue.
	/// Makes sense only for transactions that were not propagated to other peers yet
	/// like scheduled transactions or transactions in future.
	/// It might also work for some local transactions with to low gas price
	/// or excessive gas limit that are not accepted by other peers whp.
	/// Returns `true` when transaction was removed, `false` if it was not found.
	#[rpc(name = "parity_removeTransaction")]
	fn remove_transaction(&self, H256) -> Result<Option<Transaction>>;
}
