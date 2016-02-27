use util::hash::H256;

/// Represents a tree route between `from` block and `to` block:
#[derive(Debug)]
pub struct TreeRoute {
	/// A vector of hashes of all blocks, ordered from `from` to `to`.
	pub blocks: Vec<H256>,
	/// Best common ancestor of these blocks.
	pub ancestor: H256,
	/// An index where best common ancestor would be.
	pub index: usize,
}

