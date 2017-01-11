// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use std::ops::{Deref, DerefMut};
use std::cell::*;
use rlp::*;
use util::sha3::Hashable;
use util::{H256, Address, U256, Bytes, HeapSizeOf};
use ethkey::{Signature, Secret, Public, recover, public_to_address, Error as EthkeyError};
use error::*;
use evm::Schedule;
use header::BlockNumber;
use ethjson;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
/// Transaction action type.
pub enum Action {
	/// Create creates new contract.
	Create,
	/// Calls contract at given address.
	/// In the case of a transfer, this is the receiver's address.'
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
			Ok(Action::Call(rlp.as_val()?))
		}
	}
}

/// A set of information describing an externally-originating message call
/// or contract creation operation.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
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
	pub fn rlp_append_unsigned_transaction(&self, s: &mut RlpStream, network_id: Option<u64>) {
		s.begin_list(if network_id.is_none() { 6 } else { 9 });
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas);
		match self.action {
			Action::Create => s.append_empty_data(),
			Action::Call(ref to) => s.append(to)
		};
		s.append(&self.value);
		s.append(&self.data);
		if let Some(n) = network_id {
			s.append(&n);
			s.append(&0u8);
			s.append(&0u8);
		}
	}
}

impl HeapSizeOf for Transaction {
	fn heap_size_of_children(&self) -> usize {
		self.data.heap_size_of_children()
	}
}

impl From<ethjson::state::Transaction> for SignedTransaction {
	fn from(t: ethjson::state::Transaction) -> Self {
		let to: Option<ethjson::hash::Address> = t.to.into();
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
		}.sign(&t.secret.into(), None)
	}
}

impl From<ethjson::transaction::Transaction> for SignedTransaction {
	fn from(t: ethjson::transaction::Transaction) -> Self {
		let to: Option<ethjson::hash::Address> = t.to.into();
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
	pub fn hash(&self, network_id: Option<u64>) -> H256 {
		let mut stream = RlpStream::new();
		self.rlp_append_unsigned_transaction(&mut stream, network_id);
		stream.out().sha3()
	}

	/// Signs the transaction as coming from `sender`.
	pub fn sign(self, secret: &Secret, network_id: Option<u64>) -> SignedTransaction {
		let sig = ::ethkey::sign(secret, &self.hash(network_id))
			.expect("data is valid and context has signing capabilities; qed");
		self.with_signature(sig, network_id)
	}

	/// Signs the transaction with signature.
	pub fn with_signature(self, sig: Signature, network_id: Option<u64>) -> SignedTransaction {
		SignedTransaction {
			unsigned: self,
			r: sig.r().into(),
			s: sig.s().into(),
			v: sig.v() as u64 + if let Some(n) = network_id { 35 + n * 2 } else { 27 },
			hash: Cell::new(None),
			sender: Cell::new(None),
		}
	}

	/// Useful for test incorrectly signed transactions.
	#[cfg(test)]
	pub fn invalid_sign(self) -> SignedTransaction {
		SignedTransaction {
			unsigned: self,
			r: U256::default(),
			s: U256::default(),
			v: 0,
			hash: Cell::new(None),
			sender: Cell::new(None),
		}
	}

	/// Specify the sender; this won't survive the serialize/deserialize process, but can be cloned.
	pub fn fake_sign(self, from: Address) -> SignedTransaction {
		SignedTransaction {
			unsigned: self,
			r: U256::default(),
			s: U256::default(),
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
#[cfg_attr(feature = "ipc", binary)]
pub struct SignedTransaction {
	/// Plain Transaction.
	unsigned: Transaction,
	/// The V field of the signature; the LS bit described which half of the curve our point falls
	/// in. The MS bits describe which network this transaction is for. If 27/28, its for all networks.
	v: u64,
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

impl DerefMut for SignedTransaction {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.unsigned
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
				nonce: d.val_at(0)?,
				gas_price: d.val_at(1)?,
				gas: d.val_at(2)?,
				action: d.val_at(3)?,
				value: d.val_at(4)?,
				data: d.val_at(5)?,
			},
			v: d.val_at(6)?,
			r: d.val_at(7)?,
			s: d.val_at(8)?,
			hash: Cell::new(None),
			sender: Cell::new(None),
		})
	}
}

impl Encodable for SignedTransaction {
	fn rlp_append(&self, s: &mut RlpStream) { self.rlp_append_sealed_transaction(s) }
}

impl HeapSizeOf for SignedTransaction {
	fn heap_size_of_children(&self) -> usize {
		self.unsigned.heap_size_of_children()
	}
}

impl SignedTransaction {
	/// Append object with a signature into RLP stream
	fn rlp_append_sealed_transaction(&self, s: &mut RlpStream) {
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
				let h = (&*self.rlp_bytes()).sha3();
				self.hash.set(Some(h));
				h
			}
		}
	}

	/// 0 if `v` would have been 27 under "Electrum" notation, 1 if 28 or 4 if invalid.
	pub fn standard_v(&self) -> u8 { match self.v { v if v == 27 || v == 28 || v > 36 => ((v - 1) % 2) as u8, _ => 4 } }

	/// The `v` value that appears in the RLP.
	pub fn original_v(&self) -> u64 { self.v }

	/// The network ID, or `None` if this is a global transaction.
	pub fn network_id(&self) -> Option<u64> {
		match self.v {
			v if v > 36 => Some((v - 35) / 2),
			_ => None,
		}
	}

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature {
		Signature::from_rsv(&self.r.into(), &self.s.into(), self.standard_v())
	}

	/// Checks whether the signature has a low 's' value.
	pub fn check_low_s(&self) -> Result<(), Error> {
		if !self.signature().is_low_s() {
			Err(EthkeyError::InvalidSignature.into())
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
				let s = public_to_address(&self.public_key()?);
				self.sender.set(Some(s));
				Ok(s)
			}
		}
	}

	/// Returns the public key of the sender.
	pub fn public_key(&self) -> Result<Public, Error> {
		Ok(recover(&self.signature(), &self.unsigned.hash(self.network_id()))?)
	}

	/// Do basic validation, checking for valid signature and minimum gas,
	// TODO: consider use in block validation.
	#[cfg(test)]
	#[cfg(feature = "json-tests")]
	pub fn validate(self, schedule: &Schedule, require_low: bool, allow_network_id_of_one: bool) -> Result<SignedTransaction, Error> {
		if require_low && !self.signature().is_low_s() {
			return Err(EthkeyError::InvalidSignature.into())
		}
		match self.network_id() {
			None => {},
			Some(1) if allow_network_id_of_one => {},
			_ => return Err(TransactionError::InvalidNetworkId.into()),
		}
		self.sender()?;
		if self.gas < U256::from(self.gas_required(&schedule)) {
			Err(TransactionError::InvalidGasLimit(::util::OutOfBounds{min: Some(U256::from(self.gas_required(&schedule))), max: None, found: self.gas}).into())
		} else {
			Ok(self)
		}
	}
}

