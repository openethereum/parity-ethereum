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

//! EVM call types.

use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};

/// The type of the call-like instruction.
#[derive(Debug, PartialEq, Clone)]
pub enum CallType {
	/// Not a CALL.
	None,
	/// CALL.
	Call,
	/// CALLCODE.
	CallCode,
	/// DELEGATECALL.
	DelegateCall,
	/// STATICCALL
	StaticCall,
}

impl Encodable for CallType {
	fn rlp_append(&self, s: &mut RlpStream) {
		let v = match *self {
			CallType::None => 0u32,
			CallType::Call => 1,
			CallType::CallCode => 2,
			CallType::DelegateCall => 3,
			CallType::StaticCall => 4,
		};
		Encodable::rlp_append(&v, s);
	}
}

impl Decodable for CallType {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.as_val().and_then(|v| Ok(match v {
			0u32 => CallType::None,
			1 => CallType::Call,
			2 => CallType::CallCode,
			3 => CallType::DelegateCall,
			4 => CallType::StaticCall,
			_ => return Err(DecoderError::Custom("Invalid value of CallType item")),
		}))
	}
}

#[cfg(test)]
mod tests {
	use rlp::*;
	use super::CallType;

	#[test]
	fn encode_call_type() {
		let ct = CallType::Call;

		let mut s = RlpStream::new_list(2);
		s.append(&ct);
		assert!(!s.is_finished(), "List shouldn't finished yet");
		s.append(&ct);
		assert!(s.is_finished(), "List should be finished now");
		s.out();
	}

	#[test]
	fn should_encode_and_decode_call_type() {
		let original = CallType::Call;
		let encoded = encode(&original);
		let decoded = decode(&encoded).expect("failure decoding CallType");
		assert_eq!(original, decoded);
	}
}
