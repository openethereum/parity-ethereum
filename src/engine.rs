use std::collections::hash_map::*;
use util::bytes::*;
use util::hash::*;
use util::uint::*;
use util::rlp::*;
use util::semantic_version::*;
use util::error::*;
use header::Header;
use transaction::Transaction;
use block::Block;
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
	fn account_start_nonce(&self) -> U256 { decode(&self.spec().engine_params.get("account_start_nonce").unwrap()) }

	/// Block transformation functions, before and after the transactions.
	fn on_new_block(&self, _block: &mut Block) {}
	fn on_close_block(&self, _block: &mut Block) {}

	/// Verify that `header` is valid.
	/// `parent` (the parent header) and `block` (the header's full block) may be provided for additional
	/// checks. Returns either a null `Ok` or a general error detailing the problem with import.
	// TODO: consider including State in the params.
	fn verify_block(&self, _header: &Header, _parent: Option<&Header>, _block: Option<&[u8]>) -> Result<(), EthcoreError> { Ok(()) }

	/// Additional verification for transactions in blocks.
	// TODO: Add flags for which bits of the transaction to check.
	// TODO: consider including State in the params.
	fn verify_transaction(&self, _t: &Transaction, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Don't forget to call Super::populateFromParent when subclassing & overriding.
	// TODO: consider including State in the params.
	fn populate_from_parent(&self, _header: &mut Header, _parent: &Header) {}

	// TODO: builtin contract routing - to do this properly, it will require removing the built-in configuration-reading logic
	// from Spec into here and removing the Spec::builtins field.
	fn is_builtin(&self, a: &Address) -> bool { self.spec().builtins.contains_key(a) }
	fn cost_of_builtin(&self, a: &Address, input: &[u8]) -> U256 { self.spec().builtins.get(a).unwrap().cost(input.len()) }
	fn execute_builtin(&self, a: &Address, input: &[u8], output: &mut [u8]) { self.spec().builtins.get(a).unwrap().execute(input, output); }

	// TODO: sealing stuff - though might want to leave this for later.
}
