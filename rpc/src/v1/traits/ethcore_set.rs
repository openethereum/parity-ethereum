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

use std::sync::Arc;
use jsonrpc_core::*;

/// Ethcore-specific rpc interface for operations altering the settings.
pub trait EthcoreSet: Sized + Send + Sync + 'static {

	/// Sets new minimal gas price for mined blocks.
	fn set_min_gas_price(&self, _: Params) -> Result<Value, Error>;

	/// Sets new gas floor target for mined blocks.
	fn set_gas_floor_target(&self, _: Params) -> Result<Value, Error>;

	/// Sets new extra data for mined blocks.
	fn set_extra_data(&self, _: Params) -> Result<Value, Error>;

	/// Sets new author for mined block.
	fn set_author(&self, _: Params) -> Result<Value, Error>;

	/// Sets the limits for transaction queue.
	fn set_transactions_limit(&self, _: Params) -> Result<Value, Error>;

	/// Add a reserved peer.
	fn add_reserved_peer(&self, _: Params) -> Result<Value, Error>;

	/// Remove a reserved peer.
	fn remove_reserved_peer(&self, _: Params) -> Result<Value, Error>;

	/// Drop all non-reserved peers.
	fn drop_non_reserved_peers(&self, _: Params) -> Result<Value, Error>;

	/// Accept non-reserved peers (default behavior)
	fn accept_non_reserved_peers(&self, _: Params) -> Result<Value, Error>;

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("ethcore_setMinGasPrice", EthcoreSet::set_min_gas_price);
		delegate.add_method("ethcore_setGasFloorTarget", EthcoreSet::set_gas_floor_target);
		delegate.add_method("ethcore_setExtraData", EthcoreSet::set_extra_data);
		delegate.add_method("ethcore_setAuthor", EthcoreSet::set_author);
		delegate.add_method("ethcore_setTransactionsLimit", EthcoreSet::set_transactions_limit);
		delegate.add_method("ethcore_addReservedPeer", EthcoreSet::add_reserved_peer);
		delegate.add_method("ethcore_removeReservedPeer", EthcoreSet::remove_reserved_peer);
		delegate.add_method("ethcore_dropNonReservedPeers", EthcoreSet::drop_non_reserved_peers);
		delegate.add_method("ethcore_acceptNonReservedPeers", EthcoreSet::accept_non_reserved_peers);

		delegate
	}
}
