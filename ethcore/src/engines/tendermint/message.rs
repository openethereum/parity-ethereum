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
	let round_rlp = header.seal().get(0).expect("seal passed basic verification; seal has 3 fields; qed");
	UntrustedRlp::new(round_rlp.as_slice()).as_val()
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
	fn number(&self) -> u8 {
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
			0u8 => Ok(Step::Propose),
			1 => Ok(Step::Prevote),
			2 => Ok(Step::Precommit),
			_ => Err(DecoderError::Custom("Invalid step.")),
		}
	}
}

impl Encodable for Step {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&self.number());
	}
}

/// (signature, height, round, step, block_hash)
impl Decodable for ConsensusMessage {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let rlp = decoder.as_rlp();
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
		let info = message_info_rlp(self.height, self.round, self.step, self.block_hash);
		s.begin_list(2)
			.append(&self.signature)
			.append_raw(&info, 1);
	}
}

pub fn message_info_rlp(height: Height, round: Round, step: Step, block_hash: Option<BlockHash>) -> Bytes {
	// TODO: figure out whats wrong with nested list encoding
	let mut s = RlpStream::new_list(5);
	s.append(&height).append(&round).append(&step).append(&block_hash.unwrap_or(H256::zero()));
	println!("{:?}, {:?}, {:?}, {:?}", &height, &round, &step, &block_hash);
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
		s.append(signature).append_raw(&vote_info, 1);
		s.out()
	})
}

#[cfg(test)]
mod tests {
	use util::*;
	use rlp::*;
	use super::super::Step;
	use super::*;
	use account_provider::AccountProvider;
	use header::Header;

	#[test]
	fn encode_decode() {
		let message = ConsensusMessage {
			signature: H520::default(),	
			height: 10,
			round: 123,
			step: Step::Precommit,
			block_hash: Some("1".sha3())
		};
		let raw_rlp = ::rlp::encode(&message).to_vec();
		let rlp = Rlp::new(&raw_rlp);
		assert_eq!(message, rlp.as_val());

		let message = ConsensusMessage {
			signature: H520::default(),	
			height: 1314,
			round: 0,
			step: Step::Prevote,
			block_hash: None
		};
		let raw_rlp = ::rlp::encode(&message);
		let rlp = Rlp::new(&raw_rlp);
		assert_eq!(message, rlp.as_val());
	}

	#[test]
	fn generate_and_verify() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr = tap.insert_account("0".sha3(), "0").unwrap();
		tap.unlock_account_permanently(addr, "0".into()).unwrap();

		let raw_rlp = message_full_rlp(
			|mh| tap.sign(addr, None, mh).ok().map(H520::from),
			123,
			2,
			Step::Precommit,
			Some(H256::default())
		).unwrap();

		let rlp = UntrustedRlp::new(&raw_rlp);
		let message: ConsensusMessage = rlp.as_val().unwrap();
		match message.verify() { Ok(a) if a == addr => {}, _ => panic!(), };
	}

	#[test]
	fn proposal_message() {
		let mut header = Header::default();
		let seal = vec![
			::rlp::encode(&0u8).to_vec(),
			::rlp::encode(&H520::default()).to_vec(),
			Vec::new()
		];
		header.set_seal(seal);
		let message = ConsensusMessage::new_proposal(&header).unwrap();
		assert_eq!(
			message,
			ConsensusMessage {
				signature: Default::default(),
				height: 0,
				round: 0,
				step: Step::Propose,
				block_hash: Some(header.bare_hash())
			}
		);
	}

	#[test]
	fn message_info_from_header() {
		let mut header = Header::default();
		let seal = vec![
			::rlp::encode(&0u8).to_vec(),
			::rlp::encode(&H520::default()).to_vec(),
			Vec::new()
		];
		header.set_seal(seal);
		assert_eq!(message_info_rlp_from_header(&header).unwrap().to_vec(), vec![228, 128, 128, 2, 160, 39, 191, 179, 126, 80, 124, 233, 13, 161, 65, 48, 114, 4, 177, 198, 186, 36, 25, 67, 128, 97, 53, 144, 172, 80, 202, 75, 29, 113, 152, 255, 101]);
	}
}
