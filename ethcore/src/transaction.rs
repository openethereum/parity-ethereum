// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Transaction data structure.

use util::*;
use error::*;
use evm::Schedule;
use header::BlockNumber;
use ethjson;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Transaction action type.
pub enum Action {
	/// Create creates new contract.
	Create,
	/// Calls contract at given address.
	Call(Address),
}

impl Default for Action {
	fn default() -> Action { Action::Create }
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

/// A set of information describing an externally-originating message call
/// or contract creation operation.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
	/// Nonce.
	pub nonce: U256,
	/// Gas price.
	pub gas_price: U256,
	/// Gas paid up front for transaction execution.
	pub gas: U256,
	/// Action, can be either call or contract create.
	pub action: Action,
	/// Transfered value.
	pub value: U256,
	/// Transaction data.
	pub data: Bytes,
}

impl Transaction {
	/// Append object with a without signature into RLP stream
	pub fn rlp_append_unsigned_transaction(&self, s: &mut RlpStream) {
		s.begin_list(6);
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas);
		match self.action {
			Action::Create => s.append_empty_data(),
			Action::Call(ref to) => s.append(to)
		};
		s.append(&self.value);
		s.append(&self.data);
	}
}

impl From<ethjson::state::Transaction> for SignedTransaction {
	fn from(t: ethjson::state::Transaction) -> Self {
		let to: Option<_> = t.to.into();
		Transaction {
			nonce: t.nonce.into(),
			gas_price: t.gas_price.into(),
			gas: t.gas_limit.into(),
			action: match to {
				Some(to) => Action::Call(to.into()),
				None => Action::Create
			},
			value: t.value.into(),
			data: t.data.into(),
		}.sign(&t.secret.into())
	}
}

impl From<ethjson::transaction::Transaction> for SignedTransaction {
	fn from(t: ethjson::transaction::Transaction) -> Self {
		let to: Option<_> = t.to.into();
		SignedTransaction {
			unsigned: Transaction {
				nonce: t.nonce.into(),
				gas_price: t.gas_price.into(),
				gas: t.gas_limit.into(),
				action: match to {
					Some(to) => Action::Call(to.into()),
					None => Action::Create
				},
				value: t.value.into(),
				data: t.data.into(),
			},
			r: t.r.into(),
			s: t.s.into(),
			v: t.v.into(),
			sender: Cell::new(None),
			hash: Cell::new(None)
		}
	}
}

impl Transaction {
	/// The message hash of the transaction.
	pub fn hash(&self) -> H256 {
		let mut stream = RlpStream::new();
		self.rlp_append_unsigned_transaction(&mut stream);
		stream.out().sha3()
	}

	/// Signs the transaction as coming from `sender`.
	pub fn sign(self, secret: &Secret) -> SignedTransaction {
		let sig = ec::sign(secret, &self.hash());
		let (r, s, v) = sig.unwrap().to_rsv();
		SignedTransaction {
			unsigned: self,
			r: r,
			s: s,
			v: v + 27,
			hash: Cell::new(None),
			sender: Cell::new(None),
		}
	}

	/// Useful for test incorrectly signed transactions.
	#[cfg(test)]
	pub fn invalid_sign(self) -> SignedTransaction {
		SignedTransaction {
			unsigned: self,
			r: U256::zero(),
			s: U256::zero(),
			v: 0,
			hash: Cell::new(None),
			sender: Cell::new(None),
		}
	}

	/// Specify the sender; this won't survive the serialize/deserialize process, but can be cloned.
	pub fn fake_sign(self, from: Address) -> SignedTransaction {
		SignedTransaction {
			unsigned: self,
			r: U256::zero(),
			s: U256::zero(),
			v: 0,
			hash: Cell::new(None),
			sender: Cell::new(Some(from)),
		}
	}

	/// Get the transaction cost in gas for the given params.
	pub fn gas_required_for(is_create: bool, data: &[u8], schedule: &Schedule) -> u64 {
		data.iter().fold(
			(if is_create {schedule.tx_create_gas} else {schedule.tx_gas}) as u64,
			|g, b| g + (match *b { 0 => schedule.tx_data_zero_gas, _ => schedule.tx_data_non_zero_gas }) as u64
		)
	}

	/// Get the transaction cost in gas for this transaction.
	pub fn gas_required(&self, schedule: &Schedule) -> u64 {
		Self::gas_required_for(match self.action{Action::Create=>true, Action::Call(_)=>false}, &self.data, schedule)
	}
}

/// Signed transaction information.
#[derive(Debug, Clone, Eq)]
pub struct SignedTransaction {
	/// Plain Transaction.
	unsigned: Transaction,
	/// The V field of the signature, either 27 or 28; helps describe the point on the curve.
	v: u8,
	/// The R field of the signature; helps describe the point on the curve.
	r: U256,
	/// The S field of the signature; helps describe the point on the curve.
	s: U256,
	/// Cached hash.
	hash: Cell<Option<H256>>,
	/// Cached sender.
	sender: Cell<Option<Address>>,
}

