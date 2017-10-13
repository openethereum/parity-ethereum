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

use std::collections::BTreeMap;
use bigint::prelude::U256;
use util::Address;
use parking_lot::{Mutex, RwLock};
use ethcore::account_provider::DappId;
use v1::helpers::{ConfirmationRequest, ConfirmationPayload, oneshot, errors};
use v1::types::{ConfirmationResponse, H160 as RpcH160, Origin, DappId as RpcDappId};

use jsonrpc_core::Error;
use jsonrpc_core::futures::{Future, Poll};

/// Result that can be returned from JSON RPC.
pub type RpcResult = Result<ConfirmationResponse, Error>;

/// Type of default account
pub enum DefaultAccount {
	/// Default account is known
	Provided(Address),
	/// Should use default account for dapp
	ForDapp(DappId),
}

impl From<RpcDappId> for DefaultAccount {
	fn from(dapp_id: RpcDappId) -> Self {
		DefaultAccount::ForDapp(dapp_id.into())
	}
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
	/// Returns a `Result` wrapping  `ConfirmationReceiver` which is a `Future` awaiting for resolution of the given request.
	fn add_request(&self, request: ConfirmationPayload, origin: Origin) -> Result<ConfirmationReceiver, QueueAddError>;

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
// TODO [kirushik] Let's get rid of this completely, and simplify that `Result<Option<Result...` juggling somehow
pub enum ConfirmationOutcome {
	/// The request has been rejected.
	Rejected,
	/// The request has been confirmed.
	Confirmed(ConfirmationResponse),
	/// We've failed to determine the state of the confirmation
	Failed(Error),
}

impl From<Option<RpcResult>> for ConfirmationOutcome {
	fn from(result: Option<RpcResult>) -> Self {
		match result {
			None => ConfirmationOutcome::Rejected,
			Some(Ok(response)) => ConfirmationOutcome::Confirmed(response),
			Some(Err(error)) => ConfirmationOutcome::Failed(error),
		}
	}
}

impl Into<Result<Option<ConfirmationResponse>, Error>> for ConfirmationOutcome {
	fn into(self) -> Result<Option<ConfirmationResponse>, Error> {
		match self {
			ConfirmationOutcome::Rejected => Err(errors::request_rejected()),
			ConfirmationOutcome::Confirmed(confirmation) => Ok(Some(confirmation)),
			ConfirmationOutcome::Failed(error) => Err(error),
		}
	}
}

struct ConfirmationSender {
	sender: oneshot::Sender<ConfirmationOutcome>,
	request: ConfirmationRequest,
}

/// Receiving end of the Confirmation channel; can be used as a `Future` to await for `ConfirmationRequest`
/// being processed and turned into `ConfirmationOutcome`
#[must_use = "futures do nothing unless polled"]
pub struct ConfirmationReceiver {
	id: U256,
	receiver: oneshot::Receiver<ConfirmationOutcome>
}

impl ConfirmationReceiver {
	fn new(id: U256, receiver: oneshot::Receiver<ConfirmationOutcome>) -> Self {
		ConfirmationReceiver {
			id: id,
			receiver: receiver
		}
	}

	/// `U256` id of the request we're waiting to get confirmed.
	pub fn id(&self) -> U256 {
		self.id
	}
}

impl Future for ConfirmationReceiver {
	type Item = ConfirmationOutcome;
	type Error = Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		self.receiver.poll()
	}
}

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
		self.notify(QueueEvent::Finish);
		self.on_event.write().clear();
	}

	/// Notifies receiver about the event happening in this queue.
	fn notify(&self, message: QueueEvent) {
		for listener in &*self.on_event.read() {
			listener(message.clone())
		}
	}

	/// Removes requests from this queue and notifies `ConfirmationReceiver` holder about the result.
	/// Notifies also a receiver about that event.
	fn remove(&self, id: U256, result: Option<RpcResult>) -> Option<ConfirmationRequest> {
		let sender = self.queue.write().remove(&id);

		if let Some(sender) = sender {
			// notify receiver about the event
			self.notify(result.clone().map_or_else(
				|| QueueEvent::RequestRejected(id),
				|_| QueueEvent::RequestConfirmed(id)
			));

			// notify confirmation receiver about resolution
			let confirmation = result.into();
			sender.sender.send(Ok(confirmation));

			Some(sender.request)
		} else {
			None
		}
	}
}

impl Drop for ConfirmationsQueue {
	fn drop(&mut self) {
		self.finish();
	}
}

impl SigningQueue for ConfirmationsQueue {
	fn add_request(&self, request: ConfirmationPayload, origin: Origin) -> Result<ConfirmationReceiver, QueueAddError> {
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
			let (sender, receiver) = oneshot::oneshot::<ConfirmationOutcome>();

			queue.insert(id, ConfirmationSender {
				sender: sender,
				request: ConfirmationRequest {
					id: id,
					payload: request,
					origin: origin,
				},
			});
			ConfirmationReceiver::new(id, receiver)
		};
		// Notify listeners
		self.notify(QueueEvent::NewRequest(id));
		Ok(res)
	}

	fn peek(&self, id: &U256) -> Option<ConfirmationRequest> {
		self.queue.read().get(id).map(|sender| sender.request.clone())
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
	use bigint::prelude::U256;
	use util::Address;
	use parking_lot::Mutex;
	use jsonrpc_core::futures::Future;
	use v1::helpers::{
		SigningQueue, ConfirmationsQueue, QueueEvent, FilledTransactionRequest,
		ConfirmationOutcome, ConfirmationPayload,
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
		let future = queue.add_request(request, Default::default()).unwrap();

		let id = U256::from(1);
		queue.request_confirmed(id, Ok(ConfirmationResponse::SendTransaction(1.into())));

		// then
		let confirmation = future.wait().unwrap();
		assert_eq!(confirmation, ConfirmationOutcome::Confirmed(ConfirmationResponse::SendTransaction(1.into())));
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
