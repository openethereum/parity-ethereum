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

use std::sync::Mutex;
use std::collections::HashMap;
use v1::types::{TransactionRequest, TransactionConfirmation};
use util::U256;

/// A queue of transactions awaiting to be confirmed and signed.
pub trait SigningQueue: Send + Sync {
	/// Add new request to the queue.
	fn add_request(&self, transaction: TransactionRequest) -> U256;

	/// Remove request from the queue.
	fn remove_request(&self, id: U256) -> Option<TransactionConfirmation>;

	/// Return copy of all the requests in the queue.
	fn requests(&self) -> Vec<TransactionConfirmation>;
}

/// Queue for all unconfirmed transactions.
#[derive(Default)]
pub struct ConfirmationsQueue {
	id: Mutex<U256>,
	queue: Mutex<HashMap<U256, TransactionConfirmation>>,
}

impl SigningQueue for  ConfirmationsQueue {
	fn add_request(&self, transaction: TransactionRequest) -> U256 {
		// Increment id
		let id = {
			let mut last_id = self.id.lock().unwrap();
			*last_id = *last_id + U256::from(1);
			*last_id
		};
		let mut queue = self.queue.lock().unwrap();
		queue.insert(id, TransactionConfirmation {
			id: id,
			transaction: transaction,
		});
		id
	}

	fn remove_request(&self, id: U256) -> Option<TransactionConfirmation> {
		self.queue.lock().unwrap().remove(&id)
	}

	fn requests(&self) -> Vec<TransactionConfirmation> {
		let queue = self.queue.lock().unwrap();
		queue.values().cloned().collect()
	}
}


#[cfg(test)]
mod test {
	use util::hash::Address;
	use util::numbers::U256;
	use v1::types::TransactionRequest;
	use super::*;

	#[test]
	fn should_work_for_hashset() {
		// given
		let queue = ConfirmationsQueue::default();

		let request = TransactionRequest {
			from: Address::from(1),
			to: Some(Address::from(2)),
			gas_price: None,
			gas: None,
			value: Some(U256::from(10_000_000)),
			data: None,
			nonce: None,
		};

		// when
		queue.add_request(request.clone());
		let all = queue.requests();

		// then
		assert_eq!(all.len(), 1);
		let el = all.get(0).unwrap();
		assert_eq!(el.id, U256::from(1));
		assert_eq!(el.transaction, request);
	}
}
