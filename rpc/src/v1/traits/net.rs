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

//! Net rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Net rpc interface.
pub trait Net: Sized + Send + Sync + 'static {
	/// Returns protocol version.
	fn version(&self, _: Params) -> Result<Value, Error> {
		rpc_unimplemented!()
	}

	/// Returns number of peers connected to node.
	fn peer_count(&self, _: Params) -> Result<Value, Error> {
		rpc_unimplemented!()
	}

	/// Returns true if client is actively listening for network connections.
	/// Otherwise false.
	fn is_listening(&self, _: Params) -> Result<Value, Error> {
		rpc_unimplemented!()
	}

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("net_version", Net::version);
		delegate.add_method("net_peerCount", Net::peer_count);
		delegate.add_method("net_listening", Net::is_listening);
		delegate
	}
}
