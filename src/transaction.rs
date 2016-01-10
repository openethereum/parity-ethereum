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

	hash: RefCell<Option<H256>>, //TODO: make this private
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
}

impl RlpStandard for Transaction {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append_list(6);
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas);
		s.append(&self.to);
		s.append(&self.value);
		s.append(&self.data);
	}
}

impl Transaction {
	/// Get the hash of this header (sha3 of the RLP).
	pub fn hash(&self) -> H256 {
 		let mut hash = self.hash.borrow_mut();
 		match &mut *hash {
 			&mut Some(ref h) => h.clone(),
 			hash @ &mut None => {
 				*hash = Some(self.rlp_sha3());
 				hash.as_ref().unwrap().clone()
 			}
		}
	}

	/// Note that some fields have changed. Resets the memoised hash.
	pub fn note_dirty(&self) {
 		*self.hash.borrow_mut() = None;
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
			hash: RefCell::new(None)
		};

		Ok(transaction)
	}
}

