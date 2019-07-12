// todo: module docs
use std::{
	collections::BTreeMap,
	sync::Weak
};

use call_contract::CallContract;
use common_types::{
	ancestry_action::AncestryAction,
	BlockNumber,
	ids::BlockId,
	header::{Header, ExtendedHeader},
	encoded,
	transaction::{self, UnverifiedTransaction, SignedTransaction},
	engines::{
		EthashExtensions, SealingState, Headers, PendingTransitionStore,
		epoch::{EpochChange, ConstructedVerifier},
		params::CommonParams,
		machine::{self, AuxiliaryData},
	},
	errors::{EthcoreError, EngineError},
};
use ethcore_builtin::Builtin;
use ethereum_types::{H256, U256, Address};
use ethkey::Signature;
use vm::{Schedule, EnvInfo, CreateContractAddress};

/// Provides various information on a block by it's ID
pub trait BlockInfo {
	/// Get raw block header data by block id.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

	/// Get the best block header.
	fn best_block_header(&self) -> Header;

	/// Get raw block data by block header hash.
	fn block(&self, id: BlockId) -> Option<encoded::Block>;

	/// Get address code hash at given block's state.
	fn code_hash(&self, address: &Address, id: BlockId) -> Option<H256>;
}

pub trait VerifyingClient: BlockInfo + CallContract {}

/// todo rewrite docs: A consensus mechanism for the chain. Generally either proof-of-work or proof-of-stake-based.
/// Provides hooks into each of the major parts of block import.
pub trait VerifyingEngine: Sync + Send {
	/// The number of additional header fields required for this engine.
	fn seal_fields(&self, _header: &Header) -> usize { 0 }

