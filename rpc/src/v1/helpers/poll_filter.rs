//! Helper type with all filter state data.

use std::collections::HashSet;
use ethereum_types::H256;
use v1::types::{Filter, Log};

pub type BlockNumber = u64;

/// Filter state.
#[derive(Clone)]
pub enum PollFilter {
	/// Number of last block which client was notified about.
	Block(BlockNumber),
	/// Hashes of all transactions which client was notified about.
	PendingTransaction(Vec<H256>),
	/// Number of From block number, pending logs and log filter itself.
	Logs(BlockNumber, HashSet<Log>, Filter)
}

/// Returns only last `n` logs
pub fn limit_logs(mut logs: Vec<Log>, limit: Option<usize>) -> Vec<Log> {
	let len = logs.len();
	match limit {
		Some(limit) if len >= limit => logs.split_off(len - limit),
		_ => logs,
	}
}
