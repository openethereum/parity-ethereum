use std::sync::{Arc, Weak};
use std::collections::HashMap;

use jsonrpc_core::{BoxFuture, Result, Error};
use jsonrpc_core::futures::{self, Future, IntoFuture};
use jsonrpc_pubsub::{SubscriptionId, typed::{Sink, Subscriber}};

use v1::helpers::{errors, Subscribers};
use v1::metadata::Metadata;
use v1::traits::TransactionsPool;
use v1::types::{pubsub, RichHeader, Log};

use parity_runtime::Executor;
use parking_lot::{RwLock, Mutex};
use std::fs::read;
use ethereum_types::H256;
use futures::future::err;

type Client = Sink<pubsub::Result>;

pub struct TransactionsPoolClient<C> {
    handler: Arc<TransactionsNotificationHandler<C>>,
    transactions_pool_subscribers: Arc<RwLock<Subscribers<Client>>>,
}

impl<C> TransactionsPoolClient<C> {
    /// Creates new `TransactionsPoolClient`.
    pub fn new(client: Arc<C>, executor: Executor) -> Self {
        let transactions_pool_subscribers = Arc::new(RwLock::new(Subscribers::default()));

        TransactionsPoolClient {
            handler: Arc::new(TransactionsNotificationHandler {
                client,
                executor,
                transactions_pool_subscribers: transactions_pool_subscribers.clone(),
            }),
            transactions_pool_subscribers,
        }
    }

    /// Returns a transactions notification handler.
    pub fn handler(&self) -> Weak<TransactionsNotificationHandler<C>> {
        Arc::downgrade(&self.handler)
    }
}

pub struct TransactionsNotificationHandler<C> {
    client: Arc<C>,
    executor: Executor,
    transactions_pool_subscribers: Arc<RwLock<Subscribers<Client>>>,
}

impl<C> TransactionsNotificationHandler<C> {
    fn notify(executor: &Executor, subscriber: &Client, result: pubsub::Result) {
        executor.spawn(subscriber
            .notify(Ok(result))
            .map(|_| ())
            .map_err(|e| warn!(target: "rpc", "Unable to send notification: {}", e))
        );
    }

    pub fn notify_transactions(&self, hash: HashMap<H256, String>) {
        for subscriber in self.transactions_pool_subscribers.read().values() {
            Self::notify(&self.executor, subscriber, pubsub::Result::TransactionsHashMap(hash.clone()));
        }
    }
}

impl <C: Send + Sync + 'static> TransactionsPool for TransactionsPoolClient<C> {
    type Metadata = Metadata;

    fn subscribe(&self, _meta: Metadata, subscriber: Subscriber<pubsub::Result>, params: Option<pubsub::Params>) {
        let error = match params {
            None => {
                self.transactions_pool_subscribers.write().push(subscriber);
                return;
            },
            _ => {
                errors::invalid_params("parity_watchTransactionsPool", "Expected no parameters.")
            },
        };

        let _ = subscriber.reject(error);
    }

    fn unsubscribe(&self, _meta: Option<Metadata>, id: SubscriptionId) -> Result<bool> {
        let res = self.transactions_pool_subscribers.write().remove(&id).is_some();
        Ok(res)
    }
}