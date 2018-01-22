// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Privte transaction signing RPC implementation.

use std::sync::Arc;

use rlp::UntrustedRlp;

use ethcore::private_transactions::Provider as PrivateTransactionManager;
use ethereum_types::Address;
use ethcore::{contract_address, CreateContractAddress};
use transaction::SignedTransaction;

use jsonrpc_core::{Error};
use v1::types::{Bytes, PrivateTransactionReceipt, H160, H256, TransactionRequest, U256,
	BlockNumber, PrivateTransactionReceiptAndTransaction, CallRequest};
use v1::traits::Private;
use v1::helpers::{errors, fake_sign};

/// Private transaction manager API endpoint implementation.
pub struct PrivateClient {
	private: Option<Arc<PrivateTransactionManager>>,
}

impl PrivateClient {
	/// Creates a new instance.
	pub fn new(private: &Option<Arc<PrivateTransactionManager>>) -> Self {
		PrivateClient {
			private: private.clone(),
		}
	}

	fn unwrap_manager(&self) -> Result<Arc<PrivateTransactionManager>, Error> {
		match self.private {
			Some(ref arc) => Ok(arc.clone()),
			None => Err(errors::public_unsupported(None)),
		}
	}
}

impl Private for PrivateClient {
	fn send_transaction(&self, request: Bytes) -> Result<PrivateTransactionReceipt, Error> {
		let signed_transaction = UntrustedRlp::new(&request.into_vec()).as_val()
			.map_err(errors::rlp)
			.and_then(|tx| SignedTransaction::new(tx).map_err(errors::transaction))?;
		let client = self.unwrap_manager()?;
		let receipt = client.create_private_transaction(signed_transaction).map_err(errors::transaction)?;
		Ok(receipt.into())
	}

	fn compose_deployment_transaction(&self, num: BlockNumber, request: Bytes, validators: Vec<H160>, gas_price: U256) -> Result<PrivateTransactionReceiptAndTransaction, Error> {
		let signed_transaction = UntrustedRlp::new(&request.into_vec()).as_val()
			.map_err(errors::rlp)
			.and_then(|tx| SignedTransaction::new(tx).map_err(errors::transaction))?;
		let client = self.unwrap_manager()?;

		let addresses: Vec<Address> = validators.into_iter().map(Into::into).collect();

		let transaction = client.public_creation_transaction(num.into(), &signed_transaction, addresses.as_slice(), gas_price.into()).map_err(errors::transaction)?;
		let tx_hash = transaction.hash(None);
		let request = TransactionRequest {
			from: Some(signed_transaction.sender().into()),
			to: None,
			nonce: Some(transaction.nonce.into()),
			gas_price: Some(transaction.gas_price.into()),
			gas: Some(transaction.gas.into()),
			value: Some(transaction.value.into()),
			data: Some(transaction.data.into()),
			condition: None,
		};
		let (contract_address, _) = contract_address(CreateContractAddress::FromSenderAndNonce, &signed_transaction.sender().clone(), &transaction.nonce.into(), &[]);
		Ok(PrivateTransactionReceiptAndTransaction {
			transaction: request,
			receipt: PrivateTransactionReceipt {
				transaction_hash: tx_hash.into(),
				contract_address: Some(contract_address.into()),
				status_code: 0,
			}
		})
	}

	fn private_call(&self, num: BlockNumber, request: CallRequest) -> Result<Bytes, Error> {
		let request = CallRequest::into(request);
		let signed = fake_sign::sign_call(request, true)?;
		let client = self.unwrap_manager()?;
		let executed_result = client.private_call(num.into(), &signed).map_err(errors::transaction)?;
		Ok(executed_result.output.into())
	}

	fn private_contract_key(&self, contract_address: H160) -> Result<H256, Error> {
		let client = self.unwrap_manager()?;
		let key = client.contract_key_id(&contract_address.into()).map_err(errors::transaction)?;
		Ok(key.into())
	}
}

