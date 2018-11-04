mod signer_snapshot;
mod params;
mod step_service;

use rlp::{encode, Decodable, DecoderError, Encodable, RlpStream, Rlp};
use std::time::{Duration};

use std::sync::{Weak, Arc};
use std::collections::{BTreeMap, HashMap};
use std::{fmt, error};
use hash::{keccak};

use self::params::CliqueParams;
use self::step_service::StepService;

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
use io::IoService;

use ethkey::{Password, Signature, recover as ec_recover};
use parity_machine::{Machine, LocalizedMachine as Localized, TotalScoredHeader};
use ethereum_types::{H256, U256, Address};
use unexpected::{Mismatch, OutOfBounds};
use bytes::Bytes;
use types::ancestry_action::AncestryAction;
use engines::{Engine, Seal, EngineError, ConstructedVerifier};
use super::validator_set::{ValidatorSet, SimpleList};
use super::signer::EngineSigner;
use machine::{AuxiliaryData, EthereumMachine};
//use self::signer_snapshot::SignerSnapshot;

const EPOCH_LENGTH: u32 = 10; // set low for testing (should be 30000 according to clique EIP)
const SIGNER_VANITY_LENGTH: u32 = 32;
const SIGNER_SIG_LENGTH: u32 = 65;
const EXTRA_DATA_POST_LENGTH: u32 = 128;
const NONCE_DROP_VOTE: [u8; 16] = [0x00; 16];
const NONCE_AUTH_VOTE: [u8; 16] = [0xff; 16];

pub struct Clique {
  client: RwLock<Option<Weak<EngineClient>>>,
  signer: RwLock<EngineSigner>,
  signers: Vec<Address>,
  machine: EthereumMachine,
  step_service: IoService<Duration>,
}

/*
 * only sign over non-signature bytes (vanity data).  There shouldn't be a signature here to sign
 * yet.
 */
pub fn sig_hash(header: &Header) -> Result<H256, Error> {
  if header.extra_data().len() >= SIGNER_VANITY_LENGTH as usize {
    let mut reduced_header = header.clone();
    let mut extra_data: [u8; SIGNER_VANITY_LENGTH as usize] = [0; SIGNER_VANITY_LENGTH as usize];
    extra_data.clone_from_slice(&reduced_header.extra_data()[0..SIGNER_VANITY_LENGTH as usize]);
    reduced_header.set_extra_data(extra_data.to_vec());
    Ok(keccak(::rlp::encode(&reduced_header)))
  } else {
    Ok(keccak(::rlp::encode(header)))
  }
}

const step_time: Duration = Duration::from_millis(100);

impl Clique {

  /// Check if current signer is the current proposer.
  fn is_signer_proposer(&self, bh: &H256) -> bool {
    //let proposer = self.view_proposer(bh, self.height.load(AtomicOrdering::SeqCst), self.view.load(AtomicOrdering::SeqCst));
    //let proposer = self.validators.get(bh);
    if let Some(_) = self.signers.iter().find(|x| self.signer.read().is_address(x)) {
      true
    } else {
      false
    }
  }

  pub fn new(our_params: CliqueParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
    // don't let there be any duplicate signers

    //length of signers must be greater than 1
    //

    trace!(target: "engine", "clique started with {} validators", our_params.signers.len());
    let engine = Arc::new(
	  Clique {
		  client: RwLock::new(None),
		  signer: Default::default(),
		  signers: our_params.signers,
		  machine: machine,
		  step_service: IoService::<Duration>::start()?,
		});


	let handler = StepService::new(Arc::downgrade(&engine) as Weak<Engine<_>>, step_time);
	engine.step_service.register_handler(Arc::new(handler))?;

    return Ok(engine);
  }

  fn sign_header(&self, header: &Header) -> Result<Signature, Error> {
    let digest = sig_hash(header)?;
    if let Ok(sig) = self.signer.read().sign(digest) {
      Ok(sig)
    } else {
      Err(Box::new("failed to sign header").into())
    }
  }

  //pub fn snapshot(self, bn: u64) -> AuthorizationSnapshot {
    // if we are on a checkpoint block, build a snapshot
  //}
}

impl Engine<EthereumMachine> for Clique {
  fn name(&self) -> &str { "Clique" }

