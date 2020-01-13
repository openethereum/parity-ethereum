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

//! EVM action types.

use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};

/// The type of the instruction.
#[derive(Debug, PartialEq, Clone)]
pub enum ActionType {
	/// CREATE.
	Create,
	/// CALL.
	Call,
	/// CALLCODE.
	CallCode,
	/// DELEGATECALL.
	DelegateCall,
	/// STATICCALL.
	StaticCall,
	/// CREATE2.
	Create2
}

impl Encodable for ActionType {
	fn rlp_append(&self, s: &mut RlpStream) {
		let v = match *self {
			ActionType::Create => 0u32,
			ActionType::Call => 1,
			ActionType::CallCode => 2,
			ActionType::DelegateCall => 3,
			ActionType::StaticCall => 4,
			ActionType::Create2 => 5,
		};
		Encodable::rlp_append(&v, s);
	}
}

impl Decodable for ActionType {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.as_val().and_then(|v| Ok(match v {
			0u32 => ActionType::Create,
			1 => ActionType::Call,
			2 => ActionType::CallCode,
			3 => ActionType::DelegateCall,
			4 => ActionType::StaticCall,
			5 => ActionType::Create2,
			_ => return Err(DecoderError::Custom("Invalid value of ActionType item")),
		}))
	}
}

#[cfg(test)]
mod tests {
	use rlp::*;
	use super::ActionType;

	#[test]
	fn encode_call_type() {
		let ct = ActionType::Call;

		let mut s = RlpStream::new_list(2);
		s.append(&ct);
		assert!(!s.is_finished(), "List shouldn't finished yet");
		s.append(&ct);
		assert!(s.is_finished(), "List should be finished now");
		s.out();
	}

	#[test]
	fn should_encode_and_decode_call_type() {
		let original = ActionType::Call;
		let encoded = encode(&original);
		let decoded = decode(&encoded).expect("failure decoding ActionType");
		assert_eq!(original, decoded);
	}
}
