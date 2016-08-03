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
use std::sync::{mpsc, Arc};
use std::collections::BTreeMap;
use jsonrpc_core;
use util::{Mutex, RwLock, U256};
use v1::helpers::{ConfirmationRequest, ConfirmationPayload};

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
	fn add_request(&self, request: ConfirmationPayload) -> ConfirmationPromise;

	/// Removes a request from the queue.
	/// Notifies possible token holders that request was rejected.
	fn request_rejected(&self, id: U256) -> Option<ConfirmationRequest>;

	/// Removes a request from the queue.
	/// Notifies possible token holders that request was confirmed and given hash was assigned.
	fn request_confirmed(&self, id: U256, result: RpcResult) -> Option<ConfirmationRequest>;

	/// Returns a request if it is contained in the queue.
	fn peek(&self, id: &U256) -> Option<ConfirmationRequest>;

	/// Return copy of all the requests in the queue.
	fn requests(&self) -> Vec<ConfirmationRequest>;

	/// Returns number of requests awaiting confirmation.
	fn len(&self) -> usize;

	/// Returns true if there are no requests awaiting confirmation.
	fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
/// Result of a pending confirmation request.
pub enum ConfirmationResult {
	/// The request has not yet been confirmed nor rejected.
	Waiting,
	/// The request has been rejected.
	Rejected,
	/// The request has been confirmed.
	Confirmed(RpcResult),
}

/// Time you need to confirm the request in UI.
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
	request: ConfirmationRequest,
	timeout: Duration,
}

pub struct ConfirmationPromise {
	id: U256,
	result: Arc<Mutex<ConfirmationResult>>,
	timeout: Duration,
}

impl ConfirmationToken {
	/// Submit solution to all listeners
	fn resolve(&self, result: Option<RpcResult>) {
		let mut res = self.result.lock();
		*res = result.map_or(ConfirmationResult::Rejected, |h| ConfirmationResult::Confirmed(h));
		// Notify listener
		self.handle.unpark();
	}

	fn as_promise(&self) -> ConfirmationPromise {
		ConfirmationPromise {
			id: self.request.id,
			result: self.result.clone(),
			timeout: self.timeout,
		}
	}
}

impl ConfirmationPromise {
	/// Get the ID for this request.
	pub fn id(&self) -> U256 { self.id }

	/// Blocks current thread and awaits for
	/// resolution of the transaction (rejected / confirmed)
	/// Returns `None` if transaction was rejected or timeout reached.
	/// Returns `Some(result)` if transaction was confirmed.
	pub fn wait_with_timeout(&self) -> Option<RpcResult> {
		let res = self.wait_until(Instant::now() + self.timeout);
		match res {
			ConfirmationResult::Confirmed(h) => Some(h),
			ConfirmationResult::Rejected | ConfirmationResult::Waiting => None,
		}
	}

	/// Just get the result, assuming it exists.
	pub fn result(&self) -> ConfirmationResult { self.wait_until(Instant::now()) }

	/// Blocks current thread and awaits for
	/// resolution of the request (rejected / confirmed)
	/// Returns `None` if request was rejected or timeout reached.
	/// Returns `Some(result)` if request was confirmed.
	pub fn wait_until(&self, deadline: Instant) -> ConfirmationResult {
		trace!(target: "own_tx", "Signer: Awaiting confirmation... ({:?}).", self.id);
		loop {
			let now = Instant::now();
			// Check the result...
			match *self.result.lock() {
				// Waiting and deadline not yet passed continue looping.
				ConfirmationResult::Waiting if now < deadline => {}
				// Anything else - return.
				ref a => return a.clone(),
			}
			// wait a while longer - maybe the solution will arrive.
			thread::park_timeout(deadline - now);
		}
	}
}

/// Queue for all unconfirmed requests.
pub struct ConfirmationsQueue {
	id: Mutex<U256>,
	queue: RwLock<BTreeMap<U256, ConfirmationToken>>,
	sender: Mutex<mpsc::Sender<QueueEvent>>,
	receiver: Mutex<Option<mpsc::Receiver<QueueEvent>>>,
	timeout: Duration,
}

impl Default for ConfirmationsQueue {
	fn default() -> Self {
		let (send, recv) = mpsc::channel();

		ConfirmationsQueue {
			id: Mutex::new(U256::from(0)),
			queue: RwLock::new(BTreeMap::new()),
			sender: Mutex::new(send),
			receiver: Mutex::new(Some(recv)),
			timeout: Duration::from_secs(QUEUE_TIMEOUT_DURATION_SEC),
		}
	}
}