/// Signed Transaction that is a part of canon blockchain.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
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

/// Queued transaction with additional information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct PendingTransaction {
	/// Signed transaction data.
	pub transaction: SignedTransaction,
	/// To be activated at this block. `None` for immediately.
	pub min_block: Option<BlockNumber>,
}

impl PendingTransaction {
	/// Create a new pending transaction from signed transaction.
	pub fn new(signed: SignedTransaction, min_block: Option<BlockNumber>) -> Self {
		PendingTransaction {
			transaction: signed,
			min_block: min_block,
		}
	}
}

impl From<SignedTransaction> for PendingTransaction {
	fn from(t: SignedTransaction) -> Self {
		PendingTransaction {
			transaction: t,
			min_block: None,
		}
	}
}

#[test]
fn sender_test() {
	let t: SignedTransaction = decode(&::rustc_serialize::hex::FromHex::from_hex("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap());
	assert_eq!(t.data, b"");
	assert_eq!(t.gas, U256::from(0x5208u64));
	assert_eq!(t.gas_price, U256::from(0x01u64));
	assert_eq!(t.nonce, U256::from(0x00u64));
	if let Action::Call(ref to) = t.action {
		assert_eq!(*to, "095e7baea6a6c7c4c2dfeb977efac326af552d87".into());
	} else { panic!(); }
	assert_eq!(t.value, U256::from(0x0au64));
	assert_eq!(t.sender().unwrap(), "0f65fe9276bc9a24ae7083ae28e2660ef72df99e".into());
	assert_eq!(t.network_id(), None);
}

#[test]
fn signing() {
	use ethkey::{Random, Generator};

	let key = Random.generate().unwrap();
	let t = Transaction {
		action: Action::Create,
		nonce: U256::from(42),
		gas_price: U256::from(3000),
		gas: U256::from(50_000),
		value: U256::from(1),
		data: b"Hello!".to_vec()
	}.sign(&key.secret(), None);
	assert_eq!(Address::from(key.public().sha3()), t.sender().unwrap());
	assert_eq!(t.network_id(), None);
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
	assert_eq!(t.network_id(), None);

	let t = t.clone();
	assert_eq!(Address::from(0x69), t.sender().unwrap());
	assert_eq!(t.network_id(), None);
}

#[test]
fn should_recover_from_network_specific_signing() {
	use ethkey::{Random, Generator};
	let key = Random.generate().unwrap();
	let t = Transaction {
		action: Action::Create,
		nonce: U256::from(42),
		gas_price: U256::from(3000),
		gas: U256::from(50_000),
		value: U256::from(1),
		data: b"Hello!".to_vec()
	}.sign(&key.secret(), Some(69));
	assert_eq!(Address::from(key.public().sha3()), t.sender().unwrap());
	assert_eq!(t.network_id(), Some(69));
}

#[test]
fn should_agree_with_vitalik() {
	use rustc_serialize::hex::FromHex;

	let test_vector = |tx_data: &str, address: &'static str| {
		let signed: SignedTransaction = decode(&FromHex::from_hex(tx_data).unwrap());
		signed.check_low_s().unwrap();
		assert_eq!(signed.sender().unwrap(), address.into());
		flushln!("networkid: {:?}", signed.network_id());
	};

	test_vector("f864808504a817c800825208943535353535353535353535353535353535353535808025a0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116da0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d", "0xf0f6f18bca1b28cd68e4357452947e021241e9ce");
	test_vector("f864018504a817c80182a410943535353535353535353535353535353535353535018025a0489efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bcaa0489efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6", "0x23ef145a395ea3fa3deb533b8a9e1b4c6c25d112");
	test_vector("f864028504a817c80282f618943535353535353535353535353535353535353535088025a02d7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5a02d7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5", "0x2e485e0c23b4c3c542628a5f672eeab0ad4888be");
	test_vector("f865038504a817c803830148209435353535353535353535353535353535353535351b8025a02a80e1ef1d7842f27f2e6be0972bb708b9a135c38860dbe73c27c3486c34f4e0a02a80e1ef1d7842f27f2e6be0972bb708b9a135c38860dbe73c27c3486c34f4de", "0x82a88539669a3fd524d669e858935de5e5410cf0");
	test_vector("f865048504a817c80483019a28943535353535353535353535353535353535353535408025a013600b294191fc92924bb3ce4b969c1e7e2bab8f4c93c3fc6d0a51733df3c063a013600b294191fc92924bb3ce4b969c1e7e2bab8f4c93c3fc6d0a51733df3c060", "0xf9358f2538fd5ccfeb848b64a96b743fcc930554");
	test_vector("f865058504a817c8058301ec309435353535353535353535353535353535353535357d8025a04eebf77a833b30520287ddd9478ff51abbdffa30aa90a8d655dba0e8a79ce0c1a04eebf77a833b30520287ddd9478ff51abbdffa30aa90a8d655dba0e8a79ce0c1", "0xa8f7aba377317440bc5b26198a363ad22af1f3a4");
	test_vector("f866068504a817c80683023e3894353535353535353535353535353535353535353581d88025a06455bf8ea6e7463a1046a0b52804526e119b4bf5136279614e0b1e8e296a4e2fa06455bf8ea6e7463a1046a0b52804526e119b4bf5136279614e0b1e8e296a4e2d", "0xf1f571dc362a0e5b2696b8e775f8491d3e50de35");
	test_vector("f867078504a817c807830290409435353535353535353535353535353535353535358201578025a052f1a9b320cab38e5da8a8f97989383aab0a49165fc91c737310e4f7e9821021a052f1a9b320cab38e5da8a8f97989383aab0a49165fc91c737310e4f7e9821021", "0xd37922162ab7cea97c97a87551ed02c9a38b7332");
	test_vector("f867088504a817c8088302e2489435353535353535353535353535353535353535358202008025a064b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c12a064b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c10", "0x9bddad43f934d313c2b79ca28a432dd2b7281029");
	test_vector("f867098504a817c809830334509435353535353535353535353535353535353535358202d98025a052f8f61201b2b11a78d6e866abc9c3db2ae8631fa656bfe5cb53668255367afba052f8f61201b2b11a78d6e866abc9c3db2ae8631fa656bfe5cb53668255367afb", "0x3c24d7329e92f84f08556ceb6df1cdb0104ca49f");
}
