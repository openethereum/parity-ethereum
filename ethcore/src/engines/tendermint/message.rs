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

//! Tendermint message handling.

use super::{Height, Round, BlockHash};
use rlp::{View, DecoderError, Decodable, Decoder, Encodable, RlpStream, Stream};

pub enum ConsensusMessage {
	Prevote(Height, Round, BlockHash),
	Precommit(Height, Round, BlockHash),
	Commit(Height, BlockHash),
}

/// (height, step, ...)
impl Decodable for ConsensusMessage {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		// Handle according to step.
		let rlp = decoder.as_rlp();
		if decoder.as_raw().len() != try!(rlp.payload_info()).total() {
			return Err(DecoderError::RlpIsTooBig);
		}
		let height = try!(rlp.val_at(0));
		Ok(match try!(rlp.val_at(1)) {
			0u8 => ConsensusMessage::Prevote(
				height,
				try!(rlp.val_at(2)),
				try!(rlp.val_at(3))
			),
			1 => ConsensusMessage::Precommit(
				height,
				try!(rlp.val_at(2)),
				try!(rlp.val_at(3))
			),
			2 => ConsensusMessage::Commit(
				height,
				try!(rlp.val_at(2))),
			_ => return Err(DecoderError::Custom("Unknown step.")),
		})
    }
}

impl Encodable for ConsensusMessage {
	fn rlp_append(&self, s: &mut RlpStream) {
		match *self {
			ConsensusMessage::Prevote(h, r, hash) => {
				s.begin_list(4);
				s.append(&h);
				s.append(&0u8);
				s.append(&r);
				s.append(&hash);
			},
			ConsensusMessage::Precommit(h, r, hash) => {
				s.begin_list(4);
				s.append(&h);
				s.append(&1u8);
				s.append(&r);
				s.append(&hash);
			},
			ConsensusMessage::Commit(h, hash) => {
				s.begin_list(3);
				s.append(&h);
				s.append(&2u8);
				s.append(&hash);
			},
		}
	}
}
