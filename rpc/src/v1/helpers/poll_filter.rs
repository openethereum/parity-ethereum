// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Helper type with all filter state data.

use std::{
	collections::{BTreeSet, HashSet, VecDeque},
	sync::Arc,
};
use ethereum_types::H256;
use parking_lot::Mutex;
use v1::types::Log;
use types::filter::Filter;

pub type BlockNumber = u64;

/// Thread-safe filter state.
#[derive(Clone)]
pub struct SyncPollFilter(Arc<Mutex<PollFilter>>);

impl SyncPollFilter {
	/// New `SyncPollFilter`
	pub fn new(f: PollFilter) -> Self {
		SyncPollFilter(Arc::new(Mutex::new(f)))
	}

	/// Modify underlying filter
	pub fn modify<F, R>(&self, f: F) -> R where
		F: FnOnce(&mut PollFilter) -> R,
	{
		f(&mut self.0.lock())
	}
}

/// Filter state.
#[derive(Clone)]
pub enum PollFilter {
	/// Number of last block which client was notified about.
	Block {
		last_block_number: BlockNumber,
		#[doc(hidden)]
		recent_reported_hashes: VecDeque<(BlockNumber, H256)>,
	},
	/// Hashes of all pending transactions the client knows about.
	PendingTransaction(BTreeSet<H256>),
	/// Number of From block number, last seen block hash, pending logs and log filter itself.
	Logs {
		block_number: BlockNumber,
		last_block_hash: Option<H256>,
		previous_logs: HashSet<Log>,
		filter: Filter,
		include_pending: bool,
	}
}

impl PollFilter {
	pub (in v1) const MAX_BLOCK_HISTORY_SIZE: usize = 32;
}

/// Returns only last `n` logs
pub fn limit_logs(mut logs: Vec<Log>, limit: Option<usize>) -> Vec<Log> {
	let len = logs.len();
	match limit {
		Some(limit) if len >= limit => logs.split_off(len - limit),
		_ => logs,
	}
}
