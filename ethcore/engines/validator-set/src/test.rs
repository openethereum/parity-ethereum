// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

/// Used for Engine testing.

use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use log::trace;
use parity_util_mem::MallocSizeOf;
use common_types::{
	BlockNumber,
	ids::BlockId,
	header::Header,
	errors::EthcoreError,
	engines::machine::{Call, AuxiliaryData},
};
use ethereum_types::{H256, Address};
use machine::Machine;
use parity_bytes::Bytes;

use super::{ValidatorSet, SimpleList};

/// Set used for testing with a single validator.
#[derive(Clone, MallocSizeOf, Debug)]
pub struct TestSet {
	validator: SimpleList,
	#[ignore_malloc_size_of = "zero sized"]
	last_malicious: Arc<AtomicUsize>,
	#[ignore_malloc_size_of = "zero sized"]
	last_benign: Arc<AtomicUsize>,
}

impl Default for TestSet {
	fn default() -> Self {
		TestSet::new(
			Default::default(),
			Default::default(),
			vec![Address::from_str("7d577a597b2742b498cb5cf0c26cdcd726d39e6e").unwrap()]
		)
	}
}

impl TestSet {
	pub fn new(last_malicious: Arc<AtomicUsize>, last_benign: Arc<AtomicUsize>, validators: Vec<Address>) -> Self {
		TestSet {
			validator: SimpleList::new(validators),
			last_malicious,
			last_benign,
		}
	}

	pub fn from_validators(validators: Vec<Address>) -> Self {
		TestSet::new(Default::default(), Default::default(), validators)
	}

	pub fn last_malicious(&self) -> usize {
		self.last_malicious.load(AtomicOrdering::SeqCst)
	}

	pub fn last_benign(&self) -> usize {
		self.last_benign.load(AtomicOrdering::SeqCst)
	}
}

impl ValidatorSet for TestSet {
	fn default_caller(&self, _block_id: BlockId) -> Box<Call> {
		Box::new(|_, _| Err("Test set doesn't require calls.".into()))
	}

	fn is_epoch_end(&self, _first: bool, _chain_head: &Header) -> Option<Vec<u8>> { None }

	fn signals_epoch_end(&self, _: bool, _: &Header, _: AuxiliaryData)
		-> engine::EpochChange
	{
		engine::EpochChange::No
	}

	fn epoch_set(&self, _: bool, _: &Machine, _: BlockNumber, _: &[u8]) -> Result<(SimpleList, Option<H256>), EthcoreError> {
		Ok((self.validator.clone(), None))
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

	fn report_malicious(&self, _validator: &Address, _set_block: BlockNumber, block: BlockNumber, _proof: Bytes) {
		self.last_malicious.store(block as usize, AtomicOrdering::SeqCst)
	}

	fn report_benign(&self, _validator: &Address, _set_block: BlockNumber, block: BlockNumber) {
		trace!(target: "engine", "test validator set recording benign misbehaviour");
		self.last_benign.store(block as usize, AtomicOrdering::SeqCst)
	}
}
