use std::collections::hash_map::*;
use std::cell::*;
use util::uint::*;
use util::hash::*;
use util::bytes::*;
use util::triehash::*;
use util::error::*;
use util::rlp::*;
use account::Account;
use engine::Engine;
use builtin::Builtin;
use null_engine::NullEngine;
use denominations::*;

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Spec {
	// What engine are we using for this?
	pub engine_name: String,

	// Various parameters for the chain operation.
	pub block_reward: U256,
	pub maximum_extra_data_size: U256,
	pub account_start_nonce: U256,
	pub misc: HashMap<String, Bytes>,

	// Builtin-contracts are here for now but would like to abstract into Engine API eventually.
	pub builtins: HashMap<Address, Builtin>,

	// Genesis params.
	pub parent_hash: H256,
	pub author: Address,
	pub difficulty: U256,
	pub gas_limit: U256,
	pub gas_used: U256,
	pub timestamp: U256,
	pub extra_data: Bytes,
	pub genesis_state: HashMap<Address, Account>,
	pub seal_fields: usize,
	pub seal_rlp: Bytes,

	// May be prepopulated if we know this in advance.
	state_root_memo: RefCell<Option<H256>>,
}

impl Spec {
	/// Convert this object into a boxed Engine of the right underlying type.
	// TODO avoid this hard-coded nastiness - use dynamic-linked plugin framework instead.
	pub fn to_engine(self) -> Result<Box<Engine>, EthcoreError> {
		match self.engine_name.as_ref() {
			"NullEngine" => Ok(NullEngine::new_boxed(self)),
			_ => Err(EthcoreError::UnknownName)
		}
	}

	/// Return the state root for the genesis state, memoising accordingly. 
	pub fn state_root(&self) -> Ref<H256> {
		if self.state_root_memo.borrow().is_none() {
			*self.state_root_memo.borrow_mut() = Some(trie_root(self.genesis_state.iter().map(|(k, v)| (k.to_vec(), v.rlp())).collect()));
		}
		Ref::map(self.state_root_memo.borrow(), |x|x.as_ref().unwrap())
	}

	pub fn block_reward(&self) -> U256 { self.block_reward }
	pub fn maximum_extra_data_size(&self) -> U256 { self.maximum_extra_data_size }
	pub fn account_start_nonce(&self) -> U256 { self.account_start_nonce }

	/// Compose the genesis block for this chain.
	pub fn genesis_block(&self) -> Bytes {
		// TODO
		unimplemented!();
	}
}


impl Spec {
	pub fn olympic() -> Spec {
		Spec {
			engine_name: "Ethash".to_string(),
			block_reward: finney() * U256::from(1500u64),
			maximum_extra_data_size: U256::from(1024u64),
			account_start_nonce: U256::from(0u64),
			misc: vec![
				("gas_limit_bounds_divisor", encode(&1024u64)), 
				("minimum_difficulty", encode(&131_072u64)), 
				("difficulty_bound_divisor", encode(&2048u64)), 
				("duration_limit", encode(&8u64)), 
				("min_gas_limit", encode(&125_000u64)), 
				("gas_floor_target", encode(&3_141_592u64)), 
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0.to_string(), vec.1);
				acc
			}),
			builtins: HashMap::new(),			// TODO: make correct
			parent_hash: H256::new(),
			author: Address::new(),
			difficulty: U256::from(131_072u64),
			gas_limit: U256::from(0u64),
			gas_used: U256::from(0u64),
			timestamp: U256::from(0u64),
			extra_data: vec![],
			genesis_state: vec![				// TODO: make correct
				(Address::new(), Account::new_basic(U256::from(1) << 200, U256::from(0)))
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0, vec.1);
				acc
			}),
			seal_fields: 2,
			seal_rlp: encode(&vec![0u64; 2]),	// TODO: make correct
			state_root_memo: RefCell::new(None),
		}
	}

	pub fn frontier() -> Spec {
		Spec {
			engine_name: "Ethash".to_string(),
			block_reward: ether() * U256::from(5u64),
			maximum_extra_data_size: U256::from(32u64),
			account_start_nonce: U256::from(0u64),
			misc: vec![
				("gas_limit_bounds_divisor", encode(&1024u64)), 
				("minimum_difficulty", encode(&131_072u64)), 
				("difficulty_bound_divisor", encode(&2048u64)), 
				("duration_limit", encode(&13u64)), 
				("min_gas_limit", encode(&5000u64)), 
				("gas_floor_target", encode(&3_141_592u64)), 
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0.to_string(), vec.1);
				acc
			}),
			builtins: HashMap::new(),			// TODO: make correct
			parent_hash: H256::new(),
			author: Address::new(),
			difficulty: U256::from(131_072u64),
			gas_limit: U256::from(0u64),
			gas_used: U256::from(0u64),
			timestamp: U256::from(0u64),
			extra_data: vec![],
			genesis_state: vec![				// TODO: make correct
				(Address::new(), Account::new_basic(U256::from(1) << 200, U256::from(0)))
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0, vec.1);
				acc
			}),
			seal_fields: 2,
			seal_rlp: encode(&vec![0u64; 2]),	// TODO: make correct
			state_root_memo: RefCell::new(None),
		}
	}

	pub fn morden() -> Spec {
		Spec {
			engine_name: "Ethash".to_string(),
			block_reward: ether() * U256::from(5u64),
			maximum_extra_data_size: U256::from(32u64),
			account_start_nonce: U256::from(1u64) << 20,
			misc: vec![
				("gas_limit_bounds_divisor", encode(&1024u64)), 
				("minimum_difficulty", encode(&131_072u64)), 
				("difficulty_bound_divisor", encode(&2048u64)), 
				("duration_limit", encode(&13u64)), 
				("min_gas_limit", encode(&5000u64)), 
				("gas_floor_target", encode(&3_141_592u64)), 
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0.to_string(), vec.1);
				acc
			}),
			builtins: HashMap::new(),			// TODO: make correct
			parent_hash: H256::new(),
			author: Address::new(),
			difficulty: U256::from(131_072u64),
			gas_limit: U256::from(0u64),
			gas_used: U256::from(0u64),
			timestamp: U256::from(0u64),
			extra_data: vec![],
			genesis_state: vec![				// TODO: make correct
				(Address::new(), Account::new_basic(U256::from(1) << 200, U256::from(0)))
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0, vec.1);
				acc
			}),
			seal_fields: 2,
			seal_rlp: encode(&vec![0u64; 2]),	// TODO: make correct
			state_root_memo: RefCell::new(None),
		}
	}
}

