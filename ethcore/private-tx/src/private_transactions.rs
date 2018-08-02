// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use bytes::Bytes;
use ethcore_miner::pool;
use ethereum_types::{H256, U256, Address};
use ethkey::Signature;
use transaction::{UnverifiedTransaction, SignedTransaction};

use error::{Error, ErrorKind};

/// Maximum length for private transactions queues.
const MAX_QUEUE_LEN: usize = 8312;

/// Desriptor for private transaction stored in queue for verification
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PrivateTransactionDesc {
	/// Hash of the private transaction
	pub private_hash: H256,
	/// Contract's address used in private transaction
	pub contract: Address,
	/// Address that should be used for verification
	pub validator_account: Address,
}

/// Storage for private transactions for verification
pub struct VerificationStore {
	/// Descriptors for private transactions in queue for verification with key - hash of the original transaction
	descriptors: HashMap<H256, PrivateTransactionDesc>,
	/// Queue with transactions for verification
	///
	/// TODO [ToDr] Might actually be better to use `txpool` directly and:
	/// 1. Store descriptors inside `VerifiedTransaction`
	/// 2. Use custom `ready` implementation to only fetch one transaction per sender.
	/// 3. Get rid of passing dummy `block_number` and `timestamp`
	transactions: pool::TransactionQueue,
}

impl Default for VerificationStore {
	fn default() -> Self {
		VerificationStore {
			descriptors: Default::default(),
			transactions: pool::TransactionQueue::new(
				pool::Options {
					max_count: MAX_QUEUE_LEN,
					max_per_sender: MAX_QUEUE_LEN / 10,
					max_mem_usage: 8 * 1024 * 1024,
				},
				pool::verifier::Options {
					// TODO [ToDr] This should probably be based on some real values?
					minimal_gas_price: 0.into(),
					block_gas_limit: 8_000_000.into(),
					tx_gas_limit: U256::max_value(),
					no_early_reject: false
				},
				pool::PrioritizationStrategy::GasPriceOnly,
			)
		}
	}
}

impl VerificationStore {
	/// Adds private transaction for verification into the store
	pub fn add_transaction<C: pool::client::Client>(
		&mut self,
		transaction: UnverifiedTransaction,
		contract: Address,
		validator_account: Address,
		private_hash: H256,
		client: C,
	) -> Result<(), Error> {
		if self.descriptors.len() > MAX_QUEUE_LEN {
			bail!(ErrorKind::QueueIsFull);
		}

		let transaction_hash = transaction.hash();
		if self.descriptors.get(&transaction_hash).is_some() {
			bail!(ErrorKind::PrivateTransactionAlreadyImported);
		}

		let results = self.transactions.import(
			client,
			vec![pool::verifier::Transaction::Unverified(transaction)],
		);

		// Verify that transaction was imported
		results.into_iter()
			.next()
			.expect("One transaction inserted; one result returned; qed")?;

		self.descriptors.insert(transaction_hash, PrivateTransactionDesc {
			private_hash,
			contract,
			validator_account,
		});

		Ok(())
	}

	/// Returns transactions ready for verification
	/// Returns only one transaction per sender because several cannot be verified in a row without verification from other peers
	pub fn ready_transactions<C: pool::client::NonceClient>(&self, client: C) -> Vec<Arc<pool::VerifiedTransaction>> {
		// We never store PendingTransactions and we don't use internal cache,
		// so we don't need to provide real block number of timestamp here
		let block_number = 0;
		let timestamp = 0;
		let nonce_cap = None;

		self.transactions.collect_pending(client, block_number, timestamp, nonce_cap, |transactions| {
			// take only one transaction per sender
			let mut senders = HashSet::with_capacity(self.descriptors.len());
			transactions.filter(move |tx| senders.insert(tx.signed().sender())).collect()
		})
	}

	/// Returns descriptor of the corresponding private transaction
	pub fn private_transaction_descriptor(&self, transaction_hash: &H256) -> Result<&PrivateTransactionDesc, Error> {
		self.descriptors.get(transaction_hash).ok_or(ErrorKind::PrivateTransactionNotFound.into())
	}

	/// Remove transaction from the queue for verification
	pub fn remove_private_transaction(&mut self, transaction_hash: &H256) {
		self.descriptors.remove(transaction_hash);
		self.transactions.remove(&[*transaction_hash], true);
	}
}

/// Desriptor for private transaction stored in queue for signing
#[derive(Debug, Clone)]
pub struct PrivateTransactionSigningDesc {
	/// Original unsigned transaction
	pub original_transaction: SignedTransaction,
	/// Supposed validators from the contract
	pub validators: Vec<Address>,
	/// Already obtained signatures
	pub received_signatures: Vec<Signature>,
	/// State after transaction execution to compare further with received from validators
	pub state: Bytes,
	/// Build-in nonce of the contract
	pub contract_nonce: U256,
}

/// Storage for private transactions for signing
#[derive(Default)]
pub struct SigningStore {
	/// Transactions and descriptors for signing
	transactions: HashMap<H256, PrivateTransactionSigningDesc>,
}

impl SigningStore {
	/// Adds new private transaction into the store for signing
	pub fn add_transaction(
		&mut self,
		private_hash: H256,
		transaction: SignedTransaction,
		validators: Vec<Address>,
		state: Bytes,
		contract_nonce: U256,
	) -> Result<(), Error> {
		if self.transactions.len() > MAX_QUEUE_LEN {
			bail!(ErrorKind::QueueIsFull);
		}

		self.transactions.insert(private_hash, PrivateTransactionSigningDesc {
			original_transaction: transaction.clone(),
			validators: validators.clone(),
			received_signatures: Vec::new(),
			state,
			contract_nonce,
		});
		Ok(())
	}

	/// Get copy of private transaction's description from the storage
	pub fn get(&self, private_hash: &H256) -> Option<PrivateTransactionSigningDesc> {
		self.transactions.get(private_hash).cloned()
	}

	/// Removes desc from the store (after verification is completed)
	pub fn remove(&mut self, private_hash: &H256) -> Result<(), Error> {
		self.transactions.remove(private_hash);
		Ok(())
	}

	/// Adds received signature for the stored private transaction
	pub fn add_signature(&mut self, private_hash: &H256, signature: Signature) -> Result<(), Error> {
		let desc = self.transactions.get_mut(private_hash).ok_or_else(|| ErrorKind::PrivateTransactionNotFound)?;
		if !desc.received_signatures.contains(&signature) {
			desc.received_signatures.push(signature);
		}
		Ok(())
	}
}
