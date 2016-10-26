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

//! Types used in Confirmations queue (Trusted Signer)

use v1::types::{U256, TransactionRequest, H160, H256, Bytes};
use v1::helpers;


/// Confirmation waiting in a queue
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct ConfirmationRequest {
	/// Id of this confirmation
	pub id: U256,
	/// Payload
	pub payload: ConfirmationPayload,
}

impl From<helpers::ConfirmationRequest> for ConfirmationRequest {
	fn from(c: helpers::ConfirmationRequest) -> Self {
		ConfirmationRequest {
			id: c.id.into(),
			payload: c.payload.into(),
		}
	}
}

/// Sign request
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct SignRequest {
	/// Address
	pub address: H160,
	/// Hash to sign
	pub hash: H256,
}

/// Decrypt request
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct DecryptRequest {
	/// Address
	pub address: H160,
	/// Message to decrypt
	pub msg: Bytes,
}

/// Confirmation payload, i.e. the thing to be confirmed
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub enum ConfirmationPayload {
	/// Transaction
	#[serde(rename="transaction")]
	Transaction(TransactionRequest),
	/// Signature
	#[serde(rename="sign")]
	Sign(SignRequest),
	/// Decryption
	#[serde(rename="decrypt")]
	Decrypt(DecryptRequest),
}

impl From<helpers::ConfirmationPayload> for ConfirmationPayload {
	fn from(c: helpers::ConfirmationPayload) -> Self {
		match c {
			helpers::ConfirmationPayload::Transaction(t) => ConfirmationPayload::Transaction(t.into()),
			helpers::ConfirmationPayload::Sign(address, hash) => ConfirmationPayload::Sign(SignRequest {
				address: address.into(),
				hash: hash.into(),
			}),
			helpers::ConfirmationPayload::Decrypt(address, msg) => ConfirmationPayload::Decrypt(DecryptRequest {
				address: address.into(),
				msg: msg.into(),
			}),
		}
	}
}

/// Possible modifications to the confirmed transaction sent by `Trusted Signer`
#[derive(Debug, PartialEq, Deserialize)]
pub struct TransactionModification {
	/// Modified gas price
	#[serde(rename="gasPrice")]
	pub gas_price: Option<U256>,
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use serde_json;
	use v1::types::U256;
	use v1::helpers;
	use super::*;

	#[test]
	fn should_serialize_sign_confirmation() {
		// given
		let request = helpers::ConfirmationRequest {
			id: 15.into(),
			payload: helpers::ConfirmationPayload::Sign(1.into(), 5.into()),
		};

		// when
		let res = serde_json::to_string(&ConfirmationRequest::from(request));
		let expected = r#"{"id":"0xf","payload":{"sign":{"address":"0x0000000000000000000000000000000000000001","hash":"0x0000000000000000000000000000000000000000000000000000000000000005"}}}"#;

		// then
		assert_eq!(res.unwrap(), expected.to_owned());
	}

	#[test]
	fn should_serialize_transaction_confirmation() {
		// given
		let request = helpers::ConfirmationRequest {
			id: 15.into(),
			payload: helpers::ConfirmationPayload::Transaction(helpers::FilledTransactionRequest {
				from: 0.into(),
				to: None,
				gas: 15_000.into(),
				gas_price: 10_000.into(),
				value: 100_000.into(),
				data: vec![1, 2, 3],
				nonce: Some(1.into()),
			}),
		};

		// when
		let res = serde_json::to_string(&ConfirmationRequest::from(request));
		let expected = r#"{"id":"0xf","payload":{"transaction":{"from":"0x0000000000000000000000000000000000000000","to":null,"gasPrice":"0x2710","gas":"0x3a98","value":"0x186a0","data":"0x010203","nonce":"0x1"}}}"#;

		// then
		assert_eq!(res.unwrap(), expected.to_owned());
	}

	#[test]
	fn should_deserialize_modification() {
		// given
		let s1 = r#"{
			"gasPrice":"0xba43b7400"
		}"#;
		let s2 = r#"{}"#;

		// when
		let res1: TransactionModification = serde_json::from_str(s1).unwrap();
		let res2: TransactionModification = serde_json::from_str(s2).unwrap();

		// then
		assert_eq!(res1, TransactionModification {
			gas_price: Some(U256::from_str("0ba43b7400").unwrap()),
		});
		assert_eq!(res2, TransactionModification {
			gas_price: None,
		});
	}
}

