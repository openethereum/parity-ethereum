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

//! A provider for the LES protocol. This is typically a full node, who can
//! give as much data as necessary to its peers.

use std::sync::Arc;

use ethcore::blockchain_info::BlockChainInfo;
use ethcore::client::{BlockChainClient, ProvingBlockChainClient};
use ethcore::transaction::PendingTransaction;
use ethcore::ids::BlockId;
use ethcore::encoded;
use util::{Bytes, RwLock, H256};

use cht::{self, BlockInfo};
use client::{LightChainClient, AsLightClient};
use transaction_queue::TransactionQueue;


use request;

/// Defines the operations that a provider for `LES` must fulfill.
///
/// These are defined at [1], but may be subject to change.
/// Requests which can't be fulfilled should return either an empty RLP list
/// or empty vector where appropriate.
///
/// [1]: https://github.com/ethcore/parity/wiki/Light-Ethereum-Subprotocol-(LES)
#[cfg_attr(feature = "ipc", ipc(client_ident="LightProviderClient"))]
pub trait Provider: Send + Sync {
	/// Provide current blockchain info.
	fn chain_info(&self) -> BlockChainInfo;

	/// Find the depth of a common ancestor between two blocks.
	/// If either block is unknown or an ancestor can't be found
	/// then return `None`.
	fn reorg_depth(&self, a: &H256, b: &H256) -> Option<u64>;

	/// Earliest block where state queries are available.
	/// If `None`, no state queries are servable.
	fn earliest_state(&self) -> Option<u64>;

	/// Provide a list of headers starting at the requested block,
	/// possibly in reverse and skipping `skip` at a time.
	///
	/// The returned vector may have any length in the range [0, `max`], but the
	/// results within must adhere to the `skip` and `reverse` parameters.
	fn block_headers(&self, req: request::Headers) -> Vec<encoded::Header> {
		use request::HashOrNumber;

		if req.max == 0 { return Vec::new() }

		let best_num = self.chain_info().best_block_number;
		let start_num = match req.start {
			HashOrNumber::Number(start_num) => start_num,
			HashOrNumber::Hash(hash) => match self.block_header(BlockId::Hash(hash)) {
				None => {
					trace!(target: "les_provider", "Unknown block hash {} requested", hash);
					return Vec::new();
				}
				Some(header) => {
					let num = header.number();
					let canon_hash = self.block_header(BlockId::Number(num))
						.map(|h| h.hash());

					if req.max == 1 || canon_hash != Some(hash) {
						// Non-canonical header or single header requested.
						return vec![header];
					}

					num
				}
			}
		};

		(0u64..req.max as u64)
			.map(|x: u64| x.saturating_mul(req.skip + 1))
			.take_while(|x| if req.reverse { x < &start_num } else { best_num.saturating_sub(start_num) >= *x })
			.map(|x| if req.reverse { start_num - x } else { start_num + x })
			.map(|x| self.block_header(BlockId::Number(x)))
			.take_while(|x| x.is_some())
			.flat_map(|x| x)
			.collect()
	}

	/// Get a block header by id.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

	/// Provide as many as possible of the requested blocks (minus the headers) encoded
	/// in RLP format.
	fn block_bodies(&self, req: request::Bodies) -> Vec<Option<encoded::Body>> {
		req.block_hashes.into_iter()
			.map(|hash| self.block_body(BlockId::Hash(hash)))
			.collect()
	}

	/// Get a block body by id.
	fn block_body(&self, id: BlockId) -> Option<encoded::Body>;

	/// Provide the receipts as many as possible of the requested blocks.
	/// Returns a vector of RLP-encoded lists of receipts.
	fn receipts(&self, req: request::Receipts) -> Vec<Bytes> {
		req.block_hashes.into_iter()
			.map(|hash| self.block_receipts(&hash))
			.map(|receipts| receipts.unwrap_or_else(|| ::rlp::EMPTY_LIST_RLP.to_vec()))
			.collect()
	}

	/// Get a block's receipts as an RLP-encoded list by block hash.
	fn block_receipts(&self, hash: &H256) -> Option<Bytes>;

