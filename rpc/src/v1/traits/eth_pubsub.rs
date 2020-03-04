// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Eth PUB-SUB rpc interface.

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use jsonrpc_pubsub::{typed, SubscriptionId};

use v1::types::pubsub;

/// Eth PUB-SUB rpc interface.
#[rpc(server)]
pub trait EthPubSub {
	/// RPC Metadata
	type Metadata;

	/// Subscribe to Eth subscription.
	#[pubsub(subscription = "eth_subscription", subscribe, name = "eth_subscribe")]
	fn subscribe(
		&self,
		_: Self::Metadata,
		_: typed::Subscriber<pubsub::Result>,
		_: pubsub::Kind,
		_: Option<pubsub::Params>,
	);

	/// Unsubscribe from existing Eth subscription.
	#[pubsub(subscription = "eth_subscription", unsubscribe, name = "eth_unsubscribe")]
	fn unsubscribe(&self, _: Option<Self::Metadata>, _: SubscriptionId) -> Result<bool>;
}
