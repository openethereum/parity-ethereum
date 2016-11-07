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

//! Tendermint block seal.

use util::*;
use header::Header;
use account_provider::AccountProvider;
use rlp::{View, DecoderError, Decodable, Decoder, Encodable, RlpStream, Stream};
use basic_types::Seal;
use super::BlockHash;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Vote {
	block_hash: BlockHash,
	signature: H520
}

fn block_hash(header: &Header) -> H256 {
	header.rlp(Seal::WithSome(1)).sha3()
}

impl Vote {
	fn new(block_hash: BlockHash, signature: H520) -> Vote {
		Vote { block_hash: block_hash, signature: signature }
	}

	/// Try to use the author address to create a vote.
	pub fn propose(header: &Header, accounts: &AccountProvider) -> Option<Vote> {
		Self::validate(header, accounts, *header.author())
	}
	
	/// Use any unlocked validator account to create a vote.
	pub fn validate(header: &Header, accounts: &AccountProvider, validator: Address) -> Option<Vote> {
		let message = block_hash(&header);
		accounts.sign(validator, None, message)
			.ok()
			.map(Into::into)
			.map(|sig| Self::new(message, sig))
	}
}

impl Decodable for Vote {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let rlp = decoder.as_rlp();
		if decoder.as_raw().len() != try!(rlp.payload_info()).total() {
			return Err(DecoderError::RlpIsTooBig);
		}
		Ok(Self::new(try!(rlp.val_at(0)), try!(rlp.val_at(1))))
    }
}

impl Encodable for Vote {
	fn rlp_append(&self, s: &mut RlpStream) {
		let Vote { ref block_hash, ref signature } = *self;
		s.append(block_hash);
		s.append(signature);
	}
}
