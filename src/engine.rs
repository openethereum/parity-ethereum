use common::*;
use block::ExecutedBlock;
use spec::Spec;
use evm::Schedule;
use evm::Factory;

/// A consensus mechanism for the chain. Generally either proof-of-work or proof-of-stake-based.
/// Provides hooks into each of the major parts of block import.
pub trait Engine : Sync + Send {
	/// The name of this engine.
	fn name(&self) -> &str;
	/// The version of this engine. Should be of the form 
	fn version(&self) -> SemanticVersion { SemanticVersion::new(0, 0, 0) }

	/// The number of additional header fields required for this engine.
	fn seal_fields(&self) -> usize { 0 }
	/// Default values of the additional fields RLP-encoded in a raw (non-list) harness.
	fn seal_rlp(&self) -> Bytes { vec![] }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> HashMap<String, String> { HashMap::new() }

	/// Get the general parameters of the chain.
	fn spec(&self) -> &Spec;

	/// Get current EVM factory
	fn vm_factory(&self) -> &Factory;

	/// Get the EVM schedule for the given `env_info`.
	fn schedule(&self, env_info: &EnvInfo) -> Schedule;

	/// Some intrinsic operation parameters; by default they take their value from the `spec()`'s `engine_params`.
	fn maximum_extra_data_size(&self) -> usize { decode(&self.spec().engine_params.get("maximumExtraDataSize").unwrap()) }
	/// TODO [Gav Wood] Please document me
	fn maximum_uncle_count(&self) -> usize { 2 }
	/// TODO [Gav Wood] Please document me
	fn account_start_nonce(&self) -> U256 { decode(&self.spec().engine_params.get("accountStartNonce").unwrap()) }

	/// Block transformation functions, before and after the transactions.
	fn on_new_block(&self, _block: &mut ExecutedBlock) {}
	/// TODO [Gav Wood] Please document me
	fn on_close_block(&self, _block: &mut ExecutedBlock) {}

	// TODO: consider including State in the params for verification functions.
	/// Phase 1 quick block verification. Only does checks that are cheap. `block` (the header's full block) 
	/// may be provided for additional checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_basic(&self, _header: &Header,  _block: Option<&[u8]>) -> Result<(), Error> { Ok(()) }

	/// Phase 2 verification. Perform costly checks such as transaction signatures. `block` (the header's full block) 
	/// may be provided for additional checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_unordered(&self, _header: &Header, _block: Option<&[u8]>) -> Result<(), Error> { Ok(()) }

	/// Phase 3 verification. Check block information against parent and uncles. `block` (the header's full block) 
	/// may be provided for additional checks. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_family(&self, _header: &Header, _parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> { Ok(()) }

	/// Additional verification for transactions in blocks.
	// TODO: Add flags for which bits of the transaction to check.
	// TODO: consider including State in the params.
	fn verify_transaction_basic(&self, _t: &Transaction, _header: &Header) -> Result<(), Error> { Ok(()) }
	/// TODO [Gav Wood] Please document me
	fn verify_transaction(&self, _t: &Transaction, _header: &Header) -> Result<(), Error> { Ok(()) }

	/// Don't forget to call Super::populateFromParent when subclassing & overriding.
	// TODO: consider including State in the params.
	fn populate_from_parent(&self, _header: &mut Header, _parent: &Header) {}

	// TODO: builtin contract routing - to do this properly, it will require removing the built-in configuration-reading logic
	// from Spec into here and removing the Spec::builtins field.
	/// TODO [Gav Wood] Please document me
	fn is_builtin(&self, a: &Address) -> bool { self.spec().builtins.contains_key(a) }
	/// TODO [Gav Wood] Please document me
	fn cost_of_builtin(&self, a: &Address, input: &[u8]) -> U256 { self.spec().builtins.get(a).unwrap().cost(input.len()) }
	/// TODO [Gav Wood] Please document me
	fn execute_builtin(&self, a: &Address, input: &[u8], output: &mut [u8]) { self.spec().builtins.get(a).unwrap().execute(input, output); }

	// TODO: sealing stuff - though might want to leave this for later.
}
