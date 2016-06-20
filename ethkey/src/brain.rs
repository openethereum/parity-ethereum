use keccak::Keccak256;
use super::{KeyPair, Error, Generator, Secret};

/// Simple brainwallet.
pub struct Brain(String);

impl Brain {
	pub fn new(s: String) -> Self {
		Brain(s)
	}
}

impl Generator for Brain {
	fn generate(self) -> Result<KeyPair, Error> {
		let seed = self.0;
		let mut secret = seed.bytes().collect::<Vec<u8>>().keccak256();

		let mut i = 0;
		loop {
			secret = secret.keccak256();
			
			match i > 16384 {
				false => i += 1,
				true => {
					let result = KeyPair::from_secret(Secret::from(secret.clone()));
					if result.is_ok() {
						return result
					}
				},
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use {Brain, Generator};

	#[test]
	fn test_brain() {
		let words = "this is sparta!".to_owned();
		let first_keypair = Brain(words.clone()).generate().unwrap();
		let second_keypair = Brain(words.clone()).generate().unwrap();
		assert_eq!(first_keypair.secret(), second_keypair.secret());
	}
}
