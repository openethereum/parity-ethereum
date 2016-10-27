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

/// Light client implementation.
pub struct Client {
	engine: Arc<Engine>,
	header_queue: HeaderQueue,
	message_channel: IoChannel<ClientIoMessage>,
}

impl Client {
	/// Import a header as rlp-encoded bytes.
	pub fn import_header(&self, bytes: Bytes) -> Result<H256, BlockImportError> {
		let header = ::rlp::decode(&bytes);

		self.header_queue.import(header).map_err(Into::into)
	}

	/// Whether the block is already known (but not necessarily part of the canonical chain)
	pub fn is_known(&self, id: BlockID) -> bool {
		false
	}

	/// Fetch a vector of all pending transactions.
	pub fn pending_transactions(&self) -> Vec<SignedTransaction> {
		vec![]
	}

	/// Inquire about the status of a given block.
	pub fn status(&self, id: BlockID) -> BlockStatus {
		BlockStatus::Unknown
	}

	/// Get the header queue info.
	pub fn queue_info(&self) -> QueueInfo {
		self.header_queue.queue_info()
	}
}

impl Provider for Client {
	fn block_headers(&self, block: H256, skip: usize, max: usize, reverse: bool) -> Vec<Bytes> {
		Vec::new()
	}

	fn block_bodies(&self, blocks: Vec<H256>) -> Vec<Bytes> {
		Vec::new()
	}

	fn receipts(&self, blocks: Vec<H256>) -> Vec<Bytes> {
		Vec::new()
	}

	fn proofs(&self, requests: Vec<(H256, ProofRequest)>) -> Vec<Bytes> {
		Vec::new()
	}

	fn code(&self, accounts: Vec<(H256, H256)>) -> Vec<Bytes> {
		Vec::new()
	}

	fn header_proofs(&self, requests: Vec<CHTProofRequest>) -> Vec<Bytes> {
		Vec::new()
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		Vec::new()
	}
}