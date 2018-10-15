impl Engine<EthereumMachine> for Clique {
	fn name(&self) -> &str { "Clique" }
	fn seal_fields(&self, _header: &Header) -> usize { /* ? */ }
	fn machine(&self) -> &EthereumMachine { &self.machine }
	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 0 }
	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
		/* ? */
	}

    /// None means that it requires external input (e.g. PoW) to seal a block.
    /// /// Some(true) means the engine is currently prime for seal generation (i.e. node
    ///     is the current validator).
    /// /// Some(false) means that the node might seal internally but is not qualified
    ///     now.
    ///
    fn seals_internally(&self) -> Option<bool> {
        if (isTurnToValidate) {
            Some(true)
        } else {
            Some(false)
        }
    }

    /// Attempt to seal generate a proposal seal.
    ///
    /// This operation is synchronous and may (quite reasonably) not be available, in which case
    /// `Seal::None` will be returned.
    fn generate_seal(&self, block: &ExecutedBlock, _parent: &Header) -> Seal {
        /* ? */
    }

    fn handle_message(&self, rlp: &[u8]) -> Result<(), EngineError> {
		/* ? this should probably do nothing.  all consensus is handled within block headers */
	}

    fn on_new_block(&self, block: &mut ExecutedBlock, epoch_begin: bool, _ancestry: &mut Iterator<Item=ExtendedHeader>) -> Result<(), Error> {
		/* ? */
	}

    fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error>{
		/* probably a no-op.  no block rewards to apply */
	}

    fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {

	}

	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {

	}

    fn verify_block_external(&self, header: &Header) -> Result<(), Error> {

	}


    fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData)
        -> super::EpochChange<EthereumMachine>
	{

	}

    fn is_epoch_end(
        &self,
        chain_head: &Header,
        _chain: &super::Headers<Header>,
        transition_store: &super::PendingTransitionStore,
    ) -> Option<Vec<u8>> {

	}

    fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {

	}

    fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {

	}

    fn sign(&self, hash: H256) -> Result<Signature, Error> {

	}

    fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {

	}

	fn stop(&self) {
    }

	fn register_client(&self, client: Weak<EngineClient>) {
	}

    fn step(&self) {
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
	}
}
