use rand::os::OsRng;
use super::{Generator, KeyPair, Error, SECP256K1};

/// Randomly generates new keypair.
pub struct Random;

impl Generator for Random {
	fn generate(self) -> Result<KeyPair, Error> {
		let context = &SECP256K1;
		let mut rng = try!(OsRng::new());
		let (sec, publ) = try!(context.generate_keypair(&mut rng));

		Ok(KeyPair::from_keypair(sec, publ))
	}
}

