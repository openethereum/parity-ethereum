use rlp::{encode, Decodable, DecoderError, Encodable, RlpStream, Rlp};

struct Clique {
  client: RwLock<Option<Weak<EngineClient>>>,
  signer: RwLock<EngineSigner>,
  validators: Box<ValidatorSet>,
}

impl Engine<EthereumMachine> for Clique {
  const EPOCH_LENGTH: i32 = 10 // set low for testing (should be 30000 according to clique EIP)

	fn name(&self) -> &str { "Clique" }

  // nonce + mixHash + extraData
	fn seal_fields(&self, _header: &Header) -> usize { 3 }
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
    Some(self.signer.read().is_some())
  }

  /// Attempt to seal generate a proposal seal.
  ///
  /// This operation is synchronous and may (quite reasonably) not be available, in which case
  /// `Seal::None` will be returned.
  fn generate_seal(&self, block: &ExecutedBlock, _parent: &Header) -> Seal {
    if !self.is_signer_proposer(block.header.parent_hash()) {
      Seal::None
    }

    let header_seal = block.header().seal().clone()

    let seal = ::rlp::encode_list(vec![
      block.header().parent_hash(),
      block.header().uncles_hash(),
      block.header().author(),
      block.header().state_root(),
      block.header().transactions_root(),
      block.header().receipts_root(),
      block.header().log_bloom(),
      block.header().difficulty(),
      block.header().number(),
      block.header().gas_limit(),
      block.header().gas_used(),
      block.header().timestamp(),
      block.header().extra_data()[:block.header().extra_data().len()-65],
      header_seal[0],
      header_seal[1],
      ])

    Seal::Regular(seal)
  }

  /// Check if current signer is the current proposer.
  fn is_signer_proposer(&self, bh: &H256) -> bool {
    //let proposer = self.view_proposer(bh, self.height.load(AtomicOrdering::SeqCst), self.view.load(AtomicOrdering::SeqCst));
    let proposer = self.validators.get(bh, 0)
    self.signer.read().is_address(&proposer)
  }

  fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error>{
    // cast vote?

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
    if chain_head.header.get_block_number() % EPOCH_LENGTH - 1 == 0 {
      // epoch end
      Some(data)
    }
    None
  }

  fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {

  }

  fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {

  }

  fn sign(&self, hash: H256) -> Result<Signature, Error> {

  }

	fn stop(&self) { }

	fn register_client(&self, client: Weak<EngineClient>) {
	}
}
