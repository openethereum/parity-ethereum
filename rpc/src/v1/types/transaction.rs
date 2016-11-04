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

use ethcore::contract_address;
use ethcore::transaction::{LocalizedTransaction, Action, SignedTransaction};
use v1::types::{Bytes, H160, H256, U256, H512};

/// Transaction
#[derive(Debug, Default, Serialize)]
pub struct Transaction {
	/// Hash
	pub hash: H256,
	/// Nonce
	pub nonce: U256,
	/// Block hash
	#[serde(rename="blockHash")]
	pub block_hash: Option<H256>,
	/// Block number
	#[serde(rename="blockNumber")]
	pub block_number: Option<U256>,
	/// Transaction Index
	#[serde(rename="transactionIndex")]
	pub transaction_index: Option<U256>,
	/// Sender
	pub from: H160,
	/// Recipient
	pub to: Option<H160>,
	/// Transfered value
	pub value: U256,
	/// Gas Price
	#[serde(rename="gasPrice")]
	pub gas_price: U256,
	/// Gas
	pub gas: U256,
	/// Data
	pub input: Bytes,
	/// Creates contract
	pub creates: Option<H160>,
	/// Raw transaction data
	pub raw: Bytes,
	/// Public key of the signer.
	#[serde(rename="publicKey")]
	pub public_key: Option<H512>,
	/// The V field of the signature.
	pub v: u8,
	/// The R field of the signature.
	pub r: H256,
	/// The S field of the signature.
	pub s: H256,
}

impl From<LocalizedTransaction> for Transaction {
	fn from(t: LocalizedTransaction) -> Transaction {
		let signature = t.signature();
		Transaction {
			hash: t.hash().into(),
			nonce: t.nonce.into(),
			block_hash: Some(t.block_hash.clone().into()),
			block_number: Some(t.block_number.into()),
			transaction_index: Some(t.transaction_index.into()),
			from: t.sender().unwrap().into(),
			to: match t.action {
				Action::Create => None,
				Action::Call(ref address) => Some(address.clone().into())
			},
			value: t.value.into(),
			gas_price: t.gas_price.into(),
			gas: t.gas.into(),
			input: Bytes::new(t.data.clone()),
			creates: match t.action {
				Action::Create => Some(contract_address(&t.sender().unwrap(), &t.nonce).into()),
				Action::Call(_) => None,
			},
			raw: ::rlp::encode(&t.signed).to_vec().into(),
			public_key: t.public_key().ok().map(Into::into),
			v: signature.v(),
			r: signature.r().into(),
			s: signature.s().into(),
		}
	}
}

impl From<SignedTransaction> for Transaction {
	fn from(t: SignedTransaction) -> Transaction {
		let signature = t.signature();
		Transaction {
			hash: t.hash().into(),
			nonce: t.nonce.into(),
			block_hash: None,
			block_number: None,
			transaction_index: None,
			from: t.sender().unwrap().into(),
			to: match t.action {
				Action::Create => None,
				Action::Call(ref address) => Some(address.clone().into())
			},
			value: t.value.into(),
			gas_price: t.gas_price.into(),
			gas: t.gas.into(),
			input: Bytes::new(t.data.clone()),
			creates: match t.action {
				Action::Create => Some(contract_address(&t.sender().unwrap(), &t.nonce).into()),
				Action::Call(_) => None,
			},
			raw: ::rlp::encode(&t).to_vec().into(),
			public_key: t.public_key().ok().map(Into::into),
			v: signature.v(),
			r: signature.r().into(),
			s: signature.s().into(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Transaction;
	use serde_json;

	#[test]
	fn test_transaction_serialize() {
		let t = Transaction::default();
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0","blockHash":null,"blockNumber":null,"transactionIndex":null,"from":"0x0000000000000000000000000000000000000000","to":null,"value":"0x0","gasPrice":"0x0","gas":"0x0","input":"0x","creates":null,"raw":"0x","publicKey":null,"v":0,"r":"0x0000000000000000000000000000000000000000000000000000000000000000","s":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#);
	}
}

