// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use machine::executive::{contract_address};
use vm::CreateContractAddress;
use ethereum_types::{H160, H256, H512, U64, U256};
use miner;
use types::transaction::{LocalizedTransaction, Action, PendingTransaction, SignedTransaction};
use v1::types::{Bytes, TransactionCondition};

/// Transaction
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
	/// Hash
	pub hash: H256,
	/// Nonce
	pub nonce: U256,
	/// Block hash
	pub block_hash: Option<H256>,
	/// Block number
	pub block_number: Option<U256>,
	/// Transaction Index
	pub transaction_index: Option<U256>,
	/// Sender
	pub from: H160,
	/// Recipient
	pub to: Option<H160>,
	/// Transfered value
	pub value: U256,
	/// Gas Price
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
	pub public_key: Option<H512>,
	/// The network id of the transaction, if any.
	pub chain_id: Option<U64>,
	/// The standardised V field of the signature (0 or 1).
	pub standard_v: U256,
	/// The standardised V field of the signature.
	pub v: U256,
	/// The R field of the signature.
	pub r: U256,
	/// The S field of the signature.
	pub s: U256,
	/// Transaction activates at specified block.
	pub condition: Option<TransactionCondition>,
}

/// Local Transaction Status
#[derive(Debug)]
pub enum LocalTransactionStatus {
	/// Transaction is pending
	Pending,
	/// Transaction is in future part of the queue
	Future,
	/// Transaction was mined.
	Mined(Transaction),
	/// Transaction was removed from the queue, but not mined.
	Culled(Transaction),
	/// Transaction was dropped because of limit.
	Dropped(Transaction),
	/// Transaction was replaced by transaction with higher gas price.
	Replaced(Transaction, U256, H256),
	/// Transaction never got into the queue.
	Rejected(Transaction, String),
	/// Transaction is invalid.
	Invalid(Transaction),
	/// Transaction was canceled.
	Canceled(Transaction),
}

impl Serialize for LocalTransactionStatus {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer
	{
		use self::LocalTransactionStatus::*;

		let elems = match *self {
			Pending | Future => 1,
			Mined(..) | Culled(..) | Dropped(..) | Invalid(..) | Canceled(..) => 2,
			Rejected(..) => 3,
			Replaced(..) => 4,
		};

		let status = "status";
		let transaction = "transaction";

		let mut struc = serializer.serialize_struct("LocalTransactionStatus", elems)?;
		match *self {
			Pending => struc.serialize_field(status, "pending")?,
			Future => struc.serialize_field(status, "future")?,
			Mined(ref tx) => {
				struc.serialize_field(status, "mined")?;
				struc.serialize_field(transaction, tx)?;
			},
			Culled(ref tx) => {
				struc.serialize_field(status, "culled")?;
				struc.serialize_field(transaction, tx)?;
			},
			Dropped(ref tx) => {
				struc.serialize_field(status, "dropped")?;
				struc.serialize_field(transaction, tx)?;
			},
			Canceled(ref tx) => {
				struc.serialize_field(status, "canceled")?;
				struc.serialize_field(transaction, tx)?;
			},
			Invalid(ref tx) => {
				struc.serialize_field(status, "invalid")?;
				struc.serialize_field(transaction, tx)?;
			},
			Rejected(ref tx, ref reason) => {
				struc.serialize_field(status, "rejected")?;
				struc.serialize_field(transaction, tx)?;
				struc.serialize_field("error", reason)?;
			},
			Replaced(ref tx, ref gas_price, ref hash) => {
				struc.serialize_field(status, "replaced")?;
				struc.serialize_field(transaction, tx)?;
				struc.serialize_field("hash", hash)?;
				struc.serialize_field("gasPrice", gas_price)?;
			},
		}

		struc.end()
	}
}

/// Geth-compatible output for eth_signTransaction method
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct RichRawTransaction {
	/// Raw transaction RLP
	pub raw: Bytes,
	/// Transaction details
	#[serde(rename = "tx")]
	pub transaction: Transaction
}

impl RichRawTransaction {
	/// Creates new `RichRawTransaction` from `SignedTransaction`.
	pub fn from_signed(tx: SignedTransaction) -> Self {
		let tx = Transaction::from_signed(tx);
		RichRawTransaction {
			raw: tx.raw.clone(),
			transaction: tx,
		}
	}
}

