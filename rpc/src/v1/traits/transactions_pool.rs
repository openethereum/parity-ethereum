use jsonrpc_core::Result;
use jsonrpc_pubsub::{typed, SubscriptionId};
use jsonrpc_derive::rpc;

use v1::types::pubsub;

#[rpc]
pub trait TransactionsPool {
    /// Pub/Sub Metadata
    type Metadata;

    #[pubsub(subscription = "parity_watchTransactionsPool", subscribe, name = "parity_watchTransactionsPool")]
    fn subscribe(&self, Self::Metadata, typed::Subscriber<pubsub::Result>, Option<pubsub::Params>);

    #[pubsub(subscription = "parity_watchTransactionsPool", unsubscribe, name = "parity_unwatchTransactionsPool")]
    fn unsubscribe(&self, Option<Self::Metadata>, SubscriptionId) -> Result<bool>;
}