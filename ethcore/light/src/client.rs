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

use ethcore::engines::Engine;
use ethcore::ids::BlockID;
use ethcore::service::ClientIoMessage;
use ethcore::block_import_error::BlockImportError;
use ethcore::block_status::BlockStatus;
use ethcore::verification::queue::{HeaderQueue, QueueInfo};
use ethcore::transaction::SignedTransaction;
use ethcore::blockchain_info::BlockChainInfo;

use io::IoChannel;
use util::hash::H256;
use util::{Bytes, Mutex};

use provider::Provider;
use request;

/// Light client implementation.
pub struct Client {
	engine: Arc<Engine>,
	header_queue: HeaderQueue,
	message_channel: Mutex<IoChannel<ClientIoMessage>>,
}

impl Client {
	/// Import a header as rlp-encoded bytes.
	pub fn import_header(&self, bytes: Bytes) -> Result<H256, BlockImportError> {
		let header = ::rlp::decode(&bytes);

		self.header_queue.import(header).map_err(Into::into)
	}

	/// Whether the block is already known (but not necessarily part of the canonical chain)
	pub fn is_known(&self, _id: BlockID) -> bool {
		false
	}

	/// Fetch a vector of all pending transactions.
	pub fn pending_transactions(&self) -> Vec<SignedTransaction> {
		vec![]
	}

	/// Inquire about the status of a given block.
	pub fn status(&self, _id: BlockID) -> BlockStatus {
		BlockStatus::Unknown
	}

	/// Get the header queue info.
	pub fn queue_info(&self) -> QueueInfo {
		self.header_queue.queue_info()
	}
}

// dummy implementation -- may draw from canonical cache further on.
impl Provider for Client {
	fn chain_info(&self) -> BlockChainInfo {
		unimplemented!()
	}

	fn reorg_depth(&self, _a: &H256, _b: &H256) -> Option<u64> {
		None
	}

	fn earliest_state(&self) -> Option<u64> {
		None
	}

	fn block_headers(&self, _req: request::Headers) -> Vec<Bytes> {
		Vec::new()
	}

	fn block_bodies(&self, _req: request::Bodies) -> Vec<Bytes> {
		Vec::new()
	}

	fn receipts(&self, _req: request::Receipts) -> Vec<Bytes> {
		Vec::new()
	}

	fn proofs(&self, _req: request::StateProofs) -> Vec<Bytes> {
		Vec::new()
	}

	fn code(&self, _req: request::ContractCodes) -> Vec<Bytes> {
		Vec::new()
	}

	fn header_proofs(&self, _req: request::HeaderProofs) -> Vec<Bytes> {
		Vec::new()
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		Vec::new()
	}
}