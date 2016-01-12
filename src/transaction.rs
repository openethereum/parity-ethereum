use util::*;
use basic_types::*;
use error::*;
use evm::Schedule;

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
	pub r: U256,
	pub s: U256,

	hash: RefCell<Option<H256>>, //TODO: make this private
}

impl Transaction {
	/// Append object into RLP stream, optionally with or without the signature.
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

	/// Get the RLP serialisation of the object, optionally with or without the signature.
	pub fn rlp_bytes_opt(&self, with_seal: Seal) -> Bytes {
		let mut s = RlpStream::new();
		self.rlp_append_opt(&mut s, with_seal);
		s.out()
	}
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

	/// 0 is `v` is 27, 1 if 28, and 4 otherwise.
	pub fn standard_v(&self) -> u8 { match self.v { 27 => 0, 28 => 1, _ => 4 } }

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature { Signature::from_rsv(&From::from(&self.r), &From::from(&self.s), self.standard_v()) }

	/// The message hash of the transaction.
	pub fn message_hash(&self) -> H256 { self.rlp_bytes_opt(Seal::Without).sha3() }

	/// Returns transaction sender.
	pub fn sender(&self) -> Result<Address, Error> { Ok(From::from(try!(ec::recover(&self.signature(), &self.message_hash())).sha3())) }

	/// Get the transaction cost in gas for the given params.
	pub fn gas_required_for(is_create: bool, data: &[u8], schedule: &Schedule) -> U256 {
		// CRITICAL TODO XXX FIX NEED BIGINT!!!!!
		data.iter().fold(
			U256::from(if is_create {schedule.tx_create_gas} else {schedule.tx_gas}),
			|g, b| g + U256::from(match *b { 0 => schedule.tx_data_zero_gas, _ => schedule.tx_data_non_zero_gas})
		)
	}

	/// Get the transaction cost in gas for this transaction.
	pub fn gas_required(&self, schedule: &Schedule) -> U256 {
		Self::gas_required_for(match self.action{Action::Create=>true, Action::Call(_)=>false}, &self.data, schedule)
	}

	/// Do basic validation, checking for valid signature and minimum gas,
	pub fn validate(self, schedule: &Schedule) -> Result<Transaction, Error> {
		try!(self.sender());
		if self.gas < self.gas_required(&schedule) {
			Err(From::from(TransactionError::InvalidGasLimit(OutOfBounds{min: Some(self.gas_required(&schedule)), max: None, found: self.gas})))
		} else {
			Ok(self)
		}
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
		if d.len() != 9 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
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

pub fn clean(s: &str) -> &str {
	if s.len() >= 2 && &s[0..2] == "0x" {
		&s[2..]
	} else {
		s
	}
}

pub fn bytes_from_json(json: &Json) -> Bytes {
	let s = json.as_string().unwrap();
	if s.len() % 2 == 1 {
		FromHex::from_hex(&("0".to_string() + &(clean(s).to_string()))[..]).unwrap_or(vec![])
	} else {
		FromHex::from_hex(clean(s)).unwrap_or(vec![])
	}
}

pub fn address_from_json(json: &Json) -> Address {
	let s = json.as_string().unwrap();
	if s.len() % 2 == 1 {
		address_from_hex(&("0".to_string() + &(clean(s).to_string()))[..])
	} else {
		address_from_hex(clean(s))
	}
}

pub fn u256_from_json(json: &Json) -> U256 {
	let s = json.as_string().unwrap();
	if s.len() >= 2 && &s[0..2] == "0x" {
		// hex
		U256::from_str(&s[2..]).unwrap()
	}
	else {
		// dec
		U256::from_dec_str(s).unwrap()
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use evm::Schedule;
	use header::BlockNumber;
	use super::*;

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

	fn do_json_test(json_data: &[u8]) -> Vec<String> {
		let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
		let mut failed = Vec::new();
		let schedule = Schedule::new_frontier();
		for (name, test) in json.as_object().unwrap() {
			let mut fail = false;
			let mut fail_unless = |cond: bool| if !cond && fail { failed.push(name.to_string()); fail = true };
			let _ = BlockNumber::from_str(test["blocknumber"].as_string().unwrap()).unwrap();
			let rlp = bytes_from_json(&test["rlp"]);
			let res = UntrustedRlp::new(&rlp).as_val().map_err(|e| From::from(e)).and_then(|t: Transaction| t.validate(&schedule));
			fail_unless(test.find("transaction").is_none() == res.is_err());
			if let (Some(&Json::Object(ref tx)), Some(&Json::String(ref expect_sender))) = (test.find("transaction"), test.find("sender")) {
				let t = res.unwrap();
				fail_unless(t.sender().unwrap() == address_from_hex(clean(expect_sender)));
				fail_unless(t.data == bytes_from_json(&tx["data"]));
				fail_unless(t.gas == u256_from_json(&tx["gasLimit"]));
				fail_unless(t.gas_price == u256_from_json(&tx["gasPrice"]));
				fail_unless(t.nonce == u256_from_json(&tx["nonce"]));
				fail_unless(t.value == u256_from_json(&tx["value"]));
				if let Action::Call(ref to) = t.action {
					fail_unless(to == &address_from_json(&tx["to"]));
				} else {
					fail_unless(bytes_from_json(&tx["to"]).len() == 0);
				}
			}
		}
		for f in failed.iter() {
			println!("FAILED: {:?}", f);
		}
		failed
	}

	macro_rules! declare_test {
		($test_set_name: ident/$name: ident) => {
			#[test]
			#[allow(non_snake_case)]
			fn $name() {
				assert!(do_json_test(include_bytes!(concat!("../res/ethereum/tests/", stringify!($test_set_name), "/", stringify!($name), ".json"))).len() == 0);
			}
		}
	}

	declare_test!{TransactionTests/ttTransactionTest}
	declare_test!{TransactionTests/tt10mbDataField}
	declare_test!{TransactionTests/ttWrongRLPTransaction}
}