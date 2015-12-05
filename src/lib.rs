#[macro_use] 
extern crate log;
extern crate ethcore_util;

#[cfg(feature = "jit" )]
extern crate evmjit;

use ethcore_util::hash::*;
use ethcore_util::uint::*;

pub type Bytes = Vec<u8>;
pub type LogBloom = H2048;

pub static ZERO_ADDRESS: Address = Address([0x00; 20]);
pub static ZERO_H256: H256 = H256([0x00; 32]);
pub static ZERO_LOGBLOOM: LogBloom = H2048([0x00; 256]);

#[derive(Debug)]
pub struct Header {
	parent_hash: H256,
	timestamp: U256,
	number: U256,
	author: Address,

	transactions_root: H256,
	uncles_hash: H256,
	extra_data_hash: H256,

	state_root: H256,
	receipts_root: H256,
	log_bloom: LogBloom,
	gas_used: U256,
	gas_limit: U256,

	difficulty: U256,
	seal: Vec<Bytes>,
}

impl Header {
	pub fn new() -> Header {
		Header {
			parent_hash: ZERO_H256.clone(),
			timestamp: BAD_U256.clone(),
			number: ZERO_U256.clone(),
			author: ZERO_ADDRESS.clone(),

			transactions_root: ZERO_H256.clone(),
			uncles_hash: ZERO_H256.clone(),
			extra_data_hash: ZERO_H256.clone(),

			state_root: ZERO_H256.clone(),
			receipts_root: ZERO_H256.clone(),
			log_bloom: ZERO_LOGBLOOM.clone(),
			gas_used: ZERO_U256.clone(),
			gas_limit: ZERO_U256.clone(),

			difficulty: ZERO_U256.clone(),
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
