use std::ptr;
use spec::ParamType;
use token::Token;
use error::Error;

fn pad_u32(value: u32) -> [u8; 32] {
	let mut padded = [0u8; 32];
	let bytes = (32 - value.leading_zeros() + 7) / 8;
	unsafe {
		ptr::copy(&[value] as *const u32 as *const u8, padded.as_mut_ptr().offset(32 - bytes as isize), bytes as usize);
	}
	padded
}

#[derive(Debug)]
enum Mediate {
	Raw(Vec<[u8; 32]>),
	Prefixed(Vec<[u8; 32]>),
	FixedArray(Vec<Mediate>),
	Array(Vec<Mediate>),
}

impl Mediate {
	fn init_len(&self) -> u32 {
		match *self {
			Mediate::Raw(ref raw) => 32 * raw.len() as u32,
			Mediate::Prefixed(_) => 32,
			Mediate::FixedArray(ref nes) => nes.iter().fold(0, |acc, m| acc + m.init_len()),
			Mediate::Array(_) => 32,
		}
	}

	fn closing_len(&self) -> u32 {
		match *self {
			Mediate::Raw(_) => 0,
			Mediate::Prefixed(ref pre) => pre.len() as u32 * 32,
			Mediate::FixedArray(ref nes) => nes.iter().fold(0, |acc, m| acc + m.closing_len()),
			Mediate::Array(ref nes) => nes.iter().fold(32, |acc, m| acc + m.init_len() + m.closing_len()),
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
			Mediate::FixedArray(ref nes) => {
				nes.iter()
					.enumerate()
					.flat_map(|(i, m)| m.init(Mediate::offset_for(nes, i)))
					.collect()
			},
			Mediate::Prefixed(_) | Mediate::Array(_) => {
				vec![pad_u32(suffix_offset)]
			}
		}
	}

