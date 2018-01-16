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

use transaction::SignedTransaction;

pub trait WithTransaction {
	fn with_transaction(self, transaction: SignedTransaction) -> Self where Self: Sized;
}

pub struct Transaction<'a, I> where I: 'a {
	pub iter: &'a mut I,
	pub transaction: SignedTransaction,
}

impl <'a, I> Iterator for Transaction<'a, I> where I: Iterator, <I as Iterator>::Item: WithTransaction {
	type Item = <I as Iterator>::Item;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|item| item.with_transaction(self.transaction.clone()))
	}
}
