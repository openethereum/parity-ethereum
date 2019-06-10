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

use rand::rngs::OsRng;
use super::{Generator, KeyPair, SECP256K1};

/// Randomly generates new keypair, instantiating the RNG each time.
pub struct Random;

impl Generator for Random {
	type Error = ::std::io::Error;

	fn generate(&mut self) -> Result<KeyPair, Self::Error> {
		let mut rng = OsRng::new()?;
		match rng.generate() {
			Ok(pair) => Ok(pair),
			Err(void) => match void {}, // LLVM unreachable
		}
	}
}

impl Generator for OsRng {
	type Error = ::Void;

	fn generate(&mut self) -> Result<KeyPair, Self::Error> {
		let (sec, publ) = SECP256K1.generate_keypair(self)
			.expect("context always created with full capabilities; qed");

		Ok(KeyPair::from_keypair(sec, publ))
	}
}
