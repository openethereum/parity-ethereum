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
use error::Error;
use header::Header;
use rlp::*;
use ethkey::{recover, public_to_address};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConsensusMessage {
	pub signature: H520,
	pub height: Height,
	pub round: Round,
	pub step: Step,
	pub block_hash: Option<BlockHash>
}


fn consensus_round(header: &Header) -> Result<Round, ::rlp::DecoderError> {
	UntrustedRlp::new(header.seal()[0].as_slice()).as_val()
}

impl ConsensusMessage {
	pub fn new_proposal(header: &Header) -> Result<Self, ::rlp::DecoderError> {
		Ok(ConsensusMessage {
			signature: try!(UntrustedRlp::new(header.seal()[1].as_slice()).as_val()),
			height: header.number() as Height,
			round: try!(consensus_round(header)),
			step: Step::Propose,
			block_hash: Some(header.bare_hash())
		})
	}


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

	pub fn verify(&self) -> Result<Address, Error> {
		let full_rlp = ::rlp::encode(self);
		let block_info = Rlp::new(&full_rlp).at(1);
		let public_key = try!(recover(&self.signature.into(), &block_info.as_raw().sha3()));
		Ok(public_to_address(&public_key))
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
			Step::Propose => 0,
			Step::Prevote => 1,
			Step::Precommit => 2,
			Step::Commit => 3,
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

pub fn message_info_rlp(height: Height, round: Round, step: Step, block_hash: Option<BlockHash>) -> Bytes {
	let mut s = RlpStream::new_list(4);
	s.append(&height).append(&round).append(&step).append(&block_hash.unwrap_or(H256::zero()));
	s.out()
}

pub fn message_info_rlp_from_header(header: &Header) -> Result<Bytes, ::rlp::DecoderError> {
	let round = try!(consensus_round(header));
	Ok(message_info_rlp(header.number() as Height, round, Step::Precommit, Some(header.bare_hash())))
}

pub fn message_full_rlp<F>(signer: F, height: Height, round: Round, step: Step, block_hash: Option<BlockHash>) -> Option<Bytes> where F: FnOnce(H256) -> Option<H520> {
	let vote_info = message_info_rlp(height, round, step, block_hash);
	signer(vote_info.sha3()).map(|ref signature| {
		let mut s = RlpStream::new_list(2);
		s.append(signature).append(&vote_info);
		s.out()
	})
}
