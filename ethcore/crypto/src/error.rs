// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use rcrypto;
use ring;

quick_error! {
	#[derive(Debug)]
	pub enum Error {
		Scrypt(e: ScryptError) {
			cause(e)
			from()
		}
		Symm(e: SymmError) {
			cause(e)
			from()
		}
	}
}

quick_error! {
	#[derive(Debug)]
	pub enum ScryptError {
		// log(N) < r / 16
		InvalidN {
			display("Invalid N argument of the scrypt encryption")
		}
		// p <= (2^31-1 * 32)/(128 * r)
		InvalidP {
			display("Invalid p argument of the scrypt encryption")
		}
	}
}

quick_error! {
	#[derive(Debug)]
	pub enum SymmError wraps PrivSymmErr {
		RustCrypto(e: rcrypto::symmetriccipher::SymmetricCipherError) {
			display("symmetric crypto error")
			from()
		}
		Ring(e: ring::error::Unspecified) {
			display("symmetric crypto error")
			cause(e)
			from()
		}
		Offset(x: usize) {
			display("offset {} greater than slice length", x)
		}
	}
}

impl SymmError {
	pub(crate) fn offset_error(x: usize) -> SymmError {
		SymmError(PrivSymmErr::Offset(x))
	}
}

impl From<ring::error::Unspecified> for SymmError {
	fn from(e: ring::error::Unspecified) -> SymmError {
		SymmError(PrivSymmErr::Ring(e))
	}
}

impl From<rcrypto::symmetriccipher::SymmetricCipherError> for SymmError {
	fn from(e: rcrypto::symmetriccipher::SymmetricCipherError) -> SymmError {
		SymmError(PrivSymmErr::RustCrypto(e))
	}
}
