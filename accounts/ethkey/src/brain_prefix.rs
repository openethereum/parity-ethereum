// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

use super::Brain;
use parity_crypto::publickey::{Generator, KeyPair, Error};
use parity_wordlist as wordlist;

/// Tries to find brain-seed keypair with address starting with given prefix.
pub struct BrainPrefix {
	prefix: Vec<u8>,
	iterations: usize,
	no_of_words: usize,
	last_phrase: String,
}

impl BrainPrefix {
	pub fn new(prefix: Vec<u8>, iterations: usize, no_of_words: usize) -> Self {
		BrainPrefix {
			prefix,
			iterations,
			no_of_words,
			last_phrase: String::new(),
		}
	}

	pub fn phrase(&self) -> &str {
		&self.last_phrase
	}
}

impl Generator for BrainPrefix {
	type Error = Error;

	fn generate(&mut self) -> Result<KeyPair, Error> {
		for _ in 0..self.iterations {
			let phrase = wordlist::random_phrase(self.no_of_words);
			let keypair = Brain::new(phrase.clone()).generate().unwrap();
			if keypair.address().as_ref().starts_with(&self.prefix) {
				self.last_phrase = phrase;
				return Ok(keypair)
			}
		}

		Err(Error::Custom("Could not find keypair".into()))
	}
}

#[cfg(test)]
mod tests {
	use BrainPrefix;
	use parity_crypto::publickey::Generator;

	#[test]
	fn prefix_generator() {
		let prefix = vec![0x00u8];
		let keypair = BrainPrefix::new(prefix.clone(), usize::max_value(), 12).generate().unwrap();
		assert!(keypair.address().as_bytes().starts_with(&prefix));
	}
}
