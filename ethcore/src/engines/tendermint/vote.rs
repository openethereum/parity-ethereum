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

use common::{H256, Address, H520, Header};
use util::Hashable;
use account_provider::AccountProvider;
use rlp::{View, DecoderError, Decodable, Decoder, Encodable, RlpStream, Stream};
use basic_types::Seal;

#[derive(Debug)]
pub struct Vote {
	signature: H520
}

fn message(header: &Header) -> H256 {
	header.rlp(Seal::WithSome(1)).sha3()
}

impl Vote {
	fn new(signature: H520) -> Vote { Vote { signature: signature }}

	/// Try to use the author address to create a vote.
	pub fn propose(header: &Header, accounts: &AccountProvider) -> Option<Vote> {
		accounts.sign(*header.author(), message(&header)).ok().map(Into::into).map(Self::new)
	}
	
	/// Use any unlocked validator account to create a vote.
	pub fn validate(header: &Header, accounts: &AccountProvider, validator: Address) -> Option<Vote> {
		accounts.sign(validator, message(&header)).ok().map(Into::into).map(Self::new)
	}
}

impl Decodable for Vote {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let rlp = decoder.as_rlp();
		if decoder.as_raw().len() != try!(rlp.payload_info()).total() {
			return Err(DecoderError::RlpIsTooBig);
		}
		rlp.as_val().map(Self::new)
    }
}

impl Encodable for Vote {
	fn rlp_append(&self, s: &mut RlpStream) {
		let Vote { ref signature } = *self;
		s.append(signature);
	}
}
