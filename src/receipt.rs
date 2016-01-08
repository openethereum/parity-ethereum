use util::hash::*;
use util::uint::*;

/// Information describing execution of a transaction.
pub struct Receipt {
	// TODO
	pub state_root: H256,
	pub gas_used: U256,
}
