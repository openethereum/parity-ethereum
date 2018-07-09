//! Helper type with all filter state data.

use std::{
	collections::{BTreeSet, HashSet},
	sync::Arc,
};
use ethereum_types::H256;
use parking_lot::Mutex;
use v1::types::{Filter, Log};

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
	Block(BlockNumber),
	/// Hashes of all pending transactions the client knows about.
	PendingTransaction(BTreeSet<H256>),
	/// Number of From block number, last seen block hash, pending logs and log filter itself.
	Logs(BlockNumber, Option<H256>, HashSet<Log>, Filter)
}

/// Returns only last `n` logs
pub fn limit_logs(mut logs: Vec<Log>, limit: Option<usize>) -> Vec<Log> {
	let len = logs.len();
	match limit {
		Some(limit) if len >= limit => logs.split_off(len - limit),
		_ => logs,
	}
}
