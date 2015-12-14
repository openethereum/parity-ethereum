use blockheader::*;
use transaction::*;

pub struct VerifiedBlock<'a> {
	_blockview: HeaderView<'a>,
	_transactions: Vec<Transaction>
}

impl<'a> VerifiedBlock<'a> {
	// todo, new should also take transactions
	pub fn new(bytes: &'a [u8]) -> VerifiedBlock<'a> {
		VerifiedBlock {
			_blockview: HeaderView::new(bytes),
			_transactions: vec![]
		}
	}
}
