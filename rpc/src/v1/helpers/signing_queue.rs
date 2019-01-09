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

use std::collections::BTreeMap;
use ethereum_types::{U256, Address};
use parking_lot::{Mutex, RwLock};
use v1::helpers::{ConfirmationRequest, ConfirmationPayload, oneshot, errors};
use v1::types::{ConfirmationResponse, H160 as RpcH160, Origin};

use jsonrpc_core::Error;

/// Result that can be returned from JSON RPC.
pub type ConfirmationResult = Result<ConfirmationResponse, Error>;

/// Type of default account
pub enum DefaultAccount {
	/// Default account is known
	Provided(Address),
}

impl From<RpcH160> for DefaultAccount {
	fn from(address: RpcH160) -> Self {
		DefaultAccount::Provided(address.into())
	}
}

/// Possible events happening in the queue that can be listened to.
#[derive(Debug, PartialEq, Clone)]
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

/// Defines possible errors when inserting to queue
#[derive(Debug, PartialEq)]
pub enum QueueAddError {
	LimitReached,
}

// TODO [todr] to consider: timeout instead of limit?
pub const QUEUE_LIMIT: usize = 50;

/// A queue of transactions awaiting to be confirmed and signed.
pub trait SigningQueue: Send + Sync {
	/// Add new request to the queue.
	/// Returns a `Result` wrapping  `ConfirmationReceiver` together with it's unique id in the queue.
	/// `ConfirmationReceiver` is a `Future` awaiting for resolution of the given request.
	fn add_request(&self, request: ConfirmationPayload, origin: Origin) -> Result<(U256, ConfirmationReceiver), QueueAddError>;

	/// Notifies possible token holders that request was rejected.
	fn request_rejected(&self, sender: ConfirmationSender) -> Option<ConfirmationRequest>;

	/// Notifies possible token holders that request was confirmed and given hash was assigned.
	fn request_confirmed(&self, sender: ConfirmationSender, result: ConfirmationResult) -> Option<ConfirmationRequest>;

	/// Put a request taken from `SigningQueue::take` back to the queue.
	fn request_untouched(&self, sender: ConfirmationSender);

	/// Returns and removes a request if it is contained in the queue.
	fn take(&self, id: &U256) -> Option<ConfirmationSender>;

	/// Return copy of all the requests in the queue.
	fn requests(&self) -> Vec<ConfirmationRequest>;

	/// Returns number of requests awaiting confirmation.
	fn len(&self) -> usize;

	/// Returns true if there are no requests awaiting confirmation.
	fn is_empty(&self) -> bool;
}

/// Confirmation request information with result notifier.
pub struct ConfirmationSender {
	/// Confirmation request information.
	pub request: ConfirmationRequest,

	sender: oneshot::Sender<ConfirmationResult>,
}

/// Receiving end of the Confirmation channel; can be used as a `Future` to await for `ConfirmationRequest`
/// being processed and turned into `ConfirmationOutcome`
pub type ConfirmationReceiver =  oneshot::Receiver<ConfirmationResult>;

/// Queue for all unconfirmed requests.
#[derive(Default)]
pub struct ConfirmationsQueue {
	id: Mutex<U256>,
	queue: RwLock<BTreeMap<U256, ConfirmationSender>>,
	on_event: RwLock<Vec<Box<Fn(QueueEvent) -> () + Send + Sync>>>,
}

impl ConfirmationsQueue {
	/// Adds a queue listener. For each event, `listener` callback will be invoked.
	pub fn on_event<F: Fn(QueueEvent) -> () + Send + Sync + 'static>(&self, listener: F) {
		self.on_event.write().push(Box::new(listener));
	}

	/// Notifies consumer that the communcation is over.
	/// No more events will be sent after this function is invoked.
	pub fn finish(&self) {
		self.notify_message(QueueEvent::Finish);
		self.on_event.write().clear();
	}

	/// Notifies `ConfirmationReceiver` holder about the result given a request.
	fn notify_result(&self, sender: ConfirmationSender, result: Option<ConfirmationResult>) -> Option<ConfirmationRequest> {
		// notify receiver about the event
		self.notify_message(result.clone().map_or_else(
			|| QueueEvent::RequestRejected(sender.request.id),
			|_| QueueEvent::RequestConfirmed(sender.request.id)
		));

		// notify confirmation receiver about resolution
		let result = result.ok_or(errors::request_rejected());
		sender.sender.send(result);

		Some(sender.request)
	}

	/// Notifies receiver about the event happening in this queue.
	fn notify_message(&self, message: QueueEvent) {
		for listener in &*self.on_event.read() {
			listener(message.clone())
		}
	}
}

