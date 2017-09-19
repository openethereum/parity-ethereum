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

use transaction::UnverifiedTransaction;
use error::Error;

/// Storage for private transactions
pub struct PrivateTransactions {
	transactions: Vec<UnverifiedTransaction>,
	signed_transactions: Vec<UnverifiedTransaction>,
}

impl PrivateTransactions {
	/// Creates new store
	pub fn new() -> Self {
		PrivateTransactions {
			transactions: Vec::new(),
			signed_transactions: Vec::new(),
		}
	}

	/// Adds private transaction into the store
	pub fn import_transaction(&mut self, transaction: UnverifiedTransaction, _peer_id: usize) -> Result<(), Error> {
		self.transactions.push(transaction);
		Ok(())
	}

	/// Adds signed private transaction into the store
	pub fn import_signed_transaction(&mut self, transaction: UnverifiedTransaction, _peer_id: usize) -> Result<(), Error> {
		self.signed_transactions.push(transaction);
		Ok(())
	}

	/// Returns the list of all stored private transactions
	pub fn transactions_list(&self) -> Vec<UnverifiedTransaction> {
		self.transactions.clone()
	}

	/// Returns the list of all stored signed private transactions
	pub fn signed_transactions_list(&self) -> Vec<UnverifiedTransaction> {
		self.signed_transactions.clone()
	}
}