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
use std::sync::{mpsc, Mutex, RwLock, Arc};
use std::collections::HashMap;
use v1::types::{TransactionRequest, TransactionConfirmation};
use util::U256;
use jsonrpc_core;

/// Result that can be returned from JSON RPC.
pub type RpcResult = Result<jsonrpc_core::Value, jsonrpc_core::Error>;

/// Possible events happening in the queue that can be listened to.
#[derive(Debug, PartialEq)]
pub enum QueueEvent {
	/// Receiver should stop work upon receiving `Finish` message.
	Finish,
	/// Informs about new request.
	NewRequest(U256),
	/// Request rejected.
	RequestRejected(U256),
	/// Request resolved.
	RequestConfirmed(U256),
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
pub type QueueEventReceiver = mpsc::Receiver<QueueEvent>;

/// A queue of transactions awaiting to be confirmed and signed.
pub trait SigningQueue: Send + Sync {
	/// Add new request to the queue.
	/// Returns a `ConfirmationPromise` that can be used to await for resolution of given request.
	fn add_request(&self, transaction: TransactionRequest) -> ConfirmationPromise;

	/// Removes a request from the queue.
	/// Notifies possible token holders that transaction was rejected.
	fn request_rejected(&self, id: U256) -> Option<TransactionConfirmation>;

	/// Removes a request from the queue.
	/// Notifies possible token holders that transaction was confirmed and given hash was assigned.
	fn request_confirmed(&self, id: U256, result: RpcResult) -> Option<TransactionConfirmation>;

	/// Returns a request if it is contained in the queue.
	fn peek(&self, id: &U256) -> Option<TransactionConfirmation>;

	/// Return copy of all the requests in the queue.
	fn requests(&self) -> Vec<TransactionConfirmation>;

	/// Returns number of transactions awaiting confirmation.
	fn len(&self) -> usize;

	/// Returns true if there are no transactions awaiting confirmation.
	fn is_empty(&self) -> bool;
}

#[derive(Debug, PartialEq)]
enum ConfirmationResult {
	/// The transaction has not yet been confirmed nor rejected.
	Waiting,
	/// The transaction has been rejected.
	Rejected,
	/// The transaction has been confirmed.
	Confirmed(RpcResult),
}

/// Time you need to confirm the transaction in UI.
/// This is the amount of time token holder will wait before
/// returning `None`.
/// Unless we have a multi-threaded RPC this will lock
/// any other incoming call!
const QUEUE_TIMEOUT_DURATION_SEC : u64 = 20;

/// A handle to submitted request.
/// Allows to block and wait for a resolution of that request.
pub struct ConfirmationToken {
	result: Arc<Mutex<ConfirmationResult>>,
	handle: thread::Thread,
	request: TransactionConfirmation,
}

pub struct ConfirmationPromise {
	id: U256,
	result: Arc<Mutex<ConfirmationResult>>,
}

impl ConfirmationToken {
	/// Submit solution to all listeners
	fn resolve(&self, result: Option<RpcResult>) {
		let mut res = self.result.lock().unwrap();
		*res = result.map_or(ConfirmationResult::Rejected, |h| ConfirmationResult::Confirmed(h));
		// Notify listener
		self.handle.unpark();
	}

	fn as_promise(&self) -> ConfirmationPromise {
		ConfirmationPromise {
			id: self.request.id,
			result: self.result.clone(),
		}
	}
}

impl ConfirmationPromise {
	/// Blocks current thread and awaits for
	/// resolution of the transaction (rejected / confirmed)
	/// Returns `None` if transaction was rejected or timeout reached.
	/// Returns `Some(result)` if transaction was confirmed.
	pub fn wait_with_timeout(&self) -> Option<RpcResult> {
		let timeout = Duration::from_secs(QUEUE_TIMEOUT_DURATION_SEC);
		let deadline = Instant::now() + timeout;

		info!(target: "own_tx", "Signer: Awaiting transaction confirmation... ({:?}).", self.id);
		loop {
			let now = Instant::now();
			if now >= deadline {
				break;
			}
			// Park thread (may wake up spuriously)
			thread::park_timeout(deadline - now);
			// Take confirmation result
			let res = self.result.lock().unwrap();
			// Check the result
			match *res {
				ConfirmationResult::Rejected => return None,
				ConfirmationResult::Confirmed(ref h) => return Some(h.clone()),
				ConfirmationResult::Waiting => continue,
			}
		}
		// We reached the timeout. Just return `None`
		trace!(target: "own_tx", "Signer: Confirmation timeout reached... ({:?}).", self.id);
		None
	}
}

/// Queue for all unconfirmed transactions.
pub struct ConfirmationsQueue {
	id: Mutex<U256>,
	queue: RwLock<HashMap<U256, ConfirmationToken>>,
	sender: Mutex<mpsc::Sender<QueueEvent>>,
	receiver: Mutex<Option<mpsc::Receiver<QueueEvent>>>,
}

impl Default for ConfirmationsQueue {
	fn default() -> Self {
		let (send, recv) = mpsc::channel();

		ConfirmationsQueue {
			id: Mutex::new(U256::from(0)),
			queue: RwLock::new(HashMap::new()),
			sender: Mutex::new(send),
			receiver: Mutex::new(Some(recv)),
		}
	}
}

impl ConfirmationsQueue {
	/// Blocks the thread and starts listening for notifications regarding all actions in the queue.
	/// For each event, `listener` callback will be invoked.
	/// This method can be used only once (only single consumer of events can exist).
	pub fn start_listening<F>(&self, listener: F) -> Result<(), QueueError>
		where F: Fn(QueueEvent) -> () {
		let recv = self.receiver.lock().unwrap().take();
		if let None = recv {
			return Err(QueueError::AlreadyUsed);
		}
		let recv = recv.expect("Check for none is done earlier.");

		loop {
			let message = try!(recv.recv().map_err(|e| QueueError::ReceiverError(e)));
			if let QueueEvent::Finish = message {
				return Ok(());
			}

			listener(message);
		}
	}

