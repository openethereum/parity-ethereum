use std::sync::{Arc, mpsc};
use std::thread;

use jsonrpc_core::Result;
use jsonrpc_core::futures::Future;
use jsonrpc_pubsub::{SubscriptionId, typed::{Sink, Subscriber}};

use v1::helpers::Subscribers;
use v1::metadata::Metadata;
use v1::traits::TransactionsPool;
use v1::types::pubsub;

use miner::pool::TxStatus;
use parity_runtime::Executor;
use parking_lot::RwLock;
use ethereum_types::H256;

type Client = Sink<pubsub::Result>;

/// Transactions pool PubSub implementation.
pub struct TransactionsPoolClient {
	handler: Arc<TransactionsNotificationHandler>,
	transactions_pool_subscribers: Arc<RwLock<Subscribers<Client>>>,
}

impl TransactionsPoolClient {
	/// Creates new `TransactionsPoolClient`.
	pub fn new(executor: Executor) -> Self {
		let transactions_pool_subscribers = Arc::new(RwLock::new(Subscribers::default()));
		let handler = Arc::new(
			TransactionsNotificationHandler::new(
				executor,
				transactions_pool_subscribers.clone(),
			)
		);

		TransactionsPoolClient {
			handler,
			transactions_pool_subscribers,
		}
	}

	pub fn run(&self, pool_receiver: mpsc::Receiver<(H256, TxStatus)>) {
		let handler = self.handler.clone();
		thread::spawn(move || loop {
			let res = pool_receiver.recv();
			if let Ok(res) = res {
				handler.notify_transaction(res)
			}
		});
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

	fn notify(executor: &Executor, subscriber: &Client, result: pubsub::Result) {
		executor.spawn(subscriber
			.notify(Ok(result))
			.map(|_| ())
			.map_err(|e| warn!(target: "rpc", "Unable to send notification: {}", e))
		);
	}

	pub fn notify_transaction(&self, status: (H256, TxStatus)) {
		for subscriber in self.transactions_pool_subscribers.read().values() {
			Self::notify(&self.executor, subscriber, pubsub::Result::TransactionStatus(status.clone()));
		}
	}
}

impl TransactionsPool for TransactionsPoolClient {
	type Metadata = Metadata;

	fn subscribe(&self, _meta: Metadata, subscriber: Subscriber<pubsub::Result>) {
		self.transactions_pool_subscribers.write().push(subscriber);
	}

	fn unsubscribe(&self, _meta: Option<Metadata>, id: SubscriptionId) -> Result<bool> {
		let res = self.transactions_pool_subscribers.write().remove(&id).is_some();
		Ok(res)
	}
}
