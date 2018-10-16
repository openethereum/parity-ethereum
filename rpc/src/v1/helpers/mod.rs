// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

#[macro_use]
pub mod errors;

pub mod block_import;
pub mod dispatch;
pub mod fake_sign;
pub mod ipfs;
pub mod light_fetch;
pub mod nonce;
pub mod oneshot;
pub mod secretstore;

mod network_settings;
mod poll_filter;
mod poll_manager;
mod requests;
mod signer;
mod signing_queue;
mod subscribers;
mod subscription_manager;
mod work;

use std::sync::Arc;

use transaction::{PendingTransaction};

pub use self::dispatch::{Dispatcher, FullDispatcher, LightDispatcher};
pub use self::network_settings::NetworkSettings;
pub use self::poll_manager::PollManager;
pub use self::poll_filter::{PollFilter, SyncPollFilter, limit_logs};
pub use self::requests::{
	TransactionRequest, FilledTransactionRequest, ConfirmationRequest, ConfirmationPayload, CallRequest,
};
pub use self::signing_queue::{
	ConfirmationsQueue, ConfirmationReceiver, ConfirmationResult, ConfirmationSender,
	SigningQueue, QueueEvent, DefaultAccount,
	QUEUE_LIMIT as SIGNING_QUEUE_LIMIT,
};
pub use self::signer::SignerService;
pub use self::subscribers::Subscribers;
pub use self::subscription_manager::GenericPollManager;
pub use self::work::submit_work_detail;

pub fn to_url(address: &Option<::Host>) -> Option<String> {
	address.as_ref().map(|host| (**host).to_owned())
}

pub fn light_all_transactions(dispatch: &Arc<LightDispatcher>) -> impl Iterator<Item=PendingTransaction> {
	let txq = dispatch.transaction_queue.read();
	let chain_info = dispatch.client.chain_info();

	let current = txq.ready_transactions(chain_info.best_block_number, chain_info.best_block_timestamp);
	let future = txq.future_transactions(chain_info.best_block_number, chain_info.best_block_timestamp);
	current.into_iter().chain(future.into_iter())
}
