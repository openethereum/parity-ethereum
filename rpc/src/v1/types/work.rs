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

use super::{H256, U256};

use serde::{Serialize, Serializer};

/// The result of an `eth_getWork` call: it differs based on an option
/// whether to send the block number.
#[derive(Debug, PartialEq, Eq)]
pub struct Work {
	/// The proof-of-work hash.
	pub pow_hash: H256,
	/// The seed hash.
	pub seed_hash: H256,
	/// The target.
	pub target: H256,
	/// The block number: this isn't always stored.
	pub number: Option<u64>,
}

impl Serialize for Work {
	fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
		match self.number.as_ref() {
			Some(num) => (&self.pow_hash, &self.seed_hash, &self.target, U256::from(*num)).serialize(s),
			None => (&self.pow_hash, &self.seed_hash, &self.target).serialize(s),
		}
	}
}

/// The result of an `eth_submitWorkDetail` call: it differs based on an option
/// whether to send the error message.
#[derive(Debug, PartialEq, Eq)]
pub struct SubmitDetailResult {
	/// Whether the submission was successful.
	pub success: bool,
	/// The error message.
	pub error_msg: Option<String>,
	/// The block hash.
	pub block_hash: Option<H256>,
}

impl Serialize for SubmitDetailResult {
	fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
		(&self.success, &self.error_msg, &self.block_hash).serialize(s)
	}
}
