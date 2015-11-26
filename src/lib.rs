#[macro_use] extern crate log;
extern crate ethcore_util;

use ethcore_util::hash::*;

pub type Bytes = Vec<u8>;
pub type Uint256 = Hash256;
pub type LogBloom = Hash4096;

pub const ZERO_UINT256: Uint256 = Hash256([0x00; 32]);
pub const ZERO_ADDRESS: Address = Address([0x00; 20]);
pub const BAD_UINT256: Uint256 = Hash256([0xff; 32]);
pub const ZERO_HASH256: Hash256 = Hash256([0x00; 32]);
pub const ZERO_LOGBLOOM: LogBloom = Hash4096([0x00; 512]);

#[derive(Debug)]
pub struct Header {
	parent_hash: Hash256,
	timestamp: Uint256,
	number: Uint256,
	author: Address,

	transactions_root: Hash256,
	uncles_hash: Hash256,
	extra_data_hash: Hash256,

	state_root: Hash256,
	receipts_root: Hash256,
	log_bloom: LogBloom,
	gas_used: Uint256,
	gas_limit: Uint256,

	difficulty: Uint256,
	seal: Vec<Bytes>,
}

impl Header {
	pub fn new() -> Header {
		Header {
			parent_hash: ZERO_UINT256,
			timestamp: BAD_UINT256,
			number: ZERO_UINT256,
			author: ZERO_ADDRESS,

			transactions_root: ZERO_HASH256,
			uncles_hash: ZERO_HASH256,
			extra_data_hash: ZERO_HASH256,

			state_root: ZERO_HASH256,
			receipts_root: ZERO_HASH256,
			log_bloom: ZERO_LOGBLOOM,
			gas_used: ZERO_UINT256,
			gas_limit: ZERO_UINT256,

			difficulty: ZERO_UINT256,
			seal: vec![],
		}
	}
}

pub struct Transaction {
	pub to: Address,
	pub gas: Uint256,
	pub data: Bytes,
	pub code: Bytes,
}

#[test]
fn memorydb() {

}


/// Silly function to return 69.
///
/// # Example
///
/// ```
/// assert_eq!(ethcore::sixtynine(), 69);
/// ```
pub fn sixtynine() -> i32 {
	debug!("Hello world!");
	69
}