	/// Maximum number of uncles a block is allowed to declare.
	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 0 }

	/// Optional maximum gas limit.
	fn maximum_gas_limit(&self) -> Option<U256> { None }

	/// Some intrinsic operation parameters; by default they take their value from the `spec()`'s `engine_params`.
	fn maximum_extra_data_size(&self) -> usize { self.params().maximum_extra_data_size }

	/// Get the general parameters of the chain.
	fn params(&self) -> &CommonParams;

	/// Get a reference to the ethash-specific extensions.
	fn ethash_extensions(&self) -> Option<&EthashExtensions> { None }

	/// Phase 1 quick block verification. Only does checks that are cheap. Returns either a null `Ok` or a general error detailing the problem with import.
	/// The verification module can optionally avoid checking the seal (`check_seal`), if seal verification is disabled this method won't be called.
	fn verify_block_basic(&self, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Phase 2 verification. Perform costly checks such as transaction signatures. Returns either a null `Ok` or a general error detailing the problem with import.
	/// The verification module can optionally avoid checking the seal (`check_seal`), if seal verification is disabled this method won't be called.
	fn verify_block_unordered(&self, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Phase 3 verification. Check block information against parent. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_family(&self, _header: &Header, _parent: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Phase 4 verification. Verify block header against potentially external data.
	/// Should only be called when `register_client` has been called previously.
	fn verify_block_external(&self, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Perform basic/cheap transaction verification.
	///
	/// This should include all cheap checks that can be done before
	/// actually checking the signature, like chain-replay protection.
	///
	/// NOTE This is done before the signature is recovered so avoid
	/// doing any state-touching checks that might be expensive.
	///
	/// TODO: Add flags for which bits of the transaction to check.
	/// TODO: consider including State in the params.
	fn verify_transaction_basic(&self, _t: &UnverifiedTransaction, _header: &Header) -> Result<(), transaction::Error>;

	/// Verify a particular transaction is valid.
	///
	/// Unordered verification doesn't rely on the transaction execution order,
	/// i.e. it should only verify stuff that doesn't assume any previous transactions
	/// has already been verified and executed.
	///
	/// NOTE This function consumes an `UnverifiedTransaction` and produces `SignedTransaction`
	/// which implies that a heavy check of the signature is performed here.
	fn verify_transaction_unordered(&self, _t: UnverifiedTransaction, _header: &Header) -> Result<SignedTransaction, transaction::Error>;

	// Transactions are verified against the parent header since the current
	// state wasn't available when the tx was created
	fn verify_transactions(
		&self,
		txs: &Vec<SignedTransaction>,
		parent: &Header,
		client: &dyn VerifyingClient,
	) -> Result<(), transaction::Error>;

	/// Check whether the parent timestamp is valid.
	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp > parent_timestamp
	}
}

// todo: plenty of things:
//  1. ExecutedBlock is a problem: it embeds a `State<StateDB>` (both State and StateDB crates use common_types)
//  2. ConstructedVerifier uses the EpochVerifier trait that has to be extracted: this is easy, it uses primitive types.
//  3. EpochChange uses Proof which has a variant `WithState` that uses a `StateDependentProof` trait which uses Machine in one method: `check_proof`. This trait has a single implementor in `StateProof` (in safe_contract.rs)

// Slightly unrelated: ClientIoMessage::Execute entangles a `&Client` but I don't think that is needed. Seems to be used only by private-tx but there the `Client` param is not used (passes a `Provider` to a closure instead).
pub trait Engine: Sync + Send {
	/// The name of this engine.
	fn name(&self) -> &str;

	/// Get access to the underlying state machine.
	// TODO: decouple.
//	fn machine(&self) -> &Machine;

	/// The number of additional header fields required for this engine.
	fn seal_fields(&self, _header: &Header) -> usize { 0 }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> BTreeMap<String, String> { BTreeMap::new() }

	/// Optional maximum gas limit.
	fn maximum_gas_limit(&self) -> Option<U256> { None }

	/// Block transformation functions, before the transactions.
	/// `epoch_begin` set to true if this block kicks off an epoch.
	fn on_new_block(
		&self,
		_block: &mut ExecutedBlock,
		_epoch_begin: bool,
	) -> Result<(), EthcoreError> {
		Ok(())
	}

	/// Block transformation functions, after the transactions.
	fn on_close_block(
		&self,
		_block: &mut ExecutedBlock,
		_parent_header: &Header,
	) -> Result<(), EthcoreError> {
		Ok(())
	}

	/// Allow mutating the header during seal generation. Currently only used by Clique.
	fn on_seal_block(&self, _block: &mut ExecutedBlock) -> Result<(), EthcoreError> { Ok(()) }

	/// Returns the engine's current sealing state.
	fn sealing_state(&self) -> SealingState { SealingState::External }

	/// Attempt to seal the block internally.
	///
	/// If `Some` is returned, then you get a valid seal.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which None will
	/// be returned.
	///
	/// It is fine to require access to state or a full client for this function, since
	/// light clients do not generate seals.
	fn generate_seal(&self, _block: &ExecutedBlock, _parent: &Header) -> Seal { Seal::None }

	/// Verify a locally-generated seal of a header.
	///
	/// If this engine seals internally,
	/// no checks have to be done here, since all internally generated seals
	/// should be valid.
	///
	/// Externally-generated seals (e.g. PoW) will need to be checked for validity.
	///
	/// It is fine to require access to state or a full client for this function, since
	/// light clients do not generate seals.
	fn verify_local_seal(&self, header: &Header) -> Result<(), EthcoreError>;

	/// Phase 1 quick block verification. Only does checks that are cheap. Returns either a null `Ok` or a general error detailing the problem with import.
	/// The verification module can optionally avoid checking the seal (`check_seal`), if seal verification is disabled this method won't be called.
	fn verify_block_basic(&self, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Phase 2 verification. Perform costly checks such as transaction signatures. Returns either a null `Ok` or a general error detailing the problem with import.
	/// The verification module can optionally avoid checking the seal (`check_seal`), if seal verification is disabled this method won't be called.
	fn verify_block_unordered(&self, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Phase 3 verification. Check block information against parent. Returns either a null `Ok` or a general error detailing the problem with import.
	fn verify_block_family(&self, _header: &Header, _parent: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Phase 4 verification. Verify block header against potentially external data.
	/// Should only be called when `register_client` has been called previously.
	fn verify_block_external(&self, _header: &Header) -> Result<(), EthcoreError> { Ok(()) }

	/// Genesis epoch data.
	fn genesis_epoch_data<'a>(&self, _header: &Header, _state: &machine::Call) -> Result<Vec<u8>, String> { Ok(Vec::new()) }

	/// Whether an epoch change is signalled at the given header but will require finality.
	/// If a change can be enacted immediately then return `No` from this function but
	/// `Yes` from `is_epoch_end`.
	///
	/// If auxiliary data of the block is required, return an auxiliary request and the function will be
	/// called again with them.
	/// Return `Yes` or `No` when the answer is definitively known.
	///
	/// Should not interact with state.
	fn signals_epoch_end<'a>(&self, _header: &Header, _aux: AuxiliaryData<'a>) -> EpochChange {
		EpochChange::No
	}

	/// Whether a block is the end of an epoch.
	///
	/// This either means that an immediate transition occurs or a block signalling transition
	/// has reached finality. The `Headers` given are not guaranteed to return any blocks
	/// from any epoch other than the current. The client must keep track of finality and provide
	/// the latest finalized headers to check against the transition store.
	///
	/// Return optional transition proof.
	fn is_epoch_end(
		&self,
		_chain_head: &Header,
		_finalized: &[H256],
		_chain: &Headers<Header>,
		_transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		None
	}

	/// Whether a block is the end of an epoch.
	///
	/// This either means that an immediate transition occurs or a block signalling transition
	/// has reached finality. The `Headers` given are not guaranteed to return any blocks
	/// from any epoch other than the current. This is a specialized method to use for light
	/// clients since the light client doesn't track finality of all blocks, and therefore finality
	/// for blocks in the current epoch is built inside this method by the engine.
	///
	/// Return optional transition proof.
	fn is_epoch_end_light(
		&self,
		_chain_head: &Header,
		_chain: &Headers<Header>,
		_transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		None
	}

	/// Create an epoch verifier from validation proof and a flag indicating
	/// whether finality is required.
	fn epoch_verifier<'a>(&self, _header: &Header, _proof: &'a [u8]) -> ConstructedVerifier<'a> {
		ConstructedVerifier::Trusted(Box::new(NoOp))
	}

	/// Populate a header's fields based on its parent's header.
	/// Usually implements the chain scoring rule based on weight.
	fn populate_from_parent(&self, _header: &mut Header, _parent: &Header) { }

	/// Handle any potential consensus messages;
	/// updating consensus state and potentially issuing a new one.
	fn handle_message(&self, _message: &[u8]) -> Result<(), EngineError> { Err(EngineError::UnexpectedMessage) }

	/// Register a component which signs consensus messages.
	fn set_signer(&self, _signer: Box<dyn EngineSigner>) {}

	/// Sign using the EngineSigner, to be used for consensus tx signing.
	fn sign(&self, _hash: H256) -> Result<Signature, EthcoreError> { unimplemented!() }

	/// Add Client which can be used for sealing, potentially querying the state and sending messages.
	fn register_client(&self, _client: Weak<dyn EngineClient>) {}

	/// Trigger next step of the consensus engine.
	fn step(&self) {}

	/// Create a factory for building snapshot chunks and restoring from them.
	/// Returning `None` indicates that this engine doesn't support snapshot creation.
	fn snapshot_components(&self) -> Option<Box<dyn SnapshotComponents>> {
		None
	}

	/// Whether this engine supports warp sync.
	fn supports_warp(&self) -> bool {
		self.snapshot_components().is_some()
	}

	/// Return a new open block header timestamp based on the parent timestamp.
	fn open_block_header_timestamp(&self, parent_timestamp: u64) -> u64 {
		use std::{time, cmp};

		let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap_or_default();
		cmp::max(now.as_secs() as u64, parent_timestamp + 1)
	}

	/// Gather all ancestry actions. Called at the last stage when a block is committed. The Engine must guarantee that
	/// the ancestry exists.
	fn ancestry_actions(&self, _header: &Header, _ancestry: &mut dyn Iterator<Item = ExtendedHeader>) -> Vec<AncestryAction> {
		Vec::new()
	}

	/// Returns author should used when executing tx's for this block.
	fn executive_author(&self, header: &Header) -> Result<Address, EthcoreError> {
		Ok(*header.author())
	}

	/// Get the EVM schedule for the given block number.
	fn schedule(&self, block_number: BlockNumber) -> Schedule;
//	{
//		self.machine().schedule(block_number)
//	}

	/// Builtin-contracts for the chain..
	fn builtins(&self) -> &BTreeMap<Address, Builtin>;
//	{
//		self.machine().builtins()
//	}

	/// Attempt to get a handle to a built-in contract.
	/// Only returns references to activated built-ins.
	fn builtin(&self, a: &Address, block_number: BlockNumber) -> Option<&Builtin>;
//	{
//		self.machine().builtin(a, block_number)
//	}

	/// The nonce with which accounts begin at given block.
	fn account_start_nonce(&self, block: BlockNumber) -> U256;
//	{
//		self.machine().account_start_nonce(block)
//	}

	/// The network ID that transactions should be signed with.
	fn signing_chain_id(&self, env_info: &EnvInfo) -> Option<u64>;
//	{
//		self.machine().signing_chain_id(env_info)
//	}

	/// Returns new contract address generation scheme at given block number.
	fn create_address_scheme(&self, number: BlockNumber) -> CreateContractAddress;
//	{
//		self.machine().create_address_scheme(number)
//	}

	/// Verify a particular transaction is valid.
	///
	/// Unordered verification doesn't rely on the transaction execution order,
	/// i.e. it should only verify stuff that doesn't assume any previous transactions
	/// has already been verified and executed.
	///
	/// NOTE This function consumes an `UnverifiedTransaction` and produces `SignedTransaction`
	/// which implies that a heavy check of the signature is performed here.
	fn verify_transaction_unordered(&self, t: UnverifiedTransaction, header: &Header) -> Result<SignedTransaction, transaction::Error>;
//	{
//		self.machine().verify_transaction_unordered(t, header)
//	}

	/// Perform basic/cheap transaction verification.
	///
	/// This should include all cheap checks that can be done before
	/// actually checking the signature, like chain-replay protection.
	///
	/// NOTE This is done before the signature is recovered so avoid
	/// doing any state-touching checks that might be expensive.
	///
	/// TODO: Add flags for which bits of the transaction to check.
	/// TODO: consider including State in the params.
	fn verify_transaction_basic(&self, t: &UnverifiedTransaction, header: &Header) -> Result<(), transaction::Error>;
//	{
//		self.machine().verify_transaction_basic(t, header)
//	}

	/// Performs pre-validation of RLP decoded transaction before other processing
	fn decode_transaction(&self, transaction: &[u8]) -> Result<UnverifiedTransaction, transaction::Error>;
//	{
//		self.machine().decode_transaction(transaction)
//	}
}
