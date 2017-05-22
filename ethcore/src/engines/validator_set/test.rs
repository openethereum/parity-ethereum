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

/// Used for Engine testing.

use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use util::{Arc, Bytes, H256, Address, HeapSizeOf};

use engines::Call;
use header::{Header, BlockNumber};
use super::{ValidatorSet, SimpleList};

/// Set used for testing with a single validator.
pub struct TestSet {
	validator: SimpleList,
	last_malicious: Arc<AtomicUsize>,
	last_benign: Arc<AtomicUsize>,
}

impl TestSet {
	pub fn new(last_malicious: Arc<AtomicUsize>, last_benign: Arc<AtomicUsize>) -> Self {
		TestSet {
			validator: SimpleList::new(vec![Address::from_str("7d577a597b2742b498cb5cf0c26cdcd726d39e6e").unwrap()]),
			last_malicious: last_malicious,
			last_benign: last_benign,
		}
	}
}

impl HeapSizeOf for TestSet {
	fn heap_size_of_children(&self) -> usize {
		self.validator.heap_size_of_children()
	}
}

impl ValidatorSet for TestSet {
	fn default_caller(&self, _block_id: ::ids::BlockId) -> Box<Call> {
		Box::new(|_, _| Err("Test set doesn't require calls.".into()))
	}

	fn is_epoch_end(&self, _header: &Header, _block: Option<&[u8]>, _receipts: Option<&[::receipt::Receipt]>)
		-> ::engines::EpochChange
	{
		::engines::EpochChange::No
	}

	fn epoch_proof(&self, _header: &Header, _caller: &Call) -> Result<Vec<u8>, String> {
		Ok(Vec::new())
	}

	fn epoch_set(&self, _header: &Header, _: &[u8]) -> Result<(u64, SimpleList), ::error::Error> {
		Ok((0, self.validator.clone()))
	}

	fn contains_with_caller(&self, bh: &H256, address: &Address, _: &Call) -> bool {
		self.validator.contains(bh, address)
	}

	fn get_with_caller(&self, bh: &H256, nonce: usize, _: &Call) -> Address {
		self.validator.get(bh, nonce)
	}

	fn count_with_caller(&self, _bh: &H256, _: &Call) -> usize {
		1
	}

	fn report_malicious(&self, _validator: &Address, block: BlockNumber, _proof: Bytes) {
		self.last_malicious.store(block as usize, AtomicOrdering::SeqCst)
	}

	fn report_benign(&self, _validator: &Address, block: BlockNumber) {
		self.last_benign.store(block as usize, AtomicOrdering::SeqCst)
	}
}
