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

use std::thread;
use std::time::{Instant, Duration};
use std::sync::{mpsc, Mutex, RwLock};
use std::collections::HashMap;
use v1::types::{TransactionRequest, TransactionConfirmation};
use util::{U256, H256};


/// Messages that queue informs about
#[derive(Debug, PartialEq)]
pub enum QueueMessage {
	/// Receiver should stop work upon receiving `Finish` message.
	Finish,
	/// Informs about new transaction request.
	NewRequest(U256),
}

/// Defines possible errors returned from queue receiving method.
#[derive(Debug, PartialEq)]
pub enum QueueError {
	/// Returned when method has been already used (no receiver available).
	AlreadyUsed,
	/// Returned when receiver encounters an error.
	ReceiverError(mpsc::RecvError),
}

/// Message Receiver type
pub type QueueMessageReceiver = mpsc::Receiver<QueueMessage>;

/// A queue of transactions awaiting to be confirmed and signed.
pub trait SigningQueue: Send + Sync {
	/// Add new request to the queue.
	fn add_request(&self, transaction: TransactionRequest) -> U256;

	/// Remove request from the queue.
	/// Notify possible waiters that transaction was rejected.
	fn request_rejected(&self, id: U256) -> Option<TransactionConfirmation>;

	/// Remove request from the queue.
	/// Notify possible waiters that transaction was confirmed and got given hash.
	fn request_confirmed(&self, id: U256, hash: H256) -> Option<TransactionConfirmation>;

	/// Returns a request if it is contained in the queue.
	fn peek(&self, id: &U256) -> Option<TransactionConfirmation>;

	/// Return copy of all the requests in the queue.
	fn requests(&self) -> Vec<TransactionConfirmation>;

	/// Blocks for some time waiting for confirmation.
	/// Returns `None` when timeout reached or transaction was rejected.
	/// Returns transaction hash when transaction was confirmed.
	fn wait_with_timeout(&self, id: U256) -> Option<H256>;
}

/// Time you need to confirm the transaction in UI.
/// Unless we have a multi-threaded RPC this will lock
/// any other incoming call!
const QUEUE_TIMEOUT_DURATION_SEC : u64 = 20;

#[derive(Debug, Clone)]
enum QueueStatus {
	Waiting,
	Rejected,
	Confirmed(H256),
}

/// Queue for all unconfirmed transactions.
pub struct ConfirmationsQueue {
	id: Mutex<U256>,
	waiters: RwLock<HashMap<U256, QueueStatus>>,
	queue: RwLock<HashMap<U256, TransactionConfirmation>>,
	sender: Mutex<mpsc::Sender<QueueMessage>>,
	receiver: Mutex<Option<mpsc::Receiver<QueueMessage>>>,
}

impl Default for ConfirmationsQueue {
	fn default() -> Self {
		let (send, recv) = mpsc::channel();

		ConfirmationsQueue {
			id: Mutex::new(U256::from(0)),
			waiters: RwLock::new(HashMap::new()),
			queue: RwLock::new(HashMap::new()),
			sender: Mutex::new(send),
			receiver: Mutex::new(Some(recv)),
		}
	}
}

impl ConfirmationsQueue {
	/// Blocks the thread and starts listening for notifications.
	/// For each event `listener` callback function will be invoked.
	/// This method can be used only once.
	pub fn start_listening<F>(&self, listener: F) -> Result<(), QueueError>
		where F: Fn(QueueMessage) -> () {
		let recv = self.receiver.lock().unwrap().take();
		if let None = recv {
			return Err(QueueError::AlreadyUsed);
		}
		let recv = recv.expect("Check for none is done earlier.");

		loop {
			let message = try!(recv.recv().map_err(|e| QueueError::ReceiverError(e)));
			if let QueueMessage::Finish = message {
				return Ok(());
			}

			listener(message);
		}
	}

	/// Notifies receiver that the communcation is over.
	pub fn finish(&self) {
		self.notify(QueueMessage::Finish);
	}

	fn notify(&self, message: QueueMessage) {
		// We don't really care about the result
		let _ = self.sender.lock().unwrap().send(message);
	}

	fn remove(&self, id: U256) -> Option<TransactionConfirmation> {
		self.queue.write().unwrap().remove(&id)
	}

	fn update_status(&self, id: U256, status: QueueStatus) {
		let mut waiters = self.waiters.write().unwrap();
		waiters.insert(id, status);
	}
}

impl Drop for ConfirmationsQueue {
	fn drop(&mut self) {
		self.finish();
	}
}

