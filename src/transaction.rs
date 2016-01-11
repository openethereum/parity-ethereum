use util::*;
use basic_types::*;
use error::Error;

pub enum Action {
	Create,
	Call(Address),
}

/// A set of information describing an externally-originating message call
/// or contract creation operation.
pub struct Transaction {
	pub nonce: U256,
	pub gas_price: U256,
	pub gas: U256,
	pub action: Action,
	pub value: U256,
	pub data: Bytes,
	pub signature: Signature,

	hash: RefCell<Option<H256>>, //TODO: make this private
}

impl Transaction {
	pub fn rlp_append_opt(&self, s: &mut RlpStream, with_seal: Seal) {
		s.append_list(6 + match with_seal { Seal::With => 3, _ => 0 });
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas);
		match self.action {
			Action::Create => s.append_empty_data(),
			Action::Call(ref to) => s.append(to),
		};
		s.append(&self.value);
		s.append(&self.data);
		match with_seal {
			Seal::With => {
				s.append(&(self.signature.as_slice()[64] as u16));
				s.append(&&self.signature.as_slice()[0..32]);
				s.append(&&self.signature.as_slice()[32..64]);
			},
			_ => {}
		}
	}

	pub fn rlp_bytes_opt(&self, with_seal: Seal) -> Bytes {
		let mut s = RlpStream::new();
		self.rlp_append_opt(&mut s, with_seal);
		s.out()
	}

	pub fn rlp_sha3_opt(&self, with_seal: Seal) -> H256 { self.rlp_bytes_opt(with_seal).sha3() }
}

impl RlpStandard for Transaction {
	fn rlp_append(&self, s: &mut RlpStream) { self.rlp_append_opt(s, Seal::With) }
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

	/// Returns transaction type.
	pub fn action(&self) -> &Action { &self.action }

	/// Returns transaction sender.
	pub fn sender(&self) -> Result<Address, Error> {
		let p = try!(ec::recover(&self.signature, &self.rlp_sha3_opt(Seal::Without)));
		Ok(From::from(p.sha3()))
	}
}

impl Decodable for Action {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let rlp = decoder.as_rlp();
		if rlp.is_empty() {
			Ok(Action::Create)
		} else {
			Ok(Action::Call(try!(rlp.as_val())))
		}
	}
}

impl Decodable for Transaction {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = try!(decoder.as_list());
		Ok(Transaction {
			nonce: try!(Decodable::decode(&d[0])),
			gas_price: try!(Decodable::decode(&d[1])),
			gas: try!(Decodable::decode(&d[2])),
			action: try!(Decodable::decode(&d[3])),
			value: try!(Decodable::decode(&d[4])),
			data: try!(Decodable::decode(&d[5])),
			signature: Signature::from_rsv(&try!(Decodable::decode(&d[6])), &try!(Decodable::decode(&d[7])), try!(u16::decode(&d[8])) as u8),
			hash: RefCell::new(None)
		})
	}
}
