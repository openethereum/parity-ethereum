// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::convert::Infallible;
use parity_crypto::publickey::{KeyPair, Generator, Secret};
use parity_crypto::Keccak256;
use parity_wordlist;

/// Simple brainwallet.
pub struct Brain(String);

impl Brain {
	pub fn new(s: String) -> Self {
		Brain(s)
	}

	pub fn validate_phrase(phrase: &str, expected_words: usize) -> Result<(), ::WordlistError> {
		parity_wordlist::validate_phrase(phrase, expected_words)
	}
}

impl Generator for Brain {
	type Error = Infallible;

	fn generate(&mut self) -> Result<KeyPair, Self::Error> {
		let seed = self.0.clone();
		let mut secret = seed.into_bytes().keccak256();

		let mut i = 0;
		loop {
			secret = secret.keccak256();

			match i > 16384 {
				false => i += 1,
				true => {
					if let Ok(pair) = Secret::import_key(&secret)
						.and_then(KeyPair::from_secret)
					{
						if pair.address()[0] == 0 {
							trace!("Testing: {}, got: {:?}", self.0, pair.address());
							return Ok(pair)
						}
					}
				},
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use Brain;
	use parity_crypto::publickey::Generator;

	#[test]
	fn test_brain() {
		let words = "this is sparta!".to_owned();
		let first_keypair = Brain::new(words.clone()).generate().unwrap();
		let second_keypair = Brain::new(words.clone()).generate().unwrap();
		assert_eq!(first_keypair.secret(), second_keypair.secret());
	}
}
