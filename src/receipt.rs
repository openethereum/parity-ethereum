use util::*;
use basic_types::LogBloom;

/// Information describing execution of a transaction.
pub struct Receipt {
	// TODO
	pub state_root: H256,
	pub gas_used: U256,
	pub log_bloom: LogBloom,
}
