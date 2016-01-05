use std::collections::hash_map::*;
use util::bytes::*;
use util::uint::*;
use util::rlp::*;
use util::semantic_version::*;
use util::error::*;
use header::Header;
use transaction::Transaction;
use spec::Spec;
use evm_schedule::EvmSchedule;
use env_info::EnvInfo;

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
	fn spec(&self) -> &Spec;

	/// Get the EVM schedule for the given `env_info`.
	fn evm_schedule(&self, env_info: &EnvInfo) -> EvmSchedule;

	/// Some intrinsic operation parameters; by default they take their value from the `spec()`'s `engine_params`.
	fn maximum_extra_data_size(&self, _env_info: &EnvInfo) -> usize { decode(&self.spec().engine_params.get("maximum_extra_data_size").unwrap()) }
	fn account_start_nonce(&self, _env_info: &EnvInfo) -> U256 { decode(&self.spec().engine_params.get("account_start_nonce").unwrap()) }
	// TODO: refactor in terms of `on_preseal_block`
	fn block_reward(&self, _env_info: &EnvInfo) -> U256 { decode(&self.spec().engine_params.get("block_reward").unwrap()) }

	/// Block transformation functions, before and after the transactions.
//	fn on_new_block(&self, _env_info: &EnvInfo, _block: &mut Block) -> Result<(), EthcoreError> {}
//	fn on_preseal_block(&self, _env_info: &EnvInfo, _block: &mut Block) -> Result<(), EthcoreError> {}

	/// Verify that `header` is valid.
	/// `parent` (the parent header) and `block` (the header's full block) may be provided for additional
	/// checks. Returns either a null `Ok` or a general error detailing the problem with import.
	// TODO: consider including State in the params.
	fn verify(&self, _header: &Header, _parent: Option<&Header>, _block: Option<&[u8]>) -> Result<(), EthcoreError> { Ok(()) }

	/// Additional verification for transactions in blocks.
	// TODO: Add flags for which bits of the transaction to check.
	// TODO: consider including State in the params.
	fn verify_transaction(&self, _t: &Transaction, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Don't forget to call Super::populateFromParent when subclassing & overriding.
	// TODO: consider including State in the params.
	fn populate_from_parent(&self, _header: &mut Header, _parent: &Header) -> Result<(), EthcoreError> { Ok(()) }

	// TODO: buildin contract routing - to do this properly, it will require removing the built-in configuration-reading logic
	// from Spec into here and removing the Spec::builtins field.
/*	fn is_builtin(&self, a: Address) -> bool;
	fn cost_of_builtin(&self, a: Address, in: &[u8]) -> bignum;
	fn execute_builtin(&self, a: Address, in: &[u8], out: &mut [u8]);
*/

	// TODO: sealing stuff - though might want to leave this for later.
}
