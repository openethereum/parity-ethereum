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

//! Parity-specific PUB-SUB rpc interface.

use jsonrpc_core::{Error, Value, Params};
use jsonrpc_pubsub::SubscriptionId;
use jsonrpc_macros::Trailing;
use jsonrpc_macros::pubsub::Subscriber;

build_rpc_trait! {
	/// Parity-specific PUB-SUB rpc interface.
	pub trait PubSub {
		type Metadata;

		#[pubsub(name = "parity_subscription")] {
			/// Subscribe to changes of any RPC method in Parity.
			#[rpc(name = "parity_subscribe")]
			fn parity_subscribe(&self, Self::Metadata, Subscriber<Value>, String, Trailing<Params>);

			/// Unsubscribe from existing Parity subscription.
			#[rpc(name = "parity_unsubscribe")]
			fn parity_unsubscribe(&self, SubscriptionId) -> Result<bool, Error>;
		}
	}
}
