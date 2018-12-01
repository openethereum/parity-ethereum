use std::sync::Weak;
use client::EngineClient;
use ethkey::{public_to_address, Signature};
use ethereum_types::{Address, H256};
use std::collections::HashMap;
use engines::clique::{SIGNER_SIG_LENGTH, SIGNER_VANITY_LENGTH, recover};
use error::Error;
use header::{Header, ExtendedHeader};
use super::super::signer::EngineSigner;
use parking_lot::RwLock;
use std::sync::Arc;
use account_provider::AccountProvider;
use ethkey::Password;

pub const NONCE_DROP_VOTE: &[u8; 8] = &[0x0; 8];
pub const NONCE_AUTH_VOTE: &[u8; 8] = &[0xf; 8];
pub const NULL_AUTHOR:     [u8; 20] = [0; 20];
pub const DIFF_INTURN:    &[u64; 4] = &[1, 0, 0, 0];
pub const DIFF_NOT_INTURN:    &[u64; 4] = &[2, 0, 0, 0];

pub enum SignerAuthorization {
  InTurn,
  OutOfTurn,
  Unauthorized
}

pub struct SignerSnapshot {
  pub signer: RwLock<EngineSigner>,
  pub bn: u64,
  pub epoch_length: u64,
  pub pending_state: SnapshotState,
  pub final_state: SnapshotState
}

#[derive(Clone)]
pub struct SnapshotState {
  pub votes: HashMap<Address, (bool, Address)>,
  pub signers: Vec<Address>,
  pub recents: Vec<Address>,
}

impl SignerSnapshot {
  fn extract_signers(&self, _header: &Header) -> Result<Vec<Address>, Error> {
    assert_eq!(_header.number() % self.epoch_length == 0, true, "header is not an epoch block");

    let min_extra_data_size = (SIGNER_VANITY_LENGTH as usize) + (SIGNER_SIG_LENGTH as usize);

    assert!(_header.extra_data().len() >= min_extra_data_size, "need minimum genesis extra data size {}.  found {}.", min_extra_data_size, _header.extra_data().len());

    // extract only the portion of extra_data which includes the signer list
    let signers_raw = &_header.extra_data()[(SIGNER_VANITY_LENGTH as usize).._header.extra_data().len()-(SIGNER_SIG_LENGTH as usize)];

    assert_eq!(signers_raw.len() % 20, 0, "bad signer list length {}", signers_raw.len());

    let num_signers = signers_raw.len() / 20;
    let mut signers_list: Vec<Address> = vec![];

    for i in 0..num_signers {
      let mut signer = Address::default();
      signer.copy_from_slice(&signers_raw[i*20..(i+1)*20]);
      signers_list.push(signer);
    }

    trace!(target: "engine", "extracted signers {:?}", &signers_list);
    Ok(signers_list)
  }

  pub fn get_signer(&self) -> Option<Address> {
      self.signer.read().address()
  }

  pub fn set_signer(&mut self, ap: Arc<AccountProvider>, address: Address, password: Password) {
      self.signer.write().set(ap, address, password);
  }

  // finalize the pending state
  pub fn commit(&mut self) {
    self.final_state = self.pending_state.clone();
    self.pending_state = SnapshotState {
      votes: HashMap::<Address, (bool, Address)>::new(),
      signers: self.final_state.signers.clone(),
      recents: self.final_state.recents.clone(),
    };
    self.bn += 1;
  }

  // reset the pending state to the previously finalized state
  pub fn rollback(&mut self) {
    self.pending_state = SnapshotState {
      votes: HashMap::<Address, (bool, Address)>::new(),
      signers: self.final_state.signers.clone(),
      recents: self.final_state.recents.clone(),
    }
  }

