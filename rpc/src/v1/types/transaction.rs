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
	/// The network id of the transaction, if any.
	#[serde(rename="networkId")]
	pub network_id: Option<u64>,
	/// The standardised V field of the signature (0 or 1).
	#[serde(rename="standardV")]
	pub standard_v: U256,
	/// The standardised V field of the signature.
	pub v: U256,
	/// The R field of the signature.
	pub r: U256,
	/// The S field of the signature.
	pub s: U256,
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
			network_id: t.network_id(),
			standard_v: t.standard_v().into(),
			v: t.original_v().into(),
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
			network_id: t.network_id(),
			standard_v: t.standard_v().into(),
			v: t.original_v().into(),
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
		assert_eq!(serialized, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0","blockHash":null,"blockNumber":null,"transactionIndex":null,"from":"0x0000000000000000000000000000000000000000","to":null,"value":"0x0","gasPrice":"0x0","gas":"0x0","input":"0x","creates":null,"raw":"0x","publicKey":null,"networkId":null,"standardV":"0x0","v":"0x0","r":"0x0","s":"0x0"}"#);
	}

	#[test]
	fn test_local_transaction_status_serialize() {
		let tx_ser = serde_json::to_string(&Transaction::default()).unwrap();
		let status1 = LocalTransactionStatus::Pending;
		let status2 = LocalTransactionStatus::Future;
		let status3 = LocalTransactionStatus::Mined(Transaction::default());
		let status4 = LocalTransactionStatus::Dropped(Transaction::default());
		let status5 = LocalTransactionStatus::Invalid(Transaction::default());
		let status6 = LocalTransactionStatus::Rejected(Transaction::default(), "Just because".into());
		let status7 = LocalTransactionStatus::Replaced(Transaction::default(), 5.into(), 10.into());

		assert_eq!(
			serde_json::to_string(&status1).unwrap(),
			r#"{"status":"pending"}"#
		);
		assert_eq!(
			serde_json::to_string(&status2).unwrap(),
			r#"{"status":"future"}"#
		);
		assert_eq!(
			serde_json::to_string(&status3).unwrap(),
			r#"{"status":"mined","transaction":"#.to_owned() + &format!("{}", tx_ser) + r#"}"#
		);
		assert_eq!(
			serde_json::to_string(&status4).unwrap(),
			r#"{"status":"dropped","transaction":"#.to_owned() + &format!("{}", tx_ser) + r#"}"#
		);
		assert_eq!(
			serde_json::to_string(&status5).unwrap(),
			r#"{"status":"invalid","transaction":"#.to_owned() + &format!("{}", tx_ser) + r#"}"#
		);
		assert_eq!(
			serde_json::to_string(&status6).unwrap(),
			r#"{"status":"rejected","transaction":"#.to_owned() +
			&format!("{}", tx_ser) +
			r#","error":"Just because"}"#
		);
		assert_eq!(
			serde_json::to_string(&status7).unwrap(),
			r#"{"status":"replaced","transaction":"#.to_owned() +
			&format!("{}", tx_ser) +
			r#","hash":"0x000000000000000000000000000000000000000000000000000000000000000a","gasPrice":"0x5"}"#
		);
>>>>>>> 436016e... Introduce to_hex() utility in bigint. Fix tests.
	}
}