impl Drop for ConfirmationsQueue {
	fn drop(&mut self) {
		self.finish();
	}
}

impl SigningQueue for ConfirmationsQueue {
	fn add_request(&self, request: ConfirmationPayload, origin: Origin) -> Result<(U256, ConfirmationReceiver), QueueAddError> {
		if self.len() > QUEUE_LIMIT {
			return Err(QueueAddError::LimitReached);
		}

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
			let (sender, receiver) = oneshot::oneshot::<ConfirmationResult>();

			queue.insert(id, ConfirmationSender {
				sender,
				request: ConfirmationRequest {
					id,
					payload: request,
					origin,
				},
			});
			(id, receiver)
		};
		// Notify listeners
		self.notify_message(QueueEvent::NewRequest(id));
		Ok(res)
	}

	fn take(&self, id: &U256) -> Option<ConfirmationSender> {
		self.queue.write().remove(id)
	}

	fn request_rejected(&self, sender: ConfirmationSender) -> Option<ConfirmationRequest> {
		debug!(target: "own_tx", "Signer: Request rejected ({:?}).", sender.request.id);
		self.notify_result(sender, None)
	}

	fn request_confirmed(&self, sender: ConfirmationSender, result: ConfirmationResult) -> Option<ConfirmationRequest> {
		debug!(target: "own_tx", "Signer: Request confirmed ({:?}).", sender.request.id);
		self.notify_result(sender, Some(result))
	}

	fn request_untouched(&self, sender: ConfirmationSender) {
		self.queue.write().insert(sender.request.id, sender);
	}

	fn requests(&self) -> Vec<ConfirmationRequest> {
		let queue = self.queue.read();
		queue.values().map(|sender| sender.request.clone()).collect()
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
	use std::sync::Arc;
	use ethereum_types::{U256, Address};
	use parking_lot::Mutex;
	use jsonrpc_core::futures::Future;
	use v1::helpers::{
		SigningQueue, ConfirmationsQueue, QueueEvent, FilledTransactionRequest, ConfirmationPayload,
	};
	use v1::types::ConfirmationResponse;

	fn request() -> ConfirmationPayload {
		ConfirmationPayload::SendTransaction(FilledTransactionRequest {
			from: Address::from(1),
			used_default_from: false,
			to: Some(Address::from(2)),
			gas_price: 0.into(),
			gas: 10_000.into(),
			value: 10_000_000.into(),
			data: vec![],
			nonce: None,
			condition: None,
		})
	}

	#[test]
	fn should_wait_for_hash() {
		// given
		let queue = Arc::new(ConfirmationsQueue::default());
		let request = request();

		// when
		let (id, future) = queue.add_request(request, Default::default()).unwrap();
		let sender = queue.take(&id).unwrap();
		queue.request_confirmed(sender, Ok(ConfirmationResponse::SendTransaction(1.into())));

		// then
		let confirmation = future.wait().unwrap();
		assert_eq!(confirmation, Ok(ConfirmationResponse::SendTransaction(1.into())));
	}

	#[test]
	fn should_receive_notification() {
		// given
		let received = Arc::new(Mutex::new(vec![]));
		let queue = Arc::new(ConfirmationsQueue::default());
		let request = request();

		// when
		let r = received.clone();
		queue.on_event(move |notification| {
			r.lock().push(notification);
		});
		let _future = queue.add_request(request, Default::default()).unwrap();
		queue.finish();

		// then
		let r = received.lock();
		assert_eq!(r[0], QueueEvent::NewRequest(U256::from(1)));
		assert_eq!(r[1], QueueEvent::Finish);
		assert_eq!(r.len(), 2);
	}

	#[test]
	fn should_add_transactions() {
		// given
		let queue = ConfirmationsQueue::default();
		let request = request();

		// when
		let _future = queue.add_request(request.clone(), Default::default()).unwrap();
		let all = queue.requests();

		// then
		assert_eq!(all.len(), 1);
		let el = all.get(0).unwrap();
		assert_eq!(el.id, U256::from(1));
		assert_eq!(el.payload, request);
	}
}
