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

use util::*;
use super::{Height, Round, BlockHash, Step};
use rlp::{View, DecoderError, Decodable, Decoder, Encodable, RlpStream, Stream};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConsensusMessage {
	pub signature: H520,
	pub height: Height,
	pub round: Round,
	pub step: Step,
	pub block_hash: Option<BlockHash>
}

impl ConsensusMessage {
	pub fn is_height(&self, height: Height) -> bool {
		self.height == height
	}

	pub fn is_round(&self, height: Height, round: Round) -> bool {
		self.height == height && self.round == round
	}

	pub fn is_step(&self, height: Height, round: Round, step: Step) -> bool {
		self.height == height && self.round == round && self.step == step
	}

	pub fn is_aligned(&self, height: Height, round: Round, block_hash: Option<H256>) -> bool {
		self.height == height && self.round == round && self.block_hash == block_hash
	}
}

impl PartialOrd for ConsensusMessage {
	fn partial_cmp(&self, m: &ConsensusMessage) -> Option<Ordering> {
		Some(self.cmp(m))
	}
}

impl Step {
	fn number(&self) -> i8 {
		match *self {
			Step::Propose => -1,
			Step::Prevote => 0,
			Step::Precommit => 1,
			Step::Commit => 2,
		}
	}
}

impl Ord for ConsensusMessage {
	fn cmp(&self, m: &ConsensusMessage) -> Ordering {
		if self.height != m.height {
			self.height.cmp(&m.height)
		} else if self.round != m.round {
			self.round.cmp(&m.round)
		} else if self.step != m.step {
			self.step.number().cmp(&m.step.number())
		} else {
			self.block_hash.cmp(&m.block_hash)
		}
	}
}

impl Decodable for Step {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		match try!(decoder.as_rlp().as_val()) {
			0u8 => Ok(Step::Prevote),
			1 => Ok(Step::Precommit),
			_ => Err(DecoderError::Custom("Invalid step.")),
		}
	}
}


impl Encodable for Step {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&(self.number() as u8));
	}
}

/// (signature, height, round, step, block_hash)
impl Decodable for ConsensusMessage {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let rlp = decoder.as_rlp();
		if decoder.as_raw().len() != try!(rlp.payload_info()).total() {
			return Err(DecoderError::RlpIsTooBig);
		}
		let m = try!(rlp.at(1));
		let block_message: H256 = try!(m.val_at(3));
		Ok(ConsensusMessage {
			signature: try!(rlp.val_at(0)),
			height: try!(m.val_at(0)),
			round: try!(m.val_at(1)),
			step: try!(m.val_at(2)),
			block_hash: match block_message.is_zero() {
				true => None,
				false => Some(block_message),
			}
		})
    }
}

impl Encodable for ConsensusMessage {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.signature);
		s.begin_list(4);
		s.append(&self.height);
		s.append(&self.round);
		s.append(&self.step);
		s.append(&self.block_hash.unwrap_or(H256::zero()));
	}
}
