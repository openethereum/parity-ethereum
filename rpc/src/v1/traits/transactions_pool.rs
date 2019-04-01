use jsonrpc_core::Result;
use jsonrpc_pubsub::{typed, SubscriptionId};
use jsonrpc_derive::rpc;

use v1::types::pubsub;

/// Transactions Pool PUB-SUB rpc interface.
#[rpc]
pub trait TransactionsPool {
    /// Pub/Sub Metadata
    type Metadata;

    /// Subscribe to Transactions Pool subscription.
    #[pubsub(subscription = "parity_watchTransactionsPool", subscribe, name = "parity_watchTransactionsPool")]
    fn subscribe(&self, Self::Metadata, typed::Subscriber<pubsub::Result>, Option<pubsub::Params>);

    /// Unsubscribe from existing Transactions Pool subscription.
    #[pubsub(subscription = "parity_watchTransactionsPool", unsubscribe, name = "parity_unwatchTransactionsPool")]
    fn unsubscribe(&self, Option<Self::Metadata>, SubscriptionId) -> Result<bool>;
}