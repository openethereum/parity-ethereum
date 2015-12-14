use util::hash::*;
use transaction::*;

pub struct ImportRoute {
	_dead_blocks: Vec<H256>,
	_live_blocks: Vec<H256>,
	_transactions: Vec<Transaction>
}
