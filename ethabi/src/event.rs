//! Contract event.

use std::collections::HashMap;
use spec::{Event as EventInterface, ParamType};
use decoder::Decoder;
use token::Token;
use error::Error;

/// Represents decoded log.
#[derive(Debug, PartialEq)]
pub struct DecodedLog {
	/// Ordered params.
	pub params: Vec<(String, ParamType, Token)>,
	/// Address, is none for anonymous logs.
	pub address: Option<[u8; 20]>,
}

/// Contract event.
pub struct Event {
	interface: EventInterface,
}

impl Event {
	/// Creates new instance of `Event`.
	pub fn new(interface: EventInterface) -> Self {
		Event {
			interface: interface
		}
	}

	/// Decodes event indexed params and data.
	pub fn decode_log(&self, topics: Vec<[u8; 32]>, data: Vec<u8>) -> Result<DecodedLog, Error> {
		let topics_len = topics.len();
		// obtains all params info
		let topic_params = self.interface.indexed_params(true);
		let data_params = self.interface.indexed_params(false);
		// then take first topic if event is not anonymous
		let (address, to_skip) = match self.interface.anonymous {
			false => {
				let address_slice = try!(topics.get(0).ok_or(Error::InvalidData));
				let mut address = [0u8; 20];
				address.copy_from_slice(&address_slice[12..]);
				(Some(address), 1)
			},
			true => (None, 0)
		};


		let topic_types = topic_params.iter()
			.map(|p| p.kind.clone())
			.collect::<Vec<ParamType>>();

		let flat_topics = topics.into_iter()
			.skip(to_skip)
			.flat_map(|t| t.to_vec())
			.collect::<Vec<u8>>();

		let topic_tokens = try!(Decoder::decode(&topic_types, flat_topics));

		// topic may be only a 32 bytes encoded token
		if topic_tokens.len() != topics_len - to_skip {
			return Err(Error::InvalidData);
		}

		let topics_named_tokens = topic_params.into_iter()
			.map(|p| p.name)
			.zip(topic_tokens.into_iter());

		let data_types = data_params.iter()
			.map(|p| p.kind.clone())
			.collect::<Vec<ParamType>>();

		let data_tokens = try!(Decoder::decode(&data_types, data));

		let data_named_tokens = data_params.into_iter()
			.map(|p| p.name)
			.zip(data_tokens.into_iter());

		let named_tokens = topics_named_tokens
			.chain(data_named_tokens)
			.collect::<HashMap<String, Token>>();

		let decoded_params = self.interface.params_names()
			.into_iter()
			.zip(self.interface.param_types().into_iter())
			.map(|(name, kind)| (name.clone(), kind, named_tokens.get(&name).unwrap().clone()))
			.collect();

		let result = DecodedLog {
			params: decoded_params,
			address: address,
		};

		Ok(result)
	}

	/// Return the name of the event.
	pub fn name(&self) -> &str {
		&self.interface.name
	}
}

#[cfg(test)]
mod tests {
	use rustc_serialize::hex::FromHex;
	use spec::{Event as EventInterface, EventParam, ParamType};
	use super::{Event, DecodedLog};
	use token::{Token, TokenFromHex};

	#[test]
	fn test_decoding_event() {
		let i = EventInterface {
			name: "foo".to_owned(),
			inputs: vec![EventParam {
				name: "a".to_owned(),
				kind: ParamType::Int(256),
				indexed: false,
			}, EventParam {
				name: "b".to_owned(),
				kind: ParamType::Int(256),
				indexed: true,
			}, EventParam {
				name: "c".to_owned(),
				kind: ParamType::Address,
				indexed: false,
			}, EventParam {
				name: "d".to_owned(),
				kind: ParamType::Address,
				indexed: true,
			}],
			anonymous: false,
		};

		let event = Event::new(i);

		let result = event.decode_log(
			vec![
				"0000000000000000000000004444444444444444444444444444444444444444".token_from_hex().unwrap(),
				"0000000000000000000000000000000000000000000000000000000000000002".token_from_hex().unwrap(),
				"0000000000000000000000001111111111111111111111111111111111111111".token_from_hex().unwrap(),
			],
			("".to_owned() +
				"0000000000000000000000000000000000000000000000000000000000000003" +
				"0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap()
		).unwrap();

		assert_eq!(result, DecodedLog {
			params: vec![
				("a".to_owned(), ParamType::Int(256), Token::Int("0000000000000000000000000000000000000000000000000000000000000003".token_from_hex().unwrap())),
				("b".to_owned(), ParamType::Int(256), Token::Int("0000000000000000000000000000000000000000000000000000000000000002".token_from_hex().unwrap())),
				("c".to_owned(), ParamType::Address, Token::Address("2222222222222222222222222222222222222222".token_from_hex().unwrap())),
				("d".to_owned(), ParamType::Address, Token::Address("1111111111111111111111111111111111111111".token_from_hex().unwrap())),
			],
			address: Some("4444444444444444444444444444444444444444".token_from_hex().unwrap())
		});
	}
}
