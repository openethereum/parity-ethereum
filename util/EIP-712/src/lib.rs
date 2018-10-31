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
//!
//! # Specification
//!
//! `encode(domainSeparator : 𝔹²⁵⁶, message : 𝕊) = "\x19\x01" ‖ domainSeparator ‖ hashStruct(message)`
//! - data adheres to 𝕊, a structure defined in the rigorous eip-712
//! - `\x01` is needed to comply with EIP-191
//! - `domainSeparator` and `hashStruct` are defined below
//!
//! ## A) domainSeparator
//!
//! `domainSeparator = hashStruct(eip712Domain)`
//! <br/>
//! <br/>
//! Struct named `EIP712Domain` with the following fields
//!
//! - `name: String`
//! - `version: String`
//! - `chain_id: U256`,
//! - `verifying_contract: H160`
//! - `salt: Option<H256>`
//!
//! ## C) hashStruct
//!
//! `hashStruct(s : 𝕊) = keccak256(typeHash ‖ encodeData(s))`
//! <br/>
//! `typeHash = keccak256(encodeType(typeOf(s)))`
//!
//! ### i) encodeType
//!
//! - `name ‖ "(" ‖ member₁ ‖ "," ‖ member₂ ‖ "," ‖ … ‖ memberₙ ")"`
//! - each member is written as `type ‖ " " ‖ name`
//! - encodings cascade down and are sorted by name
//!
//! ### ii) encodeData
//!
//! - `enc(value₁) ‖ enc(value₂) ‖ … ‖ enc(valueₙ)`
//! - each encoded member is 32-byte long
//!
//!     #### a) atomic
//!
//!     - `boolean`     => `U256`
//!     - `address`     => `H160`
//!     - `uint`        => sign-extended `U256` in big endian order
//!     - `bytes1:31`   => `H@256`
//!
//!     #### b) dynamic
//!
//!     - `bytes`       => `keccak256(bytes)`
//!     - `string`      => `keccak256(string)`
//!
//!     #### c) referenced
//!
//!     - `array`       => `keccak256(encodeData(array))`
//!     - `struct`      => `rec(keccak256(hashStruct(struct)))`
//!
//! ## D) Example
//! ### Query
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "eth_signTypedData",
//!   "params": [
//!     "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826",
//!     {
//!       "types": {
//!         "EIP712Domain": [
//!           {
//!             "name": "name",
//!             "type": "string"
//!           },
//!           {
//!             "name": "version",
//!             "type": "string"
//!           },
//!           {
//!             "name": "chainId",
//!             "type": "uint256"
//!           },
//!           {
//!             "name": "verifyingContract",
//!             "type": "address"
//!           }
//!         ],
//!         "Person": [
//!           {
//!             "name": "name",
//!             "type": "string"
//!           },
//!           {
//!             "name": "wallet",
//!             "type": "address"
//!           }
//!         ],
//!         "Mail": [
//!           {
//!             "name": "from",
//!             "type": "Person"
//!           },
//!           {
//!             "name": "to",
//!             "type": "Person"
//!           },
//!           {
//!             "name": "contents",
//!             "type": "string"
//!           }
//!         ]
//!       },
//!       "primaryType": "Mail",
//!       "domain": {
//!         "name": "Ether Mail",
//!         "version": "1",
//!         "chainId": 1,
//!         "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
//!       },
//!       "message": {
//!         "from": {
//!           "name": "Cow",
//!           "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
//!         },
//!         "to": {
//!           "name": "Bob",
//!           "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
//!         },
//!         "contents": "Hello, Bob!"
//!       }
//!     }
//!   ],
//!   "id": 1
//! }
//! ```
//
//! ### Response
//! ```json
//! {
//!   "id":1,
//!   "jsonrpc": "2.0",
//!   "result": "0x4355c47d63924e8a72e509b65029052eb6c299d53a04e167c5775fd466751c9d07299936d304c153f6443dfa05f40ff007d72911b6f72307f996231605b915621c"
//! }
//! ```

#![warn(missing_docs, unused_extern_crates)]

extern crate serde_json;
extern crate ethabi;
extern crate ethereum_types;
extern crate keccak_hash;
extern crate itertools;
extern crate failure;
extern crate indexmap;
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
pub use eip712::EIP712;
