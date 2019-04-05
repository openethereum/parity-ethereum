use std::sync::{Arc, Weak};

use jsonrpc_core::Result;
use jsonrpc_core::futures::Future;
use jsonrpc_pubsub::{SubscriptionId, typed::{Sink, Subscriber}};

use v1::helpers::Subscribers;
use v1::metadata::Metadata;
use v1::traits::TransactionsPool;

use miner::pool::TxStatus;
use parity_runtime::Executor;
use parking_lot::RwLock;
use ethereum_types::H256;
use futures::{Stream, sync::mpsc};

type Client = Sink<(H256, TxStatus)>;

/// Transactions pool PubSub implementation.
pub struct TransactionsPoolClient {
	handler: Arc<TransactionsNotificationHandler>,
	transactions_pool_subscribers: Arc<RwLock<Subscribers<Client>>>,
}

impl TransactionsPoolClient {
	/// Creates new `TransactionsPoolClient`.
	pub fn new(executor: Executor, pool_receiver: mpsc::UnboundedReceiver<Arc<Vec<(H256, TxStatus)>>>) -> Self {
		let transactions_pool_subscribers = Arc::new(RwLock::new(Subscribers::default()));
		let handler = Arc::new(
			TransactionsNotificationHandler::new(
				executor.clone(),
				transactions_pool_subscribers.clone(),
			)
		);
		let handler2 = Arc::downgrade(&handler);

		executor.spawn(pool_receiver
			.for_each(move |tx_status| {
				if let Some(handler2) = handler2.upgrade() {
					handler2.notify_transaction(tx_status);
				}
				Ok(())
			})
			.map_err(|e| warn!("Key server listener error: {:?}", e))
		);

		TransactionsPoolClient {
			handler,
			transactions_pool_subscribers,
		}
	}

	/// Returns a chain notification handler.
	pub fn handler(&self) -> Weak<TransactionsNotificationHandler> {
		Arc::downgrade(&self.handler)
	}
}

/// Transactions pool PubSub Notification handler.
pub struct TransactionsNotificationHandler {
	executor: Executor,
	transactions_pool_subscribers: Arc<RwLock<Subscribers<Client>>>,
}

impl TransactionsNotificationHandler {
	fn new(executor: Executor, transactions_pool_subscribers: Arc<RwLock<Subscribers<Client>>>) -> Self {
		TransactionsNotificationHandler {
			executor,
			transactions_pool_subscribers,
		}
	}

	fn notify(executor: &Executor, subscriber: &Client, result: (H256, TxStatus)) {
		executor.spawn(subscriber
			.notify(Ok(result))
			.map(|_| ())
			.map_err(|e| warn!(target: "rpc", "Unable to send notification: {}", e))
		);
	}

	pub fn notify_transaction(&self, tx_statuses: Arc<Vec<(H256, TxStatus)>>) {
		for subscriber in self.transactions_pool_subscribers.read().values() {
			for tx_status in tx_statuses.to_vec() {
				Self::notify(&self.executor, subscriber, tx_status.clone());
			}
		}
	}
}

impl TransactionsPool for TransactionsPoolClient {
	type Metadata = Metadata;

	fn subscribe(&self, _meta: Metadata, subscriber: Subscriber<(H256, TxStatus)>) {
		self.transactions_pool_subscribers.write().push(subscriber);
	}

	fn unsubscribe(&self, _meta: Option<Metadata>, id: SubscriptionId) -> Result<bool> {
		let res = self.transactions_pool_subscribers.write().remove(&id).is_some();
		Ok(res)
	}
}
