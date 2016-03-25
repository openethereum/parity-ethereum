//! Helper type with all filter possibilities.

use util::hash::H256;
use ethcore::filter::Filter;

pub type BlockNumber = u64;

#[derive(Clone)]
pub enum PollFilter {
	Block(BlockNumber),
	PendingTransaction(Vec<H256>),
	Logs(BlockNumber, Filter),
}
