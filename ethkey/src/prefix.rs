// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity. If not, see <http://www.gnu.org/licenses/>.

use super::{Random, Generator, KeyPair, Error};

/// Tries to find keypair with address starting with given prefix.
pub struct Prefix {
	prefix: Vec<u8>,
	iterations: usize,
}

impl Prefix {
	pub fn new(prefix: Vec<u8>, iterations: usize) -> Self {
		Prefix {
			prefix: prefix,
			iterations: iterations,
		}
	}
}

impl Generator for Prefix {
	type Error = Error;

	fn generate(&mut self) -> Result<KeyPair, Error> {
		for _ in 0..self.iterations {
			let keypair = Random.generate()?;
			if keypair.address().starts_with(&self.prefix) {
				return Ok(keypair)
			}
		}

		Err(Error::Custom("Could not find keypair".into()))
	}
}

#[cfg(test)]
mod tests {
	use {Generator, Prefix};

	#[test]
	fn prefix_generator() {
		let prefix = vec![0xffu8];
		let keypair = Prefix::new(prefix.clone(), usize::max_value()).generate().unwrap();
		assert!(keypair.address().starts_with(&prefix));
	}
}