impl PartialEq for SignedTransaction {
	fn eq(&self, other: &SignedTransaction) -> bool {
		self.unsigned == other.unsigned && self.v == other.v && self.r == other.r && self.s == other.s
	}
}

impl Deref for SignedTransaction {
	type Target = Transaction;

	fn deref(&self) -> &Self::Target {
		&self.unsigned
	}
}

impl Decodable for SignedTransaction {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		if d.item_count() != 9 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
		Ok(SignedTransaction {
			unsigned: Transaction {
				nonce: try!(d.val_at(0)),
				gas_price: try!(d.val_at(1)),
				gas: try!(d.val_at(2)),
				action: try!(d.val_at(3)),
				value: try!(d.val_at(4)),
				data: try!(d.val_at(5)),
			},
			v: try!(d.val_at(6)),
			r: try!(d.val_at(7)),
			s: try!(d.val_at(8)),
			hash: Cell::new(None),
			sender: Cell::new(None),
		})
	}
}

impl Encodable for SignedTransaction {
	fn rlp_append(&self, s: &mut RlpStream) { self.rlp_append_sealed_transaction(s) }
}

impl SignedTransaction {
	/// Append object with a signature into RLP stream
	pub fn rlp_append_sealed_transaction(&self, s: &mut RlpStream) {
		s.begin_list(9);
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas);
		match self.action {
			Action::Create => s.append_empty_data(),
			Action::Call(ref to) => s.append(to)
		};
		s.append(&self.value);
		s.append(&self.data);
		s.append(&self.v);
		s.append(&self.r);
		s.append(&self.s);
	}

	/// Get the hash of this header (sha3 of the RLP).
	pub fn hash(&self) -> H256 {
		let hash = self.hash.get();
		match hash {
			Some(h) => h,
			None => {
				let h = self.rlp_sha3();
				self.hash.set(Some(h));
				h
			}
		}
	}

	/// 0 is `v` is 27, 1 if 28, and 4 otherwise.
	pub fn standard_v(&self) -> u8 { match self.v { 27 => 0, 28 => 1, _ => 4 } }

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature { Signature::from_rsv(&From::from(&self.r), &From::from(&self.s), self.standard_v()) }

	/// Checks whether the signature has a low 's' value.
	pub fn check_low_s(&self) -> Result<(), Error> {
		if !ec::is_low_s(&self.s) {
			Err(Error::Util(UtilError::Crypto(CryptoError::InvalidSignature)))
		} else {
			Ok(())
		}
	}

	/// Returns transaction sender.
	pub fn sender(&self) -> Result<Address, Error> {
		let sender = self.sender.get();
		match sender {
			Some(s) => Ok(s),
			None => {
				let s = Address::from(try!(ec::recover(&self.signature(), &self.unsigned.hash())).sha3());
				self.sender.set(Some(s));
				Ok(s)
			}
		}
	}

	/// Do basic validation, checking for valid signature and minimum gas,
	// TODO: consider use in block validation.
	#[cfg(test)]
	#[cfg(feature = "json-tests")]
	pub fn validate(self, schedule: &Schedule, require_low: bool) -> Result<SignedTransaction, Error> {
		if require_low && !ec::is_low_s(&self.s) {
			return Err(Error::Util(UtilError::Crypto(CryptoError::InvalidSignature)));
		}
		try!(self.sender());
		if self.gas < U256::from(self.gas_required(&schedule)) {
			Err(From::from(TransactionError::InvalidGasLimit(OutOfBounds{min: Some(U256::from(self.gas_required(&schedule))), max: None, found: self.gas})))
		} else {
			Ok(self)
		}
	}
}

/// Signed Transaction that is a part of canon blockchain.
#[derive(Debug, PartialEq, Eq)]
pub struct LocalizedTransaction {
	/// Signed part.
	pub signed: SignedTransaction,
	/// Block number.
	pub block_number: BlockNumber,
	/// Block hash.
	pub block_hash: H256,
	/// Transaction index within block.
	pub transaction_index: usize
}

impl Deref for LocalizedTransaction {
	type Target = SignedTransaction;

	fn deref(&self) -> &Self::Target {
		&self.signed
	}
}

#[test]
fn sender_test() {
	let t: SignedTransaction = decode(&FromHex::from_hex("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap());
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

#[test]
fn signing() {
	let key = KeyPair::create().unwrap();
	let t = Transaction {
		action: Action::Create,
		nonce: U256::from(42),
		gas_price: U256::from(3000),
		gas: U256::from(50_000),
		value: U256::from(1),
		data: b"Hello!".to_vec()
	}.sign(&key.secret());
	assert_eq!(Address::from(key.public().sha3()), t.sender().unwrap());
}

#[test]
fn fake_signing() {
	let t = Transaction {
		action: Action::Create,
		nonce: U256::from(42),
		gas_price: U256::from(3000),
		gas: U256::from(50_000),
		value: U256::from(1),
		data: b"Hello!".to_vec()
	}.fake_sign(Address::from(0x69));
	assert_eq!(Address::from(0x69), t.sender().unwrap());

	let t = t.clone();
	assert_eq!(Address::from(0x69), t.sender().unwrap());
}