	/// Provide a set of merkle proofs, as requested. Each request is a
	/// block hash and request parameters.
	///
	/// Returns a vector of RLP-encoded lists satisfying the requests.
	fn proofs(&self, req: request::StateProofs) -> Vec<Bytes> {
		use rlp::{RlpStream, Stream};

		let mut results = Vec::with_capacity(req.requests.len());

		for request in req.requests {
			let proof = self.state_proof(request);

			let mut stream = RlpStream::new_list(proof.len());
			for node in proof {
				stream.append_raw(&node, 1);
			}

			results.push(stream.out());
		}

		results
	}

	/// Get a state proof from a request. Each proof should be a vector
	/// of rlp-encoded trie nodes, in ascending order by distance from the root.
	fn state_proof(&self, req: request::StateProof) -> Vec<Bytes>;

	/// Provide contract code for the specified (block_hash, account_hash) pairs.
	/// Each item in the resulting vector is either the raw bytecode or empty.
	fn contract_codes(&self, req: request::ContractCodes) -> Vec<Bytes> {
		req.code_requests.into_iter()
			.map(|req| self.contract_code(req))
			.collect()
	}

	/// Get contract code by request. Either the raw bytecode or empty.
	fn contract_code(&self, req: request::ContractCode) -> Bytes;

	/// Provide header proofs from the Canonical Hash Tries as well as the headers
	/// they correspond to -- each element in the returned vector is a 2-tuple.
	/// The first element is a block header and the second a merkle proof of
	/// the header in a requested CHT.
	fn header_proofs(&self, req: request::HeaderProofs) -> Vec<Bytes> {
		use rlp::{self, RlpStream, Stream};

		req.requests.into_iter()
			.map(|req| self.header_proof(req))
			.map(|maybe_proof| match maybe_proof {
				None => rlp::EMPTY_LIST_RLP.to_vec(),
				Some((header, proof)) => {
					let mut stream = RlpStream::new_list(2);
					stream.append_raw(&header.into_inner(), 1).begin_list(proof.len());

					for node in proof {
						stream.append_raw(&node, 1);
					}

					stream.out()
				}
			})
			.collect()
	}

	/// Provide a header proof from a given Canonical Hash Trie as well as the
	/// corresponding header. The first element is the block header and the
	/// second is a merkle proof of the CHT.
	fn header_proof(&self, req: request::HeaderProof) -> Option<(encoded::Header, Vec<Bytes>)>;

	/// Provide pending transactions.
	fn ready_transactions(&self) -> Vec<PendingTransaction>;
}

// Implementation of a light client data provider for a client.
impl<T: ProvingBlockChainClient + ?Sized> Provider for T {
	fn chain_info(&self) -> BlockChainInfo {
		BlockChainClient::chain_info(self)
	}

	fn reorg_depth(&self, a: &H256, b: &H256) -> Option<u64> {
		self.tree_route(a, b).map(|route| route.index as u64)
	}

	fn earliest_state(&self) -> Option<u64> {
		Some(self.pruning_info().earliest_state)
	}

	fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		BlockChainClient::block_header(self, id)
	}

	fn block_body(&self, id: BlockId) -> Option<encoded::Body> {
		BlockChainClient::block_body(self, id)
	}

	fn block_receipts(&self, hash: &H256) -> Option<Bytes> {
		BlockChainClient::block_receipts(self, hash)
	}

	fn state_proof(&self, req: request::StateProof) -> Vec<Bytes> {
		match req.key2 {
			Some(key2) => self.prove_storage(req.key1, key2, req.from_level, BlockId::Hash(req.block)),
			None => self.prove_account(req.key1, req.from_level, BlockId::Hash(req.block)),
		}
	}

	fn contract_code(&self, req: request::ContractCode) -> Bytes {
		self.code_by_hash(req.account_key, BlockId::Hash(req.block_hash))
	}

	fn header_proof(&self, req: request::HeaderProof) -> Option<(encoded::Header, Vec<Bytes>)> {
		if Some(req.cht_number) != cht::block_to_cht_number(req.block_number) {
			debug!(target: "les_provider", "Requested CHT number mismatch with block number.");
			return None;
		}

		let mut needed_hdr = None;

		// build the CHT, caching the requested header as we pass through it.
		let cht = {
			let block_info = |id| {
				let hdr = self.block_header(id);
				let td = self.block_total_difficulty(id);

				match (hdr, td) {
					(Some(hdr), Some(td)) => {
						let info = BlockInfo {
							hash: hdr.hash(),
							parent_hash: hdr.parent_hash(),
							total_difficulty: td,
						};

						if hdr.number() == req.block_number {
							needed_hdr = Some(hdr);
						}

						Some(info)
					}
					_ => None,
				}
			};

			match cht::build(req.cht_number, block_info) {
				Some(cht) => cht,
				None => return None, // incomplete CHT.
			}
		};

		let needed_hdr = needed_hdr.expect("`needed_hdr` always set in loop, number checked before; qed");

		// prove our result.
		match cht.prove(req.block_number, req.from_level) {
			Ok(Some(proof)) => Some((needed_hdr, proof)),
			Ok(None) => None,
			Err(e) => {
				debug!(target: "les_provider", "Error looking up number in freshly-created CHT: {}", e);
				None
			}
		}
	}

	fn ready_transactions(&self) -> Vec<PendingTransaction> {
		BlockChainClient::ready_transactions(self)
	}
}

/// The light client "provider" implementation. This wraps a `LightClient` and
/// a light transaction queue.
pub struct LightProvider<L> {
	client: Arc<L>,
	txqueue: Arc<RwLock<TransactionQueue>>,
}

impl<L> LightProvider<L> {
	/// Create a new `LightProvider` from the given client and transaction queue.
	pub fn new(client: Arc<L>, txqueue: Arc<RwLock<TransactionQueue>>) -> Self {
		LightProvider {
			client: client,
			txqueue: txqueue,
		}
	}
}

// TODO: draw from cache (shared between this and the RPC layer)
impl<L: AsLightClient + Send + Sync> Provider for LightProvider<L> {
	fn chain_info(&self) -> BlockChainInfo {
		self.client.as_light_client().chain_info()
	}

	fn reorg_depth(&self, _a: &H256, _b: &H256) -> Option<u64> {
		None
	}

	fn earliest_state(&self) -> Option<u64> {
		None
	}

	fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		self.client.as_light_client().block_header(id)
	}

	fn block_body(&self, _id: BlockId) -> Option<encoded::Body> {
		None
	}

	fn block_receipts(&self, _hash: &H256) -> Option<Bytes> {
		None
	}

	fn state_proof(&self, _req: request::StateProof) -> Vec<Bytes> {
		Vec::new()
	}

	fn contract_code(&self, _req: request::ContractCode) -> Bytes {
		Vec::new()
	}

	fn header_proof(&self, _req: request::HeaderProof) -> Option<(encoded::Header, Vec<Bytes>)> {
		None
	}

	fn ready_transactions(&self) -> Vec<PendingTransaction> {
		let chain_info = self.chain_info();
		self.txqueue.read().ready_transactions(chain_info.best_block_number, chain_info.best_block_timestamp)
	}
}

impl<L: AsLightClient> AsLightClient for LightProvider<L> {
	type Client = L::Client;

	fn as_light_client(&self) -> &L::Client {
		self.client.as_light_client()
	}
}

#[cfg(test)]
mod tests {
	use ethcore::client::{EachBlockWith, TestBlockChainClient};
	use super::Provider;

	#[test]
	fn cht_proof() {
		let client = TestBlockChainClient::new();
		client.add_blocks(2000, EachBlockWith::Nothing);

		let req = ::request::HeaderProof {
			cht_number: 0,
			block_number: 1500,
			from_level: 0,
		};

		assert!(client.header_proof(req.clone()).is_none());

		client.add_blocks(48, EachBlockWith::Nothing);

		assert!(client.header_proof(req.clone()).is_some());
	}
}
