// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! Light client implementation. Stores data from light sync

use std::sync::Arc;

use ethcore::engines::Engine;
use ethcore::ids::BlockId;
use ethcore::service::ClientIoMessage;
use ethcore::block_import_error::BlockImportError;
use ethcore::block_status::BlockStatus;
use ethcore::verification::queue::{HeaderQueue, QueueInfo};
use ethcore::transaction::SignedTransaction;
use ethcore::blockchain_info::BlockChainInfo;

use io::IoChannel;
use util::hash::{H256, H256FastMap};
use util::{Bytes, Mutex};

use provider::Provider;
use request;

/// Light client implementation.
pub struct Client {
	_engine: Arc<Engine>,
	header_queue: HeaderQueue,
	_message_channel: Mutex<IoChannel<ClientIoMessage>>,
	tx_pool: Mutex<H256FastMap<SignedTransaction>>,
}

impl Client {
	/// Import a header as rlp-encoded bytes.
	pub fn import_header(&self, bytes: Bytes) -> Result<H256, BlockImportError> {
		let header = ::rlp::decode(&bytes);

		self.header_queue.import(header).map_err(Into::into)
	}

	/// Whether the block is already known (but not necessarily part of the canonical chain)
	pub fn is_known(&self, _id: BlockId) -> bool {
		false
	}

	/// Import a local transaction.
	pub fn import_own_transaction(&self, tx: SignedTransaction) {
		self.tx_pool.lock().insert(tx.hash(), tx);
	} 

	/// Fetch a vector of all pending transactions.
	pub fn pending_transactions(&self) -> Vec<SignedTransaction> {
		self.tx_pool.lock().values().cloned().collect()
	}

	/// Inquire about the status of a given block (or header).
	pub fn status(&self, _id: BlockId) -> BlockStatus {
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

	fn contract_code(&self, _req: request::ContractCodes) -> Vec<Bytes> {
		Vec::new()
	}

	fn header_proofs(&self, _req: request::HeaderProofs) -> Vec<Bytes> {
		Vec::new()
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		Vec::new()
	}
}