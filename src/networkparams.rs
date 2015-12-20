use util::uint::*;
use denominations::*;

// TODO: move to params.rs.

/// Network related const params
/// TODO: make it configurable from json file
pub struct NetworkParams {
	maximum_extra_data_size: U256,
	min_gas_limit: U256,
	gas_limit_bounds_divisor: U256,
	minimum_difficulty: U256,
	difficulty_bound_divisor: U256,
	duration_limit: U256,
	block_reward: U256,
	gas_floor_target: U256,
	account_start_nonce: U256
}

impl NetworkParams {
	pub fn olympic() -> NetworkParams {
		NetworkParams {
			maximum_extra_data_size: U256::from(1024u64),
			min_gas_limit: U256::from(125_000u64),
			gas_floor_target: U256::from(3_141_592u64),
			gas_limit_bounds_divisor: U256::from(1024u64),
			minimum_difficulty: U256::from(131_072u64),
			difficulty_bound_divisor: U256::from(2048u64),
			duration_limit: U256::from(8u64),
			block_reward: finney() * U256::from(1500u64),
			account_start_nonce: U256::from(0u64)
		}
	}

	pub fn frontier() -> NetworkParams {
		NetworkParams {
			maximum_extra_data_size: U256::from(32u64),
			min_gas_limit: U256::from(5000u64),
			gas_floor_target: U256::from(3_141_592u64),
			gas_limit_bounds_divisor: U256::from(1024u64),
			minimum_difficulty: U256::from(131_072u64),
			difficulty_bound_divisor: U256::from(2048u64),
			duration_limit: U256::from(13u64),
			block_reward: ether() * U256::from(5u64),
			account_start_nonce: U256::from(0u64)
		}
	}

	pub fn morden() -> NetworkParams {
		NetworkParams {
			maximum_extra_data_size: U256::from(32u64),
			min_gas_limit: U256::from(5000u64),
			gas_floor_target: U256::from(3_141_592u64),
			gas_limit_bounds_divisor: U256::from(1024u64),
			minimum_difficulty: U256::from(131_072u64),
			difficulty_bound_divisor: U256::from(2048u64),
			duration_limit: U256::from(13u64),
			block_reward: ether() * U256::from(5u64),
			account_start_nonce: U256::from(1u64) << 20
		}
	}

	pub fn maximum_extra_data_size(&self) -> U256 { self.maximum_extra_data_size }
	pub fn min_gas_limit(&self) -> U256 { self.min_gas_limit }
	pub fn gas_limit_bounds_divisor(&self) -> U256 { self.gas_limit_bounds_divisor }
	pub fn minimum_difficulty(&self) -> U256 { self.minimum_difficulty }
	pub fn difficulty_bound_divisor(&self) -> U256 { self.difficulty_bound_divisor }
	pub fn duration_limit(&self) -> U256 { self.duration_limit }
	pub fn block_reward(&self) -> U256 { self.block_reward }
	pub fn gas_floor_target(&self) -> U256 { self.gas_floor_target }
	pub fn account_start_nonce(&self) -> U256 { self.account_start_nonce }
}