	fn closing(&self, offset: u32) -> Vec<[u8; 32]> {
		match *self {
			Mediate::Raw(ref raw) => vec![],
			Mediate::Prefixed(ref pre) => pre.clone(),
			Mediate::FixedArray(ref nes) => {
				// offset is not taken into account, cause it would be counted twice
				// fixed array is just raw representations of similar consecutive items
				nes.iter()
					.enumerate()
					.flat_map(|(i, m)| m.closing(Mediate::offset_for(nes, i)))
					.collect()
			},
			Mediate::Array(ref nes) => {
				// + 32 added to offset represents len of the array prepanded to closing
				let prefix = vec![pad_u32(nes.len() as u32)].into_iter();

				let inits = nes.iter()
					.enumerate()
					.flat_map(|(i, m)| m.init(offset + Mediate::offset_for(nes, i) + 32));

				let closings = nes.iter()
					.enumerate()
					.flat_map(|(i, m)| m.closing(offset + Mediate::offset_for(nes, i)));

				prefix.chain(inits).chain(closings).collect()
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
			Token::Bytes(bytes) => {
				let mut result = vec![];
				let len = (bytes.len() + 31) / 32;
				result.push(pad_u32(bytes.len() as u32));

				for i in 0..len {
					let mut padded = [0u8; 32];

					let to_copy = match i == len - 1 {
						false => 32,
						true => match bytes.len() % 32 {
							0 => 32,
							x => x,
						},
					};

					let offset = 32 * i as isize;

					unsafe {
						ptr::copy(bytes.as_ptr().offset(offset), padded.as_mut_ptr(), to_copy);
					}

					result.push(padded);
				}

				Mediate::Prefixed(result)
			},
			Token::Array(tokens) => {
				let mediates = tokens.into_iter()
					.map(Encoder::encode_token)
					.collect();

				Mediate::Array(mediates)
			},
			Token::FixedArray(tokens) => {
				let mediates = tokens.into_iter()
					.map(Encoder::encode_token)
					.collect();

				Mediate::FixedArray(mediates)
			},
			_ => {
				unimplemented!();
			},
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

	#[test]
	fn encode_dynamic_array_of_addresses() {
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let addresses = Token::Array(vec![address1, address2]);
		let encoded = Encoder::encode(vec![addresses]);
		let expected = ("".to_owned() + 
			"0000000000000000000000000000000000000000000000000000000000000020" + 
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"0000000000000000000000001111111111111111111111111111111111111111" +
			"0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}

	#[test]
	fn encode_fixed_array_of_addresses() {
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let addresses = Token::FixedArray(vec![address1, address2]);
		let encoded = Encoder::encode(vec![addresses]);
		let expected = ("".to_owned() + 
			"0000000000000000000000001111111111111111111111111111111111111111" +
			"0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}

	#[test]
	fn encode_two_addresses() {
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let encoded = Encoder::encode(vec![address1, address2]);
		let expected = ("".to_owned() + 
			"0000000000000000000000001111111111111111111111111111111111111111" +
			"0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}

	#[test]
	fn encode_fixed_array_of_dynamic_array_of_addresses() {
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let address3 = Token::Address([0x33u8; 20]);
		let address4 = Token::Address([0x44u8; 20]);
		let array0 = Token::Array(vec![address1, address2]);
		let array1 = Token::Array(vec![address3, address4]);
		let fixed = Token::FixedArray(vec![array0, array1]);
		let encoded = Encoder::encode(vec![fixed]);
		let expected = ("".to_owned() + 
			"0000000000000000000000000000000000000000000000000000000000000040" + 
			"00000000000000000000000000000000000000000000000000000000000000a0" + 
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"0000000000000000000000001111111111111111111111111111111111111111" +
			"0000000000000000000000002222222222222222222222222222222222222222" +
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"0000000000000000000000003333333333333333333333333333333333333333" +
			"0000000000000000000000004444444444444444444444444444444444444444").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}

	#[test]
	fn encode_dynamic_array_of_fixed_array_of_addresses() {
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let address3 = Token::Address([0x33u8; 20]);
		let address4 = Token::Address([0x44u8; 20]);
		let array0 = Token::FixedArray(vec![address1, address2]);
		let array1 = Token::FixedArray(vec![address3, address4]);
		let dynamic = Token::Array(vec![array0, array1]);
		let encoded = Encoder::encode(vec![dynamic]);
		let expected = ("".to_owned() + 
			"0000000000000000000000000000000000000000000000000000000000000020" +
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"0000000000000000000000001111111111111111111111111111111111111111" +
			"0000000000000000000000002222222222222222222222222222222222222222" +
			"0000000000000000000000003333333333333333333333333333333333333333" +
			"0000000000000000000000004444444444444444444444444444444444444444").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}

	#[test]
	fn encode_dynamic_array_of_dynamic_arrays() {
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let array0 = Token::Array(vec![address1]);
		let array1 = Token::Array(vec![address2]);
		let dynamic = Token::Array(vec![array0, array1]);
		let encoded = Encoder::encode(vec![dynamic]);
		let expected = ("".to_owned() + 
			"0000000000000000000000000000000000000000000000000000000000000020" + 
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"0000000000000000000000000000000000000000000000000000000000000080" +
			"00000000000000000000000000000000000000000000000000000000000000c0" +
			"0000000000000000000000000000000000000000000000000000000000000001" +
			"0000000000000000000000001111111111111111111111111111111111111111" +
			"0000000000000000000000000000000000000000000000000000000000000001" +
			"0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}

	#[test]
	fn encode_dynamic_array_of_dynamic_arrays2() {
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let address3 = Token::Address([0x33u8; 20]);
		let address4 = Token::Address([0x44u8; 20]);
		let array0 = Token::Array(vec![address1, address2]);
		let array1 = Token::Array(vec![address3, address4]);
		let dynamic = Token::Array(vec![array0, array1]);
		let encoded = Encoder::encode(vec![dynamic]);
		let expected = ("".to_owned() + 
			"0000000000000000000000000000000000000000000000000000000000000020" + 
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"0000000000000000000000000000000000000000000000000000000000000080" +
			"00000000000000000000000000000000000000000000000000000000000000e0" +
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"0000000000000000000000001111111111111111111111111111111111111111" +
			"0000000000000000000000002222222222222222222222222222222222222222" +
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"0000000000000000000000003333333333333333333333333333333333333333" +
			"0000000000000000000000004444444444444444444444444444444444444444").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}

	#[test]
	fn encode_fixed_array_of_fixed_arrays() {
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let address3 = Token::Address([0x33u8; 20]);
		let address4 = Token::Address([0x44u8; 20]);
		let array0 = Token::FixedArray(vec![address1, address2]);
		let array1 = Token::FixedArray(vec![address3, address4]);
		let fixed = Token::FixedArray(vec![array0, array1]);
		let encoded = Encoder::encode(vec![fixed]);
		let expected = ("".to_owned() + 
			"0000000000000000000000001111111111111111111111111111111111111111" +
			"0000000000000000000000002222222222222222222222222222222222222222" +
			"0000000000000000000000003333333333333333333333333333333333333333" +
			"0000000000000000000000004444444444444444444444444444444444444444").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}

	#[test]
	fn encode_bytes() {
		let bytes = Token::Bytes(vec![0x12, 0x34]);
		let encoded = Encoder::encode(vec![bytes]);
		let expected = ("".to_owned() + 
			"0000000000000000000000000000000000000000000000000000000000000020" +
			"0000000000000000000000000000000000000000000000000000000000000002" +
			"1234000000000000000000000000000000000000000000000000000000000000").from_hex().unwrap();
		assert_eq!(encoded, expected);
	}
}

