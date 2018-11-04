use header::Header;
use std::sync::Weak;
use client::EngineClient;

pub struct AuthorizationSnapshot {
  //bn: u64,
  //recents: []Address,
  //signers: []Address,
}

impl AuthorizationSnapshot {
  pub fn new() -> Self {
    AuthorizationSnapshot {

    }
  }

  pub fn apply(self, header: Header) -> Result<AuthorizationSnapshot, String> {
/*
    let snap = self.copy();

    if header.number() != self.block_number + 1 {
      Err(String::from("can only import direct parent of snapshot"))
    }
*/

/*
    if header.number() % EPOCH_LENGTH == 0 {
      snap.recents = [0; Address]
      snap.signers = [0; Address]
    }
*/

    Err(String::from("").into())
    //ecrecover(header, snap.signers)?.expect("header signer should be authorized");
  }

  pub fn register_client(self, client: Weak<EngineClient>) {
    //self.client = client
  }

}
