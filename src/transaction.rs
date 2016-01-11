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

	// signature
	pub v: u8,
	pub r: H256,
	pub s: H256,

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
			Seal::With => { s.append(&(self.v as u16)).append(&self.r).append(&self.s); },
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

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature { Signature::from_rsv(&self.r, &self.s, self.v - 27) }

	/// The message hash of the transaction.
	pub fn message_hash(&self) -> H256 { self.rlp_sha3_opt(Seal::Without) }

	/// Returns transaction sender.
	pub fn sender(&self) -> Result<Address, Error> { Ok(From::from(try!(ec::recover(&self.signature(), &self.message_hash())).sha3())) }
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
			v: try!(u16::decode(&d[6])) as u8,
			r: try!(Decodable::decode(&d[7])),
			s: try!(Decodable::decode(&d[8])),
			hash: RefCell::new(None)
		})
	}
}

#[test]
fn sender_test() {
	let t: Transaction = decode(&FromHex::from_hex("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap());
	assert_eq!(t.data, b"");
	assert_eq!(t.gas, U256::from(0x5208u64));
	assert_eq!(t.gas_price, U256::from(0x01u64));
	assert_eq!(t.nonce, U256::from(0x00u64));
	if let Action::Call(ref to) = t.action {
		assert_eq!(*to, address_from_hex("095e7baea6a6c7c4c2dfeb977efac326af552d87"));
	} else { panic!(); }
	assert_eq!(t.value, U256::from(0x0au64));
	assert_eq!(t.sender().unwrap(), address_from_hex("0f65fe9276bc9a24ae7083ae28e2660ef72df99e"));
}