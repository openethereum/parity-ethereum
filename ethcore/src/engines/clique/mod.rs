mod signer_snapshot;

use rlp::{encode, Decodable, DecoderError, Encodable, RlpStream, Rlp};

use std::sync::{Weak, Arc};
use std::collections::{BTreeMap, HashMap};
use std::{fmt, error};

use super::epoch::{PendingTransition,EpochVerifier,NoOp};

use account_provider::AccountProvider;
use builtin::Builtin;
use vm::{EnvInfo, Schedule, CreateContractAddress, CallType, ActionValue};
use error::Error;
use header::{Header, BlockNumber, ExtendedHeader};
use snapshot::SnapshotComponents;
use spec::CommonParams;
use transaction::{self, UnverifiedTransaction, SignedTransaction};
use client::EngineClient;
use parking_lot::RwLock;
use block::*;

use ethkey::{Password, Signature};
use parity_machine::{Machine, LocalizedMachine as Localized, TotalScoredHeader};
use ethereum_types::{H256, U256, Address};
use unexpected::{Mismatch, OutOfBounds};
use bytes::Bytes;
use types::ancestry_action::AncestryAction;
use engines::{Engine, Seal, EngineError, ConstructedVerifier};
use super::validator_set::{ValidatorSet, SimpleList};
use super::signer::EngineSigner;
use machine::{AuxiliaryData, EthereumMachine};
use self::signer_snapshot::SignerSnapshot;

static EPOCH_LENGTH: u64 = 10; // set low for testing (should be 30000 according to clique EIP)

pub struct Clique {
  client: RwLock<Option<Weak<EngineClient>>>,
  signer: RwLock<EngineSigner>,
  validators: Box<SignerSnapshot>,
  machine: EthereumMachine,
}

impl Clique {
  /// Check if current signer is the current proposer.
  fn is_signer_proposer(&self, bh: &H256) -> bool {
    //let proposer = self.view_proposer(bh, self.height.load(AtomicOrdering::SeqCst), self.view.load(AtomicOrdering::SeqCst));
    let proposer = self.validators.get(bh, 0);
    self.signer.read().is_address(&proposer)
  }

  pub fn new(machine: EthereumMachine) -> Self {
    Clique {
      client: RwLock::new(None),
      signer: Default::default(),
      machine: machine,
      validators:  Box::new(SignerSnapshot::new())
    }
  }
}

impl Engine<EthereumMachine> for Clique {
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
      return Seal::None;
    }

    let header_seal = block.header().seal().clone();
    let extra_data = block.header().extra_data().clone();

/*
    let seal = Seal::Regular(::rlp::encode_list(vec![
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
      extra_data[0..block.header().extra_data().len()-65],
      header_seal[0],
      header_seal[1],
      ]));
*/

 //   Seal::Regular(seal)
      Seal::Regular(vec!())
  }


  fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error>{
    // cast vote?
    Ok(())
  }

  fn on_new_block(
    &self,
    _block: &mut ExecutedBlock,
    _epoch_begin: bool,
    _ancestry: &mut Iterator<Item=ExtendedHeader>,
  ) -> Result<(), Error> {

    Ok(())
  }

  fn verify_block_basic(&self, _header: &Header) -> Result<(), Error> { 
    // don't allow blocks from the future

    // Checkpoint blocks need to enforce zero beneficiary
    if _header.number() % EPOCH_LENGTH == 0 {
      if _header.author() != &[0; 20].into() {
        return Err(Box::new("Checkpoint blocks need to enforce zero beneficiary").into());
      }

      if _header.seal()[1][0..32] != [0xff; 32] {
        return Err(Box::new("Seal nonce zeros enforced on checkpoints").into());
      }
    }

    // Nonces must be 0x00..0 or 0xff..f, zeroes enforced on checkpoints

    // Check that the extra-data contains both the vanity and signature

    // Ensure that the extra-data contains a signer list on checkpoint, but none otherwise

    // Ensure that the mix digest is zero as we don't have fork protection currently

    // Ensure that the block doesn't contain any uncles which are meaningless in PoA

    // Ensure that the block's difficulty is meaningful
    
    Ok(()) 
  }

  fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData)
      -> super::EpochChange<EthereumMachine>
  {
    super::EpochChange::No
  }

  fn is_epoch_end(
      &self,
      chain_head: &Header,
      _chain: &super::Headers<Header>,
      transition_store: &super::PendingTransitionStore,
  ) -> Option<Vec<u8>> {
    if chain_head.number() % EPOCH_LENGTH - 1 == 0 {
      // epoch end
      Some(vec!(0x0))
    } else {
      None
    }
  }

  fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {
    ConstructedVerifier::Trusted(Box::new(super::epoch::NoOp))
  }

  fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {

  }

  fn sign(&self, hash: H256) -> Result<Signature, Error> {
    unimplemented!()
  }

  fn stop(&self) { }

  fn register_client(&self, client: Weak<EngineClient>) {
  }

  fn verify_local_seal(&self, header: &Header) -> Result<(), Error> { Ok(()) }

  fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
    super::total_difficulty_fork_choice(new, current)
  }
}