impl Transaction {
	/// Convert `LocalizedTransaction` into RPC Transaction.
	pub fn from_localized(mut t: LocalizedTransaction) -> Transaction {
		let signature = t.signature();
		let scheme = CreateContractAddress::FromSenderAndNonce;
		Transaction {
			hash: t.hash(),
			nonce: t.nonce,
			block_hash: Some(t.block_hash),
			block_number: Some(t.block_number.into()),
			transaction_index: Some(t.transaction_index.into()),
			from: t.sender(),
			to: match t.action {
				Action::Create => None,
				Action::Call(ref address) => Some(*address)
			},
			value: t.value,
			gas_price: t.gas_price,
			gas: t.gas,
			input: Bytes::new(t.data.clone()),
			creates: match t.action {
				Action::Create => Some(contract_address(scheme, &t.sender(), &t.nonce, &t.data).0),
				Action::Call(_) => None,
			},
			raw: ::rlp::encode(&t.signed).into(),
			public_key: t.recover_public().ok().map(Into::into),
			chain_id: t.chain_id().map(U64::from),
			standard_v: t.standard_v().into(),
			v: t.original_v().into(),
			r: signature.r().into(),
			s: signature.s().into(),
			condition: None,
		}
	}

	/// Convert `SignedTransaction` into RPC Transaction.
	pub fn from_signed(t: SignedTransaction) -> Transaction {
		let signature = t.signature();
		let scheme = CreateContractAddress::FromSenderAndNonce;
		Transaction {
			hash: t.hash(),
			nonce: t.nonce,
			block_hash: None,
			block_number: None,
			transaction_index: None,
			from: t.sender(),
			to: match t.action {
				Action::Create => None,
				Action::Call(ref address) => Some(*address)
			},
			value: t.value,
			gas_price: t.gas_price,
			gas: t.gas,
			input: Bytes::new(t.data.clone()),
			creates: match t.action {
				Action::Create => Some(contract_address(scheme, &t.sender(), &t.nonce, &t.data).0),
				Action::Call(_) => None,
			},
			raw: ::rlp::encode(&t).into(),
			public_key: t.public_key().map(Into::into),
			chain_id: t.chain_id().map(U64::from),
			standard_v: t.standard_v().into(),
			v: t.original_v().into(),
			r: signature.r().into(),
			s: signature.s().into(),
			condition: None,
		}
	}

	/// Convert `PendingTransaction` into RPC Transaction.
	pub fn from_pending(t: PendingTransaction) -> Transaction {
		let mut r = Transaction::from_signed(t.transaction);
		r.condition = r.condition.map(Into::into);
		r
	}
}

impl LocalTransactionStatus {
	/// Convert `LocalTransactionStatus` into RPC `LocalTransactionStatus`.
	pub fn from(s: miner::pool::local_transactions::Status) -> Self {
		let convert = |tx: Arc<miner::pool::VerifiedTransaction>| {
			Transaction::from_signed(tx.signed().clone())
		};
		use miner::pool::local_transactions::Status::*;
		match s {
			Pending(_) => LocalTransactionStatus::Pending,
			Mined(tx) => LocalTransactionStatus::Mined(convert(tx)),
			Culled(tx) => LocalTransactionStatus::Culled(convert(tx)),
			Dropped(tx) => LocalTransactionStatus::Dropped(convert(tx)),
			Rejected(tx, reason) => LocalTransactionStatus::Rejected(convert(tx), reason),
			Invalid(tx) => LocalTransactionStatus::Invalid(convert(tx)),
			Canceled(tx) => LocalTransactionStatus::Canceled(convert(tx)),
			Replaced { old, new } => LocalTransactionStatus::Replaced(
				convert(old),
				new.signed().gas_price,
				new.signed().hash(),
			),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{Transaction, LocalTransactionStatus};
	use serde_json;

	#[test]
	fn test_transaction_serialize() {
		let t = Transaction::default();
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0","blockHash":null,"blockNumber":null,"transactionIndex":null,"from":"0x0000000000000000000000000000000000000000","to":null,"value":"0x0","gasPrice":"0x0","gas":"0x0","input":"0x","creates":null,"raw":"0x","publicKey":null,"chainId":null,"standardV":"0x0","v":"0x0","r":"0x0","s":"0x0","condition":null}"#);
	}

	#[test]
	fn test_local_transaction_status_serialize() {
		use ethereum_types::H256;

		let tx_ser = serde_json::to_string(&Transaction::default()).unwrap();
		let status1 = LocalTransactionStatus::Pending;
		let status2 = LocalTransactionStatus::Future;
		let status3 = LocalTransactionStatus::Mined(Transaction::default());
		let status4 = LocalTransactionStatus::Dropped(Transaction::default());
		let status5 = LocalTransactionStatus::Invalid(Transaction::default());
		let status6 = LocalTransactionStatus::Rejected(Transaction::default(), "Just because".into());
		let status7 = LocalTransactionStatus::Replaced(Transaction::default(), 5.into(), H256::from_low_u64_be(10));

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
	}
}
