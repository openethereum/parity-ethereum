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

	/// Returns current transactions limit.
	fn transactions_limit(&self, _: Params) -> Result<Value, Error>;

	/// Returns mining extra data.
	fn extra_data(&self, _: Params) -> Result<Value, Error>;

	/// Returns mining gas floor target.
	fn gas_floor_target(&self, _: Params) -> Result<Value, Error>;

	/// Returns minimal gas price for transaction to be included in queue.
	fn min_gas_price(&self, _: Params) -> Result<Value, Error>;

	/// Returns latest logs
	fn dev_logs(&self, _: Params) -> Result<Value, Error>;

	/// Returns logs levels
	fn dev_logs_levels(&self, _: Params) -> Result<Value, Error>;

	/// Returns chain name
	fn net_chain(&self, _: Params) -> Result<Value, Error>;

	/// Returns max peers
	fn net_max_peers(&self, _: Params) -> Result<Value, Error>;

	/// Returns network port
	fn net_port(&self, _: Params) -> Result<Value, Error>;

	/// Returns rpc settings
	fn rpc_settings(&self, _: Params) -> Result<Value, Error>;

	/// Returns node name
	fn node_name(&self, _: Params) -> Result<Value, Error>;

	/// Returns default extra data
	fn default_extra_data(&self, _: Params) -> Result<Value, Error>;

	/// Returns distribution of gas price in latest blocks.
	fn gas_price_statistics(&self, _: Params) -> Result<Value, Error>;

	/// Returns number of unsigned transactions waiting in the signer queue (if signer enabled)
	/// Returns error when signer is disabled
	fn unsigned_transactions_count(&self, _: Params) -> Result<Value, Error>;

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));

		delegate.add_method("ethcore_extraData", Ethcore::extra_data);
		delegate.add_method("ethcore_gasFloorTarget", Ethcore::gas_floor_target);
		delegate.add_method("ethcore_minGasPrice", Ethcore::min_gas_price);
		delegate.add_method("ethcore_transactionsLimit", Ethcore::transactions_limit);
		delegate.add_method("ethcore_devLogs", Ethcore::dev_logs);
		delegate.add_method("ethcore_devLogsLevels", Ethcore::dev_logs_levels);
		delegate.add_method("ethcore_netChain", Ethcore::net_chain);
		delegate.add_method("ethcore_netMaxPeers", Ethcore::net_max_peers);
		delegate.add_method("ethcore_netPort", Ethcore::net_port);
		delegate.add_method("ethcore_rpcSettings", Ethcore::rpc_settings);
		delegate.add_method("ethcore_nodeName", Ethcore::node_name);
		delegate.add_method("ethcore_defaultExtraData", Ethcore::default_extra_data);
		delegate.add_method("ethcore_gasPriceStatistics", Ethcore::gas_price_statistics);
		delegate.add_method("ethcore_unsignedTransactionsCount", Ethcore::unsigned_transactions_count);

		delegate
	}
}
