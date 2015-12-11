use blockheader::*;
use transaction::*;

pub struct VerifiedBlock<'a> {
	blockview: BlockView<'a>,
	transactions: Vec<Transaction>
}

impl<'a> VerifiedBlock<'a> {
	// todo, new should also take transactions
	pub fn new(bytes: &'a [u8]) -> VerifiedBlock<'a> {
		VerifiedBlock {
			blockview: BlockView::new(bytes),
			transactions: vec![]
		}
	}
}
