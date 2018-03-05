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

use std::fmt;

use ethereum_types::{H256, U256, Address};
use ethcore_miner::pool;
use transaction::{
	self,
	UnverifiedTransaction,
	SignedTransaction,
};

use account_provider::AccountProvider;
use client::{TransactionId, BlockInfo, CallContract, Nonce};
use engines::EthEngine;
use header::Header;
use miner::TransactionImporterClient;
use miner::service_transaction_checker::ServiceTransactionChecker;

// TODO [ToDr] Shit
#[derive(Clone)]
pub struct FakeContainer(Header);
unsafe impl Sync for FakeContainer {}

pub struct BlockChainClient<'a, C: 'a> {
	chain: &'a C,
	engine: &'a EthEngine,
	accounts: Option<&'a AccountProvider>,
	best_block_header: FakeContainer,
	service_transaction_checker: Option<ServiceTransactionChecker>,
}

impl<'a, C: 'a> Clone for BlockChainClient<'a, C> {
	fn clone(&self) -> Self {
		BlockChainClient {
			chain: self.chain,
			engine: self.engine,
			accounts: self.accounts.clone(),
			best_block_header: self.best_block_header.clone(),
			service_transaction_checker: self.service_transaction_checker.clone(),
		}
	}
}

impl<'a, C: 'a> BlockChainClient<'a, C> where
	C: BlockInfo + CallContract,
{
	pub fn new(
		chain: &'a C,
		engine: &'a EthEngine,
		accounts: Option<&'a AccountProvider>,
		refuse_service_transactions: bool,
	) -> Self {
		let best_block_header = chain.best_block_header().decode();
		best_block_header.hash();
		best_block_header.bare_hash();
		let best_block_header = FakeContainer(best_block_header);
		BlockChainClient {
			chain,
			engine,
			accounts,
			best_block_header,
			service_transaction_checker: if refuse_service_transactions {
				None
			} else {
				Some(Default::default())
			},
		}
	}

	pub fn verify_signed(&self, tx: &SignedTransaction) -> Result<(), transaction::Error> {
		self.engine.machine().verify_transaction(&tx, &self.best_block_header.0, self.chain)
	}
}

impl<'a, C: 'a> fmt::Debug for BlockChainClient<'a, C> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "BlockChainClient")
	}
}

impl<'a, C: 'a> pool::client::Client for BlockChainClient<'a, C> where
	C: TransactionImporterClient + Sync,
{
	fn transaction_already_included(&self, hash: &H256) -> bool {
		self.chain.transaction_block(TransactionId::Hash(*hash)).is_some()
	}

	fn verify_transaction(&self, tx: UnverifiedTransaction)
		-> Result<SignedTransaction, transaction::Error>
	{
		self.engine.verify_transaction_basic(&tx, &self.best_block_header.0)?;
		let tx = self.engine.verify_transaction_unordered(tx, &self.best_block_header.0)?;

		self.verify_signed(&tx)?;

		Ok(tx)
	}

	fn account_details(&self, address: &Address) -> pool::client::AccountDetails {
		pool::client::AccountDetails {
			nonce: self.chain.latest_nonce(address),
			balance: self.chain.latest_balance(address),
			is_local: self.accounts.map_or(false, |accounts| accounts.has_account(*address).unwrap_or(false)),
		}
	}

	fn required_gas(&self, tx: &transaction::Transaction) -> U256 {
		tx.gas_required(&self.chain.latest_schedule()).into()
	}

	fn transaction_type(&self, tx: &SignedTransaction) -> pool::client::TransactionType {
		match self.service_transaction_checker {
			None => pool::client::TransactionType::Regular,
			Some(ref checker) => match checker.check(self.chain, &tx) {
				Ok(true) => pool::client::TransactionType::Service,
				Ok(false) => pool::client::TransactionType::Regular,
				Err(e) => {
					debug!(target: "txqueue", "Unable to verify service transaction: {:?}", e);
					pool::client::TransactionType::Regular
				},
			}
		}
	}
}

impl<'a, C: 'a> pool::client::StateClient for BlockChainClient<'a, C> where
	C: Nonce + Sync,
{
	fn account_nonce(&self, address: &Address) -> U256 {
		self.chain.latest_nonce(address)
	}
}

// TODO [ToDr] Remove!
pub struct NonceClient<'a, C: 'a> {
	client: &'a C,
}

impl<'a, C: 'a> Clone for NonceClient<'a, C> {
	fn clone(&self) -> Self {
		NonceClient {
			client: self.client,
		}
	}
}

impl<'a, C: 'a> fmt::Debug for NonceClient<'a, C> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "NonceClient")
	}
}

impl<'a, C: 'a> NonceClient<'a, C> {
	pub fn new(client: &'a C) -> Self {
		NonceClient {
			client,
		}
	}
}

impl<'a, C: 'a> pool::client::StateClient for NonceClient<'a, C>
	where C: Nonce + Sync,
{
	fn account_nonce(&self, address: &Address) -> U256 {
		self.client.latest_nonce(address)
	}
}