impl ConfirmationsQueue {
	#[cfg(test)]
	/// Creates new confirmations queue with specified timeout
	pub fn with_timeout(timeout: Duration) -> Self {
		let mut queue = Self::default();
		queue.timeout = timeout;
		queue
	}

	/// Blocks the thread and starts listening for notifications regarding all actions in the queue.
	/// For each event, `listener` callback will be invoked.
	/// This method can be used only once (only single consumer of events can exist).
	pub fn start_listening<F>(&self, listener: F) -> Result<(), QueueError>
		where F: Fn(QueueEvent) -> () {
		let recv = self.receiver.lock().take();
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
		let _ = self.sender.lock().send(message);
	}

	/// Removes requests from this queue and notifies `ConfirmationPromise` holders about the result.
	/// Notifies also a receiver about that event.
	fn remove(&self, id: U256, result: Option<RpcResult>) -> Option<ConfirmationRequest> {
		let token = self.queue.write().remove(&id);

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

impl SigningQueue for ConfirmationsQueue {
	fn add_request(&self, request: ConfirmationPayload) -> ConfirmationPromise {
		// Increment id
		let id = {
			let mut last_id = self.id.lock();
			*last_id = *last_id + U256::from(1);
			*last_id
		};
		// Add request to queue
		let res = {
			debug!(target: "own_tx", "Signer: New entry ({:?}) in confirmation queue.", id);
			trace!(target: "own_tx", "Signer: ({:?}) : {:?}", id, request);

			let mut queue = self.queue.write();
			queue.insert(id, ConfirmationToken {
				result: Arc::new(Mutex::new(ConfirmationResult::Waiting)),
				handle: thread::current(),
				request: ConfirmationRequest {
					id: id,
					payload: request,
				},
				timeout: self.timeout,
			});
			queue.get(&id).map(|token| token.as_promise()).expect("Token was just inserted.")
		};
		// Notify listeners
		self.notify(QueueEvent::NewRequest(id));
		res

	}

	fn peek(&self, id: &U256) -> Option<ConfirmationRequest> {
		self.queue.read().get(id).map(|token| token.request.clone())
	}

	fn request_rejected(&self, id: U256) -> Option<ConfirmationRequest> {
		debug!(target: "own_tx", "Signer: Request rejected ({:?}).", id);
		self.remove(id, None)
	}

	fn request_confirmed(&self, id: U256, result: RpcResult) -> Option<ConfirmationRequest> {
		debug!(target: "own_tx", "Signer: Transaction confirmed ({:?}).", id);
		self.remove(id, Some(result))
	}

	fn requests(&self) -> Vec<ConfirmationRequest> {
		let queue = self.queue.read();
		queue.values().map(|token| token.request.clone()).collect()
	}

	fn len(&self) -> usize {
		let queue = self.queue.read();
		queue.len()
	}

	fn is_empty(&self) -> bool {
		let queue = self.queue.read();
		queue.is_empty()
	}
}


#[cfg(test)]
mod test {
	use std::time::Duration;
	use std::thread;
	use std::sync::Arc;
	use util::{Address, U256, H256, Mutex};
	use v1::helpers::{SigningQueue, ConfirmationsQueue, QueueEvent, FilledTransactionRequest, ConfirmationPayload};
	use v1::types::H256 as NH256;
	use jsonrpc_core::to_value;

	fn request() -> ConfirmationPayload {
		ConfirmationPayload::Transaction(FilledTransactionRequest {
			from: Address::from(1),
			to: Some(Address::from(2)),
			gas_price: 0.into(),
			gas: 10_000.into(),
			value: 10_000_000.into(),
			data: vec![],
			nonce: None,
		})
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
		queue.request_confirmed(id, to_value(&NH256::from(H256::from(1))));

		// then
		assert_eq!(handle.join().expect("Thread should finish nicely"), to_value(&NH256::from(H256::from(1))));
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
				let mut v = r.lock();
				*v = Some(notification);
			}).expect("Should be closed nicely.")
		});
		queue.add_request(request);
		queue.finish();

		// then
		handle.join().expect("Thread should finish nicely");
		let r = received.lock().take();
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
		assert_eq!(el.payload, request);
	}
}
