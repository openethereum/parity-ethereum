// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Types used in Confirmations queue (Trusted Signer)

use std::fmt;
use serde::{Serialize, Serializer};
use ansi_term::Colour;
use bytes::ToPretty;

use ethereum_types::{H160, H256, H520, U256};
use v1::types::{TransactionRequest, RichRawTransaction, Bytes, TransactionCondition, Origin};
use v1::helpers;
use ethkey::Password;

/// Confirmation waiting in a queue
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmationRequest {
	/// Id of this confirmation
	pub id: U256,
	/// Payload
	pub payload: ConfirmationPayload,
	/// Request origin
	pub origin: Origin,
}

impl From<helpers::ConfirmationRequest> for ConfirmationRequest {
	fn from(c: helpers::ConfirmationRequest) -> Self {
		ConfirmationRequest {
			id: c.id,
			payload: c.payload.into(),
			origin: c.origin,
		}
	}
}

impl fmt::Display for ConfirmationRequest {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "#{}: {} coming from {}", self.id, self.payload, self.origin)
	}
}

impl fmt::Display for ConfirmationPayload {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ConfirmationPayload::SendTransaction(ref transaction) => write!(f, "{}", transaction),
			ConfirmationPayload::SignTransaction(ref transaction) => write!(f, "(Sign only) {}", transaction),
			ConfirmationPayload::EthSignMessage(ref sign) => write!(f, "{}", sign),
			ConfirmationPayload::EIP191SignMessage(ref sign) => write!(f, "{}", sign),
			ConfirmationPayload::Decrypt(ref decrypt) => write!(f, "{}", decrypt),
		}
	}
}

/// Ethereum-prefixed Sign request
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EthSignRequest {
	/// Address
	pub address: H160,
	/// Hash to sign
	pub data: Bytes,
}

/// EIP191 Sign request
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EIP191SignRequest {
	/// Address
	pub address: H160,
	/// Hash to sign
	pub data: H256,
}

impl From<(H160, H256)> for EIP191SignRequest {
	fn from(tuple: (H160, H256)) -> Self {
		EIP191SignRequest {
			address: tuple.0,
			data: tuple.1,
		}
	}
}

impl fmt::Display for EIP191SignRequest {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"sign 0x{} with {}",
			self.data.0.pretty(),
			Colour::White.bold().paint(format!("0x{:?}", self.address)),
		)
	}
}

impl From<(H160, Bytes)> for EthSignRequest {
	fn from(tuple: (H160, Bytes)) -> Self {
		EthSignRequest {
			address: tuple.0,
			data: tuple.1,
		}
	}
}

impl fmt::Display for EthSignRequest {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"sign 0x{} with {}",
			self.data.0.pretty(),
			Colour::White.bold().paint(format!("0x{:?}", self.address)),
		)
	}
}

/// Decrypt request
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecryptRequest {
	/// Address
	pub address: H160,
	/// Message to decrypt
	pub msg: Bytes,
}

impl From<(H160, Bytes)> for DecryptRequest {
	fn from(tuple: (H160, Bytes)) -> Self {
		DecryptRequest {
			address: tuple.0,
			msg: tuple.1,
		}
	}
}

impl fmt::Display for DecryptRequest {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"decrypt data with {}",
			Colour::White.bold().paint(format!("0x{:?}", self.address)),
		)
	}
}

/// Confirmation response for particular payload
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationResponse {
	/// Transaction Hash
	SendTransaction(H256),
	/// Transaction RLP
	SignTransaction(RichRawTransaction),
	/// Signature (encoded as VRS)
	Signature(H520),
	/// Decrypted data
	Decrypt(Bytes),
}

impl Serialize for ConfirmationResponse {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer
	{
		match *self {
			ConfirmationResponse::SendTransaction(ref hash) => hash.serialize(serializer),
			ConfirmationResponse::SignTransaction(ref rlp) => rlp.serialize(serializer),
			ConfirmationResponse::Signature(ref signature) => signature.serialize(serializer),
			ConfirmationResponse::Decrypt(ref data) => data.serialize(serializer),
		}
	}
}

/// Confirmation response with additional token for further requests
#[derive(Clone, PartialEq, Serialize)]
pub struct ConfirmationResponseWithToken {
	/// Actual response
	pub result: ConfirmationResponse,
	/// New token
	pub token: Password,
}

