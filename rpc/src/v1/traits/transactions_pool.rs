// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Transactions pool PUB-SUB rpc interface.

use jsonrpc_core::Result;
use jsonrpc_pubsub::{typed, SubscriptionId};
use jsonrpc_derive::rpc;
use miner::pool::TxStatus;

use ethereum_types::H256;

/// Transactions Pool PUB-SUB rpc interface.
#[rpc(server)]
pub trait TransactionsPool {
	/// Pub/Sub Metadata
	type Metadata;

	/// Subscribe to Transactions Pool subscription.
	#[pubsub(subscription = "parity_watchTransactionsPool", subscribe, name = "parity_watchTransactionsPool")]
	fn subscribe(&self, _: Self::Metadata, _: typed::Subscriber<(H256, TxStatus)>);

	/// Unsubscribe from existing Transactions Pool subscription.
	#[pubsub(subscription = "parity_watchTransactionsPool", unsubscribe, name = "parity_unwatchTransactionsPool")]
	fn unsubscribe(&self, _: Option<Self::Metadata>, _: SubscriptionId) -> Result<bool>;
}