	/// Notifies consumer that the communcation is over.
	/// No more events will be sent after this function is invoked.
	pub fn finish(&self) {
		self.notify(QueueEvent::Finish);
	}

	/// Notifies receiver about the event happening in this queue.
	fn notify(&self, message: QueueEvent) {
		// We don't really care about the result
		let _ = self.sender.lock().unwrap().send(message);
	}

	/// Removes transaction from this queue and notifies `ConfirmationPromise` holders about the result.
	/// Notifies also a receiver about that event.
	fn remove(&self, id: U256, result: Option<RpcResult>) -> Option<TransactionConfirmation> {
		let token = self.queue.write().unwrap().remove(&id);

		if let Some(token) = token {
			// notify receiver about the event
			self.notify(result.clone().map_or_else(
				|| QueueEvent::RequestRejected(id),
				|_| QueueEvent::RequestConfirmed(id)
			));
			// notify token holders about resolution
			token.resolve(result);
			// return a result
			return Some(token.request.clone());
		}
		None
	}
}

impl Drop for ConfirmationsQueue {
	fn drop(&mut self) {
		self.finish();
	}
}

impl SigningQueue for  ConfirmationsQueue {
	fn add_request(&self, transaction: TransactionRequest) -> ConfirmationPromise {
		// Increment id
		let id = {
			let mut last_id = self.id.lock().unwrap();
			*last_id = *last_id + U256::from(1);
			*last_id
		};
		// Add request to queue
		let res = {
			let mut queue = self.queue.write().unwrap();
			queue.insert(id, ConfirmationToken {
				result: Arc::new(Mutex::new(ConfirmationResult::Waiting)),
				handle: thread::current(),
				request: TransactionConfirmation {
					id: id,
					transaction: transaction,
				},
			});
			debug!(target: "own_tx", "Signer: New transaction ({:?}) in confirmation queue.", id);
			queue.get(&id).map(|token| token.as_promise()).expect("Token was just inserted.")
		};
		// Notify listeners
		self.notify(QueueEvent::NewRequest(id));
		res

	}

	fn peek(&self, id: &U256) -> Option<TransactionConfirmation> {
		self.queue.read().unwrap().get(id).map(|token| token.request.clone())
	}

	fn request_rejected(&self, id: U256) -> Option<TransactionConfirmation> {
		debug!(target: "own_tx", "Signer: Transaction rejected ({:?}).", id);
		self.remove(id, None)
	}

	fn request_confirmed(&self, id: U256, result: RpcResult) -> Option<TransactionConfirmation> {
		debug!(target: "own_tx", "Signer: Transaction confirmed ({:?}).", id);
		self.remove(id, Some(result))
	}

	fn requests(&self) -> Vec<TransactionConfirmation> {
		let queue = self.queue.read().unwrap();
		queue.values().map(|token| token.request.clone()).collect()
	}

	fn len(&self) -> usize {
		let queue = self.queue.read().unwrap();
		queue.len()
	}

	fn is_empty(&self) -> bool {
		let queue = self.queue.read().unwrap();
		queue.is_empty()
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
	use jsonrpc_core::to_value;

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
			v.wait_with_timeout().expect("Should return hash")
		});

		let id = U256::from(1);
		while queue.peek(&id).is_none() {
			// Just wait for the other thread to start
			thread::sleep(Duration::from_millis(100));
		}
		queue.request_confirmed(id, to_value(&H256::from(1)));

		// then
		assert_eq!(handle.join().expect("Thread should finish nicely"), to_value(&H256::from(1)));
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
		queue.add_request(request);
		queue.finish();

		// then
		handle.join().expect("Thread should finish nicely");
		let r = received.lock().unwrap().take();
		assert_eq!(r, Some(QueueEvent::NewRequest(U256::from(1))));
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
