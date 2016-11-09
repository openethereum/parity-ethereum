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

//! Eth rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Signing methods implementation relying on unlocked accounts.
pub trait EthSigning: Sized + Send + Sync + 'static {
	/// Signs the data with given address signature.
	fn sign(&self, _: Params, _: Ready);

	/// Sends transaction; will block for 20s to try to return the
	/// transaction hash.
	/// If it cannot yet be signed, it will return a transaction ID for
	/// later use with check_transaction.
	fn send_transaction(&self, _: Params, _: Ready);

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_async_method("eth_sign", EthSigning::sign);
		delegate.add_async_method("eth_sendTransaction", EthSigning::send_transaction);

		delegate
	}
}
