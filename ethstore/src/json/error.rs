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

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
	UnsupportedCipher,
	InvalidCipherParams,
	UnsupportedKdf,
	InvalidUuid,
	UnsupportedVersion,
	InvalidCiphertext,
	InvalidH256,
	InvalidPrf,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::InvalidUuid => write!(f, "Invalid Uuid"),
			Error::UnsupportedVersion => write!(f, "Unsupported version"),
			Error::UnsupportedKdf => write!(f, "Unsupported kdf"),
			Error::InvalidCiphertext => write!(f, "Invalid ciphertext"),
			Error::UnsupportedCipher => write!(f, "Unsupported cipher"),
			Error::InvalidCipherParams => write!(f, "Invalid cipher params"),
			Error::InvalidH256 => write!(f, "Invalid hash"),
			Error::InvalidPrf => write!(f, "Invalid prf"),
		}
	}
}

impl Into<String> for Error {
	fn into(self) -> String {
		format!("{}", self)
	}
}
