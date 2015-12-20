use util::uint::*;
use util::hash::*;
use util::bytes::*;
use header::Header;
use std::collections::hash_map::*;
use util::error::*;

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
	/*
	TODO: std::unordered_map<Address, PrecompiledContract> precompiled;
	*/
	pub block_reward: U256,
	pub maximum_extra_data_size: U256,
	pub account_start_nonce: U256,
	pub evm_schedule: EvmSchedule,
	pub builtins: HashMap<Address, Builtin>,
	pub misc: HashMap<String, String>,
}

// TODO: move to ethcore-util
/// A version value with strict meaning. Use `to_u32` to convert to a simple integer.
/// 
/// # Example
/// ```
/// extern crate ethcore;
/// use ethcore::engine::*;
/// fn main() {
///   assert_eq!(SemanticVersion::new(1, 2, 3).as_u32(), 0x010203);
/// }
/// ```
pub struct SemanticVersion {
	/// Major version - API/feature removals & breaking changes.
	pub major: u8,
	/// Minor version - API/feature additions.
	pub minor: u8,
	/// Tiny version - bug fixes.
	pub tiny: u8,
}

impl SemanticVersion {
	/// Create a new object.
	pub fn new(major: u8, minor: u8, tiny: u8) -> SemanticVersion { SemanticVersion{major: major, minor: minor, tiny: tiny} }

	/// Convert to a `u32` representation.
	pub fn as_u32(&self) -> u32 { ((self.major as u32) << 16) + ((self.minor as u32) << 8) + self.tiny as u32 }
}

// TODO: implement PartialOrdered for SemanticVersion.

/// A consensus mechanism for the chain. Generally either proof-of-work or proof-of-stake-based.
/// Provides hooks into each of the major parts of block import.
pub trait Engine {
	/// The name of this engine.
	fn name(&self) -> &str;
	/// The version of this engine. Should be of the form 
	fn version(&self) -> SemanticVersion { SemanticVersion::new(0, 0 ,0) }

	/// The number of additional header fields required for this engine.
	fn seal_fields(&self) -> u32 { 0 }
	/// Default values of the additional fields RLP-encoded in a raw (non-list) harness.
	fn seal_rlp(&self) -> Bytes { vec![] }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> HashMap<String, String> { HashMap::new() }

	/// Verify that `header` is valid.
	/// `parent` (the parent header) and `block` (the header's full block) may be provided for additional
	/// checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify(&self, _header: &Header, _parent: Option<&Header>, _block: Option<&[u8]>) -> Result<(), EthcoreError> { Ok(()) }
/*
	virtual void verify(Strictness _s, BlockHeader const& _bi, BlockHeader const& _parent = BlockHeader(), bytesConstRef _block = bytesConstRef()) const;
	/// Additional verification for transactions in blocks.
	virtual void verifyTransaction(ImportRequirements::value _ir, TransactionBase const& _t, BlockHeader const& _bi) const;
	/// Don't forget to call Super::populateFromParent when subclassing & overriding.
	virtual void populateFromParent(BlockHeader& _bi, BlockHeader const& _parent) const;
*/

	/// Get the general parameters of the chain.
	fn params(&self) -> &Params;
	/// Set the general parameters of the chain.
	fn set_params(&mut self, p: Params);
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