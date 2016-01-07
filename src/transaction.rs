use std::marker::PhantomData;
use util::hash::*;
use util::bytes::*;
use util::uint::*;
use util::rlp::*;

#[derive(Eq, PartialEq)]
pub enum TransactionKind {
	ContractCreation,
	MessageCall
}

/// A set of information describing an externally-originating message call
/// or contract creation operation.
pub struct Transaction {
	pub nonce: U256,
	pub gas_price: U256,
	pub gas: U256,
	pub to: Option<Address>,
	pub value: U256,
	pub data: Bytes,
}

impl Transaction {
	pub fn new() -> Self {
		Transaction {
			nonce: U256::zero(),
			gas_price: U256::zero(),
			gas: U256::zero(),
			to: None,
			value: U256::zero(),
			data: vec![]
		}
	}

	/// Is this transaction meant to create a contract?
	pub fn is_contract_creation(&self) -> bool {
		self.kind() == TransactionKind::ContractCreation
	}

	/// Is this transaction meant to send a message?
	pub fn is_message_call(&self) -> bool {
		self.kind() == TransactionKind::MessageCall
	}

	/// Returns transaction type.
	pub fn kind(&self) -> TransactionKind {
		match self.to.is_some() {
			true => TransactionKind::MessageCall,
			false => TransactionKind::ContractCreation
		}
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

