// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Ethcore-specific rpc interface for operations altering the settings.

use jsonrpc_core::Error;

use v1::helpers::auto_args::Wrap;
use v1::types::{Bytes, H160, U256};

build_rpc_trait! {
	/// Ethcore-specific rpc interface for operations altering the settings.
	pub trait EthcoreSet {
		/// Sets new minimal gas price for mined blocks.
		#[rpc(name = "ethcore_setMinGasPrice")]
		fn set_min_gas_price(&self, U256) -> Result<bool, Error>;

		/// Sets new gas floor target for mined blocks.
		#[rpc(name = "ethcore_setGasFloorTarget")]
		fn set_gas_floor_target(&self, U256) -> Result<bool, Error>;

		/// Sets new gas ceiling target for mined blocks.
		#[rpc(name = "ethcore_setGasCeilTarget")]
		fn set_gas_ceil_target(&self, U256) -> Result<bool, Error>;

		/// Sets new extra data for mined blocks.
		#[rpc(name = "ethcore_setExtraData")]
		fn set_extra_data(&self, Bytes) -> Result<bool, Error>;

		/// Sets new author for mined block.
		#[rpc(name = "ethcore_setAuthor")]
		fn set_author(&self, H160) -> Result<bool, Error>;

		/// Sets the limits for transaction queue.
		#[rpc(name = "ethcore_setTransactionsLimit")]
		fn set_transactions_limit(&self, usize) -> Result<bool, Error>;

		/// Sets the maximum amount of gas a single transaction may consume.
		#[rpc(name = "ethcore_setMaxTransactionGas")]
		fn set_tx_gas_limit(&self, U256) -> Result<bool, Error>;

		/// Add a reserved peer.
		#[rpc(name = "ethcore_addReservedPeer")]
		fn add_reserved_peer(&self, String) -> Result<bool, Error>;

		/// Remove a reserved peer.
		#[rpc(name = "ethcore_removeReservedPeer")]
		fn remove_reserved_peer(&self, String) -> Result<bool, Error>;

		/// Drop all non-reserved peers.
		#[rpc(name = "ethcore_dropNonReservedPeers")]
		fn drop_non_reserved_peers(&self) -> Result<bool, Error>;

		/// Accept non-reserved peers (default behavior)
		#[rpc(name = "ethcore_acceptNonReservedPeers")]
		fn accept_non_reserved_peers(&self) -> Result<bool, Error>;

		/// Start the network.
		#[rpc(name = "ethcore_startNetwork")]
		fn start_network(&self) -> Result<bool, Error>;

		/// Stop the network.
		#[rpc(name = "ethcore_stopNetwork")]
		fn stop_network(&self) -> Result<bool, Error>;
	}
}