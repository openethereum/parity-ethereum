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

//! Ethcore-specific rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Ethcore-specific rpc interface.
pub trait Ethcore: Sized + Send + Sync + 'static {

	/// Sets new minimal gas price for mined blocks.
	fn set_min_gas_price(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Sets new gas floor target for mined blocks.
	fn set_gas_floor_target(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Sets new extra data for mined blocks.
	fn set_extra_data(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Sets new author for mined block.
	fn set_author(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Sets the limits for transaction queue.
	fn set_transactions_limit(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns current transactions limit.
	fn transactions_limit(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns mining extra data.
	fn extra_data(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns mining gas floor target.
	fn gas_floor_target(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns minimal gas price for transaction to be included in queue.
	fn min_gas_price(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("ethcore_setMinGasPrice", Ethcore::set_min_gas_price);
		delegate.add_method("ethcore_setGasFloorTarget", Ethcore::set_gas_floor_target);
		delegate.add_method("ethcore_setExtraData", Ethcore::set_extra_data);
		delegate.add_method("ethcore_setAuthor", Ethcore::set_author);
		delegate.add_method("ethcore_setTransactionsLimit", Ethcore::set_transactions_limit);

		delegate.add_method("ethcore_extraData", Ethcore::extra_data);
		delegate.add_method("ethcore_gasFloorTarget", Ethcore::gas_floor_target);
		delegate.add_method("ethcore_minGasPrice", Ethcore::min_gas_price);
		delegate.add_method("ethcore_transactionsLimit", Ethcore::transactions_limit);
		delegate
	}
}
