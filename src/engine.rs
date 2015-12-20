use util::uint::*;
use util::hash::*;
use util::bytes::*;
use util::semantic_version::*;
use std::collections::hash_map::*;
use util::error::*;
use header::Header;
use account::Account;
use transaction::Transaction;

/// Definition of the cost schedule and other parameterisations for the EVM.
pub struct EvmSchedule {
	pub exceptional_failed_code_deposit: bool,
	pub have_delegate_call: bool,
	pub stack_limit: U256,
	pub tier_step_gas: [U256; 8],
	pub exp_gas: U256,
	pub exp_byte_gas: U256,
	pub sha3_gas: U256,
	pub sha3_word_gas: U256,
	pub sload_gas: U256,
	pub sstore_set_gas: U256,
	pub sstore_reset_gas: U256,
	pub sstore_refund_gas: U256,
	pub jumpdest_gas: U256,
	pub log_gas: U256,
	pub log_data_gas: U256,
	pub log_topic_gas: U256,
	pub create_gas: U256,
	pub call_gas: U256,
	pub call_stipend: U256,
	pub call_value_transfer_gas: U256,
	pub call_new_account_gas: U256,
	pub suicide_refund_gas: U256,
	pub memory_gas: U256,
	pub quad_coeff_div: U256,
	pub create_data_gas: U256,
	pub tx_gas: U256,
	pub tx_create_gas: U256,
	pub tx_data_zero_gas: U256,
	pub tx_data_non_zero_gas: U256,
	pub copy_gas: U256,
}

/// Definition of a contract whose implementation is built-in. 
pub struct Builtin {
	/// The gas cost of running this built-in for the given size of input data.
	pub cost: Box<Fn(usize) -> U256>,	// TODO: U256 should be bignum.
	/// Run this built-in function with the input being the first argument and the output
	/// being placed into the second.
	pub execute: Box<Fn(&[u8], &mut [u8])>,
}

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Params {
	pub engine_name: String,

	pub block_reward: U256,
	pub maximum_extra_data_size: U256,
	pub account_start_nonce: U256,
	pub evm_schedule: EvmSchedule,
	pub builtins: HashMap<Address, Builtin>,
	pub misc: HashMap<String, String>,

	// Genesis params.
	pub parent_hash: H256,
	pub author: Address,
	pub difficulty: U256,
	pub gas_limit: U256,
	pub gas_used: U256,
	pub timestamp: U256,
	pub extra_data: Bytes,
	pub genesis_state: HashMap<Address, Account>,
	// Only pre-populate if known equivalent to genesis_state's root. If they're different Bad Things Will Happen,
	pub state_root: Option<H256>,
	pub seal_fields: usize,
	pub seal_rlp: Bytes,
}

/// A consensus mechanism for the chain. Generally either proof-of-work or proof-of-stake-based.
/// Provides hooks into each of the major parts of block import.
pub trait Engine {
	/// The name of this engine.
	fn name(&self) -> &str;
	/// The version of this engine. Should be of the form 
	fn version(&self) -> SemanticVersion { SemanticVersion::new(0, 0, 0) }

	/// The number of additional header fields required for this engine.
	fn seal_fields(&self) -> u32 { 0 }
	/// Default values of the additional fields RLP-encoded in a raw (non-list) harness.
	fn seal_rlp(&self) -> Bytes { vec![] }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> HashMap<String, String> { HashMap::new() }

	/// Get the general parameters of the chain.
	fn params(&self) -> &Params;
	/// Set the general parameters of the chain.
	fn set_params(&mut self, p: Params);

	/// Get the EVM schedule for 
	fn evm_schedule(&self) -> &EvmSchedule { &self.params().evm_schedule }

	/// Verify that `header` is valid.
	/// `parent` (the parent header) and `block` (the header's full block) may be provided for additional
	/// checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify(&self, _header: &Header, _parent: Option<&Header>, _block: Option<&[u8]>) -> Result<(), EthcoreError> { Ok(()) }

	/// Additional verification for transactions in blocks.
	// TODO: Add flags for which bits of the transaction to check.
	fn verify_transaction(&self, _t: &Transaction, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Don't forget to call Super::populateFromParent when subclassing & overriding.
	fn populate_from_parent(&self, _header: &mut Header, _parent: &Header) -> Result<(), EthcoreError> { Ok(()) }
}

/// An engine which does not provide any consensus mechanism.
pub struct NullEngine {
	params: Params,
}

impl Engine for NullEngine {
	fn name(&self) -> &str { "NullEngine" }
	fn params(&self) -> &Params { &self.params }
	fn set_params(&mut self, params: Params) { self.params = params; }
}

impl Params {
	/// Convert this object into a boxed Engine of the right underlying type.
	pub fn to_engine(self) -> Box<Engine> { Box::new(NullEngine{params: self}) }

	/// Determine the state root for the 
	pub fn calculate_state_root(&self) -> H256 {
		// TODO: use the trie_root to calculate.
		unimplemented!();
	}

	pub fn genesis_block(&self) -> Bytes {
		// TODO
		unimplemented!();
	}
}