impl SigningQueue for  ConfirmationsQueue {
	fn add_request(&self, transaction: TransactionRequest) -> U256 {
		// Increment id
		let id = {
			let mut last_id = self.id.lock().unwrap();
			*last_id = *last_id + U256::from(1);
			*last_id
		};
		// Add request to queue
		{
			let mut queue = self.queue.write().unwrap();
			queue.insert(id, TransactionConfirmation {
				id: id,
				transaction: transaction,
			});
			debug!(target: "own_tx", "Signer: New transaction ({:?}) in confirmation queue.", id);
		}
		// Notify listeners
		self.notify(QueueMessage::NewRequest(id));
		id
	}

	fn peek(&self, id: &U256) -> Option<TransactionConfirmation> {
		self.queue.read().unwrap().get(id).cloned()
	}

	fn request_rejected(&self, id: U256) -> Option<TransactionConfirmation> {
		debug!(target: "own_tx", "Signer: Transaction rejected ({:?}).", id);
		let o = self.remove(id);
		self.update_status(id, QueueStatus::Rejected);
		o
	}

	fn request_confirmed(&self, id: U256, hash: H256) -> Option<TransactionConfirmation> {
		debug!(target: "own_tx", "Signer: Transaction confirmed ({:?}).", id);
		let o = self.remove(id);
		self.update_status(id, QueueStatus::Confirmed(hash));
		o
	}

	fn requests(&self) -> Vec<TransactionConfirmation> {
		let queue = self.queue.read().unwrap();
		queue.values().cloned().collect()
	}

	fn wait_with_timeout(&self, id: U256) -> Option<H256> {
		{
			let mut waiters = self.waiters.write().unwrap();
			let r = waiters.insert(id, QueueStatus::Waiting);
			match r {
				// This is ok, we can have many waiters
				Some(QueueStatus::Waiting) | None => {},
				// There already was a response for someone.
				// The one waiting for it will cleanup, so...
				Some(v) => {
					// ... insert old status back
					waiters.insert(id, v.clone());
					if let QueueStatus::Confirmed(h) = v {
						return Some(h);
					}
					return None;
				},
			}
		}

		info!(target: "own_tx", "Signer: Awaiting transaction confirmation... ({:?}).", id);
		// Now wait for a response
		let deadline = Instant::now() + Duration::from_secs(QUEUE_TIMEOUT_DURATION_SEC);
		while Instant::now() < deadline {
			let status = {
				let waiters = self.waiters.read().unwrap();
				waiters.get(&id).expect("Only the waiting thread can remove any message.").clone()
			};

			match status {
				QueueStatus::Waiting => thread::sleep(Duration::from_millis(50)),
				QueueStatus::Confirmed(h) => {
					self.waiters.write().unwrap().remove(&id);
					return Some(h);
				},
				QueueStatus::Rejected => {
					self.waiters.write().unwrap().remove(&id);
					return None;
				},
			}
		}
		// We reached the timeout. Just return `None` and make sure to remove waiting.
		trace!(target: "own_tx", "Signer: Confirmation timeout reached... ({:?}).", id);
		self.waiters.write().unwrap().remove(&id);
		None
	}
}


#[cfg(test)]
mod test {
	use std::time::Duration;
	use std::thread;
	use std::sync::{Arc, Mutex};
	use util::hash::Address;
	use util::numbers::{U256, H256};
	use v1::types::TransactionRequest;
	use super::*;

	fn request() -> TransactionRequest {
		TransactionRequest {
			from: Address::from(1),
			to: Some(Address::from(2)),
			gas_price: None,
			gas: None,
			value: Some(U256::from(10_000_000)),
			data: None,
			nonce: None,
		}
	}

	#[test]
	fn should_wait_for_hash() {
		// given
		let queue = Arc::new(ConfirmationsQueue::default());
		let request = request();

		// when
		let q = queue.clone();
		let handle = thread::spawn(move || {
			let v = q.add_request(request);
			q.wait_with_timeout(v).expect("Should return hash")
		});

		let id = U256::from(1);
		while queue.peek(&id).is_none() {
			// Just wait for the other thread to start
			thread::sleep(Duration::from_millis(100));
		}
		queue.request_confirmed(id, H256::from(1));

		// then
		assert_eq!(handle.join().expect("Thread should finish nicely"), H256::from(1));
	}

	#[test]
	fn should_receive_notification() {
		// given
		let received = Arc::new(Mutex::new(None));
		let queue = Arc::new(ConfirmationsQueue::default());
		let request = request();

		// when
		let q = queue.clone();
		let r = received.clone();
		let handle = thread::spawn(move || {
			q.start_listening(move |notification| {
				let mut v = r.lock().unwrap();
				*v = Some(notification);
			}).expect("Should be closed nicely.")
		});
		let v = queue.add_request(request);
		queue.finish();

		// then
		handle.join().expect("Thread should finish nicely");
		let r = received.lock().unwrap().take();
		assert_eq!(r, Some(QueueMessage::NewRequest(v)));
	}

	#[test]
	fn should_add_transactions() {
		// given
		let queue = ConfirmationsQueue::default();
		let request = request();

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
