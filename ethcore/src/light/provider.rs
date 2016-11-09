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

//! A provider for the LES protocol. This is typically a full node, who can
//! give as much data as necessary to its peers.

use ethcore::client::BlockChainClient;
use ethcore::transaction::SignedTransaction;
use ethcore::blockchain_info::BlockChainInfo;

use rlp::EMPTY_LIST_RLP;
use util::{Bytes, H256};

use request;

/// Defines the operations that a provider for `LES` must fulfill.
///
/// These are defined at [1], but may be subject to change.
/// Requests which can't be fulfilled should return an empty RLP list.
///
/// [1]: https://github.com/ethcore/parity/wiki/Light-Ethereum-Subprotocol-(LES)
pub trait Provider: Send + Sync {
	/// Provide current blockchain info.
	fn chain_info(&self) -> BlockChainInfo;

	/// Find the depth of a common ancestor between two blocks.
	fn reorg_depth(&self, a: &H256, b: &H256) -> Option<u64>;

	/// Earliest block where state queries are available.
	/// All states between this value and
	fn earliest_state(&self) -> u64;

	/// Provide a list of headers starting at the requested block,
	/// possibly in reverse and skipping `skip` at a time.
	///
	/// The returned vector may have any length in the range [0, `max`], but the
	/// results within must adhere to the `skip` and `reverse` parameters.
	fn block_headers(&self, req: request::Headers) -> Vec<Bytes>;

	/// Provide as many as possible of the requested blocks (minus the headers) encoded
	/// in RLP format.
	fn block_bodies(&self, req: request::Bodies) -> Vec<Bytes>;

	/// Provide the receipts as many as possible of the requested blocks.
	/// Returns a vector of RLP-encoded lists of receipts.
	fn receipts(&self, req: request::Receipts) -> Vec<Bytes>;

	/// Provide a set of merkle proofs, as requested. Each request is a
	/// block hash and request parameters.
	///
	/// Returns a vector to RLP-encoded lists satisfying the requests.
	fn proofs(&self, req: request::StateProofs) -> Vec<Bytes>;

	/// Provide contract code for the specified (block_hash, account_hash) pairs.
	fn code(&self, req: request::ContractCodes) -> Vec<Bytes>;

	/// Provide header proofs from the Canonical Hash Tries.
	fn header_proofs(&self, req: request::HeaderProofs) -> Vec<Bytes>;

	/// Provide pending transactions.
	fn pending_transactions(&self) -> Vec<SignedTransaction>;
}

// TODO [rob] move into trait definition file after ethcore crate
// is split up. ideally `ethcore-light` will be between `ethcore-blockchain`
// and `ethcore-client`
impl<T: BlockChainClient + ?Sized> Provider for T {
	fn chain_info(&self) -> BlockChainInfo {
		BlockChainClient::chain_info(self)
	}

	fn reorg_depth(&self, a: &H256, b: &H256) -> Option<u64> {
		self.tree_route.map(|route| route.index as u64)
	}

	fn earliest_state(&self) -> u64 {
		self.pruning_info().earliest_state
	}

	fn block_headers(&self, req: request::Headers) -> Vec<Bytes> {
		unimplemented!()
	}

	fn block_bodies(&self, req: request::Bodies) -> Vec<Bytes> {
		req.block_hashes.into_iter()
			.map(|hash| self.block_body(hash.into()))
			.map(|body| body.unwrap_or_else(|| EMPTY_LIST_RLP.into()))
			.collect()
	}

	fn receipts(&self, req: request::Receipts) -> Vec<Bytes> {
		req.block_hashes.into_iter()
			.map(|hash| self.block_receipts(&hash)
			.map(|receipts| receips.unwrap_or_else(|| EMPTY_LIST_RLP.into()))
			.collect()
	}

	fn proofs(&self, req: request::StateProofs) -> Vec<Bytes> {
		unimplemented!()
	}

	fn code(&self, req: request::ContractCodes) -> Vec<Bytes>;

	fn header_proofs(&self, req: request::HeaderProofs) -> Vec<Bytes> {
		// TODO: [rob] implement CHT stuff on `ethcore` side.
		req.requests.into_iter().map(|_| EMPTY_LIST_RLP.into()).collect()
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction>;
}