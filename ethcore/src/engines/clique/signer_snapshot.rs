use std::sync::Weak;
use client::EngineClient;
use ethkey::{public_to_address};
use ethereum_types::{Address};
use std::collections::HashMap;
use engines::clique::{SIGNER_SIG_LENGTH, SIGNER_VANITY_LENGTH, recover};
use error::Error;
use header::{Header, ExtendedHeader};

pub const NONCE_DROP_VOTE: &[u8; 8] = &[0x0; 8];
pub const NONCE_AUTH_VOTE: &[u8; 8] = &[0xf; 8];
pub const NULL_AUTHOR:     [u8; 20] = [0; 20];

pub struct SignerSnapshot {
  pub bn: u64,
  pub signers: Vec<Address>,
  pub epoch_length: u64,
  pub votes: HashMap<Address, (bool, Address)>
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

    Ok(signers_list)
  }

  pub fn apply(&mut self, _header: &Header) -> Result<(), Error> {
    if _header.number() == 0 {
      self.signers = self.extract_signers(_header).expect("should be able to extract signer list from genesis block");
      trace!(target: "engine", "extracted {} signers", self.signers.len());
      return Ok(());
    } else if _header.number() % self.epoch_length == 0 {
      // TODO: assert that no voting occurs during an epoch change 
      return Ok(());
    }

    if &_header.author()[0..20] == &NULL_AUTHOR {
      return Ok(());
    }

    trace!(target: "engine", "header author {}", _header.author());
    trace!(target: "engine", "attempting to extract creator address");
    let mut creator = public_to_address(&recover(&_header).unwrap()).clone();

    //TODO: votes that reach a majority consensus should have effects applied immediately to the signer list
    let nonce = _header.decode_seal::<Vec<&[u8]>>().unwrap()[1];
      let mut author = _header.author().clone();
      if nonce == NONCE_DROP_VOTE {
        self.votes.insert(creator, (false, author));
        return Ok(());
      } else if nonce == NONCE_AUTH_VOTE {
        self.votes.insert(creator, (true, author));
        return Ok(());
      } else {
        return Err(From::from("beneficiary specificed but nonce was not AUTH or DROP"));
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
