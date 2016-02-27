use util::hash::H256;
use util::uint::U256;
use header::BlockNumber;

/// Information about best block gathered together
#[derive(Default)]
pub struct BestBlock {
	pub hash: H256,
	pub number: BlockNumber,
	pub total_difficulty: U256
}

impl BestBlock {
	pub fn new() -> BestBlock { Default::default() }
}
		
		//BestBlock {
			//hash: H256::new(),
			//number: 0,
			//total_difficulty: U256::from(0)
		//}
	//}
//}
