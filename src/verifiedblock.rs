use blockheader::*;
use transaction::*;

pub struct VerifiedBlock<'a> {
	blockview: HeaderView<'a>,
	transactions: Vec<Transaction>
}

impl<'a> VerifiedBlock<'a> {
	// todo, new should also take transactions
	pub fn new(bytes: &'a [u8]) -> VerifiedBlock<'a> {
		VerifiedBlock {
			blockview: HeaderView::new(bytes),
			transactions: vec![]
		}
	}
}
