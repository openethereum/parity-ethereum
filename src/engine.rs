use std::collections::hash_map::*;
use util::bytes::*;
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

	/// Get the EVM schedule for 
	fn evm_schedule(&self, env_info: &EnvInfo) -> EvmSchedule;

	/// Verify that `header` is valid.
	/// `parent` (the parent header) and `block` (the header's full block) may be provided for additional
	/// checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify(&self, _header: &Header, _parent: Option<&Header>, _block: Option<&[u8]>) -> Result<(), EthcoreError> { Ok(()) }

	/// Additional verification for transactions in blocks.
	// TODO: Add flags for which bits of the transaction to check.
	fn verify_transaction(&self, _t: &Transaction, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Don't forget to call Super::populateFromParent when subclassing & overriding.
	fn populate_from_parent(&self, _header: &mut Header, _parent: &Header) -> Result<(), EthcoreError> { Ok(()) }

	// TODO: buildin contract routing - this will require removing the built-in configuration reading logic from Spec
	// into here and removing the Spec::builtins field. It's a big job.
/*	fn is_builtin(&self, a: Address) -> bool;
	fn cost_of_builtin(&self, a: Address, in: &[u8]) -> bignum;
	fn execute_builtin(&self, a: Address, in: &[u8], out: &mut [u8]);
*/
}
