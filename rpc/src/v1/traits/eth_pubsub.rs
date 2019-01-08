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

//! Eth PUB-SUB rpc interface.

use jsonrpc_core::Result;
use jsonrpc_macros::Trailing;
use jsonrpc_macros::pubsub::Subscriber;
use jsonrpc_pubsub::SubscriptionId;

use v1::types::pubsub;

build_rpc_trait! {
	/// Eth PUB-SUB rpc interface.
	pub trait EthPubSub {
		type Metadata;

		#[pubsub(name = "eth_subscription")] {
			/// Subscribe to Eth subscription.
			#[rpc(name = "eth_subscribe")]
			fn subscribe(&self, Self::Metadata, Subscriber<pubsub::Result>, pubsub::Kind, Trailing<pubsub::Params>);

			/// Unsubscribe from existing Eth subscription.
			#[rpc(name = "eth_unsubscribe")]
			fn unsubscribe(&self, SubscriptionId) -> Result<bool>;
		}
	}
}
