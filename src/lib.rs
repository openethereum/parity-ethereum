#[macro_use] extern crate log;
extern crate ethcore_util;

use ethcore_util::hash::*;
use ethcore_util::uint::*;

pub type LogBloom = Hash4096;

pub static ZERO_ADDRESS: Address = Address([0x00; 20]);
pub static ZERO_HASH256: Hash256 = Hash256([0x00; 32]);
pub static ZERO_LOGBLOOM: LogBloom = Hash4096([0x00; 512]);

#[derive(Debug)]
pub struct Header {
	parent_hash: Hash256,
	timestamp: U256,
	number: U256,
	author: Address,

	transactions_root: Hash256,
	uncles_hash: Hash256,
	extra_data_hash: Hash256,

	state_root: Hash256,
	receipts_root: Hash256,
	log_bloom: LogBloom,
	gas_used: U256,
	gas_limit: U256,

	difficulty: U256,
	seal: Vec<Bytes>,
}

impl Header {
	pub fn new() -> Header {
		Header {
			parent_hash: ZERO_HASH256,
			timestamp: BAD_U256,
			number: ZERO_U256,
			author: ZERO_ADDRESS,

			transactions_root: ZERO_HASH256,
			uncles_hash: ZERO_HASH256,
			extra_data_hash: ZERO_HASH256,

			state_root: ZERO_HASH256,
			receipts_root: ZERO_HASH256,
			log_bloom: ZERO_LOGBLOOM,
			gas_used: ZERO_U256,
			gas_limit: ZERO_U256,

			difficulty: ZERO_U256,
			seal: vec![],
		}
	}
}

pub struct Transaction {
	pub to: Address,
	pub gas: U256,
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
