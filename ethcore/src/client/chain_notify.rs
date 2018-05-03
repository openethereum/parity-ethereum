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

use bytes::Bytes;
use ethereum_types::H256;
use transaction::UnverifiedTransaction;
use std::time::Duration;

/// Messages to broadcast via chain
pub enum ChainMessageType {
	/// Consensus message
	Consensus(Vec<u8>),
	/// Message with private transaction
	PrivateTransaction(Vec<u8>),
	/// Message with signed private transaction
	SignedPrivateTransaction(Vec<u8>),
}

/// Route type to indicate whether it is enacted or retracted.
pub enum ChainRouteType {
	/// Enacted block
	Enacted,
	/// Retracted block
	Retracted
}

/// Represents what has to be handled by actor listening to chain events
pub trait ChainNotify : Send + Sync {
	/// fires when chain has new blocks.
	fn new_blocks(
		&self,
		_imported: Vec<H256>,
		_invalid: Vec<H256>,
		_route: Vec<(H256, ChainRouteType)>,
		_sealed: Vec<H256>,
		// Block bytes.
		_proposed: Vec<Bytes>,
		_duration: Duration,
	) {
		// does nothing by default
	}

	/// fires when chain achieves active mode
	fn start(&self) {
		// does nothing by default
	}

	/// fires when chain achieves passive mode
	fn stop(&self) {
		// does nothing by default
	}

	/// fires when chain broadcasts a message
	fn broadcast(&self, _message_type: ChainMessageType) {}

	/// fires when new transactions are received from a peer
	fn transactions_received(&self,
		_txs: &[UnverifiedTransaction],
		_peer_id: usize,
	) {
		// does nothing by default
	}
}
