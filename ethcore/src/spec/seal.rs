// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Spec seal.

use util::rlp::*;
use util::hash::{H64, H256};
use ethjson;

/// Classic ethereum seal.
pub struct Ethereum {
	/// Seal nonce.
	pub nonce: H64,
	/// Seal mix hash.
	pub mix_hash: H256,
}

impl Into<Generic> for Ethereum {
	fn into(self) -> Generic {
		let mut s = RlpStream::new();
		s.append(&self.mix_hash);
		s.append(&self.nonce);
		Generic {
			fields: 2,
			rlp: s.out()
		}
	}
}

/// Generic seal.
pub struct Generic {
	/// Number of seal fields.
	pub fields: usize,
	/// Seal rlp.
	pub rlp: Vec<u8>,
}

/// Genesis seal type.
pub enum Seal {
	/// Classic ethereum seal.
	Ethereum(Ethereum),
	/// Generic seal.
	Generic(Generic),
}

impl From<ethjson::spec::Seal> for Seal {
	fn from(s: ethjson::spec::Seal) -> Self {
		match s {
			ethjson::spec::Seal::Ethereum(eth) => Seal::Ethereum(Ethereum {
				nonce: eth.nonce.into(),
				mix_hash: eth.mix_hash.into()
			}),
			ethjson::spec::Seal::Generic(g) => Seal::Generic(Generic {
				fields: g.fields,
				rlp: g.rlp.into()
			})
		}
	}
}

impl Into<Generic> for Seal {
	fn into(self) -> Generic {
		match self {
			Seal::Generic(generic) => generic,
			Seal::Ethereum(eth) => eth.into()
		}
	}
}
