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

//! JSONRPC interface for Whisper. Manages ephemeral identities, signing,
//! filtering by topic, and more.

use jsonrpc_core::{Error, ErrorCode};
use jsonrpc_macros::Trailing;

use futures::BoxFuture;

use self::types::PostRequest;

mod types;

// create whisper RPC error.
fn whisper_error<T: Into<String>>(message: T) -> Error {
	const ERROR_CODE: i64 = -32085;

	Error {
		code: ErrorCode::ServerError(ERROR_CODE),
		message: message.into(),
		data: None,
	}
}

build_rpc_trait! {
	/// Whisper RPC interface.
	pub trait Whisper {
		#[rpc(name = "shh_post")]
		fn post(&self, PostRequest) -> Result<bool, Error>;
	}
}

// TODO: pub-sub in a way that keeps it easy to integrate with main Parity RPC.
