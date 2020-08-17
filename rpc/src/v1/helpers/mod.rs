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

#[macro_use]
pub mod errors;

pub mod block_import;
pub mod deprecated;
pub mod dispatch;
#[cfg(any(test, feature = "accounts"))]
pub mod eip191;
#[cfg(any(test, feature = "accounts"))]
pub mod engine_signer;
pub mod external_signer;
pub mod fake_sign;
pub mod nonce;
#[cfg(any(test, feature = "accounts"))]
pub mod secretstore;

mod network_settings;
mod poll_filter;
mod poll_manager;
mod requests;
mod signature;
mod subscribers;
mod subscription_manager;
mod work;

pub use self::{
    dispatch::{Dispatcher, FullDispatcher},
    network_settings::NetworkSettings,
    poll_filter::{limit_logs, PollFilter, SyncPollFilter},
    poll_manager::PollManager,
    requests::{
        CallRequest, ConfirmationPayload, ConfirmationRequest, FilledTransactionRequest,
        TransactionRequest,
    },
    signature::verify_signature,
    subscribers::Subscribers,
    subscription_manager::GenericPollManager,
    work::submit_work_detail,
};

pub fn to_url(address: &Option<::Host>) -> Option<String> {
    address.as_ref().map(|host| (**host).to_owned())
}
