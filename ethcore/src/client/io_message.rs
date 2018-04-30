// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::fmt;
use std::sync::Arc;
use bytes::Bytes;
use client::Client;
use ethereum_types::H256;
use snapshot::ManifestData;

/// Message type for external and internal events
#[derive(Clone, Debug)]
pub enum ClientIoMessage {
	/// Best Block Hash in chain has been changed
	NewChainHead,
	/// A block is ready
	BlockVerified,
	/// Begin snapshot restoration
	BeginRestoration(ManifestData),
	/// Feed a state chunk to the snapshot service
	FeedStateChunk(H256, Bytes),
	/// Feed a block chunk to the snapshot service
	FeedBlockChunk(H256, Bytes),
	/// Take a snapshot for the block with given number.
	TakeSnapshot(u64),
	/// New consensus message received.
	NewMessage(Bytes),
	/// New private transaction arrived
	NewPrivateTransaction,
	/// Execute wrapped closure
	Execute(Callback),
}

/// A function to invoke in the client thread.
#[derive(Clone)]
pub struct Callback(pub Arc<Fn(&Client) + Send + Sync>);

impl Callback {
	pub fn new<T: Fn(&Client) + Send + Sync + 'static>(fun: T) -> Self {
		Callback(Arc::new(fun))
	}
}

impl From<Callback> for ClientIoMessage {
	fn from(callback: Callback) -> Self {
		ClientIoMessage::Execute(callback)
	}
}

impl fmt::Debug for Callback {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Callback").finish()
	}
}

