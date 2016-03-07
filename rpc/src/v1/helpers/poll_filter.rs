//! Helper type with all filter possibilities.

use ethcore::filter::Filter;

#[derive(Clone)]
pub enum PollFilter {
	Block,
	PendingTransaction,
	Logs(Filter)
}