/// Confirmation payload, i.e. the thing to be confirmed
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationPayload {
	/// Send Transaction
	SendTransaction(TransactionRequest),
	/// Sign Transaction
	SignTransaction(TransactionRequest),
	/// Signature
	#[serde(rename = "sign")]
	EthSignMessage(EthSignRequest),
	/// signature without prefix
	EIP191SignMessage(EIP191SignRequest),
	/// Decryption
	Decrypt(DecryptRequest),
}

impl From<helpers::ConfirmationPayload> for ConfirmationPayload {
	fn from(c: helpers::ConfirmationPayload) -> Self {
		match c {
			helpers::ConfirmationPayload::SendTransaction(t) => ConfirmationPayload::SendTransaction(t.into()),
			helpers::ConfirmationPayload::SignTransaction(t) => ConfirmationPayload::SignTransaction(t.into()),
			helpers::ConfirmationPayload::EthSignMessage(address, data) => ConfirmationPayload::EthSignMessage(EthSignRequest {
				address,
				data: data.into(),
			}),
			helpers::ConfirmationPayload::SignMessage(address, data) => ConfirmationPayload::EIP191SignMessage(EIP191SignRequest {
				address,
				data,
			}),
			helpers::ConfirmationPayload::Decrypt(address, msg) => ConfirmationPayload::Decrypt(DecryptRequest {
				address,
				msg: msg.into(),
			}),
		}
	}
}

impl ConfirmationPayload {
	pub fn sender(&self) -> Option<&H160> {
		match *self {
			ConfirmationPayload::SendTransaction(ref request) => request.from.as_ref(),
			ConfirmationPayload::SignTransaction(ref request) => request.from.as_ref(),
			ConfirmationPayload::EthSignMessage(ref request) => Some(&request.address),
			ConfirmationPayload::EIP191SignMessage(ref request) => Some(&request.address),
			ConfirmationPayload::Decrypt(ref request) => Some(&request.address),
		}
	}
}

/// Possible modifications to the confirmed transaction sent by `Trusted Signer`
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TransactionModification {
	/// Modified transaction sender
	pub sender: Option<H160>,
	/// Modified gas price
	pub gas_price: Option<U256>,
	/// Modified gas
	pub gas: Option<U256>,
	/// Modified transaction condition.
	pub condition: Option<Option<TransactionCondition>>,
}

/// Represents two possible return values.
#[derive(Debug, Clone)]
pub enum Either<A, B> where
	A: fmt::Debug + Clone,
	B: fmt::Debug + Clone,
{
	/// Primary value
	Either(A),
	/// Secondary value
	Or(B),
}

impl<A, B> From<A> for Either<A, B> where
	A: fmt::Debug + Clone,
	B: fmt::Debug + Clone,
{
	fn from(a: A) -> Self {
		Either::Either(a)
	}
}

