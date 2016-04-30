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

//! RPC interface.

use std::sync::Arc;
use jsonrpc_core::*;

/// RPC Interface.
pub trait Rpc: Sized + Send + Sync + 'static {

	/// Returns supported modules.
	fn modules(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		// Geth 1.3.6 compatibility
		delegate.add_method("modules", Rpc::modules);
		// Geth 1.4.0 compatibility
		delegate.add_method("rpc_modules", Rpc::modules);
		delegate
	}
}

