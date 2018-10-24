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

//! EIP-712 encoding utilities
#![warn(missing_docs, unused_extern_crates)]

extern crate serde_json;
extern crate ethabi;
extern crate ethereum_types;
extern crate keccak_hash;
extern crate itertools;
extern crate failure;
extern crate linked_hash_set;
extern crate lunarity_lexer;
extern crate toolshed;
extern crate regex;
extern crate validator;
#[macro_use]
extern crate validator_derive;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate rustc_hex;

mod eip712;
mod error;
mod parser;
mod encode;
/// the EIP-712 encoding function
pub use encode::hash_structured_data;
/// encoding Error types
pub use error::{ErrorKind, Error};
/// EIP712 struct
pub use eip712::{EIP712};
