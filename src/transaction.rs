use util::*;

/// A set of information describing an externally-originating message call
/// or contract creation operation.
pub struct Transaction {
	nonce: U256,
	gas_price: U256,
	gas: U256,
	to: Option<Address>,
	value: U256,
	data: Bytes,
}

impl Transaction {
	/// Is this transaction meant to create a contract?
	pub fn is_contract_creation(&self) -> bool {
		self.to.is_none()
	}

	/// Is this transaction meant to send a message?
	pub fn is_message_call(&self) -> bool {
		!self.is_contract_creation()
	}

	/// Get the hash of this transaction.
	pub fn sha3(&self) -> H256 {
		unimplemented!();
	}
}

impl Encodable for Transaction {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(| e | {
			self.nonce.encode(e);
			self.gas_price.encode(e);
			self.gas.encode(e);
			self.to.encode(e);
			self.value.encode(e);
			self.data.encode(e);
		})
	}
}

impl Decodable for Transaction {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = try!(decoder.as_list());

		let transaction = Transaction {
			nonce: try!(Decodable::decode(&d[0])),
			gas_price: try!(Decodable::decode(&d[1])),
			gas: try!(Decodable::decode(&d[2])),
			to: try!(Decodable::decode(&d[3])),
			value: try!(Decodable::decode(&d[4])),
			data: try!(Decodable::decode(&d[5])),
		};

		Ok(transaction)
	}
}

