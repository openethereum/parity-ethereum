use util::hash::H256;
use util::uint::U256;
use header::BlockNumber;

pub struct BlockInfo {
	pub hash: H256,
	pub number: BlockNumber,
	pub total_difficulty: U256,
	pub location: BlockLocation
}

pub enum BlockLocation {
	CanonChain,
	Branch,
	BranchBecomingCanonChain {
		ancestor: H256,
		route: Vec<H256>
	}
}
