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

/// Implements EIP-2364 for using `fork_id` to check if the peer is compatible.

use v1::{EthPubSub, EthPubSubClient, Metadata};
use v1::types::pubsub::Kind;
use jsonrpc_pubsub::typed::Subscriber;
use futures::sync::mpsc;
use jsonrpc_pubsub::SubscriptionId;

/// Start subscribing new header.
pub fn subscribe_header<C: Send + Sync + 'static>(client: &EthPubSubClient<C>)
    -> (jsonrpc_pubsub::oneshot::Receiver<Result<SubscriptionId,
        jsonrpc_core::Error>>, mpsc::Receiver<String>) {
    let meta_data = Metadata::default();

    let (subscriber, id, subscription) = Subscriber::new_test("eip_2364");

    EthPubSubClient::subscribe(client, meta_data, subscriber, Kind::NewHeads, None);

    (id, subscription)
}