impl<A, B> Serialize for Either<A, B>  where
	A: Serialize + fmt::Debug + Clone,
	B: Serialize + fmt::Debug + Clone,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer
	{
		match *self {
			Either::Either(ref a) => a.serialize(serializer),
			Either::Or(ref b) => b.serialize(serializer),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
    use ethereum_types::{H256, U256, Address};
	use serde_json;
	use v1::types::TransactionCondition;
	use v1::helpers;
	use super::*;

	#[test]
	fn should_serialize_sign_confirmation() {
		// given
		let request = helpers::ConfirmationRequest {
			id: 15.into(),
			payload: helpers::ConfirmationPayload::EthSignMessage(Address::from_low_u64_be(1), vec![5].into()),
			origin: Origin::Rpc("test service".into()),
		};

		// when
		let res = serde_json::to_string(&ConfirmationRequest::from(request));
		let expected = r#"{"id":"0xf","payload":{"sign":{"address":"0x0000000000000000000000000000000000000001","data":"0x05"}},"origin":{"rpc":"test service"}}"#;

		// then
		assert_eq!(res.unwrap(), expected.to_owned());
	}

	#[test]
	fn should_serialize_transaction_confirmation() {
		// given
		let request = helpers::ConfirmationRequest {
			id: 15.into(),
			payload: helpers::ConfirmationPayload::SendTransaction(helpers::FilledTransactionRequest {
				from: Address::zero(),
				used_default_from: false,
				to: None,
				gas: 15_000.into(),
				gas_price: 10_000.into(),
				value: 100_000.into(),
				data: vec![1, 2, 3],
				nonce: Some(1.into()),
				condition: None,
			}),
			origin: Origin::Signer {
				session: H256::from_low_u64_be(5),
			}
		};

		// when
		let res = serde_json::to_string(&ConfirmationRequest::from(request));
		let expected = r#"{"id":"0xf","payload":{"sendTransaction":{"from":"0x0000000000000000000000000000000000000000","to":null,"gasPrice":"0x2710","gas":"0x3a98","value":"0x186a0","data":"0x010203","nonce":"0x1","condition":null}},"origin":{"signer":{"session":"0x0000000000000000000000000000000000000000000000000000000000000005"}}}"#;

		// then
		assert_eq!(res.unwrap(), expected.to_owned());
	}

	#[test]
	fn should_serialize_sign_transaction_confirmation() {
		// given
		let request = helpers::ConfirmationRequest {
			id: 15.into(),
			payload: helpers::ConfirmationPayload::SignTransaction(helpers::FilledTransactionRequest {
				from: Address::zero(),
				used_default_from: false,
				to: None,
				gas: 15_000.into(),
				gas_price: 10_000.into(),
				value: 100_000.into(),
				data: vec![1, 2, 3],
				nonce: Some(1.into()),
				condition: None,
			}),
			origin: Origin::Unknown,
		};

		// when
		let res = serde_json::to_string(&ConfirmationRequest::from(request));
		let expected = r#"{"id":"0xf","payload":{"signTransaction":{"from":"0x0000000000000000000000000000000000000000","to":null,"gasPrice":"0x2710","gas":"0x3a98","value":"0x186a0","data":"0x010203","nonce":"0x1","condition":null}},"origin":"unknown"}"#;

		// then
		assert_eq!(res.unwrap(), expected.to_owned());
	}

	#[test]
	fn should_serialize_decrypt_confirmation() {
		// given
		let request = helpers::ConfirmationRequest {
			id: 15.into(),
			payload: helpers::ConfirmationPayload::Decrypt(
				Address::from_low_u64_be(10), vec![1, 2, 3].into(),
			),
			origin: Default::default(),
		};

		// when
		let res = serde_json::to_string(&ConfirmationRequest::from(request));
		let expected = r#"{"id":"0xf","payload":{"decrypt":{"address":"0x000000000000000000000000000000000000000a","msg":"0x010203"}},"origin":"unknown"}"#;

		// then
		assert_eq!(res.unwrap(), expected.to_owned());
	}

	#[test]
	fn should_deserialize_modification() {
		// given
		let s1 = r#"{
			"sender": "0x000000000000000000000000000000000000000a",
			"gasPrice":"0xba43b7400",
			"condition": { "block": 66 }
		}"#;
		let s2 = r#"{"gas": "0x1233"}"#;
		let s3 = r#"{}"#;

		// when
		let res1: TransactionModification = serde_json::from_str(s1).unwrap();
		let res2: TransactionModification = serde_json::from_str(s2).unwrap();
		let res3: TransactionModification = serde_json::from_str(s3).unwrap();

		// then
		assert_eq!(res1, TransactionModification {
			sender: Some(Address::from_low_u64_be(10)),
			gas_price: Some(U256::from_str("0ba43b7400").unwrap()),
			gas: None,
			condition: Some(Some(TransactionCondition::Number(0x42))),
		});
		assert_eq!(res2, TransactionModification {
			sender: None,
			gas_price: None,
			gas: Some(U256::from_str("1233").unwrap()),
			condition: None,
		});
		assert_eq!(res3, TransactionModification {
			sender: None,
			gas_price: None,
			gas: None,
			condition: None,
		});
	}

	#[test]
	fn should_serialize_confirmation_response_with_token() {
		// given
		let response = ConfirmationResponseWithToken {
			result: ConfirmationResponse::SendTransaction(H256::zero()),
			token: "test-token".into(),
		};

		// when
		let res = serde_json::to_string(&response);
		let expected = r#"{"result":"0x0000000000000000000000000000000000000000000000000000000000000000","token":"test-token"}"#;

		// then
		assert_eq!(res.unwrap(), expected.to_owned());
	}
}
