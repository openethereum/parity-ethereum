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

//! Web3 rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Web3 rpc interface.
pub trait Web3: Sized + Send + Sync + 'static {
	/// Returns current client version.
	fn client_version(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Returns sha3 of the given data
	fn sha3(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("web3_clientVersion", Web3::client_version);
		delegate.add_method("web3_sha3", Web3::sha3);
		delegate
	}
}
