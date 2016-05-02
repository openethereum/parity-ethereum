use std::ptr;
use spec::ParamType;
use token::Token;
use error::Error;

enum Mediate {
	Raw(Vec<[u8; 32]>),
	Prefixed(Vec<[u8; 32]>),
	Nested(Vec<Mediate>),
}

impl Mediate {
	fn init_len(&self) -> u32 {
		let len = match *self {
			Mediate::Raw(ref raw) => raw.len(),
			Mediate::Prefixed(_) => 1,
			Mediate::Nested(_) => 1,
		};

		len as u32 * 32
	}

	fn closing_len(&self) -> u32 {
		match *self {
			Mediate::Raw(_) => 0,
			Mediate::Prefixed(ref pre) => pre.len() as u32 * 32,
			Mediate::Nested(ref nes) => nes.iter().fold(0, |acc, m| acc + m.init_len() + m.closing_len()),
		}
	}

	fn offset_for(mediates: &[Mediate], position: usize) -> u32 {
		assert!(position < mediates.len());

		let init_len = mediates.iter().fold(0, |acc, m| acc + m.init_len());
		mediates[0..position].iter().fold(init_len, |acc, m| acc + m.closing_len())
	}

	fn init(&self, suffix_offset: u32) -> Vec<[u8; 32]> {
		match *self {
			Mediate::Raw(ref raw) => raw.clone(),
			Mediate::Prefixed(_) | Mediate::Nested(_) => {
				unimplemented!();
			}
		}
	}

	fn closing(&self, offset: u32) -> Vec<[u8; 32]> {
		match *self {
			Mediate::Raw(ref raw) => vec![],
			Mediate::Prefixed(ref pre) => pre.clone(),
			Mediate::Nested(ref nes) => {
				unimplemented!();
				let inits = nes.iter()
					.enumerate()
					.flat_map(|(i, m)| m.init(offset + Mediate::offset_for(nes, i)));

				let closings = nes.iter()
					.enumerate()
					.flat_map(|(i, m)| m.closing(offset + Mediate::offset_for(nes, i)));

				inits.chain(closings).collect()
			},
		}
	}
}

pub struct Encoder;

impl Encoder {
	pub fn encode(tokens: Vec<Token>) -> Vec<u8> {
		let mediates: Vec<Mediate> = tokens.into_iter()
			.map(Self::encode_token)
			.collect();

		let inits = mediates.iter()
			.enumerate()
			.flat_map(|(i, m)| m.init(Mediate::offset_for(&mediates, i)));

		let closings = mediates.iter()
			.enumerate()
			.flat_map(|(i, m)| m.closing(Mediate::offset_for(&mediates, i)));

		inits.chain(closings)
			.flat_map(|item| item.to_vec())
			.collect()
	}

	fn encode_token(token: Token) -> Mediate {
		match token {
			Token::Address(address) => {
				let mut padded = [0u8; 32];
				unsafe {
					ptr::copy(address.as_ptr(), padded.as_mut_ptr().offset(12), 20);
				}
				Mediate::Raw(vec![padded])
			},
			_ => {
				unimplemented!();
			}
		}
	}
}

pub struct Decoder;

impl Decoder {
	pub fn decode(types: Vec<ParamType>, data: Vec<u8>) -> Result<Vec<Token>, Error> {
		unimplemented!();
	}
}

#[cfg(test)]
mod tests {
	use rustc_serialize::hex::FromHex;
	use token::Token;
	use super::Encoder;

	#[test]
	fn encode_address() {
		let address = Token::Address([0x11u8; 20]);
		let encoded = Encoder::encode(vec![address]);
		let expected = "0000000000000000000000001111111111111111111111111111111111111111".from_hex().unwrap();
		assert_eq!(encoded, expected);	
	}
}

