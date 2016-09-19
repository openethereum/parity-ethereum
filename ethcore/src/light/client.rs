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

//! Light client implementation. Used for raw data queries as well as the header
//! sync.

use std::sync::Arc;

use engines::Engine;
use ids::BlockID;
use miner::TransactionQueue;
use service::ClientIoMessage;
use block_import_error::BlockImportError;
use block_status::BlockStatus;
use verification::queue::{Config as QueueConfig, HeaderQueue, QueueInfo, Status};
use transaction::SignedTransaction;

use super::provider::{CHTProofRequest, Provider, ProofRequest};

use io::IoChannel;
use util::hash::H256;
use util::Bytes;

/// A light client.
pub struct Client {
	engine: Arc<Engine>,
	header_queue: HeaderQueue,
	message_channel: IoChannel<ClientIoMessage>,
	transaction_queue: TransactionQueue,
}

impl Client {
	/// Import a header as rlp-encoded bytes.
	fn import_header(&self, bytes: Bytes) -> Result<H256, BlockImportError> {
		let header = ::rlp::decode(&bytes);

		self.header_queue.import(header).map_err(Into::into)
	}

	/// Whether the block is already known (but not necessarily part of the canonical chain)
	fn is_known(&self, id: BlockID) -> bool {
		false
	}

	/// Fetch a vector of all pending transactions.
	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		self.transaction_queue.top_transactions()
	}

	/// Inquire about the status of a given block.
	fn status(&self, id: BlockID) -> BlockStatus {
		BlockStatus::Unknown
	}
}

// stub implementation, can be partially implemented using LRU-caches
// but will almost definitely never be able to provide complete responses.
impl Provider for Client {
	fn block_headers(&self, _block: H256, _skip: usize, _max: usize, _reverse: bool) -> Vec<Bytes> {
		vec![]
	}

	fn block_bodies(&self, _blocks: Vec<H256>) -> Vec<Bytes> {
		vec![]
	}

	fn receipts(&self, _blocks: Vec<H256>) -> Vec<Bytes> {
		vec![]
	}

	fn proofs(&self, _requests: Vec<(H256, ProofRequest)>) -> Vec<Bytes> {
		vec![]
	}

	fn code(&self, _accounts: Vec<(H256, H256)>) -> Vec<Bytes> {
		vec![]
	}

	fn header_proofs(&self, _requests: Vec<CHTProofRequest>) -> Vec<Bytes> {
		vec![]
	}
}