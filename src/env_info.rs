use util::*;
use header::BlockNumber;

/// Simple vector of hashes, should be at most 256 items large, can be smaller if being used
/// for a block whose number is less than 257.
pub type LastHashes = Vec<H256>;

/// Information concerning the execution environment for a message-call/contract-creation.
#[derive(Debug)]
pub struct EnvInfo {
	/// The block number.
	pub number: BlockNumber,
	/// The block author.
	pub author: Address,
	/// The block timestamp.
	pub timestamp: u64,
	/// The block difficulty.
	pub difficulty: U256,
	/// The block gas limit.
	pub gas_limit: U256,
	/// The last 256 block hashes.
	pub last_hashes: LastHashes,
	/// The gas used.
	pub gas_used: U256,
}

impl EnvInfo {
	pub fn new() -> EnvInfo {
		EnvInfo {
			number: 0,
			author: Address::new(),
			timestamp: 0,
			difficulty: U256::zero(),
			gas_limit: U256::zero(),
			last_hashes: vec![],
			gas_used: U256::zero()
		}
	}

	pub fn from_json(json: &Json) -> EnvInfo {
		EnvInfo {
			number: u64_from_json(&json["currentNumber"]),
			author: address_from_json(&json["currentCoinbase"]),
			difficulty: u256_from_json(&json["currentDifficulty"]),
			gas_limit: u256_from_json(&json["currentGasLimit"]),
			timestamp: u64_from_json(&json["currentTimestamp"]),
			last_hashes: vec![h256_from_json(&json["previousHash"])],
			gas_used: U256::zero(),
		}
	}
}

/// TODO: it should be the other way around.
/// `new` should call `default`.
impl Default for EnvInfo {
	fn default() -> Self {
		EnvInfo::new()
	}
}
