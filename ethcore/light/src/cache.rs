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

//! Cache for data fetched from the network.
//!
//! Stores ancient block headers, bodies, receipts, and total difficulties.
//! Furthermore, stores a "gas price corpus" of relative recency, which is a sorted
//! vector of all gas prices from a recent range of blocks.

use ethcore::encoded;
use ethcore::header::BlockNumber;
use ethcore::receipt::Receipt;

use stats::Corpus;
use time::{SteadyTime, Duration};
use heapsize::HeapSizeOf;
use bigint::prelude::U256;
use bigint::hash::H256;
use util::cache::MemoryLruCache;

/// Configuration for how much data to cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheSizes {
	/// Maximum size, in bytes, of cached headers.
	pub headers: usize,
	/// Maximum size, in bytes, of cached canonical hashes.
	pub canon_hashes: usize,
	/// Maximum size, in bytes, of cached block bodies.
	pub bodies: usize,
	/// Maximum size, in bytes, of cached block receipts.
	pub receipts: usize,
	/// Maximum size, in bytes, of cached chain score for the block.
	pub chain_score: usize,
}

impl Default for CacheSizes {
	fn default() -> Self {
		const MB: usize = 1024 * 1024;
		CacheSizes {
			headers: 10 * MB,
			canon_hashes: 3 * MB,
			bodies: 20 * MB,
			receipts: 10 * MB,
			chain_score: 7 * MB,
		}
	}
}

/// The light client data cache.
///
/// Note that almost all getter methods take `&mut self` due to the necessity to update
/// the underlying LRU-caches on read.
/// [LRU-cache](https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_Recently_Used_.28LRU.29)
pub struct Cache {
	headers: MemoryLruCache<H256, encoded::Header>,
	canon_hashes: MemoryLruCache<BlockNumber, H256>,
	bodies: MemoryLruCache<H256, encoded::Body>,
	receipts: MemoryLruCache<H256, Vec<Receipt>>,
	chain_score: MemoryLruCache<H256, U256>,
	corpus: Option<(Corpus<U256>, SteadyTime)>,
	corpus_expiration: Duration,
}

impl Cache {
	/// Create a new data cache with the given sizes and gas price corpus expiration time.
	pub fn new(sizes: CacheSizes, corpus_expiration: Duration) -> Self {
		Cache {
			headers: MemoryLruCache::new(sizes.headers),
			canon_hashes: MemoryLruCache::new(sizes.canon_hashes),
			bodies: MemoryLruCache::new(sizes.bodies),
			receipts: MemoryLruCache::new(sizes.receipts),
			chain_score: MemoryLruCache::new(sizes.chain_score),
			corpus: None,
			corpus_expiration: corpus_expiration,
		}
	}

	/// Query header by hash.
	pub fn block_header(&mut self, hash: &H256) -> Option<encoded::Header> {
		self.headers.get_mut(hash).map(|x| x.clone())
	}

	/// Query hash by number.
	pub fn block_hash(&mut self, num: &BlockNumber) -> Option<H256> {
		self.canon_hashes.get_mut(num).map(|x| x.clone())
	}

	/// Query block body by block hash.
	pub fn block_body(&mut self, hash: &H256) -> Option<encoded::Body> {
		self.bodies.get_mut(hash).map(|x| x.clone())
	}

	/// Query block receipts by block hash.
	pub fn block_receipts(&mut self, hash: &H256) -> Option<Vec<Receipt>> {
		self.receipts.get_mut(hash).map(|x| x.clone())
	}

	/// Query chain score by block hash.
	pub fn chain_score(&mut self, hash: &H256) -> Option<U256> {
		self.chain_score.get_mut(hash).map(|x| x.clone())
	}

	/// Cache the given header.
	pub fn insert_block_header(&mut self, hash: H256, hdr: encoded::Header) {
		self.headers.insert(hash, hdr);
	}

	/// Cache the given canonical block hash.
	pub fn insert_block_hash(&mut self, num: BlockNumber, hash: H256) {
		self.canon_hashes.insert(num, hash);
	}

	/// Cache the given block body.
	pub fn insert_block_body(&mut self, hash: H256, body: encoded::Body) {
		self.bodies.insert(hash, body);
	}

	/// Cache the given block receipts.
	pub fn insert_block_receipts(&mut self, hash: H256, receipts: Vec<Receipt>) {
		self.receipts.insert(hash, receipts);
	}

	/// Cache the given chain scoring.
	pub fn insert_chain_score(&mut self, hash: H256, score: U256) {
		self.chain_score.insert(hash, score);
	}

	/// Get gas price corpus, if recent enough.
	pub fn gas_price_corpus(&self) -> Option<Corpus<U256>> {
		let now = SteadyTime::now();

		self.corpus.as_ref().and_then(|&(ref corpus, ref tm)| {
			if *tm + self.corpus_expiration >= now {
				Some(corpus.clone())
			} else {
				None
			}
		})
	}

	/// Set the cached gas price corpus.
	pub fn set_gas_price_corpus(&mut self, corpus: Corpus<U256>) {
		self.corpus = Some((corpus, SteadyTime::now()))
	}

	/// Get the memory used.
	pub fn mem_used(&self) -> usize {
		self.heap_size_of_children()
	}
}

impl HeapSizeOf for Cache {
	fn heap_size_of_children(&self) -> usize {
		self.headers.current_size()
			+ self.canon_hashes.current_size()
			+ self.bodies.current_size()
			+ self.receipts.current_size()
			+ self.chain_score.current_size()
			// TODO: + corpus
	}
}

#[cfg(test)]
mod tests {
	use super::Cache;
	use time::Duration;

	#[test]
	fn corpus_inaccessible() {
		let mut cache = Cache::new(Default::default(), Duration::hours(5));

		cache.set_gas_price_corpus(vec![].into());
		assert_eq!(cache.gas_price_corpus(), Some(vec![].into()));

		{
			let corpus_time = &mut cache.corpus.as_mut().unwrap().1;
			*corpus_time = *corpus_time - Duration::hours(6);
		}
		assert!(cache.gas_price_corpus().is_none());
	}
}
