use util::hash::*;
use transaction::*;

pub struct ImportRoute {
	dead_blocks: Vec<H256>,
	live_blocks: Vec<H256>,
	transactions: Vec<Transaction>
}
