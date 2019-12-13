// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

use std::sync::{atomic, Arc};

use ethereum_types::{U256, H256, Address};
use rlp::Rlp;
use types::transaction::{self, Transaction, SignedTransaction, UnverifiedTransaction};

use pool;
use pool::client::AccountDetails;

const MAX_TRANSACTION_SIZE: usize = 15 * 1024;

#[derive(Debug, Clone)]
pub struct TestClient {
	verification_invoked: Arc<atomic::AtomicBool>,
	account_details: AccountDetails,
	gas_required: U256,
	is_service_transaction: bool,
	local_address: Address,
	max_transaction_size: usize,
}

impl Default for TestClient {
	fn default() -> Self {
		TestClient {
			verification_invoked: Default::default(),
			account_details: AccountDetails {
				nonce: 123.into(),
				balance: 63_100.into(),
				is_local: false,
			},
			gas_required: 21_000.into(),
			is_service_transaction: false,
			local_address: Default::default(),
			max_transaction_size: MAX_TRANSACTION_SIZE,
		}
	}
}

impl TestClient {
	pub fn new() -> Self {
		TestClient::default()
	}

	pub fn with_balance<T: Into<U256>>(mut self, balance: T) -> Self {
		self.account_details.balance = balance.into();
		self
	}

	pub fn with_nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
		self.account_details.nonce = nonce.into();
		self
	}

	pub fn with_gas_required<T: Into<U256>>(mut self, gas_required: T) -> Self {
		self.gas_required = gas_required.into();
		self
	}

	pub fn with_local(mut self, address: &Address) -> Self {
		self.local_address = *address;
		self
	}

	pub fn with_service_transaction(mut self) -> Self {
		self.is_service_transaction = true;
		self
	}

	pub fn verify<T: Into<transaction::PendingTransaction>>(&self, tx: T) -> pool::VerifiedTransaction {
		let tx = tx.into();
		pool::VerifiedTransaction {
			hash: tx.hash(),
			sender: tx.sender(),
			priority: pool::Priority::Regular,
			transaction: tx,
			insertion_id: 1,
		}
	}

	pub fn was_verification_triggered(&self) -> bool {
		self.verification_invoked.load(atomic::Ordering::SeqCst)
	}
}

impl pool::client::Client for TestClient {
	fn transaction_already_included(&self, _hash: &H256) -> bool {
		false
	}

	fn verify_transaction_basic(&self, _tx: &UnverifiedTransaction)
		-> Result<(), transaction::Error>
	{
		Ok(())
	}

	fn verify_transaction(&self, tx: UnverifiedTransaction)
		-> Result<SignedTransaction, transaction::Error>
	{
		self.verification_invoked.store(true, atomic::Ordering::SeqCst);
		Ok(SignedTransaction::new(tx)?)
	}

	fn account_details(&self, address: &Address) -> AccountDetails {
		let mut details = self.account_details.clone();
		if address == &self.local_address {
			details.is_local = true;
		}

		details
	}

	fn required_gas(&self, _tx: &Transaction) -> U256 {
		self.gas_required
	}

	fn transaction_type(&self, _tx: &SignedTransaction) -> pool::client::TransactionType {
		if self.is_service_transaction {
			pool::client::TransactionType::Service
		} else {
			pool::client::TransactionType::Regular
		}
	}

	fn decode_transaction(&self, transaction: &[u8]) -> Result<UnverifiedTransaction, transaction::Error> {
		let rlp = Rlp::new(&transaction);
		if rlp.as_raw().len() > self.max_transaction_size {
			return Err(transaction::Error::TooBig)
		}
		rlp.as_val().map_err(|e| transaction::Error::InvalidRlp(e.to_string()))
	}

}

impl pool::client::NonceClient for TestClient {
	fn account_nonce(&self, _address: &Address) -> U256 {
		self.account_details.nonce
	}
}
