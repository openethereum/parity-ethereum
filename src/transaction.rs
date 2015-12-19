use util::hash::*;
use util::bytes::*;
use util::uint::*;
use util::rlp::*;

pub struct Transaction {
	nonce: U256,
	gas_price: U256,
	gas: U256,
	receive_address: Option<Address>,
	value: U256,
	data: Bytes,
}

impl Transaction {
	pub fn is_contract_creation(&self) -> bool {
		self.receive_address.is_none()
	}

	pub fn is_message_call(&self) -> bool {
		!self.is_contract_creation()
	}
}

impl Encodable for Transaction {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(| e | {
			self.nonce.encode(e);
			self.gas_price.encode(e);
			self.gas.encode(e);
			self.receive_address.encode(e);
			self.value.encode(e);
			self.data.encode(e);
		})
	}
}

impl Decodable for Transaction {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>  where D: Decoder {
		let d = try!(decoder.as_list());

		let transaction = Transaction {
			nonce: try!(Decodable::decode(&d[0])),
			gas_price: try!(Decodable::decode(&d[1])),
			gas: try!(Decodable::decode(&d[2])),
			receive_address: try!(Decodable::decode(&d[3])),
			value: try!(Decodable::decode(&d[4])),
			data: try!(Decodable::decode(&d[5])),
		};

		Ok(transaction)
	}
}

