// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use jsonrpc_core;

/// RPC methods metadata.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Metadata {
	/// Current dapplication identifier
	pub dapp_id: Option<String>,
	/// Request origin
	pub origin: Origin,
}

/// RPC request origin
#[derive(Clone, Debug, PartialEq)]
pub enum Origin {
	/// RPC server
	Rpc,
	/// Dapps server
	Dapps,
	/// IPC server
	Ipc,
	/// Signer
	Signer,
	/// Unknown
	Unknown,
}

impl Default for Origin {
	fn default() -> Self {
		Origin::Unknown
	}
}

impl jsonrpc_core::Metadata for Metadata {}

