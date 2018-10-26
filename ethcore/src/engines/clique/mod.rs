mod signer_snapshot;

use rlp::{encode, Decodable, DecoderError, Encodable, RlpStream, Rlp};

use std::sync::{Weak, Arc};
use std::collections::{BTreeMap, HashMap};
use std::{fmt, error};
use hash::{keccak};
use self::params::{CliqueParams}

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

static EPOCH_LENGTH: u32 = 10; // set low for testing (should be 30000 according to clique EIP)
static SIGNER_VANITY_LENGTH: u32 = 32;
static SIGNER_SIG_LENGTH: u32 = 65;
static NONCE_DROP_VOTE: [u8; 16] = [0x00; 16];
static NONCE_AUTH_VOTE: [u8; 16] = [0xff; 16];

pub struct Clique {
  client: RwLock<Option<Weak<EngineClient>>>,
  signer: RwLock<EngineSigner>,
  signers: Box<Vec<Address>>,
  //validators: Box<SignerSnapshot>,
  machine: EthereumMachine,
}

impl Clique {
  fn sig_hash(header: &Header) -> Result<H256, Error> {
    if header.extra_data().len() != SIGNER_VANITY_LENGTH as usize + SIGNER_SIG_LENGTH as usize {
      return Err(Box::new("bad signer extra_data length").into());
    } else {
      let mut reduced_header = header.clone();

      // only sign the "vanity" bytes
      reduced_header.set_extra_data(reduced_header.extra_data()[0..SIGNER_VANITY_LENGTH as usize].to_vec());

      Ok(keccak(::rlp::encode(&reduced_header)))
    }
  }

/*
  fn ecrecover(header: Header) -> Result<Address, Error> {
    let sig = &header.extra_data()[0..SIGNER_VANITY_LENGTH];
    let hash = Self::sig_hash(&header)?;

    let r = H256::from_slice(&sig[0..32]);
    let s = H256::from_slice(&sig[32..64]);
    let v = sig[64];

    let bit = match v {
      27 | 28 if v == 0 => v - 27,
      _ => { return Err(Box::new("v not correct").into()); },
    };

    let s = Signature::from_rsv(&r, &s, bit);
    if s.is_valid() {
      if let Ok(p) = ec_recover(&s, &hash) {
        let r = keccak(p);
        Ok(r[0..160].into())
      } else {
        Err(Box::new("ec_recover failed").into())
      }
    } else {
      Err(Box::new("Invalid sig...").into())
    }
  }
*/

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

  pub fn new(our_params: CliqueParams, machine: EthereumMachine) -> Self {
    // don't let there be any duplicate signers

    //length of signers must be greater than 1
    Clique {
      client: RwLock::new(None),
      signer: Default::default(),
      signers: our_params.signers,
      machine: machine,
      //validators:  Box::new(SignerSnapshot::new())
    }
  }

  //pub fn snapshot(self, bn: u64) -> AuthorizationSnapshot {
    // if we are on a checkpoint block, build a snapshot
  //}

  fn sign_header(self, header: &Header) -> Result<Signature, Error> {
    let digest = Self::sig_hash(header)?;
    if let Some(sig) = self.signer.read().sign(digest) {
      Ok(sig)
    } else {
      Err(Box::new("sign_header: signing failed").into())
    }
  }

  fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {
    self.signer.write().set(ap, address, password);
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
    let header = block.header;

    // don't seal the genesis block
    if header.number() == 0 {
      return Seal::None;
    }

    // if sealing period is 0, refuse to seal

    // let vote_snapshot = self.snapshot.get(bh);

    // if we are not authorized to sign, don't seal

    // if we signed recently, don't seal

    let authorized = if let Some(pos) = self.signers.iter().position(|x| self.signer.read().is_address(x)) {
      block.header.number() % pos as u64 == 0 
    } else {
      false
    };

    // sign the digest of the seal
    if authorized {
      if let Some(sig) = self.sign_header(&header) {
        Seal::Regular(self.sign_header(&header)?.into())
      } else {
        Seal::None
      }
    } else {
      Seal::None
    }

    // if authorized {
      // set difficulty to "in turn"
    // } else {
      // set difficulty to "not in turn"
      // if we already delayed:
      //   kick off delay
      // else:
      //   seal
    // }

    // let header_seal = block.header().seal().clone();
    //let extra_data = block.header().extra_data().clone();
    // let extra_data: [u8; 32] = vec!

 //   Seal::Regular(seal)
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
    if _header.number() % EPOCH_LENGTH as u64 == 0 {
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
    Ok(self.signer.read().sign(hash)?)
  }

  fn stop(&self) { }

  fn register_client(&self, client: Weak<EngineClient>) {
  }

  fn verify_local_seal(&self, header: &Header) -> Result<(), Error> { Ok(()) }

  fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
    super::total_difficulty_fork_choice(new, current)
  }
}
