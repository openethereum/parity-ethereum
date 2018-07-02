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

//! General error types for use in ethcore.

#![allow(missing_docs)]
#![allow(unknown_lints)]

#[macro_use]
extern crate error_chain;

extern crate ethereum_types;
extern crate rlp;
extern crate rustc_hex;

use std::fmt;
use rustc_hex::FromHexError;
use rlp::DecoderError;
use ethereum_types::H256;

#[derive(Debug)]
/// Error in database subsystem.
pub enum BaseDataError {
	/// An entry was removed more times than inserted.
	NegativelyReferencedHash(H256),
	/// A committed value was inserted more than once.
	AlreadyExists(H256),
}

impl fmt::Display for BaseDataError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			BaseDataError::NegativelyReferencedHash(hash) =>
				write!(f, "Entry {} removed from database more times than it was added.", hash),
			BaseDataError::AlreadyExists(hash) =>
				write!(f, "Committed key already exists in database: {}", hash),
		}
	}
}

impl std::error::Error for BaseDataError {
	fn description(&self) -> &str {
		"Error in database subsystem"
	}
}

error_chain! {
	types {
		UtilError, ErrorKind, ResultExt, Result;
	}

	foreign_links {
		Io(::std::io::Error);
		FromHex(FromHexError);
		Decoder(DecoderError);
		BaseData(BaseDataError);
	}
}
