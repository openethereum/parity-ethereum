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

use std::sync::Arc;
use std::cmp;
use std::collections::{HashMap, HashSet};

use bytes::Bytes;
use ethcore_miner::pool;
use ethereum_types::{H256, U256, Address};
use heapsize::HeapSizeOf;
use ethkey::Signature;
use messages::PrivateTransaction;
use parking_lot::RwLock;
use types::transaction::{UnverifiedTransaction, SignedTransaction};
use txpool;
use txpool::{VerifiedTransaction, Verifier};
use error::Error;

type Pool = txpool::Pool<VerifiedPrivateTransaction, pool::scoring::NonceAndGasPrice>;

/// Maximum length for private transactions queues.
const MAX_QUEUE_LEN: usize = 8312;

/// Private transaction stored in queue for verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPrivateTransaction {
	/// Original private transaction
	pub private_transaction: PrivateTransaction,
	/// Address that should be used for verification
	pub validator_account: Option<Address>,
	/// Resulting verified transaction
	pub transaction: SignedTransaction,
	/// Original transaction hash
	pub transaction_hash: H256,
	/// Original transaction sender
	pub transaction_sender: Address,
}

impl txpool::VerifiedTransaction for VerifiedPrivateTransaction {
	type Hash = H256;
	type Sender = Address;

	fn hash(&self) -> &H256 {
		&self.transaction_hash
	}

	fn mem_usage(&self) -> usize {
		self.transaction.heap_size_of_children()
	}

	fn sender(&self) -> &Address {
		&self.transaction_sender
	}
}

impl pool::ScoredTransaction for VerifiedPrivateTransaction {
	fn priority(&self) -> pool::Priority {
		pool::Priority::Regular
	}

	/// Gets transaction gas price.
	fn gas_price(&self) -> &U256 {
		&self.transaction.gas_price
	}

	/// Gets transaction nonce.
	fn nonce(&self) -> U256 {
		self.transaction.nonce
	}
}

/// Checks readiness of transactions by looking if the transaction from sender already exists.
/// Guarantees only one transaction per sender
#[derive(Debug)]
pub struct PrivateReadyState<C> {
	senders: HashSet<Address>,
	state: C,
}

impl<C> PrivateReadyState<C> {
	/// Create new State checker, given client interface.
	pub fn new(
		state: C,
	) -> Self {
		PrivateReadyState {
			senders: Default::default(),
			state,
		}
	}
}

impl<C: pool::client::NonceClient> txpool::Ready<VerifiedPrivateTransaction> for PrivateReadyState<C> {
	fn is_ready(&mut self, tx: &VerifiedPrivateTransaction) -> txpool::Readiness {
		let sender = tx.sender();
		let state = &self.state;
		let state_nonce = state.account_nonce(sender);
		if self.senders.contains(sender) {
			txpool::Readiness::Future
		} else {
			self.senders.insert(*sender);
			match tx.transaction.nonce.cmp(&state_nonce) {
				cmp::Ordering::Greater => txpool::Readiness::Future,
				cmp::Ordering::Less => txpool::Readiness::Stale,
				cmp::Ordering::Equal => txpool::Readiness::Ready,
			}
		}
	}
}

/// Storage for private transactions for verification
pub struct VerificationStore {
	verification_pool: RwLock<Pool>,
	verification_options: pool::verifier::Options,
}

impl Default for VerificationStore {
	fn default() -> Self {
		VerificationStore {
			verification_pool: RwLock::new(
				txpool::Pool::new(
					txpool::NoopListener,
					pool::scoring::NonceAndGasPrice(pool::PrioritizationStrategy::GasPriceOnly),
					pool::Options {
						max_count: MAX_QUEUE_LEN,
						max_per_sender: MAX_QUEUE_LEN / 10,
						max_mem_usage: 8 * 1024 * 1024,
					},
				)
			),
			verification_options: pool::verifier::Options {
				// TODO [ToDr] This should probably be based on some real values?
				minimal_gas_price: 0.into(),
				block_gas_limit: 8_000_000.into(),
				tx_gas_limit: U256::max_value(),
				no_early_reject: false,
			},
		}
	}
}

impl VerificationStore {
	/// Adds private transaction for verification into the store
	pub fn add_transaction<C: pool::client::Client>(
		&self,
		transaction: UnverifiedTransaction,
		validator_account: Option<Address>,
		private_transaction: PrivateTransaction,
		client: C,
	) -> Result<(), Error> {

		let options = self.verification_options.clone();
		// Use pool's verifying pipeline for original transaction's verification
		let verifier = pool::verifier::Verifier::new(client, options, Default::default(), None);
		let unverified = pool::verifier::Transaction::Unverified(transaction);
		let verified_tx = verifier.verify_transaction(unverified)?;
		let signed_tx: SignedTransaction = verified_tx.signed().clone();
		let signed_hash = signed_tx.hash();
		let signed_sender = signed_tx.sender();
		let verified = VerifiedPrivateTransaction {
			private_transaction,
			validator_account,
			transaction: signed_tx,
			transaction_hash: signed_hash,
			transaction_sender: signed_sender,
		};
		let mut pool = self.verification_pool.write();
		pool.import(verified)?;
		Ok(())
	}

	/// Drains transactions ready for verification from the pool
	/// Returns only one transaction per sender because several cannot be verified in a row without verification from other peers
	pub fn drain<C: pool::client::NonceClient>(&self, client: C) -> Vec<Arc<VerifiedPrivateTransaction>> {
		let ready = PrivateReadyState::new(client);
		let transactions: Vec<_> = self.verification_pool.read().pending(ready).collect();
		let mut pool = self.verification_pool.write();
		for tx in &transactions {
			pool.remove(tx.hash(), true);
		}
		transactions
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
			return Err(Error::QueueIsFull);
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
		let desc = self.transactions.get_mut(private_hash).ok_or_else(|| Error::PrivateTransactionNotFound)?;
		if !desc.received_signatures.contains(&signature) {
			desc.received_signatures.push(signature);
		}
		Ok(())
	}
}