  pub fn new(epoch_length: u64) -> Self {
    return SignerSnapshot {
      pending_state: SnapshotState {
        votes: HashMap::<Address, (bool, Address)>::new(),
        signers: vec![],
        recents: vec![],
      },
      final_state: SnapshotState {
        votes: HashMap::<Address, (bool, Address)>::new(),
        signers: vec![],
        recents: vec![],
      },
      bn: 0,
      epoch_length: epoch_length,
      signer: Default::default(),
    }
  }

  pub fn get_own_authorization(&self) -> SignerAuthorization {
    self.get_signer_authorization(self.signer.read().address().expect("we should have an address"))
  }

  pub fn get_signer_authorization(&self, author: Address) -> SignerAuthorization {
    let signers = &self.pending_state.signers;
	if let Some(pos) = signers.iter().position(|x| self.signer.read().is_address(x)) {
	  if self.bn % signers.len() as u64 == pos as u64 {
        return SignerAuthorization::InTurn;
      } else {
        let limit = (signers.len() / 2) + 1;
        return SignerAuthorization::OutOfTurn;
      }
	}
      return SignerAuthorization::Unauthorized;
  }

  /*
  // apply a block that we sealed
  fn apply_own(&mut self, _header: &Header) -> Result<(), Error> {

  }

  fn apply_external(&mut self, _header: &Header) -> Result<(), Error> {

  }
  */

  // apply a header to the pending state
  pub fn apply(&mut self, _header: &Header) -> Result<(), Error> {
    if _header.number() == 0 {
        self.pending_state.signers = self.extract_signers(_header).expect("should be able to extractsigners from genesis block");
        return Ok(());
    }

    if _header.number() < self.bn {
      // TODO this might be called when impporting blocks from competing forks?
      return Err(From::from("tried to import block with header < chain tip"));
    }

    if &_header.author()[0..20] == &NULL_AUTHOR {
      return Ok(());
    }

    let creator = public_to_address(&recover(&_header).unwrap()).clone();

    match self.get_signer_authorization(creator) {
        SignerAuthorization::InTurn => {
            if &_header.difficulty().0 != DIFF_INTURN {
                return Err(From::from("difficulty must be set to DIFF_INTURN"));
            }
        },
        SignerAuthorization::OutOfTurn => {
            if &_header.difficulty().0 != DIFF_NOT_INTURN {
                return Err(From::from("difficulty must be set to DIFF_NOT_INTURN"));
            }
        },
        SignerAuthorization::Unauthorized => {
            return Err(From::from("unauthorized to sign at this time"));
        }
    }

    //TODO: votes that reach a majority consensus should have effects applied immediately to the signer list
    let nonce = _header.decode_seal::<Vec<&[u8]>>().unwrap()[1];
      let mut author = _header.author().clone();
      if nonce == NONCE_DROP_VOTE {
        self.pending_state.votes.insert(creator, (false, author));
        return Ok(());
      } else if nonce == NONCE_AUTH_VOTE {
        self.pending_state.votes.insert(creator, (true, author));
        return Ok(());
      } else {
        return Err(From::from("beneficiary specificed but nonce was not AUTH or DROP"));
      }

      return Ok(());
  }

  pub fn signer_address(&self) -> Option<Address> {
    self.signer.read().address().clone()
  }

  pub fn sign_data(&self, data: &H256) -> Option<Signature> {
    if let Ok(sig) = self.signer.read().sign(*data) {
      Some(sig)
    } else {
      None
    }
  }

  pub fn snapshot(&mut self, _header: &Header, _ancestry: &mut Iterator<Item=ExtendedHeader>) -> Result<Vec<Address>, Error> {
    if _header.number() % self.epoch_length == 0 {
      self.extract_signers(_header)
    } else {
      loop {
        if let Some(h) = _ancestry.next() {
          if h.header.number() % self.epoch_length == 0 {
            // verify signer signatures
            // extract signer list
            return self.extract_signers(&h.header);
          }
        } else {
          return Err(From::from("couldn't find checkpoint block in history"));
        }
      }
    }
  }
}
