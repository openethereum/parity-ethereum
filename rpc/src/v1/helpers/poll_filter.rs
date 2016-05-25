//! Helper type with all filter state data.

use std::collections::HashSet;
use util::hash::H256;
use v1::types::{Filter, Log};

pub type BlockNumber = u64;

/// Filter state.
#[derive(Clone)]
pub enum PollFilter {
	/// Number of last block which client was notified about.
	Block(BlockNumber),
	/// Hashes of all transactions which client was notified about.
	PendingTransaction(Vec<H256>),
	/// Number of From block number, pending logs and log filter iself.
	Logs(BlockNumber, HashSet<Log>, Filter)
}
