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

//! Basic account type -- the decoded RLP from the state trie.

use ethereum_types::{U256, H256};

/// Basic account type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicAccount {
	/// Nonce of the account.
	pub nonce: U256,
	/// Balance of the account.
	pub balance: U256,
	/// Storage root of the account.
	pub storage_root: H256,
	/// Code hash of the account.
	pub code_hash: H256,
	/// Code version of the account.
	pub code_version: U256,
}

impl rlp::Encodable for BasicAccount {
	fn rlp_append(&self, stream: &mut rlp::RlpStream) {
		let use_short_version = self.code_version == U256::zero();

		match use_short_version {
			true => { stream.begin_list(4); }
			false => { stream.begin_list(5); }
		}

		stream.append(&self.nonce);
		stream.append(&self.balance);
		stream.append(&self.storage_root);
		stream.append(&self.code_hash);

		if !use_short_version {
			stream.append(&self.code_version);
		}
	}
}

impl rlp::Decodable for BasicAccount {
	fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
		let use_short_version = match rlp.item_count()? {
			4 => true,
			5 => false,
			_ => return Err(rlp::DecoderError::RlpIncorrectListLen),
		};

		Ok(BasicAccount {
			nonce: rlp.val_at(0)?,
			balance: rlp.val_at(1)?,
			storage_root: rlp.val_at(2)?,
			code_hash: rlp.val_at(3)?,
			code_version: if use_short_version {
				U256::zero()
			} else {
				rlp.val_at(4)?
			},
		})
	}
}