  // nonce + mixHash + extraData
  fn seal_fields(&self, _header: &Header) -> usize { 1 }
  fn machine(&self) -> &EthereumMachine { &self.machine }
  fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 0 }
  fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
    /* ? */
  }


  fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {
    self.signer.write().set(ap, address, password);
  }

  /// None means that it requires external input (e.g. PoW) to seal a block.
  /// /// Some(true) means the engine is currently prime for seal generation (i.e. node
  ///     is the current validator).
  /// /// Some(false) means that the node might seal internally but is not qualified
  ///     now.
  ///
  fn seals_internally(&self) -> Option<bool> {
    //trace!(target: "engine", "is there a signer: {}", self.signer.read().is_some());
    //Some(self.signer.read().is_some())
    Some(true)
  }

  /// Attempt to seal generate a proposal seal.
  ///
  /// This operation is synchronous and may (quite reasonably) not be available, in which case
  /// `Seal::None` will be returned.
  fn generate_seal(&self, block: &ExecutedBlock, _parent: &Header) -> Seal {
    let mut header = block.header.clone();

    // don't seal the genesis block
    if header.number() == 0 {
      return Seal::None;
    }

    // if sealing period is 0, refuse to seal

    // let vote_snapshot = self.snapshot.get(bh);

    // if we are not authorized to sign, don't seal

    // if we signed recently, don't seal

    let authorized = if let Some(pos) = self.signers.iter().position(|x| self.signer.read().is_address(x)) {
      block.header.number() % ((pos as u64) + 1) == 0 
    } else {
      false
    };

    // sign the digest of the seal
    if authorized {

      // todo 
      let mut extra_data: Vec<u8> = vec!(0; SIGNER_VANITY_LENGTH as usize + SIGNER_SIG_LENGTH as usize);

      // ensure header has correct extra data size before signing
      header.set_extra_data(extra_data);

      match self.sign_header(&header) {
          Ok(sig) => {
            trace!(target: "engine", "sealed block {}", block.header.number());
            Seal::Regular(vec!(sig[0..65].to_vec()))
          },
          Err(err) => {
            trace!(target: "engine", "failed to seal block: {}", err);
            Seal::None
          }
      }
    } else {
      Seal::None
    }
  }

  fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error>{

      /*
       * TODO:
      if not checkpoint block:
        if the block was successfully sealed, then grab the signature from the seal data and
        append it to the block extraData
        */
    trace!(target: "engine", "closing block...");
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
    if _header.number() == 0 {
      return Err(Box::new("cannot verify genesis block").into());
    }

    // don't allow blocks from the future

    // Checkpoint blocks need to enforce zero beneficiary
    if _header.number() % EPOCH_LENGTH as u64 == 0 {
      if _header.author() != &[0; 20].into() {
        return Err(Box::new("Checkpoint blocks need to enforce zero beneficiary").into());
      }

      if _header.extra_data()[0..32] != [0xff; 32] {
        return Err(Box::new("Seal nonce zeros enforced on checkpoints").into());
      }
    } else {
        // TODO 
        // - ensure header extraData has length SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH
        // - ensure header signature corresponds to the right validator for the turn-ness of the
        // block
    }

    // Nonces must be 0x00..0 or 0xff..f, zeroes enforced on checkpoints

    // Check that the extra-data contains both the vanity and signature

    // Ensure that the extra-data contains a signer list on checkpoint, but none otherwise

    // Ensure that the mix digest is zero as we don't have fork protection currently

    // Ensure that the block doesn't contain any uncles which are meaningless in PoA

    // Ensure that the block's difficulty is meaningful

    // ...

    // TODO verify signer is valid
    // let signer_address = ec_recover(_header)?.expect(Err(Box::new("fuck").into()));

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
    if chain_head.number() % EPOCH_LENGTH as u64 - 1 == 0 {
      // epoch end
      Some(vec!(0x0))
    } else {
      None
    }
  }

  fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {
    ConstructedVerifier::Trusted(Box::new(super::epoch::NoOp))
  }

  /*
   * Continuously trigger attempts to seal new blocks
   */
  fn step(&self) {
	if let Some(ref weak) = *self.client.read() {
		if let Some(c) = weak.upgrade() {
			c.update_sealing();
		}
	}
  }

  fn sign(&self, hash: H256) -> Result<Signature, Error> {
    Ok(self.signer.read().sign(hash)?)
  }

  fn stop(&self) { }

  fn register_client(&self, client: Weak<EngineClient>) {
	*self.client.write() = Some(client.clone());
	//self.validators.register_client(client);
  }

  fn verify_local_seal(&self, header: &Header) -> Result<(), Error> { Ok(()) }

  fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
    super::total_difficulty_fork_choice(new, current)
  }
}
