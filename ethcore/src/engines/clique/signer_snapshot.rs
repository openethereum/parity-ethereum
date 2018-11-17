use header::Header;
use std::sync::Weak;
use client::EngineClient;
use ethkey::{public_to_address};

const NONCE_DROP_VOTE: &[u8; 8] = &[0x0; 8];
const NONCE_AUTH_VOTE: &[u8; 8] = &[0xf; 8];
const NULL_AUTHOR:     &[u8; 20] = &[0; 20];

pub struct AuthorizationSnapshot {
  bn: u64,
  signers: Vec<Address>,
  epoch_length: u64,
  votes: HashMap<Address, (bool, Address)>
}

impl AuthorizationSnapshot {
  pub fn new() -> Self {
    AuthorizationSnapshot {

    }
  }

  fn extract_signers(&self, _header: &Header) -> Result<Vec<u8> {
    assert_eq!(_header.number() % self.epoch, true, "header is not an epoch block");

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

  pub fn apply(mut &self, _header: &Header) -> Result<(), Error> {
    if _header.number() % self.epoch_length == 0 {
      // TODO: assert that no voting occurs during an epoch change 
      return;
    }

    if _header.author() == NULL_AUTHOR {
      return;
    }

    let creator = public_to_address(&recover(&_header).unwrap());

    //TODO: votes that reach a majority consensus should have effects applied immediately to the signer list
    let nonce = _header.decode_seal::<Vec<&[u8]>>().unwrap()[1];
      if nonce == NONCE_DROP_VOTE {
        self.votes.insert(creator, (false, _header.author());
      } else if nonce == NONCE_AUTH_VOTE {
        self.votes.insert(creator, (true, _header.author());
      } else {
        return Err(From::from("beneficiary specificed but nonce was not AUTH or DROP");
      }
  }

  pub fn snapshot(mut &self, _header: &Header, _ancestry: &mut Iterator<Item=ExtendedHeader>) -> Result<Vec<Address>> {
    if _header.number() % self.epoch_length == 0 {
      self.extract_signers(_header)
    } else {
      loop {
        if let Ok(h) = _ancestry.next() {
          if h.number() == 0 {
            return extract_signers(&h);
          } else if h.number() % self.epoch_length == 0 {
            // verify signer signatures
            // extract signer list
            return extract_signers(&h);
          }
        } else {
          Err(())
        }
      }
    }
  }
}
